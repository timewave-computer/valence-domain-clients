//-----------------------------------------------------------------------------
// EVM Base Client Interface
//-----------------------------------------------------------------------------

//! Base client interface for EVM-compatible blockchains.
//!
//! This module defines the core trait that all EVM clients must implement.

use async_trait::async_trait;

// Use the new workspace crates instead of internal modules
use valence_core::error::ClientError;
use valence_core::transaction::TransactionResponse;

// Local types
use crate::types::{EvmAddress, EvmBytes, EvmHash, EvmTransactionRequest, EvmU256};

/// Base client trait for Ethereum/EVM chains.
///
/// This trait defines the minimum functionality that must be implemented by
/// any client for an EVM-compatible blockchain.
#[async_trait]
pub trait EvmBaseClient {
    /// Get the signer's Ethereum address
    fn evm_signer_address(&self) -> EvmAddress;

    /// Get balance for an address
    async fn get_balance(
        &self,
        address: &EvmAddress,
    ) -> Result<EvmU256, ClientError>;

    /// Get the nonce for an address
    async fn get_nonce(&self, address: &EvmAddress) -> Result<u64, ClientError>;

    /// Send a raw transaction directly
    async fn send_raw_transaction(
        &self,
        tx_bytes: &EvmBytes,
    ) -> Result<EvmHash, ClientError>;

    /// Send a transaction
    async fn send_transaction(
        &self,
        tx: &EvmTransactionRequest,
    ) -> Result<TransactionResponse, ClientError>;

    /// Get transaction by hash
    async fn get_transaction(
        &self,
        tx_hash: &EvmHash,
    ) -> Result<Option<TransactionResponse>, ClientError>;

    /// Wait for a transaction receipt
    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: &EvmHash,
    ) -> Result<TransactionResponse, ClientError>;

    /// Get the current block number
    async fn get_block_number(&self) -> Result<u64, ClientError>;

    /// Get the chain ID
    async fn get_chain_id(&self) -> Result<u64, ClientError>;

    /// Get the current gas price
    async fn get_gas_price(&self) -> Result<EvmU256, ClientError>;

    /// Call a contract without sending a transaction
    async fn call_contract(
        &self,
        to: &EvmAddress,
        data: &EvmBytes,
        from: Option<&EvmAddress>,
        block: Option<u64>,
    ) -> Result<EvmBytes, ClientError>;

    /// Estimate gas for a transaction
    async fn estimate_gas(
        &self,
        to: Option<&EvmAddress>,
        data: &EvmBytes,
        value: Option<EvmU256>,
        from: Option<&EvmAddress>,
    ) -> Result<EvmU256, ClientError>;
}
