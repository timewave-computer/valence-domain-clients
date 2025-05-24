//-----------------------------------------------------------------------------
// Protocol Buffer Utilities
//-----------------------------------------------------------------------------

//! Utility functions for working with Protocol Buffers.
//!
//! This module provides common functionality for encoding and decoding
//! protocol buffer messages while shielding consumers from direct dependency
//! on specific protocol buffer implementation details.
//!
//! # Dependency Isolation
//!
//! This module uses a trait-based approach to isolate consumers from direct
//! dependencies on protocol buffer libraries like `prost`. Instead of directly
//! exposing the `prost::Message` trait in the public API, we define our own
//! traits (`ProtoEncodable` and `ProtoDecodable`) and implement them for types
//! that satisfy the protocol buffer requirements.
//!
//! This approach has several benefits:
//!
//! 1. Consumers of this library don't need to know which protocol buffer
//!    implementation we use internally
//! 2. If consumers use a different version of the same protocol buffer library,
//!    they won't experience trait implementation conflicts
//! 3. We can change the underlying protocol buffer implementation without
//!    breaking the public API

use prost::Message;
use thiserror::Error;

/// Errors that can occur during protocol buffer operations
#[derive(Error, Debug)]
pub enum ProtoError {
    /// Encoding error
    #[error("Failed to encode protocol buffer: {0}")]
    EncodeError(String),

    /// Decoding error
    #[error("Failed to decode protocol buffer: {0}")]
    DecodeError(String),
}

/// A trait for types that can be encoded to bytes
pub trait ProtoEncodable {
    /// Encode this type to bytes
    fn encode_to_bytes(&self) -> Result<Vec<u8>, ProtoError>;
}

/// A trait for types that can be decoded from bytes
pub trait ProtoDecodable: Sized {
    /// Decode this type from bytes
    fn decode_from_bytes(bytes: &[u8]) -> Result<Self, ProtoError>;
}

// Implement ProtoEncodable for any type that implements prost::Message
impl<T: Message> ProtoEncodable for T {
    fn encode_to_bytes(&self) -> Result<Vec<u8>, ProtoError> {
        let mut buf = Vec::new();
        self.encode(&mut buf)
            .map_err(|e| ProtoError::EncodeError(e.to_string()))?;
        Ok(buf)
    }
}

// Implement ProtoDecodable for any type that implements prost::Message + Default
impl<T: Message + Default> ProtoDecodable for T {
    fn decode_from_bytes(bytes: &[u8]) -> Result<Self, ProtoError> {
        T::decode(bytes).map_err(|e| ProtoError::DecodeError(e.to_string()))
    }
}

/// Encode a type that implements ProtoEncodable to bytes
pub fn encode<T: ProtoEncodable>(message: &T) -> Result<Vec<u8>, ProtoError> {
    message.encode_to_bytes()
}

/// Decode bytes to a type that implements ProtoDecodable
pub fn decode<T: ProtoDecodable>(bytes: &[u8]) -> Result<T, ProtoError> {
    T::decode_from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock message that implements the required traits
    struct TestMessage {
        pub data: Vec<u8>,
    }

    impl ProtoEncodable for TestMessage {
        fn encode_to_bytes(&self) -> Result<Vec<u8>, ProtoError> {
            Ok(self.data.clone())
        }
    }

    impl ProtoDecodable for TestMessage {
        fn decode_from_bytes(bytes: &[u8]) -> Result<Self, ProtoError> {
            Ok(TestMessage {
                data: bytes.to_vec(),
            })
        }
    }

    #[test]
    fn test_encode_decode() {
        // Create a test message
        let test_msg = TestMessage {
            data: b"test data".to_vec(),
        };

        // Encode the message
        let encoded = encode(&test_msg).expect("Failed to encode message");

        // Decode the message
        let decoded: TestMessage = decode(&encoded).expect("Failed to decode message");

        // Verify that the decoded message matches the original
        assert_eq!(decoded.data, b"test data".to_vec());
    }
}
