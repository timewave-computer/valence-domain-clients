// This file defines the GrpcSigningClient trait, which outlines methods for gRPC communication
// and transaction signing capabilities for Cosmos-based clients.

// use alloy::transports::http::reqwest; // Was for query_chain_gas_config, likely to be re-added if default impl kept
// use cosmos_sdk_proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse as ProtoSimulateResponse};
use cosmrs::Any as CosmrsAny; // For simulate_tx msg parameter
                              // use cosmrs::{
                              //     tx::{BodyBuilder, Fee as CosmrsFee, SignDoc, SignerInfo},
                              //     Coin as CosmrsCoin,
                              // };
use tonic::async_trait;
// use tonic::transport::Channel; // Removed: No longer exposing Channel directly

use crate::{
    core::error::ClientError,
    // Specific types for signatures
    cosmos::types::{CosmosAccount, CosmosFee, CosmosSimulateResponse},
};

// use super::signing_client::SigningClient; // Removed: SigningClient is now internal
// use super::CosmosServiceClient; // May be needed if default implementations are added back to the trait

/// Trait for gRPC enabled Cosmos clients that can sign transactions.
/// Implementers are expected to manage their own gRPC channel and signing capabilities internally.
#[async_trait]
pub trait GrpcSigningClient: Send + Sync {
    // Configuration getters
    fn grpc_url(&self) -> String;
    // Mnemonic is sensitive, consider if it should be part of a public trait.
    // For now, assuming it's used by the client internally upon initialization.
    // fn mnemonic(&self) -> String;
    fn chain_prefix(&self) -> String;
    fn chain_id_str(&self) -> String; // String representation of chain ID, e.g., "cosmoshub-4"
    fn chain_denom(&self) -> String; // Default denomination for the chain
    fn gas_price(&self) -> f64; // Gas price for fee calculation
    fn gas_adjustment(&self) -> f64; // Factor to adjust gas estimation

    /// Retrieves details about the current signer (address, account number, sequence).
    async fn get_signer_details(&self) -> Result<CosmosAccount, ClientError>;

    /// Calculates the transaction fee based on a simulation response.
    /// The implementation will convert the provided `CosmosSimulateResponse` (our type)
    /// into a `CosmosFee` (our type).
    fn get_tx_fee(
        &self,
        simulation_response: CosmosSimulateResponse,
    ) -> Result<CosmosFee, ClientError>;

    /// Simulates a transaction with the given message.
    /// The `msg` parameter is `cosmrs::Any` (which is `prost_types::Any`) for pragmatic reasons,
    /// as abstracting this fully is complex. Callers construct this `Any` message.
    /// Returns our encapsulated `CosmosSimulateResponse`.
    async fn simulate_tx(
        &self,
        msg: CosmrsAny,
    ) -> Result<CosmosSimulateResponse, ClientError>;

    /// Fetches chain-specific gas configuration (e.g., average gas price from a chain registry).
    /// This method remains largely unchanged as it already returned a primitive type.
    async fn query_chain_gas_config(
        &self,
        chain_name: &str,
        denom: &str,
    ) -> Result<f64, ClientError>;

    /// Signs the given message with the provided fee and memo, then broadcasts it.
    /// Implementers will use their internal SigningClient for signing and a gRPC client for broadcasting.
    /// The `msg` is CosmrsAny (prost_types::Any) for pragmatic reasons.
    async fn sign_and_broadcast_tx(
        &self,
        msg: CosmrsAny,
        fee: CosmosFee,
        memo: Option<&str>,
    ) -> Result<crate::core::transaction::TransactionResponse, ClientError>;
}
