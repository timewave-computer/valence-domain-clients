//-----------------------------------------------------------------------------
// Neutron Client Tests
//-----------------------------------------------------------------------------

use std::env;

use valence_domain_clients::core::error::ClientError;
use valence_domain_clients::cosmos::chains::NeutronClient;
use valence_domain_clients::CosmosBaseClient;
use valence_domain_clients::cosmos::grpc_client::GrpcSigningClient;

/// Create a Neutron client for testing.
///
/// This function attempts to create a client using environment variables.
/// If the required variables are not set, it returns an error.
async fn create_test_client() -> Result<NeutronClient, ClientError> {
    let grpc_url = env::var("NEUTRON_GRPC_URL")
        .unwrap_or_else(|_| "http://localhost:9090".to_string());
    
    let chain_id = env::var("NEUTRON_CHAIN_ID")
        .unwrap_or_else(|_| "neutron-1".to_string());
    
    let mnemonic = env::var("NEUTRON_MNEMONIC")
        .expect("NEUTRON_MNEMONIC must be set for tests");
    
    NeutronClient::new(&grpc_url, &chain_id, &mnemonic, None).await
}

#[tokio::test]
#[ignore] // Requires a connection to a Neutron node
async fn test_neutron_connection() {
    if env::var("NEUTRON_MNEMONIC").is_err() {
        println!("Skipping Neutron connection test (NEUTRON_MNEMONIC not set)");
        return;
    }
    
    let client = create_test_client().await;
    assert!(client.is_ok(), "Failed to create Neutron client");
}

#[tokio::test]
#[ignore] // Requires a connection to a Neutron node
async fn test_neutron_query_balance() {
    if env::var("NEUTRON_MNEMONIC").is_err() {
        println!("Skipping Neutron balance test (NEUTRON_MNEMONIC not set)");
        return;
    }
    
    let client = create_test_client().await.expect("Failed to create client");
    let signer = client.get_signer_details().await.expect("Failed to get signer details");
    
    // Convert GenericAddress to &str for query methods
    let address_str = signer.address.to_string();
    let balance = client.query_balance(&address_str, "untrn").await;
    assert!(balance.is_ok());
    assert!(balance.unwrap() > 0);
}

#[tokio::test]
#[ignore] // Requires a connection to a Neutron node
async fn test_neutron_latest_block() {
    if env::var("NEUTRON_MNEMONIC").is_err() {
        println!("Skipping Neutron block test (NEUTRON_MNEMONIC not set)");
        return;
    }
    
    let client = create_test_client().await.expect("Failed to create client");
    
    // Query latest block
    let block = client.latest_block_header().await;
    assert!(block.is_ok(), "Failed to query latest block: {:?}", block.err());
    println!("Neutron latest block height: {}", block.unwrap().height);
}

#[tokio::test]
#[ignore] // Requires a connection to a Neutron node and contract address
async fn test_neutron_contract_query() {
    if env::var("NEUTRON_MNEMONIC").is_err() || env::var("NEUTRON_TEST_CONTRACT").is_err() {
        println!("Skipping Neutron contract test (required env vars not set)");
        return;
    }
    
    let client = create_test_client().await.expect("Failed to create client");
    let contract_address = env::var("NEUTRON_TEST_CONTRACT").expect("NEUTRON_TEST_CONTRACT must be set");
    
    // Simple query to get contract info
    #[derive(serde::Serialize)]
    struct ContractInfoQuery {
        contract_info: EmptyObject,
    }
    
    #[derive(serde::Serialize)]
    struct EmptyObject {}
    
    let query = ContractInfoQuery { contract_info: EmptyObject {} };
    
    // This is a simple query that most CosmWasm contracts should support
    let result: serde_json::Value = client.query_contract(&contract_address, &query).await
        .expect("Failed to query contract");
    
    println!("Contract info: {:?}", result);
    assert!(!result.is_null(), "Contract query returned null");
}
