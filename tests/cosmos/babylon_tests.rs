//-----------------------------------------------------------------------------
// Babylon Client Tests
//-----------------------------------------------------------------------------

use std::env;

use valence_domain_clients::core::error::ClientError;
use valence_domain_clients::cosmos::chains::BabylonClient;
use valence_domain_clients::cosmos::grpc_client::GrpcSigningClient;
use valence_domain_clients::CosmosBaseClient;

/// Create a Babylon client for testing.
///
/// This function attempts to create a client using environment variables.
/// If the required variables are not set, it returns an error.
async fn create_test_client() -> Result<BabylonClient, ClientError> {
    let grpc_url = env::var("BABYLON_GRPC_URL")
        .unwrap_or_else(|_| "http://localhost:9090".to_string());

    let chain_id =
        env::var("BABYLON_CHAIN_ID").unwrap_or_else(|_| "bbn-test-1".to_string());

    let mnemonic = env::var("BABYLON_MNEMONIC")
        .expect("BABYLON_MNEMONIC must be set for tests");

    BabylonClient::new(&grpc_url, &chain_id, &mnemonic, None).await
}

#[tokio::test]
#[ignore] // Requires a connection to a Babylon node
async fn test_babylon_connection() {
    if env::var("BABYLON_MNEMONIC").is_err() {
        println!("Skipping Babylon connection test (BABYLON_MNEMONIC not set)");
        return;
    }

    let client = create_test_client().await;
    assert!(client.is_ok(), "Failed to create Babylon client");
}

#[tokio::test]
#[ignore] // Requires a connection to a Babylon node
async fn test_babylon_query_balance() {
    if env::var("BABYLON_MNEMONIC").is_err() {
        println!("Skipping Babylon balance test (BABYLON_MNEMONIC not set)");
        return;
    }

    let client = create_test_client().await.expect("Failed to create client");
    let signer = client
        .get_signer_details()
        .await
        .expect("Failed to get signer details");

    // Convert GenericAddress to &str for query methods
    let address_str = signer.address.to_string();
    let balance = client.query_balance(&address_str, "ubbn").await;
    assert!(balance.is_ok());
    let balance_value = balance.as_ref().unwrap();
    assert!(*balance_value > 0);

    // Convert GenericAddress to &str for BTC tap balance queries
    let btc_tap_balance = client.query_btc_tap_balance(&address_str).await;
    assert!(btc_tap_balance.is_ok());
    let tap_balance = btc_tap_balance.unwrap();

    println!("Babylon native token balance: {} ubbn", *balance_value);
    println!("Babylon BTC-TAP balance: {tap_balance} ubtc_tap");
}

#[tokio::test]
#[ignore] // Requires a connection to a Babylon node
async fn test_babylon_latest_block() {
    if env::var("BABYLON_MNEMONIC").is_err() {
        println!("Skipping Babylon block test (BABYLON_MNEMONIC not set)");
        return;
    }

    let client = create_test_client().await.expect("Failed to create client");

    // Query latest block
    let block = client.latest_block_header().await;
    assert!(
        block.is_ok(),
        "Failed to query latest block: {:?}",
        block.err()
    );
    println!("Babylon latest block height: {}", block.unwrap().height);
}
