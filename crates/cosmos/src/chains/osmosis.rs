//-----------------------------------------------------------------------------
// Osmosis Client Implementation
//-----------------------------------------------------------------------------

//! Osmosis blockchain client implementation.
//!
//! This module provides functionality for interacting with the Osmosis blockchain,
//! particularly for DeFi operations.

use tonic::async_trait;

use cosmrs::Any as CosmrsAny;

use crate::base_client::CosmosBaseClient;
use crate::generic_client::{CosmosClientConfig, GenericCosmosClient};
use crate::grpc_client::GrpcSigningClient;
use crate::types::{
    CosmosAccount, CosmosBlockResults, CosmosCoin, CosmosFee, CosmosHeader,
    CosmosModuleAccount, CosmosSimulateResponse,
};
use valence_core::error::ClientError;
use valence_core::transaction::TransactionResponse;

// Chain-specific constants
const CHAIN_PREFIX: &str = "osmo";
const CHAIN_DENOM: &str = "uosmo";
const DEFAULT_OSMOSIS_GAS_ADJUSTMENT: f64 = 1.5;
const DEFAULT_OSMOSIS_GAS_PRICE: f64 = 0.025;

/// Client for interacting with the Osmosis blockchain
pub struct OsmosisClient {
    inner: GenericCosmosClient,
}

impl OsmosisClient {
    /// Creates a new OsmosisClient instance
    pub async fn new(
        grpc_url: &str,
        chain_id: &str,
        mnemonic: &str,
        _derivation_path: Option<&str>, // Currently not used, cosmrs handles derivation internally
    ) -> Result<Self, ClientError> {
        let config = CosmosClientConfig {
            grpc_url: grpc_url.to_string(),
            chain_id_str: chain_id.to_string(),
            chain_prefix: CHAIN_PREFIX.to_string(),
            chain_denom: CHAIN_DENOM.to_string(),
            gas_price: DEFAULT_OSMOSIS_GAS_PRICE,
            gas_adjustment: DEFAULT_OSMOSIS_GAS_ADJUSTMENT,
        };

        let inner = GenericCosmosClient::new(config, mnemonic).await?;

        Ok(Self { inner })
    }

    /// Swaps tokens on the Osmosis DEX
    ///
    /// This method allows swapping between tokens on the Osmosis DEX.
    pub async fn swap_tokens(
        &self,
        token_in_denom: &str,
        token_out_denom: &str,
        token_in_amount: u128,
        min_output_amount: u128,
    ) -> Result<TransactionResponse, ClientError> {
        // Until we migrate the Osmosis protobuf implementation, we'll return NotImplemented
        // In the actual implementation, we would:
        // 1. Create a MsgSwapExactAmountIn with the specified parameters
        // 2. Convert it to CosmrsAny
        // 3. Simulate the transaction to get fee estimates
        // 4. Sign and broadcast the transaction

        // For now, return a clear error about the implementation status
        Err(ClientError::NotImplemented(
            format!("swap_tokens functionality awaiting Osmosis proto implementation. \
                   Would swap {token_in_amount} {token_in_denom} to at least {min_output_amount} {token_out_denom}")
        ))
    }

    /// Adds liquidity to a pool
    ///
    /// This method allows adding liquidity to an Osmosis liquidity pool.
    pub async fn add_liquidity(
        &self,
        pool_id: u64,
        token_amounts: Vec<CosmosCoin>,
    ) -> Result<TransactionResponse, ClientError> {
        // Until we migrate the Osmosis protobuf implementation, we'll return NotImplemented
        // In the actual implementation, we would:
        // 1. Create a MsgJoinPool with the specified parameters
        // 2. Convert it to CosmrsAny
        // 3. Simulate the transaction to get fee estimates
        // 4. Sign and broadcast the transaction

        // For now, return a clear error about the implementation status
        let tokens_desc = token_amounts
            .iter()
            .map(|c| format!("{} {}", c.amount, c.denom))
            .collect::<Vec<_>>()
            .join(", ");

        Err(ClientError::NotImplemented(format!(
            "add_liquidity functionality awaiting Osmosis proto implementation. \
                   Would add [{tokens_desc}] to pool {pool_id}"
        )))
    }
}

#[async_trait]
impl GrpcSigningClient for OsmosisClient {
    fn grpc_url(&self) -> String {
        self.inner.config.grpc_url.clone()
    }

    fn chain_prefix(&self) -> String {
        self.inner.config.chain_prefix.clone()
    }

    fn chain_id_str(&self) -> String {
        self.inner.config.chain_id_str.clone()
    }

    fn chain_denom(&self) -> String {
        self.inner.config.chain_denom.clone()
    }

    fn gas_price(&self) -> f64 {
        self.inner.config.gas_price
    }

    fn gas_adjustment(&self) -> f64 {
        self.inner.config.gas_adjustment
    }

    async fn query_chain_gas_config(
        &self,
        chain_name: &str,
        denom: &str,
    ) -> Result<f64, ClientError> {
        // Here we would typically fetch chain gas configuration from a registry or database
        // For now, return a default value based on chain name
        match chain_name {
            "osmosis" => Ok(DEFAULT_OSMOSIS_GAS_PRICE),
            _ => Err(ClientError::NotImplemented(
                format!("Gas config for chain {chain_name} with denom {denom} not implemented")
            )),
        }
    }

    async fn get_signer_details(&self) -> Result<CosmosAccount, ClientError> {
        self.inner.get_signer_details().await
    }

    fn get_tx_fee(
        &self,
        simulation_response: CosmosSimulateResponse,
    ) -> Result<CosmosFee, ClientError> {
        self.inner.get_tx_fee(simulation_response)
    }

    async fn simulate_tx(
        &self,
        msg: CosmrsAny,
    ) -> Result<CosmosSimulateResponse, ClientError> {
        self.inner.simulate_tx(msg).await
    }

    async fn sign_and_broadcast_tx(
        &self,
        msg: CosmrsAny,
        fee: CosmosFee,
        memo: Option<&str>,
    ) -> Result<TransactionResponse, ClientError> {
        self.inner.sign_and_broadcast_tx(msg, fee, memo).await
    }
}

#[async_trait]
impl CosmosBaseClient for OsmosisClient {
    async fn transfer(
        &self,
        to_address_str: &str,
        amount: u128,
        denom: &str,
        memo: Option<&str>,
    ) -> Result<TransactionResponse, ClientError> {
        self.inner
            .transfer(to_address_str, amount, denom, memo)
            .await
    }

    async fn latest_block_header(&self) -> Result<CosmosHeader, ClientError> {
        self.inner.latest_block_header().await
    }

    async fn block_results(
        &self,
        height: u64,
    ) -> Result<CosmosBlockResults, ClientError> {
        self.inner.block_results(height).await
    }

    async fn query_balance(
        &self,
        address: &str,
        denom: &str,
    ) -> Result<u128, ClientError> {
        self.inner.query_balance(address, denom).await
    }

    async fn query_module_account(
        &self,
        name: &str,
    ) -> Result<CosmosModuleAccount, ClientError> {
        self.inner.query_module_account(name).await
    }

    async fn poll_for_tx(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionResponse, ClientError> {
        self.inner.poll_for_tx(tx_hash).await
    }

    async fn poll_until_expected_balance(
        &self,
        address: &str,
        denom: &str,
        min_amount: u128,
        interval_sec: u64,
        max_attempts: u32,
    ) -> Result<u128, ClientError> {
        self.inner
            .poll_until_expected_balance(
                address,
                denom,
                min_amount,
                interval_sec,
                max_attempts,
            )
            .await
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
        // Osmosis has excellent IBC support
        self.inner
            .ibc_transfer(
                to_address,
                denom,
                amount,
                source_channel,
                timeout_seconds,
                memo,
            )
            .await
    }
}

// Note: Osmosis-specific proto message types and conversion functions will be implemented here
// as we integrate the Osmosis protocol buffer definitions into our new structure
