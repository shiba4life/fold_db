//! AES-256-GCM encryption utilities for DataFold encryption at rest
//!
//! This module provides secure AES-256-GCM encryption and decryption operations
//! for encrypting data at rest in the database. It follows security best practices
//! including secure nonce generation, memory zeroization, and proper error handling.
//!
//! ## Security Features
//!
//! * AES-256-GCM authenticated encryption for confidentiality and integrity
//! * Secure random nonce generation for each encryption operation
//! * Memory safety with automatic zeroization of sensitive data
//! * Integration with existing crypto error handling from PBI 8
//! * Constant-time operations to prevent side-channel attacks
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::crypto::generate_master_keypair;
//! use datafold::datafold_node::encryption_at_rest::EncryptionAtRest;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create encryption manager with a derived encryption key
//! let encryption_key = [0u8; 32]; // In practice, derive from master key
//! let encryptor = EncryptionAtRest::new(encryption_key)?;
//!
//! // Encrypt data
//! let plaintext = b"sensitive database data";
//! let encrypted_data = encryptor.encrypt(plaintext)?;
//!
//! // Decrypt data
//! let decrypted_data = encryptor.decrypt(&encrypted_data)?;
//! assert_eq!(plaintext, &decrypted_data[..]);
//! # Ok(())
//! # }
//! ```

use crate::crypto::error::{CryptoError, CryptoResult};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

/// Size of AES-256 encryption key in bytes
pub const AES_KEY_SIZE: usize = 32;

/// Size of AES-GCM nonce in bytes
pub const AES_NONCE_SIZE: usize = 12;

/// Size of AES-GCM authentication tag in bytes
pub const AES_TAG_SIZE: usize = 16;

/// Minimum size for encrypted data (nonce + tag)
pub const MIN_ENCRYPTED_SIZE: usize = AES_NONCE_SIZE + AES_TAG_SIZE;

/// Maximum plaintext size we'll encrypt (safety limit)
pub const MAX_PLAINTEXT_SIZE: usize = 1024 * 1024 * 100; // 100 MB

/// AES-256-GCM encryption key wrapper with automatic zeroization
#[derive(Clone, ZeroizeOnDrop)]
pub struct EncryptionKey {
    /// The 256-bit AES key
    key: [u8; AES_KEY_SIZE],
}

impl EncryptionKey {
    /// Create a new encryption key from raw bytes
    pub fn new(key_bytes: [u8; AES_KEY_SIZE]) -> Self {
        Self { key: key_bytes }
    }

    /// Create a new encryption key from a slice
    pub fn from_slice(slice: &[u8]) -> CryptoResult<Self> {
        if slice.len() != AES_KEY_SIZE {
            return Err(CryptoError::InvalidKey {
                message: format!(
                    "Invalid key size: expected {} bytes, got {}",
                    AES_KEY_SIZE,
                    slice.len()
                ),
            });
        }

        let mut key = [0u8; AES_KEY_SIZE];
        key.copy_from_slice(slice);
        Ok(Self::new(key))
    }

    /// Get the key as a reference to the bytes
    pub(crate) fn as_bytes(&self) -> &[u8; AES_KEY_SIZE] {
        &self.key
    }

    /// Convert to aes-gcm Key type
    pub(crate) fn to_aes_key(&self) -> Key<Aes256Gcm> {
        *Key::<Aes256Gcm>::from_slice(&self.key)
    }
}

/// Encrypted data container with nonce and ciphertext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The nonce used for this encryption (12 bytes)
    pub nonce: Vec<u8>,
    /// The encrypted data including authentication tag
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Create new encrypted data container
    pub fn new(nonce: Vec<u8>, ciphertext: Vec<u8>) -> CryptoResult<Self> {
        if nonce.len() != AES_NONCE_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid nonce size: expected {} bytes, got {}",
                AES_NONCE_SIZE,
                nonce.len()
            )));
        }

        if ciphertext.len() < AES_TAG_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid ciphertext size: must be at least {} bytes for tag",
                AES_TAG_SIZE
            )));
        }

        Ok(Self { nonce, ciphertext })
    }

    /// Get the total size of the encrypted data
    pub fn size(&self) -> usize {
        self.nonce.len() + self.ciphertext.len()
    }

    /// Convert to compact binary format (nonce + ciphertext)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.size());
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Create from compact binary format
    pub fn from_bytes(data: &[u8]) -> CryptoResult<Self> {
        if data.len() < MIN_ENCRYPTED_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Encrypted data too small: {} bytes, minimum is {}",
                data.len(),
                MIN_ENCRYPTED_SIZE
            )));
        }

        let nonce = data[..AES_NONCE_SIZE].to_vec();
        let ciphertext = data[AES_NONCE_SIZE..].to_vec();

        Self::new(nonce, ciphertext)
    }
}

/// AES-256-GCM encryption manager for data at rest
pub struct EncryptionAtRest {
    /// The encryption key (zeroized on drop)
    key: EncryptionKey,
    /// AES-GCM cipher instance
    cipher: Aes256Gcm,
}

impl EncryptionAtRest {
    /// Create a new encryption manager with the given key
    pub fn new(key: [u8; AES_KEY_SIZE]) -> CryptoResult<Self> {
        let encryption_key = EncryptionKey::new(key);
        let cipher = Aes256Gcm::new(&encryption_key.to_aes_key());

        Ok(Self {
            key: encryption_key,
            cipher,
        })
    }

    /// Create a new encryption manager from a key slice
    pub fn from_key_slice(key_slice: &[u8]) -> CryptoResult<Self> {
        let encryption_key = EncryptionKey::from_slice(key_slice)?;
        let cipher = Aes256Gcm::new(&encryption_key.to_aes_key());

        Ok(Self {
            key: encryption_key,
            cipher,
        })
    }

    /// Generate a secure random nonce for encryption
    pub fn generate_nonce() -> CryptoResult<[u8; AES_NONCE_SIZE]> {
        // Use the AES-GCM crate's secure nonce generation
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
        nonce_bytes.copy_from_slice(&nonce);
        Ok(nonce_bytes)
    }

    /// Generate a secure random nonce using the rand crate
    pub fn generate_nonce_with_rand() -> CryptoResult<[u8; AES_NONCE_SIZE]> {
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; AES_NONCE_SIZE];
        rng.fill(&mut nonce);
        Ok(nonce)
    }

    /// Encrypt data with AES-256-GCM
    ///
    /// # Arguments
    /// * `plaintext` - The data to encrypt
    ///
    /// # Returns
    /// * `Ok(EncryptedData)` - The encrypted data with nonce
    /// * `Err(CryptoError)` - If encryption fails
    pub fn encrypt(&self, plaintext: &[u8]) -> CryptoResult<EncryptedData> {
        // Safety check for plaintext size
        if plaintext.len() > MAX_PLAINTEXT_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Plaintext too large: {} bytes, maximum is {}",
                plaintext.len(),
                MAX_PLAINTEXT_SIZE
            )));
        }

        // Generate a secure random nonce
        let nonce_bytes = Self::generate_nonce()?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext =
            self.cipher
                .encrypt(nonce, plaintext)
                .map_err(|e| CryptoError::KeyGeneration {
                    message: format!("AES-GCM encryption failed: {}", e),
                })?;

        EncryptedData::new(nonce_bytes.to_vec(), ciphertext)
    }

    /// Decrypt data with AES-256-GCM
    ///
    /// # Arguments
    /// * `encrypted_data` - The encrypted data to decrypt
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - The decrypted plaintext
    /// * `Err(CryptoError)` - If decryption fails or authentication fails
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> CryptoResult<Vec<u8>> {
        // Validate nonce size
        if encrypted_data.nonce.len() != AES_NONCE_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid nonce size: expected {} bytes, got {}",
                AES_NONCE_SIZE,
                encrypted_data.nonce.len()
            )));
        }

        let nonce = Nonce::from_slice(&encrypted_data.nonce);

        // Decrypt and verify the data
        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| CryptoError::Signature {
                message: format!("AES-GCM decryption/verification failed: {}", e),
            })?;

        Ok(plaintext)
    }

    /// Encrypt data from bytes and return as bytes
    ///
    /// Convenience method that encrypts data and returns the compact binary format
    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
        let encrypted_data = self.encrypt(plaintext)?;
        Ok(encrypted_data.to_bytes())
    }

    /// Decrypt data from bytes
    ///
    /// Convenience method that parses the compact binary format and decrypts
    pub fn decrypt_bytes(&self, encrypted_bytes: &[u8]) -> CryptoResult<Vec<u8>> {
        let encrypted_data = EncryptedData::from_bytes(encrypted_bytes)?;
        self.decrypt(&encrypted_data)
    }

    /// Get the encryption key fingerprint for identification
    ///
    /// Returns a SHA-256 hash of the encryption key for identification purposes
    /// without exposing the actual key material.
    pub fn key_fingerprint(&self) -> CryptoResult<[u8; 32]> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.key.as_bytes());
        let result = hasher.finalize();

        let mut fingerprint = [0u8; 32];
        fingerprint.copy_from_slice(&result);
        Ok(fingerprint)
    }

    /// Test if the encryption is working correctly
    ///
    /// Performs a round-trip encryption/decryption test with test data
    pub fn self_test(&self) -> CryptoResult<()> {
        let test_data = b"DataFold encryption at rest self-test";
        let encrypted = self.encrypt(test_data)?;
        let decrypted = self.decrypt(&encrypted)?;

        if decrypted != test_data {
            return Err(CryptoError::Signature {
                message: "Self-test failed: decrypted data does not match original".to_string(),
            });
        }

        Ok(())
    }
}

impl Drop for EncryptionAtRest {
    fn drop(&mut self) {
        // Key will be zeroized by EncryptionKey's Drop implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::crypto::key_derivation::{self, KeyDerivationManager, integration, legacy, contexts};
    use crate::crypto::generate_master_keypair;

    fn create_test_key() -> [u8; AES_KEY_SIZE] {
        let mut key = [0u8; AES_KEY_SIZE];
        for (i, byte) in key.iter_mut().enumerate().take(AES_KEY_SIZE) {
            *byte = (i % 256) as u8;
        }
        key
    }

    #[test]
    fn test_encryption_key_creation() {
        let key_bytes = create_test_key();
        let key = EncryptionKey::new(key_bytes);
        assert_eq!(key.as_bytes(), &key_bytes);
    }

    #[test]
    fn test_encryption_key_from_slice() {
        let key_bytes = create_test_key();
        let key = EncryptionKey::from_slice(&key_bytes).unwrap();
        assert_eq!(key.as_bytes(), &key_bytes);

        // Test invalid size
        let invalid_key = [0u8; 16];
        assert!(EncryptionKey::from_slice(&invalid_key).is_err());
    }

    #[test]
    fn test_encrypted_data_creation() {
        let nonce = vec![0u8; AES_NONCE_SIZE];
        let ciphertext = vec![1u8; 32]; // Must be at least AES_TAG_SIZE

        let encrypted = EncryptedData::new(nonce, ciphertext).unwrap();
        assert_eq!(encrypted.nonce.len(), AES_NONCE_SIZE);
        assert_eq!(encrypted.ciphertext.len(), 32);

        // Test invalid nonce size
        let invalid_nonce = vec![0u8; 8];
        let ciphertext = vec![1u8; 32];
        assert!(EncryptedData::new(invalid_nonce, ciphertext).is_err());

        // Test invalid ciphertext size (too small for tag)
        let nonce = vec![0u8; AES_NONCE_SIZE];
        let invalid_ciphertext = vec![1u8; 8];
        assert!(EncryptedData::new(nonce, invalid_ciphertext).is_err());
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let nonce = vec![0u8; AES_NONCE_SIZE];
        let ciphertext = vec![1u8; 32];
        let encrypted = EncryptedData::new(nonce, ciphertext).unwrap();

        let bytes = encrypted.to_bytes();
        assert_eq!(bytes.len(), AES_NONCE_SIZE + 32);

        let reconstructed = EncryptedData::from_bytes(&bytes).unwrap();
        assert_eq!(reconstructed.nonce, encrypted.nonce);
        assert_eq!(reconstructed.ciphertext, encrypted.ciphertext);
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = EncryptionAtRest::generate_nonce().unwrap();
        let nonce2 = EncryptionAtRest::generate_nonce().unwrap();

        // Nonces should be different (extremely high probability)
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), AES_NONCE_SIZE);

        let nonce3 = EncryptionAtRest::generate_nonce_with_rand().unwrap();
        assert_eq!(nonce3.len(), AES_NONCE_SIZE);
    }

    #[test]
    fn test_encryption_at_rest_creation() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        // Test self-test
        assert!(encryptor.self_test().is_ok());
    }

    #[test]
    fn test_encryption_roundtrip() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        let plaintext = b"Hello, DataFold encryption at rest!";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encryption_bytes_roundtrip() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        let plaintext = b"Test data for byte-level encryption";
        let encrypted_bytes = encryptor.encrypt_bytes(plaintext).unwrap();
        let decrypted = encryptor.decrypt_bytes(&encrypted_bytes).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encryption_different_nonces() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        let plaintext = b"Same plaintext, different nonces";
        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext due to different nonces
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // Both should decrypt to the same plaintext
        let decrypted1 = encryptor.decrypt(&encrypted1).unwrap();
        let decrypted2 = encryptor.decrypt(&encrypted2).unwrap();
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(plaintext, &decrypted1[..]);
    }

    #[test]
    fn test_encryption_authentication_failure() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        let plaintext = b"Test data for authentication";
        let mut encrypted = encryptor.encrypt(plaintext).unwrap();

        // Corrupt the ciphertext
        if let Some(last_byte) = encrypted.ciphertext.last_mut() {
            *last_byte = last_byte.wrapping_add(1);
        }

        // Decryption should fail due to authentication failure
        assert!(encryptor.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_key_fingerprint() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        let fingerprint1 = encryptor.key_fingerprint().unwrap();
        let fingerprint2 = encryptor.key_fingerprint().unwrap();

        // Fingerprints should be identical for the same key
        assert_eq!(fingerprint1, fingerprint2);
        assert_eq!(fingerprint1.len(), 32);

        // Different key should produce different fingerprint
        let mut different_key = key;
        different_key[0] = different_key[0].wrapping_add(1);
        let different_encryptor = EncryptionAtRest::new(different_key).unwrap();
        let different_fingerprint = different_encryptor.key_fingerprint().unwrap();
        assert_ne!(fingerprint1, different_fingerprint);
    }

    #[test]
    fn test_empty_and_large_data() {
        let key = create_test_key();
        let encryptor = EncryptionAtRest::new(key).unwrap();

        // Test empty data
        let empty_data = b"";
        let encrypted = encryptor.encrypt(empty_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(empty_data, &decrypted[..]);

        // Test large data (but within limits)
        let large_data = vec![42u8; 1024 * 1024]; // 1 MB
        let encrypted = encryptor.encrypt(&large_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(large_data, decrypted);

        // Test oversized data (should fail)
        let oversized_data = vec![42u8; MAX_PLAINTEXT_SIZE + 1];
        assert!(encryptor.encrypt(&oversized_data).is_err());
    }
}