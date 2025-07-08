// Solana localnet client for testing against local Solana test validator
use crate::solana::{
    base_client::SolanaBaseClient,
    rpc_client::SolanaRpcClient,
    signing_client::{SolanaSigningClient, SolanaSigningClientImpl},
};
use async_trait::async_trait;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
};

/// Solana localnet client for development and testing
pub struct SolanaLocalnetClient {
    inner: SolanaSigningClientImpl,
}

impl SolanaLocalnetClient {
    /// Create a new localnet client
    pub fn new(keypair: Keypair) -> Self {
        let inner = SolanaSigningClientImpl::new(keypair, "http://localhost:8899");
        Self { inner }
    }
    
    /// Create a new localnet client with custom RPC URL
    pub fn with_rpc_url(keypair: Keypair, rpc_url: &str) -> Self {
        let inner = SolanaSigningClientImpl::new(keypair, rpc_url);
        Self { inner }
    }
    
    /// Create a new localnet client from private key bytes
    pub fn from_bytes(private_key: &[u8]) -> anyhow::Result<Self> {
        let inner = SolanaSigningClientImpl::from_bytes(private_key, "http://localhost:8899")?;
        Ok(Self { inner })
    }
    
    /// Create a new localnet client from base58 encoded private key
    pub fn from_base58(private_key: &str) -> anyhow::Result<Self> {
        let inner = SolanaSigningClientImpl::from_base58(private_key, "http://localhost:8899")?;
        Ok(Self { inner })
    }
    
    /// Generate a new client with random keypair
    pub fn generate_new() -> Self {
        let inner = SolanaSigningClientImpl::generate_new("http://localhost:8899");
        Self { inner }
    }
    
    /// Get the underlying keypair
    pub fn get_keypair(&self) -> &Keypair {
        self.inner.get_keypair()
    }
    
    /// Get the public key as a string
    pub fn get_pubkey_string(&self) -> String {
        self.inner.get_pubkey().to_string()
    }
}

#[async_trait]
impl SolanaRpcClient for SolanaLocalnetClient {
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
impl SolanaSigningClient for SolanaLocalnetClient {
    fn get_keypair(&self) -> &Keypair {
        self.inner.get_keypair()
    }
}

#[async_trait]
impl SolanaBaseClient for SolanaLocalnetClient {}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    
    const TEST_RPC_URL: &str = "http://localhost:8899";
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_client_creation() {
        let client = SolanaLocalnetClient::generate_new();
        assert!(!client.get_pubkey_string().is_empty());
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_get_latest_block_height() {
        let client = SolanaLocalnetClient::generate_new();
        let block_height = client.latest_block_height().await.unwrap();
        assert!(block_height > 0);
    }
    
    #[tokio::test]
    #[ignore = "requires local solana test validator"]
    async fn test_localnet_airdrop_and_balance() {
        let client = SolanaLocalnetClient::generate_new();
        
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
        let client1 = SolanaLocalnetClient::generate_new();
        let client2 = SolanaLocalnetClient::generate_new();
        
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
        let client = SolanaLocalnetClient::generate_new();
        
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
        let client = SolanaLocalnetClient::generate_new();
        
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
        let client1 = SolanaLocalnetClient::generate_new();
        let client2 = SolanaLocalnetClient::generate_new();
        
        // Airdrop to first client
        client1.airdrop_sol_amount(2.0).await.unwrap();
        
        // Start transfer in background
        let client2_pubkey = client2.get_pubkey_string();
        let client1_clone = SolanaLocalnetClient::from_bytes(&client1.get_keypair().to_bytes()).unwrap();
        
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let _ = client1_clone.transfer_sol_amount(&client2_pubkey, 0.5).await;
        });
        
        // Poll for balance
        let balance = client2.poll_until_expected_sol_balance(&client2.get_pubkey_string(), 0.1, 1, 10).await.unwrap();
        assert!(balance >= 0.1);
    }
} 