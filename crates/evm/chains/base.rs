//-----------------------------------------------------------------------------
// Base Client Implementation
//-----------------------------------------------------------------------------

//! Client implementation for Base blockchain, an Ethereum L2 chain.
//!
//! This module provides the `BaseClient` for interacting with Base mainnet and testnet.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

use crate::core::error::ClientError;
use crate::core::transaction::TransactionResponse;
use crate::evm::base_client::EvmBaseClient;
use crate::evm::generic_client::{EvmClientConfig, GenericEvmClient};
use crate::evm::types::{EvmAddress, EvmBytes, EvmHash, EvmTransactionRequest, EvmU256};

/// Network options for Base blockchain
#[derive(Debug, Clone, Copy)]
pub enum BaseNetwork {
    /// Base Mainnet
    Mainnet,
    /// Base Sepolia Testnet
    Sepolia,
}

impl BaseNetwork {
    /// Get the chain ID for the Base network
    pub fn chain_id(&self) -> u64 {
        match self {
            BaseNetwork::Mainnet => 8453,
            BaseNetwork::Sepolia => 84532,
        }
    }

    /// Get the RPC URL for the Base network
    pub fn rpc_url(&self) -> &'static str {
        match self {
            BaseNetwork::Mainnet => "https://mainnet.base.org",
            BaseNetwork::Sepolia => "https://sepolia.base.org",
        }
    }

    /// Get the block explorer URL for the Base network
    pub fn explorer_url(&self) -> &'static str {
        match self {
            BaseNetwork::Mainnet => "https://base.blockscout.com/",
            BaseNetwork::Sepolia => "https://sepolia-explorer.base.org",
        }
    }
}

/// Base network configuration loaded from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseNetworkConfig {
    pub networks: BaseNetworkData,
}

/// Base network data for serialization/deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseNetworkData {
    pub mainnet: BaseNetworkInfo,
    pub sepolia: BaseNetworkInfo,
}

/// Base network information for a specific network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseNetworkInfo {
    pub rpc: String,
    pub chain_id: u64,
    pub currency_symbol: String,
    pub block_explorer: String,
}

/// Base client for interacting with the Base blockchain
pub struct BaseClient {
    inner: GenericEvmClient,
    network: BaseNetwork,
}

impl BaseClient {
    /// Create a new Base client with the specified network and private key
    pub fn new(
        network: BaseNetwork,
        _private_key: Option<&str>,
    ) -> Result<Self, ClientError> {
        let config = EvmClientConfig {
            chain_id: network.chain_id(),
            rpc_url: network.rpc_url().to_string(),
            gas_adjustment: 1.3, // Add 30% to gas estimates
            default_gas_limit: 1_000_000, // Default gas limit
            max_gas_price_gwei: 100.0, // Maximum 100 gwei for gas price
        };

        let inner = GenericEvmClient::new(config);

        Ok(Self {
            inner,
            network,
        })
    }

    /// Create a new Base client with the specified network and private key from a local configuration file
    pub fn from_config_file(
        network: BaseNetwork,
        _private_key: Option<&str>,
        config_path: Option<&Path>,
    ) -> Result<Self, ClientError> {
        // Try to load configuration from file if provided
        if let Some(path) = config_path {
            if path.exists() {
                let config_str = fs::read_to_string(path)
                    .map_err(|e| ClientError::ClientError(format!("Failed to read config file: {e}")))?;
                
                let config: BaseNetworkConfig = serde_json::from_str(&config_str)
                    .map_err(|e| ClientError::ClientError(format!("Failed to parse config file: {e}")))?;
                
                let (rpc_url, chain_id) = match network {
                    BaseNetwork::Mainnet => (
                        config.networks.mainnet.rpc,
                        config.networks.mainnet.chain_id,
                    ),
                    BaseNetwork::Sepolia => (
                        config.networks.sepolia.rpc,
                        config.networks.sepolia.chain_id,
                    ),
                };
                
                let config = EvmClientConfig {
                    chain_id,
                    rpc_url,
                    gas_adjustment: 1.3, // Add 30% to gas estimates
                    default_gas_limit: 1_000_000, // Default gas limit
                    max_gas_price_gwei: 100.0, // Maximum 100 gwei for gas price
                };
                
                let inner = GenericEvmClient::new(config);
                
                return Ok(Self {
                    inner,
                    network,
                });
            }
        }
        
        // Fall back to default configuration
        Self::new(network, _private_key)
    }

    /// Get the network this client is connected to
    pub fn network(&self) -> BaseNetwork {
        self.network
    }

    /// Get the chain ID for the current network
    pub fn network_chain_id(&self) -> u64 {
        self.network.chain_id()
    }

    /// Get the block explorer URL for the current network
    pub fn explorer_url(&self) -> &'static str {
        self.network.explorer_url()
    }
}

/// Implement EvmBaseClient for BaseClient by delegating to the inner GenericEvmClient
#[async_trait]
impl EvmBaseClient for BaseClient {
    fn evm_signer_address(&self) -> EvmAddress {
        self.inner.evm_signer_address()
    }

    async fn get_balance(&self, address: &EvmAddress) -> Result<EvmU256, ClientError> {
        self.inner.get_balance(address).await
    }

    async fn get_nonce(&self, address: &EvmAddress) -> Result<u64, ClientError> {
        self.inner.get_nonce(address).await
    }

    async fn send_raw_transaction(&self, tx_bytes: &EvmBytes) -> Result<EvmHash, ClientError> {
        self.inner.send_raw_transaction(tx_bytes).await
    }

    async fn send_transaction(&self, tx: &EvmTransactionRequest) -> Result<TransactionResponse, ClientError> {
        self.inner.send_transaction(tx).await
    }

    async fn get_transaction(&self, tx_hash: &EvmHash) -> Result<Option<TransactionResponse>, ClientError> {
        self.inner.get_transaction(tx_hash).await
    }

    async fn wait_for_transaction_receipt(&self, tx_hash: &EvmHash) -> Result<TransactionResponse, ClientError> {
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