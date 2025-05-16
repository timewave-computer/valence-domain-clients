//-----------------------------------------------------------------------------
// EVM Utility Functions
//-----------------------------------------------------------------------------

//! Utility functions for working with Ethereum Virtual Machine (EVM) chains.
//! 
//! This module provides helper functions for common EVM operations such as
//! address validation, transaction handling, and data encoding/decoding.

// Use our abstracted crypto module instead of directly using ethers
use crate::evm::crypto::{keccak256, sign_message, get_address_from_private_key};
use hex;

use crate::core::error::ClientError;
use crate::evm::types::{EvmAddress, EvmBytes, EvmHash, EvmTransactionReceipt, EvmU256};

//-----------------------------------------------------------------------------
// Type Conversions
//-----------------------------------------------------------------------------

/// Convert a string to an EvmAddress
pub fn parse_address(address: &str) -> Result<EvmAddress, String> {
    address.parse()
}

/// Convert a string to an EvmHash
pub fn parse_hash(hash: &str) -> Result<EvmHash, String> {
    hash.parse()
}

/// Convert a hex string (with or without 0x prefix) to bytes
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, ClientError> {
    let clean_hex = hex_str.trim_start_matches("0x");
    
    hex::decode(clean_hex)
        .map_err(|e| ClientError::ParseError(format!("Failed to decode hex: {}", e)))
}

/// Convert bytes to a hex string with 0x prefix
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Parse a hex string to an EvmU256
pub fn parse_hex_u256(hex_str: &str) -> Result<EvmU256, ClientError> {
    let clean_hex = hex_str.trim_start_matches("0x");
    
    // Handle empty string
    if clean_hex.is_empty() {
        return Ok(EvmU256::from_u64(0));
    }
    
    let bytes = hex::decode(clean_hex)
        .map_err(|e| ClientError::ParseError(format!("Failed to decode hex: {}", e)))?;
        
    // Convert bytes to u64 (simplified)
    let mut value: u64 = 0;
    for byte in bytes.iter().take(8) {
        value = (value << 8) + (*byte as u64);
    }
    
    Ok(EvmU256::from_u64(value))
}

/// Validate an Ethereum address
pub fn validate_ethereum_address(address: &str) -> bool {
    // Check if it starts with 0x and has the right length
    if !address.starts_with("0x") || address.len() != 42 {
        return false;
    }
    
    // Check if it's a valid hex string
    hex::decode(&address[2..]).is_ok()
}

//-----------------------------------------------------------------------------
// ABI Helper Functions
//-----------------------------------------------------------------------------

/// Encode a function call with parameters
pub fn encode_function_call(function_signature: &str, params: Vec<&str>) -> Result<EvmBytes, String> {
    // Calculate function selector (first 4 bytes of keccak256 hash of function signature)
    let signature_hash = keccak256(function_signature.as_bytes());
    let selector = &signature_hash.0[0..4];
    
    // Simple implementation - would need proper ABI encoding in real implementation
    let mut encoded = Vec::with_capacity(4 + params.len() * 32);
    encoded.extend_from_slice(selector);
    
    // Encode parameters (simplified)
    for param in params {
        let param_bytes = param.as_bytes();
        let mut padded = vec![0u8; 32];
        
        // Right-align bytes in 32-byte word
        let copy_len = std::cmp::min(param_bytes.len(), 32);
        padded[32 - copy_len..].copy_from_slice(&param_bytes[..copy_len]);
        
        encoded.extend_from_slice(&padded);
    }
    
    Ok(EvmBytes(encoded))
}

/// Decode a function response
pub fn decode_function_response(response: &EvmBytes) -> Vec<String> {
    // Simple implementation - would need proper ABI decoding in real implementation
    let mut results = Vec::new();
    
    // Decode each 32-byte word
    for i in 0..(response.0.len() / 32) {
        let word = &response.0[i * 32..(i + 1) * 32];
        
        // Remove leading zeros and convert to hex
        let first_non_zero = word.iter().position(|&b| b != 0).unwrap_or(31);
        let hex = hex::encode(&word[first_non_zero..]);
        results.push(format!("0x{}", hex));
    }
    
    results
}

//-----------------------------------------------------------------------------
// Crypto Utility Functions
//-----------------------------------------------------------------------------

/// Compute recovery id from v value (EIP-155 compatible)
pub fn get_recovery_id(v: u64, _chain_id: u64) -> u8 {
    // EIP-155: v = {0,1} + CHAIN_ID * 2 + 35
    if v >= 35 {
        let recovery_id = ((v - 35) % 2) as u8;
        // Verify chain ID matches expected (in practice)
        recovery_id
    } else {
        // Legacy transaction (pre-EIP-155)
        v as u8 - 27
    }
}

/// Format an EVM receipt for display
pub fn format_receipt(receipt: &EvmTransactionReceipt) -> String {
    format!(
        "Transaction Receipt:\n  Status: {}\n  Block: {}\n  Gas Used: {:?}",
        if receipt.status == 1 { "Success" } else { "Failed" },
        receipt.block_number,
        receipt.gas_used
    )
}

/// Parse an Ethereum chain ID string to a u64
pub fn parse_chain_id(chain_id: &str) -> Result<u64, ClientError> {
    match chain_id {
        "ethereum" | "mainnet" => Ok(1),
        "goerli" => Ok(5),
        "sepolia" => Ok(11155111),
        "base" | "base-mainnet" => Ok(8453),
        "optimism" | "optimism-mainnet" => Ok(10),
        "arbitrum" | "arbitrum-one" => Ok(42161),
        "polygon" | "polygon-mainnet" => Ok(137),
        "avalanche" | "avalanche-c-chain" => Ok(43114),
        _ => {
            // Try to parse as a number
            chain_id.parse::<u64>()
                .map_err(|e| ClientError::ParseError(format!("Invalid chain ID: {}", e)))
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        // Valid address
        let valid_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
        let result = parse_address(valid_address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string().to_lowercase(), valid_address.to_lowercase());

        // Invalid address (too short)
        let invalid_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f4";
        let result = parse_address(invalid_address);
        assert!(result.is_err());

        // Invalid address (not hex)
        let invalid_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44z";
        let result = parse_address(invalid_address);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hash() {
        // Valid hash
        let valid_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = parse_hash(valid_hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string().to_lowercase(), valid_hash.to_lowercase());

        // Invalid hash (too short)
        let invalid_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd";
        let result = parse_hash(invalid_hash);
        assert!(result.is_err());

        // Invalid hash (not hex)
        let invalid_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdez";
        let result = parse_hash(invalid_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_bytes() {
        let hex = "0x1234";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x12, 0x34]);

        let hex = "1234";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x12, 0x34]);
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![0x12, 0x34];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "0x1234");
    }

    #[test]
    fn test_encode_function_call() {
        // Test with no parameters
        let func_sig = "getValue()";
        let params: Vec<&str> = vec![];
        let result = encode_function_call(func_sig, params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 4); // Only selector

        // Test with one parameter
        let func_sig = "setValue(uint256)";
        let params: Vec<&str> = vec!["123"];
        let result = encode_function_call(func_sig, params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 4 + 32); // Selector + 1 param

        // Test with multiple parameters
        let func_sig = "setValues(uint256,string)";
        let params: Vec<&str> = vec!["123", "hello"];
        let result = encode_function_call(func_sig, params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 4 + 32 * 2); // Selector + 2 params

        // Test with parameter larger than 32 bytes (should be truncated)
        let func_sig = "setValue(string)";
        let long_param = "a".repeat(40);
        let params: Vec<&str> = vec![&long_param];
        let result = encode_function_call(func_sig, params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 4 + 32); // Selector + 1 param (truncated)
    }

    #[test]
    fn test_decode_function_response() {
        // Test empty response
        let empty_response = EvmBytes(Vec::new());
        let result = decode_function_response(&empty_response);
        assert_eq!(result.len(), 0);

        // Test response with one word (all zeros)
        let zeros = vec![0u8; 32];
        let response = EvmBytes(zeros);
        let result = decode_function_response(&response);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "0x00");

        // Test response with one word (with content)
        let mut word = vec![0u8; 32];
        word[31] = 42; // Set last byte to 42
        let response = EvmBytes(word);
        let result = decode_function_response(&response);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "0x2a"); // 42 in hex

        // Test response with multiple words
        let mut words = Vec::new();
        // First word
        let mut word1 = vec![0u8; 32];
        word1[31] = 123;
        words.extend_from_slice(&word1);
        // Second word
        let mut word2 = vec![0u8; 32];
        word2[30] = 65; // 'A'
        word2[31] = 66; // 'B'
        words.extend_from_slice(&word2);
        
        let response = EvmBytes(words);
        let result = decode_function_response(&response);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "0x7b"); // 123 in hex
        assert_eq!(result[1], "0x4142"); // 'AB' in hex
    }

    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let hash = keccak256(data);
        assert_eq!(
            hash.0,
            hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad").unwrap()
        );
    }

    #[test]
    fn test_get_recovery_id() {
        // Test EIP-155 transaction (v >= 35)
        let chain_id = 1; // Ethereum mainnet
        
        // v = 0 + chainId * 2 + 35 = 37
        let recovery_id = get_recovery_id(37, chain_id);
        assert_eq!(recovery_id, 0);
        
        // v = 1 + chainId * 2 + 35 = 38
        let recovery_id = get_recovery_id(38, chain_id);
        assert_eq!(recovery_id, 1);
        
        // Test different chain_id
        let chain_id = 42; // Kovan testnet
        
        // v = 0 + chainId * 2 + 35 = 119
        let recovery_id = get_recovery_id(119, chain_id);
        assert_eq!(recovery_id, 0);
        
        // v = 1 + chainId * 2 + 35 = 120
        let recovery_id = get_recovery_id(120, chain_id);
        assert_eq!(recovery_id, 1);
        
        // Test legacy transaction (v < 35)
        let recovery_id = get_recovery_id(27, chain_id);
        assert_eq!(recovery_id, 0);
        
        let recovery_id = get_recovery_id(28, chain_id);
        assert_eq!(recovery_id, 1);
    }

    #[test]
    fn test_sign_message() {
        // Create a known private key for testing
        let private_key = [
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        
        let message = b"hello world";
        let result = sign_message(&private_key, message);
        assert!(result.is_ok());
        
        let (signature, recovery_id) = result.unwrap();
        assert_eq!(signature.len(), 64); // r and s values, 32 bytes each
        assert!(recovery_id <= 1); // 0 or 1
    }

    #[test]
    fn test_get_address_from_private_key() {
        // Create a known private key for testing
        let private_key = [
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        
        let address = get_address_from_private_key(&private_key).unwrap();
        assert_eq!(address.len(), 20); // Ethereum address is 20 bytes
    }

    #[test]
    fn test_validate_ethereum_address() {
        let valid_address = "0x1234567890123456789012345678901234567890";
        assert!(validate_ethereum_address(valid_address));
        
        let invalid_address = "1234567890123456789012345678901234567890";
        assert!(!validate_ethereum_address(invalid_address));
        
        let invalid_address = "0x12345";
        assert!(!validate_ethereum_address(invalid_address));
    }
}
