// Solana signing client for transaction signing and keypair management
use async_trait::async_trait;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

use super::rpc_client::SolanaRpcClient;
use crate::common::transaction::TransactionResponse;

/// Default timeout for transaction confirmation in seconds
const DEFAULT_TRANSACTION_TIMEOUT_SECONDS: u64 = 30;



/// Trait for Solana transaction signing operations
#[async_trait]
pub trait SolanaSigningClient: SolanaRpcClient {
    /// Get the signing keypair
    fn get_keypair(&self) -> &Keypair;
    
    /// Get the public key of the signing keypair
    fn get_pubkey(&self) -> Pubkey {
        self.get_keypair().pubkey()
    }
    
    /// Sign a transaction
    fn sign_transaction(&self, transaction: &mut Transaction) -> anyhow::Result<()> {
        let keypair = self.get_keypair();
        transaction.sign(&[keypair], transaction.message.recent_blockhash);
        Ok(())
    }
    
    /// Create and sign a transaction with the given instructions
    async fn create_and_sign_transaction(&self, instructions: Vec<Instruction>) -> anyhow::Result<Transaction> {
        let keypair = self.get_keypair();
        let recent_blockhash = self.get_latest_blockhash().await?;
        
        let message = Message::new(&instructions, Some(&keypair.pubkey()));
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[keypair], recent_blockhash);
        
        Ok(transaction)
    }
    
    /// Send instructions as a transaction
    async fn send_instructions(&self, instructions: Vec<Instruction>) -> anyhow::Result<TransactionResponse> {
        let transaction = self.create_and_sign_transaction(instructions).await?;
        let signature = self.send_transaction(&transaction).await?;
        
        // Poll for confirmation
        let confirmed = self.poll_for_signature_confirmation(&signature, DEFAULT_TRANSACTION_TIMEOUT_SECONDS).await?;
        
        if !confirmed {
            return Err(anyhow::anyhow!("Transaction not confirmed"));
        }
        
        // Get the transaction details for response
        let slot = self.get_slot().await?;
        
        Ok(TransactionResponse {
            hash: signature.to_string(),
            success: confirmed,
            block_height: slot,
            gas_used: 0, // Solana doesn't have gas, we could use compute units if needed
        })
    }
    
    /// Transfer SOL to another account
    async fn transfer_sol(&self, to: &str, amount_lamports: u64) -> anyhow::Result<TransactionResponse> {
        let keypair = self.get_keypair();
        let to_pubkey = Pubkey::from_str(to)?;
        
        let instruction = solana_sdk::system_instruction::transfer(
            &keypair.pubkey(),
            &to_pubkey,
            amount_lamports,
        );
        
        self.send_instructions(vec![instruction]).await
    }
    
    /// Transfer SOL with amount in SOL (not lamports)
    async fn transfer_sol_amount(
        &self,
        to: &str,
        amount_sol: f64,
    ) -> anyhow::Result<TransactionResponse> {
        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        self.transfer_sol(to, amount_lamports).await
    }
    
    /// Get SOL balance in lamports for own keypair
    async fn get_sol_balance(&self) -> anyhow::Result<u64> {
        let pubkey = self.get_pubkey();
        self.get_balance(&pubkey).await
    }
    
    /// Airdrop SOL (only works on test networks)
    async fn airdrop_sol(&self, amount_lamports: u64) -> anyhow::Result<TransactionResponse> {
        let rpc_client = self.get_rpc_client();
        let pubkey = self.get_pubkey();
        
        let signature = rpc_client.request_airdrop(&pubkey, amount_lamports).await?;
        
        // Poll for confirmation
        let confirmed = self.poll_for_signature_confirmation(&signature, DEFAULT_TRANSACTION_TIMEOUT_SECONDS).await?;
        
        if !confirmed {
            return Err(anyhow::anyhow!("Airdrop not confirmed"));
        }
        
        let slot = self.get_slot().await?;
        
        Ok(TransactionResponse {
            hash: signature.to_string(),
            success: confirmed,
            block_height: slot,
            gas_used: 0,
        })
    }
    
    /// Airdrop SOL with amount in SOL (not lamports)
    async fn airdrop_sol_amount(&self, amount_sol: f64) -> anyhow::Result<TransactionResponse> {
        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        self.airdrop_sol(amount_lamports).await
    }
}

 