// Solana test client for development and testing (supports any Solana cluster)
use crate::solana::{
    base_client::SolanaBaseClient,
    rpc_client::SolanaRpcClient,
    signing_client::{SolanaSigningClient, SolanaClient},
};
use async_trait::async_trait;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
};

/// Solana client for development and testing (supports any Solana cluster)
/// 
/// This client can connect to any Solana cluster by specifying the RPC URL.
/// The default methods connect to http://localhost:8899 for local development.
pub struct SolanaTestClient {
    inner: SolanaClient,
}

impl SolanaTestClient {
    /// Create a new localnet client
    pub fn new(keypair: Keypair) -> Self {
        let inner = SolanaClient::new(keypair, "http://localhost:8899");
        Self { inner }
    }
    
    /// Create a new localnet client with custom RPC URL
    pub fn with_rpc_url(keypair: Keypair, rpc_url: &str) -> Self {
        let inner = SolanaClient::new(keypair, rpc_url);
        Self { inner }
    }
    
    /// Create a new localnet client from private key bytes
    pub fn from_bytes(private_key: &[u8]) -> anyhow::Result<Self> {
        Self::from_bytes_with_rpc_url(private_key, "http://localhost:8899")
    }
    
    /// Create a new localnet client from private key bytes with custom RPC URL
    pub fn from_bytes_with_rpc_url(private_key: &[u8], rpc_url: &str) -> anyhow::Result<Self> {
        let inner = SolanaClient::from_bytes(private_key, rpc_url)?;
        Ok(Self { inner })
    }
    
    /// Create a new localnet client from base58 encoded private key
    pub fn from_base58(private_key: &str) -> anyhow::Result<Self> {
        Self::from_base58_with_rpc_url(private_key, "http://localhost:8899")
    }
    
    /// Create a new localnet client from base58 encoded private key with custom RPC URL
    pub fn from_base58_with_rpc_url(private_key: &str, rpc_url: &str) -> anyhow::Result<Self> {
        let inner = SolanaClient::from_base58(private_key, rpc_url)?;
        Ok(Self { inner })
    }
    
    /// Generate a new client with random keypair
    pub fn generate_new() -> Self {
        Self::generate_new_with_rpc_url("http://localhost:8899")
    }
    
    /// Generate a new client with random keypair and custom RPC URL
    pub fn generate_new_with_rpc_url(rpc_url: &str) -> Self {
        let inner = SolanaClient::generate_new(rpc_url);
        Self { inner }
    }
    
    /// Create a new localnet client from a mnemonic
    pub fn from_mnemonic(mnemonic: &str) -> anyhow::Result<Self> {
        let inner = SolanaClient::from_mnemonic(mnemonic, "http://localhost:8899")?;
        Ok(Self { inner })
    }
    
    /// Create a new localnet client from a mnemonic with custom RPC URL
    pub fn from_mnemonic_with_rpc_url(mnemonic: &str, rpc_url: &str) -> anyhow::Result<Self> {
        let inner = SolanaClient::from_mnemonic(mnemonic, rpc_url)?;
        Ok(Self { inner })
    }
    
    /// Get the public key as a string
    pub fn get_pubkey_string(&self) -> String {
        self.inner.get_pubkey().to_string()
    }
}

#[async_trait]
impl SolanaRpcClient for SolanaTestClient {
    fn get_rpc_client(&self) -> &solana_client::nonblocking::rpc_client::RpcClient {
        self.inner.get_rpc_client()
    }
    
    fn rpc_url(&self) -> &str {
        self.inner.rpc_url()
    }
    
    fn commitment(&self) -> CommitmentConfig {
        self.inner.commitment()
    }
}

#[async_trait]
impl SolanaSigningClient for SolanaTestClient {
    fn get_keypair(&self) -> &Keypair {
        self.inner.get_keypair()
    }
}

#[async_trait]
impl SolanaBaseClient for SolanaTestClient {}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    
    const TEST_RPC_URL: &str = "http://localhost:8899";
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_client_creation() {
        let client = SolanaTestClient::generate_new();
        assert!(!client.get_pubkey_string().is_empty());
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_client_from_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let client = SolanaTestClient::from_mnemonic(mnemonic).unwrap();
        assert!(!client.get_pubkey_string().is_empty());
        
        // Test with custom RPC URL
        let client2 = SolanaTestClient::from_mnemonic_with_rpc_url(mnemonic, TEST_RPC_URL).unwrap();
        assert_eq!(client.get_pubkey_string(), client2.get_pubkey_string());
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_get_latest_block_height() {
        let client = SolanaTestClient::generate_new();
        let block_height = client.latest_block_height().await.unwrap();
        assert!(block_height > 0);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_airdrop_and_balance() {
        let client = SolanaTestClient::generate_new();
        
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
        let client1 = SolanaTestClient::generate_new();
        let client2 = SolanaTestClient::generate_new();
        
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
        let client = SolanaTestClient::generate_new();
        
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
        let client = SolanaTestClient::generate_new();
        
        // Test epoch info
        let epoch_info = client.get_epoch_info().await.unwrap();
        assert!(epoch_info.epoch >= 0);
        
        // Test transaction count
        let tx_count = client.get_transaction_count().await.unwrap();
        assert!(tx_count >= 0);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_poll_balance() {
        let client1 = SolanaTestClient::generate_new();
        let client2 = SolanaTestClient::generate_new();
        
        // Airdrop to first client
        client1.airdrop_sol_amount(2.0).await.unwrap();
        
        // Start transfer in background
        let client2_pubkey = client2.get_pubkey_string();
        let client1_clone = SolanaTestClient::from_bytes(&client1.get_keypair().to_bytes()).unwrap();
        
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let _ = client1_clone.transfer_sol_amount(&client2_pubkey, 0.5).await;
        });
        
        // Poll for balance
        let balance = client2.poll_until_expected_sol_balance(&client2.get_pubkey_string(), 0.1, 1, 10).await.unwrap();
        assert!(balance >= 0.1);
    }
} 