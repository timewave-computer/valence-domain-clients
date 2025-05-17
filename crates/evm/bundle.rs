//-----------------------------------------------------------------------------
// Flashbots Bundle Support
//-----------------------------------------------------------------------------

//! This module provides support for Flashbots bundles on Ethereum.
//! 
//! It implements both the traditional eth_sendBundle method and the newer mev_sendBundle
//! method used by MEV-Share, allowing for more flexible bundle configurations including
//! privacy settings and multi-builder support.

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::error::ClientError;

//-----------------------------------------------------------------------------
// Constants
//-----------------------------------------------------------------------------

/// Flashbots RPC endpoint for retail users
pub const FLASHBOTS_RPC_URL: &str = "https://rpc.flashbots.net";

/// Flashbots relay endpoint for advanced users (bundles, etc.)
pub const FLASHBOTS_RELAY_URL: &str = "https://relay.flashbots.net";

//-----------------------------------------------------------------------------
// Flashbots Bundle Types
//-----------------------------------------------------------------------------

/// Enum representing different types of privacy hints for MEV-Share bundles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyHint {
    /// Share data sent to the smart contract
    Calldata,
    /// Share logs emitted by executing the transaction
    Logs,
    /// Share specific subset of logs related to defi swaps
    DefaultLogs,
    /// Share the 4-byte function identifier
    FunctionSelector,
    /// Share the address of the recipient
    ContractAddress,
    /// Share the transaction hash
    Hash,
    /// Share individual transaction hashes in the bundle
    TxHash,
    /// Share all fields of individual transactions except signature
    Full,
}

/// Privacy configuration for MEV-Share bundles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Hints to share with searchers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<PrivacyHint>>,
    /// Builders to send the bundle to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builders: Option<Vec<String>>,
}

/// Inclusion parameters for bundles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInclusion {
    /// Target block number (hex-encoded)
    pub block: String,
    /// Maximum block number (hex-encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block: Option<String>,
}

/// Refund configuration for a transaction in a bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundConfig {
    /// Index of the transaction in the bundle body
    pub body_idx: usize,
    /// Percentage of MEV to refund
    pub percent: u8,
}

/// Validity configuration for MEV-Share bundles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityConfig {
    /// Refund parameters for specific transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund: Option<Vec<RefundConfig>>,
    /// Global refund configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_config: Option<Vec<RefundAddress>>,
}

/// Address and percentage for refund
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundAddress {
    /// Address to refund
    pub address: String,
    /// Percentage of MEV to refund
    pub percent: u8,
}

/// Metadata for MEV-Share bundles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Origin ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,
}

/// Represents an item in a bundle body - either a transaction, hash, or nested bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BundleItem {
    /// A transaction hash reference
    Hash {
        /// Transaction hash
        hash: String,
    },
    /// A raw transaction with revert setting
    Transaction {
        /// Raw signed transaction (hex-encoded)
        tx: String,
        /// Whether the transaction can revert
        can_revert: bool,
    },
    /// A nested bundle
    Bundle {
        /// Nested bundle
        bundle: Box<MevSendBundleParams>,
    },
}

/// Parameters for traditional eth_sendBundle method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthSendBundleParams {
    /// Transactions to include in the bundle (hex-encoded raw transactions)
    pub txs: Vec<String>,
    /// Target block number
    pub block_number: String,
    /// Minimum timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_timestamp: Option<u64>,
    /// Maximum timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timestamp: Option<u64>,
    /// Reversion check (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverting_tx_hashes: Option<Vec<String>>,
}

/// Parameters for the newer mev_sendBundle method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevSendBundleParams {
    /// Protocol version
    pub version: String,
    /// Inclusion parameters
    pub inclusion: BundleInclusion,
    /// Bundle body containing transactions or references
    pub body: Vec<BundleItem>,
    /// Validity parameters (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity: Option<ValidityConfig>,
    /// Privacy configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<PrivacyConfig>,
    /// Metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BundleMetadata>,
}

/// Response from eth_sendBundle or mev_sendBundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResponse {
    /// Bundle hash
    pub bundle_hash: String,
}

//-----------------------------------------------------------------------------
// Flashbots Bundle Trait
//-----------------------------------------------------------------------------

/// Trait for Flashbots bundle operations
#[async_trait]
pub trait FlashbotsBundle {
    /// Sends a bundle to Flashbots using the legacy eth_sendBundle method
    async fn send_eth_bundle(&self, params: EthSendBundleParams) -> Result<BundleResponse, ClientError>;
    
    /// Sends a bundle to Flashbots using the newer mev_sendBundle method for MEV-Share
    async fn send_mev_bundle(&self, params: MevSendBundleParams) -> Result<BundleResponse, ClientError>;
    
    /// Simulates a bundle execution using eth_callBundle
    async fn simulate_bundle(&self, params: EthSendBundleParams) -> Result<HashMap<String, serde_json::Value>, ClientError>;
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Creates a basic eth_sendBundle parameters object
pub fn create_eth_bundle(
    transactions: Vec<String>,
    target_block: u64,
    reverting_hashes: Option<Vec<String>>,
) -> EthSendBundleParams {
    EthSendBundleParams {
        txs: transactions,
        block_number: format!("0x{target_block:x}"),
        min_timestamp: None,
        max_timestamp: None,
        reverting_tx_hashes: reverting_hashes,
    }
}

/// Creates a mev_sendBundle parameters object for MEV-Share
pub fn create_mev_bundle(
    transactions: Vec<(String, bool)>,
    target_block: u64,
    max_block: Option<u64>,
    hints: Option<Vec<PrivacyHint>>,
    builders: Option<Vec<String>>,
) -> MevSendBundleParams {
    let body = transactions
        .into_iter()
        .map(|(tx, can_revert)| BundleItem::Transaction { tx, can_revert })
        .collect();
    
    let max_block_str = max_block.map(|block| format!("0x{block:x}"));
    
    MevSendBundleParams {
        version: "v0.1".to_string(),
        inclusion: BundleInclusion {
            block: format!("0x{target_block:x}"),
            max_block: max_block_str,
        },
        body,
        validity: None,
        privacy: if hints.is_some() || builders.is_some() {
            Some(PrivacyConfig {
                hints,
                builders,
            })
        } else {
            None
        },
        metadata: None,
    }
}

/// Create a bundle with MEV sharing and refunds
pub fn create_mev_share_bundle(
    transactions: Vec<(String, bool)>,
    target_block: u64,
    max_block: Option<u64>,
    hints: Option<Vec<PrivacyHint>>,
    builders: Option<Vec<String>>,
    refund_addresses: Vec<(String, u8)>,
) -> MevSendBundleParams {
    let mut bundle = create_mev_bundle(
        transactions,
        target_block,
        max_block,
        hints,
        builders,
    );
    
    // Add refund configuration
    if !refund_addresses.is_empty() {
        let refund_config = refund_addresses
            .into_iter()
            .map(|(address, percent)| RefundAddress { address, percent })
            .collect();
        
        bundle.validity = Some(ValidityConfig {
            refund: None,
            refund_config: Some(refund_config),
        });
    }
    
    bundle
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_eth_bundle() {
        let transactions = vec![
            "0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(),
            "0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702e2f5c4d257d008025a0cf15a4ceeea92d11a4778a221bf9140844b97c9faa07302aea66e3118cced38ca013c5c55c958671a0f6868c0fa8f11e942fe646a5205ad1fb7fbc33a3ca5caed9".to_string(),
        ];
        
        let bundle = create_eth_bundle(transactions.clone(), 12345678, None);
        
        assert_eq!(bundle.txs, transactions);
        assert_eq!(bundle.block_number, "0xbc614e");
        assert!(bundle.reverting_tx_hashes.is_none());
    }
    
    #[test]
    fn test_create_mev_bundle() {
        let transactions = vec![
            ("0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(), false),
            ("0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702e2f5c4d257d008025a0cf15a4ceeea92d11a4778a221bf9140844b97c9faa07302aea66e3118cced38ca013c5c55c958671a0f6868c0fa8f11e942fe646a5205ad1fb7fbc33a3ca5caed9".to_string(), true),
        ];
        
        let hints = Some(vec![PrivacyHint::TxHash, PrivacyHint::Calldata]);
        let builders = Some(vec!["flashbots".to_string(), "builder0x69".to_string()]);
        
        let bundle = create_mev_bundle(
            transactions.clone(),
            12345678,
            Some(12345680),
            hints.clone(),
            builders.clone(),
        );
        
        assert_eq!(bundle.version, "v0.1");
        assert_eq!(bundle.inclusion.block, "0xbc614e");
        assert_eq!(bundle.inclusion.max_block, Some("0xbc6150".to_string()));
        assert_eq!(bundle.body.len(), 2);
        
        if let BundleItem::Transaction { tx, can_revert } = &bundle.body[0] {
            assert_eq!(tx, &transactions[0].0);
            assert_eq!(*can_revert, transactions[0].1);
        } else {
            panic!("Expected Transaction bundle item");
        }
        
        if let BundleItem::Transaction { tx, can_revert } = &bundle.body[1] {
            assert_eq!(tx, &transactions[1].0);
            assert_eq!(*can_revert, transactions[1].1);
        } else {
            panic!("Expected Transaction bundle item");
        }
        
        assert!(bundle.privacy.is_some());
        let privacy = bundle.privacy.unwrap();
        assert_eq!(privacy.hints, hints);
        assert_eq!(privacy.builders, builders);
    }
    
    #[test]
    fn test_create_mev_share_bundle() {
        let transactions = vec![
            ("0x02f86c0180843b9aca00825208945555763613a12d8f3e73be831dff8598089d4fde8702ce7de1537c008025a0bfe7d7e3e56fc3513640105b5c0ce4d50c47e6ec40b5876459105744a5238f07a05ec5fd4b2e1c0f15164604ac370929c397e8ea82a11527be39583135093162c3".to_string(), false),
        ];
        
        let hints = Some(vec![PrivacyHint::TxHash]);
        let refund_addresses = vec![
            ("0x1234567890123456789012345678901234567890".to_string(), 90),
        ];
        
        let bundle = create_mev_share_bundle(
            transactions.clone(),
            12345678,
            Some(12345680),
            hints.clone(),
            None,
            refund_addresses,
        );
        
        assert_eq!(bundle.version, "v0.1");
        assert_eq!(bundle.inclusion.block, "0xbc614e");
        
        // Check validity configuration
        assert!(bundle.validity.is_some());
        let validity = bundle.validity.unwrap();
        assert!(validity.refund_config.is_some());
        let refund_config = validity.refund_config.unwrap();
        assert_eq!(refund_config.len(), 1);
        assert_eq!(refund_config[0].address, "0x1234567890123456789012345678901234567890");
        assert_eq!(refund_config[0].percent, 90);
    }
} 