//-----------------------------------------------------------------------------
// Valence Domain Clients - Main Entry Point
//-----------------------------------------------------------------------------

//! Valence Domain Clients is a library for interacting with various blockchain networks.
//!
//! This crate provides a standardized interface for performing common operations
//! across both Cosmos ecosystem and EVM-compatible blockchains, making it easier to build
//! applications that need to interact with multiple chains simultaneously.

// Re-export core functionality
pub use valence_core as core;

// Re-export Cosmos functionality
pub use valence_cosmos as cosmos;

// Re-export EVM functionality
pub use valence_evm as evm;

// Re-export Protocol buffer definitions
pub use valence_proto as proto;
// Re-export specific proto types for convenience
pub use valence_proto::utils::{ProtoDecodable, ProtoEncodable, ProtoError};

//-----------------------------------------------------------------------------
// Convenience Re-exports
//-----------------------------------------------------------------------------

// Core types for easier access
pub use valence_core::error::ClientError;
pub use valence_core::transaction::{Event, EventAttribute, TransactionResponse};
pub use valence_core::types::{GenericAddress, GenericHash, GenericU256};

// Cosmos-specific exports
pub use valence_cosmos::base_client::CosmosBaseClient;
pub use valence_cosmos::grpc_client::GrpcSigningClient;

// Client implementations for Cosmos chains
#[cfg(feature = "babylon")]
pub use valence_cosmos::chains::babylon::BabylonClient;
#[cfg(feature = "gaia")]
pub use valence_cosmos::chains::gaia::GaiaClient;
#[cfg(feature = "neutron")]
pub use valence_cosmos::chains::neutron::NeutronClient;
#[cfg(feature = "noble")]
pub use valence_cosmos::chains::noble::NobleClient;
#[cfg(feature = "osmosis")]
pub use valence_cosmos::chains::osmosis::OsmosisClient;
pub use valence_cosmos::types::{CosmosCoin, CosmosFee, CosmosHeader};

// EVM-specific exports
pub use valence_evm::base_client::EvmBaseClient;

// Flashbots bundle functionality
pub use valence_evm::bundle::{FlashbotsBundle, FlashbotsBundleOperations};

// Client implementations for EVM chains
#[cfg(feature = "base")]
pub use valence_evm::chains::base::{BaseClient, BaseNetwork};
#[cfg(feature = "ethereum")]
pub use valence_evm::chains::ethereum::EthereumClient;
