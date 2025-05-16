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
    println!("Connected to Base network with chain ID: {}", client.chain_id());
    println!("Network explorer URL: {}", client.explorer_url());
    
    // Get the latest block number
    let block_number = client.client().block_number().await?;
    println!("Latest block number: {}", block_number);
    
    // You can also create a client using a configuration file
    let config_path = Path::new("src/config/base_networks.json");
    if config_path.exists() {
        println!("\nCreating client from configuration file:");
        let file_client = BaseClient::from_config_file(
            BaseNetwork::Mainnet,
            None,
            Some(config_path),
        )?;
        
        println!("File-based client chain ID: {}", file_client.chain_id());
    } else {
        println!("\nConfiguration file not found at: {}", config_path.display());
        println!("Run 'nix run .#fetch-protos -- --base' to generate the config file");
    }
    
    Ok(())
} 