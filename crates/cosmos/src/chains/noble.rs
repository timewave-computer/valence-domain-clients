//-----------------------------------------------------------------------------
// Noble Client Implementation
//-----------------------------------------------------------------------------

//! Noble blockchain client implementation.
//!
//! This module provides functionality for interacting with the Noble blockchain,
//! particularly for CCTP and token factory operations.

use cosmrs::Any as CosmrsAny;
use prost_types::Any;
use tonic::async_trait;

use valence_core::error::ClientError;

// Conversion from prost::EncodeError to ClientError is already implemented in core/error.rs
use crate::base_client::CosmosBaseClient;
use crate::generic_client::{CosmosClientConfig, GenericCosmosClient};
use crate::grpc_client::GrpcSigningClient;
use crate::types::{
    CosmosAccount, CosmosBlockResults, CosmosFee, CosmosHeader, CosmosModuleAccount,
    CosmosSimulateResponse,
};
use valence_core::transaction::TransactionResponse;

// Chain-specific constants
const CHAIN_PREFIX: &str = "noble";
const CHAIN_DENOM: &str = "uusdc";
const _CCTP_MODULE_NAME: &str = "cctp";
const DEFAULT_NOBLE_GAS_ADJUSTMENT: f64 = 1.5;
const DEFAULT_NOBLE_GAS_PRICE: f64 = 0.1;

/// Client for interacting with the Noble blockchain
pub struct NobleClient {
    inner: GenericCosmosClient,
}

impl NobleClient {
    /// Creates a new NobleClient instance
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
            gas_price: DEFAULT_NOBLE_GAS_PRICE,
            gas_adjustment: DEFAULT_NOBLE_GAS_ADJUSTMENT,
        };

        let generic_client = GenericCosmosClient::new(config, mnemonic).await?;

        Ok(Self {
            inner: generic_client,
        })
    }

    /// Mints fiat tokens to a specified receiver
    ///
    /// This method allows authorized accounts to mint fiat-backed tokens
    /// to a specific receiver address.
    pub async fn mint_fiat(
        &self,
        _receiver: &str,
        amount: &str,
        denom: &str,
    ) -> Result<TransactionResponse, ClientError> {
        // Create the MintFiat message - directly using Any
        let any_msg = prost_types::Any {
            type_url: "/noble.tokenfactory.v1.MsgMint".to_string(),
            value: serde_json::to_vec(&serde_json::json!({
                "sender": self.get_signer_details().await?.address,
                "amount": {
                    "denom": denom,
                    "amount": amount
                }
            }))
            .map_err(|e| ClientError::SerializationError(e.to_string()))?,
        };

        // Simulate the transaction
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;

        // Sign and broadcast
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }

    /// Deposits USDC to the CCTP contract for cross-chain transfer
    ///
    /// This method initiates a transfer of USDC from Noble to another blockchain
    /// through the Circle Cross-Chain Transfer Protocol (CCTP).
    ///
    /// # Arguments
    ///
    /// * `destination_domain` - The domain identifier of the destination chain
    /// * `destination_address` - The hex-encoded recipient address (without 0x prefix)
    /// * `amount` - Amount of USDC to transfer (in base units, e.g., 1000000 for 1 USDC)
    /// * `burn_token_denom` - The denomination of the token to burn (usually "uusdc")
    ///
    /// # Returns
    ///
    /// Result containing transaction response or error
    pub async fn cctp_deposit_for_burn(
        &self,
        destination_domain: u32,
        destination_address: &str,
        amount: &str,
        burn_token_denom: &str,
    ) -> Result<TransactionResponse, ClientError> {
        // Ensure destination_address has no 0x prefix
        let dest_addr = destination_address.trim_start_matches("0x");

        // Convert hex address to bytes
        let mut recipient_bytes = Vec::with_capacity(32);
        for i in (0..dest_addr.len()).step_by(2) {
            if i + 2 <= dest_addr.len() {
                let byte =
                    u8::from_str_radix(&dest_addr[i..i + 2], 16).map_err(|e| {
                        ClientError::ParseError(format!(
                            "Failed to parse hex address: {e}"
                        ))
                    })?;
                recipient_bytes.push(byte);
            }
        }

        // Pad to 32 bytes if needed
        while recipient_bytes.len() < 32 {
            recipient_bytes.push(0);
        }

        // Create the DepositForBurn message directly using Any
        let any_msg = prost_types::Any {
            type_url: "/noble.cctp.v1.MsgDepositForBurn".to_string(),
            value: serde_json::to_vec(&serde_json::json!({
                "from": self.get_signer_details().await?.address,
                "amount": amount,
                "destinationDomain": destination_domain.to_string(),
                "mintRecipient": recipient_bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                "burnToken": burn_token_denom
            })).map_err(|e| ClientError::SerializationError(e.to_string()))?,
        };

        // Simulate for fee estimation
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;

        // Sign and broadcast
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }

    /// Receives a message from another chain via CCTP
    ///
    /// This processes an attestation from Circle to complete a cross-chain transfer
    /// to Noble.
    ///
    /// # Arguments
    ///
    /// * `message` - The cross-chain message bytes
    /// * `attestation` - The attestation signature bytes
    ///
    /// # Returns
    ///
    /// Result containing transaction response or error
    pub async fn cctp_receive_message(
        &self,
        message: Vec<u8>,
        attestation: Vec<u8>,
    ) -> Result<TransactionResponse, ClientError> {
        // Create the ReceiveMessage message directly as Any
        let any_msg = Any {
            type_url: "/noble.cctp.v1.MsgReceiveMessage".to_string(),
            value: serde_json::to_vec(&serde_json::json!({
                "from": self.get_signer_details().await?.address,
                "message": message,
                "attestation": attestation
            }))
            .map_err(|e| ClientError::SerializationError(e.to_string()))?,
        };

        // Simulate for fee estimation
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;

        // Sign and broadcast
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }
}

#[async_trait]
impl GrpcSigningClient for NobleClient {
    fn grpc_url(&self) -> String {
        self.inner.grpc_url()
    }

    fn chain_prefix(&self) -> String {
        self.inner.chain_prefix()
    }

    fn chain_id_str(&self) -> String {
        self.inner.chain_id_str()
    }

    fn chain_denom(&self) -> String {
        self.inner.chain_denom()
    }

    fn gas_price(&self) -> f64 {
        self.inner.gas_price()
    }

    fn gas_adjustment(&self) -> f64 {
        self.inner.gas_adjustment()
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

    async fn query_chain_gas_config(
        &self,
        _chain_name: &str,
        _chain_denom: &str,
    ) -> Result<f64, ClientError> {
        // Default Noble gas price
        Ok(DEFAULT_NOBLE_GAS_PRICE)
    }
}

#[async_trait]
impl CosmosBaseClient for NobleClient {
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
        // Noble chain specific handling for IBC transfers
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

// Note: Noble-specific proto message types and conversion functions will be implemented here

// Note: Noble-specific implementations
// as we integrate the Noble protocol buffer definitions into our new structure
