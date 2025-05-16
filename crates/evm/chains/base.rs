//-----------------------------------------------------------------------------
// Base Client Implementation
//-----------------------------------------------------------------------------

//! Client implementation for Base blockchain, an Ethereum L2 chain.
//!
//! This module provides the `BaseClient` for interacting with Base mainnet and testnet.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

use crate::core::error::ClientError;
use crate::evm::base_client::EvmBaseClient;
use crate::evm::generic_client::{EvmClientConfig, GenericEvmClient};
use crate::evm::types::EvmAddress;

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
#[derive(Debug, Clone)]
pub struct BaseClient {
    inner: GenericEvmClient,
    network: BaseNetwork,
}

impl BaseClient {
    /// Create a new Base client with the specified network and private key
    pub fn new(
        network: BaseNetwork,
        private_key: Option<&str>,
    ) -> Result<Self, ClientError> {
        let config = EvmClientConfig {
            chain_id: network.chain_id(),
            rpc_url: network.rpc_url().to_string(),
            private_key: private_key.map(String::from),
        };

        let inner = GenericEvmClient::new(config)?;

        Ok(Self {
            inner,
            network,
        })
    }

    /// Create a new Base client with the specified network and private key from a local configuration file
    pub fn from_config_file(
        network: BaseNetwork,
        private_key: Option<&str>,
        config_path: Option<&Path>,
    ) -> Result<Self, ClientError> {
        // Try to load configuration from file if provided
        if let Some(path) = config_path {
            if path.exists() {
                let config_str = fs::read_to_string(path)
                    .map_err(|e| ClientError::ConfigurationError(format!("Failed to read config file: {}", e)))?;
                
                let config: BaseNetworkConfig = serde_json::from_str(&config_str)
                    .map_err(|e| ClientError::ConfigurationError(format!("Failed to parse config file: {}", e)))?;
                
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
                    private_key: private_key.map(String::from),
                };
                
                let inner = GenericEvmClient::new(config)?;
                
                return Ok(Self {
                    inner,
                    network,
                });
            }
        }
        
        // Fall back to default configuration
        Self::new(network, private_key)
    }

    /// Get the network this client is connected to
    pub fn network(&self) -> BaseNetwork {
        self.network
    }

    /// Get the chain ID for the current network
    pub fn chain_id(&self) -> u64 {
        self.network.chain_id()
    }

    /// Get the block explorer URL for the current network
    pub fn explorer_url(&self) -> &'static str {
        self.network.explorer_url()
    }
}

/// Implement EvmBaseClient for BaseClient by delegating to the inner GenericEvmClient
impl EvmBaseClient for BaseClient {
    fn chain_id(&self) -> u64 {
        self.inner.chain_id()
    }

    fn client(&self) -> Arc<alloy_transport_http::Http> {
        self.inner.client()
    }

    fn signer(&self) -> Option<Arc<dyn alloy_signer::Signer + Send + Sync>> {
        self.inner.signer()
    }

    fn signer_address(&self) -> Option<EvmAddress> {
        self.inner.signer_address()
    }
} 