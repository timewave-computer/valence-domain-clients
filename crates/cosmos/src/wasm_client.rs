// This file defines the WasmClient trait, providing functionalities for interacting
// with CosmWasm smart contracts on Cosmos-based chains.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

// Original cosmrs types, no longer directly in public signatures:
// use cosmrs::Coin as CosmrsCoin;
// use cosmrs::tx::Fee as CosmrsFee;

use crate::{
    common::{error::ClientError, transaction::TransactionResponse},
    cosmos::types_cosmos::{CosmosCoin, CosmosFee}, // Corrected path
};

use super::grpc_client::GrpcSigningClient; // Inherits from the refactored GrpcSigningClient

// Dependencies for *potential future* default implementations (if any remain here)
// use cosmrs::cosmwasm::MsgExecuteContract;
// use cosmrs::tx::Msg;
// use cosmrs::AccountId;
// use std::str::FromStr;
// use crate::common::transaction::convert_from_proto_tx_response; // If converting raw proto
// use super::CosmosServiceClient; // For broadcasting if not using sign_and_broadcast_tx
// use super::WasmQueryClient; // For query_contract_state default impl
// use tonic::Request as TonicRequest;

/// Trait for CosmWasm capable clients.
/// Implementers will manage their own gRPC connections and signing logic,
/// leveraging their `GrpcSigningClient` implementation.
#[async_trait]
pub trait WasmClient: GrpcSigningClient {
    /// Queries a smart contract's state.
    /// `query_data` is serialized to JSON for the query message.
    /// The response is deserialized into type `T`.
    async fn query_contract_state<T: DeserializeOwned + Send>(
        &self,
        contract_address: &str,
        query_data: impl Serialize + Send,
    ) -> Result<T, ClientError>;

    /// Executes a command on a smart contract.
    /// `msg` is serialized to JSON for the execute message.
    /// `funds` are sent with the transaction, using our `CosmosCoin` type.
    /// `fees` can be provided, or if `None`, the client should attempt to estimate them.
    async fn execute_wasm(
        &self,
        contract_address_str: &str,
        msg: impl Serialize + Send,
        funds: Vec<CosmosCoin>,
        fees: Option<CosmosFee>,
    ) -> Result<TransactionResponse, ClientError>;
}
