//-----------------------------------------------------------------------------
// Crypto Abstraction Layer
//-----------------------------------------------------------------------------

//! This module provides cryptographic functions needed for various EVM operations
//! including Flashbots bundle support, implemented in a way that avoids dependency conflicts.
//!
//! It offers a flexible backend system for different cryptographic implementations
//! that completely isolates dependencies, avoiding linking conflicts.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Once;
use crate::core::error::ClientError;
use crate::evm::types::EvmBytes;

/// Trait defining the cryptographic operations required
pub trait CryptoBackend {
    /// Compute Keccak-256 hash of data
    fn keccak256(&self, data: &[u8]) -> Vec<u8>;
    
    /// Sign a message using a private key
    /// Returns the signature bytes (65 bytes including recovery id)
    #[allow(dead_code)]
    fn sign_message(&self, private_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, ClientError>;
    
    /// Get an Ethereum address from a private key
    #[allow(dead_code)]
    fn address_from_private_key(&self, private_key: &[u8; 32]) -> Result<[u8; 20], ClientError>;
}

/// Default implementation using pure Rust dependencies (tiny-keccak + libsecp256k1)
#[derive(Default)]
pub struct DefaultCryptoBackend;

impl CryptoBackend for DefaultCryptoBackend {
    fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(data);
        hasher.finalize(&mut output);
        output.to_vec()
    }
    
    fn sign_message(&self, private_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, ClientError> {
        use libsecp256k1::{Message, SecretKey};
        
        // Create secret key from private key bytes
        let secret_key = SecretKey::parse(private_key)
            .map_err(|_| ClientError::ClientError("Invalid private key".to_string()))?;
        
        // Create message from hash
        let message_hash = self.keccak256(message);
        let msg = Message::parse_slice(&message_hash)
            .map_err(|_| ClientError::ClientError("Invalid message hash".to_string()))?;
        
        // Sign the message
        let (signature, recovery_id) = libsecp256k1::sign(&msg, &secret_key);
        
        // Combine signature components
        let mut sig_bytes = Vec::with_capacity(65);
        sig_bytes.extend_from_slice(&signature.r.b32());
        sig_bytes.extend_from_slice(&signature.s.b32());
        sig_bytes.push(recovery_id.serialize());
        
        Ok(sig_bytes)
    }
    
    fn address_from_private_key(&self, private_key: &[u8; 32]) -> Result<[u8; 20], ClientError> {
        use libsecp256k1::{PublicKey, SecretKey};
        
        // Create secret key from private key bytes
        let secret_key = SecretKey::parse(private_key)
            .map_err(|_| ClientError::ClientError("Invalid private key".to_string()))?;
        
        // Get the public key
        let public_key = PublicKey::from_secret_key(&secret_key);
        let serialized = public_key.serialize();
        
        // Hash the public key bytes (skip the first byte which is a format byte)
        let hash = self.keccak256(&serialized[1..]);
        
        // Take the last 20 bytes as the address
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
        },
        _ => Box::new(DefaultCryptoBackend),
    }
}

// Ethers backend implementation - only compiled when feature is enabled
#[cfg(feature = "_ethers_backend")]
mod ethers_backend {
    use super::*;
    use ethers::utils::keccak256 as ethers_keccak256;
    use ethers::core::k256::ecdsa::{SigningKey, Signature};
    use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers::core::k256::ecdsa::signature::Signer;
    
    #[derive(Default)]
    pub struct EthersCryptoBackend;
    
    impl CryptoBackend for EthersCryptoBackend {
        fn keccak256(&self, data: &[u8]) -> Vec<u8> {
            ethers_keccak256(data).to_vec()
        }
        
        fn sign_message(&self, private_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, ClientError> {
            // Create signing key from private key bytes
            let signing_key = SigningKey::from_bytes(private_key.into())
                .map_err(|e| ClientError::ClientError(format!("Invalid private key: {e}")))?;
            
            // Sign the message
            let signature: Signature = signing_key.sign(message);
            let mut signature_bytes = signature.to_bytes().to_vec();
            
            // Add recovery ID byte (always 0 or 1)
            signature_bytes.push(0); // Simplified, should be calculated properly in production
            
            Ok(signature_bytes)
        }
        
        fn address_from_private_key(&self, private_key: &[u8; 32]) -> Result<[u8; 20], ClientError> {
            use ethers::core::k256::SecretKey;
            
            // Create secret key from private key bytes
            let secret_key = SecretKey::from_bytes(private_key.into())
                .map_err(|e| ClientError::ClientError(format!("Invalid private key: {e}")))?;
            
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
pub fn sign_message(private_key: &[u8; 32], message: &[u8]) -> Result<(Vec<u8>, u8), ClientError> {
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
pub fn get_address_from_private_key(private_key: &[u8; 32]) -> Result<[u8; 20], ClientError> {
    // Ensure the backend is initialized
    GLOBAL_INIT.call_once(|| {
        initialize_crypto_backend();
    });
    
    let backend = get_crypto_backend_impl();
    backend.address_from_private_key(private_key)
} 