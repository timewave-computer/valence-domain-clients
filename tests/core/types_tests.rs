//-----------------------------------------------------------------------------
// Core Types Tests
//-----------------------------------------------------------------------------

//! Tests for the core types module

use valence_domain_clients::core::types::{
    GenericAddress, GenericHash, GenericU256,
};

#[test]
fn test_generic_address() {
    let addr = GenericAddress::from("0x1234567890abcdef".to_string());
    assert_eq!(addr.0, "0x1234567890abcdef");

    let addr_str = "0x1234567890abcdef".to_string();
    let addr2 = GenericAddress::from(addr_str);
    assert_eq!(addr2.0, "0x1234567890abcdef");

    let addr_display = addr.to_string();
    assert_eq!(addr_display, "0x1234567890abcdef");

    // Test Clone
    let address_clone = addr.clone();
    assert_eq!(addr, address_clone);

    // Test Debug formatting
    let debug_str = format!("{:?}", addr);
    assert!(debug_str.contains("GenericAddress"));
    assert!(debug_str.contains("0x1234567890abcdef"));

    // Test PartialEq
    let same_address = GenericAddress::from("0x1234567890abcdef".to_string());
    let different_address = GenericAddress::from("0xabcdef1234567890".to_string());
    assert_eq!(addr, same_address);
    assert_ne!(addr, different_address);

    // Test AsRef<str>
    let address_ref: &str = addr.as_ref();
    assert_eq!(address_ref, "0x1234567890abcdef");
}

#[test]
fn test_generic_hash() {
    let hash = GenericHash::from("0xabcdef1234567890".to_string());
    assert_eq!(hash.0, "0xabcdef1234567890");

    let hash_str = "0xabcdef1234567890".to_string();
    let hash2 = GenericHash::from(hash_str);
    assert_eq!(hash2.0, "0xabcdef1234567890");

    let hash_display = hash.to_string();
    assert_eq!(hash_display, "0xabcdef1234567890");

    // Test Clone
    let hash_clone = hash.clone();
    assert_eq!(hash, hash_clone);

    // Test Debug formatting
    let debug_str = format!("{:?}", hash);
    assert!(debug_str.contains("GenericHash"));
    assert!(debug_str.contains("0xabcdef1234567890"));

    // Test PartialEq
    let same_hash = GenericHash::from("0xabcdef1234567890".to_string());
    let different_hash = GenericHash::from("0x1234567890abcdef".to_string());
    assert_eq!(hash, same_hash);
    assert_ne!(hash, different_hash);

    // Test AsRef<str>
    let hash_ref: &str = hash.as_ref();
    assert_eq!(hash_ref, "0xabcdef1234567890");
}

#[test]
fn test_generic_u256() {
    let u256 = GenericU256::from("123456789".to_string());
    assert_eq!(u256.0, "123456789");

    let u256_str = "123456789".to_string();
    let u256_2 = GenericU256::from(u256_str);
    assert_eq!(u256_2.0, "123456789");

    let u256_display = u256.to_string();
    assert_eq!(u256_display, "123456789");

    // Test Clone
    let u256_clone = u256.clone();
    assert_eq!(u256, u256_clone);

    // Test Debug formatting
    let debug_str = format!("{:?}", u256);
    assert!(debug_str.contains("GenericU256"));
    assert!(debug_str.contains("123456789"));

    // Test PartialEq
    let same_u256 = GenericU256::from("123456789".to_string());
    let different_u256 = GenericU256::from("234567890".to_string());
    assert_eq!(u256, same_u256);
    assert_ne!(u256, different_u256);

    // Test AsRef<str>
    let u256_ref: &str = u256.as_ref();
    assert_eq!(u256_ref, "123456789");
}
