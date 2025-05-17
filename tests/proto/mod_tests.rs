//-----------------------------------------------------------------------------
// Protocol Buffer Module Tests
//-----------------------------------------------------------------------------

// Import the ProtoError type for testing
use valence_domain_clients::proto::utils::ProtoError;

// Define a simple test to validate imports are working
#[test]
fn test_proto_error_creation() {
    // Create a ProtoError::EncodeError
    let encode_error = ProtoError::EncodeError("Test encode error".to_string());

    // Create a ProtoError::DecodeError
    let decode_error = ProtoError::DecodeError("Test decode error".to_string());

    // Convert to string and verify error messages
    assert!(encode_error.to_string().contains("encode"));
    assert!(decode_error.to_string().contains("decode"));
}
