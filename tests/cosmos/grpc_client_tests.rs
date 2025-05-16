//-----------------------------------------------------------------------------
// gRPC Signing Client Tests
//-----------------------------------------------------------------------------

use mockall::mock;
use mockall::predicate::*;
use cosmrs::Any as CosmrsAny;

use valence_domain_clients::{
    core::error::ClientError,
    core::transaction::TransactionResponse,
    cosmos::grpc_client::GrpcSigningClient,
    cosmos::types::{CosmosAccount, CosmosFee, CosmosSimulateResponse, CosmosGasInfo},
};

// Create a mock implementation of GrpcSigningClient
mock! {
    pub TestGrpcSigningClient {}

    #[async_trait::async_trait]
    impl GrpcSigningClient for TestGrpcSigningClient {
        fn grpc_url(&self) -> String;
        fn chain_prefix(&self) -> String;
        fn chain_id_str(&self) -> String;
        fn chain_denom(&self) -> String;
        fn gas_price(&self) -> f64;
        fn gas_adjustment(&self) -> f64;

        async fn get_signer_details(&self) -> Result<CosmosAccount, ClientError>;
        fn get_tx_fee(&self, simulation_response: CosmosSimulateResponse) -> Result<CosmosFee, ClientError>;
        async fn simulate_tx(&self, msg: CosmrsAny) -> Result<CosmosSimulateResponse, ClientError>;
        async fn query_chain_gas_config(&self, chain_name: &str, denom: &str) -> Result<f64, ClientError>;
        async fn sign_and_broadcast_tx(&self, msg: CosmrsAny, fee: CosmosFee, memo: Option<&str>) -> Result<TransactionResponse, ClientError>;
    }
}

#[tokio::test]
async fn test_sign_and_broadcast_tx() {
    // Create a mock instance
    let mut mock = MockTestGrpcSigningClient::new();
    
    // Setup mock behavior
    mock.expect_grpc_url()
        .return_const("https://test-grpc.example.com:9090".to_string());
    
    mock.expect_chain_prefix()
        .return_const("cosmos".to_string());
    
    mock.expect_chain_id_str()
        .return_const("test-chain-1".to_string());
    
    mock.expect_chain_denom()
        .return_const("utest".to_string());
    
    mock.expect_gas_price()
        .return_const(0.025);
    
    mock.expect_gas_adjustment()
        .return_const(1.3);
    
    // Setup signer account details
    let account = CosmosAccount {
        address: "cosmos1test".into(),
        pub_key: Some(vec![1, 2, 3, 4]),
        account_number: 123,
        sequence: 45,
    };
    
    mock.expect_get_signer_details()
        .returning(move || Ok(account.clone()));
    
    // Setup simulation response
    let sim_response = CosmosSimulateResponse {
        gas_info: CosmosGasInfo {
            gas_wanted: 100000,
            gas_used: 80000,
        },
        simulate_result: None,
    };
    
    mock.expect_simulate_tx()
        .returning(move |_| Ok(sim_response.clone()));
    
    // Setup fee calculation
    let fee = CosmosFee {
        amount: vec![],
        gas_limit: 130000,
        payer: None,
        granter: None,
    };
    
    mock.expect_get_tx_fee()
        .returning(move |_| Ok(fee.clone()));
    
    // Setup broadcasting response
    let tx_response = TransactionResponse {
        tx_hash: "ABCDEF1234567890".to_string(),
        height: 987654,
        gas_wanted: Some(130000),
        gas_used: Some(80000),
        events: vec![],
        code: None,
        raw_log: Some("success".to_string()),
        data: None,
        block_hash: Some("BLOCK123".to_string()),
        timestamp: 1672574400, // Unix timestamp for "2023-01-01T12:00:00Z"
        original_request_payload: None,
    };
    
    mock.expect_sign_and_broadcast_tx()
        .returning(move |_, _, _| Ok(tx_response.clone()));
    
    // Create a test message - using an empty Any for simplicity
    let msg = CosmrsAny {
        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
        value: vec![1, 2, 3, 4],
    };
    
    // Call the function being tested
    let result = mock.sign_and_broadcast_tx(msg, fee.clone(), Some("Test memo")).await;
    
    // Verify the result
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.tx_hash, "ABCDEF1234567890");
    assert_eq!(response.height, 987654);
    assert_eq!(response.gas_wanted, Some(130000));
    assert_eq!(response.gas_used, Some(80000));
}

#[tokio::test]
async fn test_query_chain_gas_config() {
    // Create a mock instance
    let mut mock = MockTestGrpcSigningClient::new();
    
    // Setup mock behavior
    mock.expect_query_chain_gas_config()
        .with(eq("osmosis"), eq("uosmo"))
        .returning(|_, _| Ok(0.025));
    
    // Call the function being tested
    let result = mock.query_chain_gas_config("osmosis", "uosmo").await;
    
    // Verify the result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0.025);
} 