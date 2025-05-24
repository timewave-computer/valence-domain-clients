//-----------------------------------------------------------------------------
// Base Client Unit Tests and Integration Tests
//-----------------------------------------------------------------------------

use std::env;
use std::str::FromStr;

use mockall::predicate::*;
use mockall::mock;

use valence_domain_clients::{
    core::error::ClientError,
    evm::chains::base::{BaseClient, BaseNetwork, BaseNetworkConfig, BaseNetworkData, BaseNetworkInfo},
    evm::types::{
        EvmAddress, 
        EvmU256, 
        EvmBytes,
        EvmHash,
        EvmTransactionRequest,
    },
    core::transaction::TransactionResponse,
    EvmBaseClient,
};

// Create mock for testing
mock! {
    pub BaseTestClient {
        fn evm_signer_address(&self) -> EvmAddress;
        
        async fn get_balance(&self, address: EvmAddress) -> Result<EvmU256, ClientError>;
        
        async fn get_nonce(&self, address: EvmAddress) -> Result<u64, ClientError>;
        
        async fn send_raw_transaction(&self, tx_bytes: EvmBytes) -> Result<EvmHash, ClientError>;
        
        async fn send_transaction(&self, tx: EvmTransactionRequest) -> Result<TransactionResponse, ClientError>;
        
        async fn get_transaction(&self, tx_hash: EvmHash) -> Result<Option<TransactionResponse>, ClientError>;
        
        async fn wait_for_transaction_receipt(&self, tx_hash: EvmHash) -> Result<TransactionResponse, ClientError>;
        
        async fn get_block_number(&self) -> Result<u64, ClientError>;
        
        async fn get_chain_id(&self) -> Result<u64, ClientError>;
        
        async fn get_gas_price(&self) -> Result<EvmU256, ClientError>;
        
        async fn call_contract(
            &self,
            to: EvmAddress,
            data: EvmBytes,
            from: Option<EvmAddress>,
            block: Option<u64>,
        ) -> Result<EvmBytes, ClientError>;
        
        async fn estimate_gas(
            &self,
            to: Option<EvmAddress>,
            data: EvmBytes,
            value: Option<EvmU256>,
            from: Option<EvmAddress>,
        ) -> Result<EvmU256, ClientError>;
    }
}

//-----------------------------------------------------------------------------
// Unit Tests
//-----------------------------------------------------------------------------

#[tokio::test]
async fn test_base_client_initialization() {
    // Test initialization without calling actual Base network
    let client = BaseClient::new(BaseNetwork::Sepolia, None);
    
    assert!(client.is_ok(), "Failed to initialize Base client");
    
    let client = client.unwrap();
    assert_eq!(client.network_chain_id(), 84532);
    assert_eq!(client.explorer_url(), "https://sepolia-explorer.base.org");
}

#[tokio::test]
async fn test_base_mainnet_initialization() {
    // Test initialization of mainnet
    let client = BaseClient::new(BaseNetwork::Mainnet, None);
    
    assert!(client.is_ok(), "Failed to initialize Base Mainnet client");
    
    let client = client.unwrap();
    assert_eq!(client.network_chain_id(), 8453);
    assert_eq!(client.explorer_url(), "https://base.blockscout.com/");
}

#[tokio::test]
async fn test_base_from_config_file() {
    // Create a temporary config file
    let tmp_dir = tempfile::tempdir().unwrap();
    let config_path = tmp_dir.path().join("base_config.json");
    
    let config = BaseNetworkConfig {
        networks: BaseNetworkData {
            mainnet: BaseNetworkInfo {
                rpc: "https://custom-mainnet.base.org".to_string(),
                chain_id: 8453,
                currency_symbol: "ETH".to_string(),
                block_explorer: "https://custom-explorer.base.org".to_string(),
            },
            sepolia: BaseNetworkInfo {
                rpc: "https://custom-sepolia.base.org".to_string(),
                chain_id: 84532,
                currency_symbol: "ETH".to_string(),
                block_explorer: "https://custom-sepolia-explorer.base.org".to_string(),
            },
        },
    };
    
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, config_json).unwrap();
    
    // Test loading from config file
    let client = BaseClient::from_config_file(
        BaseNetwork::Mainnet,
        None,
        Some(&config_path),
    );
    
    assert!(client.is_ok(), "Failed to initialize Base client from config file");
}

#[tokio::test]
async fn test_mock_get_balance() {
    let mut mock = MockBaseTestClient::new();
    
    // Create a valid test address
    let address = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
    
    // Set up expectations
    mock.expect_get_balance()
        .with(eq(address.clone()))
        .times(1)
        .returning(|_| Ok(EvmU256::from_u64(1000000000000000000))); // 1 ETH
    
    // Call the method
    let result = mock.get_balance(address).await.unwrap();
    
    // Verify the result
    assert_eq!(result.0[0], 1000000000000000000);
    assert_eq!(result.0[1], 0);
    assert_eq!(result.0[2], 0);
    assert_eq!(result.0[3], 0);
}

#[tokio::test]
async fn test_mock_get_block_number() {
    let mut mock = MockBaseTestClient::new();
    
    // Set up expectations
    mock.expect_get_block_number()
        .times(1)
        .returning(|| Ok(12345678));
    
    // Call the method
    let result = mock.get_block_number().await.unwrap();
    
    // Verify the result
    assert_eq!(result, 12345678);
}

#[tokio::test]
async fn test_mock_get_chain_id() {
    let mut mock = MockBaseTestClient::new();
    
    // Set up expectations for Sepolia
    mock.expect_get_chain_id()
        .times(1)
        .returning(|| Ok(84532));
    
    // Call the method
    let result = mock.get_chain_id().await.unwrap();
    
    // Verify the result
    assert_eq!(result, 84532);
}

#[tokio::test]
async fn test_mock_transaction() {
    let mut mock = MockBaseTestClient::new();
    
    // Test addresses
    let sender = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
    let recipient = EvmAddress::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
    
    // Transaction hash
    let tx_hash = EvmHash::from_str("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap();
    
    // Transaction request
    let tx_request = EvmTransactionRequest {
        from: sender.clone(),
        to: Some(recipient.clone()),
        value: Some(EvmU256::from_u64(100000000000000000)), // 0.1 ETH
        gas_limit: Some(EvmU256::from_u64(21000)),
        gas_price: Some(EvmU256::from_u64(1000000000)), // 1 Gwei
        nonce: None,
        data: None,
        chain_id: Some(84532),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    
    // Mock transaction response
    let tx_response = TransactionResponse {
        tx_hash: tx_hash.to_string(),
        height: 12345678,
        gas_wanted: Some(21000),
        gas_used: Some(21000),
        code: Some(0),
        events: vec![],
        data: Some("{}".to_string()),
        raw_log: None,
        timestamp: 1677721600, // 2023-03-02 00:00:00
        block_hash: Some("0x9876543210abcdef9876543210abcdef9876543210abcdef9876543210abcdef".to_string()),
        original_request_payload: None,
    };
    
    // Set up expectations
    let recipient_clone = recipient.clone();
    let tx_response_clone1 = tx_response.clone();
    let tx_response_clone2 = tx_response.clone();
    
    mock.expect_send_transaction()
        .with(function(move |tx: &EvmTransactionRequest| {
            tx.to.is_some() && 
            tx.to.as_ref().unwrap() == &recipient_clone &&
            tx.value.is_some() &&
            tx.value.as_ref().unwrap().0[0] == 100000000000000000 &&
            tx.chain_id.unwrap() == 84532
        }))
        .times(1)
        .returning(move |_| Ok(tx_response_clone1.clone()));
    
    // Mock get transaction
    let tx_hash_clone = tx_hash.clone();
    mock.expect_get_transaction()
        .with(eq(tx_hash_clone))
        .times(1)
        .returning(move |_| Ok(Some(tx_response_clone2.clone())));
    
    // Call the methods
    let send_result = mock.send_transaction(tx_request).await.unwrap();
    let get_result = mock.get_transaction(tx_hash.clone()).await.unwrap().unwrap();
    
    // Verify the results
    assert_eq!(send_result.tx_hash, tx_hash.to_string());
    assert_eq!(send_result.height, 12345678);
    assert_eq!(get_result.tx_hash, tx_hash.to_string());
    assert_eq!(get_result.height, 12345678);
}

//-----------------------------------------------------------------------------
// Integration Tests - Enable by setting BASE_INTEGRATION_TEST=1
//-----------------------------------------------------------------------------

#[tokio::test]
async fn test_base_sepolia_integration() {
    // Skip test if integration tests are not enabled
    if env::var("BASE_INTEGRATION_TEST").is_err() {
        println!("Skipping Base integration test (set BASE_INTEGRATION_TEST=1 to run)");
        return;
    }

    // Use custom RPC URL if provided, otherwise use default
    let rpc_url = env::var("BASE_RPC_URL").ok();
    
    let client = if let Some(url) = rpc_url {
        // Create client with custom URL
        let config = BaseNetworkConfig {
            networks: BaseNetworkData {
                mainnet: BaseNetworkInfo {
                    rpc: url.clone(),
                    chain_id: 8453,
                    currency_symbol: "ETH".to_string(),
                    block_explorer: "https://base.blockscout.com/".to_string(),
                },
                sepolia: BaseNetworkInfo {
                    rpc: url,
                    chain_id: 84532,
                    currency_symbol: "ETH".to_string(),
                    block_explorer: "https://sepolia-explorer.base.org".to_string(),
                },
            },
        };
        
        let tmp_dir = tempfile::tempdir().unwrap();
        let config_path = tmp_dir.path().join("base_config.json");
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, config_json).unwrap();
        
        BaseClient::from_config_file(
            BaseNetwork::Sepolia,
            None,
            Some(&config_path),
        ).expect("Failed to create Base client with custom URL")
    } else {
        // Use default URL
        BaseClient::new(
            BaseNetwork::Sepolia,
            None,
        ).expect("Failed to create Base client with default URL")
    };
    
    // Test basic RPC calls
    let block_number = client.get_block_number().await
        .expect("Failed to get block number");
    
    println!("Base Sepolia current block number: {}", block_number);
    assert!(block_number > 0, "Block number should be greater than 0");
    
    let chain_id = client.get_chain_id().await
        .expect("Failed to get chain ID");
    
    println!("Base Sepolia chain ID: {}", chain_id);
    assert_eq!(chain_id, 84532, "Chain ID should be 84532 for Base Sepolia");
    
    // Use a known address to test balance query (the zero address should work)
    let zero_address = EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap();
    let balance = client.get_balance(&zero_address).await
        .expect("Failed to get balance");
    
    println!("Balance of zero address on Base Sepolia: {:?}", balance.0);
} 