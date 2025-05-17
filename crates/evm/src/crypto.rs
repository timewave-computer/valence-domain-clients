//-----------------------------------------------------------------------------
// Crypto Abstraction Layer
//-----------------------------------------------------------------------------

//! This module provides cryptographic functions needed for various EVM operations
//! including Flashbots bundle support, implemented in a way that avoids dependency conflicts.
//!
//! It offers a flexible backend system for different cryptographic implementations
//! that completely isolates dependencies, avoiding linking conflicts.

use crate::types::EvmBytes;
use hex;
use libsecp256k1::{recover, sign, Message, PublicKey, RecoveryId, SecretKey};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Once;
use tiny_keccak::{Hasher, Keccak};
use valence_core::error::ClientError;

/// Trait defining the cryptographic operations required
pub trait CryptoBackend {
    /// Compute Keccak-256 hash of data
    fn keccak256(&self, data: &[u8]) -> Vec<u8>;

    /// Sign a message using a private key
    /// Returns the signature bytes (65 bytes including recovery id)
    #[allow(dead_code)]
    fn sign_message(
        &self,
        private_key: &[u8; 32],
        message: &[u8],
    ) -> Result<Vec<u8>, ClientError>;

    /// Get an Ethereum address from a private key
    #[allow(dead_code)]
    fn address_from_private_key(
        &self,
        private_key: &[u8; 32],
    ) -> Result<[u8; 20], ClientError>;
}

/// Default implementation using pure Rust dependencies (tiny-keccak + libsecp256k1)
#[derive(Default)]
pub struct DefaultCryptoBackend;

impl CryptoBackend for DefaultCryptoBackend {
    fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(data);
        hasher.finalize(&mut output);
        output.to_vec()
    }

    fn sign_message(
        &self,
        private_key: &[u8; 32],
        message: &[u8],
    ) -> Result<Vec<u8>, ClientError> {
        let secret_key = SecretKey::parse(private_key).map_err(|_| {
            ClientError::ClientError("Invalid private key".to_string())
        })?;

        let message_hash = self.keccak256(message);
        let msg = Message::parse_slice(&message_hash).map_err(|_| {
            ClientError::ClientError("Invalid message hash".to_string())
        })?;

        let (signature, recovery_id) = sign(&msg, &secret_key);

        let mut sig_bytes = Vec::with_capacity(65);
        sig_bytes.extend_from_slice(&signature.r.b32());
        sig_bytes.extend_from_slice(&signature.s.b32());
        sig_bytes.push(recovery_id.serialize());

        Ok(sig_bytes)
    }

    fn address_from_private_key(
        &self,
        private_key: &[u8; 32],
    ) -> Result<[u8; 20], ClientError> {
        let secret_key = SecretKey::parse(private_key).map_err(|_| {
            ClientError::ClientError("Invalid private key".to_string())
        })?;

        let public_key = PublicKey::from_secret_key(&secret_key);
        let serialized = public_key.serialize();

        let hash = self.keccak256(&serialized[1..]);

        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..32]);

        Ok(address)
    }
}

// Runtime backend selection system
static BACKEND_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = DefaultCryptoBackend
static INIT: Once = Once::new();

/// Backend type: 0 = DefaultCryptoBackend, 1 = EthersCryptoBackend
pub enum BackendType {
    Default = 0,
    Ethers = 1,
}

/// Set the crypto backend type (0 = Default, 1 = Ethers if available)
/// Returns true if backend was set successfully, false if the requested backend is not available
pub fn set_crypto_backend(backend_type: BackendType) -> bool {
    let backend_id = backend_type as u8;

    // Only allow initialization once
    let mut success = true;
    INIT.call_once(|| {
        // Check if ethers backend was requested but not available
        if backend_id == 1 && !has_ethers_support() {
            success = false;
            return;
        }
        BACKEND_TYPE.store(backend_id, Ordering::SeqCst);
    });
    success
}

/// Get current backend type
#[allow(dead_code)]
pub fn get_crypto_backend_type() -> BackendType {
    match BACKEND_TYPE.load(Ordering::SeqCst) {
        1 => BackendType::Ethers,
        _ => BackendType::Default,
    }
}

/// Check if ethers support is available
/// This is determined at runtime by trying to dynamically load the ethers backend
pub fn has_ethers_support() -> bool {
    // First, check if we're compiled with ethers support
    let compiled_with_ethers = cfg!(feature = "_ethers_backend");

    if !compiled_with_ethers {
        return false;
    }

    // Then confirm we can actually use it by trying to create the backend
    #[cfg(feature = "_ethers_backend")]
    {
        match std::panic::catch_unwind(|| {
            // Try to create an instance of the ethers backend
            let _test_backend = ethers_backend::EthersCryptoBackend;
            true
        }) {
            Ok(true) => true,
            _ => false,
        }
    }

    #[cfg(not(feature = "_ethers_backend"))]
    {
        false
    }
}

/// Initialize the crypto backend system
/// This will automatically choose the best available backend
pub fn initialize_crypto_backend() {
    // Try to use ethers backend first if available
    if has_ethers_support() {
        let _ = set_crypto_backend(BackendType::Ethers);
    } else {
        let _ = set_crypto_backend(BackendType::Default);
    }
}

// Private function to get the appropriate backend
fn get_crypto_backend_impl() -> Box<dyn CryptoBackend> {
    match BACKEND_TYPE.load(Ordering::SeqCst) {
        1 if has_ethers_support() => {
            // This block is compiled only when "_ethers_backend" feature is enabled
            #[cfg(feature = "_ethers_backend")]
            {
                Box::new(ethers_backend::EthersCryptoBackend)
            }

            // This block is compiled when "_ethers_backend" feature is disabled
            #[cfg(not(feature = "_ethers_backend"))]
            {
                Box::new(DefaultCryptoBackend::default())
            }
        }
        _ => Box::new(DefaultCryptoBackend),
    }
}

// Ethers backend implementation - only compiled when feature is enabled
#[cfg(feature = "_ethers_backend")]
mod ethers_backend {
    use super::*;
    use ethers::core::k256::ecdsa::signature::Signer;
    use ethers::core::k256::ecdsa::{Signature, SigningKey};
    use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers::utils::keccak256 as ethers_keccak256;

    #[derive(Default)]
    pub struct EthersCryptoBackend;

    impl CryptoBackend for EthersCryptoBackend {
        fn keccak256(&self, data: &[u8]) -> Vec<u8> {
            ethers_keccak256(data).to_vec()
        }

        fn sign_message(
            &self,
            private_key: &[u8; 32],
            message: &[u8],
        ) -> Result<Vec<u8>, ClientError> {
            // Create signing key from private key bytes
            let signing_key =
                SigningKey::from_bytes(private_key.into()).map_err(|e| {
                    ClientError::ClientError(format!("Invalid private key: {e}"))
                })?;

            // Sign the message
            let signature: Signature = signing_key.sign(message);
            let mut signature_bytes = signature.to_bytes().to_vec();

            // Add recovery ID byte (always 0 or 1)
            signature_bytes.push(0); // Simplified, should be calculated properly in production

            Ok(signature_bytes)
        }

        fn address_from_private_key(
            &self,
            private_key: &[u8; 32],
        ) -> Result<[u8; 20], ClientError> {
            use ethers::core::k256::SecretKey;

            // Create secret key from private key bytes
            let secret_key =
                SecretKey::from_bytes(private_key.into()).map_err(|e| {
                    ClientError::ClientError(format!("Invalid private key: {e}"))
                })?;

            // Get the public key
            let public_key = secret_key.public_key();

            // Get encoded public key bytes
            let encoded_point = public_key.to_encoded_point(false);
            let public_key_bytes = encoded_point.as_bytes();

            // Hash the public key bytes (skip first byte which is format byte)
            let hash = ethers_keccak256(&public_key_bytes[1..]);

            // Get the address (last 20 bytes of the hash)
            let mut address = [0u8; 20];
            address.copy_from_slice(&hash[12..32]);

            Ok(address)
        }
    }
}

// Initialize the crypto backend automatically when the module is loaded
static GLOBAL_INIT: Once = Once::new();

// Public API functions - these are the stable interface regardless of which backend is used

/// Compute Keccak-256 hash of data
pub fn keccak256(data: &[u8]) -> EvmBytes {
    // Ensure the backend is initialized
    GLOBAL_INIT.call_once(|| {
        initialize_crypto_backend();
    });

    let backend = get_crypto_backend_impl();
    EvmBytes(backend.keccak256(data))
}

/// Sign a message using a private key
/// Returns the signature bytes and recovery ID
#[allow(dead_code)]
pub fn sign_message(
    private_key: &[u8; 32],
    message: &[u8],
) -> Result<(Vec<u8>, u8), ClientError> {
    // Ensure the backend is initialized
    GLOBAL_INIT.call_once(|| {
        initialize_crypto_backend();
    });

    let backend = get_crypto_backend_impl();
    let signature = backend.sign_message(private_key, message)?;

    // Extract signature components
    let recovery_id = signature[64];
    let signature_bytes = signature[0..64].to_vec();

    Ok((signature_bytes, recovery_id))
}

/// Get Ethereum address from private key
#[allow(dead_code)]
pub fn get_address_from_private_key(
    private_key: &[u8; 32],
) -> Result<[u8; 20], ClientError> {
    // Ensure the backend is initialized
    GLOBAL_INIT.call_once(|| {
        initialize_crypto_backend();
    });

    let backend = get_crypto_backend_impl();
    backend.address_from_private_key(private_key)
}

/// Compute the Keccak256 hash of data
pub fn keccak256_core(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

/// Sign a message with a private key
/// Returns the 65-byte signature (r, s, v) as bytes
/// Where v is the recovery ID + 27
pub fn sign_message_core(
    message: &[u8],
    private_key: &[u8],
) -> Result<Vec<u8>, ClientError> {
    let message_hash = keccak256_core(message);
    let secret_key = match SecretKey::parse_slice(private_key) {
        Ok(key) => key,
        Err(e) => {
            return Err(ClientError::ClientError(format!(
                "Invalid private key: {e}"
            )))
        }
    };

    let msg = Message::parse_slice(&message_hash).map_err(|e| {
        ClientError::ClientError(format!("Invalid message hash: {e}"))
    })?;

    let (signature, recovery_id) = sign(&msg, &secret_key);

    // Convert to 65-byte format (r, s, v)
    let mut sig_bytes = [0u8; 65];
    sig_bytes[..32].copy_from_slice(&signature.r.b32());
    sig_bytes[32..64].copy_from_slice(&signature.s.b32());
    sig_bytes[64] = recovery_id.serialize() + 27; // Add 27 for Ethereum compatibility

    Ok(sig_bytes.to_vec())
}

/// Recover the public key from a signature and message
pub fn recover_public_key_core(
    message: &[u8],
    signature: &[u8],
) -> Result<Vec<u8>, ClientError> {
    if signature.len() != 65 {
        return Err(ClientError::ClientError(
            "Invalid signature length".to_string(),
        ));
    }

    let message_hash = keccak256_core(message);
    let msg = Message::parse_slice(&message_hash).map_err(|e| {
        ClientError::ClientError(format!("Invalid message hash: {e}"))
    })?;

    let recovery_id = (signature[64] - 27) & 0x03;
    let recovery_id = RecoveryId::parse(recovery_id).map_err(|e| {
        ClientError::ClientError(format!("Invalid recovery ID: {e}"))
    })?;

    let r = &signature[0..32];
    let s = &signature[32..64];

    let signature = libsecp256k1::Signature::parse_standard_slice(&[r, s].concat())
        .map_err(|e| ClientError::ClientError(format!("Invalid signature: {e}")))?;

    let public_key = recover(&msg, &signature, &recovery_id).map_err(|e| {
        ClientError::ClientError(format!("Could not recover public key: {e}"))
    })?;

    let public_key_serialized = public_key.serialize();
    Ok(public_key_serialized.to_vec())
}

/// Get the Ethereum address from a private key
pub fn get_address_from_private_key_core(
    private_key: &[u8],
) -> Result<[u8; 20], ClientError> {
    let secret_key = match SecretKey::parse_slice(private_key) {
        Ok(key) => key,
        Err(e) => {
            return Err(ClientError::ClientError(format!(
                "Invalid private key: {e}"
            )))
        }
    };

    let public_key = PublicKey::from_secret_key(&secret_key);
    let public_key_serialized = public_key.serialize();

    // Remove the first byte (0x04 prefix) and hash the rest
    let public_key_hash = keccak256_core(&public_key_serialized[1..]);

    // Take the last 20 bytes
    let mut address = [0u8; 20];
    address.copy_from_slice(&public_key_hash[12..32]);

    Ok(address)
}

/// Get the Ethereum address from a public key
pub fn get_address_from_public_key_core(
    public_key: &[u8],
) -> Result<[u8; 20], ClientError> {
    if public_key.len() != 65 && public_key.len() != 64 {
        return Err(ClientError::ClientError(format!(
            "Invalid public key length: {} (expected 64 or 65)",
            public_key.len()
        )));
    }

    let key_to_hash = if public_key.len() == 65 {
        // Skip the first byte (0x04 prefix) if present
        &public_key[1..]
    } else {
        public_key
    };

    let public_key_hash = keccak256_core(key_to_hash);

    // Take the last 20 bytes
    let mut address = [0u8; 20];
    address.copy_from_slice(&public_key_hash[12..32]);

    Ok(address)
}

/// Convert a hex string to bytes
pub fn hex_to_bytes_core(hex_str: &str) -> Result<Vec<u8>, ClientError> {
    let clean_hex = if let Some(stripped) = hex_str.strip_prefix("0x") {
        stripped
    } else {
        hex_str
    };

    hex::decode(clean_hex)
        .map_err(|e| ClientError::ClientError(format!("Invalid hex string: {e}")))
}

/// Convert bytes to a hex string
pub fn bytes_to_hex_core(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}
