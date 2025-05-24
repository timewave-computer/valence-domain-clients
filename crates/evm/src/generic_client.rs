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
// Erigon Tracing Implementation
//-----------------------------------------------------------------------------

#[cfg(feature = "erigon-tracing")]
use crate::tracing::ErigonTracing;
#[cfg(feature = "erigon-tracing")]
use crate::types::{
    BlockTrace, CallTraceRequest, TraceFilter, TraceType, TransactionTrace,
};

#[cfg(feature = "erigon-tracing")]
#[async_trait::async_trait]
impl ErigonTracing for GenericEvmClient {
    async fn trace_transaction(
        &self,
        tx_hash: &EvmHash,
        trace_types: Vec<TraceType>,
    ) -> Result<TransactionTrace, ClientError> {
        let params = json!([tx_hash.to_string(), trace_types]);

        let response: TransactionTrace = self
            .provider
            .call("trace_transaction", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error tracing transaction: {e}"))
            })?;

        Ok(response)
    }

    async fn trace_block(
        &self,
        block_number: u64,
        trace_types: Vec<TraceType>,
    ) -> Result<BlockTrace, ClientError> {
        let block_param = format!("0x{block_number:x}");
        let params = json!([block_param, trace_types]);

        let response: BlockTrace = self
            .provider
            .call("trace_block", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error tracing block: {e}"))
            })?;

        Ok(response)
    }

    async fn trace_filter(
        &self,
        filter: &TraceFilter,
    ) -> Result<Vec<TransactionTrace>, ClientError> {
        let params = json!([filter]);

        let response: Vec<TransactionTrace> = self
            .provider
            .call("trace_filter", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error filtering traces: {e}"))
            })?;

        Ok(response)
    }

    async fn trace_call(
        &self,
        call_request: &CallTraceRequest,
        trace_types: Vec<TraceType>,
        block_number: Option<u64>,
    ) -> Result<TransactionTrace, ClientError> {
        let block_param = match block_number {
            Some(num) => format!("0x{num:x}"),
            None => "latest".to_string(),
        };

        let params = json!([call_request, trace_types, block_param]);

        let response: TransactionTrace = self
            .provider
            .call("trace_call", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error tracing call: {e}"))
            })?;

        Ok(response)
    }

    async fn trace_call_many(
        &self,
        call_requests: &[CallTraceRequest],
        trace_types: Vec<TraceType>,
        block_number: Option<u64>,
    ) -> Result<Vec<TransactionTrace>, ClientError> {
        let block_param = match block_number {
            Some(num) => format!("0x{num:x}"),
            None => "latest".to_string(),
        };

        let calls_with_types: Vec<_> = call_requests
            .iter()
            .map(|req| json!([req, trace_types.clone()]))
            .collect();

        let params = json!([calls_with_types, block_param]);

        let response: Vec<TransactionTrace> = self
            .provider
            .call("trace_callMany", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error tracing multiple calls: {e}"))
            })?;

        Ok(response)
    }

    async fn trace_raw_transaction(
        &self,
        raw_tx: &[u8],
        trace_types: Vec<TraceType>,
        block_number: Option<u64>,
    ) -> Result<TransactionTrace, ClientError> {
        let raw_tx_hex = format!("0x{}", hex::encode(raw_tx));
        let block_param = match block_number {
            Some(num) => format!("0x{num:x}"),
            None => "latest".to_string(),
        };

        let params = json!([raw_tx_hex, trace_types, block_param]);

        let response: TransactionTrace = self
            .provider
            .call("trace_rawTransaction", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error tracing raw transaction: {e}"))
            })?;

        Ok(response)
    }

    async fn trace_replay_block_transactions(
        &self,
        block_number: u64,
        trace_types: Vec<TraceType>,
    ) -> Result<Vec<TransactionTrace>, ClientError> {
        let block_param = format!("0x{block_number:x}");
        let params = json!([block_param, trace_types]);

        let response: Vec<TransactionTrace> = self
            .provider
            .call("trace_replayBlockTransactions", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!(
                    "Error replaying block transactions: {e}"
                ))
            })?;

        Ok(response)
    }

    async fn trace_replay_transaction(
        &self,
        tx_hash: &EvmHash,
        trace_types: Vec<TraceType>,
    ) -> Result<TransactionTrace, ClientError> {
        let params = json!([tx_hash.to_string(), trace_types]);

        let response: TransactionTrace = self
            .provider
            .call("trace_replayTransaction", params)
            .await
            .map_err(|e| {
                ClientError::ClientError(format!("Error replaying transaction: {e}"))
            })?;

        Ok(response)
    }
}

//-----------------------------------------------------------------------------
// Lodestar Consensus Implementation
//-----------------------------------------------------------------------------

#[cfg(feature = "lodestar-consensus")]
use crate::consensus::{
    AttesterDuty, LodestarConsensus, NodePeer, NodeVersion, ProposerDuty, SyncDuty,
};
#[cfg(feature = "lodestar-consensus")]
use crate::types::{
    Attestation, BeaconBlock, BeaconBlockHeader, Committee, Epoch, FinalityCheckpoints, Fork,
    GenesisData, NodeIdentity, Root, Slot, SyncingStatus, Validator, ValidatorBalance,
    ValidatorIndex, ValidatorStatus,
};

#[cfg(feature = "lodestar-consensus")]
#[async_trait::async_trait]
impl LodestarConsensus for GenericEvmClient {
    async fn get_genesis(&self) -> Result<GenesisData, ClientError> {
        // Call Lodestar REST API
        let url = format!("{}/eth/v1/beacon/genesis", self.rpc_url());
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get genesis: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse genesis response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing 'data' in genesis response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize genesis: {e}")))
    }

    async fn get_state_root(&self, state_id: &str) -> Result<Root, ClientError> {
        let url = format!("{}/eth/v1/beacon/states/{}/root", self.rpc_url(), state_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get state root: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse state root response: {e}")))?;

        let root_str = response_json
            .get("data")
            .and_then(|d| d.get("root"))
            .and_then(|r| r.as_str())
            .ok_or_else(|| ClientError::ParseError("Missing root in response".to_string()))?;

        root_str.parse()
            .map_err(|e| ClientError::ParseError(format!("Failed to parse root: {e}")))
    }

    async fn get_state_fork(&self, state_id: &str) -> Result<Fork, ClientError> {
        let url = format!("{}/eth/v1/beacon/states/{}/fork", self.rpc_url(), state_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get fork: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse fork response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing 'data' in fork response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize fork: {e}")))
    }

    async fn get_state_finality_checkpoints(
        &self,
        state_id: &str,
    ) -> Result<FinalityCheckpoints, ClientError> {
        let url = format!("{}/eth/v1/beacon/states/{}/finality_checkpoints", self.rpc_url(), state_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get finality checkpoints: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse finality response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing 'data' in finality response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize finality checkpoints: {e}")))
    }

    async fn get_beacon_block(&self, block_id: &str) -> Result<BeaconBlock, ClientError> {
        let url = format!("{}/eth/v2/beacon/blocks/{}", self.rpc_url(), block_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get beacon block: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse block response: {e}")))?;

        let data = response_json
            .get("data")
            .and_then(|d| d.get("message"))
            .ok_or_else(|| ClientError::ParseError("Missing block data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize beacon block: {e}")))
    }

    async fn get_beacon_block_header(
        &self,
        block_id: &str,
    ) -> Result<BeaconBlockHeader, ClientError> {
        let url = format!("{}/eth/v1/beacon/headers/{}", self.rpc_url(), block_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get block header: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse header response: {e}")))?;

        let data = response_json
            .get("data")
            .and_then(|d| d.get("header"))
            .and_then(|h| h.get("message"))
            .ok_or_else(|| ClientError::ParseError("Missing header data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize block header: {e}")))
    }

    async fn get_beacon_block_headers(
        &self,
        slot: Option<Slot>,
        parent_root: Option<&Root>,
    ) -> Result<Vec<BeaconBlockHeader>, ClientError> {
        let mut url = format!("{}/eth/v1/beacon/headers", self.rpc_url());
        let mut params = Vec::new();

        if let Some(s) = slot {
            params.push(format!("slot={s}"));
        }
        if let Some(root) = parent_root {
            params.push(format!("parent_root={root}"));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get block headers: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse headers response: {e}")))?;

        let data = response_json
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ClientError::ParseError("Missing headers data in response".to_string()))?;

        let mut headers = Vec::new();
        for header_data in data {
            let header = header_data
                .get("header")
                .and_then(|h| h.get("message"))
                .ok_or_else(|| ClientError::ParseError("Missing header in response".to_string()))?;

            let beacon_header: BeaconBlockHeader = serde_json::from_value(header.clone())
                .map_err(|e| ClientError::ParseError(format!("Failed to deserialize header: {e}")))?;
            headers.push(beacon_header);
        }

        Ok(headers)
    }

    async fn get_block_root(&self, block_id: &str) -> Result<Root, ClientError> {
        let url = format!("{}/eth/v1/beacon/blocks/{}/root", self.rpc_url(), block_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get block root: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse block root response: {e}")))?;

        let root_str = response_json
            .get("data")
            .and_then(|d| d.get("root"))
            .and_then(|r| r.as_str())
            .ok_or_else(|| ClientError::ParseError("Missing root in response".to_string()))?;

        root_str.parse()
            .map_err(|e| ClientError::ParseError(format!("Failed to parse root: {e}")))
    }

    async fn get_block_attestations(
        &self,
        block_id: &str,
    ) -> Result<Vec<Attestation>, ClientError> {
        let url = format!("{}/eth/v1/beacon/blocks/{}/attestations", self.rpc_url(), block_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get attestations: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse attestations response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing attestations data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize attestations: {e}")))
    }

    async fn get_pending_attestations(
        &self,
        slot: Option<Slot>,
        committee_index: Option<u64>,
    ) -> Result<Vec<Attestation>, ClientError> {
        let mut url = format!("{}/eth/v1/beacon/pool/attestations", self.rpc_url());
        let mut params = Vec::new();

        if let Some(s) = slot {
            params.push(format!("slot={s}"));
        }
        if let Some(idx) = committee_index {
            params.push(format!("committee_index={idx}"));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get pending attestations: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse pending attestations response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing attestations data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize pending attestations: {e}")))
    }

    async fn submit_attestations(
        &self,
        attestations: &[Attestation],
    ) -> Result<(), ClientError> {
        let url = format!("{}/eth/v1/beacon/pool/attestations", self.rpc_url());
        let response = self
            .provider
            .client
            .post(&url)
            .json(attestations)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to submit attestations: {e}")))?;

        if !response.status().is_success() {
            return Err(ClientError::ClientError(format!(
                "Failed to submit attestations: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn submit_block(&self, block: &BeaconBlock) -> Result<(), ClientError> {
        let url = format!("{}/eth/v1/beacon/blocks", self.rpc_url());
        let response = self
            .provider
            .client
            .post(&url)
            .json(block)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to submit block: {e}")))?;

        if !response.status().is_success() {
            return Err(ClientError::ClientError(format!(
                "Failed to submit block: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn get_validator(
        &self,
        state_id: &str,
        validator_id: &str,
    ) -> Result<Validator, ClientError> {
        let url = format!("{}/eth/v1/beacon/states/{}/validators/{}", self.rpc_url(), state_id, validator_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get validator: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse validator response: {e}")))?;

        let data = response_json
            .get("data")
            .and_then(|d| d.get("validator"))
            .ok_or_else(|| ClientError::ParseError("Missing validator data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize validator: {e}")))
    }

    async fn get_validators(
        &self,
        state_id: &str,
        validator_ids: &[String],
        status_filter: Option<&[ValidatorStatus]>,
    ) -> Result<Vec<Validator>, ClientError> {
        let mut url = format!("{}/eth/v1/beacon/states/{}/validators", self.rpc_url(), state_id);
        let mut params = Vec::new();

        if !validator_ids.is_empty() {
            params.push(format!("id={}", validator_ids.join(",")));
        }

        if let Some(statuses) = status_filter {
            let status_strings: Vec<String> = statuses.iter()
                .map(|s| serde_json::to_string(s).unwrap_or_default().trim_matches('"').to_string())
                .collect();
            params.push(format!("status={}", status_strings.join(",")));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get validators: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse validators response: {e}")))?;

        let data = response_json
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ClientError::ParseError("Missing validators data in response".to_string()))?;

        let mut validators = Vec::new();
        for validator_data in data {
            let validator = validator_data
                .get("validator")
                .ok_or_else(|| ClientError::ParseError("Missing validator in response".to_string()))?;

            let v: Validator = serde_json::from_value(validator.clone())
                .map_err(|e| ClientError::ParseError(format!("Failed to deserialize validator: {e}")))?;
            validators.push(v);
        }

        Ok(validators)
    }

    async fn get_validator_balances(
        &self,
        state_id: &str,
        validator_ids: &[String],
    ) -> Result<Vec<ValidatorBalance>, ClientError> {
        let mut url = format!("{}/eth/v1/beacon/states/{}/validator_balances", self.rpc_url(), state_id);

        if !validator_ids.is_empty() {
            url.push('?');
            url.push_str(&format!("id={}", validator_ids.join(",")));
        }

        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get validator balances: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse balances response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing balances data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize validator balances: {e}")))
    }

    async fn get_epoch_committees(
        &self,
        state_id: &str,
        epoch: Option<Epoch>,
        index: Option<u64>,
        slot: Option<Slot>,
    ) -> Result<Vec<Committee>, ClientError> {
        let mut url = format!("{}/eth/v1/beacon/states/{}/committees", self.rpc_url(), state_id);
        let mut params = Vec::new();

        if let Some(e) = epoch {
            params.push(format!("epoch={e}"));
        }
        if let Some(i) = index {
            params.push(format!("index={i}"));
        }
        if let Some(s) = slot {
            params.push(format!("slot={s}"));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get committees: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse committees response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing committees data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize committees: {e}")))
    }

    async fn get_attester_duties(
        &self,
        epoch: Epoch,
        validator_indices: &[ValidatorIndex],
    ) -> Result<Vec<AttesterDuty>, ClientError> {
        let url = format!("{}/eth/v1/validator/duties/attester/{}", self.rpc_url(), epoch);
        let response = self
            .provider
            .client
            .post(&url)
            .json(validator_indices)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get attester duties: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse duties response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing duties data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize attester duties: {e}")))
    }

    async fn get_proposer_duties(&self, epoch: Epoch) -> Result<Vec<ProposerDuty>, ClientError> {
        let url = format!("{}/eth/v1/validator/duties/proposer/{}", self.rpc_url(), epoch);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get proposer duties: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse duties response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing duties data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize proposer duties: {e}")))
    }

    async fn get_sync_duties(
        &self,
        epoch: Epoch,
        validator_indices: &[ValidatorIndex],
    ) -> Result<Vec<SyncDuty>, ClientError> {
        let url = format!("{}/eth/v1/validator/duties/sync/{}", self.rpc_url(), epoch);
        let response = self
            .provider
            .client
            .post(&url)
            .json(validator_indices)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get sync duties: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse duties response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing duties data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize sync duties: {e}")))
    }

    async fn get_node_identity(&self) -> Result<NodeIdentity, ClientError> {
        let url = format!("{}/eth/v1/node/identity", self.rpc_url());
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get node identity: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse identity response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing identity data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize node identity: {e}")))
    }

    async fn get_node_peers(&self) -> Result<Vec<NodePeer>, ClientError> {
        let url = format!("{}/eth/v1/node/peers", self.rpc_url());
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get node peers: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse peers response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing peers data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize node peers: {e}")))
    }

    async fn get_sync_status(&self) -> Result<SyncingStatus, ClientError> {
        let url = format!("{}/eth/v1/node/syncing", self.rpc_url());
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get sync status: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse sync response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing sync data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize sync status: {e}")))
    }

    async fn get_node_version(&self) -> Result<NodeVersion, ClientError> {
        let url = format!("{}/eth/v1/node/version", self.rpc_url());
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get node version: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse version response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing version data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize node version: {e}")))
    }

    async fn get_debug_beacon_state(&self, state_id: &str) -> Result<serde_json::Value, ClientError> {
        let url = format!("{}/eth/v1/debug/beacon/states/{}", self.rpc_url(), state_id);
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get debug state: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse debug state response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing debug state data in response".to_string()))?;

        Ok(data.clone())
    }

    async fn get_debug_beacon_heads(&self) -> Result<Vec<BeaconBlockHeader>, ClientError> {
        let url = format!("{}/eth/v1/debug/beacon/heads", self.rpc_url());
        let response = self
            .provider
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::ClientError(format!("Failed to get debug heads: {e}")))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse debug heads response: {e}")))?;

        let data = response_json
            .get("data")
            .ok_or_else(|| ClientError::ParseError("Missing debug heads data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize debug heads: {e}")))
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
