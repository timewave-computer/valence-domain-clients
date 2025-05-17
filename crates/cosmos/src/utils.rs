//-----------------------------------------------------------------------------
// Cosmos Utility Functions
//-----------------------------------------------------------------------------

//! Utility functions for working with Cosmos-based chains.
//!
//! This module provides helper functions for common Cosmos operations such as
//! address validation, denomination handling, and data encoding/decoding.

use bech32::{self, ToBase32, Variant};

use crate::types::{CosmosCoin, CosmosFee};
use cosmos_sdk_proto::{
    cosmos::base::abci::v1beta1::TxResponse, tendermint::abci::Event,
};
use std::str::FromStr;
use valence_core::error::ClientError;
use valence_core::transaction::{self, TransactionResponse};

// Import needed for EventV034 and proper Event conversion
use cosmos_sdk_proto::cosmos::base::abci::v1beta1::StringEvent;

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
    bech32::encode(hrp, bytes.to_base32(), Variant::Bech32).map_err(|e| {
        ClientError::ParseError(format!("Failed to create Cosmos address: {e}"))
    })
}

/// Creates a fee object for Cosmos transactions
pub fn create_fee(amount: Vec<CosmosCoin>, gas_limit: u64) -> CosmosFee {
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
        return Err(ClientError::ParseError(format!(
            "Invalid coin string format: {coin_str}"
        )));
    }

    let amount = amount_chars.parse::<u128>().map_err(|e| {
        ClientError::ParseError(format!(
            "Failed to parse amount '{amount_chars}' from coin string: {e}"
        ))
    })?;

    Ok((amount, denom_chars))
}

/// Convert bytes to a String
pub fn bytes_to_string(b: &prost::bytes::Bytes) -> String {
    String::from_utf8_lossy(b.as_ref()).to_string()
}

/// Convert proto events to our internal event type
pub fn convert_proto_events(proto_events: Vec<Event>) -> Vec<transaction::Event> {
    proto_events
        .into_iter()
        .map(|e| transaction::Event {
            event_type: e.r#type,
            attributes: e
                .attributes
                .into_iter()
                .map(|a| {
                    // Convert Vec<u8> to String
                    let key = String::from_utf8_lossy(a.key.as_ref()).to_string();
                    let value =
                        String::from_utf8_lossy(a.value.as_ref()).to_string();
                    (key, value)
                })
                .collect(),
        })
        .collect()
}

/// Convert a proto tx response to our internal type
pub fn convert_from_proto_tx_response(
    tx_response: TxResponse,
) -> Result<TransactionResponse, ClientError> {
    // Convert events directly - but properly convert Bytes to Strings
    let events_vec = if !tx_response.events.is_empty() {
        tx_response
            .events
            .into_iter()
            .map(|e| transaction::Event {
                event_type: e.r#type,
                attributes: e
                    .attributes
                    .into_iter()
                    .map(|a| {
                        // Convert using helper function
                        (bytes_to_string(&a.key), bytes_to_string(&a.value))
                    })
                    .collect::<Vec<(String, String)>>(),
            })
            .collect()
    } else {
        vec![]
    };

    // Convert height from i64 to u64
    let height = tx_response.height as u64;

    // Use gas values directly
    let gas_wanted = tx_response.gas_wanted;
    let gas_used = tx_response.gas_used;

    // Parse timestamp as i64 or default to 0
    let timestamp = i64::from_str(&tx_response.timestamp).unwrap_or(0);

    Ok(TransactionResponse {
        tx_hash: tx_response.txhash,
        height,
        code: Some(tx_response.code),
        raw_log: Some(tx_response.raw_log),
        gas_wanted: Some(gas_wanted),
        gas_used: Some(gas_used),
        events: events_vec,
        data: Some(tx_response.data),
        timestamp,
        block_hash: None, // Optional field, not provided in Cosmos response
        original_request_payload: None, // Optional field, not provided in Cosmos response
    })
}

/// Convert a StringEvent to our standard Event type
pub fn convert_tendermint_proto_event(event: StringEvent) -> Event {
    // Create a new tendermint ABCI Event
    let mut cosmos_evt = Event {
        r#type: event.r#type,
        attributes: vec![],
    };

    // Convert attributes from strings to EventAttribute with String fields
    for attr in event.attributes {
        cosmos_evt.attributes.push(
            cosmos_sdk_proto::tendermint::abci::EventAttribute {
                key: attr.key,
                value: attr.value,
                index: false,
            },
        );
    }

    cosmos_evt
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
        let bytes = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ];

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
        let coins = vec![CosmosCoin {
            denom: "uatom".to_string(),
            amount: 5000,
        }];

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

    #[test]
    fn test_convert_proto_events() {
        // Create a proto event
        let mut proto_event = Event {
            r#type: "transfer".to_string(),
            attributes: vec![],
        };

        // Add some attributes
        proto_event.attributes.push(
            cosmos_sdk_proto::tendermint::abci::EventAttribute {
                key: "sender".as_bytes().to_vec(),
                value: "cosmos123".as_bytes().to_vec(),
                index: false,
            },
        );

        proto_event.attributes.push(
            cosmos_sdk_proto::tendermint::abci::EventAttribute {
                key: "amount".as_bytes().to_vec(),
                value: "100uatom".as_bytes().to_vec(),
                index: false,
            },
        );

        // Convert to our event type
        let events = convert_proto_events(vec![proto_event]);

        // Validate
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "transfer");
        assert_eq!(events[0].attributes.len(), 2);
        assert_eq!(events[0].attributes.get("sender").unwrap(), "cosmos123");
        assert_eq!(events[0].attributes.get("amount").unwrap(), "100uatom");
    }

    #[test]
    fn test_convert_from_proto_tx_response() {
        // Create a minimal TxResponse with empty events
        let tx_response = TxResponse {
            height: 123,
            txhash: "ABCDEF".to_string(),
            code: 0,
            raw_log: "success".to_string(),
            gas_wanted: 100000,
            gas_used: 50000,
            timestamp: "2023-05-01T12:00:00Z".to_string(),
            events: vec![],
            ..Default::default()
        };

        // Convert to our type
        let response = convert_from_proto_tx_response(tx_response).unwrap();

        // Validate
        assert_eq!(response.tx_hash, "ABCDEF");
        assert_eq!(response.height, 123);
        assert_eq!(response.code, Some(0));
        assert_eq!(response.raw_log, Some("success".to_string()));
        assert_eq!(response.gas_wanted, Some(100000));
        assert_eq!(response.gas_used, Some(50000));
        assert!(response.events.is_empty());
        assert!(response.block_hash.is_none());
        assert!(response.original_request_payload.is_none());
    }
}
