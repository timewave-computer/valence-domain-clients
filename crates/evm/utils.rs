//-----------------------------------------------------------------------------
// EVM Utility Functions
//-----------------------------------------------------------------------------

use crate::evm::types::{EvmAddress, EvmBytes, EvmHash};

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

/// Create bytes from a hex string
pub fn hex_to_bytes(hex: &str) -> Result<EvmBytes, String> {
    EvmBytes::from_hex(hex)
}

//-----------------------------------------------------------------------------
// ABI Helper Functions
//-----------------------------------------------------------------------------

/// Encode a function call with parameters
pub fn encode_function_call(function_signature: &str, params: Vec<&str>) -> Result<EvmBytes, String> {
    // Calculate function selector (first 4 bytes of keccak256 hash of function signature)
    let signature_hash = keccak256(function_signature.as_bytes());
    let selector = &signature_hash[0..4];
    
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

/// Compute keccak256 hash
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    // This is a placeholder implementation
    // In a real implementation, we would use the keccak256 function from a crate
    // like tiny-keccak or sha3
    
    // Simple placeholder hash for demonstration
    let mut output = [0u8; 32];
    for (i, byte) in data.iter().enumerate() {
        if i < 32 {
            output[i] = *byte;
        }
    }
    output
}

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
        // Valid hex
        let valid_hex = "0x1234";
        let result = hex_to_bytes(valid_hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, vec![0x12, 0x34]);

        // Valid hex without 0x prefix
        let valid_hex = "1234";
        let result = hex_to_bytes(valid_hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, vec![0x12, 0x34]);

        // Invalid hex
        let invalid_hex = "0x123z";
        let result = hex_to_bytes(invalid_hex);
        assert!(result.is_err());

        // Empty string
        let empty_hex = "";
        let result = hex_to_bytes(empty_hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, Vec::<u8>::new());
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
        // Test empty data
        let empty_data = b"";
        let result = keccak256(empty_data);
        // Since this is a placeholder implementation, just verify the behavior
        assert_eq!(result.len(), 32);
        
        // Test with data shorter than 32 bytes
        let short_data = b"hello";
        let result = keccak256(short_data);
        assert_eq!(result.len(), 32);
        // Verify that the first 5 bytes match our input (per the placeholder implementation)
        assert_eq!(result[0..5], *short_data);
        // Verify remaining bytes are zeros
        for i in 5..32 {
            assert_eq!(result[i], 0);
        }

        // Test with data longer than 32 bytes
        let long_data = b"this is a test string that is longer than 32 bytes";
        let result = keccak256(long_data);
        assert_eq!(result.len(), 32);
        // Verify that only the first 32 bytes are copied (per the placeholder implementation)
        assert_eq!(&result[0..32], &long_data[0..32]);
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
}
