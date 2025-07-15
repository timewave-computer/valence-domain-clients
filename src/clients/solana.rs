// Solana client for interacting with any Solana cluster
use crate::solana::{
    base_client::SolanaBaseClient,
    query_client::SolanaQueryClient,
    rpc_client::SolanaRpcClient,
    signing_client::SolanaSigningClient,
};

#[cfg(test)]
use crate::solana::query_client::SolanaReadOnlyClient;
use async_trait::async_trait;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    bs58,
};
use solana_sdk::signer::keypair::keypair_from_seed_phrase_and_passphrase;

/// Default localhost RPC URL for Solana test validator
const DEFAULT_RPC_URL: &str = "http://localhost:8899";

/// Solana client for interacting with any Solana cluster
/// 
/// This client can connect to any Solana cluster by specifying the RPC URL.
/// Common RPC URLs:
/// - Local: http://localhost:8899
/// - Devnet: https://api.devnet.solana.com
/// - Testnet: https://api.testnet.solana.com  
/// - Mainnet: https://api.mainnet-beta.solana.com
pub struct SolanaClient {
    keypair: Keypair,
    rpc_client: solana_client::nonblocking::rpc_client::RpcClient,
    rpc_url: String,
    commitment: CommitmentConfig,
}

impl SolanaClient {
    /// Create a new Solana client from a mnemonic (primary constructor)
    pub fn new(
        rpc_url: &str,
        mnemonic: &str,
    ) -> anyhow::Result<Self> {
        let passphrase = ""; // No passphrase
        
        // Use solana-sdk's official function for deriving keypair from mnemonic
        let keypair = keypair_from_seed_phrase_and_passphrase(mnemonic, passphrase)
            .map_err(|e| anyhow::anyhow!("Failed to derive keypair from mnemonic: {}", e))?;
        
        let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());
        let commitment = CommitmentConfig::confirmed();
        
        Ok(Self {
            keypair,
            rpc_client,
            rpc_url: rpc_url.to_string(),
            commitment,
        })
    }
    
    /// Create a new client with localhost defaults
    pub fn new_localhost(mnemonic: &str) -> anyhow::Result<Self> {
        Self::new(DEFAULT_RPC_URL, mnemonic)
    }
    
    /// Create a new client from a keypair (alternative constructor)
    pub fn from_keypair(keypair: Keypair, rpc_url: &str) -> Self {
        let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());
        let commitment = CommitmentConfig::confirmed();
        
        Self {
            keypair,
            rpc_client,
            rpc_url: rpc_url.to_string(),
            commitment,
        }
    }
    
    /// Create a new client from private key bytes
    pub fn from_bytes(private_key: &[u8], rpc_url: &str) -> anyhow::Result<Self> {
        let keypair = Keypair::from_bytes(private_key)
            .map_err(|e| anyhow::anyhow!("Invalid keypair bytes: {}", e))?;
        Ok(Self::from_keypair(keypair, rpc_url))
    }
    
    /// Create a new client from base58 encoded private key
    pub fn from_base58(private_key: &str, rpc_url: &str) -> anyhow::Result<Self> {
        let bytes = bs58::decode(private_key).into_vec()?;
        Self::from_bytes(&bytes, rpc_url)
    }
    
    /// Generate a new client with random keypair
    pub fn generate_new(rpc_url: &str) -> Self {
        let keypair = Keypair::new();
        Self::from_keypair(keypair, rpc_url)
    }
    
    /// Get the public key as a string  
    pub fn get_pubkey_string(&self) -> String {
        self.keypair.pubkey().to_string()
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
    
    fn commitment(&self) -> CommitmentConfig {
        self.commitment
    }
}

#[async_trait]
impl SolanaSigningClient for SolanaClient {
    fn get_keypair(&self) -> &Keypair {
        &self.keypair
    }
}

#[async_trait]
impl SolanaQueryClient for SolanaClient {}

#[async_trait]
impl SolanaBaseClient for SolanaClient {}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::{Keypair, Signer};
    
    const TEST_RPC_URL: &str = DEFAULT_RPC_URL;
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_client_creation() {
        let client = SolanaClient::generate_new(TEST_RPC_URL);
        assert!(!client.get_pubkey_string().is_empty());
    }
    
    #[tokio::test]
    async fn test_mnemonic_keypair_derivation() {
        // Test client creation from mnemonic using the new primary constructor
        let client = SolanaClient::new_localhost(TEST_MNEMONIC).unwrap();
        assert!(!client.get_pubkey_string().is_empty());
        
        // Test deterministic derivation - same mnemonic should produce same keys
        let client2 = SolanaClient::new_localhost(TEST_MNEMONIC).unwrap();
        assert_eq!(client.get_pubkey_string(), client2.get_pubkey_string());
        
        // Test with different RPC URL but same mnemonic
        let client3 = SolanaClient::new(TEST_RPC_URL, TEST_MNEMONIC).unwrap();
        assert_eq!(client.get_pubkey_string(), client3.get_pubkey_string());
        
        // Verify the derived public key matches expected format (base58 encoded, 32-44 chars)
        let pubkey = client.get_pubkey_string();
        assert!(pubkey.len() >= 32 && pubkey.len() <= 44);
        
        // The derived address should be deterministic and match the expected Solana address format
        // For this specific test mnemonic with derivation path m/44'/501'/0'/0', we expect a specific address
        println!("Derived address: {}", pubkey);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_read_only_client_queries() {
        // Create a read-only client (no keypair required)
        let read_only_client = SolanaReadOnlyClient::new(TEST_RPC_URL);
        
        // Test that we can query blockchain state without signing capabilities
        let block_height = read_only_client.latest_block_height().await.unwrap();
        assert!(block_height > 0, "Should get a valid block height");
        
        let _transaction_count = read_only_client.get_transaction_count().await.unwrap();
        // Transaction count should be retrievable
        
        let cluster_nodes = read_only_client.get_cluster_nodes().await.unwrap();
        assert!(!cluster_nodes.is_empty(), "Should have at least one cluster node");
        
        // Test querying a specific address balance
        let test_address = "EHqmfkN89RJ7Y33CXM6uCzhVeuywHoJXZZLszBHHZy7o";
        let _balance = read_only_client.get_sol_balance_for_address(test_address).await.unwrap();
        // Balance should be retrievable (even if zero)
        
        println!("✅ Read-only client can query blockchain state without keypair");
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_client_from_mnemonic() {
        let client = SolanaClient::new_localhost(TEST_MNEMONIC).unwrap();
        assert!(!client.get_pubkey_string().is_empty());
        
        // Test with custom RPC URL
        let client2 = SolanaClient::new(TEST_RPC_URL, TEST_MNEMONIC).unwrap();
        assert_eq!(client.get_pubkey_string(), client2.get_pubkey_string());
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_get_latest_block_height() {
        let client = SolanaClient::generate_new(TEST_RPC_URL);
        let block_height = client.latest_block_height().await.unwrap();
        assert!(block_height > 0);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_airdrop_and_balance() {
        let client = SolanaClient::generate_new(TEST_RPC_URL);
        
        // Test airdrop
        let airdrop_result = client.airdrop_sol_amount(1.0).await.unwrap();
        assert!(airdrop_result.success);
        
        // Test balance check
        let balance = client.get_sol_balance_as_sol().await.unwrap();
        assert!(balance >= 1.0);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_transfer() {
        let client1 = SolanaClient::generate_new(TEST_RPC_URL);
        let client2 = SolanaClient::generate_new(TEST_RPC_URL);
        
        // Airdrop to first client
        client1.airdrop_sol_amount(2.0).await.unwrap();
        
        // Get initial balance of second client
        let initial_balance = client2.get_sol_balance_as_sol().await.unwrap();
        
        // Transfer from first to second
        let transfer_result = client1.transfer_sol_amount(&client2.get_pubkey_string(), 0.5).await.unwrap();
        assert!(transfer_result.success);
        
        // Check balance increased
        let final_balance = client2.get_sol_balance_as_sol().await.unwrap();
        assert!(final_balance > initial_balance);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_account_operations() {
        let client = SolanaClient::generate_new(TEST_RPC_URL);
        
        // Airdrop some SOL
        client.airdrop_sol_amount(1.0).await.unwrap();
        
        // Test account existence
        let exists = client.account_exists(&client.get_pubkey_string()).await.unwrap();
        assert!(exists);
        
        // Test non-existent account
        let fake_pubkey = Keypair::new().pubkey().to_string();
        let exists = client.account_exists(&fake_pubkey).await.unwrap();
        assert!(!exists);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_cluster_info() {
        let client = SolanaClient::generate_new(TEST_RPC_URL);
        
        // Test epoch info
        let _epoch_info = client.get_epoch_info().await.unwrap();
        // Epoch should be a valid value (u64 is always >= 0)
        
        // Test transaction count  
        let _tx_count = client.get_transaction_count().await.unwrap();
        // Transaction count should be a valid value (u64 is always >= 0)
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_poll_balance() {
        let client1 = SolanaClient::generate_new(TEST_RPC_URL);
        let client2 = SolanaClient::generate_new(TEST_RPC_URL);
        
        // Airdrop to first client
        client1.airdrop_sol_amount(2.0).await.unwrap();
        
        // Start transfer in background
        let client2_pubkey = client2.get_pubkey_string();
        let client1_clone = SolanaClient::from_bytes(&client1.get_keypair().to_bytes(), TEST_RPC_URL).unwrap();
        
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let _ = client1_clone.transfer_sol_amount(&client2_pubkey, 0.5).await;
        });
        
        // Poll for balance
        let balance = client2.poll_until_expected_sol_balance(0.1, 1, 10).await.unwrap();
        assert!(balance >= 0.1);
    }
} 