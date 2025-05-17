//-----------------------------------------------------------------------------
// Cosmos Ecosystem Client Implementation
//-----------------------------------------------------------------------------

//! Clients for interacting with Cosmos ecosystem blockchains.
//!
//! This crate provides implementations for various Cosmos-based chains including
//! Noble, Osmosis, Gaia (Cosmos Hub), Neutron, and Babylon. It handles common operations
//! like sending transactions, querying balances, and staking operations.

// Base client interfaces
pub mod base_client;

// Chain-specific implementations
pub mod chains;

// Error handling
pub mod errors;

// Generic client implementation
pub mod generic_client;

// GRPC client for Cosmos chain interactions
pub mod grpc_client;

// Protocol buffer timestamp conversion utilities
pub mod proto_timestamp;

// Signing client for handling transaction signatures
pub mod signing_client;

// Type definitions
pub mod types;

// Utility functions
pub mod utils;

// Re-exports for easier access
pub use base_client::CosmosBaseClient;
pub use errors::CosmosError;
pub use generic_client::{CosmosClientConfig, GenericCosmosClient};
pub use grpc_client::GrpcSigningClient;
pub use types::{CosmosCoin, CosmosFee, CosmosHeader};
