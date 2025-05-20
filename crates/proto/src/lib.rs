//-----------------------------------------------------------------------------
// Protocol Buffer Definitions and Utilities
//-----------------------------------------------------------------------------

//! Protocol buffer definitions and utilities for Valence Domain Clients.
//!
//! This crate provides the generated protocol buffer code and utilities
//! for working with protocol buffer messages across different blockchain
//! ecosystems.

// Protocol buffer utility functions
pub mod utils;

// Empty modules to ensure they exist even when proto files haven't been generated
// These are populated when running `nix run .#fetch-protos -- --all`
#[allow(missing_docs)]
pub mod noble {
    pub mod cctp {}
    pub mod fiattokenfactory {}
    pub mod tokenfactory {}
}

#[allow(missing_docs)]
pub mod osmosis {
    pub mod concentratedliquidity {}
    pub mod gamm {}
    pub mod poolmanager {}
    pub mod superfluid {}
    pub mod tokenfactory {}
}

// Re-exports for easier access
pub use utils::*;
