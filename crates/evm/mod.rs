//-----------------------------------------------------------------------------
// EVM Module
//-----------------------------------------------------------------------------

//! This module provides client implementations for Ethereum Virtual Machine (EVM)
//! compatible blockchains.
//!
//! It includes functionality for interacting with Ethereum mainnet as well as
//! various sidechains, L2s, and alternative EVM-compatible chains.

// Base client type re-exported at the crate level
pub use base_client::EvmBaseClient;

// Chain-specific errors and utils
pub mod errors;
pub mod utils;
pub mod types;
pub mod base_client;
pub mod generic_client;

// Crypto implementations with dependency isolation
mod crypto; // Original implementation - kept for transition
pub mod crypto_adapter; // New fully isolated implementation with type translation

#[cfg(test)]
mod crypto_adapter_test; // Tests for the crypto adapter

// Flashbots bundle support - available regardless of features
pub mod bundle;

// Chain-specific implementations
pub mod chains;

// Internal re-exports for convenience
pub use generic_client::{GenericEvmClient, EvmClientConfig};

//-----------------------------------------------------------------------------
// Public Exports
//-----------------------------------------------------------------------------

// Re-export key types for easier access
pub use types::{EvmAddress, EvmBytes, EvmHash, EvmLog, EvmTransactionReceipt, EvmTransactionRequest, EvmU256};
pub use errors::EvmError;

// Re-export chain implementations
pub use chains::ethereum::EthereumClient;
pub use chains::base::{BaseClient, BaseNetwork};

// Re-export the crypto adapter for public use
pub use crypto_adapter::{keccak256, sign_message, address_from_private_key, has_ethers_backend, get_active_backend_type};
