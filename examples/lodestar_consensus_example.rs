//-----------------------------------------------------------------------------
// Lodestar Consensus Client API Example
//-----------------------------------------------------------------------------

//! This example demonstrates how to use the Lodestar consensus client API to interact
//! with the Ethereum beacon chain and consensus layer operations.
//!
//! The example shows how to:
//! 1. Set up an Ethereum client with Lodestar consensus capabilities
//! 2. Query beacon chain state and finality information
//! 3. Get validator information and duties
//! 4. Access beacon blocks and attestations
//! 5. Monitor node status and sync progress
//! 6. Query committees and epoch information

use std::env;
use std::error::Error;

#[cfg(feature = "lodestar-consensus")]
use valence_domain_clients::{
    // Core functionality
    core::error::ClientError,
    
    // EVM types and consensus
    evm::{
        chains::ethereum::EthereumClient,
        consensus::{helpers, LodestarConsensus},
        types::{Epoch, Slot, ValidatorStatus},
    },
};

#[cfg(not(feature = "lodestar-consensus"))]
fn main() {
    println!("This example requires the 'lodestar-consensus' feature to be enabled.");
    println!("Run with: cargo run --example lodestar_consensus_example --features lodestar-consensus");
}

#[cfg(feature = "lodestar-consensus")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();

    // Get consensus client URL from environment or use default
    let consensus_url = env::var("LODESTAR_CONSENSUS_URL")
        .unwrap_or_else(|_| "http://localhost:9596".to_string());

    println!("🔍 Lodestar Consensus Client API Example");
    println!("Connected to: {consensus_url}");
    println!();

    // Create Ethereum client configured for consensus layer
    let client = EthereumClient::new(&consensus_url, "", None)?;

    // Example 1: Get genesis information and node status
    println!("📊 Example 1: Genesis and Node Information");
    match get_genesis_and_node_info(&client).await {
        Ok(_) => println!("✅ Genesis and node info retrieved"),
        Err(e) => println!("⚠️  Genesis and node info failed: {e}"),
    }
    println!();

    // Example 2: Get current beacon chain state
    println!("🔗 Example 2: Beacon Chain State");
    match get_beacon_chain_state(&client).await {
        Ok(_) => println!("✅ Beacon chain state retrieved"),
        Err(e) => println!("⚠️  Beacon chain state failed: {e}"),
    }
    println!();

    // Example 3: Get validator information
    println!("👥 Example 3: Validator Information");
    match get_validator_information(&client).await {
        Ok(_) => println!("✅ Validator information retrieved"),
        Err(e) => println!("⚠️  Validator information failed: {e}"),
    }
    println!();

    // Example 4: Get validator duties for current epoch
    println!("📋 Example 4: Validator Duties");
    match get_validator_duties(&client).await {
        Ok(_) => println!("✅ Validator duties retrieved"),
        Err(e) => println!("⚠️  Validator duties failed: {e}"),
    }
    println!();

    // Example 5: Get recent beacon blocks
    println!("🧱 Example 5: Recent Beacon Blocks");
    match get_recent_beacon_blocks(&client).await {
        Ok(_) => println!("✅ Recent beacon blocks retrieved"),
        Err(e) => println!("⚠️  Recent beacon blocks failed: {e}"),
    }
    println!();

    // Example 6: Get committee information
    println!("🏛️ Example 6: Committee Information");
    match get_committee_information(&client).await {
        Ok(_) => println!("✅ Committee information retrieved"),
        Err(e) => println!("⚠️  Committee information failed: {e}"),
    }

    println!();
    println!("🎉 Lodestar consensus example completed!");

    Ok(())
}

#[cfg(feature = "lodestar-consensus")]
async fn get_genesis_and_node_info(client: &EthereumClient) -> Result<(), ClientError> {
    println!("  📊 Getting genesis information...");
    
    // Get genesis data
    match client.get_genesis().await {
        Ok(genesis) => {
            println!("     - Genesis time: {}", genesis.genesis_time);
            println!("     - Genesis validators root: {}", genesis.genesis_validators_root);
            println!("     - Genesis fork version: {:?}", genesis.genesis_fork_version);
        }
        Err(e) => {
            println!("     ❌ Failed to get genesis: {e}");
            return Err(e);
        }
    }

    // Get node version
    match client.get_node_version().await {
        Ok(version) => {
            println!("     - Node version: {}", version.version);
        }
        Err(e) => {
            println!("     ⚠️  Failed to get node version: {e}");
        }
    }

    // Get sync status
    match client.get_sync_status().await {
        Ok(sync_status) => {
            println!("     - Head slot: {}", sync_status.head_slot);
            println!("     - Sync distance: {}", sync_status.sync_distance);
            println!("     - Is syncing: {}", sync_status.is_syncing);
            println!("     - Is optimistic: {}", sync_status.is_optimistic);
        }
        Err(e) => {
            println!("     ⚠️  Failed to get sync status: {e}");
        }
    }

    // Get node identity
    match client.get_node_identity().await {
        Ok(identity) => {
            println!("     - Peer ID: {}", identity.peer_id);
            println!("     - ENR: {}", identity.enr);
            println!("     - P2P addresses: {}", identity.p2p_addresses.len());
        }
        Err(e) => {
            println!("     ⚠️  Failed to get node identity: {e}");
        }
    }

    Ok(())
}

#[cfg(feature = "lodestar-consensus")]
async fn get_beacon_chain_state(client: &EthereumClient) -> Result<(), ClientError> {
    println!("  🔗 Getting beacon chain state...");
    
    let head_state = helpers::head_state_id();
    let finalized_state = helpers::finalized_state_id();

    // Get head state root
    match client.get_state_root(&head_state).await {
        Ok(head_root) => {
            println!("     - Head state root: {}", head_root);
        }
        Err(e) => {
            println!("     ❌ Failed to get head state root: {e}");
            return Err(e);
        }
    }

    // Get finalized state root
    match client.get_state_root(&finalized_state).await {
        Ok(finalized_root) => {
            println!("     - Finalized state root: {}", finalized_root);
        }
        Err(e) => {
            println!("     ⚠️  Failed to get finalized state root: {e}");
        }
    }

    // Get fork information
    match client.get_state_fork(&head_state).await {
        Ok(fork) => {
            println!("     - Current fork version: {:?}", fork.current_version);
            println!("     - Previous fork version: {:?}", fork.previous_version);
            println!("     - Fork epoch: {}", fork.epoch);
        }
        Err(e) => {
            println!("     ⚠️  Failed to get fork info: {e}");
        }
    }

    // Get finality checkpoints
    match client.get_state_finality_checkpoints(&head_state).await {
        Ok(checkpoints) => {
            println!("     - Finalized checkpoint epoch: {}", checkpoints.finalized.epoch);
            println!("     - Current justified epoch: {}", checkpoints.current_justified.epoch);
            println!("     - Previous justified epoch: {}", checkpoints.previous_justified.epoch);
        }
        Err(e) => {
            println!("     ⚠️  Failed to get finality checkpoints: {e}");
        }
    }

    Ok(())
}

#[cfg(feature = "lodestar-consensus")]
async fn get_validator_information(client: &EthereumClient) -> Result<(), ClientError> {
    println!("  👥 Getting validator information...");
    
    let head_state = helpers::head_state_id();

    // Get a few validators (indices 0-9)
    let validator_ids: Vec<String> = (0..10).map(|i| i.to_string()).collect();
    
    match client.get_validators(&head_state, &validator_ids, None).await {
        Ok(validators) => {
            println!("     - Retrieved {} validators", validators.len());
            
            for (i, validator) in validators.iter().take(3).enumerate() {
                println!("     - Validator {}: pubkey = {}", i, validator.pubkey);
                println!("       - Effective balance: {} Gwei", validator.effective_balance);
                println!("       - Slashed: {}", validator.slashed);
                println!("       - Activation epoch: {}", validator.activation_epoch);
                println!("       - Exit epoch: {}", validator.exit_epoch);
            }
            
            // Show active vs inactive validators
            let active_count = validators.iter()
                .filter(|v| helpers::is_validator_active(v, 0)) // Use epoch 0 for simplicity
                .count();
            println!("     - Active validators: {}/{}", active_count, validators.len());
        }
        Err(e) => {
            println!("     ❌ Failed to get validators: {e}");
            return Err(e);
        }
    }

    // Get validator balances
    match client.get_validator_balances(&head_state, &validator_ids).await {
        Ok(balances) => {
            println!("     - Retrieved {} validator balances", balances.len());
            
            let total_balance: u64 = balances.iter().map(|b| b.balance).sum();
            let average_balance = if !balances.is_empty() {
                total_balance / balances.len() as u64
            } else {
                0
            };
            
            println!("     - Total balance: {} Gwei ({:.2} ETH)", 
                     total_balance, 
                     helpers::effective_balance_to_eth(total_balance));
            println!("     - Average balance: {} Gwei ({:.2} ETH)", 
                     average_balance, 
                     helpers::effective_balance_to_eth(average_balance));
        }
        Err(e) => {
            println!("     ⚠️  Failed to get validator balances: {e}");
        }
    }

    // Filter by validator status
    let active_status = vec![ValidatorStatus::ActiveOngoing];
    match client.get_validators(&head_state, &[], Some(&active_status)).await {
        Ok(active_validators) => {
            println!("     - Total active validators: {}", active_validators.len());
        }
        Err(e) => {
            println!("     ⚠️  Failed to get active validators: {e}");
        }
    }

    Ok(())
}

#[cfg(feature = "lodestar-consensus")]
async fn get_validator_duties(client: &EthereumClient) -> Result<(), ClientError> {
    println!("  📋 Getting validator duties...");
    
    // Get current epoch (simplified - using epoch 0)
    let current_epoch: Epoch = 0;
    let validator_indices: Vec<u64> = (0..10).collect();

    // Get attester duties
    match client.get_attester_duties(current_epoch, &validator_indices).await {
        Ok(duties) => {
            println!("     - Found {} attester duties for epoch {}", duties.len(), current_epoch);
            
            for (i, duty) in duties.iter().take(3).enumerate() {
                println!("     - Duty {}: validator {} -> slot {}, committee {}", 
                         i, duty.validator_index, duty.slot, duty.committee_index);
                println!("       - Committee length: {}", duty.committee_length);
                println!("       - Validator committee index: {}", duty.validator_committee_index);
            }
        }
        Err(e) => {
            println!("     ⚠️  Failed to get attester duties: {e}");
        }
    }

    // Get proposer duties
    match client.get_proposer_duties(current_epoch).await {
        Ok(duties) => {
            println!("     - Found {} proposer duties for epoch {}", duties.len(), current_epoch);
            
            for (i, duty) in duties.iter().take(3).enumerate() {
                println!("     - Proposer {}: validator {} -> slot {}", 
                         i, duty.validator_index, duty.slot);
            }
        }
        Err(e) => {
            println!("     ⚠️  Failed to get proposer duties: {e}");
        }
    }

    // Get sync committee duties
    match client.get_sync_duties(current_epoch, &validator_indices).await {
        Ok(duties) => {
            println!("     - Found {} sync committee duties for epoch {}", duties.len(), current_epoch);
            
            for (i, duty) in duties.iter().take(3).enumerate() {
                println!("     - Sync duty {}: validator {} -> indices {:?}", 
                         i, duty.validator_index, duty.validator_sync_committee_indices);
            }
        }
        Err(e) => {
            println!("     ⚠️  Failed to get sync duties: {e}");
        }
    }

    Ok(())
}

#[cfg(feature = "lodestar-consensus")]
async fn get_recent_beacon_blocks(client: &EthereumClient) -> Result<(), ClientError> {
    println!("  🧱 Getting recent beacon blocks...");
    
    // Get recent block headers
    match client.get_beacon_block_headers(None, None).await {
        Ok(headers) => {
            println!("     - Found {} recent block headers", headers.len());
            
            for (i, header) in headers.iter().take(3).enumerate() {
                println!("     - Block {}: slot {}, proposer {}", 
                         i, header.slot, header.proposer_index);
                println!("       - Parent root: {}", header.parent_root);
                println!("       - State root: {}", header.state_root);
            }
            
            // Get full block for the first header
            if let Some(first_header) = headers.first() {
                let block_id = helpers::slot_to_block_id(first_header.slot);
                
                match client.get_beacon_block(&block_id).await {
                    Ok(block) => {
                        println!("     - Full block retrieved for slot {}", block.slot);
                        println!("       - Attestations: {}", block.body.attestations.len());
                        println!("       - Deposits: {}", block.body.deposits.len());
                        println!("       - Voluntary exits: {}", block.body.voluntary_exits.len());
                        println!("       - Proposer slashings: {}", block.body.proposer_slashings.len());
                        println!("       - Attester slashings: {}", block.body.attester_slashings.len());
                    }
                    Err(e) => {
                        println!("       ⚠️  Failed to get full block: {e}");
                    }
                }

                // Get block attestations
                match client.get_block_attestations(&block_id).await {
                    Ok(attestations) => {
                        println!("       - Block contains {} attestations", attestations.len());
                        
                        for (i, attestation) in attestations.iter().take(2).enumerate() {
                            println!("         - Attestation {}: slot {}, index {}", 
                                     i, attestation.data.slot, attestation.data.index);
                        }
                    }
                    Err(e) => {
                        println!("       ⚠️  Failed to get block attestations: {e}");
                    }
                }
            }
        }
        Err(e) => {
            println!("     ❌ Failed to get block headers: {e}");
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(feature = "lodestar-consensus")]
async fn get_committee_information(client: &EthereumClient) -> Result<(), ClientError> {
    println!("  🏛️ Getting committee information...");
    
    let head_state = helpers::head_state_id();
    let current_epoch: Epoch = 0;

    // Get committees for current epoch
    match client.get_epoch_committees(&head_state, Some(current_epoch), None, None).await {
        Ok(committees) => {
            println!("     - Found {} committees for epoch {}", committees.len(), current_epoch);
            
            for (i, committee) in committees.iter().take(3).enumerate() {
                println!("     - Committee {}: slot {}, index {}", 
                         i, committee.slot, committee.index);
                println!("       - Validators: {} members", committee.validators.len());
                
                // Show first few validator indices
                let validator_preview: Vec<String> = committee.validators
                    .iter()
                    .take(5)
                    .map(|v| v.to_string())
                    .collect();
                println!("       - First validators: [{}{}]", 
                         validator_preview.join(", "),
                         if committee.validators.len() > 5 { ", ..." } else { "" });
            }
            
            // Calculate some statistics
            let total_validators: usize = committees.iter().map(|c| c.validators.len()).sum();
            let avg_committee_size = if !committees.is_empty() {
                total_validators / committees.len()
            } else {
                0
            };
            
            println!("     - Total validator slots: {}", total_validators);
            println!("     - Average committee size: {}", avg_committee_size);
        }
        Err(e) => {
            println!("     ❌ Failed to get committees: {e}");
            return Err(e);
        }
    }

    // Get committees for a specific slot
    let target_slot: Slot = helpers::epoch_to_first_slot(current_epoch);
    match client.get_epoch_committees(&head_state, None, None, Some(target_slot)).await {
        Ok(slot_committees) => {
            println!("     - Found {} committees for slot {}", slot_committees.len(), target_slot);
        }
        Err(e) => {
            println!("     ⚠️  Failed to get slot committees: {e}");
        }
    }

    Ok(())
} 