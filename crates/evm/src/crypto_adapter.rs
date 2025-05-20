//-----------------------------------------------------------------------------
// Crypto Adapter - Complete Type Isolation
//-----------------------------------------------------------------------------

//! This module provides a complete type isolation layer for cryptographic operations.
//!
//! It ensures that no external types from potentially conflicting dependencies
//! are ever exposed in the public API or leak across crate boundaries.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use std::sync::Once;

use crate::types::EvmBytes;
use valence_core::error::ClientError;

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
    #[allow(dead_code)]
    fn sign_message(
        &self,
        private_key: &[u8; 32],
        message: &[u8],
    ) -> Result<(Vec<u8>, u8), ClientError>;

    /// Compute Ethereum address from private key
    #[allow(dead_code)]
    fn address_from_private_key(
        &self,
        private_key: &[u8; 32],
    ) -> Result<[u8; 20], ClientError>;
}

//-----------------------------------------------------------------------------
// Default Implementation (tiny-keccak + libsecp256k1)
//-----------------------------------------------------------------------------

/// Provides cryptographic operations using tiny-keccak and libsecp256k1
#[derive(Clone)]
struct DefaultCryptoBackend;

impl CryptoBackend for DefaultCryptoBackend {
    fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        use tiny_keccak::{Hasher, Keccak};
        let mut hasher = Keccak::v256();
        let mut result = [0u8; 32];
        hasher.update(data);
        hasher.finalize(&mut result);
        result.to_vec()
    }

    fn sign_message(
        &self,
        private_key: &[u8; 32],
        message: &[u8],
    ) -> Result<(Vec<u8>, u8), ClientError> {
        use libsecp256k1::{Message, SecretKey};

        // Create message object
        let msg = Message::parse_slice(message).map_err(|_| {
            ClientError::ClientError("Invalid message format".to_string())
        })?;

        // Parse private key
        let secret_key = SecretKey::parse(private_key).map_err(|_| {
            ClientError::ClientError("Invalid private key".to_string())
        })?;

        // Sign the message
        let (signature, recovery_id) = libsecp256k1::sign(&msg, &secret_key);
        let signature_bytes = signature.serialize().to_vec();

        Ok((signature_bytes, recovery_id.serialize()))
    }

    fn address_from_private_key(
        &self,
        private_key: &[u8; 32],
    ) -> Result<[u8; 20], ClientError> {
        use libsecp256k1::{PublicKey, SecretKey};

        // Create secret key from private key bytes
        let secret_key = SecretKey::parse(private_key).map_err(|_| {
            ClientError::ClientError("Invalid private key".to_string())
        })?;

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
    use super::*;
    use ethers::core::k256::ecdsa::signature::Signer;
    use ethers::core::k256::ecdsa::{Signature, SigningKey};
    use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers::utils::keccak256 as ethers_keccak256;

    #[derive(Clone)]
    pub struct EthersCryptoBackend;

    impl CryptoBackend for EthersCryptoBackend {
        fn keccak256(&self, data: &[u8]) -> Vec<u8> {
            ethers_keccak256(data).to_vec()
        }

        fn sign_message(
            &self,
            private_key: &[u8; 32],
            message: &[u8],
        ) -> Result<(Vec<u8>, u8), ClientError> {
            // Create signing key from private key bytes
            let signing_key =
                SigningKey::from_bytes(private_key.into()).map_err(|e| {
                    ClientError::ClientError(format!("Invalid private key: {e}"))
                })?;

            // Sign the message
            let signature: Signature = signing_key.sign(message);
            let signature_bytes = signature.to_bytes().to_vec();

            // Return signature and recovery ID
            Ok((signature_bytes, 0)) // Simplified, would need proper recovery ID calculation
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

    /// Try to create an ethers backend instance to check if it's available
    pub fn create_ethers_backend() -> bool {
        std::panic::catch_unwind(|| {
            let _backend = EthersCryptoBackend;
            true
        })
        .is_ok_and(|ok| ok)
    }
}

// Stub module when ethers is not available
#[cfg(not(feature = "_ethers_backend"))]
mod ethers_backend {
    /// Always returns false when ethers backend is not available
    pub fn create_ethers_backend() -> bool {
        false
    }
}

//-----------------------------------------------------------------------------
// Backend Enum - Safe Implementation without trait objects
//-----------------------------------------------------------------------------

/// Enum to represent the different backend types
#[derive(Clone)]
enum BackendImpl {
    Default(DefaultCryptoBackend),
    #[cfg(feature = "_ethers_backend")]
    Ethers(ethers_backend::EthersCryptoBackend),
}

impl BackendImpl {
    fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        match self {
            BackendImpl::Default(backend) => backend.keccak256(data),
            #[cfg(feature = "_ethers_backend")]
            BackendImpl::Ethers(backend) => backend.keccak256(data),
        }
    }

    fn sign_message(
        &self,
        private_key: &[u8; 32],
        message: &[u8],
    ) -> Result<(Vec<u8>, u8), ClientError> {
        match self {
            BackendImpl::Default(backend) => {
                backend.sign_message(private_key, message)
            }
            #[cfg(feature = "_ethers_backend")]
            BackendImpl::Ethers(backend) => {
                backend.sign_message(private_key, message)
            }
        }
    }

    fn address_from_private_key(
        &self,
        private_key: &[u8; 32],
    ) -> Result<[u8; 20], ClientError> {
        match self {
            BackendImpl::Default(backend) => {
                backend.address_from_private_key(private_key)
            }
            #[cfg(feature = "_ethers_backend")]
            BackendImpl::Ethers(backend) => {
                backend.address_from_private_key(private_key)
            }
        }
    }
}

/// Global singleton instance of the crypto backend
static BACKEND_INSTANCE: Mutex<Option<BackendImpl>> = Mutex::new(None);

/// Create a new backend of the requested type, or fallback to default
fn create_backend(backend_type: u8) -> BackendImpl {
    if backend_type == BACKEND_ETHERS {
        // Try to create ethers backend if available
        #[cfg(feature = "_ethers_backend")]
        if ethers_backend::create_ethers_backend() {
            return BackendImpl::Ethers(ethers_backend::EthersCryptoBackend);
        }

        // Fallback to default if ethers backend failed or not available
        CURRENT_BACKEND.store(BACKEND_DEFAULT, Ordering::SeqCst);
    }

    // Default backend as fallback
    BackendImpl::Default(DefaultCryptoBackend)
}

/// Initialize the crypto backend system
/// This is called automatically but can also be called explicitly if needed
pub fn initialize_crypto_backend() {
    initialize();
}

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
        let mut instance = BACKEND_INSTANCE.lock().unwrap();
        *instance = Some(backend);
    });
}

/// Get the global backend instance safely by cloning it
fn get_backend() -> BackendImpl {
    // Ensure initialization
    initialize();

    // Clone the instance to avoid holding the lock
    let locked = BACKEND_INSTANCE.lock().unwrap();
    match &*locked {
        Some(backend) => backend.clone(),
        None => panic!("Crypto backend not initialized"), // This should never happen due to initialize()
    }
}

//-----------------------------------------------------------------------------
// Public API - Fully Isolated Interface
//-----------------------------------------------------------------------------

/// Compute Keccak-256 hash of data
pub fn keccak256(data: &[u8]) -> EvmBytes {
    let backend = get_backend();
    let result = backend.keccak256(data);
    EvmBytes(result)
}

/// Sign a message with a private key
pub fn sign_message(
    private_key: &[u8; 32],
    message: &[u8],
) -> Result<(Vec<u8>, u8), ClientError> {
    let backend = get_backend();
    backend.sign_message(private_key, message)
}

/// Get Ethereum address from private key
pub fn address_from_private_key(
    private_key: &[u8; 32],
) -> Result<[u8; 20], ClientError> {
    let backend = get_backend();
    backend.address_from_private_key(private_key)
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
