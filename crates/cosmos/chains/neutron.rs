//-----------------------------------------------------------------------------
// Neutron Client Implementation
//-----------------------------------------------------------------------------

use tonic::async_trait;
use serde::Serialize;

use crate::core::error::ClientError;
use crate::core::transaction::TransactionResponse;
use crate::cosmos::base_client::CosmosBaseClient;
use crate::cosmos::generic_client::{CosmosClientConfig, GenericCosmosClient};
use crate::cosmos::grpc_client::GrpcSigningClient;
use crate::cosmos::types::{
    CosmosAccount,
    CosmosBlockResults,
    CosmosCoin,
    CosmosFee,
    CosmosHeader,
    CosmosModuleAccount,
    CosmosSimulateResponse,
};
use cosmrs::tx::Msg;
use cosmrs;

/// Client for interacting with the Neutron blockchain.
///
/// Neutron is an interoperable smart contract platform in the Cosmos ecosystem
/// focused on enhanced CosmWasm functionality and interchain security.
pub struct NeutronClient {
    base_client: Box<dyn CosmosBaseClient>,
}

impl NeutronClient {
    /// Create a new Neutron client with the specified parameters.
    ///
    /// # Arguments
    /// * `grpc_url` - The URL of the Neutron gRPC endpoint
    /// * `chain_id` - The chain ID of the Neutron network
    /// * `mnemonic` - The mnemonic for key derivation
    /// * `derivation_path` - Optional custom derivation path
    ///
    /// # Returns
    /// A new NeutronClient or an error if initialization fails
    pub async fn new(
        grpc_url: &str,
        chain_id: &str,
        mnemonic: &str,
        _derivation_path: Option<&str>,
    ) -> Result<Self, ClientError> {
        // Configure the Neutron client with appropriate parameters
        let config = CosmosClientConfig {
            grpc_url: grpc_url.to_string(),
            chain_id_str: chain_id.to_string(),
            chain_prefix: "neutron".to_string(),
            chain_denom: "untrn".to_string(),
            gas_price: 0.025,
            gas_adjustment: 1.5,
        };

        // Create the generic client and wrap it
        let base_client = GenericCosmosClient::new(
            config, 
            mnemonic
        ).await?;
        
        Ok(Self {
            base_client: Box::new(base_client),
        })
    }

    /// Execute a CosmWasm contract call on Neutron
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract to execute
    /// * `msg` - The execution message to send to the contract
    /// * `funds` - Optional funds to send with the execution
    ///
    /// # Returns
    /// Transaction result with execution information
    pub async fn execute_contract<T: Serialize + Send>(
        &self,
        contract_address: &str,
        msg: &T,
        funds: Vec<CosmosCoin>,
    ) -> Result<TransactionResponse, ClientError> {
        // Serialize the message
        let msg_bytes = serde_json::to_vec(msg)
            .map_err(|e| ClientError::ParseError(format!("Failed to serialize message: {e}")))?;
            
        // Prepare the CosmWasm execute message
        let execute_msg = cosmrs::cosmwasm::MsgExecuteContract {
            sender: self.get_signer_details().await?.address.0.parse::<cosmrs::AccountId>()?,
            contract: contract_address.parse()?,
            msg: msg_bytes,
            funds: funds.into_iter().map(|coin| cosmrs::Coin {
                denom: coin.denom.parse().unwrap(),
                amount: coin.amount,
            }).collect(),
        };
        
        // Convert to Any
        let any_msg = execute_msg.to_any()
            .map_err(|e| ClientError::ClientError(format!("Failed to convert message to Any: {e}")))?;
            
        // Simulate for fee estimation
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;
        
        // Execute transaction
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }

    /// Query a CosmWasm contract's state on Neutron
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract to query
    /// * `query_msg` - The query message to send to the contract
    ///
    /// # Returns
    /// The deserialized response from the contract query
    pub async fn query_contract<T: for<'de> serde::Deserialize<'de>, Q: Serialize + Send>(
        &self,
        contract_address: &str,
        query_msg: &Q,
    ) -> Result<T, ClientError> {
        // Serialize the query message to JSON
        let query_msg_string = serde_json::to_string(query_msg)
            .map_err(|e| ClientError::ParseError(format!("Failed to serialize query message: {e}")))?;
        
        // Create a tonic client for the CosmWasm query service
        let mut wasm_query_client = cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient::connect(
            format!("http://{}", self.grpc_url())
        ).await
            .map_err(|e| ClientError::ClientError(format!("Failed to connect to Neutron wasm query service: {e}")))?;
        
        // Prepare the query request
        let request = cosmos_sdk_proto::cosmwasm::wasm::v1::QuerySmartContractStateRequest {
            address: contract_address.to_string(),
            query_data: query_msg_string.into_bytes(),
        };
        
        // Execute the query
        let response = wasm_query_client.smart_contract_state(request).await
            .map_err(|e| ClientError::ClientError(format!("Failed to query contract state: {e}")))?;
        
        // Deserialize the response
        serde_json::from_slice(&response.get_ref().data)
            .map_err(|e| ClientError::ParseError(format!("Failed to deserialize contract response: {e}")))
    }
}

#[async_trait]
impl GrpcSigningClient for NeutronClient {
    fn grpc_url(&self) -> String {
        self.base_client.grpc_url()
    }

    fn chain_prefix(&self) -> String {
        self.base_client.chain_prefix()
    }

    fn chain_id_str(&self) -> String {
        self.base_client.chain_id_str()
    }

    fn chain_denom(&self) -> String {
        self.base_client.chain_denom()
    }

    fn gas_price(&self) -> f64 {
        self.base_client.gas_price()
    }

    fn gas_adjustment(&self) -> f64 {
        self.base_client.gas_adjustment()
    }

    async fn get_signer_details(&self) -> Result<CosmosAccount, ClientError> {
        self.base_client.get_signer_details().await
    }
    
    fn get_tx_fee(&self, simulation_response: CosmosSimulateResponse) -> Result<CosmosFee, ClientError> {
        self.base_client.get_tx_fee(simulation_response)
    }
    
    async fn simulate_tx(&self, msg: cosmrs::Any) -> Result<CosmosSimulateResponse, ClientError> {
        self.base_client.simulate_tx(msg).await
    }
    
    async fn sign_and_broadcast_tx(
        &self, 
        msg: cosmrs::Any, 
        fee: CosmosFee, 
        memo: Option<&str>
    ) -> Result<TransactionResponse, ClientError> {
        self.base_client.sign_and_broadcast_tx(msg, fee, memo).await
    }
    
    async fn query_chain_gas_config(
        &self,
        _chain_name: &str, 
        _chain_denom: &str
    ) -> Result<f64, ClientError> {
        // Default implementation for Neutron
        Ok(0.025)
    }
}

#[async_trait]
impl CosmosBaseClient for NeutronClient {
    async fn transfer(
        &self,
        to_address: &str,
        amount: u128,
        denom: &str,
        memo: Option<&str>,
    ) -> Result<TransactionResponse, ClientError> {
        self.base_client.transfer(to_address, amount, denom, memo).await
    }

    async fn latest_block_header(&self) -> Result<CosmosHeader, ClientError> {
        self.base_client.latest_block_header().await
    }

    async fn block_results(&self, height: u64) -> Result<CosmosBlockResults, ClientError> {
        self.base_client.block_results(height).await
    }

    async fn query_balance(&self, address: &str, denom: &str) -> Result<u128, ClientError> {
        self.base_client.query_balance(address, denom).await
    }

    async fn query_module_account(&self, name: &str) -> Result<CosmosModuleAccount, ClientError> {
        self.base_client.query_module_account(name).await
    }

    async fn poll_for_tx(&self, tx_hash: &str) -> Result<TransactionResponse, ClientError> {
        self.base_client.poll_for_tx(tx_hash).await
    }

    async fn poll_until_expected_balance(
        &self,
        address: &str,
        denom: &str,
        min_amount: u128,
        interval_sec: u64,
        max_attempts: u32,
    ) -> Result<u128, ClientError> {
        self.base_client.poll_until_expected_balance(
            address,
            denom,
            min_amount,
            interval_sec,
            max_attempts,
        ).await
    }
    
    async fn ibc_transfer(
        &self,
        to_address: String,
        denom: String,
        amount: String,
        source_channel: String,
        timeout_seconds: u64,
        memo: Option<String>,
    ) -> Result<TransactionResponse, ClientError> {
        // Neutron might have specific IBC implementation requirements
        // For now, delegate to the base implementation
        self.base_client.ibc_transfer(
            to_address,
            denom,
            amount,
            source_channel,
            timeout_seconds,
            memo,
        ).await
    }
    

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // This test requires a connection to a Neutron node and valid mnemonic
    async fn test_neutron_connection() {
        let grpc_url = "http://localhost:9090";
        let chain_id = "neutron-1";
        let mnemonic = "test test test test test test test test test test test junk";
        
        let client = NeutronClient::new(grpc_url, chain_id, mnemonic, None).await;
        assert!(client.is_ok(), "Failed to create Neutron client");
    }
}
