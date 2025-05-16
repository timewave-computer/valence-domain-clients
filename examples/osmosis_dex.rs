//-----------------------------------------------------------------------------
// Osmosis DEX Example
//-----------------------------------------------------------------------------
//! This example demonstrates how to interact with the Osmosis DEX 
//! using the valence-domain-clients library.
//!
//! It shows:
//! 1. How to initialize an Osmosis client
//! 2. How to query pool information
//! 3. How to swap tokens on the DEX
//! 4. How to add liquidity to a pool

use std::error::Error;

use valence_domain_clients::{
    core::{error::ClientError, transaction::TransactionResponse},
    cosmos::{
        chains::osmosis::OsmosisClient,
        types::CosmosCoin,
    },
    CosmosBaseClient,
    GrpcSigningClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // In a real application, these would come from environment variables
    let grpc_url = "https://osmosis-testnet-grpc.example.com:9090";
    let chain_id = "osmo-test-5";
    let mnemonic = "test test test test test test test test test test test junk";
    
    println!("Initializing Osmosis client...");
    
    // Initialize Osmosis client
    let osmosis = OsmosisClient::new(
        grpc_url,
        chain_id,
        mnemonic,
        None,
    ).await?;
    
    // Get our address
    let address = osmosis.get_signer_details().await?.address.0;
    println!("Signer address: {}", address);
    
    //-----------------------------------------------------------------------------
    // Balance queries
    //-----------------------------------------------------------------------------
    
    // Query OSMO balance
    let osmo_balance = osmosis.query_balance(&address, "uosmo").await?;
    println!("OSMO balance: {} uosmo", osmo_balance);
    
    // Query ATOM balance (if any)
    let atom_denom = "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2";
    let atom_balance = osmosis.query_balance(&address, atom_denom).await?;
    println!("ATOM balance: {} {}", atom_balance, atom_denom);
    
    //-----------------------------------------------------------------------------
    // Token swap example
    //-----------------------------------------------------------------------------
    
    // Check if we have enough balance to proceed
    if osmo_balance >= 1_000_000 { // 1 OSMO
        println!("Performing token swap...");
        
        // Execute a swap from OSMO to ATOM
        let swap_result = match swap_tokens(&osmosis, "uosmo", atom_denom, 1_000_000u128).await {
            Ok(tx) => {
                println!("Swap successful! Transaction hash: {}", tx.tx_hash);
                println!("Waiting for confirmation...");
                
                // In a real app, you would poll for confirmation here
                // For now, we'll just wait a few seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                // Query the new balances
                let new_osmo_balance = osmosis.query_balance(&address, "uosmo").await?;
                let new_atom_balance = osmosis.query_balance(&address, atom_denom).await?;
                
                println!("New OSMO balance: {} uosmo", new_osmo_balance);
                println!("New ATOM balance: {} {}", new_atom_balance, atom_denom);
                
                Ok(())
            },
            Err(e) => Err(e),
        };
        
        if let Err(e) = swap_result {
            println!("Swap failed: {}", e);
        }
    } else {
        println!("Not enough OSMO balance to perform swap example");
    }
    
    //-----------------------------------------------------------------------------
    // Add liquidity example
    //-----------------------------------------------------------------------------
    
    // In a real application, you would:
    // 1. Query available pools
    // 2. Find the appropriate pool ID
    // 3. Calculate appropriate liquidity amounts
    
    // For this example, we'll use a fictional pool ID 1
    let pool_id = 1;
    
    // Check if we have enough balance for liquidity provision
    if osmo_balance >= 5_000_000 && atom_balance >= 1_000_000 {
        println!("Adding liquidity to pool {}...", pool_id);
        
        // Create the tokens array for liquidity provision
        let tokens = vec![
            CosmosCoin { denom: "uosmo".to_string(), amount: 5_000_000u128 },
            CosmosCoin { denom: atom_denom.to_string(), amount: 1_000_000u128 },
        ];
        
        // Add liquidity to the pool
        match add_liquidity(&osmosis, pool_id, tokens).await {
            Ok(tx) => {
                println!("Liquidity addition successful! Transaction hash: {}", tx.tx_hash);
                println!("Waiting for confirmation...");
                
                // In a real app, you would poll for confirmation here
                // For now, we'll just wait a few seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                // Query the new balances
                let new_osmo_balance = osmosis.query_balance(&address, "uosmo").await?;
                println!("New OSMO balance: {} uosmo", new_osmo_balance);
            },
            Err(e) => println!("Liquidity addition failed: {}", e),
        }
    } else {
        println!("Not enough balance to perform liquidity provision example");
    }
    
    println!("Osmosis DEX examples completed!");
    Ok(())
}

// Execute a token swap on Osmosis
async fn swap_tokens(
    osmosis: &OsmosisClient,
    token_in_denom: &str,
    token_out_denom: &str,
    token_in_amount: u128,
) -> Result<TransactionResponse, ClientError> {
    // Calculate minimum output based on a 2% slippage tolerance
    // In a real app, you would query the pool for accurate pricing
    let min_output_amount = token_in_amount * 98 / 100;
    
    // Execute swap
    osmosis.swap_tokens(
        token_in_denom,
        token_out_denom,
        token_in_amount,
        min_output_amount,
    ).await
}

// Add liquidity to an Osmosis pool
async fn add_liquidity(
    osmosis: &OsmosisClient,
    pool_id: u64,
    tokens: Vec<CosmosCoin>,
) -> Result<TransactionResponse, ClientError> {
    // Execute liquidity addition
    osmosis.add_liquidity(pool_id, tokens).await
}
