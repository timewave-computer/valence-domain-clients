//-----------------------------------------------------------------------------
// Gaia (Cosmos Hub) Client Tests
//-----------------------------------------------------------------------------

use std::env;

use valence_domain_clients::core::error::ClientError;
use valence_domain_clients::cosmos::chains::GaiaClient;
use valence_domain_clients::CosmosBaseClient;
use valence_domain_clients::cosmos::grpc_client::GrpcSigningClient;

/// Create a Cosmos Hub client for testing.
///
/// This function attempts to create a client using environment variables.
/// If the required variables are not set, it returns an error.
async fn create_test_client() -> Result<GaiaClient, ClientError> {
    let grpc_url = env::var("GAIA_GRPC_URL")
        .unwrap_or_else(|_| "http://localhost:9090".to_string());
    
    let chain_id = env::var("GAIA_CHAIN_ID")
        .unwrap_or_else(|_| "cosmoshub-4".to_string());
    
    let mnemonic = env::var("GAIA_MNEMONIC")
        .expect("GAIA_MNEMONIC must be set for tests");
    
    GaiaClient::new(&grpc_url, &chain_id, &mnemonic, None).await
}

#[tokio::test]
#[ignore] // Requires a connection to a Cosmos Hub node
async fn test_gaia_connection() {
    if env::var("GAIA_MNEMONIC").is_err() {
        println!("Skipping Cosmos Hub connection test (GAIA_MNEMONIC not set)");
        return;
    }
    
    let client = create_test_client().await;
    assert!(client.is_ok(), "Failed to create Cosmos Hub client");
}

#[tokio::test]
#[ignore] // Requires a connection to a Cosmos Hub node
async fn test_gaia_query_balance() {
    if env::var("GAIA_MNEMONIC").is_err() {
        println!("Skipping Cosmos Hub balance test (GAIA_MNEMONIC not set)");
        return;
    }
    
    let client = create_test_client().await.expect("Failed to create client");
    let signer = client.get_signer_details().await.expect("Failed to get signer details");
    
    // Convert GenericAddress to &str for query methods
    let address_str = signer.address.to_string();
    let balance = client.query_balance(&address_str, "uatom").await;
    assert!(balance.is_ok());
    assert!(balance.unwrap() > 0);
}

#[tokio::test]
#[ignore] // Requires a connection to a Cosmos Hub node
async fn test_gaia_latest_block() {
    if env::var("GAIA_MNEMONIC").is_err() {
        println!("Skipping Cosmos Hub block test (GAIA_MNEMONIC not set)");
        return;
    }
    
    let client = create_test_client().await.expect("Failed to create client");
    
    // Query latest block
    let block = client.latest_block_header().await;
    assert!(block.is_ok(), "Failed to query latest block: {:?}", block.err());
    println!("Cosmos Hub latest block height: {}", block.unwrap().height);
}

#[tokio::test]
#[ignore] // Requires a connection to a Cosmos Hub node
async fn test_gaia_staking_queries() {
    if env::var("GAIA_MNEMONIC").is_err() {
        println!("Skipping Cosmos Hub staking test (GAIA_MNEMONIC not set)");
        return;
    }
    
    // This test doesn't actually perform delegation operations,
    // but verifies that the client can connect to the staking module
    // and query the distribution module account
    
    let client = create_test_client().await.expect("Failed to create client");
    
    // Query the distribution module account, which holds staking rewards
    let distribution_account = client.query_module_account("distribution").await;
    assert!(distribution_account.is_ok(), 
        "Failed to query distribution module account: {:?}", distribution_account.err());
    
    println!("Distribution module account: {:?}", distribution_account.unwrap());
}
