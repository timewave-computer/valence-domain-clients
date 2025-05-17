//-----------------------------------------------------------------------------
// Osmosis Client Integration Tests
//-----------------------------------------------------------------------------

use std::env;

use mockall::mock;
use mockall::predicate::*;
use tokio::test;

use valence_domain_clients::{
    core::error::ClientError,
    core::transaction::TransactionResponse,
    cosmos::chains::osmosis::OsmosisClient,
    cosmos::grpc_client::GrpcSigningClient,
    cosmos::types::{CosmosAddress, CosmosCoin},
    CosmosBaseClient,
};

// Create mock for Osmosis gRPC client
mock! {
    pub OsmosisGrpcClient {
        async fn query_balance(&self, address: &str, denom: &str) -> Result<u128, ClientError>;
        async fn swap_tokens(
            &self,
            token_in_denom: &str,
            token_out_denom: &str,
            token_in_amount: u128,
            minimum_out_amount: u128
        ) -> Result<TransactionResponse, ClientError>;
        async fn add_liquidity(
            &self,
            pool_id: u64,
            tokens: Vec<CosmosCoin>
        ) -> Result<TransactionResponse, ClientError>;
        async fn get_signer_address(&self) -> Result<CosmosAddress, ClientError>;
    }
}

//-----------------------------------------------------------------------------
// Unit Tests with Mocks
//-----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "Requires connection to an Osmosis node"]
async fn test_osmosis_client_initialization() {
    if env::var("MNEMONIC").is_err() {
        println!("Skipping Osmosis client test (MNEMONIC not set)");
        return;
    }

    // Use a local constructor for the client
    let client = OsmosisClient::new(
        "https://osmosis-grpc.polkachu.com:14590",
        "osmosis-1",
        &env::var("MNEMONIC").unwrap_or_default(),
        None,
    )
    .await;

    assert!(client.is_ok(), "Failed to initialize Osmosis client");
}

#[test]
async fn test_query_balance() {
    // Create a mock instance
    let mut mock = MockOsmosisGrpcClient::new();

    // --- Set expectations
    mock.expect_query_balance()
        .with(eq("osmo1test123"), eq("uosmo"))
        .times(1)
        .returning(|_, _| Ok(5_000_000u128));

    // Call the method
    let result = mock.query_balance("osmo1test123", "uosmo").await;

    // Verify the result
    assert_eq!(result.unwrap(), 5_000_000u128);
}

#[test]
async fn test_swap_tokens() {
    // Create a mock instance
    let mut mock = MockOsmosisGrpcClient::new();

    // --- Setup expected transaction response
    let expected_tx = TransactionResponse {
        tx_hash: "ABCDEF1234567890".to_string(),
        height: 12345,
        gas_wanted: Some(200000),
        gas_used: Some(150000),
        events: vec![],
        code: None,
        raw_log: Some("success".to_string()),
        data: None,
        block_hash: Some("block123".to_string()),
        timestamp: 1672574400, // Unix timestamp for "2023-01-01T12:00:00Z"
        original_request_payload: None,
    };

    // --- Set expectations
    mock.expect_swap_tokens()
        .with(
            eq("uosmo"),
            eq("ibc/atom"),
            eq(1_000_000u128),
            eq(950_000u128),
        )
        .times(1)
        .returning(move |_, _, _, _| Ok(expected_tx.clone()));

    // Call the method
    let result = mock
        .swap_tokens("uosmo", "ibc/atom", 1_000_000u128, 950_000u128)
        .await;

    // --- Verify the result
    assert!(result.is_ok());
    let tx = result.unwrap();
    assert_eq!(tx.tx_hash, "ABCDEF1234567890");
    assert_eq!(tx.height, 12345);
}

#[test]
async fn test_add_liquidity() {
    // Create a mock instance
    let mut mock = MockOsmosisGrpcClient::new();

    // --- Setup expected transaction response
    let expected_tx = TransactionResponse {
        tx_hash: "LIQUIDITY987654".to_string(),
        height: 9876543,
        gas_used: Some(200000),
        gas_wanted: Some(250000),
        events: Vec::new(),
        code: None,
        raw_log: None,
        data: None,
        block_hash: Some("block123".to_string()),
        timestamp: 1672574400, // Unix timestamp for "2023-01-01T12:00:00Z"
        original_request_payload: None,
    };

    // --- Create test tokens
    let tokens = vec![
        CosmosCoin {
            denom: "uosmo".to_string(),
            amount: 10_000_000u128,
        },
        CosmosCoin {
            denom: "ibc/atom".to_string(),
            amount: 2_000_000u128,
        },
    ];

    // --- Set expectations
    mock.expect_add_liquidity()
        .with(
            eq(1),
            function(|arg: &Vec<CosmosCoin>| {
                arg.len() == 2
                    && arg[0].denom == "uosmo"
                    && arg[0].amount == 10_000_000u128
                    && arg[1].denom == "ibc/atom"
                    && arg[1].amount == 2_000_000u128
            }),
        )
        .times(1)
        .returning(move |_, _| Ok(expected_tx.clone()));

    // Call the method
    let result = mock.add_liquidity(1, tokens).await;

    // --- Verify the result
    assert!(result.is_ok());
    let tx = result.unwrap();
    assert_eq!(tx.tx_hash, "LIQUIDITY987654");
    assert_eq!(tx.height, 9876543);
}

//-----------------------------------------------------------------------------
// Integration Tests
//-----------------------------------------------------------------------------

// Add this test only if OSMOSIS_INTEGRATION_TEST environment variable is set
#[test]
#[ignore]
async fn test_integration_osmosis_balance() {
    // Check if integration test should run
    if env::var("OSMOSIS_INTEGRATION_TEST").is_err() {
        println!("Skipping Osmosis integration test (set OSMOSIS_INTEGRATION_TEST to run)");
        return;
    }

    // These would normally come from environment variables
    let grpc_url = env::var("OSMOSIS_GRPC_URL").unwrap_or_else(|_| {
        "https://osmosis-testnet-grpc.example.com:9090".to_string()
    });
    let chain_id = env::var("OSMOSIS_CHAIN_ID")
        .unwrap_or_else(|_| "osmosis-testnet-1".to_string());
    let mnemonic = env::var("OSMOSIS_MNEMONIC")
        .expect("OSMOSIS_MNEMONIC must be set for integration tests");

    // Create actual client
    let client = OsmosisClient::new(&grpc_url, &chain_id, &mnemonic, None)
        .await
        .expect("Failed to create Osmosis client");

    // Get signer address
    let signer = client
        .get_signer_details()
        .await
        .expect("Failed to get signer details");

    // Query balance
    let balance = client
        .query_balance(&signer.address.0, "uosmo")
        .await
        .expect("Failed to query balance");

    println!("Osmosis balance: {} uosmo", balance);

    // Simple assertion to verify query works
    assert!(balance > 0, "Balance query returned unexpected value");
}
