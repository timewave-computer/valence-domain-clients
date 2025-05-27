use std::{fs, path::Path, str::FromStr};

use async_trait::async_trait;
use cosmrs::{tx::Fee, Coin};
use serde::{de::DeserializeOwned, Serialize};

use crate::common::{error::StrategistError, transaction::TransactionResponse};
use tonic::Request;

use super::{grpc_client::GrpcSigningClient, CosmosServiceClient, WasmQueryClient};

use cosmrs::{
    cosmwasm::{MsgExecuteContract, MsgStoreCode}, 
    proto::cosmwasm::wasm::v1::QuerySmartContractStateRequest,
    tx::Msg, AccountId,
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
        }.to_any()?;

        let simulation_response = self.simulate_tx(store_code_msg.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client.create_tx(store_code_msg, fee, None).await?;

        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        let tx_response = match &broadcast_tx_response.tx_response {
            Some(response) => TransactionResponse::try_from(response.clone())?,
            None => {
                return Err(StrategistError::TransactionError(
                    "No transaction response returned".to_string(),
                ))
            }
        };
        
        if !tx_response.success {
            return Err(StrategistError::TransactionError(
                "Failed to upload WASM code".to_string(),
            ));
        }

        // 3. Extract the code ID from the transaction events
        // We know tx_response is Some based on previous check
        let response = broadcast_tx_response.tx_response.as_ref().unwrap();
        
        // Find the "store_code" event and extract the code_id attribute
        for event in response.logs.iter().flat_map(|log| &log.events) {
            if event.r#type == "store_code" {
                for attr in &event.attributes {
                    if attr.key == "code_id" {
                        return attr.value.parse::<u64>().map_err(|_| {
                            StrategistError::ParseError("Failed to parse code_id".to_string())
                        });
                    }
                }
            }
        }

        Err(StrategistError::ParseError(
            "Failed to find code_id in transaction response".to_string(),
        ))
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
