//-----------------------------------------------------------------------------
// Generic EVM Client Implementation
//-----------------------------------------------------------------------------

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::base_client::EvmBaseClient;
use crate::types::{EvmAddress, EvmBytes, EvmHash, EvmTransactionRequest, EvmU256};
use valence_core::error::ClientError;
use valence_core::transaction::{Event, TransactionResponse};

//-----------------------------------------------------------------------------
// Configuration Types
//-----------------------------------------------------------------------------

/// Configuration for an EVM client
#[derive(Debug, Clone)]
pub struct EvmClientConfig {
    /// RPC URL for the EVM node
    pub rpc_url: String,
    /// Chain ID for the EVM network
    pub chain_id: u64,
    /// Gas price multiplier for fee estimation (1.0 = exact estimate)
    pub gas_adjustment: f64,
    /// Default gas limit if not specified
    pub default_gas_limit: u64,
    /// Maximum gas price to use (in gwei)
    pub max_gas_price_gwei: f64,
}

/// HTTP Provider for Ethereum JSON-RPC API
pub struct Provider {
    client: reqwest::Client,
    url: String,
}

//-----------------------------------------------------------------------------
// Generic EVM Client Implementation
//-----------------------------------------------------------------------------

/// A generic client for interacting with EVM-compatible blockchains
pub struct GenericEvmClient {
    config: EvmClientConfig,
    provider: Arc<Provider>,
}

impl GenericEvmClient {
    /// Create a new generic EVM client
    pub fn new(config: EvmClientConfig) -> Self {
        let client = reqwest::Client::new();
        let provider = Provider {
            client,
            url: config.rpc_url.clone(),
        };

        Self {
            config,
            provider: Arc::new(provider),
        }
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.provider.url
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.config.chain_id
    }

    /// Get the gas adjustment factor
    pub fn gas_adjustment(&self) -> f64 {
        self.config.gas_adjustment
    }

    /// Get the default gas limit
    pub fn default_gas_limit(&self) -> u64 {
        self.config.default_gas_limit
    }

    /// Convert gas price to gwei
    pub fn gas_price_to_gwei(&self, wei: EvmU256) -> f64 {
        // Simple implementation assuming wei is small enough to fit in u64
        wei.0[0] as f64 / 1_000_000_000.0
    }

    /// Sign a transaction (to be implemented by specific clients)
    pub async fn sign_transaction(
        &self,
        _tx: &EvmTransactionRequest,
    ) -> Result<EvmBytes, ClientError> {
        Err(ClientError::NotImplemented(
            "Signing not implemented in generic client".to_string(),
        ))
    }

    //-----------------------------------------------------------------------------
    // Helper Functions
    //-----------------------------------------------------------------------------

    /// Convert a receipt to a transaction response
    fn receipt_to_response(
        &self,
        receipt: Value,
    ) -> Result<TransactionResponse, ClientError> {
        let tx_hash = receipt["transactionHash"]
            .as_str()
            .ok_or_else(|| {
                ClientError::ParseError("Missing transaction hash".to_string())
            })?
            .to_string();

        let block_number = if let Some(block_num) = receipt["blockNumber"].as_str() {
            u64::from_str_radix(block_num.trim_start_matches("0x"), 16).map_err(
                |e| ClientError::ParseError(format!("Invalid block number: {e}")),
            )?
        } else {
            0
        };

        let gas_used = if let Some(gas) = receipt["gasUsed"].as_str() {
            i64::from_str_radix(gas.trim_start_matches("0x"), 16).map_err(|e| {
                ClientError::ParseError(format!("Invalid gas used: {e}"))
            })?
        } else {
            0
        };

        let block_hash = receipt["blockHash"].as_str().map(|s| s.to_string());

        let status = receipt["status"]
            .as_str()
            .map(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16))
            .transpose()
            .map_err(|e| ClientError::ParseError(format!("Invalid status: {e}")))?
            .unwrap_or(0);

        // Parse logs as events
        let mut events = Vec::new();
        if let Some(logs) = receipt["logs"].as_array() {
            for log in logs {
                if let Some(topics) = log["topics"].as_array() {
                    if let Some(first_topic) = topics.first() {
                        if let Some(first_topic_str) = first_topic.as_str() {
                            let event_type = first_topic_str.to_string();
                            events.push(Event {
                                event_type,
                                attributes: vec![],
                            });
                        }
                    }
                }
            }
        }

        Ok(TransactionResponse {
            tx_hash,
            height: block_number,
            gas_wanted: Some(gas_used),
            gas_used: Some(gas_used),
            code: Some(status),
            events,
            data: Some(serde_json::to_string(&receipt).unwrap_or_default()),
            raw_log: None,
            timestamp: Utc::now().timestamp(),
            block_hash,
            original_request_payload: None,
        })
    }
}

//-----------------------------------------------------------------------------
// Implementation of EvmBaseClient for GenericEvmClient
//-----------------------------------------------------------------------------

#[async_trait]
impl EvmBaseClient for GenericEvmClient {
    async fn get_balance(
        &self,
        address: &EvmAddress,
    ) -> Result<EvmU256, ClientError> {
        let params = json!([address.to_string(), "latest"]);

        let response: String = self
            .provider
            .call("eth_getBalance", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error getting balance: {e}"))
            })?;

        // Parse the hex string (simplified)
        let hex = response.trim_start_matches("0x");
        let value = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse balance: {e}"))
        })?;

        Ok(EvmU256::from_u64(value))
    }

    async fn get_nonce(&self, address: &EvmAddress) -> Result<u64, ClientError> {
        let params = json!([address.to_string(), "latest"]);

        let response: String = self
            .provider
            .call("eth_getTransactionCount", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error getting nonce: {e}"))
            })?;

        // Parse the hex string
        let hex = response.trim_start_matches("0x");
        let nonce = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse nonce: {e}"))
        })?;

        Ok(nonce)
    }

    async fn send_raw_transaction(
        &self,
        tx_bytes: &EvmBytes,
    ) -> Result<EvmHash, ClientError> {
        let params = json!([tx_bytes.to_hex()]);

        let response: String = self
            .provider
            .call("eth_sendRawTransaction", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error sending raw tx: {e}"))
            })?;

        // Parse the transaction hash
        response.parse().map_err(|e| {
            ClientError::ParseError(format!("Failed to parse transaction hash: {e}"))
        })
    }

    async fn send_transaction(
        &self,
        tx: &EvmTransactionRequest,
    ) -> Result<TransactionResponse, ClientError> {
        // Sign the transaction
        let signed_tx = self.sign_transaction(tx).await?;

        // Send the signed transaction
        let tx_hash = self.send_raw_transaction(&signed_tx).await?;

        // Wait for the transaction receipt
        self.wait_for_transaction_receipt(&tx_hash).await
    }

    async fn get_transaction(
        &self,
        tx_hash: &EvmHash,
    ) -> Result<Option<TransactionResponse>, ClientError> {
        let params = json!([tx_hash.to_string()]);

        let response: Value = self
            .provider
            .call("eth_getTransactionByHash", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error getting tx: {e}"))
            })?;

        if response.is_null() {
            return Ok(None);
        }

        // For transaction details, let's check if it has a receipt
        let receipt_params = json!([tx_hash.to_string()]);
        let receipt: Value = self
            .provider
            .call("eth_getTransactionReceipt", receipt_params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error getting receipt: {e}"))
            })?;

        if !receipt.is_null() {
            let response = self.receipt_to_response(receipt)?;
            return Ok(Some(response));
        }

        // If no receipt, construct a basic response
        let tx_response = TransactionResponse {
            tx_hash: tx_hash.to_string(),
            height: 0,
            gas_wanted: Some(0),
            gas_used: Some(0),
            code: Some(0),
            events: vec![],
            data: Some(serde_json::to_string(&response).unwrap_or_default()),
            raw_log: None,
            timestamp: Utc::now().timestamp(),
            block_hash: None,
            original_request_payload: None,
        };

        Ok(Some(tx_response))
    }

    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: &EvmHash,
    ) -> Result<TransactionResponse, ClientError> {
        // Simple polling implementation
        for _ in 0..30 {
            let params = json!([tx_hash.to_string()]);

            let receipt: Value = self
                .provider
                .call("eth_getTransactionReceipt", params)
                .await
                .map_err(|e| {
                    ClientError::ClientError(format!(
                        "Error waiting for receipt: {e}"
                    ))
                })?;

            if !receipt.is_null() {
                // Transaction has been mined
                return self.receipt_to_response(receipt);
            }

            // Wait before polling again
            sleep(Duration::from_secs(2)).await;
        }

        Err(ClientError::TimeoutError(
            "Transaction not mined within timeout".to_string(),
        ))
    }

    async fn get_block_number(&self) -> Result<u64, ClientError> {
        let response: String = self
            .provider
            .call("eth_blockNumber", json!([]))
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error getting block number: {e}"))
            })?;

        // Parse the hex string
        let hex = response.trim_start_matches("0x");
        let block_number = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse block number: {e}"))
        })?;

        Ok(block_number)
    }

    async fn get_chain_id(&self) -> Result<u64, ClientError> {
        // Use the stored chain ID if available
        if self.config.chain_id > 0 {
            return Ok(self.config.chain_id);
        }

        // Otherwise query from the node
        let response: String = self
            .provider
            .call("eth_chainId", json!([]))
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error getting chain ID: {e}"))
            })?;

        // Parse the hex string
        let hex = response.trim_start_matches("0x");
        let chain_id = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse chain ID: {e}"))
        })?;

        Ok(chain_id)
    }

    async fn get_gas_price(&self) -> Result<EvmU256, ClientError> {
        let response: String = self
            .provider
            .call("eth_gasPrice", json!([]))
            .await
            .map_err(|e| {
            ClientError::ClientError(format!("Error getting gas price: {e}"))
        })?;

        // Parse the hex string (simplified)
        let hex = response.trim_start_matches("0x");
        let value = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse gas price: {e}"))
        })?;

        Ok(EvmU256::from_u64(value))
    }

    async fn call_contract(
        &self,
        to: &EvmAddress,
        data: &EvmBytes,
        from: Option<&EvmAddress>,
        block: Option<u64>,
    ) -> Result<EvmBytes, ClientError> {
        let mut call_object = json!({
            "to": to.to_string(),
            "data": data.to_hex()
        });

        if let Some(from_addr) = from {
            call_object["from"] = json!(from_addr.to_string());
        }

        let block_param = match block {
            Some(num) => format!("0x{num:x}"),
            None => "latest".to_string(),
        };

        let params = json!([call_object, block_param]);

        let response: String =
            self.provider.call("eth_call", params).await.map_err(|e| {
                ClientError::ClientError(format!("Error calling contract: {e}"))
            })?;

        // Convert to EvmBytes
        EvmBytes::from_hex(&response).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse call result: {e}"))
        })
    }

    async fn estimate_gas(
        &self,
        to: Option<&EvmAddress>,
        data: &EvmBytes,
        value: Option<EvmU256>,
        from: Option<&EvmAddress>,
    ) -> Result<EvmU256, ClientError> {
        let mut call_object = json!({
            "data": data.to_hex()
        });

        if let Some(to_addr) = to {
            call_object["to"] = json!(to_addr.to_string());
        }

        if let Some(from_addr) = from {
            call_object["from"] = json!(from_addr.to_string());
        }

        if let Some(val) = value {
            call_object["value"] = json!(format!("0x{}", val.to_string()));
        }

        let params = json!([call_object]);

        let response: String = self
            .provider
            .call("eth_estimateGas", params)
            .await
            .map_err(|e| {
            ClientError::ClientError(format!("Error estimating gas: {e}"))
        })?;

        // Parse the hex string (simplified)
        let hex = response.trim_start_matches("0x");
        let value = u64::from_str_radix(hex, 16).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse gas estimate: {e}"))
        })?;

        // Apply gas adjustment
        let adjusted_value = (value as f64 * self.config.gas_adjustment) as u64;

        Ok(EvmU256::from_u64(adjusted_value))
    }

    fn evm_signer_address(&self) -> EvmAddress {
        // Since the generic client doesn't have a specific signer,
        // return a placeholder address or generate one dynamically
        // In a real implementation, this would return the actual signer address
        EvmAddress([0u8; 20]) // Zero address as a placeholder
    }
}

//-----------------------------------------------------------------------------
// Provider Implementation
//-----------------------------------------------------------------------------

impl Provider {
    /// Make a JSON-RPC call to the Ethereum node
    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, String> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Connection error: {e}"))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| format!("Serialization error: {e}"))?;

        if let Some(error) = response_json.get("error") {
            return Err(format!("RPC error: {error}"));
        }

        let result = response_json
            .get("result")
            .ok_or_else(|| "Missing 'result' field in response".to_string())?;

        serde_json::from_value(result.clone())
            .map_err(|e| format!("Deserialization error: {e}"))
    }
}
