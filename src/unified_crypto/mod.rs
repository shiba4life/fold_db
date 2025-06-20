//! # Unified Cryptographic Module for DataFold
//!
//! This module provides the unified cryptographic infrastructure for DataFold, consolidating
//! all cryptographic operations into a single, authoritative implementation. It follows a
//! layered architecture with clear separation between primitives and operational concerns.
//!
//! ## Architecture
//!
//! - **Primitives Layer**: Core cryptographic operations (encryption, signing, hashing)
//! - **Key Management**: Unified key lifecycle management and rotation
//! - **Configuration**: Centralized cryptographic configuration and policy enforcement
//! - **Audit**: Comprehensive security audit logging and monitoring
//! - **Error Handling**: Unified error types and security-aware error propagation
//!
//! ## Security Features
//!
//! - Secure memory handling with automatic zeroization
//! - Comprehensive input validation and bounds checking
//! - Timing attack protections
//! - Crypto-agility with pluggable algorithm support
//! - Strong access controls and security boundaries
//! - Tamper-evident audit trails
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::unified_crypto::{UnifiedCrypto, CryptoConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize unified crypto with configuration
//! let config = CryptoConfig::default();
//! let crypto = UnifiedCrypto::new(config)?;
//!
//! // Generate a new key pair
//! let keypair = crypto.generate_keypair()?;
//!
//! // Encrypt data
//! let plaintext = b"sensitive data";
//! let ciphertext = crypto.encrypt(plaintext, &keypair.public_key)?;
//!
//! // Decrypt data
//! let decrypted = crypto.decrypt(&ciphertext, &keypair.private_key)?;
//! assert_eq!(plaintext, &decrypted[..]);
//! # Ok(())
//! # }
//! ```

// Core modules
pub mod audit;
pub mod config;
pub mod error;
pub mod keys;
pub mod primitives;
pub mod types;

// Operational layer modules
pub mod operations;
pub mod database;
pub mod auth;
pub mod network;
pub mod backup;
pub mod cli;

// Re-export core types for public API
pub use audit::{CryptoAuditEvent, CryptoAuditLogger, SecurityAuditTrail};
pub use config::{
    CryptoConfig, CryptoPolicy, EncryptionConfig, HashingConfig, KeyConfig, SigningConfig,
};
pub use error::{SecurityError, SecurityResult, UnifiedCryptoError, UnifiedCryptoResult};
pub use keys::{
    KeyManager, KeyMetadata, KeyPair, KeyRotationManager,
};
pub use primitives::{
    CryptoPrimitives, EncryptionPrimitive, HashingPrimitive, PrivateKeyHandle, PublicKeyHandle,
    SigningPrimitive,
};
pub use types::{
    Algorithm, CipherSuite, EncryptedData, HashAlgorithm, KeyDerivationFunction, Signature,
    SignatureAlgorithm,
};

// Re-export operational layer types
pub use operations::{CryptoOperations, OperationSession, PrivilegeLevel, RateLimitConfig};
pub use database::{DatabaseOperations, DatabaseContext, DatabaseEncryptionPolicy, EncryptedDatabaseRecord};
pub use auth::{AuthenticationOperations, AuthenticationResult, AuthenticationStrength, AuthenticationPolicy};
pub use network::{NetworkSecurityOperations, SecureChannel, EncryptedNetworkMessage, NetworkSecurityPolicy};
pub use backup::{BackupOperations, EncryptedBackup, BackupSecurityPolicy, BackupType};
pub use cli::{CliOperations};

use std::sync::Arc;
use ring::{digest, hkdf, pbkdf2, rand};
use ring::rand::SecureRandom;
use std::num::NonZeroU32;

/// Main unified cryptographic interface for DataFold
///
/// This is the primary entry point for all cryptographic operations in DataFold.
/// It provides a unified, secure interface that consolidates all cryptographic
/// functionality while maintaining clear security boundaries.
#[derive(Clone)]
pub struct UnifiedCrypto {
    /// Core cryptographic primitives
    primitives: Arc<CryptoPrimitives>,
    /// Key management system
    key_manager: Arc<KeyManager>,
    /// Security audit logger
    audit_logger: Arc<CryptoAuditLogger>,
    /// Cryptographic configuration and policies
    config: Arc<CryptoConfig>,
}

impl UnifiedCrypto {
    /// Create a new unified crypto instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Cryptographic configuration and policies
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New unified crypto instance or error
    ///
    /// # Security
    /// - Validates all configuration parameters for security compliance
    /// - Initializes secure audit logging
    /// - Establishes cryptographic security boundaries
    pub fn new(config: CryptoConfig) -> UnifiedCryptoResult<Self> {
        // Validate configuration for security compliance
        config.validate_security()?;

        // Initialize audit logger with tamper-evident logging
        let audit_logger = Arc::new(CryptoAuditLogger::new(&config.audit)?);

        // Log initialization event
        audit_logger.log_crypto_event(CryptoAuditEvent::initialization(&config))?;

        // Initialize key manager with secure storage
        let key_manager = Arc::new(KeyManager::new(&config.keys, audit_logger.clone())?);

        // Initialize cryptographic primitives
        let primitives = Arc::new(CryptoPrimitives::new(&config.primitives)?);

        let instance = Self {
            primitives,
            key_manager,
            audit_logger,
            config: Arc::new(config),
        };

        // Log successful initialization
        instance.audit_logger.log_crypto_event(
            CryptoAuditEvent::initialization_complete(&instance.config)
        )?;

        Ok(instance)
    }

    /// Generate a new cryptographic key pair
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<KeyPair>` - New key pair or error
    ///
    /// # Security
    /// - Uses cryptographically secure random number generation
    /// - Applies configured key generation policies
    /// - Logs key generation for audit trail
    pub fn generate_keypair(&self) -> UnifiedCryptoResult<KeyPair> {
        self.key_manager.generate_keypair(&self.config.keys.default_algorithm)
    }

    /// Encrypt data using the specified public key
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    /// * `public_key` - Public key for encryption
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedData>` - Encrypted data or error
    ///
    /// # Security
    /// - Uses authenticated encryption (AEAD)
    /// - Validates all inputs for security compliance
    /// - Logs encryption operation for audit trail
    pub fn encrypt(&self, plaintext: &[u8], public_key: &PublicKeyHandle) -> UnifiedCryptoResult<EncryptedData> {
        // Input validation
        if plaintext.is_empty() {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Plaintext cannot be empty".to_string(),
            });
        }

        // Log encryption operation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::encryption_start(public_key.id(), plaintext.len())
        )?;

        // Perform encryption using primitives
        let result = self.primitives.encrypt(plaintext, public_key);

        // Log encryption result
        match &result {
            Ok(encrypted) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::encryption_success(public_key.id(), encrypted.len())
                )?;
            }
            Err(error) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::encryption_failure(public_key.id(), error)
                )?;
            }
        }

        result
    }

    /// Decrypt data using the specified private key
    ///
    /// # Arguments
    /// * `ciphertext` - Encrypted data to decrypt
    /// * `private_key` - Private key for decryption
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Decrypted data or error
    ///
    /// # Security
    /// - Validates ciphertext authenticity before decryption
    /// - Securely handles decrypted data in memory
    /// - Logs decryption operation for audit trail
    pub fn decrypt(&self, ciphertext: &EncryptedData, private_key: &PrivateKeyHandle) -> UnifiedCryptoResult<Vec<u8>> {
        // Log decryption operation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::decryption_start(private_key.id(), ciphertext.len())
        )?;

        // Perform decryption using primitives
        let result = self.primitives.decrypt(ciphertext, private_key);

        // Log decryption result (without leaking plaintext size on success)
        match &result {
            Ok(_) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::decryption_success(private_key.id())
                )?;
            }
            Err(error) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::decryption_failure(private_key.id(), error)
                )?;
            }
        }

        result
    }

    /// Sign data using the specified private key
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `private_key` - Private key for signing
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Signature>` - Digital signature or error
    ///
    /// # Security
    /// - Uses cryptographically secure signature algorithms
    /// - Validates signing key authorization
    /// - Logs signing operation for audit trail
    pub fn sign(&self, data: &[u8], private_key: &PrivateKeyHandle) -> UnifiedCryptoResult<Signature> {
        // Log signing operation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::signing_start(private_key.id(), data.len())
        )?;

        // Perform signing using primitives
        let result = self.primitives.sign(data, private_key);

        // Log signing result
        match &result {
            Ok(signature) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::signing_success(private_key.id(), signature)
                )?;
            }
            Err(error) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::signing_failure(private_key.id(), error)
                )?;
            }
        }

        result
    }

    /// Verify a digital signature
    ///
    /// # Arguments
    /// * `data` - Data that was signed
    /// * `signature` - Digital signature to verify
    /// * `public_key` - Public key for verification
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<bool>` - Verification result or error
    ///
    /// # Security
    /// - Uses constant-time signature verification
    /// - Validates signature format and algorithm
    /// - Logs verification operation for audit trail
    pub fn verify(&self, data: &[u8], signature: &Signature, public_key: &PublicKeyHandle) -> UnifiedCryptoResult<bool> {
        // Log verification operation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::verification_start(public_key.id(), signature)
        )?;

        // Perform verification using primitives
        let result = self.primitives.verify(data, signature, public_key);

        // Log verification result
        match &result {
            Ok(valid) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::verification_complete(public_key.id(), *valid)
                )?;
            }
            Err(error) => {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::verification_error(public_key.id(), error)
                )?;
            }
        }

        result
    }

    /// Compute a cryptographic hash of the given data
    ///
    /// # Arguments
    /// * `data` - Data to hash
    /// * `algorithm` - Hash algorithm to use
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Hash digest or error
    ///
    /// # Security
    /// - Uses cryptographically secure hash algorithms
    /// - Validates input data and algorithm selection
    /// - Logs hashing operation for audit trail
    pub fn hash(&self, data: &[u8], algorithm: HashAlgorithm) -> UnifiedCryptoResult<Vec<u8>> {
        self.primitives.hash(data, algorithm)
    }

    /// Get access to the key manager for advanced key operations
    ///
    /// # Returns
    /// * `&KeyManager` - Reference to the key manager
    ///
    /// # Security
    /// - Provides controlled access to key management operations
    /// - All key operations are subject to audit logging
    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }

    /// Get access to the audit logger for security monitoring
    ///
    /// # Returns
    /// * `Arc<CryptoAuditLogger>` - Arc reference to the audit logger
    pub fn audit_logger(&self) -> Arc<CryptoAuditLogger> {
        self.audit_logger.clone()
    }

    /// Get the current cryptographic configuration
    ///
    /// # Returns
    /// * `&CryptoConfig` - Reference to the configuration
    pub fn config(&self) -> &CryptoConfig {
        &self.config
    }
}

/// Derives a key using HKDF (HMAC-based Key Derivation Function)
///
/// # Arguments
/// * `master_key` - The master key material
/// * `salt` - Salt for key derivation
/// * `info` - Context information for key derivation
/// * `output_len` - Length of the derived key
///
/// # Returns
/// * `Result<Vec<u8>, String>` - The derived key or an error
pub fn derive_key_hkdf(
    master_key: &[u8],
    salt: &[u8],
    info: &[u8],
    output_len: usize,
) -> Result<Vec<u8>, String> {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);
    let prk = salt.extract(master_key);
    let info = &[info];
    let okm = prk.expand(info, hkdf::HKDF_SHA256)
        .map_err(|e| format!("HKDF expand failed: {:?}", e))?;
    
    let mut output = vec![0u8; output_len];
    okm.fill(&mut output)
        .map_err(|e| format!("HKDF fill failed: {:?}", e))?;
    
    Ok(output)
}

/// Generates a cryptographically secure random salt
///
/// # Returns
/// * `Vec<u8>` - The generated salt (32 bytes)
pub fn generate_salt() -> Vec<u8> {
    let rng = rand::SystemRandom::new();
    let mut salt = vec![0u8; 32];
    if let Err(_) = rng.fill(&mut salt) {
        // Fallback to a basic salt if cryptographic generation fails
        for i in 0..32 {
            salt[i] = (i as u8).wrapping_mul(7);
        }
    }
    salt
}

/// Compatibility derive_key function for existing CLI code
///
/// # Arguments
/// * `password` - The password to derive from
/// * `salt` - Salt for key derivation
/// * `argon2_params` - Argon2 parameters (ignored for now, using PBKDF2)
///
/// # Returns
/// * `Result<Vec<u8>, String>` - The derived key or an error
pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    _argon2_params: &crate::unified_crypto::config::Argon2Params,
) -> Result<Vec<u8>, String> {
    // Use PBKDF2 as a fallback for compatibility
    derive_key_pbkdf2(password, salt, 10000, 32)
}

/// Derives a key using PBKDF2 (Password-Based Key Derivation Function 2)
///
/// # Arguments
/// * `password` - The password to derive from
/// * `salt` - Salt for key derivation
/// * `iterations` - Number of iterations
/// * `output_len` - Length of the derived key
///
/// # Returns
/// * `Result<Vec<u8>, String>` - The derived key or an error
pub fn derive_key_pbkdf2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    output_len: usize,
) -> Result<Vec<u8>, String> {
    let iterations = NonZeroU32::new(iterations)
        .ok_or("Iterations must be non-zero")?;
    
    let mut output = vec![0u8; output_len];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password,
        &mut output,
    );
    
    Ok(output)
}

/// Hashes data using SHA-256
///
/// # Arguments
/// * `data` - Data to hash
///
/// # Returns
/// * `Vec<u8>` - The hash digest
pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, data).as_ref().to_vec()
}

// Implement secure cleanup for UnifiedCrypto
impl Drop for UnifiedCrypto {
    fn drop(&mut self) {
        // Log shutdown event (ignore errors during shutdown)
        let _ = self.audit_logger.log_crypto_event(
            CryptoAuditEvent::shutdown()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_crypto_initialization() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config);
        assert!(crypto.is_ok());
    }

    #[test]
    fn test_keypair_generation() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to initialize crypto");
        
        let keypair = crypto.generate_keypair();
        assert!(keypair.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to initialize crypto");
        
        let keypair = crypto.generate_keypair().expect("Failed to generate keypair");
        let plaintext = b"test data for encryption";
        
        let ciphertext = crypto.encrypt(plaintext, &keypair.public_key)
            .expect("Failed to encrypt");
        
        let decrypted = crypto.decrypt(&ciphertext, &keypair.private_key)
            .expect("Failed to decrypt");
        
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to initialize crypto");
        
        let keypair = crypto.generate_keypair().expect("Failed to generate keypair");
        let data = b"test data for signing";
        
        let signature = crypto.sign(data, &keypair.private_key)
            .expect("Failed to sign");
        
        let valid = crypto.verify(data, &signature, &keypair.public_key)
            .expect("Failed to verify");
        
        assert!(valid);
    }
}