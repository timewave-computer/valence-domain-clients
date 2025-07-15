// Solana query client trait and implementation
use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
};
use std::str::FromStr;

use super::rpc_client::SolanaRpcClient;

/// Trait for read-only Solana operations that don't require signing
/// 
/// This trait provides utility functions and query operations that can be
/// performed with any Solana RPC client without requiring a keypair.
#[async_trait]
pub trait SolanaQueryClient: SolanaRpcClient {
    /// Convert SOL to lamports
    fn sol_to_lamports(sol: f64) -> u64 {
        (sol * LAMPORTS_PER_SOL as f64) as u64
    }
    
    /// Convert lamports to SOL
    fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / LAMPORTS_PER_SOL as f64
    }
    
    /// Get SOL balance for a specific address in lamports
    async fn get_sol_balance_for_address(&self, address: &str) -> anyhow::Result<u64> {
        let pubkey = Pubkey::from_str(address)?;
        self.get_balance(&pubkey).await
    }
    
    /// Get SOL balance for address in SOL (not lamports)
    async fn get_sol_balance_for_address_as_sol(&self, address: &str) -> anyhow::Result<f64> {
        let lamports = self.get_sol_balance_for_address(address).await?;
        Ok(Self::lamports_to_sol(lamports))
    }
    
    /// Get the latest block height
    async fn latest_block_height(&self) -> anyhow::Result<u64> {
        self.get_slot().await
    }
    
    /// Get account data
    async fn get_account_data(&self, address: &str) -> anyhow::Result<Vec<u8>> {
        let pubkey = Pubkey::from_str(address)?;
        let account = self.get_account(&pubkey).await?;
        
        match account {
            Some(account) => Ok(account.data),
            None => Err(anyhow::anyhow!("Account not found")),
        }
    }
    
    /// Check if account exists
    async fn account_exists(&self, address: &str) -> anyhow::Result<bool> {
        let pubkey = Pubkey::from_str(address)?;
        let account = self.get_account(&pubkey).await?;
        Ok(account.is_some())
    }
    
    /// Get minimum balance for rent exemption
    async fn get_minimum_balance_for_rent_exemption(&self, space: usize) -> anyhow::Result<u64> {
        let rpc_client = self.get_rpc_client();
        let rent = rpc_client.get_minimum_balance_for_rent_exemption(space).await?;
        Ok(rent)
    }
    
    /// Get recent performance samples
    async fn get_recent_performance_samples(&self) -> anyhow::Result<Vec<solana_rpc_client_api::response::RpcPerfSample>> {
        let rpc_client = self.get_rpc_client();
        let samples = rpc_client.get_recent_performance_samples(None).await?;
        Ok(samples)
    }
    
    /// Get cluster nodes
    async fn get_cluster_nodes(&self) -> anyhow::Result<Vec<solana_rpc_client_api::response::RpcContactInfo>> {
        let rpc_client = self.get_rpc_client();
        let nodes = rpc_client.get_cluster_nodes().await?;
        Ok(nodes)
    }
    
    /// Get epoch info
    async fn get_epoch_info(&self) -> anyhow::Result<u64> {
        let rpc_client = self.get_rpc_client();
        let epoch_info = rpc_client.get_epoch_info().await?;
        Ok(epoch_info.epoch)
    }
    
    /// Get transaction count (similar to nonce in other chains)
    async fn get_transaction_count(&self) -> anyhow::Result<u64> {
        let rpc_client = self.get_rpc_client();
        let count = rpc_client.get_transaction_count().await?;
        Ok(count)
    }
    
    /// Poll until expected SOL balance is reached for any address
    async fn poll_until_expected_sol_balance_for_address(
        &self,
        address: &str,
        min_amount_sol: f64,
        interval_sec: u64,
        max_attempts: u32,
    ) -> anyhow::Result<f64> {
        let min_amount_lamports = Self::sol_to_lamports(min_amount_sol);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_sec));
        
        log::info!("Polling {} SOL balance to exceed {}", address, min_amount_sol);
        
        for attempt in 1..max_attempts + 1 {
            interval.tick().await;
            
            match self.get_sol_balance_for_address(address).await {
                Ok(balance_lamports) => {
                    if balance_lamports >= min_amount_lamports {
                        let balance_sol = Self::lamports_to_sol(balance_lamports);
                        return Ok(balance_sol);
                    }
                    let current_sol = Self::lamports_to_sol(balance_lamports);
                    log::info!(
                        "Balance polling attempt {}/{}: current={} SOL, target={} SOL",
                        attempt, max_attempts, current_sol, min_amount_sol
                    );
                }
                Err(e) => {
                    log::warn!(
                        "Balance polling attempt {}/{} failed: {:?}",
                        attempt, max_attempts, e
                    );
                }
            }
        }
        
        Err(anyhow::anyhow!(
            "Balance did not exceed {} SOL after {} attempts",
            min_amount_sol, max_attempts
        ))
    }
}

/// Read-only Solana client for querying blockchain state without signing capabilities
/// 
/// This client can only perform read operations like querying balances, accounts,
/// and blockchain metadata. It does not require a keypair and cannot sign transactions.
pub struct SolanaReadOnlyClient {
    rpc_client: RpcClient,
    rpc_url: String,
    commitment: CommitmentConfig,
}

impl SolanaReadOnlyClient {
    /// Create a new read-only Solana client
    pub fn new(rpc_url: &str) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        let commitment = CommitmentConfig::confirmed();
        
        Self {
            rpc_client,
            rpc_url: rpc_url.to_string(),
            commitment,
        }
    }
    
    /// Create a new read-only client with custom commitment level
    pub fn new_with_commitment(rpc_url: &str, commitment: CommitmentConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(), commitment);
        
        Self {
            rpc_client,
            rpc_url: rpc_url.to_string(),
            commitment,
        }
    }
    
    /// Get the RPC client instance for advanced operations
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }
}

#[async_trait]
impl SolanaRpcClient for SolanaReadOnlyClient {
    fn get_rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }
    
    fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
    
    fn commitment(&self) -> CommitmentConfig {
        self.commitment
    }
}

#[async_trait]
impl SolanaQueryClient for SolanaReadOnlyClient {} 