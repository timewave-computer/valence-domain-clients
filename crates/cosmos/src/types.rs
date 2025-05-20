//-----------------------------------------------------------------------------
// Cosmos Type Definitions
//-----------------------------------------------------------------------------

//! Type definitions specific to Cosmos ecosystem interactions.
//!
//! This module encapsulates complex or third-party types into simple, controlled
//! structures that provide a consistent interface across the codebase.

use serde::{Deserialize, Serialize};
use valence_core::types::GenericAddress;

// Add CosmosAddress as an alias for GenericAddress to maintain backward compatibility
/// Cosmos blockchain address - alias for GenericAddress for consistency
pub type CosmosAddress = GenericAddress;

//-----------------------------------------------------------------------------
// Basic Types
//-----------------------------------------------------------------------------

/// Representation of a Cosmos coin with denomination and amount.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosCoin {
    pub denom: String,
    pub amount: u128,
}

/// Fee structure for Cosmos transactions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosFee {
    pub amount: Vec<CosmosCoin>,
    pub gas_limit: u64,
    pub payer: Option<String>,
    pub granter: Option<String>,
}

/// Account information for a Cosmos address.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosAccount {
    pub address: GenericAddress,
    pub pub_key: Option<Vec<u8>>,
    pub account_number: u64,
    pub sequence: u64,
}

/// Base account structure containing account details.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosBaseAccount {
    pub address: String,
    pub pub_key: Option<CosmosAccountPubKey>,
    pub account_number: u64,
    pub sequence: u64,
}

/// Module account with specialized permissions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosModuleAccount {
    pub base_account: Option<CosmosBaseAccount>,
    pub name: String,
    pub permissions: Vec<String>,
}

/// Public key variants for Cosmos accounts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CosmosAccountPubKey {
    Secp256k1(Vec<u8>),
    Ed25519(Vec<u8>),
    Unsupported { type_url: String, value: Vec<u8> },
}

//-----------------------------------------------------------------------------
// Consensus and Block Types
//-----------------------------------------------------------------------------

/// Consensus version information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosConsensusVersion {
    pub block: u64,
    pub app: u64,
}

/// Block header information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosHeader {
    pub version: Option<CosmosConsensusVersion>,
    pub chain_id: String,
    pub height: i64,
    pub time: Option<String>, // ISO8601 timestamp string
    pub app_hash: Option<String>,
    pub proposer_address: Option<GenericAddress>,
}

/// Gas consumption information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosGasInfo {
    pub gas_wanted: u64,
    pub gas_used: u64,
}

/// Transaction simulation result.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosSimulateResult {
    pub data: Option<Vec<u8>>,
    pub log: String,
    pub events: Vec<valence_core::transaction::Event>,
}

/// Response from a transaction simulation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosSimulateResponse {
    pub simulate_result: CosmosSimulateResult,
    pub gas_info: CosmosGasInfo,
}

/// Ed25519 public key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosEd25519PublicKey {
    pub value: String, // Hex encoded string of the public key bytes
}

/// Types of public keys supported in Cosmos.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CosmosPublicKeyType {
    Ed25519(String),   // Hex encoded public key bytes
    Secp256k1(String), // Hex encoded public key bytes
    Sr25519(String),   // Hex encoded public key bytes
}

/// Validator update information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosValidatorUpdate {
    pub public_key: CosmosPublicKeyType, // Simplified public key representation
    pub power: i64,                      // Voting power
}

/// Block parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosBlockParams {
    pub max_bytes: i64,
    pub max_gas: i64,
}

/// Evidence parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosEvidenceParams {
    pub max_age_num_blocks: i64,
    pub max_age_duration: Option<String>,
    pub max_bytes: i64,
}

/// Validator parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosValidatorParams {
    pub pub_key_types: Vec<String>,
}

/// Version parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosVersionParams {
    pub app_version: u64,
}

/// Consensus parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosConsensusParams {
    pub block: Option<CosmosBlockParams>,
    pub evidence: Option<CosmosEvidenceParams>,
    pub validator: Option<CosmosValidatorParams>,
    pub version: Option<CosmosVersionParams>,
}

/// Block results information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmosBlockResults {
    pub height: u64,
    pub txs_results: Option<Vec<valence_core::transaction::TransactionResponse>>,
    pub begin_block_events: Option<Vec<valence_core::transaction::Event>>,
    pub end_block_events: Option<Vec<valence_core::transaction::Event>>,
    pub validator_updates: Vec<CosmosValidatorUpdate>,
    pub consensus_param_updates: Option<CosmosConsensusParams>,
}

//-----------------------------------------------------------------------------
// Transaction Broadcasting Types
//-----------------------------------------------------------------------------

/// Broadcast modes for Cosmos transactions.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CosmosBroadcastMode {
    Async,
    #[default]
    Sync,
    Block,
}

/// Fee settings for transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FeeSetting {
    /// Automatic gas estimation with specified adjustment
    Auto { gas_adjustment: f64 },
    /// Fixed gas and fee amount
    Fixed {
        fee_amount: Vec<CosmosCoin>,
        gas_limit: u64,
    },
    /// Custom gas settings
    Custom { gas_limit: u64, price: CosmosCoin },
}

impl Default for FeeSetting {
    fn default() -> Self {
        FeeSetting::Auto {
            gas_adjustment: 1.2,
        }
    }
}

/// Broadcast transaction request parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CosmosBroadcastTxRequest {
    pub tx_bytes: Vec<u8>,
    pub mode: CosmosBroadcastMode,
    pub fee_setting: FeeSetting,
    pub memo: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_cosmos_broadcast_mode_default() {
        let default_mode = CosmosBroadcastMode::default();
        assert_eq!(default_mode, CosmosBroadcastMode::Sync);
    }

    #[test]
    fn test_fee_setting_default() {
        let default_fee = FeeSetting::default();
        match default_fee {
            FeeSetting::Auto { gas_adjustment } => {
                assert_eq!(gas_adjustment, 1.2);
            }
            _ => panic!("Expected Auto variant as default"),
        }
    }

    #[test]
    fn test_cosmos_coin_serialization() {
        let coin = CosmosCoin {
            denom: "uatom".to_string(),
            amount: 1000000,
        };

        let json = serde_json::to_string(&coin).unwrap();
        let deserialized: CosmosCoin = serde_json::from_str(&json).unwrap();

        assert_eq!(coin, deserialized);
        assert_eq!(coin.denom, "uatom");
        assert_eq!(coin.amount, 1000000);
    }

    #[test]
    fn test_cosmos_fee_serialization() {
        let fee = CosmosFee {
            amount: vec![CosmosCoin {
                denom: "uatom".to_string(),
                amount: 1000,
            }],
            gas_limit: 200000,
            payer: Some("cosmos1payer".to_string()),
            granter: None,
        };

        let json = serde_json::to_string(&fee).unwrap();
        let deserialized: CosmosFee = serde_json::from_str(&json).unwrap();

        assert_eq!(fee, deserialized);
        assert_eq!(fee.amount[0].denom, "uatom");
        assert_eq!(fee.amount[0].amount, 1000);
        assert_eq!(fee.gas_limit, 200000);
        assert_eq!(fee.payer, Some("cosmos1payer".to_string()));
        assert_eq!(fee.granter, None);
    }

    #[test]
    fn test_cosmos_account_pubkey_variants() {
        // Test Secp256k1 variant
        let secp_key = CosmosAccountPubKey::Secp256k1(vec![1, 2, 3, 4]);
        let json = serde_json::to_string(&secp_key).unwrap();
        let deserialized: CosmosAccountPubKey = serde_json::from_str(&json).unwrap();
        match deserialized {
            CosmosAccountPubKey::Secp256k1(bytes) => {
                assert_eq!(bytes, vec![1, 2, 3, 4])
            }
            _ => panic!("Expected Secp256k1 variant"),
        }

        // Test Ed25519 variant
        let ed_key = CosmosAccountPubKey::Ed25519(vec![5, 6, 7, 8]);
        let json = serde_json::to_string(&ed_key).unwrap();
        let deserialized: CosmosAccountPubKey = serde_json::from_str(&json).unwrap();
        match deserialized {
            CosmosAccountPubKey::Ed25519(bytes) => {
                assert_eq!(bytes, vec![5, 6, 7, 8])
            }
            _ => panic!("Expected Ed25519 variant"),
        }

        // Test Unsupported variant
        let unsupported_key = CosmosAccountPubKey::Unsupported {
            type_url: "/custom.PublicKey".to_string(),
            value: vec![9, 10, 11, 12],
        };
        let json = serde_json::to_string(&unsupported_key).unwrap();
        let deserialized: CosmosAccountPubKey = serde_json::from_str(&json).unwrap();
        match deserialized {
            CosmosAccountPubKey::Unsupported { type_url, value } => {
                assert_eq!(type_url, "/custom.PublicKey");
                assert_eq!(value, vec![9, 10, 11, 12]);
            }
            _ => panic!("Expected Unsupported variant"),
        }
    }

    #[test]
    fn test_cosmos_broadcast_tx_request() {
        let request = CosmosBroadcastTxRequest {
            tx_bytes: vec![1, 2, 3, 4, 5],
            mode: CosmosBroadcastMode::Block,
            fee_setting: FeeSetting::Fixed {
                fee_amount: vec![CosmosCoin {
                    denom: "uatom".to_string(),
                    amount: 5000,
                }],
                gas_limit: 300000,
            },
            memo: Some("test transaction".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CosmosBroadcastTxRequest =
            serde_json::from_str(&json).unwrap();

        assert_eq!(request, deserialized);
        assert_eq!(request.tx_bytes, vec![1, 2, 3, 4, 5]);
        assert_eq!(request.mode, CosmosBroadcastMode::Block);
        assert_eq!(request.memo, Some("test transaction".to_string()));

        match request.fee_setting {
            FeeSetting::Fixed {
                fee_amount,
                gas_limit,
            } => {
                assert_eq!(fee_amount[0].denom, "uatom");
                assert_eq!(fee_amount[0].amount, 5000);
                assert_eq!(gas_limit, 300000);
            }
            _ => panic!("Expected Fixed fee setting"),
        }
    }
}
