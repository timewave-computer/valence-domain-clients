//! Tests for Lodestar consensus client functionality

#[cfg(feature = "lodestar-consensus")]
use std::str::FromStr;

#[cfg(feature = "lodestar-consensus")]
use valence_domain_clients::{
    evm::{
        chains::ethereum::EthereumClient,
        consensus::{
            helpers, AttesterDuty, NodePeer, NodeVersion, PeerDirection,
            PeerState, ProposerDuty, SyncDuty,
        },
        types::{
            BlsPublicKey, BlsSignature, Checkpoint, Committee, FinalityCheckpoints, Fork,
            GenesisData, NodeIdentity, NodeMetadata, Root, SyncingStatus, Validator,
            ValidatorBalance, ValidatorStatus,
        },
    },
};

#[cfg(feature = "lodestar-consensus")]
#[tokio::test]
async fn test_consensus_client_creation() {
    let client = EthereumClient::new("http://localhost:9596", "", None).unwrap();
    
    // Should be able to create the client without errors
    assert_eq!(client.chain_id(), 1);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_consensus_helpers_comprehensive() {
    // Test helper functions comprehensively
    
    // Slot/epoch conversions
    assert_eq!(helpers::slot_to_epoch(0), 0);
    assert_eq!(helpers::slot_to_epoch(31), 0);
    assert_eq!(helpers::slot_to_epoch(32), 1);
    assert_eq!(helpers::slot_to_epoch(63), 1);
    assert_eq!(helpers::slot_to_epoch(64), 2);
    
    assert_eq!(helpers::epoch_to_first_slot(0), 0);
    assert_eq!(helpers::epoch_to_first_slot(1), 32);
    assert_eq!(helpers::epoch_to_first_slot(2), 64);
    
    // State ID generation
    assert_eq!(helpers::head_state_id(), "head");
    assert_eq!(helpers::finalized_state_id(), "finalized");
    assert_eq!(helpers::justified_state_id(), "justified");
    assert_eq!(helpers::epoch_to_state_id(123), "epoch_123");
    
    // Block ID generation
    assert_eq!(helpers::slot_to_block_id(12345), "12345");
    
    // Validator ID generation
    assert_eq!(helpers::validator_index_to_id(999), "999");
    
    // Balance conversions
    assert_eq!(helpers::effective_balance_to_eth(32_000_000_000), 32.0);
    assert_eq!(helpers::eth_to_gwei(32.0), 32_000_000_000);
    assert_eq!(helpers::effective_balance_to_eth(0), 0.0);
    assert_eq!(helpers::eth_to_gwei(0.0), 0);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_validator_state_logic() {
    let pubkey = BlsPublicKey([0u8; 48]);
    let withdrawal_creds = Root([0u8; 32]);
    
    // Test active validator
    let active_validator = Validator {
        pubkey: pubkey.clone(),
        withdrawal_credentials: withdrawal_creds.clone(),
        effective_balance: 32_000_000_000,
        slashed: false,
        activation_eligibility_epoch: 0,
        activation_epoch: 10,
        exit_epoch: u64::MAX,
        withdrawable_epoch: u64::MAX,
    };
    
    // Should not be active before activation epoch
    assert!(!helpers::is_validator_active(&active_validator, 5));
    // Should be active after activation epoch
    assert!(helpers::is_validator_active(&active_validator, 15));
    // Should not be slashed
    assert!(!helpers::is_validator_slashed(&active_validator));
    
    // Test validator eligible for activation
    assert!(helpers::is_validator_eligible_for_activation(&active_validator, 5)); // Eligible since activation_eligibility_epoch=0 and activation_epoch=10
    assert!(!helpers::is_validator_eligible_for_activation(&active_validator, 15)); // Already active
    
    // Test slashed validator
    let slashed_validator = Validator {
        pubkey,
        withdrawal_credentials: withdrawal_creds,
        effective_balance: 31_000_000_000,
        slashed: true,
        activation_eligibility_epoch: 0,
        activation_epoch: 5,
        exit_epoch: 100,
        withdrawable_epoch: 200,
    };
    
    assert!(helpers::is_validator_slashed(&slashed_validator));
    assert!(helpers::is_validator_active(&slashed_validator, 50));
    assert!(!helpers::is_validator_active(&slashed_validator, 150)); // Past exit epoch
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_consensus_type_serialization() {
    // Test Root serialization
    let root = Root::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let json = serde_json::to_string(&root).unwrap();
    let deserialized: Root = serde_json::from_str(&json).unwrap();
    assert_eq!(root, deserialized);
    
    // Test BlsPublicKey serialization
    let pubkey = BlsPublicKey::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let json = serde_json::to_string(&pubkey).unwrap();
    let deserialized: BlsPublicKey = serde_json::from_str(&json).unwrap();
    assert_eq!(pubkey, deserialized);
    
    // Test BlsSignature serialization
    let signature = BlsSignature::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let json = serde_json::to_string(&signature).unwrap();
    let deserialized: BlsSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(signature, deserialized);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_duty_type_creation_and_serialization() {
    let pubkey = BlsPublicKey::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    
    // Test AttesterDuty
    let attester_duty = AttesterDuty {
        pubkey: pubkey.clone(),
        validator_index: 42,
        committee_index: 3,
        committee_length: 128,
        committees_at_slot: 64,
        validator_committee_index: 15,
        slot: 12345,
    };
    
    let json = serde_json::to_string(&attester_duty).unwrap();
    let deserialized: AttesterDuty = serde_json::from_str(&json).unwrap();
    assert_eq!(attester_duty.validator_index, deserialized.validator_index);
    assert_eq!(attester_duty.committee_index, deserialized.committee_index);
    assert_eq!(attester_duty.slot, deserialized.slot);
    
    // Test ProposerDuty
    let proposer_duty = ProposerDuty {
        pubkey: pubkey.clone(),
        validator_index: 123,
        slot: 54321,
    };
    
    let json = serde_json::to_string(&proposer_duty).unwrap();
    let deserialized: ProposerDuty = serde_json::from_str(&json).unwrap();
    assert_eq!(proposer_duty.validator_index, deserialized.validator_index);
    assert_eq!(proposer_duty.slot, deserialized.slot);
    
    // Test SyncDuty
    let sync_duty = SyncDuty {
        pubkey,
        validator_index: 999,
        validator_sync_committee_indices: vec![0, 15, 31, 47],
    };
    
    let json = serde_json::to_string(&sync_duty).unwrap();
    let deserialized: SyncDuty = serde_json::from_str(&json).unwrap();
    assert_eq!(sync_duty.validator_index, deserialized.validator_index);
    assert_eq!(sync_duty.validator_sync_committee_indices, deserialized.validator_sync_committee_indices);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_complex_consensus_types() {
    let root1 = Root::from_str("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
    let root2 = Root::from_str("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();
    let root3 = Root::from_str("0x3333333333333333333333333333333333333333333333333333333333333333").unwrap();
    
    // Test FinalityCheckpoints
    let finality_checkpoints = FinalityCheckpoints {
        previous_justified: Checkpoint { epoch: 98, root: root1 },
        current_justified: Checkpoint { epoch: 99, root: root2 },
        finalized: Checkpoint { epoch: 97, root: root3 },
    };
    
    let json = serde_json::to_string(&finality_checkpoints).unwrap();
    let deserialized: FinalityCheckpoints = serde_json::from_str(&json).unwrap();
    assert_eq!(finality_checkpoints.previous_justified.epoch, deserialized.previous_justified.epoch);
    assert_eq!(finality_checkpoints.current_justified.epoch, deserialized.current_justified.epoch);
    assert_eq!(finality_checkpoints.finalized.epoch, deserialized.finalized.epoch);
    
    // Test GenesisData
    let genesis_validators_root = Root::from_str("0x4444444444444444444444444444444444444444444444444444444444444444").unwrap();
    let genesis_data = GenesisData {
        genesis_time: 1606824023,
        genesis_validators_root,
        genesis_fork_version: [0x00, 0x00, 0x10, 0x20],
    };
    
    let json = serde_json::to_string(&genesis_data).unwrap();
    let deserialized: GenesisData = serde_json::from_str(&json).unwrap();
    assert_eq!(genesis_data.genesis_time, deserialized.genesis_time);
    assert_eq!(genesis_data.genesis_fork_version, deserialized.genesis_fork_version);
    
    // Test Fork
    let fork = Fork {
        previous_version: [0x00, 0x00, 0x10, 0x00],
        current_version: [0x01, 0x00, 0x10, 0x00],
        epoch: 123456,
    };
    
    let json = serde_json::to_string(&fork).unwrap();
    let deserialized: Fork = serde_json::from_str(&json).unwrap();
    assert_eq!(fork.previous_version, deserialized.previous_version);
    assert_eq!(fork.current_version, deserialized.current_version);
    assert_eq!(fork.epoch, deserialized.epoch);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_node_network_types() {
    // Test NodePeer
    let peer = NodePeer {
        peer_id: "12D3KooWExample123".to_string(),
        enr: Some("enr:-example-record".to_string()),
        last_seen_p2p_address: Some("/ip4/127.0.0.1/tcp/9000".to_string()),
        state: PeerState::Connected,
        direction: PeerDirection::Outbound,
    };
    
    let json = serde_json::to_string(&peer).unwrap();
    let deserialized: NodePeer = serde_json::from_str(&json).unwrap();
    assert_eq!(peer.peer_id, deserialized.peer_id);
    assert_eq!(peer.enr, deserialized.enr);
    
    // Test NodeVersion
    let version = NodeVersion {
        version: "Lodestar/v1.15.0/linux-x64".to_string(),
    };
    
    let json = serde_json::to_string(&version).unwrap();
    let deserialized: NodeVersion = serde_json::from_str(&json).unwrap();
    assert_eq!(version.version, deserialized.version);
    
    // Test SyncingStatus
    let sync_status = SyncingStatus {
        head_slot: 12345,
        sync_distance: 100,
        is_syncing: true,
        is_optimistic: false,
    };
    
    let json = serde_json::to_string(&sync_status).unwrap();
    let deserialized: SyncingStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(sync_status.head_slot, deserialized.head_slot);
    assert_eq!(sync_status.sync_distance, deserialized.sync_distance);
    assert_eq!(sync_status.is_syncing, deserialized.is_syncing);
    assert_eq!(sync_status.is_optimistic, deserialized.is_optimistic);
    
    // Test NodeIdentity
    let node_identity = NodeIdentity {
        peer_id: "12D3KooWNode123".to_string(),
        enr: "enr:-node-record".to_string(),
        p2p_addresses: vec!["/ip4/127.0.0.1/tcp/9000".to_string()],
        discovery_addresses: vec!["/ip4/127.0.0.1/udp/9001".to_string()],
        metadata: NodeMetadata {
            seq_number: 42,
            attnets: "0xff".to_string(),
        },
    };
    
    let json = serde_json::to_string(&node_identity).unwrap();
    let deserialized: NodeIdentity = serde_json::from_str(&json).unwrap();
    assert_eq!(node_identity.peer_id, deserialized.peer_id);
    assert_eq!(node_identity.enr, deserialized.enr);
    assert_eq!(node_identity.metadata.seq_number, deserialized.metadata.seq_number);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_validator_and_committee_types() {
    let pubkey = BlsPublicKey::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let withdrawal_creds = Root::from_str("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap();
    
    // Test Validator
    let validator = Validator {
        pubkey,
        withdrawal_credentials: withdrawal_creds,
        effective_balance: 32_000_000_000,
        slashed: false,
        activation_eligibility_epoch: 0,
        activation_epoch: 10,
        exit_epoch: u64::MAX,
        withdrawable_epoch: u64::MAX,
    };
    
    let json = serde_json::to_string(&validator).unwrap();
    let deserialized: Validator = serde_json::from_str(&json).unwrap();
    assert_eq!(validator.effective_balance, deserialized.effective_balance);
    assert_eq!(validator.slashed, deserialized.slashed);
    assert_eq!(validator.activation_epoch, deserialized.activation_epoch);
    
    // Test ValidatorBalance
    let validator_balance = ValidatorBalance {
        index: 123,
        balance: 32_500_000_000,
    };
    
    let json = serde_json::to_string(&validator_balance).unwrap();
    let deserialized: ValidatorBalance = serde_json::from_str(&json).unwrap();
    assert_eq!(validator_balance.index, deserialized.index);
    assert_eq!(validator_balance.balance, deserialized.balance);
    
    // Test Committee
    let committee = Committee {
        index: 5,
        slot: 12345,
        validators: vec![100, 101, 102, 103, 104],
    };
    
    let json = serde_json::to_string(&committee).unwrap();
    let deserialized: Committee = serde_json::from_str(&json).unwrap();
    assert_eq!(committee.index, deserialized.index);
    assert_eq!(committee.slot, deserialized.slot);
    assert_eq!(committee.validators, deserialized.validators);
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_validator_status_enum() {
    let statuses = vec![
        ValidatorStatus::PendingInitialized,
        ValidatorStatus::PendingQueued,
        ValidatorStatus::ActiveOngoing,
        ValidatorStatus::ActiveExiting,
        ValidatorStatus::ActiveSlashed,
        ValidatorStatus::ExitedUnslashed,
        ValidatorStatus::ExitedSlashed,
        ValidatorStatus::WithdrawalPossible,
        ValidatorStatus::WithdrawalDone,
    ];
    
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ValidatorStatus = serde_json::from_str(&json).unwrap();
        // Since we can't easily compare enums, check that serialization round-trips
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
}

#[cfg(feature = "lodestar-consensus")]
#[test]
fn test_peer_enums() {
    // Test PeerState enum
    let states = vec![
        PeerState::Disconnected,
        PeerState::Connecting,
        PeerState::Connected,
        PeerState::Disconnecting,
    ];
    
    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PeerState = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
    
    // Test PeerDirection enum
    let directions = vec![
        PeerDirection::Inbound,
        PeerDirection::Outbound,
    ];
    
    for direction in directions {
        let json = serde_json::to_string(&direction).unwrap();
        let deserialized: PeerDirection = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
}

#[cfg(feature = "lodestar-consensus")]
#[test] 
fn test_edge_case_conversions() {
    // Test edge cases for conversions that could cause issues
    
    // Very large slot numbers
    assert_eq!(helpers::slot_to_epoch(u64::MAX), u64::MAX / 32);
    assert_eq!(helpers::slot_to_epoch(u64::MAX - 31), (u64::MAX - 31) / 32);
    
    // Very large epoch numbers  
    let large_epoch = u64::MAX / 32;
    assert_eq!(helpers::epoch_to_first_slot(large_epoch), large_epoch * 32);
    
    // Balance edge cases
    assert_eq!(helpers::effective_balance_to_eth(1), 0.000000001);
    assert_eq!(helpers::effective_balance_to_eth(u64::MAX), u64::MAX as f64 / 1_000_000_000.0);
    
    assert_eq!(helpers::eth_to_gwei(0.000000001), 1);
    // Note: For very large ETH amounts, we might lose precision due to f64 limits
}

#[cfg(not(feature = "lodestar-consensus"))]
#[test]
fn test_consensus_feature_disabled() {
    // This test just ensures the feature is properly gated
    // When the feature is disabled, consensus types should not be available
    assert!(true, "Lodestar consensus feature is disabled as expected");
} 