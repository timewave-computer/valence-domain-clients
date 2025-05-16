//-----------------------------------------------------------------------------
// EVM Base Client 
//-----------------------------------------------------------------------------

use async_trait::async_trait;
use crate::core::error::ClientError;
use crate::core::transaction::TransactionResponse;
use crate::evm::types::{EvmAddress, EvmBytes, EvmHash, EvmTransactionRequest, EvmU256};

/// Base trait for all EVM client implementations
#[async_trait]
pub trait EvmBaseClient: Send + Sync {

    //-----------------------------------------------------------------------------
    // Account Information
    //-----------------------------------------------------------------------------

    /// Get the address of the account used for signing transactions
    fn evm_signer_address(&self) -> EvmAddress;

    /// Get the balance of an address in wei
    async fn get_balance(&self, address: &EvmAddress) -> Result<EvmU256, ClientError>;

    /// Get the current nonce (transaction count) for an address
    async fn get_nonce(&self, address: &EvmAddress) -> Result<u64, ClientError>;

    //-----------------------------------------------------------------------------
    // Transaction Operations
    //-----------------------------------------------------------------------------

    /// Send a raw, signed transaction
    async fn send_raw_transaction(&self, tx_bytes: &EvmBytes) -> Result<EvmHash, ClientError>;

    /// Send a transaction and wait for it to be mined
    async fn send_transaction(&self, tx: &EvmTransactionRequest) -> Result<TransactionResponse, ClientError>;

    /// Get transaction by hash
    async fn get_transaction(&self, tx_hash: &EvmHash) -> Result<Option<TransactionResponse>, ClientError>;

    /// Wait for a transaction to be confirmed
    async fn wait_for_transaction_receipt(&self, tx_hash: &EvmHash) -> Result<TransactionResponse, ClientError>;

    //-----------------------------------------------------------------------------
    // Chain Information
    //-----------------------------------------------------------------------------

    /// Get the current block number
    async fn get_block_number(&self) -> Result<u64, ClientError>;

    /// Get the chain ID
    async fn get_chain_id(&self) -> Result<u64, ClientError>;

    /// Get the current gas price in wei
    async fn get_gas_price(&self) -> Result<EvmU256, ClientError>;

    //-----------------------------------------------------------------------------
    // Smart Contract Interaction
    //-----------------------------------------------------------------------------

    /// Call a contract view function (does not modify state)
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
