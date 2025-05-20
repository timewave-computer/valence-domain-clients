//-----------------------------------------------------------------------------
// Crypto Adapter Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::crypto_adapter::{
        get_active_backend_type, has_ethers_backend, keccak256,
    };

    #[test]
    fn test_keccak256_hash() {
        let input = "hello world".as_bytes();
        let hash = keccak256(input);

        // Known keccak256 hash of "hello world"
        let expected_hex =
            "47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad";

        assert_eq!(hex::encode(&hash.0), expected_hex);
    }

    #[test]
    fn test_backend_selection() {
        // Should report which backend is active
        println!("Using backend: {}", get_active_backend_type());
        println!("Has ethers support: {}", has_ethers_backend());

        // Basic verification that functionality works regardless of backend
        let input = "test data".as_bytes();
        let hash = keccak256(input);
        assert_eq!(hash.0.len(), 32); // Keccak256 produces a 32-byte hash
    }
}
