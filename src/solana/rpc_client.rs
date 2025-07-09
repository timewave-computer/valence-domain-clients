// Solana RPC client trait for communicating with Solana RPC nodes
use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

/// Default polling interval for transaction confirmation
const DEFAULT_POLLING_INTERVAL_MS: u64 = 500;

/// Default maximum retries for transaction submission
const DEFAULT_MAX_RETRIES: usize = 3;

/// Default maximum supported transaction version
const DEFAULT_MAX_TRANSACTION_VERSION: u8 = 0;

/// Trait for Solana RPC client operations
#[async_trait]
pub trait SolanaRpcClient {
    /// Get the underlying RPC client
    fn get_rpc_client(&self) -> &RpcClient;
    
    /// Get the RPC URL
    fn rpc_url(&self) -> &str;
    
    /// Get the commitment level
    fn commitment(&self) -> CommitmentConfig;
    
    /// Get the latest blockhash
    async fn get_latest_blockhash(&self) -> anyhow::Result<solana_sdk::hash::Hash> {
        let rpc_client = self.get_rpc_client();
        let blockhash = rpc_client.get_latest_blockhash().await?;
        Ok(blockhash)
    }
    
    /// Get account balance in lamports
    async fn get_balance(&self, pubkey: &Pubkey) -> anyhow::Result<u64> {
        let rpc_client = self.get_rpc_client();
        let balance = rpc_client.get_balance(pubkey).await?;
        Ok(balance)
    }
    
    /// Get account info
    async fn get_account(&self, pubkey: &Pubkey) -> anyhow::Result<Option<solana_sdk::account::Account>> {
        let rpc_client = self.get_rpc_client();
        let account = rpc_client.get_account(pubkey).await?;
        Ok(Some(account))
    }
    
    /// Send a transaction
    async fn send_transaction(&self, transaction: &Transaction) -> anyhow::Result<Signature> {
        let rpc_client = self.get_rpc_client();
        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(self.commitment().commitment),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: Some(DEFAULT_MAX_RETRIES),
            min_context_slot: None,
        };
        
        let signature = rpc_client.send_transaction_with_config(transaction, config).await?;
        Ok(signature)
    }
    
    /// Confirm a transaction
    async fn confirm_transaction(&self, signature: &Signature) -> anyhow::Result<bool> {
        let rpc_client = self.get_rpc_client();
        let result = rpc_client.confirm_transaction_with_spinner(signature, &self.commitment()).await?;
        Ok(result)
    }
    
    /// Get slot (block height)
    async fn get_slot(&self) -> anyhow::Result<u64> {
        let rpc_client = self.get_rpc_client();
        let slot = rpc_client.get_slot().await?;
        Ok(slot)
    }
    
    /// Get transaction details
    async fn get_transaction(&self, signature: &Signature) -> anyhow::Result<Option<solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta>> {
        let rpc_client = self.get_rpc_client();
        let config = solana_rpc_client_api::config::RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(self.commitment()),
            max_supported_transaction_version: Some(DEFAULT_MAX_TRANSACTION_VERSION),
        };
        
        let transaction = rpc_client.get_transaction_with_config(signature, config).await?;
        Ok(Some(transaction))
    }
    
    /// Poll for transaction confirmation
    async fn poll_for_signature_confirmation(&self, signature: &Signature, timeout_seconds: u64) -> anyhow::Result<bool> {
        let rpc_client = self.get_rpc_client();
        let timeout = std::time::Duration::from_secs(timeout_seconds);
        
        let start_time = std::time::Instant::now();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(DEFAULT_POLLING_INTERVAL_MS));
        
        loop {
            interval.tick().await;
            
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Transaction confirmation timeout"));
            }
            
            match rpc_client.get_signature_status(signature).await {
                Ok(Some(status)) => {
                    if let Some(result) = status {
                        match result {
                            Ok(()) => return Ok(true),
                            Err(e) => return Err(anyhow::anyhow!("Transaction failed: {:?}", e)),
                        }
                    }
                }
                Ok(None) => {
                    // Transaction not yet processed, continue polling
                    continue;
                }
                Err(e) => {
                    log::warn!("Error checking signature status: {:?}", e);
                    continue;
                }
            }
        }
    }
} 