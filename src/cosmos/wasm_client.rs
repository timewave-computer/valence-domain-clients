use std::{fs, path::Path, str::FromStr};

use async_trait::async_trait;
use cosmos_sdk_proto::cosmwasm::wasm::v1::{
    MsgInstantiateContract2, QueryBuildAddressRequest, QueryBuildAddressResponse, QueryCodeRequest,
    QueryCodeResponse,
};
use cosmrs::{cosmwasm::MsgInstantiateContract, tx::Fee, Any, Coin};
use prost::{Message, Name};
use serde::{de::DeserializeOwned, Serialize};

use crate::common::{error::StrategistError, transaction::TransactionResponse};
use tonic::Request;

use super::{grpc_client::GrpcSigningClient, CosmosServiceClient, WasmQueryClient};

use cosmrs::{
    cosmwasm::{MsgExecuteContract, MsgStoreCode},
    proto::cosmwasm::wasm::v1::QuerySmartContractStateRequest,
    tx::Msg,
    AccountId,
};

/// wasm funcionality trait with default implementations for cosmos-sdk based clients.
///
/// for chains which are somehow unique in their wasm module implementations,
/// these function definitions can be overridden to match that of the chain.
#[async_trait]
pub trait WasmClient: GrpcSigningClient {
    async fn upload_code(&self, wasm_path: &str) -> Result<u64, StrategistError> {
        // 1. load the wasm from storage
        let wasm_path = Path::new(wasm_path);
        if !wasm_path.exists() {
            return Err(StrategistError::ClientError(format!(
                "WASM file not found at path: {}",
                wasm_path.display()
            )));
        }

        let wasm_bytes = fs::read(wasm_path).map_err(|e| {
            StrategistError::ClientError(format!("Failed to read WASM file: {}", e))
        })?;

        // 2. upload the wasm to the chain
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let store_code_msg = MsgStoreCode {
            sender: signing_client.address.clone(),
            wasm_byte_code: wasm_bytes,
            instantiate_permission: None,
        }
        .to_any()?;

        let simulation_response = self.simulate_tx(store_code_msg.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client.create_tx(store_code_msg, fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        let tx_response = match &broadcast_tx_response.tx_response {
            Some(response) => response,
            None => {
                return Err(StrategistError::TransactionError(
                    "No transaction response returned".to_string(),
                ))
            }
        };

        // poll the node until txhash resolves to a response
        let query_tx_response = self.poll_for_tx(&tx_response.txhash).await?;

        // filter abci logs to find the resulting code id
        for abci_msg_log in query_tx_response.logs.iter() {
            for event in abci_msg_log.events.iter() {
                for attr in event.attributes.iter() {
                    if attr.key == "code_id" {
                        return attr.value.parse::<u64>().map_err(|_| {
                            StrategistError::ParseError("Failed to parse code_id".to_string())
                        });
                    }
                }
            }
        }

        Err(StrategistError::ParseError(format!(
            "Failed to find code_id in transaction response: {:?}",
            tx_response
        )))
    }

    async fn instantiate(
        &self,
        code_id: u64,
        label: Option<String>,
        msg: (impl Serialize + Send),
    ) -> Result<String, StrategistError> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let msg_bytes = serde_json::to_vec(&msg)?;

        let instantiate_tx = MsgInstantiateContract {
            sender: signing_client.address.clone(),
            admin: None,
            code_id,
            label,
            msg: msg_bytes,
            funds: vec![],
        }
        .to_any()?;

        let simulation_response = self.simulate_tx(instantiate_tx.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client.create_tx(instantiate_tx, fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        let tx_response = match &broadcast_tx_response.tx_response {
            Some(response) => response,
            None => {
                return Err(StrategistError::TransactionError(
                    "No transaction response returned".to_string(),
                ))
            }
        };

        // poll the node until txhash resolves to a response
        let query_tx_response = self.poll_for_tx(&tx_response.txhash).await?;

        // filter abci logs to find the contract address
        for abci_msg_log in query_tx_response.logs.iter() {
            for event in abci_msg_log.events.iter() {
                if event.r#type == "instantiate" || event.r#type == "instantiate_contract" {
                    for attr in event.attributes.iter() {
                        if attr.key == "_contract_address" || attr.key == "contract_address" {
                            return Ok(attr.value.clone());
                        }
                    }
                }
            }
        }

        Err(StrategistError::ParseError(format!(
            "Failed to find contract address in transaction response: {:?}",
            query_tx_response
        )))
    }

    async fn query_code_info(&self, code_id: u64) -> Result<QueryCodeResponse, StrategistError> {
        let channel = self.get_grpc_channel().await?;

        let mut grpc_client = WasmQueryClient::new(channel);

        let code_query_request = QueryCodeRequest { code_id };

        let code_query_response = grpc_client.code(code_query_request).await?.into_inner();

        Ok(code_query_response)
    }

    async fn predict_instantiate2_addr(
        &self,
        code_id: u64,
        salt: String,
        creator: String,
    ) -> Result<QueryBuildAddressResponse, StrategistError> {
        let code_info = self.query_code_info(code_id).await?.code_info;

        let code_id_hash = match code_info {
            Some(ci) => ci.data_hash,
            None => {
                return Err(StrategistError::TransactionError(
                    "failed to get code info from code query".to_string(),
                ))
            }
        };

        let channel = self.get_grpc_channel().await?;

        let mut grpc_client = WasmQueryClient::new(channel);

        let build_address_query_request = QueryBuildAddressRequest {
            code_hash: hex::encode(code_id_hash),
            creator_address: creator,
            salt,
            init_args: vec![],
        };

        let build_address_query_response = grpc_client
            .build_address(build_address_query_request)
            .await?;

        let inner = build_address_query_response.into_inner();

        Ok(inner)
    }

    async fn instantiate2(
        &self,
        code_id: u64,
        label: Option<String>,
        msg: (impl Serialize + Send),
        admin: Option<String>,
        salt: String,
    ) -> Result<String, StrategistError> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let msg_bytes = serde_json::to_vec(&msg)?;

        let sender = signing_client.address.to_string();

        let instantiate_contract2_msg = MsgInstantiateContract2 {
            admin: admin.unwrap_or(sender.to_string()),
            sender,
            code_id,
            label: label.unwrap_or_default(),
            msg: msg_bytes,
            funds: vec![],
            salt: hex::decode(salt).map_err(|e| StrategistError::ParseError(e.to_string()))?,
            fix_msg: false,
        };

        // manually encode proto because MsgInstantiateContract2 does not
        // expose the helper method
        let mut value = Vec::new();
        Message::encode(&instantiate_contract2_msg, &mut value)?;

        let any_msg = Any {
            type_url: MsgInstantiateContract2::type_url(),
            value,
        };

        let simulation_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client.create_tx(any_msg, fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        let tx_response = match &broadcast_tx_response.tx_response {
            Some(response) => response,
            None => {
                return Err(StrategistError::TransactionError(
                    "No transaction response returned".to_string(),
                ))
            }
        };
        // poll the node until txhash resolves to a response
        let query_tx_response = self.poll_for_tx(&tx_response.txhash).await?;

        // filter abci logs to find the contract address
        for event in query_tx_response.events.iter() {
            for event_attr in event.attributes.iter() {
                if event_attr.key == "_contract_address" {
                    return Ok(event_attr.value.to_string());
                }
            }
        }

        Err(StrategistError::ParseError(format!(
            "Failed to find contract address in transaction response: {:?}",
            query_tx_response
        )))
    }

    async fn query_contract_state<T: DeserializeOwned>(
        &self,
        contract_address: &str,
        query_data: (impl Serialize + Send),
    ) -> Result<T, StrategistError> {
        let channel = self.get_grpc_channel().await?;

        let mut grpc_client = WasmQueryClient::new(channel);

        let bin_query = serde_json::to_vec(&query_data)?;

        let request = QuerySmartContractStateRequest {
            address: contract_address.to_string(),
            query_data: bin_query,
        };

        let response = grpc_client
            .smart_contract_state(Request::new(request))
            .await?
            .into_inner();

        let parsed: T = serde_json::from_slice(&response.data)?;

        Ok(parsed)
    }

    async fn execute_wasm(
        &self,
        contract: &str,
        msg: (impl Serialize + Send),
        funds: Vec<Coin>,
        fees: Option<Fee>,
    ) -> Result<TransactionResponse, StrategistError> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let msg_bytes = serde_json::to_vec(&msg)?;

        let wasm_tx = MsgExecuteContract {
            sender: signing_client.address.clone(),
            contract: AccountId::from_str(contract)?,
            msg: msg_bytes,
            funds,
        }
        .to_any()?;

        let simulation_response = self.simulate_tx(wasm_tx.clone()).await?;

        // if no fees were specified we simulate the tx and use the estimated fee
        let tx_fee = fees.unwrap_or(self.get_tx_fee(simulation_response)?);

        let raw_tx = signing_client.create_tx(wasm_tx, tx_fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);

        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        match broadcast_tx_response.tx_response {
            Some(tx_response) => Ok(TransactionResponse::try_from(tx_response)?),
            None => Err(StrategistError::TransactionError("failed".to_string())),
        }
    }
}
