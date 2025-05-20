//-----------------------------------------------------------------------------
// Core Transaction Tests
//-----------------------------------------------------------------------------

//! Tests for the core transaction module

use valence_domain_clients::core::transaction::{Event, TransactionResponse};

#[test]
fn test_transaction_response_creation() {
    // Create a basic transaction response
    let tx = TransactionResponse {
        tx_hash: "0xABCDEF1234567890".to_string(),
        height: 12345,
        gas_used: Some(50000),
        gas_wanted: Some(70000),
        events: vec![],
        code: None,
        raw_log: None,
        data: None,
        timestamp: 1625097600, // Example timestamp
        block_hash: None,
        original_request_payload: None,
    };

    // Verify basic properties
    assert_eq!(tx.tx_hash, "0xABCDEF1234567890");
    assert_eq!(tx.height, 12345);
    assert_eq!(tx.gas_used, Some(50000));
    assert_eq!(tx.gas_wanted, Some(70000));
    assert_eq!(tx.events.len(), 0);
    assert_eq!(tx.code, None);
    assert_eq!(tx.raw_log, None);
    assert_eq!(tx.data, None);
}

#[test]
fn test_transaction_response_with_events() {
    // Create some events
    let event1 = Event {
        event_type: "transfer".to_string(),
        attributes: vec![
            ("sender".to_string(), "addr1".to_string()),
            ("recipient".to_string(), "addr2".to_string()),
            ("amount".to_string(), "100".to_string()),
        ],
    };

    let event2 = Event {
        event_type: "message".to_string(),
        attributes: vec![
            ("module".to_string(), "bank".to_string()),
            ("action".to_string(), "transfer".to_string()),
        ],
    };

    // Create a transaction response with events
    let tx = TransactionResponse {
        tx_hash: "0x123456789abcdef".to_string(),
        height: 123456,
        gas_wanted: Some(200000),
        gas_used: Some(150000),
        code: Some(0),
        events: vec![event1.clone(), event2.clone()],
        data: None,
        raw_log: None,
        timestamp: 1625097600, // Example timestamp
        block_hash: None,
        original_request_payload: None,
    };

    // Verify properties
    assert_eq!(tx.tx_hash, "0x123456789abcdef");
    assert_eq!(tx.height, 123456);
    assert_eq!(tx.gas_used, Some(150000));
    assert_eq!(tx.gas_wanted, Some(200000));
    assert_eq!(tx.events.len(), 2);
    assert_eq!(tx.events[0].event_type, "transfer");
    assert_eq!(tx.events[0].attributes.len(), 3);
    assert_eq!(tx.events[0].attributes[0].0, "sender");
    assert_eq!(tx.events[0].attributes[0].1, "addr1");
    assert_eq!(tx.events[1].event_type, "message");
    assert_eq!(tx.events[1].attributes.len(), 2);
    assert_eq!(tx.events[1].attributes[0].0, "module");
    assert_eq!(tx.events[1].attributes[0].1, "bank");
    assert_eq!(tx.events[1].attributes[1].0, "action");
    assert_eq!(tx.events[1].attributes[1].1, "transfer");
    assert_eq!(tx.code, Some(0));
    assert_eq!(tx.raw_log, None);
    assert_eq!(tx.data, None);
    assert_eq!(tx.timestamp, 1625097600);
    assert_eq!(tx.block_hash, None);
    assert_eq!(tx.original_request_payload, None);
}

#[test]
fn test_event_creation_and_access() {
    // Create an event
    let event = Event {
        event_type: "token_transfer".to_string(),
        attributes: vec![
            ("token".to_string(), "USDC".to_string()),
            ("amount".to_string(), "5000000".to_string()),
            ("sender".to_string(), "0xSender".to_string()),
            ("receiver".to_string(), "0xReceiver".to_string()),
        ],
    };

    // Verify event properties
    assert_eq!(event.event_type, "token_transfer");
    assert_eq!(event.attributes.len(), 4);

    // Test attribute access
    assert_eq!(event.attributes[0].0, "token");
    assert_eq!(event.attributes[0].1, "USDC");
    assert_eq!(event.attributes[1].0, "amount");
    assert_eq!(event.attributes[1].1, "5000000");

    // Test finding attributes by key
    let amount = event
        .attributes
        .iter()
        .find(|(k, _)| k == "amount")
        .map(|(_, v)| v)
        .unwrap();
    assert_eq!(amount, "5000000");

    // Test attribute not found
    let not_found = event
        .attributes
        .iter()
        .find(|(k, _)| k == "nonexistent")
        .map(|(_, v)| v);
    assert_eq!(not_found, None);
}

#[test]
fn test_transaction_response_debug_and_clone() {
    // Create a transaction response
    let tx = TransactionResponse {
        tx_hash: "0xHASH".to_string(),
        height: 1000,
        gas_used: Some(21000),
        gas_wanted: Some(30000),
        events: vec![Event {
            event_type: "test_event".to_string(),
            attributes: vec![("test_key".to_string(), "test_value".to_string())],
        }],
        code: None,
        raw_log: None,
        data: None,
        timestamp: 1625097600, // Example timestamp
        block_hash: None,
        original_request_payload: None,
    };

    // Test Debug implementation
    let debug_str = format!("{tx:?}");
    assert!(debug_str.contains("0xHASH"));
    assert!(debug_str.contains("1000"));
    assert!(debug_str.contains("21000"));

    // Test Clone implementation
    let tx_clone = tx.clone();
    assert_eq!(tx.tx_hash, tx_clone.tx_hash);
    assert_eq!(tx.height, tx_clone.height);
    assert_eq!(tx.gas_used, tx_clone.gas_used);
    assert_eq!(tx.events.len(), tx_clone.events.len());
    assert_eq!(tx.events[0].event_type, tx_clone.events[0].event_type);
    assert_eq!(
        tx.events[0].attributes.len(),
        tx_clone.events[0].attributes.len()
    );
}

#[test]
fn test_transaction_response_with_all_fields() {
    // Create a transaction response with all fields
    let tx = TransactionResponse {
        tx_hash: "0x123456789abcdef".to_string(),
        height: 123456,
        gas_wanted: Some(200000),
        gas_used: Some(150000),
        code: Some(0),
        events: vec![],
        data: None,
        raw_log: None,
        timestamp: 1625097600, // Example timestamp
        block_hash: None,
        original_request_payload: None,
    };

    // Verify properties
    assert_eq!(tx.tx_hash, "0x123456789abcdef");
    assert_eq!(tx.height, 123456);
    assert_eq!(tx.gas_used, Some(150000));
    assert_eq!(tx.gas_wanted, Some(200000));
    assert_eq!(tx.events.len(), 0);
    assert_eq!(tx.code, Some(0));
    assert_eq!(tx.raw_log, None);
    assert_eq!(tx.data, None);
    assert_eq!(tx.timestamp, 1625097600);
    assert_eq!(tx.block_hash, None);
    assert_eq!(tx.original_request_payload, None);
}

#[test]
fn test_transaction_response_clone() {
    // Create a transaction response
    let tx = TransactionResponse {
        tx_hash: "0x123456789abcdef".to_string(),
        height: 123456,
        gas_wanted: Some(200000),
        gas_used: Some(150000),
        code: Some(0),
        events: vec![],
        data: None,
        raw_log: None,
        timestamp: 1625097600, // Example timestamp
        block_hash: None,
        original_request_payload: None,
    };

    // Clone transaction response
    let tx_clone = tx.clone();

    // Verify they are the same
    assert_eq!(tx.tx_hash, tx_clone.tx_hash);
    assert_eq!(tx.height, tx_clone.height);
    assert_eq!(tx.gas_used, tx_clone.gas_used);
    assert_eq!(tx.gas_wanted, tx_clone.gas_wanted);
    assert_eq!(tx.code, tx_clone.code);
    assert_eq!(tx.timestamp, tx_clone.timestamp);
    assert_eq!(tx.block_hash, tx_clone.block_hash);
    assert_eq!(
        tx.original_request_payload,
        tx_clone.original_request_payload
    );
}

#[test]
fn test_event_attributes() {
    // Create an event with attributes
    let event = Event {
        event_type: "transfer".to_string(),
        attributes: vec![
            ("sender".to_string(), "addr1".to_string()),
            ("recipient".to_string(), "addr2".to_string()),
            ("amount".to_string(), "100".to_string()),
        ],
    };

    // Create a transaction response with the event
    let tx = TransactionResponse {
        tx_hash: "0x123456789abcdef".to_string(),
        height: 123456,
        gas_wanted: Some(200000),
        gas_used: Some(150000),
        code: Some(0),
        events: vec![event],
        data: None,
        raw_log: None,
        timestamp: 1625097600, // Example timestamp
        block_hash: None,
        original_request_payload: None,
    };

    // Verify event attributes
    assert_eq!(tx.events.len(), 1);
    assert_eq!(tx.events[0].event_type, "transfer");
    assert_eq!(tx.events[0].attributes.len(), 3);

    // Check that we can access attributes by index
    let attribute0 = &tx.events[0].attributes[0];
    assert_eq!(attribute0.0, "sender");
    assert_eq!(attribute0.1, "addr1");

    let attribute1 = &tx.events[0].attributes[1];
    assert_eq!(attribute1.0, "recipient");
    assert_eq!(attribute1.1, "addr2");

    let attribute2 = &tx.events[0].attributes[2];
    assert_eq!(attribute2.0, "amount");
    assert_eq!(attribute2.1, "100");
}
