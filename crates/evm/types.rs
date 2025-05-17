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
        
        let bytes = hex::decode(s)
            .map_err(|e| format!("Invalid hex: {e}"))?;
            
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
        
        let bytes = hex::decode(s)
            .map_err(|e| format!("Invalid hex: {e}"))?;
            
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        
        Ok(EvmHash(hash))
    }
}

impl EvmBytes {
    /// Create a new instance from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str)
            .map_err(|e| format!("Invalid hex: {e}"))?;
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
}

impl fmt::Display for EvmU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Simple implementation for demonstration
        // In practice, would need proper big integer handling
        if self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0 {
            write!(f, "{}", self.0[0])
        } else {
            write!(f, "[{}, {}, {}, {}]", self.0[0], self.0[1], self.0[2], self.0[3])
        }
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
        assert_eq!(address.to_string().to_lowercase(), address_str.to_lowercase());
        
        // Test FromStr with and without 0x prefix
        let address1 = EvmAddress::from_str(address_str).unwrap();
        let address2 = EvmAddress::from_str(&address_str[2..]).unwrap();
        assert_eq!(address1, address2);
        
        // Test error cases
        // Too short
        assert!(EvmAddress::from_str("0x1234").is_err());
        // Invalid hex
        assert!(EvmAddress::from_str("0x12345678901234567890123456789012345678zz").is_err());
        
        // Test serialization/deserialization
        let json = serde_json::to_string(&address).unwrap();
        let deserialized: EvmAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(address, deserialized);
    }
    
    #[test]
    fn test_evm_hash() {
        // Test creation from string
        let hash_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
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
        assert!(EvmHash::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdez").is_err());
        
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
        let address = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
        let topic1 = EvmHash::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let topic2 = EvmHash::from_str("0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321").unwrap();
        let data = EvmBytes::from_hex("0xabcdef").unwrap();
        let tx_hash = EvmHash::from_str("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").unwrap();
        
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
        let tx_hash = EvmHash::from_str("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").unwrap();
        let from_address = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
        let to_address = EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap();
        let contract_address = EvmAddress::from_str("0x9b1f7f645351af3631a656421ed2e40f2802e6c0").unwrap();
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
        assert_eq!(receipt.transaction_hash.to_string(), "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        assert_eq!(receipt.block_number, 12345);
        assert_eq!(receipt.transaction_index, 3);
        assert_eq!(receipt.from.to_string().to_lowercase(), "0x742d35cc6634c0532925a3b844bc454e4438f44e");
        assert_eq!(receipt.to.as_ref().unwrap().to_string().to_lowercase(), "0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0");
        assert_eq!(receipt.contract_address.as_ref().unwrap().to_string().to_lowercase(), "0x9b1f7f645351af3631a656421ed2e40f2802e6c0");
        assert_eq!(receipt.cumulative_gas_used.to_string(), "100000");
        assert_eq!(receipt.gas_used.to_string(), "21000");
        assert_eq!(receipt.status, 1);
        assert_eq!(receipt.logs.len(), 1);
        
        // Test serialization/deserialization
        let json = serde_json::to_string(&receipt).unwrap();
        let deserialized: EvmTransactionReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.transaction_hash, receipt.transaction_hash);
        assert_eq!(deserialized.block_number, receipt.block_number);
        assert_eq!(deserialized.from, receipt.from);
        assert_eq!(deserialized.to, receipt.to);
        assert_eq!(deserialized.status, receipt.status);
    }
    
    #[test]
    fn test_evm_transaction_request() {
        // Create test data
        let from_address = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
        let to_address = EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap();
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
        assert_eq!(tx_request.from.to_string().to_lowercase(), "0x742d35cc6634c0532925a3b844bc454e4438f44e");
        assert_eq!(tx_request.to.as_ref().unwrap().to_string().to_lowercase(), "0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0");
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
        
        assert_eq!(eip1559_tx.max_fee_per_gas.unwrap().to_string(), "100000000000");
        assert_eq!(eip1559_tx.max_priority_fee_per_gas.unwrap().to_string(), "2000000000");
        assert!(eip1559_tx.gas_price.is_none());
        
        // Test serialization/deserialization
        let json = serde_json::to_string(&tx_request).unwrap();
        let deserialized: EvmTransactionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.from, tx_request.from);
        assert_eq!(deserialized.to, tx_request.to);
        assert_eq!(deserialized.nonce, tx_request.nonce);
        assert_eq!(deserialized.gas_limit, tx_request.gas_limit);
        assert_eq!(deserialized.chain_id, tx_request.chain_id);
    }
}
