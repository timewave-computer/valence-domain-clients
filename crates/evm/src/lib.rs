//-----------------------------------------------------------------------------
// Ethereum Virtual Machine (EVM) Client Implementation
//-----------------------------------------------------------------------------

//! Clients for interacting with Ethereum Virtual Machine (EVM) compatible blockchains.
//!
//! This crate provides implementations for various EVM-compatible chains including
//! Ethereum, Base, and more. It handles common operations like sending transactions,
//! querying balances, and interacting with smart contracts.

// Base client implementation
pub mod base_client;

// Bundled transaction support (Flashbots)
pub mod bundle;

// Chain-specific implementations
pub mod chains;

// Crypto utilities with abstraction layer
pub mod crypto;
pub mod crypto_adapter;
pub mod crypto_adapter_test;

// Error handling
pub mod errors;

// Generic client implementation
pub mod generic_client;

// Type definitions
pub mod types;

// Erigon tracing API support
#[cfg(feature = "erigon-tracing")]
pub mod tracing;

// Lodestar consensus client API support
#[cfg(feature = "lodestar-consensus")]
pub mod consensus;

// Utility functions
pub mod utils;

// Re-exports for easier access
pub use base_client::EvmBaseClient;
pub use bundle::{FlashbotsBundle, FlashbotsBundleOperations};
pub use generic_client::{EvmClientConfig, GenericEvmClient};
pub use types::{EvmAddress, EvmBytes, EvmHash, EvmU256};

// Erigon tracing re-exports
#[cfg(feature = "erigon-tracing")]
pub use tracing::ErigonTracing;
#[cfg(feature = "erigon-tracing")]
pub use types::{
    BlockTrace, CallTraceRequest, TraceAction, TraceFilter, TraceResult, TraceType,
    TransactionTrace, VmTrace,
};

// Lodestar consensus re-exports
#[cfg(feature = "lodestar-consensus")]
pub use consensus::{
    AttesterDuty, LodestarConsensus, NodePeer, NodeVersion, PeerDirection, PeerState, ProposerDuty,
    SyncDuty,
};
#[cfg(feature = "lodestar-consensus")]
pub use types::{
    Attestation, AttestationData, BeaconBlock, BeaconBlockBody, BeaconBlockHeader, BlsPublicKey,
    BlsSignature, Checkpoint, Committee, Epoch, FinalityCheckpoints, Fork, GenesisData,
    NodeIdentity, Root, Slot, SyncingStatus, Validator, ValidatorBalance, ValidatorIndex,
    ValidatorStatus,
};
