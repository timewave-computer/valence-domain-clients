//! Example of using the Base client to interact with the Base blockchain
//!
//! This example demonstrates how to create a Base client and make basic queries.

use std::path::Path;
use valence_domain_clients::{BaseClient, BaseNetwork};
use valence_domain_clients::EvmBaseClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client for Base Sepolia testnet without a private key (read-only)
    let client = BaseClient::new(BaseNetwork::Sepolia, None)?;
    
    // Print information about the client
    println!("Connected to Base network with chain ID: {}", client.network_chain_id());
    println!("Network explorer URL: {}", client.explorer_url());
    
    // Get the latest block number
    let block_number = client.get_block_number().await?;
    println!("Latest block number: {}", block_number);
    
    // Get the chain ID from the network
    let chain_id = client.get_chain_id().await?;
    println!("Chain ID from RPC: {}", chain_id);
    
    // Get the current gas price
    let gas_price = client.get_gas_price().await?;
    println!("Current gas price: {:#?} wei", gas_price.0);
    
    // You can also create a client using a configuration file
    let config_path = Path::new("src/config/base_networks.json");
    if config_path.exists() {
        println!("\nCreating client from configuration file:");
        let file_client = BaseClient::from_config_file(
            BaseNetwork::Mainnet,
            None,
            Some(config_path),
        )?;
        
        println!("File-based client chain ID: {}", file_client.network_chain_id());
    } else {
        println!("\nConfiguration file not found at: {}", config_path.display());
        println!("Run 'nix run .#fetch-protos -- --base' to generate the config file");
    }
    
    Ok(())
} 