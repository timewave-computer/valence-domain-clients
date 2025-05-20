//-----------------------------------------------------------------------------
// Ethereum Chain Client
//-----------------------------------------------------------------------------

//! Client implementation for the Ethereum blockchain.
//!
//! This module provides a full client for interacting with the Ethereum blockchain
//! through its JSON-RPC API.

// Import basic standard library components
use std::collections::HashMap;

// Import tokio async runtime components

// Import ethers.js library components
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde_json::{json, Value};

// Import core types from valence-core
use valence_core::error::ClientError;
use valence_core::transaction::TransactionResponse;

// Import local types and interfaces
use crate::base_client::EvmBaseClient;
use crate::types::{EvmAddress, EvmBytes, EvmHash, EvmTransactionRequest, EvmU256};
use crate::{EvmClientConfig, GenericEvmClient};

// Import Flashbots bundle types and operations
use crate::bundle::{
    BundleResponse, EthSendBundleParams, FlashbotsBundleOperations,
    MevSendBundleParams,
};

// Define the Flashbots relay URL constant
const FLASHBOTS_RELAY_URL: &str = "https://relay.flashbots.net";

// Include the fully isolated crypto adapter for signing
use crate::crypto_adapter::{keccak256, sign_message};

//-----------------------------------------------------------------------------
// Ethereum Client Structure
//-----------------------------------------------------------------------------

/// Client for interacting with Ethereum and compatible networks
pub struct EthereumClient {
    inner: Box<GenericEvmClient>,
    private_key: Option<[u8; 32]>,
    http_client: HttpClient,
}

//-----------------------------------------------------------------------------
// Client Implementation
//-----------------------------------------------------------------------------

impl EthereumClient {
    /// Create a new Ethereum client with the specified endpoint
    pub fn new(
        rpc_url: &str,
        _mnemonic: &str,
        _derivation_path: Option<&str>,
    ) -> Result<Self, ClientError> {
        // For now, we'll just use a hardcoded chain ID and add proper implementation later
        let chain_id = 1; // Ethereum mainnet

        let config = EvmClientConfig {
            rpc_url: rpc_url.to_string(),
            chain_id,
            gas_adjustment: 1.3,
            default_gas_limit: 21000,
            max_gas_price_gwei: 200.0,
        };

        let inner = GenericEvmClient::new(config);

        Ok(Self {
            inner: Box::new(inner),
            private_key: None,
            http_client: HttpClient::new(),
        })
    }

    /// Create a new Ethereum client with a private key for transaction signing
    pub fn with_private_key(
        rpc_url: &str,
        _chain_id: u64,
        private_key: [u8; 32],
    ) -> Self {
        let mut client = Self::new(rpc_url, "", None).unwrap();
        client.private_key = Some(private_key);
        client
    }

    /// Get the RPC provider URL
    pub fn rpc_url(&self) -> &str {
        self.inner.rpc_url()
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.inner.chain_id()
    }

    /// Check if the client has a private key for signing
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    /// Set the Flashbots authentication key for bundle submission
    pub fn with_flashbots_auth(mut self, auth_key: [u8; 32]) -> Self {
        self.private_key = Some(auth_key);
        self
    }

    /// Convert Ethereum client errors to ClientError
    fn handle_flashbots_error(&self, error: reqwest::Error) -> ClientError {
        ClientError::ClientError(format!("Flashbots request failed: {error}"))
    }

    //-----------------------------------------------------------------------------
    // ERC-20 Token Methods
    //-----------------------------------------------------------------------------

    /// Get the balance of an ERC-20 token
    pub async fn get_token_balance(
        &self,
        token_address: &EvmAddress,
        wallet_address: &EvmAddress,
    ) -> Result<EvmU256, ClientError> {
        // ERC-20 balanceOf function signature
        let data = EvmBytes(
            hex::decode(
                "70a08231000000000000000000000000".to_string()
                    + &hex::encode(&wallet_address.0[..]),
            )
            .map_err(|e| ClientError::ParseError(e.to_string()))?,
        );

        let result = self
            .inner
            .call_contract(token_address, &data, None, None)
            .await?;

        // Parse the result (simplified)
        let hex_string = result.to_hex();
        let hex = hex_string.trim_start_matches("0x");
        let value = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse token balance: {e}"))
        })?;

        Ok(EvmU256::from_u64(value))
    }

    /// Transfer ERC-20 tokens
    pub async fn transfer_tokens(
        &self,
        token_address: &EvmAddress,
        to_address: &EvmAddress,
        amount: EvmU256,
    ) -> Result<TransactionResponse, ClientError> {
        if self.private_key.is_none() {
            return Err(ClientError::ClientError(
                "No private key available for signing".to_string(),
            ));
        }

        // ERC-20 transfer function signature + params
        let mut data = hex::decode("a9059cbb000000000000000000000000").unwrap();
        data.extend_from_slice(&to_address.0);

        // Pad amount to 32 bytes
        let mut amount_bytes = [0u8; 32];
        let amount_str = amount.to_string();
        let amount_hex = hex::decode(amount_str.trim_start_matches("0x"))
            .map_err(|e| ClientError::ParseError(e.to_string()))?;

        // Right-align amount in 32-byte field
        let start_idx = 32 - amount_hex.len();
        amount_bytes[start_idx..].copy_from_slice(&amount_hex);
        data.extend_from_slice(&amount_bytes);

        let tx_request = EvmTransactionRequest {
            from: EvmAddress([0u8; 20]), // Will be set during signing
            to: Some(token_address.clone()),
            nonce: None,     // Will be fetched during signing
            gas_limit: None, // Will be estimated
            gas_price: None, // Will be fetched
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            value: Some(EvmU256::from_u64(0)),
            data: Some(EvmBytes(data)),
            chain_id: Some(self.chain_id()),
        };

        self.inner.send_transaction(&tx_request).await
    }

    // Sign a Flashbots request using the private key
    // This method uses our isolated crypto adapter
    fn sign_flashbots_request(&self, message: &[u8]) -> Result<String, ClientError> {
        let private_key = self.private_key.as_ref().ok_or_else(|| {
            ClientError::ClientError(
                "No private key available for signing".to_string(),
            )
        })?;

        // Hash the message with keccak256
        let message_hash = keccak256(message);

        // Sign the hash with the private key
        let (signature, _recovery_id) = sign_message(private_key, &message_hash.0)?;

        // Format the signature as hex string
        let signature_hex = format!("0x{}", hex::encode(signature));

        Ok(signature_hex)
    }
}

//-----------------------------------------------------------------------------
// EvmBaseClient Implementation
//-----------------------------------------------------------------------------

#[async_trait]
impl EvmBaseClient for EthereumClient {
    fn evm_signer_address(&self) -> EvmAddress {
        // Return a placeholder zero address since we don't have a signer
        EvmAddress([0u8; 20])
    }

    async fn get_balance(
        &self,
        address: &EvmAddress,
    ) -> Result<EvmU256, ClientError> {
        self.inner.get_balance(address).await
    }

    async fn get_nonce(&self, address: &EvmAddress) -> Result<u64, ClientError> {
        self.inner.get_nonce(address).await
    }

    async fn send_raw_transaction(
        &self,
        tx_bytes: &EvmBytes,
    ) -> Result<EvmHash, ClientError> {
        self.inner.send_raw_transaction(tx_bytes).await
    }

    async fn send_transaction(
        &self,
        tx: &EvmTransactionRequest,
    ) -> Result<TransactionResponse, ClientError> {
        // In a real implementation, we'd check if we have a private key and sign it here
        // For now, delegate to the inner client
        self.inner.send_transaction(tx).await
    }

    async fn get_transaction(
        &self,
        tx_hash: &EvmHash,
    ) -> Result<Option<TransactionResponse>, ClientError> {
        self.inner.get_transaction(tx_hash).await
    }

    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: &EvmHash,
    ) -> Result<TransactionResponse, ClientError> {
        self.inner.wait_for_transaction_receipt(tx_hash).await
    }

    async fn get_block_number(&self) -> Result<u64, ClientError> {
        self.inner.get_block_number().await
    }

    async fn get_chain_id(&self) -> Result<u64, ClientError> {
        self.inner.get_chain_id().await
    }

    async fn get_gas_price(&self) -> Result<EvmU256, ClientError> {
        self.inner.get_gas_price().await
    }

    async fn call_contract(
        &self,
        to: &EvmAddress,
        data: &EvmBytes,
        from: Option<&EvmAddress>,
        block: Option<u64>,
    ) -> Result<EvmBytes, ClientError> {
        self.inner.call_contract(to, data, from, block).await
    }

    async fn estimate_gas(
        &self,
        to: Option<&EvmAddress>,
        data: &EvmBytes,
        value: Option<EvmU256>,
        from: Option<&EvmAddress>,
    ) -> Result<EvmU256, ClientError> {
        self.inner.estimate_gas(to, data, value, from).await
    }
}

//-----------------------------------------------------------------------------
// FlashbotsBundleOperations Implementation
//-----------------------------------------------------------------------------

#[async_trait]
impl FlashbotsBundleOperations for EthereumClient {
    async fn send_eth_bundle(
        &self,
        params: EthSendBundleParams,
    ) -> Result<BundleResponse, ClientError> {
        if self.private_key.is_none() {
            return Err(ClientError::ClientError(
                "No Flashbots authentication key provided".to_string(),
            ));
        }

        // Create the request body
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_sendBundle",
            "params": [params]
        });

        // Calculate signature for Flashbots authentication
        let message = format!("{request_body}");
        let signature = self.sign_flashbots_request(message.as_bytes())?;

        // Send the request to Flashbots relay
        let response = self
            .http_client
            .post(FLASHBOTS_RELAY_URL)
            .header("X-Flashbots-Signature", signature)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| self.handle_flashbots_error(e))?;

        // Parse the response
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| self.handle_flashbots_error(e))?;

        if let Some(error) = response_json.get("error") {
            return Err(ClientError::ClientError(format!(
                "Flashbots error: {error}"
            )));
        }

        let result = response_json.get("result").ok_or_else(|| {
            ClientError::ClientError("No result in Flashbots response".to_string())
        })?;

        let bundle_hash = result
            .get("bundleHash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ClientError::ClientError("No bundleHash in response".to_string())
            })?;

        Ok(BundleResponse {
            bundle_hash: bundle_hash.to_string(),
        })
    }

    async fn send_mev_bundle(
        &self,
        params: MevSendBundleParams,
    ) -> Result<BundleResponse, ClientError> {
        if self.private_key.is_none() {
            return Err(ClientError::ClientError(
                "No Flashbots authentication key provided".to_string(),
            ));
        }

        // Create the request body
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "mev_sendBundle",
            "params": [params]
        });

        // Calculate signature for Flashbots authentication
        let message = format!("{request_body}");
        let signature = self.sign_flashbots_request(message.as_bytes())?;

        // Send the request to Flashbots relay
        let response = self
            .http_client
            .post(FLASHBOTS_RELAY_URL)
            .header("X-Flashbots-Signature", signature)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| self.handle_flashbots_error(e))?;

        // Parse the response
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| self.handle_flashbots_error(e))?;

        if let Some(error) = response_json.get("error") {
            return Err(ClientError::ClientError(format!(
                "Flashbots error: {error}"
            )));
        }

        let result = response_json.get("result").ok_or_else(|| {
            ClientError::ClientError("No result in Flashbots response".to_string())
        })?;

        let bundle_hash = result
            .get("bundleHash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ClientError::ClientError("No bundleHash in response".to_string())
            })?;

        Ok(BundleResponse {
            bundle_hash: bundle_hash.to_string(),
        })
    }

    async fn simulate_bundle(
        &self,
        params: EthSendBundleParams,
    ) -> Result<HashMap<String, serde_json::Value>, ClientError> {
        if self.private_key.is_none() {
            return Err(ClientError::ClientError(
                "No Flashbots authentication key provided".to_string(),
            ));
        }

        // Create the request body
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_callBundle",
            "params": [params, "latest"] // Use latest block for simulation
        });

        // Calculate signature for Flashbots authentication
        let message = format!("{request_body}");
        let signature = self.sign_flashbots_request(message.as_bytes())?;

        // Send the request to Flashbots relay
        let response = self
            .http_client
            .post(FLASHBOTS_RELAY_URL)
            .header("X-Flashbots-Signature", signature)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| self.handle_flashbots_error(e))?;

        // Parse the response
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| self.handle_flashbots_error(e))?;

        if let Some(error) = response_json.get("error") {
            return Err(ClientError::ClientError(format!(
                "Flashbots error: {error}"
            )));
        }

        let result = response_json.get("result").ok_or_else(|| {
            ClientError::ClientError("No result in Flashbots response".to_string())
        })?;

        // Convert the simulation results to a HashMap
        let mut simulation_results = HashMap::new();

        if let Some(obj) = result.as_object() {
            for (key, value) in obj {
                simulation_results.insert(key.clone(), value.clone());
            }
        }

        Ok(simulation_results)
    }
}

//-----------------------------------------------------------------------------
// Error Conversion
//-----------------------------------------------------------------------------

impl From<crate::errors::EvmError> for ClientError {
    fn from(err: crate::errors::EvmError) -> Self {
        match err {
            crate::errors::EvmError::ConnectionError(msg) => {
                ClientError::ClientError(format!("Connection error: {msg}"))
            }
            crate::errors::EvmError::SerializationError(msg) => {
                ClientError::SerializationError(msg)
            }
            crate::errors::EvmError::TransactionError(msg) => {
                ClientError::TransactionError(msg)
            }
            crate::errors::EvmError::ContractError(msg) => {
                ClientError::ClientError(format!("Contract error: {msg}"))
            }
            crate::errors::EvmError::InsufficientBalance(msg) => {
                ClientError::ClientError(format!("Insufficient balance: {msg}"))
            }
            crate::errors::EvmError::InvalidParameter(msg) => {
                ClientError::ClientError(format!("Invalid parameter: {msg}"))
            }
            crate::errors::EvmError::ClientError(msg) => {
                ClientError::ClientError(msg)
            }
            crate::errors::EvmError::NotImplemented(msg) => {
                ClientError::NotImplemented(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_client_creation() {
        let client =
            EthereumClient::new("https://eth-mainnet.example.com", "", None);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.chain_id(), 1); // Ethereum mainnet
        assert!(!client.has_private_key());
    }

    #[test]
    fn test_with_flashbots_auth() {
        let client =
            EthereumClient::new("https://eth-mainnet.example.com", "", None)
                .unwrap();
        let auth_key = [0u8; 32];

        let client_with_auth = client.with_flashbots_auth(auth_key);
        assert!(client_with_auth.has_private_key());
    }
}
