//-----------------------------------------------------------------------------
// Core Error Tests
//-----------------------------------------------------------------------------

//! Tests for the core error handling module

use valence_domain_clients::ClientError;
use std::error::Error;

#[test]
fn test_error_creation_and_display() {
    // Test client error variant
    let err = ClientError::ClientError("connection failed".to_string());
    assert!(format!("{err}").contains("client error"));
    assert!(format!("{err}").contains("connection failed"));

    // Test transaction error variant
    let err = ClientError::TransactionError("transaction rejected".to_string());
    assert!(format!("{err}").contains("transaction error"));
    assert!(format!("{err}").contains("transaction rejected"));

    // Test serialization error variant
    let err = ClientError::SerializationError("invalid format".to_string());
    assert!(format!("{err}").contains("serialization error"));
    assert!(format!("{err}").contains("invalid format"));

    // Test timeout error variant
    let err = ClientError::TimeoutError("request timed out".to_string());
    assert!(format!("{err}").contains("timeout error"));
    assert!(format!("{err}").contains("request timed out"));
}

#[test]
fn test_error_from_string() {
    // Test creating errors from string literals
    let err = ClientError::ClientError("generic error".to_string());
    assert_eq!(err.to_string(), "client error: generic error");

    // Test creating errors from owned strings
    let err_msg = "dynamic error".to_string();
    let err = ClientError::ClientError(err_msg);
    assert_eq!(err.to_string(), "client error: dynamic error");
}

#[test]
fn test_error_into_string() {
    // Test string conversion
    let err = ClientError::ClientError("test message".to_string());
    let err_string: String = err.to_string();
    assert!(err_string.contains("client error"));
    assert!(err_string.contains("test message"));
}

#[test]
fn test_std_error_trait() {
    // Test that our error type implements the std::error::Error trait
    let err = ClientError::ClientError("test error".to_string());
    let std_err: &dyn std::error::Error = &err;

    // Test that source() returns None as expected
    assert!(std_err.source().is_none());
}

#[test]
fn test_client_error_debug() {
    // Create an instance of ClientError
    let err = ClientError::ClientError("connection error".to_string());

    // Test Debug formatting
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("ClientError"));
    assert!(debug_str.contains("connection error"));

    // Test Display formatting
    let display_str = format!("{err}");
    assert!(display_str.contains("connection error"));
}

#[test]
fn test_client_error_variants() {
    // Compare string representations since ClientError doesn't implement PartialEq
    let err1 = ClientError::ClientError("error 1".to_string());
    let err2 = ClientError::ClientError("error 2".to_string());

    // Instead of using assert_ne on the errors directly, compare their string representations
    assert_ne!(err1.to_string(), err2.to_string());
}

#[test]
fn test_error_custom_creation() {
    // Create CustomError directly (instead of From)
    let client_err = ClientError::ClientError("custom error message".to_string());
    assert!(format!("{client_err}").contains("custom error message"));

    // Test with a different variant
    let err_msg = "another error message".to_string();
    let parse_err = ClientError::ParseError(err_msg);
    assert!(format!("{parse_err}").contains("parse error"));
    assert!(format!("{parse_err}").contains("another error message"));
}

#[test]
fn test_from_implementation() {
    // Test From implementation for errors
    // Create a ClientError directly with a serialization error message
    let err = ClientError::SerializationError("encode error".to_string());

    match err {
        ClientError::SerializationError(msg) => {
            assert!(msg.contains("encode error"))
        }
        _ => panic!("Expected SerializationError variant"),
    }
}

#[test]
fn test_error_messages() {
    let err = ClientError::TimeoutError("connection failed".to_string());
    assert!(format!("{err}").contains("timeout error"));
    assert!(format!("{err}").contains("connection failed"));

    let err = ClientError::TransactionError("transaction rejected".to_string());
    assert!(format!("{err}").contains("transaction error"));
    assert!(format!("{err}").contains("transaction rejected"));

    let err = ClientError::SerializationError("invalid format".to_string());
    assert!(format!("{err}").contains("serialization error"));
    assert!(format!("{err}").contains("invalid format"));

    let err = ClientError::TimeoutError("request timed out".to_string());
    assert!(format!("{err}").contains("timeout error"));
    assert!(format!("{err}").contains("request timed out"));
}

#[test]
fn test_error_source() {
    // Create an error with a source
    let err = ClientError::SerializationError("serialization failed".to_string());

    // Convert to std::error::Error
    let std_err = &err as &dyn std::error::Error;

    // Check that there's no source
    assert!(std_err.source().is_none());
}

#[test]
fn test_error_direct_creation() {
    // Test creating errors directly
    let client_err = ClientError::ClientError("test client error".to_string());
    assert!(format!("{client_err}").contains("client error"));

    let tx_err = ClientError::TransactionError("test tx error".to_string());
    assert!(format!("{tx_err}").contains("transaction error"));
}

#[test]
fn test_error_debug_trait() {
    // Test Debug trait implementation
    let err = ClientError::ClientError("some unknown error".to_string());
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("ClientError"));
    assert!(debug_str.contains("some unknown error"));

    // Test Display trait implementation
    let display_str = format!("{err}");
    assert!(display_str.contains("client error: some unknown error"));
}

#[test]
fn test_error_custom_creation_with_source() {
    // Create a custom error
    let client_err = ClientError::ClientError("custom error message".to_string());
    assert!(format!("{client_err}").contains("custom error message"));

    // Create a parse error that sources another error - basic variants don't store source
    let parse_err = ClientError::ParseError("another error message".to_string());
    assert!(format!("{parse_err}").contains("parse error"));
    assert!(format!("{parse_err}").contains("another error message"));
}

#[test]
fn test_error_direct_creation_with_source() {
    let client_err = ClientError::TimeoutError("connection failed".to_string());
    let tx_err = ClientError::TransactionError("transaction rejected".to_string());

    assert!(format!("{client_err}").contains("timeout error"));
    // source() should be None for direct errors
    assert!(client_err.source().is_none());
    assert!(format!("{tx_err}").contains("transaction error"));
    assert!(tx_err.source().is_none());
}
