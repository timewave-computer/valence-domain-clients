//-----------------------------------------------------------------------------
// Core Module - Shared functionality across clients
//-----------------------------------------------------------------------------

//! Core abstractions and shared functionality for all domain clients.
//! 
//! This module contains the foundational types, error definitions, and
//! transaction handling capabilities used across both EVM and Cosmos ecosystems.

pub mod error;
pub mod types;
pub mod transaction;

// Re-export commonly used types for convenience
pub use error::ClientError;
pub use types::*;
pub use transaction::*;
