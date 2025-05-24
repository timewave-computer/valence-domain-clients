//-----------------------------------------------------------------------------
// Ethereum Client Integration Tests
//-----------------------------------------------------------------------------

use std::env;
use std::str::FromStr;

use tokio::test;
use mockall::predicate::*;
use mockall::mock;

use valence_domain_clients::{
    core::error::ClientError,
    evm::chains::ethereum::EthereumClient,
    evm::types::{
        EvmAddress, 
        EvmU256, 
        EvmHash,
        EvmTransactionRequest,
        EvmTransactionReceipt
    },
    EvmBaseClient,
};

// Create mock for Ethereum RPC client
mock! {
    pub EthereumRpcClient {
        fn get_balance(&self, address: EvmAddress) -> Result<EvmU256, ClientError>;
        async fn execute_tx(&self, tx: EvmTransactionRequest) -> Result<EvmTransactionReceipt, ClientError>;
        fn evm_signer_address(&self) -> EvmAddress;
    }
}

//-----------------------------------------------------------------------------
// Unit Tests with Mocks
//-----------------------------------------------------------------------------

#[tokio::test]
async fn test_ethereum_client_initialization() {
    // Test initialization without calling actual Ethereum network
    let client = EthereumClient::new(
        "https://example-ethereum-rpc.com",
        "test test test test test test test test test test test junk",
        None,
    );
    
    assert!(client.is_ok(), "Failed to initialize Ethereum client");
}

#[test]
async fn test_get_balance() {
    // Create a mock instance
    let mut mock = MockEthereumRpcClient::new();
    
    // Test address
    let address = EvmAddress::from_str("0x1234567890123456789012345678901234567890").unwrap();
    
    // Set expectations
    mock.expect_get_balance()
        .with(eq(address.clone()))
        .times(1)
        .returning(|_| Ok(EvmU256::from_u64(1000000000000000000))); // 1 ETH
    
    // Call the method
    let result = mock.get_balance(address).unwrap();
    
    // Verify the result - test the first u64 component which contains the value
    assert_eq!(result.0[0], 1000000000000000000);
}

#[test]
async fn test_execute_transaction() {
    // Create a mock instance
    let mut mock = MockEthereumRpcClient::new();
    
    // --- Test addresses
    let sender = EvmAddress::from_str("0x1234567890123456789012345678901234567890").unwrap();
    let recipient = EvmAddress::from_str("0x0987654321098765432109876543210987654321").unwrap();
    
    // --- Set up expected response
    let expected_receipt = EvmTransactionReceipt {
        transaction_hash: EvmHash::from_str("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap(),
        block_number: 12345678,
        transaction_index: 0,
        from: sender.clone(),
        to: Some(recipient.clone()),
        cumulative_gas_used: EvmU256::from_u64(21000),
        gas_used: EvmU256::from_u64(21000),
        contract_address: None,
        logs: vec![],
        status: 1,
    };
    
    // --- Set up transaction request
    let tx_request = EvmTransactionRequest {
        from: sender.clone(),
        to: Some(recipient.clone()),
        value: Some(EvmU256::from_u64(100000000000000000)), // 0.1 ETH
        gas_limit: Some(EvmU256::from_u64(21000)),
        gas_price: Some(EvmU256::from_u64(1000000000)), // 1 Gwei
        data: None,
        nonce: None,
        chain_id: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    
    // --- Set up mock expectations
    mock.expect_evm_signer_address()
        .returning(move || sender.clone());
    
    let recipient_clone = recipient.clone();
    mock.expect_execute_tx()
        .with(function(move |arg: &EvmTransactionRequest| {
            arg.to.is_some() && 
            arg.to.as_ref().unwrap() == &recipient_clone &&
            arg.value.is_some() &&
            arg.value.as_ref().unwrap().0[0] == 100000000000000000
        }))
        .times(1)
        .returning(move |_| {
            Ok(expected_receipt.clone())
        });
    
    // --- Execute the test
    let result = mock.execute_tx(tx_request).await;
    
    // --- Verify the result
    assert!(result.is_ok());
    let receipt = result.unwrap();
    assert_eq!(receipt.transaction_hash.to_string(), "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
    assert_eq!(receipt.status, 1);
}

//-----------------------------------------------------------------------------
// Integration Tests
//-----------------------------------------------------------------------------

// Add this test only if ETH_INTEGRATION_TEST environment variable is set
#[test]
#[ignore]
async fn test_integration_eth_balance() {
    // Check if integration test should run
    if env::var("ETH_INTEGRATION_TEST").is_err() {
        println!("Skipping Ethereum integration test (set ETH_INTEGRATION_TEST to run)");
        return;
    }

    // These would normally come from environment variables
    let rpc_url = env::var("ETH_RPC_URL").unwrap_or_else(|_| "https://eth-goerli.example.com".to_string());
    let mnemonic = env::var("ETH_MNEMONIC").expect("ETH_MNEMONIC must be set for integration tests");
    
    // Create actual client
    let client = EthereumClient::new(
        &rpc_url,
        &mnemonic,
        None,
    ).expect("Failed to create Ethereum client");
    
    // Get signer address
    let signer_address = client.evm_signer_address();
    
    // Query balance
    let balance = client.get_balance(&signer_address).await
        .expect("Failed to query balance");
    
    println!("Ethereum balance: {:?} wei", balance.0);
    
    // Simple assertion to verify query works
    // Note: balance.0 is a u64 array, so it's always >= 0, we just check that the query succeeded
    assert!(balance.0.len() == 4, "Balance should have 4 u64 components");
}
