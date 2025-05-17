// Example demonstrating Noble blockchain integration with valence-domain-clients

use std::env;
use valence_domain_clients::{
    cosmos::chains::noble::NobleClient, CosmosBaseClient, GrpcSigningClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //-----------------------------------------------------------------------------
    // Noble Client Setup
    //-----------------------------------------------------------------------------
    println!("=== Noble Client Example with CCTP ===");

    // Get connection parameters from environment variables or use defaults
    let grpc_url = env::var("NOBLE_GRPC_URL")
        .unwrap_or_else(|_| "https://noble-grpc.polkachu.com:14990".to_string());

    let chain_id =
        env::var("NOBLE_CHAIN_ID").unwrap_or_else(|_| "noble-1".to_string());

    // In production, use a secure method to retrieve the mnemonic
    let mnemonic = env::var("NOBLE_MNEMONIC").expect("NOBLE_MNEMONIC must be set");

    // Create a Noble client with the specified parameters
    let noble_client = NobleClient::new(
        &grpc_url, &chain_id, &mnemonic, None, // Using default derivation path
    )
    .await
    .expect("Failed to create Noble client");

    //-----------------------------------------------------------------------------
    // Query account information
    //-----------------------------------------------------------------------------
    let signer = noble_client.get_signer_details().await?.address;
    println!("Connected to Noble as address: {}", signer);

    // Query USDC balance
    let balance = noble_client.query_balance(&signer.0, "uusdc").await?;
    println!(
        "USDC balance: {} uusdc ({} USDC)",
        balance,
        balance as f64 / 1_000_000.0
    );

    //-----------------------------------------------------------------------------
    // Basic Noble Operations
    //-----------------------------------------------------------------------------

    // Transfer tokens to another Noble address
    let to_address = "noble1qu8ak5zl9nxvfaqvj7fvzcg5xl96g6wlmspkq9"; // Example address
    println!("\nTransferring 0.1 USDC to {}...", to_address);

    let transfer_result = noble_client
        .transfer(
            to_address,
            100000, // 0.1 USDC (6 decimals)
            "uusdc",
            Some("Example transfer from Valence client"),
        )
        .await;

    match transfer_result {
        Ok(result) => println!("Transfer successful - TX hash: {}", result.tx_hash),
        Err(e) => println!("Transfer failed: {}", e),
    }

    //-----------------------------------------------------------------------------
    // CCTP Cross-Chain Transfer Example
    //-----------------------------------------------------------------------------
    println!("\n=== Cross-Chain Transfer Protocol (CCTP) Example ===");

    // Destination chain information
    let destination_domain = 1; // Ethereum Mainnet = 1, Arbitrum = 3, Avalanche = 6, etc.
    let destination_address = "0x71C7656EC7ab88b098defB751B7401B5f6d8976F"; // Example ETH recipient

    // Amount to transfer
    let transfer_amount = "500000"; // 0.5 USDC
    println!(
        "\nInitiating CCTP transfer of 0.5 USDC to {} on domain {}...",
        destination_address, destination_domain
    );

    // Execute the deposit for burn
    let cctp_result = noble_client
        .cctp_deposit_for_burn(
            destination_domain,
            destination_address,
            transfer_amount,
            "uusdc",
        )
        .await;

    match cctp_result {
        Ok(result) => {
            println!("CCTP deposit for burn successful!");
            println!("Transaction hash: {}", result.tx_hash);
            println!("Block height: {}", result.height);

            // The transaction will generate a 'burn message' which Circle will attest to
            // After attestation, the message and attestation can be submitted to the
            // destination chain to complete the transfer

            println!("\nThe transfer will complete after:");
            println!("1. Circle attests to the burn message (automatic)");
            println!(
                "2. The message and attestation are submitted to the destination chain"
            );
        }
        Err(e) => println!("CCTP transfer failed: {}", e),
    }

    //-----------------------------------------------------------------------------
    // CCTP Receiving Message Example
    //-----------------------------------------------------------------------------
    println!("\n=== CCTP Receive Message Example (requires valid attestation) ===");
    println!(
        "Note: This example requires the message and attestation from a cross-chain transfer."
    );
    println!(
        "      In a real application, you would obtain these from an attestation API or indexer."
    );

    // In a real application, you would get these from an indexer or API
    // For this example, we're showing the usage pattern only
    let _dummy_message = vec![0u8; 32]; // Placeholder for an actual message
    let _dummy_attestation = vec![0u8; 32]; // Placeholder for an actual attestation

    println!("\nTo complete a CCTP transfer to Noble, you would use:");
    println!("client.cctp_receive_message(message_bytes, attestation_bytes)");

    Ok(())
}
