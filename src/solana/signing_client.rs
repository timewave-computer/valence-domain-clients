// Solana signing client for transaction signing and keypair management
use async_trait::async_trait;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
    derivation_path::DerivationPath,
};
use std::str::FromStr;
use bip32::{Language, Mnemonic};

use super::rpc_client::SolanaRpcClient;
use crate::common::transaction::TransactionResponse;

/// Default timeout for transaction confirmation in seconds
const DEFAULT_TRANSACTION_TIMEOUT_SECONDS: u64 = 30;

/// Standard Solana BIP44 derivation path used by browser wallets (Phantom, Solflare)
const SOLANA_DERIVATION_PATH: &str = "m/44'/501'/0'/0'";

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
    
    /// Get SOL balance in lamports
    async fn get_sol_balance(&self) -> anyhow::Result<u64> {
        let pubkey = self.get_pubkey();
        self.get_balance(&pubkey).await
    }
    
    /// Get SOL balance for a specific address
    async fn get_sol_balance_for_address(&self, address: &str) -> anyhow::Result<u64> {
        let pubkey = Pubkey::from_str(address)?;
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

/// Implementation of signing client that wraps a keypair
pub struct SolanaClient {
    keypair: Keypair,
    rpc_client: solana_client::nonblocking::rpc_client::RpcClient,
    rpc_url: String,
    commitment: solana_sdk::commitment_config::CommitmentConfig,
}

impl SolanaClient {
    /// Create a new signing client from a keypair
    pub fn new(keypair: Keypair, rpc_url: &str) -> Self {
        let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());
        let commitment = solana_sdk::commitment_config::CommitmentConfig::confirmed();
        
        Self {
            keypair,
            rpc_client,
            rpc_url: rpc_url.to_string(),
            commitment,
        }
    }
    
    /// Create a new signing client from a private key byte array
    pub fn from_bytes(private_key: &[u8], rpc_url: &str) -> anyhow::Result<Self> {
        let keypair = Keypair::from_bytes(private_key)?;
        Ok(Self::new(keypair, rpc_url))
    }
    
    /// Create a new signing client from a base58 encoded private key
    pub fn from_base58(private_key: &str, rpc_url: &str) -> anyhow::Result<Self> {
        let bytes = bs58::decode(private_key).into_vec()?;
        Self::from_bytes(&bytes, rpc_url)
    }
    
    /// Generate a new random keypair
    pub fn generate_new(rpc_url: &str) -> Self {
        let keypair = Keypair::new();
        Self::new(keypair, rpc_url)
    }
    
    /// Create a new signing client from a mnemonic
    pub fn from_mnemonic(mnemonic: &str, rpc_url: &str) -> anyhow::Result<Self> {
        let mnemonic = Mnemonic::new(mnemonic, Language::English)?;
        let seed = mnemonic.to_seed("");
        
        // Use the standard Solana derivation path
        let derivation_path = DerivationPath::from_str(SOLANA_DERIVATION_PATH)?;
        
        // Derive the keypair from the seed using the derivation path
        let extended_key = bip32::ExtendedPrivateKey::derive_from_path(&seed, &derivation_path)?;
        let private_key = extended_key.private_key();
        
        // Create Solana keypair from the private key
        let keypair = Keypair::from_bytes(&private_key.to_bytes())?;
        
        Ok(Self::new(keypair, rpc_url))
    }
}

#[async_trait]
impl SolanaRpcClient for SolanaClient {
    fn get_rpc_client(&self) -> &solana_client::nonblocking::rpc_client::RpcClient {
        &self.rpc_client
    }
    
    fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
    
    fn commitment(&self) -> solana_sdk::commitment_config::CommitmentConfig {
        self.commitment
    }
}

#[async_trait]
impl SolanaSigningClient for SolanaClient {
    fn get_keypair(&self) -> &Keypair {
        &self.keypair
    }
} 