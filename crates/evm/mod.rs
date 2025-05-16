//-----------------------------------------------------------------------------
// EVM Module - Ethereum Virtual Machine Client Implementations
//-----------------------------------------------------------------------------

//! EVM client implementations and types for interacting with Ethereum-compatible blockchains.
//!
//! This module provides a standardized interface for working with EVM-based chains through
//! the `EvmBaseClient` trait and chain-specific implementations.

// Core functionality
pub mod base_client;
pub mod generic_client;
pub mod types;
pub mod utils;

// Chain-specific implementations
pub mod chains;

// Error types
pub mod errors;

//-----------------------------------------------------------------------------
// Public Exports
//-----------------------------------------------------------------------------

// Re-export key types for easier access
pub use base_client::EvmBaseClient;
pub use types::{EvmAddress, EvmBytes, EvmHash, EvmLog, EvmTransactionReceipt, EvmTransactionRequest, EvmU256};
pub use errors::EvmError;

// Re-export chain implementations
pub use chains::ethereum::EthereumClient;
pub use chains::base::{BaseClient, BaseNetwork};

// Re-export client configurations
pub use generic_client::{EvmClientConfig, GenericEvmClient};
