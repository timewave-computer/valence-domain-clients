//-----------------------------------------------------------------------------
// Erigon Tracing API Example
//-----------------------------------------------------------------------------

//! This example demonstrates how to use the Erigon tracing API to analyze
//! transaction execution, track state changes, and debug smart contract calls.
//!
//! The example shows how to:
//! 1. Set up an Ethereum client with Erigon tracing capabilities
//! 2. Trace specific transactions to see detailed execution steps
//! 3. Trace entire blocks to analyze all transactions
//! 4. Use trace filters to find specific patterns
//! 5. Trace calls without executing them on-chain
//! 6. Replay transactions with different trace options

use std::env;
use std::error::Error;
use std::str::FromStr;

#[cfg(feature = "erigon-tracing")]
use valence_domain_clients::{
    // Core functionality
    core::error::ClientError,
    
    // EVM types and tracing
    evm::{
        chains::ethereum::EthereumClient,
        tracing::{helpers, ErigonTracing},
        types::{EvmAddress, EvmBytes, EvmHash, EvmU256},
        CallTraceRequest, TraceAction,
    },
    
    // Base client functionality
    EvmBaseClient,
};

#[cfg(not(feature = "erigon-tracing"))]
fn main() {
    println!("This example requires the 'erigon-tracing' feature to be enabled.");
    println!("Run with: cargo run --example erigon_tracing_example --features erigon-tracing");
}

#[cfg(feature = "erigon-tracing")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Get RPC URL from environment or use default
    let rpc_url = env::var("ETHEREUM_RPC_URL")
        .unwrap_or_else(|_| "https://ethereum-rpc.publicnode.com".to_string());

    println!("🔍 Erigon Tracing API Example");
    println!("Connected to: {rpc_url}");
    println!();

    // Create Ethereum client
    let client = EthereumClient::new(&rpc_url, "", None)?;

    // Get the latest block number
    let latest_block = client.get_block_number().await?;
    println!("📊 Latest block: {latest_block}");
    println!();

    // Example 1: Trace a specific transaction
    println!("🔍 Example 1: Tracing a specific transaction");
    let example_tx_hash = EvmHash::from_str(
        "0xa3ece39ae137617669c6933b7578b94e705e765683f260fcfe30eaa41932539f"
    )?;
    
    match trace_transaction_example(&client, &example_tx_hash).await {
        Ok(_) => println!("✅ Transaction tracing completed"),
        Err(e) => println!("⚠️  Transaction tracing failed: {e}"),
    }
    println!();

    // Example 2: Trace recent blocks
    println!("🔍 Example 2: Tracing recent blocks");
    match trace_recent_blocks(&client, latest_block).await {
        Ok(_) => println!("✅ Block tracing completed"),
        Err(e) => println!("⚠️  Block tracing failed: {e}"),
    }
    println!();

    // Example 3: Filter traces for specific addresses
    println!("🔍 Example 3: Filtering traces by address");
    match filter_traces_example(&client, latest_block).await {
        Ok(_) => println!("✅ Trace filtering completed"),
        Err(e) => println!("⚠️  Trace filtering failed: {e}"),
    }
    println!();

    // Example 4: Trace a hypothetical call
    println!("🔍 Example 4: Tracing a hypothetical call");
    match trace_call_example(&client).await {
        Ok(_) => println!("✅ Call tracing completed"),
        Err(e) => println!("⚠️  Call tracing failed: {e}"),
    }
    println!();

    // Example 5: Replay transactions with tracing
    println!("🔍 Example 5: Replaying transactions with tracing");
    match replay_transaction_example(&client, &example_tx_hash).await {
        Ok(_) => println!("✅ Transaction replay completed"),
        Err(e) => println!("⚠️  Transaction replay failed: {e}"),
    }

    println!();
    println!("🎉 Erigon tracing example completed!");

    Ok(())
}

#[cfg(feature = "erigon-tracing")]
async fn trace_transaction_example(
    client: &EthereumClient,
    tx_hash: &EvmHash,
) -> Result<(), ClientError> {
    println!("  🔍 Tracing transaction: {tx_hash}");

    // Trace with all available information
    let trace_types = helpers::all_trace_types();
    
    match client.trace_transaction(tx_hash, trace_types).await {
        Ok(trace) => {
            println!("  📋 Transaction trace obtained:");
            println!("     - Block number: {}", trace.block_number);
            println!("     - Subtraces: {}", trace.subtraces);
            
            if let Some(result) = &trace.result {
                println!("     - Result: {result:?}");
            }
            
            if let Some(error) = &trace.error {
                println!("     - Error: {error}");
            }
        }
        Err(e) => {
            println!("  ❌ Failed to trace transaction: {e}");
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(feature = "erigon-tracing")]
async fn trace_recent_blocks(
    client: &EthereumClient,
    latest_block: u64,
) -> Result<(), ClientError> {
    // Trace the last 2 blocks (or fewer if near genesis)
    let blocks_to_trace = if latest_block >= 2 { 2 } else { latest_block };
    let start_block = latest_block - blocks_to_trace + 1;

    for block_num in start_block..=latest_block {
        println!("  🔍 Tracing block: {block_num}");
        
        // Use basic trace types to avoid overwhelming output
        let trace_types = helpers::basic_trace_types();
        
        match client.trace_block(block_num, trace_types).await {
            Ok(block_trace) => {
                println!("     - Found {} traces in block", block_trace.traces.len());
                
                // Show summary of trace types
                let mut call_count = 0;
                let mut create_count = 0;
                
                for trace in &block_trace.traces {
                    match &trace.action {
                        TraceAction::Call { .. } => call_count += 1,
                        TraceAction::Create { .. } => create_count += 1,
                        _ => {}
                    }
                }
                
                println!("     - Calls: {call_count}, Creates: {create_count}");
            }
            Err(e) => {
                println!("     ❌ Failed to trace block {block_num}: {e}");
            }
        }
    }

    Ok(())
}

#[cfg(feature = "erigon-tracing")]
async fn filter_traces_example(
    client: &EthereumClient,
    latest_block: u64,
) -> Result<(), ClientError> {
    // Create a filter for recent blocks
    let filter = helpers::create_recent_filter(10, latest_block);
    
    println!("  🔍 Filtering traces from block {} to {}", 
             filter.from_block.unwrap_or(0), 
             filter.to_block.unwrap_or(latest_block));

    match client.trace_filter(&filter).await {
        Ok(traces) => {
            println!("     - Found {} matching traces", traces.len());
            
            // Show some statistics
            let mut unique_addresses = std::collections::HashSet::new();
            for trace in traces.iter().take(10) { // Limit output
                match &trace.action {
                    TraceAction::Call { from, to, .. } => {
                        unique_addresses.insert(from);
                        unique_addresses.insert(to);
                    }
                    TraceAction::Create { from, .. } => {
                        unique_addresses.insert(from);
                    }
                    _ => {}
                }
            }
            
            println!("     - Unique addresses involved: {}", unique_addresses.len());
        }
        Err(e) => {
            println!("     ❌ Failed to filter traces: {e}");
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(feature = "erigon-tracing")]
async fn trace_call_example(client: &EthereumClient) -> Result<(), ClientError> {
    // Example: Trace a call to get ETH balance (should be a simple call)
    let call_request = CallTraceRequest {
        from: Some(EvmAddress::zero()),
        to: EvmAddress::from_str("0xA0b86a33E6417c6ba5b5c5F6B8f7b3a7c5C8b5E3")
            .map_err(|e| ClientError::ParseError(e))?,
        gas: Some(EvmU256::from_u64(100_000)),
        gas_price: Some(EvmU256::from_u64(20_000_000_000)), // 20 gwei
        value: Some(EvmU256::zero()),
        data: Some(EvmBytes(vec![])), // Empty data for balance check
    };

    println!("  🔍 Tracing hypothetical call to: {}", call_request.to);

    let trace_types = helpers::basic_trace_types();

    match client.trace_call(&call_request, trace_types, None).await {
        Ok(trace) => {
            println!("     - Call trace successful");
            println!("     - Subtraces: {}", trace.subtraces);
            
            if let Some(result) = &trace.result {
                println!("     - Result: {result:?}");
            }
        }
        Err(e) => {
            println!("     ❌ Failed to trace call: {e}");
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(feature = "erigon-tracing")]
async fn replay_transaction_example(
    client: &EthereumClient,
    tx_hash: &EvmHash,
) -> Result<(), ClientError> {
    println!("  🔍 Replaying transaction: {tx_hash}");

    // Replay with VM trace to get detailed execution info
    let trace_types = helpers::vm_trace_types();

    match client.trace_replay_transaction(tx_hash, trace_types).await {
        Ok(trace) => {
            println!("     - Transaction replay successful");
            println!("     - Block number: {}", trace.block_number);
            println!("     - Subtraces: {}", trace.subtraces);
            
            // Show action details
            match &trace.action {
                TraceAction::Call { from, to, value, gas, .. } => {
                    println!("     - Call from {} to {}", from, to);
                    println!("     - Value: {value:?}, Gas: {gas:?}");
                }
                TraceAction::Create { from, value, gas, .. } => {
                    println!("     - Contract creation from {}", from);
                    println!("     - Value: {value:?}, Gas: {gas:?}");
                }
                _ => {
                    println!("     - Other action type: {:#?}", trace.action);
                }
            }
        }
        Err(e) => {
            println!("     ❌ Failed to replay transaction: {e}");
            return Err(e);
        }
    }

    Ok(())
} 