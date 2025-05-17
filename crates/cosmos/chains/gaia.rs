//-----------------------------------------------------------------------------
// Gaia Client Implementation (Cosmos Hub)
//-----------------------------------------------------------------------------

use tonic::async_trait;
use crate::core::error::ClientError;
use crate::core::transaction::TransactionResponse;
use crate::cosmos::base_client::CosmosBaseClient;
use crate::cosmos::generic_client::{CosmosClientConfig, GenericCosmosClient};
use crate::cosmos::grpc_client::GrpcSigningClient;
use crate::cosmos::types::{
    CosmosAccount,
    CosmosBlockResults,
    CosmosFee,
    CosmosHeader,
    CosmosModuleAccount,
    CosmosSimulateResponse,
};
use cosmrs;

/// Client for interacting with the Cosmos Hub (Gaia) blockchain.
///
/// Cosmos Hub is the first blockchain in the Cosmos ecosystem and serves
/// as a hub for cross-chain communication via IBC.
pub struct GaiaClient {
    base_client: Box<dyn CosmosBaseClient>,
}

impl GaiaClient {
    /// Create a new Cosmos Hub client with the specified parameters.
    ///
    /// # Arguments
    /// * `grpc_url` - The URL of the Cosmos Hub gRPC endpoint
    /// * `chain_id` - The chain ID of the Cosmos Hub network (e.g. "cosmoshub-4")
    /// * `mnemonic` - The mnemonic for key derivation
    /// * `_derivation_path` - Optional custom derivation path
    ///
    /// # Returns
    /// A new GaiaClient or an error if initialization fails
    pub async fn new(
        grpc_url: &str,
        chain_id: &str,
        mnemonic: &str,
        _derivation_path: Option<&str>,
    ) -> Result<Self, ClientError> {
        // Configure the Cosmos Hub client with appropriate parameters
        let config = CosmosClientConfig {
            grpc_url: grpc_url.to_string(),
            chain_id_str: chain_id.to_string(),
            chain_prefix: "cosmos".to_string(),
            chain_denom: "uatom".to_string(),
            gas_price: 0.025,
            gas_adjustment: 1.3,
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

    /// Delegate ATOM tokens to a validator
    ///
    /// # Arguments
    /// * `validator_address` - The validator's operator address
    /// * `amount` - Amount of ATOM to delegate in uatom (micro ATOM)
    ///
    /// # Returns
    /// Transaction response with delegation information
    pub async fn delegate(
        &self,
        validator_address: &str,
        amount: u128,
    ) -> Result<TransactionResponse, ClientError> {
        let delegator: cosmrs::AccountId = self.get_signer_details().await?.address.0.parse()
            .map_err(|e| ClientError::ParseError(format!("Failed to parse delegator address: {e}")))?;
        
        let _validator = validator_address.parse::<String>()
            .map_err(|e| ClientError::ParseError(format!("Failed to parse validator address: {e}")))?;
            
        // Create delegation message
        // Create a staking message using our own proto definitions
        let delegation_msg = prost_types::Any {
            type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
            value: serde_json::to_vec(&serde_json::json!({
                "delegatorAddress": delegator.to_string(),
                "validatorAddress": validator_address.to_string(),
                "amount": {
                    "denom": self.chain_denom(),
                    "amount": amount.to_string()
                }
            })).map_err(|e| ClientError::SerializationError(e.to_string()))?,
        };
        
        // Use the Any message directly
        let any_msg = delegation_msg;
            
        // Simulate for fee estimation
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;
        
        // Execute transaction
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }

    /// Undelegate ATOM tokens from a validator
    ///
    /// # Arguments
    /// * `validator_address` - The validator's operator address
    /// * `amount` - Amount of ATOM to undelegate in uatom (micro ATOM)
    ///
    /// # Returns
    /// Transaction response with undelegation information
    pub async fn undelegate(
        &self,
        validator_address: &str,
        amount: u128,
    ) -> Result<TransactionResponse, ClientError> {
        let delegator: cosmrs::AccountId = self.get_signer_details().await?.address.0.parse()
            .map_err(|e| ClientError::ParseError(format!("Failed to parse delegator address: {e}")))?;
        
        // Create undelegation message using direct Any message
        let any_msg = prost_types::Any {
            type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
            value: serde_json::to_vec(&serde_json::json!({
                "delegatorAddress": delegator.to_string(),
                "validatorAddress": validator_address.to_string(),
                "amount": {
                    "denom": self.chain_denom(),
                    "amount": amount.to_string()
                }
            })).map_err(|e| ClientError::SerializationError(e.to_string()))?,
        };
            
        // Simulate for fee estimation
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;
        
        // Execute transaction
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }

    /// Claim rewards from a validator
    ///
    /// # Arguments
    /// * `validator_address` - The validator's operator address
    ///
    /// # Returns
    /// Transaction response with reward claim information
    pub async fn withdraw_rewards(
        &self,
        validator_address: &str,
    ) -> Result<TransactionResponse, ClientError> {
        let delegator: cosmrs::AccountId = self.get_signer_details().await?.address.0.parse()
            .map_err(|e| ClientError::ParseError(format!("Failed to parse delegator address: {e}")))?;
        
        // Create withdraw rewards message using direct Any message
        let any_msg = prost_types::Any {
            type_url: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
            value: serde_json::to_vec(&serde_json::json!({
                "delegatorAddress": delegator.to_string(),
                "validatorAddress": validator_address.to_string()
            })).map_err(|e| ClientError::SerializationError(e.to_string()))?,
        };
            
        // Simulate for fee estimation
        let sim_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(sim_response)?;
        
        // Execute transaction
        self.sign_and_broadcast_tx(any_msg, fee, None).await
    }
}

#[async_trait]
impl GrpcSigningClient for GaiaClient {
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
        // Default implementation for Cosmos Hub
        Ok(0.025)
    }
}

#[async_trait]
impl CosmosBaseClient for GaiaClient {
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
        // Cosmos Hub (Gaia) is the original chain with IBC support
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
    #[ignore] // This test requires a connection to a Cosmos Hub node and valid mnemonic
    async fn test_gaia_connection() {
        let grpc_url = "http://localhost:9090";
        let chain_id = "cosmoshub-4";
        let mnemonic = "test test test test test test test test test test test junk";
        
        let client = GaiaClient::new(grpc_url, chain_id, mnemonic, None).await;
        assert!(client.is_ok(), "Failed to create Gaia client");
    }
}
