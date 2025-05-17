// Example demonstrating cross-chain transfers between Ethereum and Noble
//-----------------------------------------------------------------------------
// Cross-Chain Transfer Example
//-----------------------------------------------------------------------------
//! This example demonstrates how to transfer assets between Ethereum and Noble
//! chains using the new modular client organization.
//!
//! It shows:
//! 1. How to initialize both EVM and Cosmos clients in the same application
//! 2. How to query balances on both chains
//! 3. How to execute a transfer on Ethereum
//! 4. How to execute a parallel transfer on Noble
//! 5. How to wait for transaction confirmation on both chains

use std::error::Error;
use std::time::Duration;

use tokio::time::sleep;

use valence_domain_clients::{
    // Core types shared across all chains
    core::{error::ClientError, transaction::TransactionResponse},

    // Cosmos-specific imports
    cosmos::chains::noble::NobleClient,
    // EVM-specific imports
    evm::{
        chains::ethereum::EthereumClient,
        types::{EvmAddress, EvmTransactionRequest, EvmU256},
    },
    CosmosBaseClient,
    EvmBaseClient,

    GrpcSigningClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Environment variables would normally be used for these values
    // We're hardcoding them here for the example
    let eth_rpc_url = "https://eth-goerli.example.com";
    let eth_mnemonic = "test test test test test test test test test test test junk";

    let noble_grpc_url = "https://noble-testnet-grpc.example.com:9090";
    let noble_chain_id = "noble-testnet-4";
    let noble_mnemonic =
        "test test test test test test test test test test test junk";

    // Ethereum amount parameters - using new type constructor format
    let eth_amount = EvmU256([0, 0, 0, 10000000000000000]); // 0.01 ETH
    let eth_recipient = EvmAddress([
        0x74, 0x2d, 0x35, 0xCc, 0x66, 0x34, 0xC0, 0x53, 0x29, 0x25, 0xa3, 0xb8,
        0x44, 0xBc, 0x45, 0x4e, 0x44, 0x38, 0xf4, 0x4e,
    ]);

    // Noble amount parameters
    let noble_amount = 1_000_000u128; // 1 USDC (6 decimals)
    let noble_recipient = "noble1vf5amhvs83h7mzf3ucyd23fcszl2aw8yrrt9u2";

    println!("Initializing clients...");

    //-----------------------------------------------------------------------------
    // Client initialization
    //-----------------------------------------------------------------------------

    // Initialize Ethereum client
    let ethereum = EthereumClient::new(eth_rpc_url, eth_mnemonic, None)?;

    // Initialize Noble client
    let noble =
        NobleClient::new(noble_grpc_url, noble_chain_id, noble_mnemonic, None)
            .await?;

    //-----------------------------------------------------------------------------
    // Balance queries
    //-----------------------------------------------------------------------------

    // Query ETH balance
    let eth_sender = ethereum.evm_signer_address();
    let eth_balance = ethereum.get_balance(&eth_sender).await?;
    println!("ETH balance: {:?} wei", eth_balance.0);

    // Query Noble USDC balance
    let noble_sender = noble.get_signer_details().await?.address.0;
    let noble_balance = noble.query_balance(&noble_sender, "uusdc").await?;
    println!("Noble USDC balance: {} uusdc", noble_balance);

    //-----------------------------------------------------------------------------
    // Execute parallel transfers
    //-----------------------------------------------------------------------------

    println!("Executing transfers...");

    // This demonstrates parallel execution using tokio
    let (eth_result, noble_result) = tokio::join!(
        execute_eth_transfer(&ethereum, eth_recipient, eth_amount),
        execute_noble_transfer(&noble, noble_recipient, noble_amount)
    );

    let eth_tx = eth_result?;
    let noble_tx = noble_result?;

    println!("ETH transaction hash: {:?}", eth_tx.tx_hash);
    println!("Noble transaction hash: {}", noble_tx.tx_hash);

    // In a real application, we might want to poll for confirmation of both transactions
    println!("Waiting for transaction confirmations...");
    sleep(Duration::from_secs(10)).await;

    // After confirmation, we could query the balances again to verify the transfers

    println!("Transfers completed successfully!");
    Ok(())
}

// Execute an Ethereum transfer
async fn execute_eth_transfer(
    ethereum: &EthereumClient,
    recipient: EvmAddress,
    amount: EvmU256,
) -> Result<TransactionResponse, ClientError> {
    // Create transaction request
    let tx = EvmTransactionRequest {
        from: ethereum.evm_signer_address(),
        to: Some(recipient),
        value: Some(amount),
        gas_limit: Some(EvmU256([0, 0, 0, 21000])), // Standard ETH transfer gas
        gas_price: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        data: None,
        nonce: None,
        chain_id: None,
    };

    // Execute transaction
    ethereum.send_transaction(&tx).await
}

// Execute a Noble USDC transfer
async fn execute_noble_transfer(
    noble: &NobleClient,
    recipient: &str,
    amount: u128,
) -> Result<TransactionResponse, ClientError> {
    // Execute transfer
    noble
        .transfer(
            recipient,
            amount,
            "uusdc",
            Some("Cross-chain example transfer"),
        )
        .await
}
