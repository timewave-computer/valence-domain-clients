//-----------------------------------------------------------------------------
// Core Types and Utilities
//-----------------------------------------------------------------------------

//! Core types, errors, and utilities shared by all Valence Domain Clients.
//!
//! This crate provides fundamental abstractions for blockchain interactions
//! that are common across different blockchain ecosystems.

// Error handling
pub mod error;

// Transaction types and utilities
pub mod transaction;

// Common type definitions
pub mod types;

// Module re-exports
pub use error::ClientError;
pub use transaction::{Event, EventAttribute, TransactionResponse};
pub use types::{GenericAddress, GenericHash, GenericU256};
