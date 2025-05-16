//-----------------------------------------------------------------------------
// Valence Domain Clients - Main Entry Point
//-----------------------------------------------------------------------------

//! Valence Domain Clients is a library for interacting with various blockchain networks.
//! 
//! This crate provides a standardized interface for performing common operations
//! across both Cosmos ecosystem and EVM-compatible blockchains, making it easier to build
//! applications that need to interact with multiple chains simultaneously.

// Core abstractions and shared functionality
pub mod core;

// Cosmos ecosystem clients
pub mod cosmos;

// Ethereum Virtual Machine (EVM) clients
pub mod evm;

// Protocol buffer definitions
pub mod proto;

//-----------------------------------------------------------------------------
// Convenience Re-exports
//-----------------------------------------------------------------------------

// Core types for easier access
pub use core::error::ClientError;
pub use core::transaction::{Event, EventAttribute, TransactionResponse};
pub use core::types::{GenericAddress, GenericHash, GenericU256};

// Cosmos-specific exports - Making traits available without feature flags
pub use cosmos::base_client::CosmosBaseClient;
pub use cosmos::grpc_client::GrpcSigningClient;

// Client implementations may remain behind feature flags if desired
#[cfg(feature = "cosmos")]
pub use cosmos::chains::babylon::BabylonClient;
#[cfg(feature = "cosmos")]
pub use cosmos::chains::gaia::GaiaClient;
#[cfg(feature = "cosmos")]
pub use cosmos::chains::neutron::NeutronClient;
#[cfg(feature = "cosmos")]
pub use cosmos::chains::noble::NobleClient;
#[cfg(feature = "cosmos")]
pub use cosmos::chains::osmosis::OsmosisClient;
#[cfg(feature = "cosmos")]
pub use cosmos::types::{CosmosCoin, CosmosFee, CosmosHeader};

// EVM-specific exports
pub use evm::base_client::EvmBaseClient;
