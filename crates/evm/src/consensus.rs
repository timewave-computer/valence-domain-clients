//-----------------------------------------------------------------------------
// Lodestar Consensus Client API Implementation
//-----------------------------------------------------------------------------

//! Lodestar consensus client API support for interacting with Ethereum beacon chain.
//!
//! This module provides consensus layer capabilities that are compatible with the Lodestar
//! Ethereum consensus client, allowing for detailed analysis of beacon chain state,
//! validator operations, and consensus layer interactions.

#[cfg(feature = "lodestar-consensus")]
use async_trait::async_trait;

#[cfg(feature = "lodestar-consensus")]
use crate::types::{
    Attestation, BeaconBlock, BeaconBlockHeader, BlsPublicKey,
    Committee, Epoch, FinalityCheckpoints, Fork, GenesisData, NodeIdentity, Root, Slot,
    SyncingStatus, Validator, ValidatorBalance, ValidatorIndex, ValidatorStatus,
};

#[cfg(feature = "lodestar-consensus")]
use valence_core::error::ClientError;

/// Trait for Lodestar consensus client API operations
#[cfg(feature = "lodestar-consensus")]
#[async_trait]
pub trait LodestarConsensus {
    //-----------------------------------------------------------------------------
    // Beacon API - Chain access
    //-----------------------------------------------------------------------------

    /// Get the genesis information
    async fn get_genesis(&self) -> Result<GenesisData, ClientError>;

    /// Get the current state root
    async fn get_state_root(&self, state_id: &str) -> Result<Root, ClientError>;

    /// Get fork information for a state
    async fn get_state_fork(&self, state_id: &str) -> Result<Fork, ClientError>;

    /// Get finality checkpoints for a state
    async fn get_state_finality_checkpoints(
        &self,
        state_id: &str,
    ) -> Result<FinalityCheckpoints, ClientError>;

    /// Get a beacon block by block ID
    async fn get_beacon_block(&self, block_id: &str) -> Result<BeaconBlock, ClientError>;

    /// Get a beacon block header by block ID
    async fn get_beacon_block_header(
        &self,
        block_id: &str,
    ) -> Result<BeaconBlockHeader, ClientError>;

    /// Get beacon block headers with optional filters
    async fn get_beacon_block_headers(
        &self,
        slot: Option<Slot>,
        parent_root: Option<&Root>,
    ) -> Result<Vec<BeaconBlockHeader>, ClientError>;

    /// Get block root by block ID
    async fn get_block_root(&self, block_id: &str) -> Result<Root, ClientError>;

    /// Get attestations from a block
    async fn get_block_attestations(
        &self,
        block_id: &str,
    ) -> Result<Vec<Attestation>, ClientError>;

    //-----------------------------------------------------------------------------
    // Beacon API - Pool operations
    //-----------------------------------------------------------------------------

    /// Get pending attestations from the pool
    async fn get_pending_attestations(
        &self,
        slot: Option<Slot>,
        committee_index: Option<u64>,
    ) -> Result<Vec<Attestation>, ClientError>;

    /// Submit an attestation to the pool
    async fn submit_attestations(
        &self,
        attestations: &[Attestation],
    ) -> Result<(), ClientError>;

    /// Submit a signed beacon block
    async fn submit_block(&self, block: &BeaconBlock) -> Result<(), ClientError>;

    //-----------------------------------------------------------------------------
    // Beacon API - Validator operations
    //-----------------------------------------------------------------------------

    /// Get validator by index or public key
    async fn get_validator(
        &self,
        state_id: &str,
        validator_id: &str,
    ) -> Result<Validator, ClientError>;

    /// Get multiple validators by indices or public keys
    async fn get_validators(
        &self,
        state_id: &str,
        validator_ids: &[String],
        status_filter: Option<&[ValidatorStatus]>,
    ) -> Result<Vec<Validator>, ClientError>;

    /// Get validator balances
    async fn get_validator_balances(
        &self,
        state_id: &str,
        validator_ids: &[String],
    ) -> Result<Vec<ValidatorBalance>, ClientError>;

    /// Get committees for a given epoch
    async fn get_epoch_committees(
        &self,
        state_id: &str,
        epoch: Option<Epoch>,
        index: Option<u64>,
        slot: Option<Slot>,
    ) -> Result<Vec<Committee>, ClientError>;

    //-----------------------------------------------------------------------------
    // Beacon API - Validator duties
    //-----------------------------------------------------------------------------

    /// Get attester duties for validators in an epoch
    async fn get_attester_duties(
        &self,
        epoch: Epoch,
        validator_indices: &[ValidatorIndex],
    ) -> Result<Vec<AttesterDuty>, ClientError>;

    /// Get proposer duties for an epoch
    async fn get_proposer_duties(&self, epoch: Epoch) -> Result<Vec<ProposerDuty>, ClientError>;

    /// Get sync committee duties for validators in an epoch
    async fn get_sync_duties(
        &self,
        epoch: Epoch,
        validator_indices: &[ValidatorIndex],
    ) -> Result<Vec<SyncDuty>, ClientError>;

    //-----------------------------------------------------------------------------
    // Node API operations
    //-----------------------------------------------------------------------------

    /// Get node identity information
    async fn get_node_identity(&self) -> Result<NodeIdentity, ClientError>;

    /// Get node peers
    async fn get_node_peers(&self) -> Result<Vec<NodePeer>, ClientError>;

    /// Get node sync status
    async fn get_sync_status(&self) -> Result<SyncingStatus, ClientError>;

    /// Get node version
    async fn get_node_version(&self) -> Result<NodeVersion, ClientError>;

    //-----------------------------------------------------------------------------
    // Debug API operations (optional)
    //-----------------------------------------------------------------------------

    /// Get debug beacon state (if debug APIs are enabled)
    async fn get_debug_beacon_state(&self, state_id: &str) -> Result<serde_json::Value, ClientError>;

    /// Get debug beacon heads
    async fn get_debug_beacon_heads(&self) -> Result<Vec<BeaconBlockHeader>, ClientError>;
}

/// Attester duty information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttesterDuty {
    /// Public key of the validator
    pub pubkey: BlsPublicKey,
    /// Validator index
    pub validator_index: ValidatorIndex,
    /// Committee index
    pub committee_index: u64,
    /// Committee length
    pub committee_length: u64,
    /// Committees at slot
    pub committees_at_slot: u64,
    /// Validator committee index
    pub validator_committee_index: u64,
    /// Slot
    pub slot: Slot,
}

/// Proposer duty information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProposerDuty {
    /// Public key of the validator
    pub pubkey: BlsPublicKey,
    /// Validator index
    pub validator_index: ValidatorIndex,
    /// Slot
    pub slot: Slot,
}

/// Sync committee duty information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncDuty {
    /// Public key of the validator
    pub pubkey: BlsPublicKey,
    /// Validator index
    pub validator_index: ValidatorIndex,
    /// Validator sync committee indices
    pub validator_sync_committee_indices: Vec<u64>,
}

/// Node peer information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodePeer {
    /// Peer ID
    pub peer_id: String,
    /// ENR
    pub enr: Option<String>,
    /// Last seen timestamp
    pub last_seen_p2p_address: Option<String>,
    /// State
    pub state: PeerState,
    /// Direction
    pub direction: PeerDirection,
}

/// Peer state
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeerState {
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Disconnecting
    Disconnecting,
}

/// Peer direction
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeerDirection {
    /// Inbound
    Inbound,
    /// Outbound
    Outbound,
}

/// Node version information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeVersion {
    /// Version string
    pub version: String,
}

/// Helper functions for constructing consensus requests
#[cfg(feature = "lodestar-consensus")]
pub mod helpers {
    use super::*;

    /// Convert slot to block ID string
    pub fn slot_to_block_id(slot: Slot) -> String {
        slot.to_string()
    }

    /// Convert epoch to state ID string
    pub fn epoch_to_state_id(epoch: Epoch) -> String {
        format!("epoch_{epoch}")
    }

    /// Get head state ID
    pub fn head_state_id() -> String {
        "head".to_string()
    }

    /// Get finalized state ID
    pub fn finalized_state_id() -> String {
        "finalized".to_string()
    }

    /// Get justified state ID
    pub fn justified_state_id() -> String {
        "justified".to_string()
    }

    /// Convert validator index to validator ID string
    pub fn validator_index_to_id(index: ValidatorIndex) -> String {
        index.to_string()
    }

    /// Convert public key to validator ID string
    pub fn pubkey_to_validator_id(pubkey: &BlsPublicKey) -> String {
        pubkey.to_string()
    }

    /// Get current epoch from slot
    pub fn slot_to_epoch(slot: Slot) -> Epoch {
        slot / 32 // 32 slots per epoch
    }

    /// Get first slot of epoch
    pub fn epoch_to_first_slot(epoch: Epoch) -> Slot {
        epoch * 32
    }

    /// Check if validator is active
    pub fn is_validator_active(validator: &Validator, epoch: Epoch) -> bool {
        validator.activation_epoch <= epoch && epoch < validator.exit_epoch
    }

    /// Check if validator is eligible for activation
    pub fn is_validator_eligible_for_activation(validator: &Validator, epoch: Epoch) -> bool {
        validator.activation_eligibility_epoch <= epoch && validator.activation_epoch > epoch
    }

    /// Check if validator is slashed
    pub fn is_validator_slashed(validator: &Validator) -> bool {
        validator.slashed
    }

    /// Get validator effective balance in ETH
    pub fn effective_balance_to_eth(balance_gwei: u64) -> f64 {
        balance_gwei as f64 / 1_000_000_000.0
    }

    /// Convert ETH to Gwei
    pub fn eth_to_gwei(eth: f64) -> u64 {
        (eth * 1_000_000_000.0) as u64
    }
}

#[cfg(feature = "lodestar-consensus")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_slot_to_epoch() {
        assert_eq!(helpers::slot_to_epoch(0), 0);
        assert_eq!(helpers::slot_to_epoch(31), 0);
        assert_eq!(helpers::slot_to_epoch(32), 1);
        assert_eq!(helpers::slot_to_epoch(64), 2);
    }

    #[test]
    fn test_epoch_to_first_slot() {
        assert_eq!(helpers::epoch_to_first_slot(0), 0);
        assert_eq!(helpers::epoch_to_first_slot(1), 32);
        assert_eq!(helpers::epoch_to_first_slot(2), 64);
    }

    #[test]
    fn test_effective_balance_conversion() {
        assert_eq!(helpers::effective_balance_to_eth(32_000_000_000), 32.0);
        assert_eq!(helpers::eth_to_gwei(32.0), 32_000_000_000);
    }

    #[test]
    fn test_state_id_helpers() {
        assert_eq!(helpers::head_state_id(), "head");
        assert_eq!(helpers::finalized_state_id(), "finalized");
        assert_eq!(helpers::justified_state_id(), "justified");
        assert_eq!(helpers::epoch_to_state_id(123), "epoch_123");
    }

    #[test]
    fn test_validator_id_helpers() {
        assert_eq!(helpers::validator_index_to_id(12345), "12345");
        
        let pubkey = BlsPublicKey::from_str(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        ).unwrap();
        assert_eq!(
            helpers::pubkey_to_validator_id(&pubkey),
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }

    #[test]
    fn test_validator_status_checks() {
        let validator = Validator {
            pubkey: BlsPublicKey([0u8; 48]),
            withdrawal_credentials: Root([0u8; 32]),
            effective_balance: 32_000_000_000,
            slashed: false,
            activation_eligibility_epoch: 0,
            activation_epoch: 10,
            exit_epoch: u64::MAX,
            withdrawable_epoch: u64::MAX,
        };

        // Not active yet
        assert!(!helpers::is_validator_active(&validator, 5));
        // Active
        assert!(helpers::is_validator_active(&validator, 15));
        // Not slashed
        assert!(!helpers::is_validator_slashed(&validator));
    }

    #[test]
    fn test_block_id_helpers() {
        assert_eq!(helpers::slot_to_block_id(12345), "12345");
        assert_eq!(helpers::slot_to_block_id(0), "0");
    }

    #[test]
    fn test_slot_epoch_edge_cases() {
        // Test edge cases for slot to epoch conversion
        assert_eq!(helpers::slot_to_epoch(1), 0);
        assert_eq!(helpers::slot_to_epoch(31), 0);
        assert_eq!(helpers::slot_to_epoch(32), 1);
        assert_eq!(helpers::slot_to_epoch(33), 1);
        assert_eq!(helpers::slot_to_epoch(63), 1);
        assert_eq!(helpers::slot_to_epoch(64), 2);
        
        // Large numbers
        assert_eq!(helpers::slot_to_epoch(1000000), 31250);
    }

    #[test] 
    fn test_epoch_slot_edge_cases() {
        // Test edge cases for epoch to slot conversion
        assert_eq!(helpers::epoch_to_first_slot(0), 0);
        assert_eq!(helpers::epoch_to_first_slot(1), 32);
        assert_eq!(helpers::epoch_to_first_slot(100), 3200);
        
        // Large numbers
        assert_eq!(helpers::epoch_to_first_slot(31250), 1000000);
    }

    #[test]
    fn test_balance_conversion_edge_cases() {
        // Test zero
        assert_eq!(helpers::effective_balance_to_eth(0), 0.0);
        assert_eq!(helpers::eth_to_gwei(0.0), 0);
        
        // Test small amounts
        assert_eq!(helpers::effective_balance_to_eth(1), 0.000000001);
        assert_eq!(helpers::eth_to_gwei(0.000000001), 1);
        
        // Test large amounts
        assert_eq!(helpers::effective_balance_to_eth(100_000_000_000_000), 100_000.0);
        assert_eq!(helpers::eth_to_gwei(100_000.0), 100_000_000_000_000);
    }

    #[test]
    fn test_validator_eligibility_checks() {
        let validator = Validator {
            pubkey: BlsPublicKey([0u8; 48]),
            withdrawal_credentials: Root([0u8; 32]),
            effective_balance: 32_000_000_000,
            slashed: false,
            activation_eligibility_epoch: 5,
            activation_epoch: 10,
            exit_epoch: u64::MAX,
            withdrawable_epoch: u64::MAX,
        };

        // Not eligible yet
        assert!(!helpers::is_validator_eligible_for_activation(&validator, 4));
        // Eligible but not active
        assert!(helpers::is_validator_eligible_for_activation(&validator, 7));
        // Active, so not eligible for activation
        assert!(!helpers::is_validator_eligible_for_activation(&validator, 15));
    }

    #[test]
    fn test_slashed_validator() {
        let slashed_validator = Validator {
            pubkey: BlsPublicKey([1u8; 48]),
            withdrawal_credentials: Root([1u8; 32]),
            effective_balance: 31_000_000_000,
            slashed: true,
            activation_eligibility_epoch: 0,
            activation_epoch: 5,
            exit_epoch: 100,
            withdrawable_epoch: 200,
        };

        assert!(helpers::is_validator_slashed(&slashed_validator));
        assert!(helpers::is_validator_active(&slashed_validator, 50));
    }

    #[test]
    fn test_attester_duty_creation() {
        let pubkey = BlsPublicKey::from_str(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        ).unwrap();

        let duty = AttesterDuty {
            pubkey: pubkey.clone(),
            validator_index: 42,
            committee_index: 3,
            committee_length: 128,
            committees_at_slot: 64,
            validator_committee_index: 15,
            slot: 12345,
        };

        assert_eq!(duty.pubkey, pubkey);
        assert_eq!(duty.validator_index, 42);
        assert_eq!(duty.committee_index, 3);
        assert_eq!(duty.committee_length, 128);
        assert_eq!(duty.committees_at_slot, 64);
        assert_eq!(duty.validator_committee_index, 15);
        assert_eq!(duty.slot, 12345);

        // Test serialization
        let json = serde_json::to_string(&duty).unwrap();
        let deserialized: AttesterDuty = serde_json::from_str(&json).unwrap();
        assert_eq!(duty.validator_index, deserialized.validator_index);
        assert_eq!(duty.committee_index, deserialized.committee_index);
        assert_eq!(duty.slot, deserialized.slot);
    }

    #[test]
    fn test_proposer_duty_creation() {
        let pubkey = BlsPublicKey::from_str(
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        ).unwrap();

        let duty = ProposerDuty {
            pubkey: pubkey.clone(),
            validator_index: 123,
            slot: 54321,
        };

        assert_eq!(duty.pubkey, pubkey);
        assert_eq!(duty.validator_index, 123);
        assert_eq!(duty.slot, 54321);

        // Test serialization
        let json = serde_json::to_string(&duty).unwrap();
        let deserialized: ProposerDuty = serde_json::from_str(&json).unwrap();
        assert_eq!(duty.validator_index, deserialized.validator_index);
        assert_eq!(duty.slot, deserialized.slot);
    }

    #[test]
    fn test_sync_duty_creation() {
        let pubkey = BlsPublicKey::from_str(
            "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
        ).unwrap();

        let duty = SyncDuty {
            pubkey: pubkey.clone(),
            validator_index: 999,
            validator_sync_committee_indices: vec![0, 15, 31, 47],
        };

        assert_eq!(duty.pubkey, pubkey);
        assert_eq!(duty.validator_index, 999);
        assert_eq!(duty.validator_sync_committee_indices, vec![0, 15, 31, 47]);

        // Test serialization
        let json = serde_json::to_string(&duty).unwrap();
        let deserialized: SyncDuty = serde_json::from_str(&json).unwrap();
        assert_eq!(duty.validator_index, deserialized.validator_index);
        assert_eq!(duty.validator_sync_committee_indices, deserialized.validator_sync_committee_indices);
    }

    #[test]
    fn test_node_peer_creation() {
        let peer = NodePeer {
            peer_id: "12D3KooWExample123".to_string(),
            enr: Some("enr:-example-record".to_string()),
            last_seen_p2p_address: Some("/ip4/127.0.0.1/tcp/9000".to_string()),
            state: PeerState::Connected,
            direction: PeerDirection::Outbound,
        };

        assert_eq!(peer.peer_id, "12D3KooWExample123");
        assert_eq!(peer.enr, Some("enr:-example-record".to_string()));
        assert!(matches!(peer.state, PeerState::Connected));
        assert!(matches!(peer.direction, PeerDirection::Outbound));

        // Test serialization
        let json = serde_json::to_string(&peer).unwrap();
        let deserialized: NodePeer = serde_json::from_str(&json).unwrap();
        assert_eq!(peer.peer_id, deserialized.peer_id);
        assert_eq!(peer.enr, deserialized.enr);
    }

    #[test]
    fn test_peer_state_serialization() {
        let states = vec![
            PeerState::Disconnected,
            PeerState::Connecting,
            PeerState::Connected,
            PeerState::Disconnecting,
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: PeerState = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{state:?}"), format!("{deserialized:?}"));
        }
    }

    #[test]
    fn test_peer_direction_serialization() {
        let directions = vec![
            PeerDirection::Inbound,
            PeerDirection::Outbound,
        ];

        for direction in directions {
            let json = serde_json::to_string(&direction).unwrap();
            let deserialized: PeerDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{direction:?}"), format!("{deserialized:?}"));
        }
    }

    #[test]
    fn test_node_version_creation() {
        let version = NodeVersion {
            version: "Lodestar/v1.15.0/linux-x64".to_string(),
        };

        assert_eq!(version.version, "Lodestar/v1.15.0/linux-x64");

        // Test serialization
        let json = serde_json::to_string(&version).unwrap();
        let deserialized: NodeVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(version.version, deserialized.version);
    }
} 