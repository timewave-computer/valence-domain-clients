//-----------------------------------------------------------------------------
// Crypto Adapter - Complete Type Isolation
//-----------------------------------------------------------------------------

//! This module provides a complete type isolation layer for cryptographic operations.
//! 
//! It ensures that no external types from potentially conflicting dependencies
//! are ever exposed in the public API or leak across crate boundaries.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Once;

use crate::core::error::ClientError;
use crate::evm::types::EvmBytes;

// Backend types
const BACKEND_DEFAULT: u8 = 0;
const BACKEND_ETHERS: u8 = 1;

// Singleton state management
static CURRENT_BACKEND: AtomicU8 = AtomicU8::new(BACKEND_DEFAULT);
static INIT: Once = Once::new();

//-----------------------------------------------------------------------------
// Crypto Backend Trait - Full Isolation
//-----------------------------------------------------------------------------

/// The core trait that all crypto implementations must implement
trait CryptoBackend: Send + Sync {
    /// Compute Keccak-256 hash of data
    fn keccak256(&self, data: &[u8]) -> Vec<u8>;
    
    /// Sign a message using a private key
    /// Returns the signature bytes without recovery ID
    fn sign_message(&self, private_key: &[u8; 32], message: &[u8]) -> Result<(Vec<u8>, u8), ClientError>;
    
    /// Compute Ethereum address from private key
    fn address_from_private_key(&self, private_key: &[u8; 32]) -> Result<[u8; 20], ClientError>;
}

//-----------------------------------------------------------------------------
// Default Implementation (tiny-keccak + libsecp256k1)
//-----------------------------------------------------------------------------

/// Provides cryptographic operations using tiny-keccak and libsecp256k1
struct DefaultCryptoBackend;

impl CryptoBackend for DefaultCryptoBackend {
    fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(data);
        hasher.finalize(&mut output);
        output.to_vec()
    }
    
    fn sign_message(&self, private_key: &[u8; 32], message: &[u8]) -> Result<(Vec<u8>, u8), ClientError> {
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
        
        // Extract components
        let mut sig_bytes = Vec::with_capacity(64);
        sig_bytes.extend_from_slice(&signature.r.b32());
        sig_bytes.extend_from_slice(&signature.s.b32());
        
        Ok((sig_bytes, recovery_id.serialize()))
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

//-----------------------------------------------------------------------------
// Ethers Backend Implementation
//-----------------------------------------------------------------------------

// The backend is split into a module that is conditionally compiled
#[cfg(feature = "_ethers_backend")]
mod ethers_backend {
    use ethers::utils::keccak256 as ethers_keccak256;
    use ethers::core::k256::ecdsa::{SigningKey, Signature};
    use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers::core::k256::ecdsa::signature::Signer;
    use super::*;
    
    pub struct EthersCryptoBackend;
    
    impl CryptoBackend for EthersCryptoBackend {
        fn keccak256(&self, data: &[u8]) -> Vec<u8> {
            ethers_keccak256(data).to_vec()
        }
        
        fn sign_message(&self, private_key: &[u8; 32], message: &[u8]) -> Result<(Vec<u8>, u8), ClientError> {
            // Create signing key from private key bytes
            let signing_key = SigningKey::from_bytes(private_key.into())
                .map_err(|e| ClientError::ClientError(format!("Invalid private key: {}", e)))?;
            
            // Sign the message
            let signature: Signature = signing_key.sign(message);
            let signature_bytes = signature.to_bytes().to_vec();
            
            // Return signature and recovery ID
            Ok((signature_bytes, 0)) // Simplified, would need proper recovery ID calculation
        }
        
        fn address_from_private_key(&self, private_key: &[u8; 32]) -> Result<[u8; 20], ClientError> {
            use ethers::core::k256::SecretKey;
            
            // Create secret key from private key bytes
            let secret_key = SecretKey::from_bytes(private_key.into())
                .map_err(|e| ClientError::ClientError(format!("Invalid private key: {}", e)))?;
            
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
    
    /// Try to create an ethers backend instance to check if it's available
    pub fn create_ethers_backend() -> Option<Box<dyn CryptoBackend>> {
        match std::panic::catch_unwind(|| {
            Box::new(EthersCryptoBackend) as Box<dyn CryptoBackend>
        }) {
            Ok(backend) => Some(backend),
            Err(_) => None,
        }
    }
}

// Stub module when ethers is not available
#[cfg(not(feature = "_ethers_backend"))]
mod ethers_backend {
    use super::*;
    
    /// Always returns None when ethers backend is not available
    pub fn create_ethers_backend() -> Option<Box<dyn CryptoBackend>> {
        None
    }
}

//-----------------------------------------------------------------------------
// Backend Factory - Runtime Backend Selection
//-----------------------------------------------------------------------------

/// Create a new backend of the requested type, or fallback to default
fn create_backend(backend_type: u8) -> Box<dyn CryptoBackend> {
    match backend_type {
        BACKEND_ETHERS => {
            // Try to create ethers backend first
            match ethers_backend::create_ethers_backend() {
                Some(backend) => return backend,
                None => {
                    // Fallback to default if ethers backend failed
                    CURRENT_BACKEND.store(BACKEND_DEFAULT, Ordering::SeqCst);
                }
            }
        }
        _ => {}
    }
    
    // Default backend as fallback
    Box::new(DefaultCryptoBackend)
}

/// Global singleton instance of the crypto backend
static mut BACKEND_INSTANCE: Option<Box<dyn CryptoBackend>> = None;

/// Initialize the crypto backend system
fn initialize() {
    INIT.call_once(|| {
        // First try ethers backend if the feature is enabled
        #[cfg(feature = "_ethers_backend")]
        let backend_type = BACKEND_ETHERS;
        
        // Otherwise use default
        #[cfg(not(feature = "_ethers_backend"))]
        let backend_type = BACKEND_DEFAULT;
        
        // Create the backend
        let backend = create_backend(backend_type);
        
        // Store the backend type
        CURRENT_BACKEND.store(backend_type, Ordering::SeqCst);
        
        // Store the backend instance
        unsafe {
            BACKEND_INSTANCE = Some(backend);
        }
    });
}

/// Get the global backend instance
fn get_backend() -> &'static dyn CryptoBackend {
    // Ensure initialization
    initialize();
    
    unsafe {
        // This is safe because:
        // 1. We only call this after initialize() has been called
        // 2. The static is initialized exactly once by the INIT Once guard
        // 3. We never modify BACKEND_INSTANCE after initialization
        BACKEND_INSTANCE.as_ref().unwrap().as_ref()
    }
}

//-----------------------------------------------------------------------------
// Public API - Fully Isolated Interface
//-----------------------------------------------------------------------------

/// Compute Keccak-256 hash of data
pub fn keccak256(data: &[u8]) -> EvmBytes {
    let result = get_backend().keccak256(data);
    EvmBytes(result)
}

/// Sign a message with a private key
pub fn sign_message(private_key: &[u8; 32], message: &[u8]) -> Result<(Vec<u8>, u8), ClientError> {
    get_backend().sign_message(private_key, message)
}

/// Get Ethereum address from private key
pub fn address_from_private_key(private_key: &[u8; 32]) -> Result<[u8; 20], ClientError> {
    get_backend().address_from_private_key(private_key)
}

/// Check if the ethers backend is available and working
pub fn has_ethers_backend() -> bool {
    // Ensure the system is initialized
    initialize();
    
    // Check what backend was actually loaded
    CURRENT_BACKEND.load(Ordering::SeqCst) == BACKEND_ETHERS
}

/// Get the currently active backend type
pub fn get_active_backend_type() -> &'static str {
    match CURRENT_BACKEND.load(Ordering::SeqCst) {
        BACKEND_ETHERS => "ethers",
        _ => "default",
    }
} 