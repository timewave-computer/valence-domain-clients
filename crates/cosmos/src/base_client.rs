//-----------------------------------------------------------------------------
// Cosmos Base Client Interface
//-----------------------------------------------------------------------------

//! Base client interface for Cosmos ecosystem blockchains.
//!
//! This module defines the core traits that all Cosmos clients must implement.

use async_trait::async_trait;

// Use the new workspace crates instead of internal modules
use valence_core::error::ClientError;
use valence_core::transaction::TransactionResponse;

// Local modules
use crate::grpc_client::GrpcSigningClient;
use crate::types::{
    CosmosBlockResults, CosmosCoin, CosmosHeader, CosmosModuleAccount,
};

/// Base client trait for Cosmos SDK-based chains.
///
/// Concrete client implementations for Cosmos-based chains will implement these methods,
/// leveraging their internal gRPC connections and signing capabilities.
#[async_trait]
pub trait CosmosBaseClient: GrpcSigningClient + Send + Sync {
    /// Creates a `CosmosCoin` instance with the specified denomination and amount.
    ///
    /// This is a utility function and does not involve any network calls.
    fn create_cosmos_coin(
        &self,
        denom: &str,
        amount: u128,
    ) -> Result<CosmosCoin, ClientError> {
        if denom.is_empty() {
            return Err(ClientError::ParseError(
                "Denom cannot be empty".to_string(),
            ));
        }
        Ok(CosmosCoin {
            denom: denom.to_string(),
            amount,
        })
    }

    /// Initiates a token transfer.
    async fn transfer(
        &self,
        to_address_str: &str,
        amount: u128,
        denom: &str,
        memo: Option<&str>,
    ) -> Result<TransactionResponse, ClientError>;

    /// Fetches the latest block header from the chain.
    async fn latest_block_header(&self) -> Result<CosmosHeader, ClientError>;

    /// Fetches block results for a given height.
    async fn block_results(
        &self,
        height: u64,
    ) -> Result<CosmosBlockResults, ClientError>;

    /// Queries the balance of a specific account for a given denomination.
    async fn query_balance(
        &self,
        address: &str,
        denom: &str,
    ) -> Result<u128, ClientError>;

    /// Queries a module account by its name.
    async fn query_module_account(
        &self,
        name: &str,
    ) -> Result<CosmosModuleAccount, ClientError>;

    /// Polls for a transaction receipt by its hash until it is confirmed or times out.
    async fn poll_for_tx(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionResponse, ClientError>;

    /// Polls for an account balance until it reaches an expected minimum or times out.
    async fn poll_until_expected_balance(
        &self,
        address: &str,
        denom: &str,
        min_amount: u128,
        interval_sec: u64,
        max_attempts: u32,
    ) -> Result<u128, ClientError>;

    /// Initiates an IBC token transfer.
    async fn ibc_transfer(
        &self,
        to_address: String,
        denom: String,
        amount: String,
        source_channel: String,
        timeout_seconds: u64,
        memo: Option<String>,
    ) -> Result<TransactionResponse, ClientError>;
}
