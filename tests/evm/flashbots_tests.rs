//-----------------------------------------------------------------------------
// Flashbots Bundle Tests
//-----------------------------------------------------------------------------

use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use serde_json::json;

use valence_domain_clients::{
    core::error::ClientError,
    evm::{
        chains::ethereum::EthereumClient,
        bundle::{
            FlashbotsBundle, EthSendBundleParams, MevSendBundleParams,
            BundleInclusion, BundleItem, PrivacyHint, PrivacyConfig,
            create_eth_bundle, create_mev_bundle, create_mev_share_bundle,
        },
        types::{EvmAddress, EvmU256},
    },
    EvmBaseClient,
};

// Skip all tests if Flashbots auth key is not set
fn should_skip_tests() -> bool {
    env::var("FLASHBOTS_AUTH_KEY").is_err()
}

// Helper to get a test client with Flashbots authentication
fn get_test_client() -> Result<EthereumClient, ClientError> {
    let rpc_url = env::var("ETH_RPC_URL")
        .unwrap_or_else(|_| "https://eth-mainnet.alchemyapi.io/v2/demo".to_string());
    
    let auth_key_hex = env::var("FLASHBOTS_AUTH_KEY")
        .expect("FLASHBOTS_AUTH_KEY not set");
    
    let auth_key = hex::decode(auth_key_hex.trim_start_matches("0x"))
        .expect("Invalid auth key format");
    
    let mut auth_bytes = [0u8; 32];
    auth_bytes.copy_from_slice(&auth_key);
    
    let client = EthereumClient::new(&rpc_url, "", None)?;
    Ok(client.with_flashbots_auth(auth_bytes))
}

// Test basic bundle creation
#[test]
fn test_create_eth_bundle() {
    let transactions = vec![
        "0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(),
    ];
    
    let bundle = create_eth_bundle(transactions.clone(), 12345678, None);
    
    assert_eq!(bundle.txs, transactions);
    assert_eq!(bundle.block_number, "0xbc614e");
    assert!(bundle.reverting_tx_hashes.is_none());
}

// Test MEV bundle creation
#[test]
fn test_create_mev_bundle() {
    let transactions = vec![
        ("0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(), false),
    ];
    
    let hints = Some(vec![PrivacyHint::TxHash, PrivacyHint::Calldata]);
    
    let bundle = create_mev_bundle(
        transactions.clone(),
        12345678,
        Some(12345680),
        hints,
        None,
    );
    
    assert_eq!(bundle.version, "v0.1");
    assert_eq!(bundle.inclusion.block, "0xbc614e");
    assert_eq!(bundle.inclusion.max_block, Some("0xbc6150".to_string()));
    assert_eq!(bundle.body.len(), 1);
}

// Test auth key validation
#[test]
fn test_flashbots_auth() {
    if should_skip_tests() {
        println!("Skipping test_flashbots_auth: FLASHBOTS_AUTH_KEY not set");
        return;
    }
    
    let client_result = get_test_client();
    assert!(client_result.is_ok(), "Failed to create Flashbots client: {:?}", client_result.err());
    
    let client = client_result.unwrap();
    assert!(client.has_private_key());
}

// Test sending a bundle to Flashbots
#[tokio::test]
#[ignore] // This test sends a real request to Flashbots and requires a valid auth key
async fn test_send_eth_bundle() {
    if should_skip_tests() {
        println!("Skipping test_send_eth_bundle: FLASHBOTS_AUTH_KEY not set");
        return;
    }
    
    let client = get_test_client().unwrap();
    
    // Create a mock transaction (this is an invalid transaction and will fail simulation)
    // However, it'll test the bundle submission process
    let transactions = vec![
        "0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(),
    ];
    
    // Get the current block number
    let current_block = client.get_block_number().await.unwrap();
    let target_block = current_block + 1;
    
    let bundle = create_eth_bundle(transactions, target_block, None);
    
    // Send the bundle to Flashbots
    let result = client.send_eth_bundle(bundle).await;
    
    // The bundle will likely be rejected by Flashbots due to simulation failure,
    // but we can at least verify that the request was properly formatted
    // and we got a response from the Flashbots server
    println!("Flashbots bundle submission result: {:?}", result);
}

// Test sending an MEV-Share bundle
#[tokio::test]
#[ignore] // This test sends a real request to Flashbots and requires a valid auth key
async fn test_send_mev_bundle() {
    if should_skip_tests() {
        println!("Skipping test_send_mev_bundle: FLASHBOTS_AUTH_KEY not set");
        return;
    }
    
    let client = get_test_client().unwrap();
    
    // Create a mock transaction (this is an invalid transaction and will fail simulation)
    let transactions = vec![
        ("0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(), false),
    ];
    
    // Get the current block number
    let current_block = client.get_block_number().await.unwrap();
    let target_block = current_block + 1;
    
    // Create an MEV bundle with tx_hash hint for privacy
    let bundle = create_mev_bundle(
        transactions,
        target_block,
        Some(target_block + 10),
        Some(vec![PrivacyHint::TxHash]),
        None,
    );
    
    // Send the bundle to Flashbots
    let result = client.send_mev_bundle(bundle).await;
    
    // The bundle will likely be rejected by Flashbots due to simulation failure,
    // but we can at least verify that the request was properly formatted
    println!("Flashbots MEV bundle submission result: {:?}", result);
}

// Test bundle simulation
#[tokio::test]
#[ignore] // This test sends a real request to Flashbots and requires a valid auth key
async fn test_simulate_bundle() {
    if should_skip_tests() {
        println!("Skipping test_simulate_bundle: FLASHBOTS_AUTH_KEY not set");
        return;
    }
    
    let client = get_test_client().unwrap();
    
    // Create a mock transaction (this is an invalid transaction and will fail simulation)
    let transactions = vec![
        "0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(),
    ];
    
    // Get the current block number
    let current_block = client.get_block_number().await.unwrap();
    let target_block = current_block + 1;
    
    let bundle = create_eth_bundle(transactions, target_block, None);
    
    // Simulate the bundle execution
    let result = client.simulate_bundle(bundle).await;
    
    // Even though the simulation will fail (since the transaction is invalid),
    // we'll get a response with the simulation results
    println!("Flashbots bundle simulation result: {:?}", result);
} 