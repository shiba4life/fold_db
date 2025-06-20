//! # Unified Cryptographic Types
//!
//! This module defines all fundamental cryptographic types and structures used
//! throughout the unified cryptographic system. It provides type-safe abstractions
//! for cryptographic algorithms, keys, and data structures.

use crate::security_types::SecurityLevel;
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Supported cryptographic algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize)]
pub enum Algorithm {
    /// Ed25519 digital signature algorithm
    Ed25519,
    /// RSA with PSS padding (2048-bit minimum)
    RsaPss2048,
    /// RSA with PSS padding (4096-bit)
    RsaPss4096,
    /// AES-256 with GCM mode
    Aes256Gcm,
    /// ChaCha20-Poly1305 AEAD
    ChaCha20Poly1305,
    /// SHA-256 hash function
    Sha256,
    /// SHA-3-256 hash function
    Sha3_256,
    /// BLAKE3 hash function
    Blake3,
    /// Argon2id key derivation
    Argon2id,
    /// HKDF key derivation
    Hkdf,
    /// PBKDF2 key derivation
    Pbkdf2,
}

impl Algorithm {
    /// Get the security level provided by this algorithm
    pub fn security_level(&self) -> SecurityLevel {
        match self {
            Algorithm::Ed25519 => SecurityLevel::High,
            Algorithm::RsaPss2048 => SecurityLevel::Standard,
            Algorithm::RsaPss4096 => SecurityLevel::High,
            Algorithm::Aes256Gcm => SecurityLevel::High,
            Algorithm::ChaCha20Poly1305 => SecurityLevel::High,
            Algorithm::Sha256 => SecurityLevel::Standard,
            Algorithm::Sha3_256 => SecurityLevel::High,
            Algorithm::Blake3 => SecurityLevel::High,
            Algorithm::Argon2id => SecurityLevel::High,
            Algorithm::Hkdf => SecurityLevel::Standard,
            Algorithm::Pbkdf2 => SecurityLevel::Standard,
        }
    }

    /// Check if this algorithm is approved for the given security level
    pub fn is_approved_for_level(&self, level: SecurityLevel) -> bool {
        match level {
            SecurityLevel::Basic => true, // All algorithms approved for basic level
            SecurityLevel::Low => true, // All algorithms approved
            SecurityLevel::Standard => {
                matches!(self.security_level(), SecurityLevel::Standard | SecurityLevel::High)
            }
            SecurityLevel::High => {
                // Only the most secure algorithms for high security use
                matches!(
                    self,
                    Algorithm::Ed25519
                        | Algorithm::RsaPss4096
                        | Algorithm::Aes256Gcm
                        | Algorithm::ChaCha20Poly1305
                        | Algorithm::Sha3_256
                        | Algorithm::Blake3
                        | Algorithm::Argon2id
                )
            }
        }
    }

    /// Get the key size in bits for this algorithm
    pub fn key_size_bits(&self) -> Option<usize> {
        match self {
            Algorithm::Ed25519 => Some(256),
            Algorithm::RsaPss2048 => Some(2048),
            Algorithm::RsaPss4096 => Some(4096),
            Algorithm::Aes256Gcm => Some(256),
            Algorithm::ChaCha20Poly1305 => Some(256),
            _ => None, // Hash and KDF algorithms don't have fixed key sizes
        }
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Algorithm::Ed25519 => write!(f, "Ed25519"),
            Algorithm::RsaPss2048 => write!(f, "RSA-PSS-2048"),
            Algorithm::RsaPss4096 => write!(f, "RSA-PSS-4096"),
            Algorithm::Aes256Gcm => write!(f, "AES-256-GCM"),
            Algorithm::ChaCha20Poly1305 => write!(f, "ChaCha20-Poly1305"),
            Algorithm::Sha256 => write!(f, "SHA-256"),
            Algorithm::Sha3_256 => write!(f, "SHA3-256"),
            Algorithm::Blake3 => write!(f, "BLAKE3"),
            Algorithm::Argon2id => write!(f, "Argon2id"),
            Algorithm::Hkdf => write!(f, "HKDF"),
            Algorithm::Pbkdf2 => write!(f, "PBKDF2"),
        }
    }
}

/// Digital signature algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// Ed25519 signature algorithm
    Ed25519,
    /// RSA with PSS padding and SHA-256
    RsaPssSha256,
    /// RSA with PSS padding and SHA-3-256
    RsaPssSha3_256,
}

impl From<SignatureAlgorithm> for Algorithm {
    fn from(sig_alg: SignatureAlgorithm) -> Self {
        match sig_alg {
            SignatureAlgorithm::Ed25519 => Algorithm::Ed25519,
            SignatureAlgorithm::RsaPssSha256 => Algorithm::RsaPss2048,
            SignatureAlgorithm::RsaPssSha3_256 => Algorithm::RsaPss4096,
        }
    }
}

/// Hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// SHA-256
    Sha256,
    /// SHA-3-256
    Sha3_256,
    /// BLAKE3
    Blake3,
}

impl From<HashAlgorithm> for Algorithm {
    fn from(hash_alg: HashAlgorithm) -> Self {
        match hash_alg {
            HashAlgorithm::Sha256 => Algorithm::Sha256,
            HashAlgorithm::Sha3_256 => Algorithm::Sha3_256,
            HashAlgorithm::Blake3 => Algorithm::Blake3,
        }
    }
}

/// Key derivation functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyDerivationFunction {
    /// Argon2id (recommended for password-based derivation)
    Argon2id,
    /// HKDF (for key expansion)
    Hkdf,
    /// PBKDF2 (legacy support)
    Pbkdf2,
}

impl From<KeyDerivationFunction> for Algorithm {
    fn from(kdf: KeyDerivationFunction) -> Self {
        match kdf {
            KeyDerivationFunction::Argon2id => Algorithm::Argon2id,
            KeyDerivationFunction::Hkdf => Algorithm::Hkdf,
            KeyDerivationFunction::Pbkdf2 => Algorithm::Pbkdf2,
        }
    }
}

/// Cipher suite combining encryption and authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CipherSuite {
    /// AES-256-GCM (recommended)
    Aes256Gcm,
    /// ChaCha20-Poly1305 (alternative for platforms without AES acceleration)
    ChaCha20Poly1305,
}

impl From<CipherSuite> for Algorithm {
    fn from(cipher: CipherSuite) -> Self {
        match cipher {
            CipherSuite::Aes256Gcm => Algorithm::Aes256Gcm,
            CipherSuite::ChaCha20Poly1305 => Algorithm::ChaCha20Poly1305,
        }
    }
}

/// Unique identifier for cryptographic keys
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize)]
pub struct KeyId {
    /// Unique identifier string
    id: String,
    /// Algorithm associated with this key
    algorithm: Algorithm,
}

impl KeyId {
    /// Create a new key ID
    pub fn new(id: String, algorithm: Algorithm) -> Self {
        Self { id, algorithm }
    }

    /// Generate a new random key ID
    pub fn generate(algorithm: Algorithm) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        
        // Simple ID generation - in production, consider using UUID
        let id = format!("key-{}-{:x}", algorithm, timestamp);
        
        Self::new(id, algorithm)
    }

    /// Get the key ID string
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the algorithm for this key
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.id)
    }
}

/// Encrypted data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    ciphertext: Vec<u8>,
    /// Initialization vector or nonce
    iv: Vec<u8>,
    /// Authentication tag (for AEAD ciphers)
    tag: Vec<u8>,
    /// Algorithm used for encryption
    algorithm: Algorithm,
    /// Key ID used for encryption
    key_id: KeyId,
    /// Additional authenticated data (if any)
    aad: Vec<u8>,
}

impl EncryptedData {
    /// Create new encrypted data
    pub fn new(
        ciphertext: Vec<u8>,
        iv: Vec<u8>,
        tag: Vec<u8>,
        algorithm: Algorithm,
        key_id: KeyId,
        aad: Vec<u8>,
    ) -> Self {
        Self {
            ciphertext,
            iv,
            tag,
            algorithm,
            key_id,
            aad,
        }
    }

    /// Get the ciphertext
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Get the initialization vector
    pub fn iv(&self) -> &[u8] {
        &self.iv
    }

    /// Get the authentication tag
    pub fn tag(&self) -> &[u8] {
        &self.tag
    }

    /// Get the encryption algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Get the key ID
    pub fn key_id(&self) -> &KeyId {
        &self.key_id
    }

    /// Get additional authenticated data
    pub fn aad(&self) -> &[u8] {
        &self.aad
    }

    /// Get total encrypted data size
    pub fn len(&self) -> usize {
        self.ciphertext.len() + self.iv.len() + self.tag.len() + self.aad.len()
    }

    /// Check if encrypted data is empty
    pub fn is_empty(&self) -> bool {
        self.ciphertext.is_empty()
    }
}

/// Digital signature with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// The signature bytes
    signature: Vec<u8>,
    /// Algorithm used for signing
    algorithm: SignatureAlgorithm,
    /// Key ID used for signing
    key_id: KeyId,
}

impl Signature {
    /// Create a new signature
    pub fn new(signature: Vec<u8>, algorithm: SignatureAlgorithm, key_id: KeyId) -> Self {
        Self {
            signature,
            algorithm,
            key_id,
        }
    }

    /// Get the signature bytes
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Get the signature algorithm
    pub fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    /// Get the key ID
    pub fn key_id(&self) -> &KeyId {
        &self.key_id
    }

    /// Get signature length
    pub fn len(&self) -> usize {
        self.signature.len()
    }

    /// Check if signature is empty
    pub fn is_empty(&self) -> bool {
        self.signature.is_empty()
    }
}

/// Secure key material with automatic zeroization
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct KeyMaterial {
    /// The key bytes (automatically zeroized on drop)
    bytes: Vec<u8>,
}

impl KeyMaterial {
    /// Create new key material from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Get the key bytes (careful - this exposes sensitive data)
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the key length
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if key material is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Secure clone that zeroizes the original
    pub fn secure_clone(&mut self) -> Self {
        let cloned = Self {
            bytes: self.bytes.clone(),
        };
        self.bytes.zeroize();
        cloned
    }
}

impl fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyMaterial")
            .field("len", &self.bytes.len())
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}


/// Key usage permissions and restrictions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyUsage {
    /// Can be used for encryption
    pub encrypt: bool,
    /// Can be used for decryption
    pub decrypt: bool,
    /// Can be used for signing
    pub sign: bool,
    /// Can be used for verification
    pub verify: bool,
    /// Can be used for key derivation
    pub derive: bool,
    /// Maximum number of operations (None = unlimited)
    pub max_operations: Option<u64>,
    /// Key expiration time (None = no expiration)
    pub expires_at: Option<std::time::SystemTime>,
}

impl KeyUsage {
    /// Create key usage for signing keys
    pub fn signing_key() -> Self {
        Self {
            encrypt: false,
            decrypt: false,
            sign: true,
            verify: true,
            derive: false,
            max_operations: None,
            expires_at: None,
        }
    }

    /// Create key usage for encryption keys
    pub fn encryption_key() -> Self {
        Self {
            encrypt: true,
            decrypt: true,
            sign: false,
            verify: false,
            derive: false,
            max_operations: None,
            expires_at: None,
        }
    }

    /// Create key usage for derivation keys
    pub fn derivation_key() -> Self {
        Self {
            encrypt: false,
            decrypt: false,
            sign: false,
            verify: false,
            derive: true,
            max_operations: None,
            expires_at: None,
        }
    }

    /// Create key usage for master keys (can do everything)
    pub fn master_key() -> Self {
        Self {
            encrypt: true,
            decrypt: true,
            sign: true,
            verify: true,
            derive: true,
            max_operations: None,
            expires_at: None,
        }
    }

    /// Check if the key can be used for the specified operation
    pub fn allows_operation(&self, operation: KeyOperation) -> bool {
        match operation {
            KeyOperation::Encrypt => self.encrypt,
            KeyOperation::Decrypt => self.decrypt,
            KeyOperation::Sign => self.sign,
            KeyOperation::Verify => self.verify,
            KeyOperation::Derive => self.derive,
        }
    }

    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            std::time::SystemTime::now() > expires_at
        } else {
            false
        }
    }
}

impl Default for KeyUsage {
    fn default() -> Self {
        Self {
            encrypt: false,
            decrypt: false,
            sign: false,
            verify: false,
            derive: false,
            max_operations: None,
            expires_at: None,
        }
    }
}

/// Key operations for usage validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOperation {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    Derive,
}

impl fmt::Display for KeyOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyOperation::Encrypt => write!(f, "encrypt"),
            KeyOperation::Decrypt => write!(f, "decrypt"),
            KeyOperation::Sign => write!(f, "sign"),
            KeyOperation::Verify => write!(f, "verify"),
            KeyOperation::Derive => write!(f, "derive"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_security_levels() {
        assert_eq!(Algorithm::Ed25519.security_level(), SecurityLevel::High);
        assert_eq!(Algorithm::Sha256.security_level(), SecurityLevel::Standard);
        assert!(Algorithm::Aes256Gcm.is_approved_for_level(SecurityLevel::High));
        assert!(Algorithm::Sha256.is_approved_for_level(SecurityLevel::Standard));
    }

    #[test]
    fn test_key_id_generation() {
        let key_id = KeyId::generate(Algorithm::Ed25519);
        assert_eq!(key_id.algorithm(), Algorithm::Ed25519);
        assert!(key_id.id().contains("Ed25519"));
    }

    #[test]
    fn test_key_material_zeroization() {
        let mut key_material = KeyMaterial::from_bytes(vec![1, 2, 3, 4]);
        assert_eq!(key_material.len(), 4);
        
        let cloned = key_material.secure_clone();
        // Original should be zeroized after secure_clone
        assert_eq!(key_material.bytes(), &[0, 0, 0, 0]);
        assert_eq!(cloned.bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_key_usage_validation() {
        let signing_usage = KeyUsage::signing_key();
        assert!(signing_usage.allows_operation(KeyOperation::Sign));
        assert!(!signing_usage.allows_operation(KeyOperation::Encrypt));
        
        let encryption_usage = KeyUsage::encryption_key();
        assert!(encryption_usage.allows_operation(KeyOperation::Encrypt));
        assert!(!encryption_usage.allows_operation(KeyOperation::Sign));
    }

    #[test]
    fn test_encrypted_data_creation() {
        let key_id = KeyId::generate(Algorithm::Aes256Gcm);
        let encrypted = EncryptedData::new(
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            Algorithm::Aes256Gcm,
            key_id.clone(),
            vec![],
        );
        
        assert_eq!(encrypted.ciphertext(), &[1, 2, 3, 4]);
        assert_eq!(encrypted.algorithm(), Algorithm::Aes256Gcm);
        assert_eq!(encrypted.key_id(), &key_id);
    }

    #[test]
    fn test_signature_creation() {
        let key_id = KeyId::generate(Algorithm::Ed25519);
        let signature = Signature::new(
            vec![1, 2, 3, 4],
            SignatureAlgorithm::Ed25519,
            key_id.clone(),
        );
        
        assert_eq!(signature.signature(), &[1, 2, 3, 4]);
        assert_eq!(signature.algorithm(), SignatureAlgorithm::Ed25519);
        assert_eq!(signature.key_id(), &key_id);
    }
}