//! Legacy Cryptographic Utilities - DEPRECATED
//!
//! This entire module is deprecated in favor of the unified cryptographic system.
//! All new code should use `crate::unified_crypto` instead.
//!
//! ## Migration Guide
//!
//! - `crate::crypto::generate_master_keypair()` → `UnifiedCrypto::generate_keypair()`
//! - `crate::crypto::MasterKeyPair` → `crate::unified_crypto::keys::KeyPair`
//! - `crate::crypto::PublicKey` → `crate::unified_crypto::primitives::PublicKeyHandle`
//! - `crate::crypto::CryptoError` → `crate::unified_crypto::error::UnifiedCryptoError`

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
pub use crate::unified_crypto::error::UnifiedCryptoError as CryptoError;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::error::UnifiedCryptoResult instead")]
pub use crate::unified_crypto::error::UnifiedCryptoResult as CryptoResult;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::keys::KeyPair instead")]
pub use crate::unified_crypto::keys::KeyPair as MasterKeyPair;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::primitives::PublicKeyHandle instead")]
pub use crate::unified_crypto::primitives::PublicKeyHandle as PublicKey;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::primitives::PrivateKeyHandle instead")]
pub use crate::unified_crypto::primitives::PrivateKeyHandle as PrivateKey;

// Key generation functions
#[deprecated(since = "1.0.0", note = "Use UnifiedCrypto::generate_keypair instead")]
pub fn generate_master_keypair() -> crate::unified_crypto::error::UnifiedCryptoResult<crate::unified_crypto::keys::KeyPair> {
    let config = crate::unified_crypto::config::CryptoConfig::default();
    let crypto = crate::unified_crypto::UnifiedCrypto::new(config)?;
    crypto.generate_keypair()
}

// Key rotation types for compatibility
#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::keys::RotationReason instead")]
pub use crate::unified_crypto::keys::RotationReason;

// Signature verification function
#[deprecated(since = "1.0.0", note = "Use unified crypto primitives instead")]
pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> crate::unified_crypto::error::UnifiedCryptoResult<bool> {
    use crate::unified_crypto::primitives::PublicKeyHandle;
    use crate::unified_crypto::types::Algorithm;
    
    let public_key_handle = PublicKeyHandle::from_bytes(public_key, Algorithm::Ed25519)?;
    public_key_handle.verify(message, signature)
}


// Legacy modules for compatibility (empty stubs to prevent compilation errors)
pub mod argon2 {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
    
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Salt([u8; 32]);
    
    impl Salt {
        pub fn from_bytes(bytes: &[u8]) -> Self {
            let mut salt = [0u8; 32];
            let len = bytes.len().min(32);
            salt[..len].copy_from_slice(&bytes[..len]);
            Self(salt)
        }
        
        pub fn as_bytes(&self) -> &[u8] {
            &self.0
        }
    }
    
    // Missing function for compatibility
    #[deprecated(since = "1.0.0", note = "Use unified crypto primitives instead")]
    pub fn generate_salt_and_derive_keypair(_passphrase: &str) -> crate::unified_crypto::error::UnifiedCryptoResult<(Salt, Vec<u8>)> {
        Err(crate::unified_crypto::error::UnifiedCryptoError::CryptographicOperation {
            operation: "generate_salt_and_derive_keypair not implemented yet".to_string()
        })
    }
}

pub mod ed25519 {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
    
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::UnifiedCrypto::generate_keypair instead")]
    pub use super::generate_master_keypair;
    
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::keys::KeyPair instead")]
    pub use super::MasterKeyPair;
    
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::primitives::PublicKeyHandle instead")]
    pub use super::PublicKey;
    
    #[deprecated(since = "1.0.0", note = "Use unified crypto primitives instead")]
    pub use super::verify_signature;
    
    // Missing function for compatibility
    #[deprecated(since = "1.0.0", note = "Use unified crypto primitives instead")]
    pub fn generate_master_keypair_from_seed(seed: &[u8]) -> crate::unified_crypto::error::UnifiedCryptoResult<crate::unified_crypto::keys::KeyPair> {
        use crate::unified_crypto::types::Algorithm;
        crate::unified_crypto::keys::KeyPair::from_secret_bytes(seed, Algorithm::Ed25519)
    }
    
    // Constants for backward compatibility
    pub const SIGNATURE_LENGTH: usize = 64;
    pub const PUBLIC_KEY_LENGTH: usize = 32;
    pub const PRIVATE_KEY_LENGTH: usize = 32;
}

pub mod error {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::error instead")]
    pub use crate::unified_crypto::error::*;
}

pub mod audit_logger {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::audit instead")]
    pub use crate::unified_crypto::audit::*;
    
    // Specific exports that tests expect
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::audit::CryptoAuditLogger instead")]
    pub use crate::unified_crypto::audit::CryptoAuditLogger;
    
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::audit::OperationStatus instead")]
    pub use crate::unified_crypto::audit::OperationStatus;
    
    // Create a compatibility type for SecurityEventDetails
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::audit::SecurityContext instead")]
    pub type SecurityEventDetails = crate::unified_crypto::audit::SecurityContext;
}

pub mod enhanced_error {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::error instead")]
    pub use crate::unified_crypto::error::*;
}


pub mod key_rotation {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod key_rotation_audit {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::audit instead")]
    pub use crate::unified_crypto::audit::*;
}

pub mod key_rotation_security {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod threat_types {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod threat_config {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod threat_detection {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod threat_monitor {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod threat_tests {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}

pub mod security_monitor {
    #[deprecated(since = "1.0.0", note = "Use crate::unified_crypto instead")]
    pub use super::*;
}
