// Example demonstrating Flashbots integration with valence-domain-clients
//-----------------------------------------------------------------------------
// Flashbots Bundle Example
//-----------------------------------------------------------------------------
//! This example demonstrates how to create and send Flashbots bundles on Ethereum
//! using the valence-domain-clients library.
//!
//! It shows:
//! 1. How to set up an Ethereum client with Flashbots authentication
//! 2. How to create a basic bundle for eth_sendBundle
//! 3. How to create a MEV-Share bundle with privacy hints
//! 4. How to simulate bundle execution
//! 5. How to submit bundles to Flashbots

use std::env;
use std::error::Error;

use valence_domain_clients::{
    evm::{
        bundle::{create_eth_bundle, create_mev_share_bundle, PrivacyHint},
        chains::ethereum::EthereumClient,
        FlashbotsBundleOperations,
    },
    EvmBaseClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //-----------------------------------------------------------------------------
    // Client Setup with Flashbots Authentication
    //-----------------------------------------------------------------------------
    println!("=== Flashbots Bundle Example ===");

    // Get Ethereum RPC URL from environment or use a default
    let rpc_url = env::var("ETH_RPC_URL")
        .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string());

    // Get Flashbots authentication key from environment
    // This is required for submitting bundles to Flashbots
    let flashbots_auth_key = env::var("FLASHBOTS_AUTH_KEY").unwrap_or_else(|_| {
        println!("FLASHBOTS_AUTH_KEY not set! Using a dummy key for demonstration.");
        "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string()
    });

    // Convert the authentication key to bytes
    let auth_key = hex::decode(flashbots_auth_key.trim_start_matches("0x"))
        .expect("Invalid auth key format");

    let mut auth_bytes = [0u8; 32];
    auth_bytes.copy_from_slice(&auth_key);

    // Create a new Ethereum client with Flashbots authentication
    let eth_client =
        EthereumClient::new(&rpc_url, "", None)?.with_flashbots_auth(auth_bytes);

    println!("Ethereum client initialized with Flashbots authentication");

    //-----------------------------------------------------------------------------
    // Get Current Block and Transaction Data
    //-----------------------------------------------------------------------------

    // Get the current block number to target the next block
    let current_block = eth_client.get_block_number().await?;
    let target_block = current_block + 1;

    println!(
        "Current block: {current_block}, targeting block: {target_block}"
    );

    // In a real scenario, you would have your own signed transactions to include
    // Here we use a placeholder transaction for demonstration purposes
    let placeholder_tx = "0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde\
        8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a\
        05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3";

    //-----------------------------------------------------------------------------
    // Create and Send a Basic Flashbots Bundle
    //-----------------------------------------------------------------------------
    println!("\nCreating a basic Flashbots bundle...");

    // Create a bundle for the next block
    let current_block_for_bundle = eth_client.get_block_number().await?;
    let target_block_for_bundle = current_block_for_bundle + 1;
    println!(
        "Current block for bundle: {current_block_for_bundle}, targeting block: {target_block_for_bundle}"
    );

    // Create EthSendBundleParams
    let eth_bundle_params = create_eth_bundle(
        vec![placeholder_tx.to_string()], // Transactions to include
        target_block_for_bundle,          // Target block number
        None,                             // No specific reverting_tx_hashes
    );

    println!("EthSendBundleParams created for block: 0x{target_block_for_bundle:x}");

    // Simulate the bundle
    match eth_client.simulate_bundle(eth_bundle_params.clone()).await { // Clone params for simulation
        Ok(simulation_result_map) => {
            println!("Simulation successful!");
            // Log the entire simulation result for inspection
            println!("Simulation result: {simulation_result_map:?}");
            // Example: Check for a specific key if you know what to expect
            if let Some(first_tx_sim) = simulation_result_map.get("some_expected_key_for_first_tx_hash_or_status") {
                println!("First transaction simulation details: {first_tx_sim:?}");
            } else {
                println!("Relevant simulation details not found directly, inspect the map above.");
            }
        }
        Err(e) => println!("Simulation failed: {e}"),
    }

    // Submit the bundle
    match eth_client.send_eth_bundle(eth_bundle_params).await { // Use original params for sending
        Ok(submitted_bundle_response) => {
            println!("Bundle submitted successfully: bundle_hash = {}", submitted_bundle_response.bundle_hash);
        }
        Err(e) => println!("Bundle submission failed: {e}"),
    }

    //-----------------------------------------------------------------------------
    // Create and Send a MEV-Share Bundle
    //-----------------------------------------------------------------------------
    println!("\nCreating an MEV-Share bundle with privacy settings...");

    // Create transactions with can_revert setting
    let transactions = vec![
        (placeholder_tx.to_string(), false), // This transaction must not revert
    ];

    // Set privacy hints for MEV-Share
    let privacy_hints = vec![
        PrivacyHint::TxHash,   // Share transaction hashes
        PrivacyHint::Calldata, // Share calldata
    ];

    // Specify builders to include
    let builders = vec![
        "flashbots".to_string(),   // Flashbots builder
        "builder0x69".to_string(), // Another builder
    ];

    // Create a refund configuration to get MEV refunds
    let refund_addresses = vec![
        // 90% of MEV goes to this address
        ("0x1234567890123456789012345678901234567890".to_string(), 90),
    ];

    // Create the MEV-Share bundle
    let mev_bundle = create_mev_share_bundle(
        transactions,
        target_block,
        Some(target_block + 3), // Valid for 3 blocks
        Some(privacy_hints),
        Some(builders),
        refund_addresses,
    );

    println!(
        "MEV-Share bundle created for blocks: {} to {}",
        target_block,
        target_block + 3
    );

    // Send the MEV-Share bundle to Flashbots
    println!("Sending MEV-Share bundle to Flashbots...");

    match eth_client.send_mev_bundle(mev_bundle).await {
        Ok(response) => {
            println!("MEV-Share bundle submitted successfully!");
            println!("Bundle hash: {}", response.bundle_hash);
        }
        Err(e) => {
            println!("MEV-Share bundle submission failed: {e}");
            println!("This is expected with our placeholder transaction.");
        }
    }

    println!("\nExample completed. In a real scenario, you would:");
    println!("1. Create properly signed transactions with ethers-rs or similar");
    println!("2. Use a valid Flashbots authentication key");
    println!("3. Monitor bundle inclusion status via eth_getTransactionReceipt");

    Ok(())
}
