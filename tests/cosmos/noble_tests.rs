//-----------------------------------------------------------------------------
// Noble Client Tests
//-----------------------------------------------------------------------------

use std::env;

use valence_domain_clients::{
    cosmos::chains::noble::NobleClient, cosmos::grpc_client::GrpcSigningClient,
    CosmosBaseClient,
};

// This test just verifies that NobleClient implements the necessary traits
#[test]
fn test_noble_client_implements_traits() {
    // These functions would fail to compile if NobleClient doesn't implement the traits
    fn assert_implements_cosmos_base_client<T: CosmosBaseClient>() {}
    fn assert_implements_grpc_signing_client<T: GrpcSigningClient>() {}

    // These will fail at compile time if NobleClient doesn't implement the traits
    assert_implements_cosmos_base_client::<NobleClient>();
    assert_implements_grpc_signing_client::<NobleClient>();
}

// This test would connect to a real Noble client if the environment variables are set
#[tokio::test]
#[ignore] // Ignored by default since it requires external connection
async fn test_noble_client_connection() {
    // Skip this test if no mnemonic is set
    if env::var("NOBLE_TEST_MNEMONIC").is_err() {
        println!("Skipping Noble connection test - NOBLE_TEST_MNEMONIC not set");
        return;
    }

    // Get connection parameters from environment variables or use defaults
    let grpc_url = env::var("NOBLE_GRPC_URL")
        .unwrap_or_else(|_| "https://noble-grpc.polkachu.com:14990".to_string());

    let chain_id =
        env::var("NOBLE_CHAIN_ID").unwrap_or_else(|_| "noble-1".to_string());

    let mnemonic = env::var("NOBLE_TEST_MNEMONIC").unwrap();

    // Create a Noble client with the specified parameters
    let noble_client = NobleClient::new(
        &grpc_url, &chain_id, &mnemonic, None, // Using default derivation path
    )
    .await;

    // Verify client creation was successful
    assert!(
        noble_client.is_ok(),
        "Failed to create Noble client: {:?}",
        noble_client.err()
    );
}
