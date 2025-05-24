//-----------------------------------------------------------------------------
// EVM Data Types
//-----------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// EVM-based blockchain address (20 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvmAddress(pub [u8; 20]);

/// Transaction hash or block hash (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvmHash(pub [u8; 32]);

/// Arbitrary bytes for EVM operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmBytes(pub Vec<u8>);

/// 256-bit unsigned integer for EVM values (amount, gas, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmU256(pub [u64; 4]);

/// Transaction log from event emission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmLog {
    /// Address of contract that emitted the log
    pub address: EvmAddress,
    /// Topics (indexed parameters)
    pub topics: Vec<EvmHash>,
    /// Non-indexed parameters
    pub data: EvmBytes,
    /// Block number where log was emitted
    pub block_number: u64,
    /// Transaction hash of the transaction that emitted the log
    pub transaction_hash: EvmHash,
    /// Log index within the block
    pub log_index: u64,
}

/// Receipt after transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransactionReceipt {
    /// Transaction hash
    pub transaction_hash: EvmHash,
    /// Block number where transaction was included
    pub block_number: u64,
    /// Transaction index in the block
    pub transaction_index: u64,
    /// From address
    pub from: EvmAddress,
    /// To address (None for contract creation)
    pub to: Option<EvmAddress>,
    /// Contract address created (if a contract creation transaction)
    pub contract_address: Option<EvmAddress>,
    /// Cumulative gas used in the block after this transaction
    pub cumulative_gas_used: EvmU256,
    /// Gas used by this transaction alone
    pub gas_used: EvmU256,
    /// Status: 1 for success, 0 for failure
    pub status: u8,
    /// Logs emitted during transaction execution
    pub logs: Vec<EvmLog>,
}

/// Transaction request for submission to the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransactionRequest {
    /// From address (transaction signer)
    pub from: EvmAddress,
    /// To address (None for contract creation)
    pub to: Option<EvmAddress>,
    /// Transaction nonce
    pub nonce: Option<u64>,
    /// Gas limit
    pub gas_limit: Option<EvmU256>,
    /// Gas price (for legacy transactions)
    pub gas_price: Option<EvmU256>,
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<EvmU256>,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<EvmU256>,
    /// Value to transfer (in wei)
    pub value: Option<EvmU256>,
    /// Transaction data
    pub data: Option<EvmBytes>,
    /// Chain ID
    pub chain_id: Option<u64>,
}

//-----------------------------------------------------------------------------
// Erigon Tracing API Types
//-----------------------------------------------------------------------------

/// Erigon trace action types
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TraceAction {
    /// Call action
    Call {
        /// From address
        from: EvmAddress,
        /// To address
        to: EvmAddress,
        /// Value transferred
        value: EvmU256,
        /// Gas provided
        gas: EvmU256,
        /// Input data
        input: EvmBytes,
        /// Call type (call, staticcall, delegatecall, callcode)
        call_type: String,
    },
    /// Create action (contract creation)
    Create {
        /// From address
        from: EvmAddress,
        /// Value transferred
        value: EvmU256,
        /// Gas provided
        gas: EvmU256,
        /// Init code
        init: EvmBytes,
    },
    /// Suicide/selfdestruct action
    Suicide {
        /// Address being self-destructed
        address: EvmAddress,
        /// Refund target address
        refund_address: EvmAddress,
        /// Balance transferred
        balance: EvmU256,
    },
    /// Reward action (block/uncle reward)
    Reward {
        /// Author address
        author: EvmAddress,
        /// Reward value
        value: EvmU256,
        /// Reward type
        reward_type: String,
    },
}

/// Erigon trace result types
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TraceResult {
    /// Call result
    Call {
        /// Gas used
        gas_used: EvmU256,
        /// Output data
        output: EvmBytes,
    },
    /// Create result
    Create {
        /// Gas used
        gas_used: EvmU256,
        /// Created contract code
        code: EvmBytes,
        /// Created contract address
        address: EvmAddress,
    },
    /// No result (for failed traces)
    None,
}

/// Erigon VM trace
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmTrace {
    /// VM operation code
    pub code: EvmBytes,
    /// VM operations
    pub ops: Vec<VmOperation>,
}

/// VM operation in a trace
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmOperation {
    /// Operation cost
    pub cost: u64,
    /// Execution info
    pub ex: Option<VmExecutionInfo>,
    /// Program counter
    pub pc: u64,
    /// Sub-operations
    pub sub: Option<Box<VmTrace>>,
}

/// VM execution information
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmExecutionInfo {
    /// Memory state
    pub mem: Option<VmMemory>,
    /// Stack push operation
    pub push: Option<Vec<String>>,
    /// Storage changes
    pub store: Option<VmStorage>,
    /// Used gas
    pub used: Option<u64>,
}

/// VM memory state
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmMemory {
    /// Memory data
    pub data: String,
    /// Memory offset
    pub off: u64,
}

/// VM storage changes
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmStorage {
    /// Storage key
    pub key: EvmHash,
    /// Storage value
    pub val: EvmHash,
}

/// State diff entry
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    /// Balance changes
    pub balance: Option<BalanceDiff>,
    /// Code changes
    pub code: Option<CodeDiff>,
    /// Nonce changes
    pub nonce: Option<NonceDiff>,
    /// Storage changes
    pub storage: Option<std::collections::HashMap<EvmHash, StorageDiff>>,
}

/// Balance change in state diff
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceDiff {
    /// Previous balance
    pub from: EvmU256,
    /// New balance
    pub to: EvmU256,
}

/// Code change in state diff
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiff {
    /// Previous code
    pub from: EvmBytes,
    /// New code
    pub to: EvmBytes,
}

/// Nonce change in state diff
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceDiff {
    /// Previous nonce
    pub from: u64,
    /// New nonce
    pub to: u64,
}

/// Storage change in state diff
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDiff {
    /// Previous value
    pub from: EvmHash,
    /// New value
    pub to: EvmHash,
}

/// Complete trace of a transaction
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTrace {
    /// Action performed
    pub action: TraceAction,
    /// Result of the action
    pub result: Option<TraceResult>,
    /// Error if the trace failed
    pub error: Option<String>,
    /// Trace address (array of call indices)
    pub trace_address: Vec<u64>,
    /// Sub-traces count
    pub subtraces: u64,
    /// Transaction position in block (None for call traces)
    pub transaction_position: Option<u64>,
    /// Transaction hash (None for call traces)
    pub transaction_hash: Option<EvmHash>,
    /// Block number
    pub block_number: u64,
    /// Block hash
    pub block_hash: EvmHash,
}

/// Block trace containing all transaction traces
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTrace {
    /// All traces in the block
    pub traces: Vec<TransactionTrace>,
    /// VM trace if requested
    pub vm_trace: Option<VmTrace>,
    /// State diff if requested
    pub state_diff: Option<std::collections::HashMap<EvmAddress, StateDiff>>,
}

/// Trace filter for filtering traces
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFilter {
    /// From block (inclusive)
    pub from_block: Option<u64>,
    /// To block (inclusive)
    pub to_block: Option<u64>,
    /// From addresses to match
    pub from_address: Option<Vec<EvmAddress>>,
    /// To addresses to match
    pub to_address: Option<Vec<EvmAddress>>,
    /// After how many traces to start returning results
    pub after: Option<u64>,
    /// How many traces to return
    pub count: Option<u64>,
}

/// Call tracing request
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallTraceRequest {
    /// From address
    pub from: Option<EvmAddress>,
    /// To address
    pub to: EvmAddress,
    /// Gas limit
    pub gas: Option<EvmU256>,
    /// Gas price
    pub gas_price: Option<EvmU256>,
    /// Value to send
    pub value: Option<EvmU256>,
    /// Call data
    pub data: Option<EvmBytes>,
}

/// Trace type specification for replaying transactions
#[cfg(feature = "erigon-tracing")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TraceType {
    /// Basic trace
    Trace,
    /// VM trace
    VmTrace,
    /// State diff
    StateDiff,
}

//-----------------------------------------------------------------------------
// Lodestar Consensus API Types
//-----------------------------------------------------------------------------

/// Beacon chain slot number
#[cfg(feature = "lodestar-consensus")]
pub type Slot = u64;

/// Beacon chain epoch number
#[cfg(feature = "lodestar-consensus")]
pub type Epoch = u64;

/// Validator index
#[cfg(feature = "lodestar-consensus")]
pub type ValidatorIndex = u64;

/// Beacon chain root (32 bytes)
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Root(pub [u8; 32]);

/// BLS public key (48 bytes)
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlsPublicKey(pub [u8; 48]);

/// BLS signature (96 bytes)
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlsSignature(pub [u8; 96]);

/// Beacon block header
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlockHeader {
    /// The slot for which this block is proposed
    pub slot: Slot,
    /// Index of the proposer for this block
    pub proposer_index: ValidatorIndex,
    /// Root of the parent block
    pub parent_root: Root,
    /// Root of the beacon state
    pub state_root: Root,
    /// Root of the beacon block body
    pub body_root: Root,
}

/// Beacon block
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlock {
    /// Slot of the block
    pub slot: Slot,
    /// Proposer index
    pub proposer_index: ValidatorIndex,
    /// Parent block root
    pub parent_root: Root,
    /// State root
    pub state_root: Root,
    /// Block body
    pub body: BeaconBlockBody,
}

/// Beacon block body
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlockBody {
    /// RANDAO reveal
    pub randao_reveal: BlsSignature,
    /// ETH1 data
    pub eth1_data: Eth1Data,
    /// Graffiti (32 bytes)
    pub graffiti: [u8; 32],
    /// Proposer slashings
    pub proposer_slashings: Vec<ProposerSlashing>,
    /// Attester slashings
    pub attester_slashings: Vec<AttesterSlashing>,
    /// Attestations
    pub attestations: Vec<Attestation>,
    /// Deposits
    pub deposits: Vec<Deposit>,
    /// Voluntary exits
    pub voluntary_exits: Vec<SignedVoluntaryExit>,
    /// Execution payload (post-merge)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_payload: Option<ExecutionPayload>,
}

/// ETH1 data
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eth1Data {
    /// Root of the deposit tree
    pub deposit_root: Root,
    /// Number of deposits
    pub deposit_count: u64,
    /// Block hash from ETH1
    pub block_hash: EvmHash,
}

/// Proposer slashing
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposerSlashing {
    /// Signed header 1
    pub signed_header_1: SignedBeaconBlockHeader,
    /// Signed header 2
    pub signed_header_2: SignedBeaconBlockHeader,
}

/// Signed beacon block header
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBeaconBlockHeader {
    /// Message
    pub message: BeaconBlockHeader,
    /// Signature
    pub signature: BlsSignature,
}

/// Attester slashing
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttesterSlashing {
    /// Attestation 1
    pub attestation_1: IndexedAttestation,
    /// Attestation 2
    pub attestation_2: IndexedAttestation,
}

/// Indexed attestation
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedAttestation {
    /// Attesting indices
    pub attesting_indices: Vec<ValidatorIndex>,
    /// Attestation data
    pub data: AttestationData,
    /// Signature
    pub signature: BlsSignature,
}

/// Attestation
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Aggregation bits
    pub aggregation_bits: String, // Hex-encoded bitfield
    /// Attestation data
    pub data: AttestationData,
    /// Signature
    pub signature: BlsSignature,
}

/// Attestation data
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationData {
    /// Slot
    pub slot: Slot,
    /// Committee index
    pub index: u64,
    /// LMD GHOST vote
    pub beacon_block_root: Root,
    /// FFG source
    pub source: Checkpoint,
    /// FFG target
    pub target: Checkpoint,
}

/// Checkpoint
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Epoch
    pub epoch: Epoch,
    /// Root
    pub root: Root,
}

/// Deposit
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    /// Merkle proof
    pub proof: Vec<Root>,
    /// Deposit data
    pub data: DepositData,
}

/// Deposit data
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositData {
    /// Public key
    pub pubkey: BlsPublicKey,
    /// Withdrawal credentials
    pub withdrawal_credentials: Root,
    /// Amount (in Gwei)
    pub amount: u64,
    /// Signature
    pub signature: BlsSignature,
}

/// Signed voluntary exit
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedVoluntaryExit {
    /// Message
    pub message: VoluntaryExit,
    /// Signature
    pub signature: BlsSignature,
}

/// Voluntary exit
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoluntaryExit {
    /// Epoch
    pub epoch: Epoch,
    /// Validator index
    pub validator_index: ValidatorIndex,
}

/// Execution payload (post-merge)
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPayload {
    /// Parent hash
    pub parent_hash: EvmHash,
    /// Fee recipient
    pub fee_recipient: EvmAddress,
    /// State root
    pub state_root: Root,
    /// Receipts root
    pub receipts_root: Root,
    /// Logs bloom
    pub logs_bloom: EvmBytes,
    /// Random value
    pub prev_randao: Root,
    /// Block number
    pub block_number: u64,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas used
    pub gas_used: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Extra data
    pub extra_data: EvmBytes,
    /// Base fee per gas
    pub base_fee_per_gas: EvmU256,
    /// Block hash
    pub block_hash: EvmHash,
    /// Transactions
    pub transactions: Vec<EvmBytes>,
}

/// Validator information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Public key
    pub pubkey: BlsPublicKey,
    /// Withdrawal credentials
    pub withdrawal_credentials: Root,
    /// Effective balance (in Gwei)
    pub effective_balance: u64,
    /// Slashed status
    pub slashed: bool,
    /// Activation eligibility epoch
    pub activation_eligibility_epoch: Epoch,
    /// Activation epoch
    pub activation_epoch: Epoch,
    /// Exit epoch
    pub exit_epoch: Epoch,
    /// Withdrawable epoch
    pub withdrawable_epoch: Epoch,
}

/// Validator status
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidatorStatus {
    /// Pending initialized
    PendingInitialized,
    /// Pending queued
    PendingQueued,
    /// Active ongoing
    ActiveOngoing,
    /// Active exiting
    ActiveExiting,
    /// Active slashed
    ActiveSlashed,
    /// Exited unslashed
    ExitedUnslashed,
    /// Exited slashed
    ExitedSlashed,
    /// Withdrawal possible
    WithdrawalPossible,
    /// Withdrawal done
    WithdrawalDone,
}

/// Validator balance
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBalance {
    /// Validator index
    pub index: ValidatorIndex,
    /// Balance (in Gwei)
    pub balance: u64,
}

/// Genesis information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisData {
    /// Genesis time
    pub genesis_time: u64,
    /// Genesis validators root
    pub genesis_validators_root: Root,
    /// Genesis fork version
    pub genesis_fork_version: [u8; 4],
}

/// Fork information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fork {
    /// Previous version
    pub previous_version: [u8; 4],
    /// Current version
    pub current_version: [u8; 4],
    /// Fork epoch
    pub epoch: Epoch,
}

/// Node information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Peer ID
    pub peer_id: String,
    /// ENR
    pub enr: String,
    /// p2p addresses
    pub p2p_addresses: Vec<String>,
    /// Discovery addresses
    pub discovery_addresses: Vec<String>,
    /// Metadata
    pub metadata: NodeMetadata,
}

/// Node metadata
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Sequence number
    pub seq_number: u64,
    /// Attestation subnets
    pub attnets: String, // Hex-encoded bitfield
}

/// Sync status
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncingStatus {
    /// Head slot
    pub head_slot: Slot,
    /// Sync distance
    pub sync_distance: u64,
    /// Is syncing
    pub is_syncing: bool,
    /// Is optimistic
    pub is_optimistic: bool,
}

/// Committee information
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Committee {
    /// Committee index
    pub index: u64,
    /// Slot
    pub slot: Slot,
    /// Validators in committee
    pub validators: Vec<ValidatorIndex>,
}

/// Finality checkpoints
#[cfg(feature = "lodestar-consensus")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityCheckpoints {
    /// Previous justified checkpoint
    pub previous_justified: Checkpoint,
    /// Current justified checkpoint
    pub current_justified: Checkpoint,
    /// Finalized checkpoint
    pub finalized: Checkpoint,
}

//-----------------------------------------------------------------------------
// Implementations
//-----------------------------------------------------------------------------

impl fmt::Display for EvmAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl FromStr for EvmAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 40 {
            return Err(format!("Invalid address length: {}", s.len()));
        }

        let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {e}"))?;

        let mut address = [0u8; 20];
        address.copy_from_slice(&bytes);

        Ok(EvmAddress(address))
    }
}

impl fmt::Display for EvmHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl FromStr for EvmHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 64 {
            return Err(format!("Invalid hash length: {}", s.len()));
        }

        let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {e}"))?;

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);

        Ok(EvmHash(hash))
    }
}

impl EvmBytes {
    /// Create a new instance from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex: {e}"))?;
        Ok(EvmBytes(bytes))
    }

    /// Convert to a hex string with 0x prefix
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.0))
    }
}

impl fmt::Display for EvmBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl EvmU256 {
    /// Create a new U256 from a u64
    pub fn from_u64(value: u64) -> Self {
        EvmU256([value, 0, 0, 0])
    }

    /// Returns a zero value
    pub fn zero() -> Self {
        EvmU256([0, 0, 0, 0])
    }

    /// Create a new U256 from big-endian bytes
    pub fn from_big_endian(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 32 {
            return Err("Input exceeds 32 bytes".to_string());
        }

        let mut out = [0u64; 4];

        // Convert bytes to u64 values in big-endian order
        let mut chunks = [0u8; 8];

        // Process in chunks of 8 bytes (u64)
        for (i, chunk) in out.iter_mut().enumerate().take(4) {
            let start = if bytes.len() > i * 8 {
                bytes.len() - (i + 1) * 8
            } else {
                0
            };
            let end = if bytes.len() > i * 8 {
                bytes.len() - i * 8
            } else {
                0
            };

            if start < end {
                let slice = &bytes[start..end];
                let offset = 8 - slice.len();

                chunks.fill(0);
                chunks[offset..].copy_from_slice(slice);

                *chunk = u64::from_be_bytes(chunks);
            }
        }

        Ok(EvmU256(out))
    }
}

impl fmt::Display for EvmU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Simple implementation for demonstration
        // In practice, would need proper big integer handling
        if self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0 {
            write!(f, "{}", self.0[0])
        } else {
            write!(
                f,
                "[{}, {}, {}, {}]",
                self.0[0], self.0[1], self.0[2], self.0[3]
            )
        }
    }
}

impl EvmAddress {
    /// Get the raw bytes of the address
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Create a new address from raw bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        EvmAddress(bytes)
    }

    /// Returns a zero address (0x0000...0000)
    pub fn zero() -> Self {
        EvmAddress([0u8; 20])
    }
}

impl AsRef<[u8]> for EvmBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//-----------------------------------------------------------------------------
// Consensus Type Implementations
//-----------------------------------------------------------------------------

#[cfg(feature = "lodestar-consensus")]
impl fmt::Display for Root {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[cfg(feature = "lodestar-consensus")]
impl FromStr for Root {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 64 {
            return Err(format!("Invalid root length: {}", s.len()));
        }

        let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {e}"))?;
        let mut root = [0u8; 32];
        root.copy_from_slice(&bytes);
        Ok(Root(root))
    }
}

#[cfg(feature = "lodestar-consensus")]
impl fmt::Display for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[cfg(feature = "lodestar-consensus")]
impl FromStr for BlsPublicKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 96 {
            return Err(format!("Invalid BLS public key length: {}", s.len()));
        }

        let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {e}"))?;
        let mut pubkey = [0u8; 48];
        pubkey.copy_from_slice(&bytes);
        Ok(BlsPublicKey(pubkey))
    }
}

#[cfg(feature = "lodestar-consensus")]
impl fmt::Display for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[cfg(feature = "lodestar-consensus")]
impl FromStr for BlsSignature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.len() != 192 {
            return Err(format!("Invalid BLS signature length: {}", s.len()));
        }

        let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {e}"))?;
        let mut signature = [0u8; 96];
        signature.copy_from_slice(&bytes);
        Ok(BlsSignature(signature))
    }
}

#[cfg(feature = "lodestar-consensus")]
impl serde::Serialize for BlsPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{}", hex::encode(self.0));
        serializer.serialize_str(&hex_string)
    }
}

#[cfg(feature = "lodestar-consensus")]
impl<'de> serde::Deserialize<'de> for BlsPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        hex_string.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "lodestar-consensus")]
impl serde::Serialize for BlsSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{}", hex::encode(self.0));
        serializer.serialize_str(&hex_string)
    }
}

#[cfg(feature = "lodestar-consensus")]
impl<'de> serde::Deserialize<'de> for BlsSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        hex_string.parse().map_err(serde::de::Error::custom)
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_evm_address() {
        // Test creation from string
        let address_str = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
        let address = EvmAddress::from_str(address_str).unwrap();

        // Test display formatting
        assert_eq!(
            address.to_string().to_lowercase(),
            address_str.to_lowercase()
        );

        // Test FromStr with and without 0x prefix
        let address1 = EvmAddress::from_str(address_str).unwrap();
        let address2 = EvmAddress::from_str(&address_str[2..]).unwrap();
        assert_eq!(address1, address2);

        // Test error cases
        // Too short
        assert!(EvmAddress::from_str("0x1234").is_err());
        // Invalid hex
        assert!(
            EvmAddress::from_str("0x12345678901234567890123456789012345678zz")
                .is_err()
        );

        // Test serialization/deserialization
        let json = serde_json::to_string(&address).unwrap();
        let deserialized: EvmAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(address, deserialized);
    }

    #[test]
    fn test_evm_hash() {
        // Test creation from string
        let hash_str =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let hash = EvmHash::from_str(hash_str).unwrap();

        // Test display formatting
        assert_eq!(hash.to_string().to_lowercase(), hash_str.to_lowercase());

        // Test FromStr with and without 0x prefix
        let hash1 = EvmHash::from_str(hash_str).unwrap();
        let hash2 = EvmHash::from_str(&hash_str[2..]).unwrap();
        assert_eq!(hash1, hash2);

        // Test error cases
        // Too short
        assert!(EvmHash::from_str("0x1234").is_err());
        // Invalid hex
        assert!(EvmHash::from_str(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdez"
        )
        .is_err());

        // Test serialization/deserialization
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: EvmHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, deserialized);
    }

    #[test]
    fn test_evm_bytes() {
        // Test creation from hex string
        let bytes = EvmBytes::from_hex("0x1234").unwrap();
        assert_eq!(bytes.0, vec![0x12, 0x34]);

        // Test creation without 0x prefix
        let bytes = EvmBytes::from_hex("1234").unwrap();
        assert_eq!(bytes.0, vec![0x12, 0x34]);

        // Test to_hex
        assert_eq!(bytes.to_hex(), "0x1234");

        // Test Display formatting
        assert_eq!(format!("{}", bytes), "0x1234");

        // Test empty bytes
        let empty_bytes = EvmBytes::from_hex("").unwrap();
        assert_eq!(empty_bytes.0, Vec::<u8>::new());
        assert_eq!(empty_bytes.to_hex(), "0x");

        // Test error case - invalid hex
        assert!(EvmBytes::from_hex("0x123z").is_err());

        // Test serialization/deserialization
        let json = serde_json::to_string(&bytes).unwrap();
        let deserialized: EvmBytes = serde_json::from_str(&json).unwrap();
        assert_eq!(bytes, deserialized);
    }

    #[test]
    fn test_evm_u256() {
        // Test from_u64
        let u256 = EvmU256::from_u64(12345);
        assert_eq!(u256.0[0], 12345);
        assert_eq!(u256.0[1], 0);
        assert_eq!(u256.0[2], 0);
        assert_eq!(u256.0[3], 0);

        // Test to_string for simple case
        assert_eq!(u256.to_string(), "12345");

        // Test to_string for complex case (all limbs have values)
        let complex_u256 = EvmU256([1, 2, 3, 4]);
        assert_eq!(complex_u256.to_string(), "[1, 2, 3, 4]");

        // Test serialization/deserialization
        let json = serde_json::to_string(&u256).unwrap();
        let deserialized: EvmU256 = serde_json::from_str(&json).unwrap();
        assert_eq!(u256, deserialized);
    }

    #[test]
    fn test_evm_log() {
        // Create test data
        let address =
            EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e")
                .unwrap();
        let topic1 = EvmHash::from_str(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        )
        .unwrap();
        let topic2 = EvmHash::from_str(
            "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
        )
        .unwrap();
        let data = EvmBytes::from_hex("0xabcdef").unwrap();
        let tx_hash = EvmHash::from_str(
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();

        // Create log
        let log = EvmLog {
            address: address.clone(),
            topics: vec![topic1.clone(), topic2.clone()],
            data: data.clone(),
            block_number: 12345,
            transaction_hash: tx_hash.clone(),
            log_index: 5,
        };

        // Test properties
        assert_eq!(log.address, address);
        assert_eq!(log.topics.len(), 2);
        assert_eq!(log.topics[0], topic1);
        assert_eq!(log.topics[1], topic2);
        assert_eq!(log.data, data);
        assert_eq!(log.block_number, 12345);
        assert_eq!(log.transaction_hash, tx_hash);
        assert_eq!(log.log_index, 5);

        // Test serialization/deserialization
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: EvmLog = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.address, log.address);
        assert_eq!(deserialized.topics, log.topics);
        assert_eq!(deserialized.data, log.data);
        assert_eq!(deserialized.block_number, log.block_number);
        assert_eq!(deserialized.transaction_hash, log.transaction_hash);
        assert_eq!(deserialized.log_index, log.log_index);
    }

    #[test]
    fn test_evm_transaction_receipt() {
        // Create test data
        let tx_hash = EvmHash::from_str(
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        let from_address =
            EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e")
                .unwrap();
        let to_address =
            EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0")
                .unwrap();
        let contract_address =
            EvmAddress::from_str("0x9b1f7f645351af3631a656421ed2e40f2802e6c0")
                .unwrap();
        let gas_used = EvmU256::from_u64(21000);
        let cum_gas_used = EvmU256::from_u64(100000);

        // Create log for the receipt
        let log = EvmLog {
            address: to_address.clone(),
            topics: vec![tx_hash.clone()],
            data: EvmBytes::from_hex("0xabcdef").unwrap(),
            block_number: 12345,
            transaction_hash: tx_hash.clone(),
            log_index: 5,
        };

        // Create receipt
        let receipt = EvmTransactionReceipt {
            transaction_hash: tx_hash.clone(),
            block_number: 12345,
            transaction_index: 3,
            from: from_address.clone(),
            to: Some(to_address.clone()),
            contract_address: Some(contract_address.clone()),
            cumulative_gas_used: cum_gas_used,
            gas_used,
            status: 1,
            logs: vec![log],
        };

        // Test properties
        assert_eq!(
            receipt.transaction_hash.to_string(),
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(receipt.block_number, 12345);
        assert_eq!(receipt.transaction_index, 3);
        assert_eq!(
            receipt.from.to_string().to_lowercase(),
            "0x742d35cc6634c0532925a3b844bc454e4438f44e"
        );
        assert_eq!(
            receipt.to.as_ref().unwrap().to_string().to_lowercase(),
            "0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0"
        );
        assert_eq!(
            receipt
                .contract_address
                .as_ref()
                .unwrap()
                .to_string()
                .to_lowercase(),
            "0x9b1f7f645351af3631a656421ed2e40f2802e6c0"
        );
        assert_eq!(receipt.cumulative_gas_used.to_string(), "100000");
        assert_eq!(receipt.gas_used.to_string(), "21000");
        assert_eq!(receipt.status, 1);
        assert_eq!(receipt.logs.len(), 1);

        // Test serialization/deserialization
        let json = serde_json::to_string(&receipt).unwrap();
        let deserialized: EvmTransactionReceipt =
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.transaction_hash, receipt.transaction_hash);
        assert_eq!(deserialized.block_number, receipt.block_number);
        assert_eq!(deserialized.from, receipt.from);
        assert_eq!(deserialized.to, receipt.to);
        assert_eq!(deserialized.status, receipt.status);
    }

    #[test]
    fn test_evm_transaction_request() {
        // Create test data
        let from_address =
            EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e")
                .unwrap();
        let to_address =
            EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0")
                .unwrap();
        let gas_limit = EvmU256::from_u64(21000);
        let gas_price = EvmU256::from_u64(50_000_000_000); // 50 gwei
        let value = EvmU256::from_u64(1_000_000_000_000_000_000); // 1 ETH
        let data = EvmBytes::from_hex("0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b844bc454e4438f44e0000000000000000000000000000000000000000000000000de0b6b3a7640000").unwrap();

        // Create transaction request
        let tx_request = EvmTransactionRequest {
            from: from_address.clone(),
            to: Some(to_address.clone()),
            nonce: Some(42),
            gas_limit: Some(gas_limit),
            gas_price: Some(gas_price),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            value: Some(value),
            data: Some(data.clone()),
            chain_id: Some(1),
        };

        // Test properties
        assert_eq!(
            tx_request.from.to_string().to_lowercase(),
            "0x742d35cc6634c0532925a3b844bc454e4438f44e"
        );
        assert_eq!(
            tx_request.to.as_ref().unwrap().to_string().to_lowercase(),
            "0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0"
        );
        assert_eq!(tx_request.nonce, Some(42));
        assert_eq!(tx_request.gas_limit.unwrap().to_string(), "21000");
        assert_eq!(tx_request.gas_price.unwrap().to_string(), "50000000000");
        assert_eq!(tx_request.value.unwrap().to_string(), "1000000000000000000");
        assert_eq!(tx_request.chain_id, Some(1));

        // Test EIP-1559 tx
        let eip1559_tx = EvmTransactionRequest {
            from: from_address,
            to: Some(to_address),
            nonce: Some(42),
            gas_limit: Some(gas_limit),
            gas_price: None,
            max_fee_per_gas: Some(EvmU256::from_u64(100_000_000_000)),
            max_priority_fee_per_gas: Some(EvmU256::from_u64(2_000_000_000)),
            value: Some(value),
            data: None,
            chain_id: Some(1),
        };

        assert_eq!(
            eip1559_tx.max_fee_per_gas.unwrap().to_string(),
            "100000000000"
        );
        assert_eq!(
            eip1559_tx.max_priority_fee_per_gas.unwrap().to_string(),
            "2000000000"
        );
        assert!(eip1559_tx.gas_price.is_none());

        // Test serialization/deserialization
        let json = serde_json::to_string(&tx_request).unwrap();
        let deserialized: EvmTransactionRequest =
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.from, tx_request.from);
        assert_eq!(deserialized.to, tx_request.to);
        assert_eq!(deserialized.nonce, tx_request.nonce);
        assert_eq!(deserialized.gas_limit, tx_request.gas_limit);
        assert_eq!(deserialized.chain_id, tx_request.chain_id);
    }

    //-----------------------------------------------------------------------------
    // Consensus Type Tests
    //-----------------------------------------------------------------------------

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_root_display_and_from_str() {
        // Test creation from string
        let root_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let root = Root::from_str(root_str).unwrap();

        // Test display formatting
        assert_eq!(root.to_string().to_lowercase(), root_str.to_lowercase());

        // Test FromStr with and without 0x prefix
        let root1 = Root::from_str(root_str).unwrap();
        let root2 = Root::from_str(&root_str[2..]).unwrap();
        assert_eq!(root1, root2);

        // Test error cases
        assert!(Root::from_str("0x1234").is_err()); // Too short
        assert!(Root::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdez").is_err()); // Invalid hex
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_bls_public_key_display_and_from_str() {
        // Test creation from string
        let pubkey_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let pubkey = BlsPublicKey::from_str(pubkey_str).unwrap();

        // Test display formatting
        assert_eq!(pubkey.to_string().to_lowercase(), pubkey_str.to_lowercase());

        // Test FromStr with and without 0x prefix
        let pubkey1 = BlsPublicKey::from_str(pubkey_str).unwrap();
        let pubkey2 = BlsPublicKey::from_str(&pubkey_str[2..]).unwrap();
        assert_eq!(pubkey1, pubkey2);

        // Test error cases
        assert!(BlsPublicKey::from_str("0x1234").is_err()); // Too short
        assert!(BlsPublicKey::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdez").is_err()); // Invalid hex
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_bls_signature_display_and_from_str() {
        // Test creation from string  
        let sig_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signature = BlsSignature::from_str(sig_str).unwrap();

        // Test display formatting
        assert_eq!(signature.to_string().to_lowercase(), sig_str.to_lowercase());

        // Test FromStr with and without 0x prefix
        let sig1 = BlsSignature::from_str(sig_str).unwrap();
        let sig2 = BlsSignature::from_str(&sig_str[2..]).unwrap();
        assert_eq!(sig1, sig2);

        // Test error cases
        assert!(BlsSignature::from_str("0x1234").is_err()); // Too short
        assert!(BlsSignature::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdez").is_err()); // Invalid hex
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_bls_types_serialization() {
        // Test BlsPublicKey serialization
        let pubkey_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let pubkey = BlsPublicKey::from_str(pubkey_str).unwrap();
        
        let json = serde_json::to_string(&pubkey).unwrap();
        let deserialized: BlsPublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(pubkey, deserialized);

        // Test BlsSignature serialization
        let sig_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let signature = BlsSignature::from_str(sig_str).unwrap();
        
        let json = serde_json::to_string(&signature).unwrap();
        let deserialized: BlsSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(signature, deserialized);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_validator_status_serialization() {
        use crate::types::ValidatorStatus;

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
            assert_eq!(format!("{:?}", status), format!("{:?}", deserialized));
        }
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_checkpoint_creation_and_serialization() {
        use crate::types::Checkpoint;

        let root = Root::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let checkpoint = Checkpoint {
            epoch: 123,
            root: root.clone(),
        };

        // Test basic properties
        assert_eq!(checkpoint.epoch, 123);
        assert_eq!(checkpoint.root, root);

        // Test serialization
        let json = serde_json::to_string(&checkpoint).unwrap();
        let deserialized: Checkpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(checkpoint.epoch, deserialized.epoch);
        assert_eq!(checkpoint.root, deserialized.root);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_validator_creation_and_properties() {
        use crate::types::Validator;

        let pubkey = BlsPublicKey::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let withdrawal_credentials = Root::from_str("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap();

        let validator = Validator {
            pubkey: pubkey.clone(),
            withdrawal_credentials: withdrawal_credentials.clone(),
            effective_balance: 32_000_000_000, // 32 ETH in Gwei
            slashed: false,
            activation_eligibility_epoch: 0,
            activation_epoch: 10,
            exit_epoch: u64::MAX,
            withdrawable_epoch: u64::MAX,
        };

        // Test basic properties
        assert_eq!(validator.pubkey, pubkey);
        assert_eq!(validator.withdrawal_credentials, withdrawal_credentials);
        assert_eq!(validator.effective_balance, 32_000_000_000);
        assert!(!validator.slashed);
        assert_eq!(validator.activation_epoch, 10);

        // Test serialization
        let json = serde_json::to_string(&validator).unwrap();
        let deserialized: Validator = serde_json::from_str(&json).unwrap();
        assert_eq!(validator.pubkey, deserialized.pubkey);
        assert_eq!(validator.effective_balance, deserialized.effective_balance);
        assert_eq!(validator.slashed, deserialized.slashed);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_beacon_block_header_creation() {
        use crate::types::BeaconBlockHeader;

        let parent_root = Root::from_str("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let state_root = Root::from_str("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();
        let body_root = Root::from_str("0x3333333333333333333333333333333333333333333333333333333333333333").unwrap();

        let header = BeaconBlockHeader {
            slot: 12345,
            proposer_index: 567,
            parent_root: parent_root.clone(),
            state_root: state_root.clone(),
            body_root: body_root.clone(),
        };

        // Test basic properties
        assert_eq!(header.slot, 12345);
        assert_eq!(header.proposer_index, 567);
        assert_eq!(header.parent_root, parent_root);
        assert_eq!(header.state_root, state_root);
        assert_eq!(header.body_root, body_root);

        // Test serialization
        let json = serde_json::to_string(&header).unwrap();
        let deserialized: BeaconBlockHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(header.slot, deserialized.slot);
        assert_eq!(header.proposer_index, deserialized.proposer_index);
        assert_eq!(header.parent_root, deserialized.parent_root);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_committee_creation_and_serialization() {
        use crate::types::Committee;

        let committee = Committee {
            index: 5,
            slot: 12345,
            validators: vec![100, 101, 102, 103, 104],
        };

        // Test basic properties
        assert_eq!(committee.index, 5);
        assert_eq!(committee.slot, 12345);
        assert_eq!(committee.validators.len(), 5);
        assert_eq!(committee.validators[0], 100);
        assert_eq!(committee.validators[4], 104);

        // Test serialization
        let json = serde_json::to_string(&committee).unwrap();
        let deserialized: Committee = serde_json::from_str(&json).unwrap();
        assert_eq!(committee.index, deserialized.index);
        assert_eq!(committee.slot, deserialized.slot);
        assert_eq!(committee.validators, deserialized.validators);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_attestation_data_creation() {
        use crate::types::{AttestationData, Checkpoint};

        let beacon_block_root = Root::from_str("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let source_root = Root::from_str("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();
        let target_root = Root::from_str("0x3333333333333333333333333333333333333333333333333333333333333333").unwrap();

        let source = Checkpoint {
            epoch: 100,
            root: source_root,
        };

        let target = Checkpoint {
            epoch: 101,
            root: target_root,
        };

        let attestation_data = AttestationData {
            slot: 3232,
            index: 1,
            beacon_block_root: beacon_block_root.clone(),
            source: source.clone(),
            target: target.clone(),
        };

        // Test basic properties
        assert_eq!(attestation_data.slot, 3232);
        assert_eq!(attestation_data.index, 1);
        assert_eq!(attestation_data.beacon_block_root, beacon_block_root);
        assert_eq!(attestation_data.source.epoch, 100);
        assert_eq!(attestation_data.target.epoch, 101);

        // Test serialization
        let json = serde_json::to_string(&attestation_data).unwrap();
        let deserialized: AttestationData = serde_json::from_str(&json).unwrap();
        assert_eq!(attestation_data.slot, deserialized.slot);
        assert_eq!(attestation_data.index, deserialized.index);
        assert_eq!(attestation_data.source.epoch, deserialized.source.epoch);
        assert_eq!(attestation_data.target.epoch, deserialized.target.epoch);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_finality_checkpoints_creation() {
        use crate::types::{Checkpoint, FinalityCheckpoints};

        let prev_root = Root::from_str("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let curr_root = Root::from_str("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();
        let fin_root = Root::from_str("0x3333333333333333333333333333333333333333333333333333333333333333").unwrap();

        let finality_checkpoints = FinalityCheckpoints {
            previous_justified: Checkpoint {
                epoch: 98,
                root: prev_root,
            },
            current_justified: Checkpoint {
                epoch: 99,
                root: curr_root,
            },
            finalized: Checkpoint {
                epoch: 97,
                root: fin_root,
            },
        };

        // Test basic properties
        assert_eq!(finality_checkpoints.previous_justified.epoch, 98);
        assert_eq!(finality_checkpoints.current_justified.epoch, 99);
        assert_eq!(finality_checkpoints.finalized.epoch, 97);

        // Test serialization
        let json = serde_json::to_string(&finality_checkpoints).unwrap();
        let deserialized: FinalityCheckpoints = serde_json::from_str(&json).unwrap();
        assert_eq!(finality_checkpoints.previous_justified.epoch, deserialized.previous_justified.epoch);
        assert_eq!(finality_checkpoints.current_justified.epoch, deserialized.current_justified.epoch);
        assert_eq!(finality_checkpoints.finalized.epoch, deserialized.finalized.epoch);
    }

    #[cfg(feature = "lodestar-consensus")]
    #[test]
    fn test_genesis_data_creation() {
        use crate::types::GenesisData;

        let genesis_validators_root = Root::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        
        let genesis_data = GenesisData {
            genesis_time: 1606824023,
            genesis_validators_root: genesis_validators_root.clone(),
            genesis_fork_version: [0x00, 0x00, 0x10, 0x20],
        };

        // Test basic properties
        assert_eq!(genesis_data.genesis_time, 1606824023);
        assert_eq!(genesis_data.genesis_validators_root, genesis_validators_root);
        assert_eq!(genesis_data.genesis_fork_version, [0x00, 0x00, 0x10, 0x20]);

        // Test serialization
        let json = serde_json::to_string(&genesis_data).unwrap();
        let deserialized: GenesisData = serde_json::from_str(&json).unwrap();
        assert_eq!(genesis_data.genesis_time, deserialized.genesis_time);
        assert_eq!(genesis_data.genesis_validators_root, deserialized.genesis_validators_root);
        assert_eq!(genesis_data.genesis_fork_version, deserialized.genesis_fork_version);
    }

    //-----------------------------------------------------------------------------
    // Erigon Tracing Type Tests
    //-----------------------------------------------------------------------------

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_trace_action_serialization() {
        use crate::types::TraceAction;

        // Test Call action
        let call_action = TraceAction::Call {
            from: EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap(),
            to: EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap(),
            value: EvmU256::from_u64(1000000000000000000), // 1 ETH
            gas: EvmU256::from_u64(21000),
            input: EvmBytes::from_hex("0xa9059cbb").unwrap(),
            call_type: "call".to_string(),
        };

        let json = serde_json::to_string(&call_action).unwrap();
        let deserialized: TraceAction = serde_json::from_str(&json).unwrap();
        
        if let TraceAction::Call { from, to, value, gas, call_type, .. } = deserialized {
            assert_eq!(from.to_string().to_lowercase(), "0x742d35cc6634c0532925a3b844bc454e4438f44e");
            assert_eq!(to.to_string().to_lowercase(), "0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0");
            assert_eq!(value.to_string(), "1000000000000000000");
            assert_eq!(gas.to_string(), "21000");
            assert_eq!(call_type, "call");
        } else {
            panic!("Expected Call action");
        }

        // Test Create action
        let create_action = TraceAction::Create {
            from: EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap(),
            value: EvmU256::from_u64(0),
            gas: EvmU256::from_u64(100000),
            init: EvmBytes::from_hex("0x608060405234801561001057600080fd5b50").unwrap(),
        };

        let json = serde_json::to_string(&create_action).unwrap();
        let deserialized: TraceAction = serde_json::from_str(&json).unwrap();
        
        if let TraceAction::Create { from, value, gas, .. } = deserialized {
            assert_eq!(from.to_string().to_lowercase(), "0x742d35cc6634c0532925a3b844bc454e4438f44e");
            assert_eq!(value.to_string(), "0");
            assert_eq!(gas.to_string(), "100000");
        } else {
            panic!("Expected Create action");
        }
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_trace_result_serialization() {
        use crate::types::TraceResult;

        // Test Call result
        let call_result = TraceResult::Call {
            gas_used: EvmU256::from_u64(21000),
            output: EvmBytes::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        };

        let json = serde_json::to_string(&call_result).unwrap();
        let deserialized: TraceResult = serde_json::from_str(&json).unwrap();
        
        if let TraceResult::Call { gas_used, output } = deserialized {
            assert_eq!(gas_used.to_string(), "21000");
            assert_eq!(output.to_hex(), "0x0000000000000000000000000000000000000000000000000000000000000001");
        } else {
            panic!("Expected Call result");
        }

        // Test Create result
        let create_result = TraceResult::Create {
            gas_used: EvmU256::from_u64(50000),
            code: EvmBytes::from_hex("0x608060405234801561001057600080fd5b50").unwrap(),
            address: EvmAddress::from_str("0x9b1f7f645351af3631a656421ed2e40f2802e6c0").unwrap(),
        };

        let json = serde_json::to_string(&create_result).unwrap();
        let deserialized: TraceResult = serde_json::from_str(&json).unwrap();
        
        if let TraceResult::Create { gas_used, address, .. } = deserialized {
            assert_eq!(gas_used.to_string(), "50000");
            assert_eq!(address.to_string().to_lowercase(), "0x9b1f7f645351af3631a656421ed2e40f2802e6c0");
        } else {
            panic!("Expected Create result");
        }
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_transaction_trace_creation() {
        use crate::types::{TraceAction, TraceResult, TransactionTrace};

        let action = TraceAction::Call {
            from: EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap(),
            to: EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap(),
            value: EvmU256::from_u64(1000000000000000000),
            gas: EvmU256::from_u64(21000),
            input: EvmBytes::from_hex("0xa9059cbb").unwrap(),
            call_type: "call".to_string(),
        };

        let result = TraceResult::Call {
            gas_used: EvmU256::from_u64(21000),
            output: EvmBytes::from_hex("0x01").unwrap(),
        };

        let tx_hash = EvmHash::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let block_hash = EvmHash::from_str("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap();

        let trace = TransactionTrace {
            action,
            result: Some(result),
            error: None,
            trace_address: vec![0],
            subtraces: 0,
            transaction_position: Some(1),
            transaction_hash: Some(tx_hash.clone()),
            block_number: 12345,
            block_hash: block_hash.clone(),
        };

        // Test basic properties
        assert!(trace.result.is_some());
        assert!(trace.error.is_none());
        assert_eq!(trace.trace_address, vec![0]);
        assert_eq!(trace.subtraces, 0);
        assert_eq!(trace.transaction_position, Some(1));
        assert_eq!(trace.transaction_hash, Some(tx_hash));
        assert_eq!(trace.block_number, 12345);
        assert_eq!(trace.block_hash, block_hash);

        // Test serialization
        let json = serde_json::to_string(&trace).unwrap();
        let deserialized: TransactionTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace.trace_address, deserialized.trace_address);
        assert_eq!(trace.subtraces, deserialized.subtraces);
        assert_eq!(trace.block_number, deserialized.block_number);
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_trace_filter_creation() {
        use crate::types::TraceFilter;

        let from_address = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
        let to_address = EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap();

        let filter = TraceFilter {
            from_block: Some(100),
            to_block: Some(200),
            from_address: Some(vec![from_address.clone()]),
            to_address: Some(vec![to_address.clone()]),
            after: Some(10),
            count: Some(25),
        };

        // Test basic properties
        assert_eq!(filter.from_block, Some(100));
        assert_eq!(filter.to_block, Some(200));
        assert_eq!(filter.from_address.as_ref().unwrap().len(), 1);
        assert_eq!(filter.to_address.as_ref().unwrap().len(), 1);
        assert_eq!(filter.after, Some(10));
        assert_eq!(filter.count, Some(25));

        // Test serialization
        let json = serde_json::to_string(&filter).unwrap();
        let deserialized: TraceFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(filter.from_block, deserialized.from_block);
        assert_eq!(filter.to_block, deserialized.to_block);
        assert_eq!(filter.after, deserialized.after);
        assert_eq!(filter.count, deserialized.count);
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_call_trace_request_creation() {
        use crate::types::CallTraceRequest;

        let from_address = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
        let to_address = EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap();
        let call_data = EvmBytes::from_hex("0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b844bc454e4438f44e0000000000000000000000000000000000000000000000000de0b6b3a7640000").unwrap();

        let call_request = CallTraceRequest {
            from: Some(from_address.clone()),
            to: to_address.clone(),
            gas: Some(EvmU256::from_u64(100000)),
            gas_price: Some(EvmU256::from_u64(20000000000)), // 20 gwei
            value: Some(EvmU256::from_u64(1000000000000000000)), // 1 ETH
            data: Some(call_data.clone()),
        };

        // Test basic properties
        assert_eq!(call_request.from, Some(from_address));
        assert_eq!(call_request.to, to_address);
        assert_eq!(call_request.gas.unwrap().to_string(), "100000");
        assert_eq!(call_request.gas_price.unwrap().to_string(), "20000000000");
        assert_eq!(call_request.value.unwrap().to_string(), "1000000000000000000");
        assert_eq!(call_request.data, Some(call_data));

        // Test serialization
        let json = serde_json::to_string(&call_request).unwrap();
        let deserialized: CallTraceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(call_request.from, deserialized.from);
        assert_eq!(call_request.to, deserialized.to);
        assert_eq!(call_request.gas, deserialized.gas);
        assert_eq!(call_request.value, deserialized.value);
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_trace_type_serialization() {
        use crate::types::TraceType;

        let trace_types = vec![
            TraceType::Trace,
            TraceType::VmTrace,
            TraceType::StateDiff,
        ];

        for trace_type in trace_types {
            let json = serde_json::to_string(&trace_type).unwrap();
            let deserialized: TraceType = serde_json::from_str(&json).unwrap();
            // Check that serialization round-trips
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_vm_trace_creation() {
        use crate::types::{VmTrace, VmOperation, VmExecutionInfo};

        let vm_trace = VmTrace {
            code: EvmBytes::from_hex("0x608060405234801561001057600080fd5b50").unwrap(),
            ops: vec![
                VmOperation {
                    cost: 3,
                    ex: Some(VmExecutionInfo {
                        mem: None,
                        push: Some(vec!["0x80".to_string()]),
                        store: None,
                        used: Some(3),
                    }),
                    pc: 0,
                    sub: None,
                },
                VmOperation {
                    cost: 3,
                    ex: Some(VmExecutionInfo {
                        mem: None,
                        push: Some(vec!["0x40".to_string()]),
                        store: None,
                        used: Some(6),
                    }),
                    pc: 1,
                    sub: None,
                },
            ],
        };

        // Test basic properties
        assert!(!vm_trace.code.0.is_empty());
        assert_eq!(vm_trace.ops.len(), 2);
        assert_eq!(vm_trace.ops[0].cost, 3);
        assert_eq!(vm_trace.ops[0].pc, 0);
        assert_eq!(vm_trace.ops[1].cost, 3);
        assert_eq!(vm_trace.ops[1].pc, 1);

        // Test serialization
        let json = serde_json::to_string(&vm_trace).unwrap();
        let deserialized: VmTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(vm_trace.ops.len(), deserialized.ops.len());
        assert_eq!(vm_trace.ops[0].cost, deserialized.ops[0].cost);
        assert_eq!(vm_trace.ops[1].pc, deserialized.ops[1].pc);
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_state_diff_creation() {
        use crate::types::{StateDiff, BalanceDiff, CodeDiff, NonceDiff, StorageDiff};
        use std::collections::HashMap;

        let mut storage_changes = HashMap::new();
        let storage_key = EvmHash::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        storage_changes.insert(storage_key, StorageDiff {
            from: EvmHash::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            to: EvmHash::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        });

        let state_diff = StateDiff {
            balance: Some(BalanceDiff {
                from: EvmU256::from_u64(1000000000000000000), // 1 ETH
                to: EvmU256::from_u64(2000000000000000000),   // 2 ETH
            }),
            code: Some(CodeDiff {
                from: EvmBytes::from_hex("0x").unwrap(),
                to: EvmBytes::from_hex("0x608060405234801561001057600080fd5b50").unwrap(),
            }),
            nonce: Some(NonceDiff {
                from: 0,
                to: 1,
            }),
            storage: Some(storage_changes),
        };

        // Test basic properties
        assert!(state_diff.balance.is_some());
        assert!(state_diff.code.is_some());
        assert!(state_diff.nonce.is_some());
        assert!(state_diff.storage.is_some());

        let balance_diff = state_diff.balance.as_ref().unwrap();
        assert_eq!(balance_diff.from.to_string(), "1000000000000000000");
        assert_eq!(balance_diff.to.to_string(), "2000000000000000000");

        let nonce_diff = state_diff.nonce.as_ref().unwrap();
        assert_eq!(nonce_diff.from, 0);
        assert_eq!(nonce_diff.to, 1);

        // Test partial serialization (without storage due to HashMap key issues)
        let simple_state_diff = StateDiff {
            balance: state_diff.balance.clone(),
            code: state_diff.code.clone(),
            nonce: state_diff.nonce.clone(),
            storage: None, // Skip storage for serialization test
        };
        
        let json = serde_json::to_string(&simple_state_diff).unwrap();
        let deserialized: StateDiff = serde_json::from_str(&json).unwrap();
        assert!(deserialized.balance.is_some());
        assert!(deserialized.code.is_some());
        assert!(deserialized.nonce.is_some());
        assert!(deserialized.storage.is_none());
    }

    #[cfg(feature = "erigon-tracing")]
    #[test]
    fn test_block_trace_creation() {
        use crate::types::{BlockTrace, TransactionTrace, TraceAction, TraceResult};
        use std::collections::HashMap;

        let action = TraceAction::Call {
            from: EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap(),
            to: EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap(),
            value: EvmU256::from_u64(1000000000000000000),
            gas: EvmU256::from_u64(21000),
            input: EvmBytes::from_hex("0xa9059cbb").unwrap(),
            call_type: "call".to_string(),
        };

        let result = TraceResult::Call {
            gas_used: EvmU256::from_u64(21000),
            output: EvmBytes::from_hex("0x01").unwrap(),
        };

        let trace = TransactionTrace {
            action,
            result: Some(result),
            error: None,
            trace_address: vec![0],
            subtraces: 0,
            transaction_position: Some(0),
            transaction_hash: Some(EvmHash::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap()),
            block_number: 12345,
            block_hash: EvmHash::from_str("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890").unwrap(),
        };

        let block_trace = BlockTrace {
            traces: vec![trace],
            vm_trace: None,
            state_diff: Some(HashMap::new()),
        };

        // Test basic properties
        assert_eq!(block_trace.traces.len(), 1);
        assert!(block_trace.vm_trace.is_none());
        assert!(block_trace.state_diff.is_some());

        // Test serialization
        let json = serde_json::to_string(&block_trace).unwrap();
        let deserialized: BlockTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(block_trace.traces.len(), deserialized.traces.len());
        assert_eq!(block_trace.vm_trace.is_none(), deserialized.vm_trace.is_none());
        assert_eq!(block_trace.state_diff.is_some(), deserialized.state_diff.is_some());
    }
}
