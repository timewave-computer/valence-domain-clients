// This file defines a generic Cosmos client structure to be shared by specific chain clients.
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, Endpoint};
use crate::cosmos::grpc_client::GrpcSigningClient;
use prost::Message;
use hex;
use chrono::{Utc, TimeZone};

// Cosmos SDK proto imports
use cosmos_sdk_proto::cosmos::auth::v1beta1::{QueryModuleAccountByNameRequest, QueryModuleAccountByNameResponse};
use cosmos_sdk_proto::cosmos::auth::v1beta1::{ModuleAccount as ProtoModuleAccount, BaseAccount as ProtoBaseAccount};
use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient as AuthQueryClient;
use cosmos_sdk_proto::cosmos::bank::v1beta1::{QueryBalanceRequest, QueryBalanceResponse};
use cosmos_sdk_proto::cosmos::bank::v1beta1::query_client::QueryClient as BankQueryClient;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{GetTxRequest, GetTxResponse, SimulateRequest};
use cosmos_sdk_proto::cosmos::tx::v1beta1::{BroadcastTxRequest as ProtoBroadcastTxRequest, BroadcastMode as ProtoBroadcastMode};
use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient as TxServiceClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{Fee as ProtoFee};
use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo as ProtoAuthInfo, SignerInfo as ProtoSignerInfo, Tx as ProtoTx, TxBody as ProtoTxBody};
use cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::{self, Sum};
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::{GetBlockByHeightRequest, GetLatestBlockRequest};
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient as TendermintServiceClient;
use cosmos_sdk_proto::Any as CosmosSdkProtoAny;

// Cosmrs imports
use cosmrs::tx::Msg;
use cosmrs::tx::{Body, BodyBuilder, SignMode};
use cosmrs::tendermint::block::Height as CosmrsHeight;
use cosmrs::{AccountId, Coin as CosmrsCoin, Denom as CosmrsDenom, Any as CosmrsAny};

use crate::{
    core::types::GenericAddress,
    core::{
        error::ClientError,
        transaction::{
            self, convert_from_proto_tx_response, convert_proto_events,
            TransactionResponse,
        },
    },
    cosmos::{
        signing_client::SigningClient,
        types::{
            CosmosAccount, CosmosAccountPubKey, CosmosBaseAccount,
            CosmosBlockResults, CosmosFee,
            CosmosCoin as CosmosTypesCoin, CosmosConsensusVersion,
            CosmosGasInfo, CosmosHeader, CosmosModuleAccount,
            CosmosSimulateResponse, CosmosSimulateResult,
        },
    },
};

// Constants for poll_for_tx
const DEFAULT_POLL_MAX_ATTEMPTS: u32 = 60;
const DEFAULT_POLL_INTERVAL_SECONDS: u64 = 1;

#[derive(Clone, Debug)]
pub struct CosmosClientConfig {
    pub grpc_url: String,
    pub chain_id_str: String,
    pub chain_prefix: String,
    pub chain_denom: String,
    pub gas_price: f64,
    pub gas_adjustment: f64,
    // pub chain_name: String, // Optional: if needed for more than logging
}

pub struct GenericCosmosClient {
    pub config: CosmosClientConfig,
    pub channel: Channel,
    pub(crate) signing_client: Arc<Mutex<SigningClient>>,
    // We can add pre-initialized service clients here if they are commonly used
    // pub tendermint_service: Arc<Mutex<TendermintServiceClient<Channel>>>,
    // pub tx_service: Arc<Mutex<TxServiceClient<Channel>>>,
}

// Implement GrpcSigningClient trait for GenericCosmosClient
#[async_trait::async_trait]
impl GrpcSigningClient for GenericCosmosClient {
    fn grpc_url(&self) -> String {
        self.config.grpc_url.clone()
    }

    fn chain_prefix(&self) -> String {
        self.config.chain_prefix.clone()
    }

    fn chain_id_str(&self) -> String {
        self.config.chain_id_str.clone()
    }

    fn chain_denom(&self) -> String {
        self.config.chain_denom.clone()
    }

    fn gas_price(&self) -> f64 {
        self.config.gas_price
    }

    fn gas_adjustment(&self) -> f64 {
        self.config.gas_adjustment
    }

    async fn get_signer_details(&self) -> Result<CosmosAccount, ClientError> {
        // Already implemented method, delegate to it
        self.get_signer_details().await
    }

    fn get_tx_fee(
        &self,
        simulation_response: CosmosSimulateResponse,
    ) -> Result<CosmosFee, ClientError> {
        // Use the simulation response to calculate fee
        let gas_used = simulation_response.gas_info.gas_used;
        let gas_limit = (gas_used as f64 * self.config.gas_adjustment).ceil() as u64;
        
        let amount = self.config.gas_price * (gas_limit as f64);
        
        Ok(CosmosFee {
            amount: vec![CosmosTypesCoin {
                denom: self.config.chain_denom.clone(),
                amount: amount as u128,
            }],
            gas_limit,
            payer: None,
            granter: None,
        })
    }

    async fn simulate_tx(
        &self,
        msg: CosmrsAny,
    ) -> Result<CosmosSimulateResponse, ClientError> {
        // Already implemented method, delegate to it
        self.simulate_tx(msg).await
    }

    async fn query_chain_gas_config(
        &self,
        _chain_name: &str,
        _denom: &str,
    ) -> Result<f64, ClientError> {
        // Simple implementation, just return the configured gas price
        // In a real implementation, this might query an external source
        Ok(self.config.gas_price)
    }

    async fn sign_and_broadcast_tx(
        &self,
        msg: CosmrsAny,
        fee: CosmosFee,
        memo: Option<&str>,
    ) -> Result<crate::core::transaction::TransactionResponse, ClientError> {
        // Already implemented method, delegate to it
        self.sign_and_broadcast_tx(msg, fee, memo).await
    }
}

// We're implementing a minimal version of the CosmosBaseClient to get the code building
#[async_trait::async_trait]
impl crate::cosmos::base_client::CosmosBaseClient for GenericCosmosClient {
    // Minimal implementations just to get the code building
    async fn transfer(&self, _to_address_str: &str, _amount: u128, _denom: &str, _memo: Option<&str>) -> Result<TransactionResponse, ClientError> {
        Err(ClientError::NotImplemented("transfer".to_string()))
    }

    async fn latest_block_header(&self) -> Result<CosmosHeader, ClientError> {
        Err(ClientError::NotImplemented("latest_block_header".to_string()))
    }

    async fn block_results(&self, _height: u64) -> Result<CosmosBlockResults, ClientError> {
        Err(ClientError::NotImplemented("block_results".to_string()))
    }

    async fn query_balance(&self, _address: &str, _denom: &str) -> Result<u128, ClientError> {
        Err(ClientError::NotImplemented("query_balance".to_string()))
    }

    async fn query_module_account(&self, _name: &str) -> Result<CosmosModuleAccount, ClientError> {
        Err(ClientError::NotImplemented("query_module_account".to_string()))
    }

    async fn poll_for_tx(&self, _tx_hash: &str) -> Result<TransactionResponse, ClientError> {
        Err(ClientError::NotImplemented("poll_for_tx".to_string()))
    }

    async fn poll_until_expected_balance(&self, _address: &str, _denom: &str, _min_amount: u128, _interval_sec: u64, _max_attempts: u32) -> Result<u128, ClientError> {
        Err(ClientError::NotImplemented("poll_until_expected_balance".to_string()))
    }

    async fn ibc_transfer(
        &self,
        to_address: String,
        denom: String,
        amount: String,
        source_channel: String,
        timeout_seconds: u64,
        memo: Option<String>,
    ) -> Result<TransactionResponse, ClientError> {
        // Create the IBC transfer message
        let sc_details = self.get_signer_details().await?;
        let from_address = sc_details.address.0.clone();

        // Parse amount - IBC uses string amounts
        let amount_u128 = amount.parse::<u128>().map_err(|e| {
            ClientError::ParseError(format!("Failed to parse amount '{amount}': {e}"))
        })?;

        // Create cosmrs::Coin for the amount
        let denom_parsed: CosmrsDenom = denom.parse().map_err(|e| {
            ClientError::ParseError(format!("Invalid denom '{denom}': {e}"))
        })?;

        // Calculate timeout timestamp (current time + timeout_seconds)
        let timeout_timestamp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(timeout_seconds as i64))
            .ok_or_else(|| {
                ClientError::ParseError("Failed to calculate timeout timestamp".to_string())
            })?
            .timestamp_nanos_opt()
            .unwrap_or(0);

        // Create token structure
        let token = cosmrs::Coin {
            denom: denom_parsed, 
            amount: amount_u128,
        };
        
        // Manually construct the IBC transfer message as JSON with our own custom implementation
        let msg_bytes = serde_json::json!({
            "source_port": "transfer",
            "source_channel": source_channel,
            "token": {
                "denom": token.denom.to_string(),
                "amount": token.amount.to_string()
            },
            "sender": from_address,
            "receiver": to_address,
            "timeout_timestamp": timeout_timestamp.to_string()
        }).to_string().into_bytes();
        
        // Create the Any message for IBC transfer
        let any_msg = CosmrsAny {
            type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
            value: msg_bytes,
        };

        // Estimate fee and broadcast
        let simulated = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(simulated)?;

        // Sign and broadcast
        self.sign_and_broadcast_tx(any_msg, fee, memo.as_deref()).await
    }
}

impl GenericCosmosClient {
    pub async fn new(
        config: CosmosClientConfig,
        mnemonic_str: &str,
    ) -> Result<Self, ClientError> {
        let endpoint =
            Endpoint::from_shared(config.grpc_url.clone()).map_err(|e| {
                ClientError::ClientError(format!(
                    "Invalid gRPC URL {}: {}",
                    config.grpc_url, e
                ))
            })?;
        let channel = endpoint.connect().await.map_err(|e| {
            ClientError::ClientError(format!(
                "Failed to connect to gRPC endpoint {}: {}",
                config.grpc_url, e
            ))
        })?;

        let sc = SigningClient::from_mnemonic(
            channel.clone(),
            mnemonic_str,
            &config.chain_prefix,
            &config.chain_id_str,
        )
        .await?;

        Ok(Self {
            config,
            channel,
            signing_client: Arc::new(Mutex::new(sc)),
        })
    }

    pub async fn get_signer_details(
        &self,
    ) -> Result<CosmosAccount, ClientError> {
        let sc = self.signing_client.lock().await;
        let address_str = sc.address.to_string();
        Ok(CosmosAccount {
            address: GenericAddress(address_str),
            account_number: sc.account_number,
            sequence: sc.sequence,
            pub_key: Some(sc.public_key.to_bytes()),
        })
    }

    pub fn get_tx_fee(
        &self,
        simulation_response: CosmosSimulateResponse,
    ) -> Result<CosmosFee, ClientError> {
        let gas_limit_adjusted = (simulation_response.gas_info.gas_used as f64
            * self.config.gas_adjustment)
            .round() as u64;
        let fee_amount_calculated =
            (gas_limit_adjusted as f64 * self.config.gas_price).round() as u128;

        if fee_amount_calculated == 0
            && gas_limit_adjusted > 0
            && self.config.gas_price > 0.0
        {
            Ok(CosmosFee {
                amount: vec![CosmosTypesCoin {
                    denom: self.config.chain_denom.clone(),
                    amount: 1, // Min fee if calculated is 0 but gas is used
                }],
                gas_limit: gas_limit_adjusted,
                payer: None,
                granter: None,
            })
        } else {
            Ok(CosmosFee {
                amount: vec![CosmosTypesCoin {
                    denom: self.config.chain_denom.clone(),
                    amount: fee_amount_calculated,
                }],
                gas_limit: gas_limit_adjusted,
                payer: None,
                granter: None,
            })
        }
    }

    pub async fn simulate_tx(
        &self,
        msg: CosmrsAny,
    ) -> Result<CosmosSimulateResponse, ClientError> {
        let sc = self.signing_client.lock().await;
        let chain_denom_parsed: CosmrsDenom =
            self.config.chain_denom.parse().map_err(|e| {
                ClientError::ClientError(format!(
                    "Failed to parse chain_denom \'{}\' for simulation fee: {}",
                    self.config.chain_denom, e
                ))
            })?;

        let sim_gas_limit = 300_000u64; // Consider making this configurable
        let sim_fee_amount_cosmrs: Vec<CosmrsCoin> = vec![CosmrsCoin {
            denom: chain_denom_parsed,
            amount: 0u128, // Fee is typically 0 for simulation
        }];
        let sim_memo = "".to_string(); // Memo is usually empty for simulation

        let cosmrs_tx_body: Body = BodyBuilder::new()
            .msg(msg.clone())
            .memo(sim_memo)
            .timeout_height(CosmrsHeight::from(0u32))
            .finish();

        let proto_tx_body = ProtoTxBody {
            messages: cosmrs_tx_body.messages.clone(),
            memo: cosmrs_tx_body.memo,
            timeout_height: cosmrs_tx_body.timeout_height.value(),
            extension_options: cosmrs_tx_body.extension_options.clone(),
            non_critical_extension_options: cosmrs_tx_body
                .non_critical_extension_options
                .clone(),
        };

        let pk_sdk_any = CosmosSdkProtoAny {
            type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
            value: sc.public_key.to_bytes(),
        };

        // Create ModeInfo with Single signing mode
        let mode_info = cosmos_sdk_proto::cosmos::tx::v1beta1::ModeInfo {
            sum: Some(Sum::Single(mode_info::Single {
                mode: SignMode::Direct.into(),
            })),
        };

        let single_signer_info = ProtoSignerInfo {
            public_key: Some(pk_sdk_any),
            mode_info: Some(mode_info),
            sequence: sc.sequence,
        };

        let proto_auth_info = ProtoAuthInfo {
            signer_infos: vec![single_signer_info],
            fee: Some(ProtoFee {
                amount: sim_fee_amount_cosmrs
                    .iter()
                    .map(|c| ProtoCoin {
                        denom: c.denom.to_string(),
                        amount: c.amount.to_string(),
                    })
                    .collect(),
                gas_limit: sim_gas_limit,
                payer: sc.address.to_string(),
                granter: "".to_string(),
            }),
            tip: None,
        };

        let tx_for_sim = ProtoTx {
            body: Some(proto_tx_body),
            auth_info: Some(proto_auth_info),
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx_for_sim.encode(&mut tx_bytes).map_err(|e| {
            ClientError::ClientError(format!(
                "Failed to encode ProtoTx for simulation: {e}"
            ))
        })?;

        let mut tx_service_client =
            TxServiceClient::connect(self.config.grpc_url.clone())
                .await
                .map_err(|e| {
                    ClientError::ClientError(format!(
                        "Failed to connect to gRPC for simulation: {e}"
                    ))
                })?;

        // Using tx_bytes directly as the tx field is deprecated
        let sim_req = SimulateRequest { tx_bytes, ..Default::default() };

        let sim_res_inner = tx_service_client
            .simulate(sim_req)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Failed to simulate tx: {e}"))
            })?
            .into_inner();

        Ok(CosmosSimulateResponse {
            simulate_result: sim_res_inner
                .result
                .map(|res| CosmosSimulateResult {
                    // Safely handle data which is a deprecated field
                    data: if res.events.is_empty() {
                        None
                    } else {
                        Some(Vec::new()) // Empty data since we can't rely on the deprecated field
                    },
                    log: res.log,
                    events: {
                        // Convert events directly using the event conversion function
                        // without using the EventWrapper, as the types don't match correctly
                        let converted_events = res.events
                            .iter()
                            .map(|evt| {
                                // Create cosmos_sdk_proto event with same attributes
                                let mut cosmos_evt = cosmos_sdk_proto::tendermint::abci::Event {
                                    r#type: evt.r#type.clone(),
                                    attributes: Vec::new(),
                                };
                                
                                // Convert each attribute
                                for attr in &evt.attributes {
                                    cosmos_evt.attributes.push(cosmos_sdk_proto::tendermint::abci::EventAttribute {
                                        // Convert Bytes to String
                                        key: String::from_utf8_lossy(&attr.key).to_string(),
                                        value: String::from_utf8_lossy(&attr.value).to_string(),
                                        index: attr.index,
                                    });
                                }
                                
                                cosmos_evt
                            })
                            .collect::<Vec<_>>();
                        
                        // Finally convert the proto events
                        convert_proto_events(converted_events)
                    },
                })
                .ok_or_else(|| {
                    ClientError::ParseError(
                        "Result missing from simulate response".to_string(),
                    )
                })?,
            gas_info: sim_res_inner
                .gas_info
                .map(|gi| CosmosGasInfo {
                    gas_wanted: gi.gas_wanted,
                    gas_used: gi.gas_used,
                })
                .ok_or_else(|| {
                    ClientError::ParseError(
                        "GasInfo missing from simulate response".to_string(),
                    )
                })?,
        })
    }

    pub async fn sign_and_broadcast_tx(
        &self,
        msg: CosmrsAny,
        fee: CosmosFee,
        memo: Option<&str>,
    ) -> Result<TransactionResponse, ClientError> {
        // Connect to the gRPC service for broadcasting transactions
        let mut tx_service_client =
            TxServiceClient::connect(self.config.grpc_url.clone())
                .await
                .map_err(|e| {
                    ClientError::ClientError(format!(
                        "Failed to connect to gRPC for broadcast: {e}"
                    ))
                })?;
        
        // Acquire lock on the signing client to perform signing
        let mut sc = self.signing_client.lock().await;

        // Parse the chain default denom for fee conversion
        let chain_default_denom_str = self.config.chain_denom.clone();
        let chain_default_denom: CosmrsDenom =
            chain_default_denom_str.parse().map_err(|e| {
                ClientError::ParseError(format!(
                    "Failed to parse chain_default_denom '{chain_default_denom_str}': {e}"
                ))
            })?;

        // Create the transaction
        let our_tx_bytes = sc.sign_tx(
            msg,
            fee.clone(),
            memo,
            &chain_default_denom
        ).await?;
        
        // Use Sync broadcast mode as the default
        let proto_mode = ProtoBroadcastMode::Sync;

        // Create the protobuf request
        let grpc_broadcast_req = ProtoBroadcastTxRequest {
            tx_bytes: our_tx_bytes,
            mode: proto_mode.into(),
        };

        // Update the sequence for future transactions
        sc.sequence += 1;

        // Broadcast the transaction
        let broadcast_res = tx_service_client
            .broadcast_tx(grpc_broadcast_req)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!(
                    "Failed to broadcast tx: {e}"
                ))
            })?
            .into_inner();

        if let Some(tx_response_proto) = broadcast_res.tx_response {
            convert_from_proto_tx_response(tx_response_proto)
        } else {
            Err(ClientError::ClientError(
                "Broadcast successful but no TxResponse received".to_string(),
            ))
        }
    }

    pub async fn transfer(
        &self,
        to_address_str: &str,
        amount_u128: u128,
        denom_str: &str,
        memo: Option<&str>,
    ) -> Result<TransactionResponse, ClientError> {
        let sc_details = self.get_signer_details().await?;
        let from_address_str = sc_details.address.0;

        let from_account_id: AccountId = from_address_str.parse().map_err(|e| {
            ClientError::ParseError(format!(
                "Failed to parse from_address (signer) '{from_address_str}': {e}"
            ))
        })?;

        let to_account_id: AccountId = to_address_str.parse().map_err(|e| {
            ClientError::ParseError(format!(
                "Failed to parse to_address '{to_address_str}': {e}"
            ))
        })?;

        let denom_parsed: CosmrsDenom = denom_str.parse().map_err(|e| {
            ClientError::ParseError(format!(
                "Invalid denom '{denom_str}': {e}"
            ))
        })?;

        let cosmrs_coin_amount = CosmrsCoin {
            denom: denom_parsed,
            amount: amount_u128,
        };

        let msg_send = cosmrs::bank::MsgSend {
            from_address: from_account_id,
            to_address: to_account_id,
            amount: vec![cosmrs_coin_amount],
        };

        let any_msg = msg_send.to_any().map_err(|e| {
            ClientError::ClientError(format!(
                "Failed to convert MsgSend to Any: {e}"
            ))
        })?;

        // Simulate, get fee, sign and broadcast
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee_details = self.get_tx_fee(sim_response)?;
        self.sign_and_broadcast_tx(any_msg, fee_details, memo).await
    }

    pub async fn latest_block_header(
        &self,
    ) -> Result<CosmosHeader, ClientError> {
        let mut tendermint_service = TendermintServiceClient::connect(self.config.grpc_url.clone())
            .await
            .map_err(|e| {
                ClientError::ClientError(format!(
                    "Failed to connect to Tendermint service for latest_block_header: {e}"
                ))
            })?;
        let request = tonic::Request::new(GetLatestBlockRequest {});

        let response =
            tendermint_service
                .get_latest_block(request)
                .await
                .map_err(|e| {
                    ClientError::QueryError(format!(
                        "Failed to get latest block: {e}"
                    ))
                })?;

        let proto_block_outer = response.into_inner();

        let proto_header = proto_block_outer
            .block
            .and_then(|b| b.header)
            .ok_or_else(|| {
                ClientError::QueryError(
                    "No header found in latest block response".to_string(),
                )
            })?;

        let cosmos_version = proto_header.version.map(|v| CosmosConsensusVersion {
            block: v.block,
            app: v.app,
        });

        // Extract timestamp fields directly
        let formatted_time = proto_header.time.as_ref().map(|ts| {
            Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
                .single()
                .map_or_else(
                    || "Invalid timestamp".to_string(),
                    |dt| dt.to_rfc3339(),
                )
        });

        let app_hash_hex = if proto_header.app_hash.is_empty() {
            None
        } else {
            Some(hex::encode(proto_header.app_hash))
        };

        let proposer_address_str = if proto_header.proposer_address.is_empty() {
            None
        } else {
            Some(GenericAddress(hex::encode(proto_header.proposer_address)))
        };

        Ok(CosmosHeader {
            version: cosmos_version,
            chain_id: proto_header.chain_id,
            height: proto_header.height,
            time: formatted_time,
            app_hash: app_hash_hex,
            proposer_address: proposer_address_str,
        })
    }

    pub async fn block_results(
        &self,
        height: u64,
    ) -> Result<CosmosBlockResults, ClientError> {
        let mut tendermint_service =
            TendermintServiceClient::connect(self.config.grpc_url.clone())
                .await
                .map_err(|e| {
                    ClientError::ClientError(format!(
                    "Failed to connect to Tendermint service for block_results: {e}"
                ))
                })?;

        let request = tonic::Request::new(GetBlockByHeightRequest {
            height: height as i64, // Proto expects i64
        });

        let response = tendermint_service
            .get_block_by_height(request)
            .await
            .map_err(|e| {
                ClientError::QueryError(format!(
                    "Failed to get block by height {height}: {e}"
                ))
            })?;

        let proto_block_opt = response.into_inner().block;

        if let Some(proto_block) = proto_block_opt {
            Ok(CosmosBlockResults {
                height: proto_block.header.as_ref().map_or(0, |h| h.height as u64),
                txs_results: None, // Placeholder, as in BabylonClient
                begin_block_events: None, // Placeholder
                end_block_events: None, // Placeholder
                validator_updates: vec![], // Placeholder
                consensus_param_updates: None, // Placeholder
            })
        } else {
            Err(ClientError::QueryError(format!(
                "Block {height} not found or empty response"
            )))
        }
    }

    pub async fn query_balance(
        &self,
        address: &str,
        denom: &str,
    ) -> Result<u128, ClientError> {
        let mut bank_query_client =
            BankQueryClient::connect(self.config.grpc_url.clone())
                .await
                .map_err(|e| {
                    ClientError::ClientError(format!(
                    "Failed to connect to BankQuery service for query_balance: {e}"
                ))
                })?;

        let request = tonic::Request::new(QueryBalanceRequest {
            address: address.to_string(),
            denom: denom.to_string(),
        });

        let response = bank_query_client.balance(request).await.map_err(|e| {
            ClientError::QueryError(format!("Failed to query balance: {e}"))
        })?;

        let balance_response: QueryBalanceResponse = response.into_inner();

        if let Some(coin) = balance_response.balance {
            if coin.denom == denom {
                coin.amount.parse::<u128>().map_err(|e| {
                    ClientError::ParseError(format!(
                        "Failed to parse amount '{}' for denom '{}': {}",
                        coin.amount, denom, e
                    ))
                })
            } else {
                Err(ClientError::QueryError(format!(
                    "Query for denom '{}' returned balance for denom '{}'",
                    denom, coin.denom
                )))
            }
        } else {
            Ok(0) // Return 0 if no balance is found for the denom
        }
    }

    pub async fn query_module_account(
        &self,
        name: &str,
    ) -> Result<CosmosModuleAccount, ClientError> {
        let mut auth_query_client = AuthQueryClient::connect(self.config.grpc_url.clone())
            .await
            .map_err(|e| {
                ClientError::ClientError(format!(
                    "Failed to connect to AuthQuery service for query_module_account: {e}"
                ))
            })?;

        let request = tonic::Request::new(QueryModuleAccountByNameRequest {
            name: name.to_string(),
        });

        let response = auth_query_client
            .module_account_by_name(request)
            .await
            .map_err(|e| {
                ClientError::QueryError(format!(
                    "Failed to query module account by name '{name}': {e}"
                ))
            })?;

        let module_account_response: QueryModuleAccountByNameResponse =
            response.into_inner();

        if let Some(account_any) = module_account_response.account {
            // Attempt to decode as cosmos_sdk_proto::cosmos::auth::v1beta1::ModuleAccount first
            if account_any.type_url == "/cosmos.auth.v1beta1.ModuleAccount" {
                let ma: ProtoModuleAccount =
                    ProtoModuleAccount::decode(account_any.value.as_slice())
                        .map_err(|e| {
                            ClientError::ParseError(format!(
                                "Failed to decode ModuleAccount for '{name}': {e}"
                            ))
                        })?;

                let cosmos_base_account_opt = ma.base_account.map(|proto_ba| {
                    let pub_key_enum_opt = proto_ba.pub_key.map(|pk_any| {
                        if pk_any.type_url == "/cosmos.crypto.secp256k1.PubKey" {
                            CosmosAccountPubKey::Secp256k1(pk_any.value)
                        } else {
                            CosmosAccountPubKey::Unsupported {
                                type_url: pk_any.type_url,
                                value: pk_any.value,
                            }
                        }
                    });
                    CosmosBaseAccount {
                        // Constructing CosmosBaseAccount
                        address: proto_ba.address, // String, not GenericAddress
                        pub_key: pub_key_enum_opt, // Option<CosmosAccountPubKey>
                        account_number: proto_ba.account_number,
                        sequence: proto_ba.sequence,
                    }
                });

                Ok(CosmosModuleAccount {
                    name: ma.name,                         // String
                    base_account: cosmos_base_account_opt, // Option<CosmosBaseAccount>
                    permissions: ma.permissions,
                })
            } else if account_any.type_url == "/cosmos.auth.v1beta1.BaseAccount" {
                let ba: ProtoBaseAccount = ProtoBaseAccount::decode(
                    account_any.value.as_slice(),
                )
                .map_err(|e| {
                    ClientError::ParseError(format!(
                        "Failed to decode BaseAccount for '{}' (module {}): {}",
                        name, account_any.type_url, e
                    ))
                })?;

                let pub_key_enum_opt = ba.pub_key.map(|pk_any| {
                    if pk_any.type_url == "/cosmos.crypto.secp256k1.PubKey" {
                        CosmosAccountPubKey::Secp256k1(pk_any.value)
                    } else {
                        CosmosAccountPubKey::Unsupported {
                            type_url: pk_any.type_url,
                            value: pk_any.value,
                        }
                    }
                });
                let cosmos_base_account_from_ba = CosmosBaseAccount {
                    // Constructing CosmosBaseAccount
                    address: ba.address,       // String
                    pub_key: pub_key_enum_opt, // Option<CosmosAccountPubKey>
                    account_number: ba.account_number,
                    sequence: ba.sequence,
                };

                Ok(CosmosModuleAccount {
                    name: name.to_string(), // Queried name as String
                    base_account: Some(cosmos_base_account_from_ba), // Option<CosmosBaseAccount>
                    permissions: vec![], // BaseAccount doesn't have permissions list
                })
            } else {
                Err(ClientError::ParseError(format!(
                    "Unsupported account type '{}' for module account '{}'",
                    account_any.type_url, name
                )))
            }
        } else {
            Err(ClientError::QueryError(format!(
                "Module account '{name}' not found or empty response"
            )))
        }
    }

    pub async fn poll_for_tx(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionResponse, ClientError> {
        let mut attempts = 0;
        // Create a new TxServiceClient for each call or manage it if Tonic allows reuse across await points
        // For simplicity, creating it here. Ensure connect() is efficient or managed.
        let mut client = TxServiceClient::connect(self.config.grpc_url.clone())
            .await
            .map_err(|e| {
                ClientError::ClientError(format!(
                    "Failed to connect to gRPC for polling: {e}"
                ))
            })?;

        loop {
            if attempts >= DEFAULT_POLL_MAX_ATTEMPTS {
                return Err(ClientError::TimeoutError(format!(
                    "Timeout polling for tx_hash '{tx_hash}' after {DEFAULT_POLL_MAX_ATTEMPTS} attempts."
                )));
            }
            attempts += 1;

            let request = tonic::Request::new(GetTxRequest {
                hash: tx_hash.to_string(),
            });

            match client.get_tx(request).await {
                Ok(response) => {
                    let get_tx_response: GetTxResponse = response.into_inner();
                    if let Some(tx_response_proto) = get_tx_response.tx_response {
                        // Ensure convert_from_proto_tx_response is accessible
                        return transaction::convert_from_proto_tx_response(
                            tx_response_proto,
                        );
                    }
                    // If tx_response is None, it means the tx is found but not yet processed or still pending in a block
                    // Consider this a "try again" scenario.
                }
                Err(status) => {
                    if status.code() == tonic::Code::NotFound
                        || status.code() == tonic::Code::InvalidArgument
                    // Some nodes return InvalidArgument for not found tx
                    {
                        // Tx not found yet, continue polling
                    } else {
                        // Other gRPC or network error
                        return Err(ClientError::QueryError(format!(
                            "Failed to poll for tx_hash '{tx_hash}': {status}"
                        )));
                    }
                }
            }
            sleep(Duration::from_secs(DEFAULT_POLL_INTERVAL_SECONDS)).await;
        }
    }

    pub async fn poll_until_expected_balance(
        &self,
        address: &str,
        denom: &str,
        min_amount: u128,
        interval_sec: u64,
        max_attempts: u32,
    ) -> Result<u128, ClientError> {
        let mut attempts = 0;
        loop {
            if attempts >= max_attempts {
                return Err(ClientError::TimeoutError(format!(
                    "Timeout polling for balance >= {min_amount} {denom} for address '{address}' after {max_attempts} attempts."
                )));
            }
            attempts += 1;
            // This will call GenericCosmosClient::query_balance (self.query_balance)
            match self.query_balance(address, denom).await {
                Ok(current_balance) => {
                    if current_balance >= min_amount {
                        return Ok(current_balance);
                    }
                }
                Err(e) => {
                    // It might be better to log and continue polling for some errors,
                    // but for now, propagate the error immediately.
                    return Err(ClientError::QueryError(format!(
                        "Error during balance poll for address '{address}' (attempt {attempts}): {e}"
                    )));
                }
            }
            sleep(Duration::from_secs(interval_sec)).await;
        }
    }

    // Placeholder for GrpcSigningClient methods that will be moved here
    // Example:
    // pub fn grpc_url(&self) -> String {
    //     self.config.grpc_url.clone()
    // }
    // ... other methods ...
}
