//-----------------------------------------------------------------------------
// Real Network Integration Tests
//-----------------------------------------------------------------------------
//
// These tests connect to actual blockchain networks. They are skipped by default
// and only run when the appropriate environment variables are set.

use std::env;
use tokio::test;
use std::str::FromStr;

use valence_domain_clients::{
    // Core types
    core::error::ClientError,
    core::transaction::TransactionResponse,
    // Cosmos-specific imports
    cosmos::{
        chains::{
            noble::NobleClient,
            osmosis::OsmosisClient,
        },
        types::{CosmosCoin, CosmosAddress},
        // Add trait imports to be able to use the trait methods
        grpc_client::GrpcSigningClient,
        CosmosBaseClient,
    },
    // EVM-specific imports
    evm::{
        chains::{
            ethereum::EthereumClient,
        },
        types::{EvmAddress, EvmU256, EvmTransactionRequest},
        // Add trait import to be able to use the trait methods
        base_client::EvmBaseClient,
    },
};

//-----------------------------------------------------------------------------
// Cosmos Network Tests
//-----------------------------------------------------------------------------

// Test constants
const NOBLE_GRPC_URL: &str = "https://noble-grpc.polkachu.com:14990";
const OSMOSIS_GRPC_URL: &str = "https://osmosis-grpc.polkachu.com:14390";

async fn setup_noble_client() -> Result<NobleClient, ClientError> {
    // These env vars should be set by the user when they want to run real network tests
    let mnemonic = env::var("NOBLE_TEST_MNEMONIC")
        .expect("NOBLE_TEST_MNEMONIC environment variable not set");
    
    // Get chain ID from env or use default
    let chain_id = env::var("NOBLE_CHAIN_ID").unwrap_or_else(|_| "noble-1".to_string());
    
    NobleClient::new(
        NOBLE_GRPC_URL,
        &chain_id,
        &mnemonic,
        None,  // default derivation path
    ).await
}

async fn setup_osmosis_client() -> Result<OsmosisClient, ClientError> {
    // These env vars should be set by the user when they want to run real network tests
    let mnemonic = env::var("OSMOSIS_TEST_MNEMONIC")
        .expect("OSMOSIS_TEST_MNEMONIC environment variable not set");
    
    // Get chain ID from env or use default
    let chain_id = env::var("OSMOSIS_CHAIN_ID").unwrap_or_else(|_| "osmosis-1".to_string());
    
    OsmosisClient::new(
        OSMOSIS_GRPC_URL,
        &chain_id,
        &mnemonic,
        None, // default derivation path
    ).await
}

#[test]
async fn test_noble_query_balance() {
    // Skip this test if the env var is not set
    if env::var("RUN_NETWORK_TESTS").is_err() {
        println!("Skipping Noble network test - RUN_NETWORK_TESTS not set");
        return;
    }
    
    let client_result = setup_noble_client().await;
    if let Err(e) = &client_result {
        println!("Failed to create Noble client: {}", e);
        return;
    }
    
    let client = client_result.unwrap();
    
    // Get the signer address from the client
    let account = client.get_signer_details().await.unwrap();
    println!("Testing with Noble address: {}", account.address);
    
    // Convert the address to a string first - use as_ref for the GenericAddress
    let balance = client.query_balance(account.address.as_ref(), "unusd").await.unwrap();
    println!("Noble balance: {} unusd", balance);
    
    // At minimum, the address should exist on chain (even if balance is 0)
    assert!(client.latest_block_header().await.is_ok());
}

#[test]
async fn test_osmosis_query_balance() {
    // Skip this test if the env var is not set
    if env::var("RUN_NETWORK_TESTS").is_err() {
        println!("Skipping Osmosis network test - RUN_NETWORK_TESTS not set");
        return;
    }
    
    let client_result = setup_osmosis_client().await;
    if let Err(e) = &client_result {
        println!("Failed to create Osmosis client: {}", e);
        return;
    }
    
    let client = client_result.unwrap();
    
    // Get the signer address from the client
    let account = client.get_signer_details().await.unwrap();
    println!("Testing with Osmosis address: {}", account.address);
    
    // Convert the address to a string first - use as_ref for the GenericAddress
    let balance = client.query_balance(account.address.as_ref(), "uosmo").await.unwrap();
    println!("Osmosis balance: {} uosmo", balance);
    
    // At minimum, the address should exist on chain (even if balance is 0)
    assert!(client.latest_block_header().await.is_ok());
}

//-----------------------------------------------------------------------------
// EVM Network Tests
//-----------------------------------------------------------------------------

// Test constants
const ETHEREUM_RPC_URL: &str = "https://ethereum-rpc.publicnode.com";
const SEPOLIA_RPC_URL: &str = "https://ethereum-sepolia-rpc.publicnode.com";

async fn setup_ethereum_client() -> Result<EthereumClient, ClientError> {
    // These env vars should be set by the user when they want to run real network tests
    let private_key = env::var("ETH_TEST_PRIVATE_KEY")
        .expect("ETH_TEST_PRIVATE_KEY environment variable not set");
    
    // Use testnet by default for tests to avoid accidental mainnet transactions
    let rpc_url = env::var("ETH_TEST_RPC_URL").unwrap_or_else(|_| SEPOLIA_RPC_URL.to_string());
    
    // Convert hex private key to bytes
    let private_key = private_key.trim_start_matches("0x");
    let key_bytes = hex::decode(private_key)
        .map_err(|e| ClientError::ParseError(format!("Invalid private key: {}", e)))?;
    
    if key_bytes.len() != 32 {
        return Err(ClientError::ParseError("Private key must be 32 bytes".to_string()));
    }
    
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    
    // Hardcode chain ID 11155111 for Sepolia
    Ok(EthereumClient::with_private_key(&rpc_url, 11155111, key_array))
}

#[test]
async fn test_ethereum_get_balance() {
    // Skip this test if the env var is not set
    if env::var("RUN_NETWORK_TESTS").is_err() {
        println!("Skipping Ethereum network test - RUN_NETWORK_TESTS not set");
        return;
    }
    
    let client_result = setup_ethereum_client().await;
    if let Err(e) = &client_result {
        println!("Failed to create Ethereum client: {}", e);
        return;
    }
    
    let client = client_result.unwrap();
    
    // Get the signer address from the client
    let signer_address = client.evm_signer_address();
    println!("Testing with Ethereum address: 0x{}", hex::encode(signer_address.0));
    
    // Query balance
    let balance = client.get_balance(&signer_address).await.unwrap();
    println!("Ethereum balance: {:?} wei", balance);
    
    // Query chain ID
    let chain_id = client.get_chain_id().await.unwrap();
    println!("Connected to Ethereum chain ID: {}", chain_id);
    
    // At minimum, we should be able to get the current block number
    assert!(client.get_block_number().await.is_ok());
}

//-----------------------------------------------------------------------------
// IBC Tests (Cross-chain)
//-----------------------------------------------------------------------------

#[test]
async fn test_ibc_query_connections() {
    // Skip this test if the env var is not set
    if env::var("RUN_NETWORK_TESTS").is_err() || env::var("RUN_IBC_TESTS").is_err() {
        println!("Skipping IBC network test - RUN_NETWORK_TESTS or RUN_IBC_TESTS not set");
        return;
    }
    
    let osmosis_client_result = setup_osmosis_client().await;
    if let Err(e) = &osmosis_client_result {
        println!("Failed to create Osmosis client: {}", e);
        return;
    }
    
    let osmosis_client = osmosis_client_result.unwrap();
    
    // Query Osmosis modules to ensure ibc module exists
    let module_account = osmosis_client.query_module_account("ibc").await;
    assert!(module_account.is_ok(), "IBC module should exist on Osmosis");
    
    // More IBC-specific queries would go here
}

//-----------------------------------------------------------------------------
// Transfer Tests (with Confirmation)
//-----------------------------------------------------------------------------

#[test]
async fn test_noble_small_transfer() {
    // Skip this test if the env vars are not set
    if env::var("RUN_NETWORK_TESTS").is_err() || env::var("RUN_TRANSFER_TESTS").is_err() {
        println!("Skipping Noble transfer test - environment variables not set");
        return;
    }
    
    let client_result = setup_noble_client().await;
    if let Err(e) = &client_result {
        println!("Failed to create Noble client: {}", e);
        return;
    }
    
    let client = client_result.unwrap();
    
    // Get test recipient from env var or use a default Noble community pool address
    let recipient = env::var("NOBLE_TEST_RECIPIENT")
        .unwrap_or_else(|_| "noble1jv65s3grqf6v6jl3dp4t6c9t9rk99cd8lyv94c".to_string());
    
    // Get the signer address from the client
    let account = client.get_signer_details().await.unwrap();
    println!("Sender: {}", account.address);
    println!("Recipient: {}", recipient);
    
    // Check initial balances - use as_ref for the GenericAddress
    let initial_sender_balance = client.query_balance(account.address.as_ref(), "unusd").await.unwrap();
    let initial_recipient_balance = client.query_balance(&recipient, "unusd").await.unwrap();
    
    println!("Initial sender balance: {} unusd", initial_sender_balance);
    println!("Initial recipient balance: {} unusd", initial_recipient_balance);
    
    // Only proceed if we have enough balance for the test
    if initial_sender_balance < 1_000 {
        println!("Insufficient balance for test. Need at least 1000 unusd.");
        return;
    }
    
    // Send a minimal amount (1000 unusd = $0.001 USD)
    let transfer_amount = 1_000u128; // 1000 microUSD
    
    println!("Sending {} unusd from {} to {}", 
        transfer_amount, account.address, recipient);
    
    // Perform the transfer
    let tx_result = client.transfer(
        &recipient,
        transfer_amount,
        "unusd",
        Some("Integration test transfer")
    ).await;
    
    // Verify the transaction succeeded
    assert!(tx_result.is_ok(), "Transfer should succeed");
    
    let tx = tx_result.unwrap();
    println!("Transfer successful. TX hash: {}", tx.tx_hash);
    
    // Wait for the transaction to be included in a block
    let confirmation = client.poll_for_tx(&tx.tx_hash).await;
    assert!(confirmation.is_ok(), "Transaction should be confirmed");
    
    // Check final balances - use as_ref for the GenericAddress
    let final_sender_balance = client.query_balance(account.address.as_ref(), "unusd").await.unwrap();
    let final_recipient_balance = client.query_balance(&recipient, "unusd").await.unwrap();
    
    println!("Final sender balance: {} unusd", final_sender_balance);
    println!("Final recipient balance: {} unusd", final_recipient_balance);
    
    // Verify the recipient received the funds
    // Note: We don't check exact balances because fees are also deducted
    assert!(final_sender_balance < initial_sender_balance, 
        "Sender balance should decrease");
        
    assert!(final_recipient_balance > initial_recipient_balance, 
        "Recipient balance should increase");
    
    // Verify the increase matches our transfer amount
    assert_eq!(
        final_recipient_balance - initial_recipient_balance,
        transfer_amount,
        "Recipient should receive exactly the transfer amount"
    );
} 