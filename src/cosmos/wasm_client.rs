use std::{fs, path::Path, str::FromStr};

use async_trait::async_trait;
use cosmos_sdk_proto::cosmwasm::wasm::v1::{
    MsgInstantiateContract2, QueryBuildAddressRequest, QueryBuildAddressResponse, QueryCodeRequest,
    QueryCodeResponse,
};
use cosmrs::{cosmwasm::MsgInstantiateContract, tx::Fee, Any, Coin};
use prost::{Message, Name};
use serde::{de::DeserializeOwned, Serialize};

use crate::common::transaction::TransactionResponse;
use tonic::Request;

use super::{
    base_client::BaseClient, grpc_client::GrpcSigningClient, CosmosServiceClient, WasmQueryClient,
};

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
pub trait WasmClient: GrpcSigningClient + BaseClient {
    async fn upload_code(&self, wasm_path: &str) -> anyhow::Result<u64> {
        // 1. load the wasm from storage
        let wasm_path = Path::new(wasm_path);
        if !wasm_path.exists() {
            return Err(anyhow::anyhow!(
                "WASM file not found at path: {}",
                wasm_path.display()
            ));
        }

        let wasm_bytes =
            fs::read(wasm_path).map_err(|e| anyhow::anyhow!("Failed to read WASM file: {}", e))?;

        // 2. upload the wasm to the chain
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let store_code_msg = MsgStoreCode {
            sender: signing_client.address.clone(),
            wasm_byte_code: wasm_bytes,
            instantiate_permission: None,
        }
        .to_any()
        .map_err(|e| anyhow::anyhow!("failed to convert MsgStoreCode to proto Any: {e}"))?;

        let simulation_response = self.simulate_tx(store_code_msg.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client.create_tx(store_code_msg, fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        let tx_response = match &broadcast_tx_response.tx_response {
            Some(response) => response,
            None => return Err(anyhow::anyhow!("No transaction response returned")),
        };

        // poll the node until txhash resolves to a response
        let query_tx_response = self.poll_for_tx(&tx_response.txhash).await?;

        // filter events to find the resulting code id
        for event in query_tx_response.events.iter() {
            if event.r#type == "store_code" {
                for attr in event.attributes.iter() {
                    if attr.key == "code_id" {
                        return attr
                            .value
                            .parse::<u64>()
                            .map_err(|_| anyhow::anyhow!("Failed to parse code_id"));
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to find code_id in transaction response: {:?}",
            query_tx_response
        ))
    }

    async fn instantiate(
        &self,
        code_id: u64,
        label: String,
        msg: impl Serialize + Send,
        admin: Option<String>,
    ) -> anyhow::Result<String> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let msg_bytes = serde_json::to_vec(&msg)?;

        if label.is_empty() {
            return Err(anyhow::anyhow!("contract label cannot be empty"));
        }

        let admin = admin
            .map(|a| AccountId::from_str(&a))
            .transpose()
            .map_err(|e| anyhow::anyhow!("failed to get AccountId from str: {e}"))?;

        let instantiate_tx = MsgInstantiateContract {
            sender: signing_client.address.clone(),
            admin,
            code_id,
            label: Some(label),
            msg: msg_bytes,
            funds: vec![],
        }
        .to_any()
        .map_err(|e| {
            anyhow::anyhow!("failed to convert MsgInstantiateContract to proto Any: {e}")
        })?;

        let simulation_response = self.simulate_tx(instantiate_tx.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client.create_tx(instantiate_tx, fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        let tx_response = match &broadcast_tx_response.tx_response {
            Some(response) => response,
            None => return Err(anyhow::anyhow!("No transaction response returned")),
        };

        // poll the node until txhash resolves to a response
        let query_tx_response = self.poll_for_tx(&tx_response.txhash).await?;

        // filter events to find the contract address
        for event in query_tx_response.events.iter() {
            if event.r#type == "instantiate" || event.r#type == "instantiate_contract" {
                for attr in event.attributes.iter() {
                    if attr.key == "_contract_address" || attr.key == "contract_address" {
                        return Ok(attr.value.clone());
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to find contract address in transaction response: {:?}",
            query_tx_response
        ))
    }

    async fn query_code_info(&self, code_id: u64) -> anyhow::Result<QueryCodeResponse> {
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
    ) -> anyhow::Result<QueryBuildAddressResponse> {
        let code_info = self.query_code_info(code_id).await?.code_info;

        let code_id_hash = match code_info {
            Some(ci) => ci.data_hash,
            None => return Err(anyhow::anyhow!("failed to get code info from code query")),
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
        label: String,
        msg: impl Serialize + Send,
        admin: Option<String>,
        salt: String,
    ) -> anyhow::Result<String> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let msg_bytes = serde_json::to_vec(&msg)?;

        let sender = signing_client.address.to_string();

        if label.is_empty() {
            return Err(anyhow::anyhow!("contract label cannot be empty"));
        }

        let instantiate_contract2_msg = MsgInstantiateContract2 {
            admin: admin.unwrap_or_default(),
            sender,
            code_id,
            label,
            msg: msg_bytes,
            funds: vec![],
            salt: hex::decode(salt)
                .map_err(|e| anyhow::anyhow!("failed to decode hex salt: {e}"))?,
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
            None => return Err(anyhow::anyhow!("No transaction response returned")),
        };
        // poll the node until txhash resolves to a response
        let query_tx_response = self.poll_for_tx(&tx_response.txhash).await?;

        // filter events to find the contract address
        for event in query_tx_response.events.iter() {
            for event_attr in event.attributes.iter() {
                if event_attr.key == "_contract_address" {
                    return Ok(event_attr.value.to_string());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to find contract address in transaction response: {:?}",
            query_tx_response
        ))
    }

    async fn query_contract_state<T: DeserializeOwned>(
        &self,
        contract_address: &str,
        query_data: impl Serialize + Send,
    ) -> anyhow::Result<T> {
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
        msg: impl Serialize + Send,
        funds: Vec<Coin>,
        fees: Option<Fee>,
    ) -> anyhow::Result<TransactionResponse> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        let msg_bytes = serde_json::to_vec(&msg)?;

        let wasm_tx = MsgExecuteContract {
            sender: signing_client.address.clone(),
            contract: AccountId::from_str(contract).map_err(|e| {
                anyhow::anyhow!("failed to parse contract addr into AccountId: {e}")
            })?,
            msg: msg_bytes,
            funds,
        }
        .to_any()
        .map_err(|e| anyhow::anyhow!("failed to convert MsgExecuteContract to proto Any: {e}"))?;

        let simulation_response = self.simulate_tx(wasm_tx.clone()).await?;

        // if no fees were specified we simulate the tx and use the estimated fee
        let tx_fee = fees.unwrap_or(self.get_tx_fee(simulation_response)?);

        let raw_tx = signing_client.create_tx(wasm_tx, tx_fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);

        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        match broadcast_tx_response.tx_response {
            Some(tx_response) => Ok(TransactionResponse::try_from(tx_response)?),
            None => Err(anyhow::anyhow!("failed")),
        }
    }
}
