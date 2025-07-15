// Solana base client trait with default implementations for Solana-based clients
use async_trait::async_trait;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
};
use solana_system_interface::{instruction as system_instruction};

use super::{signing_client::SolanaSigningClient, query_client::SolanaQueryClient};
use crate::common::transaction::TransactionResponse;

/// Base client trait with default implementations for Solana-based clients.
/// 
/// For chains which are somehow unique in their common module implementations,
/// these function definitions can be overridden to match the custom chain logic.
#[async_trait]
pub trait SolanaBaseClient: SolanaSigningClient + SolanaQueryClient {
    /// Get SOL balance for own keypair in SOL (not lamports)
    async fn get_sol_balance_as_sol(&self) -> anyhow::Result<f64> {
        let lamports = self.get_sol_balance().await?;
        Ok(Self::lamports_to_sol(lamports))
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
    
    /// Poll until expected SOL balance is reached for own keypair
    async fn poll_until_expected_sol_balance(
        &self,
        min_amount_sol: f64,
        interval_sec: u64,
        max_attempts: u32,
    ) -> anyhow::Result<f64> {
        let keypair = self.get_keypair();
        let address = keypair.pubkey().to_string();
        self.poll_until_expected_sol_balance_for_address(&address, min_amount_sol, interval_sec, max_attempts).await
    }
    
    /// Execute multiple instructions in a single transaction
    async fn execute_instructions(&self, instructions: Vec<Instruction>) -> anyhow::Result<TransactionResponse> {
        self.send_instructions(instructions).await
    }
} 