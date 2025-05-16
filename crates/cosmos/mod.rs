//-----------------------------------------------------------------------------
// Cosmos Module - Cosmos Ecosystem Client Implementations
//-----------------------------------------------------------------------------

//! Cosmos client implementations and types for interacting with Cosmos-based blockchains.
//!
//! This module provides a standardized interface for working with Cosmos-based chains through
//! the base clients and chain-specific implementations.

// Core functionality
pub mod base_client;
pub mod generic_client;
pub mod types;
pub mod utils;

// Chain-specific implementations
pub mod chains;

// Internal implementation details
pub mod signing_client;
pub mod grpc_client;
pub(crate) mod proto_timestamp;

// Error types
pub mod errors;

//-----------------------------------------------------------------------------
// Public Exports
//-----------------------------------------------------------------------------

// Re-export key types for easier access
pub use base_client::CosmosBaseClient;
pub use types::*;
pub use chains::*;

// Client type aliases for internal use
#[allow(dead_code)]
pub(crate) type WasmQueryClient<T> =
    cosmrs::proto::cosmwasm::wasm::v1::query_client::QueryClient<T>;
pub(crate) type AuthQueryClient<T> =
    cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient<T>;
