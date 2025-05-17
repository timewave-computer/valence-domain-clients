//-----------------------------------------------------------------------------
// Cosmos Utility Functions
//-----------------------------------------------------------------------------

//! Utility functions for working with Cosmos-based chains.
//! 
//! This module provides helper functions for common Cosmos operations such as
//! address validation, denomination handling, and data encoding/decoding.

use bech32::{self, ToBase32, Variant};

use crate::core::error::ClientError;
use crate::cosmos::types::{CosmosCoin, CosmosFee};

/// Validates a Cosmos address string for the given prefix
/// 
/// # Examples
/// ```
/// use valence_domain_clients::cosmos::utils::validate_cosmos_address;
/// 
/// let is_valid = validate_cosmos_address("cosmos1...", "cosmos");
/// ```
pub fn validate_cosmos_address(address: &str, prefix: &str) -> bool {
    if !address.starts_with(prefix) {
        return false;
    }
    
    match bech32::decode(address) {
        Ok((decoded_prefix, _, _)) => decoded_prefix == prefix,
        Err(_) => false,
    }
}

/// Converts a human-readable address with a given prefix to a Cosmos address
pub fn to_cosmos_address(hrp: &str, bytes: &[u8]) -> Result<String, ClientError> {
    bech32::encode(hrp, bytes.to_base32(), Variant::Bech32)
        .map_err(|e| ClientError::ParseError(format!("Failed to create Cosmos address: {e}")))
}

/// Creates a fee object for Cosmos transactions
pub fn create_fee(
    amount: Vec<CosmosCoin>,
    gas_limit: u64,
) -> CosmosFee {
    CosmosFee {
        amount,
        gas_limit,
        payer: None,
        granter: None,
    }
}

/// Calculates the adjusted gas limit based on a simulated gas usage
pub fn calculate_gas_limit(simulated_gas: u64, gas_adjustment: f64) -> u64 {
    (simulated_gas as f64 * gas_adjustment).ceil() as u64
}

/// Formats coin amount and denomination into a standard Cosmos format
/// 
/// # Examples
/// ```
/// use valence_domain_clients::cosmos::utils::format_coin_amount;
///
/// let formatted = format_coin_amount(1000000, "uatom"); // "1000000uatom"
/// ```
pub fn format_coin_amount(amount: u128, denom: &str) -> String {
    format!("{amount}{denom}")
}

/// Parse a coin string into amount and denomination
/// 
/// # Examples
/// ```
/// use valence_domain_clients::cosmos::utils::parse_coin_string;
/// use valence_domain_clients::core::error::ClientError;
///
/// let (amount, denom) = parse_coin_string("1000000uatom").unwrap();
/// assert_eq!(amount, 1000000);
/// assert_eq!(denom, "uatom");
/// ```
pub fn parse_coin_string(coin_str: &str) -> Result<(u128, String), ClientError> {
    let mut amount_chars = String::new();
    let mut denom_chars = String::new();
    let mut collecting_amount = true;

    for c in coin_str.chars() {
        if collecting_amount && c.is_ascii_digit() {
            amount_chars.push(c);
        } else {
            collecting_amount = false;
            denom_chars.push(c);
        }
    }

    if amount_chars.is_empty() || denom_chars.is_empty() {
        return Err(ClientError::ParseError(
            format!("Invalid coin string format: {coin_str}")
        ));
    }

    let amount = amount_chars.parse::<u128>()
        .map_err(|e| ClientError::ParseError(
            format!("Failed to parse amount '{amount_chars}' from coin string: {e}")
        ))?;

    Ok((amount, denom_chars))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cosmos_address() {
        // Valid cosmos address
        let valid_address = "cosmos1jxv0u20scum4trha72c7ltfgfqef6nscj25050";
        assert!(validate_cosmos_address(valid_address, "cosmos"));
        
        // Valid address but wrong prefix
        assert!(!validate_cosmos_address(valid_address, "osmo"));
        
        // Invalid bech32 address
        let invalid_address = "cosmos1jxv0u20scum4trha72c7ltfgfqef6nscj2505O"; // Contains capital O instead of 0
        assert!(!validate_cosmos_address(invalid_address, "cosmos"));
        
        // Not starting with prefix
        let incorrect_prefix = "osmo1jxv0u20scum4trha72c7ltfgfqef6nscj25050";
        assert!(!validate_cosmos_address(incorrect_prefix, "cosmos"));
    }
    
    #[test]
    fn test_to_cosmos_address() {
        // Sample public key bytes (this is just a test example, not a real key)
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
        
        // Test with cosmos prefix
        let cosmos_address = to_cosmos_address("cosmos", &bytes).unwrap();
        assert!(cosmos_address.starts_with("cosmos1"));
        assert!(validate_cosmos_address(&cosmos_address, "cosmos"));
        
        // Test with osmo prefix
        let osmo_address = to_cosmos_address("osmo", &bytes).unwrap();
        assert!(osmo_address.starts_with("osmo1"));
        assert!(validate_cosmos_address(&osmo_address, "osmo"));
    }
    
    #[test]
    fn test_create_fee() {
        let coins = vec![
            CosmosCoin {
                denom: "uatom".to_string(),
                amount: 5000,
            }
        ];
        
        let gas_limit = 200000u64;
        
        let fee = create_fee(coins.clone(), gas_limit);
        
        assert_eq!(fee.amount, coins);
        assert_eq!(fee.gas_limit, gas_limit);
        assert_eq!(fee.payer, None);
        assert_eq!(fee.granter, None);
    }
    
    #[test]
    fn test_calculate_gas_limit() {
        // Test with whole number results
        let simulated_gas = 100000u64;
        let adjustment = 1.5f64;
        assert_eq!(calculate_gas_limit(simulated_gas, adjustment), 150000u64);
        
        // Test with fractional results (should ceil)
        let simulated_gas = 100001u64;
        let adjustment = 1.2f64;
        assert_eq!(calculate_gas_limit(simulated_gas, adjustment), 120002u64); // 100001 * 1.2 = 120001.2, ceiled to 120002
        
        // Test with adjustment < 1.0
        let simulated_gas = 100000u64;
        let adjustment = 0.8f64;
        assert_eq!(calculate_gas_limit(simulated_gas, adjustment), 80000u64);
    }
    
    #[test]
    fn test_format_coin_amount() {
        assert_eq!(format_coin_amount(1000000, "uatom"), "1000000uatom");
        assert_eq!(format_coin_amount(0, "uosmo"), "0uosmo");
        assert_eq!(format_coin_amount(999, "stake"), "999stake");
    }
    
    #[test]
    fn test_parse_coin_string() {
        // Test valid coin strings
        let result = parse_coin_string("1000000uatom").unwrap();
        assert_eq!(result.0, 1000000u128);
        assert_eq!(result.1, "uatom");
        
        let result = parse_coin_string("0stake").unwrap();
        assert_eq!(result.0, 0u128);
        assert_eq!(result.1, "stake");
        
        // Test with large numbers
        let result = parse_coin_string("18446744073709551615uosmo").unwrap();
        assert_eq!(result.0, 18446744073709551615u128);
        assert_eq!(result.1, "uosmo");
        
        // Test invalid coin strings
        let err_result = parse_coin_string("uatom"); // No amount
        assert!(err_result.is_err());
        
        let err_result = parse_coin_string("1000000"); // No denom
        assert!(err_result.is_err());
        
        let err_result = parse_coin_string("invalid"); // No digits
        assert!(err_result.is_err());
        
        let err_result = parse_coin_string(""); // Empty string
        assert!(err_result.is_err());
    }
}
