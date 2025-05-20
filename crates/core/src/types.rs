//-----------------------------------------------------------------------------
// Core Type Definitions
//-----------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

//-----------------------------------------------------------------------------
// Generic Address Types
//-----------------------------------------------------------------------------

/// Generic blockchain address representation.
///
/// This type provides a chain-agnostic way to represent addresses across
/// different blockchain ecosystems. Specific modules (Cosmos, EVM) have
/// their own more detailed address types that can convert to/from this.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericAddress(pub String);

impl From<&str> for GenericAddress {
    fn from(s: &str) -> Self {
        GenericAddress(s.to_string())
    }
}

impl AsRef<str> for GenericAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for GenericAddress {
    fn from(s: String) -> Self {
        GenericAddress(s)
    }
}

impl std::fmt::Display for GenericAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//-----------------------------------------------------------------------------
// Generic Hash Types
//-----------------------------------------------------------------------------

/// Generic blockchain hash representation.
///
/// This type provides a chain-agnostic way to represent cryptographic hashes
/// (block hashes, transaction hashes, etc.) across different blockchain ecosystems.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericHash(pub String);

impl From<&str> for GenericHash {
    fn from(s: &str) -> Self {
        GenericHash(s.to_string())
    }
}

impl AsRef<str> for GenericHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for GenericHash {
    fn from(s: String) -> Self {
        GenericHash(s)
    }
}

impl std::fmt::Display for GenericHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//-----------------------------------------------------------------------------
// Numeric Types
//-----------------------------------------------------------------------------

/// Generic 256-bit unsigned integer representation.
///
/// Stored as a string to avoid specific library dependencies here.
/// Actual U256 operations are handled within the respective EVM/Cosmos modules
/// using their native U256 types, converting to/from this string representation
/// at the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericU256(pub String);

impl From<&str> for GenericU256 {
    fn from(s: &str) -> Self {
        GenericU256(s.to_string())
    }
}

impl AsRef<str> for GenericU256 {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for GenericU256 {
    fn from(s: String) -> Self {
        GenericU256(s)
    }
}

impl std::fmt::Display for GenericU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
    fn test_generic_address() {
        // Test creation from &str
        let addr_str = "cosmos1abcdef";
        let addr = GenericAddress::from(addr_str);
        assert_eq!(addr.0, addr_str);

        // Test creation from String
        let addr_string = String::from("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
        let addr = GenericAddress::from(addr_string.clone());
        assert_eq!(addr.0, addr_string);

        // Test AsRef<str>
        let addr = GenericAddress::from("test_address");
        let addr_ref: &str = addr.as_ref();
        assert_eq!(addr_ref, "test_address");

        // Test Display
        let addr = GenericAddress::from("display_test");
        assert_eq!(format!("{}", addr), "display_test");

        // Test serialization/deserialization
        let addr = GenericAddress::from("ser_test");
        let json = serde_json::to_string(&addr).unwrap();
        let deserialized: GenericAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, deserialized);
    }

    #[test]
    fn test_generic_hash() {
        // Test creation from &str
        let hash_str =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let hash = GenericHash::from(hash_str);
        assert_eq!(hash.0, hash_str);

        // Test creation from String
        let hash_string =
            String::from("A36BAB31FEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE");
        let hash = GenericHash::from(hash_string.clone());
        assert_eq!(hash.0, hash_string);

        // Test AsRef<str>
        let hash = GenericHash::from("test_hash");
        let hash_ref: &str = hash.as_ref();
        assert_eq!(hash_ref, "test_hash");

        // Test Display
        let hash = GenericHash::from("display_test");
        assert_eq!(format!("{}", hash), "display_test");

        // Test serialization/deserialization
        let hash = GenericHash::from("ser_test");
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: GenericHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, deserialized);
    }

    #[test]
    fn test_generic_u256() {
        // Test creation from &str
        let num_str = "123456789";
        let num = GenericU256::from(num_str);
        assert_eq!(num.0, num_str);

        // Test creation from String
        let num_string = String::from("1000000000000000000"); // 1 ETH in wei
        let num = GenericU256::from(num_string.clone());
        assert_eq!(num.0, num_string);

        // Test AsRef<str>
        let num = GenericU256::from("42");
        let num_ref: &str = num.as_ref();
        assert_eq!(num_ref, "42");

        // Test Display
        let num = GenericU256::from("987654321");
        assert_eq!(format!("{}", num), "987654321");

        // Test serialization/deserialization
        let num = GenericU256::from("115792089237316195423570985008687907853269984665640564039457584007913129639935"); // Max U256 value
        let json = serde_json::to_string(&num).unwrap();
        let deserialized: GenericU256 = serde_json::from_str(&json).unwrap();
        assert_eq!(num, deserialized);
    }
}
