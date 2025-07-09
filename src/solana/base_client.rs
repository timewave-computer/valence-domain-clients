// Solana base client trait with default implementations for Solana-based clients
use async_trait::async_trait;
use solana_sdk::{
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;
use log::{info, warn};

use super::signing_client::SolanaSigningClient;
use crate::common::transaction::TransactionResponse;

/// Base client trait with default implementations for Solana-based clients.
/// 
/// For chains which are somehow unique in their common module implementations,
/// these function definitions can be overridden to match the custom chain logic.
#[async_trait]
pub trait SolanaBaseClient: SolanaSigningClient {
    /// Convert SOL to lamports
    fn sol_to_lamports(sol: f64) -> u64 {
        (sol * LAMPORTS_PER_SOL as f64) as u64
    }
    
    /// Convert lamports to SOL
    fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / LAMPORTS_PER_SOL as f64
    }
    

    
    /// Get SOL balance in SOL (not lamports)
    async fn get_sol_balance_as_sol(&self) -> anyhow::Result<f64> {
        let lamports = self.get_sol_balance().await?;
        Ok(Self::lamports_to_sol(lamports))
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
    
    /// Create a new account with specified space and owner
    async fn create_account(
        &self,
        new_account_pubkey: &Pubkey,
        space: u64,
        owner: &Pubkey,
        lamports: u64,
    ) -> anyhow::Result<TransactionResponse> {
        let keypair = self.get_keypair();
        
        let instruction = system_instruction::create_account(
            &keypair.pubkey(),
            new_account_pubkey,
            lamports,
            space,
            owner,
        );
        
        self.send_instructions(vec![instruction]).await
    }
    
    /// Poll for transaction confirmation with custom timeout
    async fn poll_for_tx_confirmation(
        &self,
        signature: &Signature,
        timeout_seconds: u64,
    ) -> anyhow::Result<TransactionResponse> {
        let confirmed = self.poll_for_signature_confirmation(signature, timeout_seconds).await?;
        
        if !confirmed {
            return Err(anyhow::anyhow!("Transaction not confirmed within timeout"));
        }
        
        let slot = self.get_slot().await?;
        
        Ok(TransactionResponse {
            hash: signature.to_string(),
            success: confirmed,
            block_height: slot,
            gas_used: 0,
        })
    }
    
    /// Poll until expected SOL balance is reached
    async fn poll_until_expected_sol_balance(
        &self,
        address: &str,
        min_amount_sol: f64,
        interval_sec: u64,
        max_attempts: u32,
    ) -> anyhow::Result<f64> {
        let min_amount_lamports = Self::sol_to_lamports(min_amount_sol);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_sec));
        
        info!("Polling {} SOL balance to exceed {}", address, min_amount_sol);
        
        for attempt in 1..max_attempts + 1 {
            interval.tick().await;
            
            match self.get_sol_balance_for_address(address).await {
                Ok(balance_lamports) => {
                    if balance_lamports >= min_amount_lamports {
                        let balance_sol = Self::lamports_to_sol(balance_lamports);
                        return Ok(balance_sol);
                    }
                    let current_sol = Self::lamports_to_sol(balance_lamports);
                    info!(
                        "Balance polling attempt {}/{}: current={} SOL, target={} SOL",
                        attempt, max_attempts, current_sol, min_amount_sol
                    );
                }
                Err(e) => {
                    warn!(
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
    
    /// Execute multiple instructions in a single transaction
    async fn execute_instructions(&self, instructions: Vec<Instruction>) -> anyhow::Result<TransactionResponse> {
        self.send_instructions(instructions).await
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
    async fn get_epoch_info(&self) -> anyhow::Result<solana_rpc_client_api::response::EpochInfo> {
        let rpc_client = self.get_rpc_client();
        let epoch_info = rpc_client.get_epoch_info().await?;
        Ok(epoch_info)
    }
    
    /// Get transaction count (similar to nonce in other chains)
    async fn get_transaction_count(&self) -> anyhow::Result<u64> {
        let rpc_client = self.get_rpc_client();
        let count = rpc_client.get_transaction_count().await?;
        Ok(count)
    }
} 