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
//! use datafold_node::encryption_at_rest::EncryptionAtRest;
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
    fn as_bytes(&self) -> &[u8; AES_KEY_SIZE] {
        &self.key
    }

    /// Convert to aes-gcm Key type
    fn to_aes_key(&self) -> Key<Aes256Gcm> {
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
        let ciphertext = self
            .cipher
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

/// Enhanced BLAKE3-based key derivation for DataFold encryption at rest
///
/// This module provides secure key derivation functionality that integrates with
/// PBI 8's master key infrastructure, supporting multiple encryption contexts
/// and comprehensive error handling.
pub mod key_derivation {
    use super::*;
    use crate::crypto::{MasterKeyPair, CryptoError, CryptoResult};
    use crate::config::crypto::{CryptoConfig, MasterKeyConfig};
    use blake3::Hasher;
    use std::collections::HashMap;

    /// Enhanced key derivation manager that integrates with PBI 8 master keys
    pub struct KeyDerivationManager {
        master_key_bytes: [u8; 32],
        config_hash: [u8; 32],
    }

    impl KeyDerivationManager {
        /// Create a new key derivation manager from an Ed25519 master key pair
        ///
        /// # Arguments
        /// * `master_keypair` - The Ed25519 master key pair from PBI 8
        /// * `crypto_config` - The crypto configuration for additional entropy
        ///
        /// # Returns
        /// * `Ok(KeyDerivationManager)` - Ready to derive keys
        /// * `Err(CryptoError)` - If key extraction fails
        pub fn new(master_keypair: &MasterKeyPair, crypto_config: &CryptoConfig) -> CryptoResult<Self> {
            // Extract the Ed25519 private key bytes as master key material
            let master_key_bytes = master_keypair.secret_key_bytes();
            
            // Create a configuration hash for additional entropy
            let config_hash = Self::compute_config_hash(crypto_config)?;
            
            Ok(Self {
                master_key_bytes,
                config_hash,
            })
        }

        /// Create a key derivation manager from raw master key bytes
        ///
        /// # Arguments
        /// * `master_key_bytes` - Raw 32-byte master key material
        /// * `crypto_config` - The crypto configuration for additional entropy
        ///
        /// # Returns
        /// * `Ok(KeyDerivationManager)` - Ready to derive keys
        /// * `Err(CryptoError)` - If validation fails
        pub fn from_bytes(master_key_bytes: [u8; 32], crypto_config: &CryptoConfig) -> CryptoResult<Self> {
            let config_hash = Self::compute_config_hash(crypto_config)?;
            
            Ok(Self {
                master_key_bytes,
                config_hash,
            })
        }

        /// Derive a single AES-256 encryption key for a specific context
        ///
        /// # Arguments
        /// * `context` - The encryption context (e.g., "atom_data", "schema_metadata")
        /// * `salt` - Optional additional salt for key strengthening
        ///
        /// # Returns
        /// * `[u8; 32]` - A 256-bit AES encryption key
        pub fn derive_key(&self, context: &str, salt: Option<&[u8]>) -> [u8; AES_KEY_SIZE] {
            let mut hasher = Hasher::new();

            // Domain separation prefix
            hasher.update(b"DataFold_KeyDerivation_v1:");

            // Add master key material
            hasher.update(&self.master_key_bytes);

            // Add configuration hash for additional entropy
            hasher.update(&self.config_hash);

            // Add context for key separation
            hasher.update(context.as_bytes());

            // Add optional salt
            if let Some(salt_bytes) = salt {
                hasher.update(b":salt:");
                hasher.update(salt_bytes);
            }

            // Derive the key
            let mut derived_key = [0u8; AES_KEY_SIZE];
            let output = hasher.finalize();
            derived_key.copy_from_slice(&output.as_bytes()[..AES_KEY_SIZE]);

            derived_key
        }

        /// Derive multiple encryption keys for different contexts efficiently
        ///
        /// # Arguments
        /// * `contexts` - List of encryption contexts
        /// * `salt` - Optional salt applied to all derivations
        ///
        /// # Returns
        /// * `HashMap<String, [u8; 32]>` - Map of context names to derived keys
        pub fn derive_multiple_keys(
            &self,
            contexts: &[&str],
            salt: Option<&[u8]>,
        ) -> HashMap<String, [u8; AES_KEY_SIZE]> {
            contexts
                .iter()
                .map(|&context| (context.to_string(), self.derive_key(context, salt)))
                .collect()
        }

        /// Create an EncryptionAtRest instance for a specific context
        ///
        /// # Arguments
        /// * `context` - The encryption context
        /// * `salt` - Optional salt for key derivation
        ///
        /// # Returns
        /// * `Ok(EncryptionAtRest)` - Ready-to-use encryption manager
        /// * `Err(CryptoError)` - If encryption manager creation fails
        pub fn create_encryptor(&self, context: &str, salt: Option<&[u8]>) -> CryptoResult<EncryptionAtRest> {
            let derived_key = self.derive_key(context, salt);
            EncryptionAtRest::new(derived_key)
        }

        /// Create multiple EncryptionAtRest instances for different contexts
        ///
        /// # Arguments
        /// * `contexts` - List of encryption contexts
        /// * `salt` - Optional salt applied to all derivations
        ///
        /// # Returns
        /// * `Ok(HashMap<String, EncryptionAtRest>)` - Map of context names to encryptors
        /// * `Err(CryptoError)` - If any encryptor creation fails
        pub fn create_multiple_encryptors(
            &self,
            contexts: &[&str],
            salt: Option<&[u8]>,
        ) -> CryptoResult<HashMap<String, EncryptionAtRest>> {
            let mut encryptors = HashMap::new();
            
            for &context in contexts {
                let encryptor = self.create_encryptor(context, salt)?;
                encryptors.insert(context.to_string(), encryptor);
            }
            
            Ok(encryptors)
        }

        /// Get a fingerprint of the master key for identification
        ///
        /// Returns a SHA-256 hash of the master key material for identification
        /// without exposing the actual key.
        pub fn master_key_fingerprint(&self) -> [u8; 32] {
            use sha2::{Digest, Sha256};
            
            let mut hasher = Sha256::new();
            hasher.update(b"DataFold_MasterKey_Fingerprint:");
            hasher.update(self.master_key_bytes);
            
            let result = hasher.finalize();
            let mut fingerprint = [0u8; 32];
            fingerprint.copy_from_slice(&result);
            fingerprint
        }

        /// Compute a hash of the crypto configuration for additional entropy
        fn compute_config_hash(crypto_config: &CryptoConfig) -> CryptoResult<[u8; 32]> {
            use sha2::{Digest, Sha256};
            
            let mut hasher = Sha256::new();
            
            // Include enabled state
            hasher.update([crypto_config.enabled as u8]);
            
            // Include master key config type (but not sensitive data)
            match &crypto_config.master_key {
                MasterKeyConfig::Random => hasher.update(b"random"),
                MasterKeyConfig::Passphrase { .. } => hasher.update(b"passphrase"),
                MasterKeyConfig::External { .. } => hasher.update(b"external"),
            }
            
            // Include key derivation parameters
            if let Ok(params) = crypto_config.key_derivation.to_argon2_params() {
                hasher.update(params.memory_cost.to_le_bytes());
                hasher.update(params.time_cost.to_le_bytes());
                hasher.update(params.parallelism.to_le_bytes());
            }
            
            let result = hasher.finalize();
            let mut config_hash = [0u8; 32];
            config_hash.copy_from_slice(&result);
            Ok(config_hash)
        }
    }

    impl Drop for KeyDerivationManager {
        fn drop(&mut self) {
            // Zeroize sensitive key material
            use zeroize::Zeroize;
            self.master_key_bytes.zeroize();
            self.config_hash.zeroize();
        }
    }

    /// Legacy key derivation functions for backward compatibility
    pub mod legacy {
        use super::*;

        /// Derive an AES-256 encryption key from a master key using BLAKE3
        ///
        /// **Note**: This is a legacy function. Use `KeyDerivationManager` for new code.
        ///
        /// # Arguments
        /// * `master_key` - The master key material (e.g., from Ed25519 private key)
        /// * `context` - Context string for key separation (e.g., "database_encryption")
        /// * `salt` - Optional salt for additional entropy
        ///
        /// # Returns
        /// * `[u8; 32]` - A 256-bit encryption key suitable for AES-256-GCM
        pub fn derive_encryption_key(
            master_key: &[u8],
            context: &str,
            salt: Option<&[u8]>,
        ) -> [u8; AES_KEY_SIZE] {
            let mut hasher = Hasher::new();

            // Add master key material
            hasher.update(master_key);

            // Add context for key separation
            hasher.update(context.as_bytes());

            // Add salt if provided
            if let Some(salt_bytes) = salt {
                hasher.update(salt_bytes);
            }

            // Derive the key
            let mut derived_key = [0u8; AES_KEY_SIZE];
            let output = hasher.finalize();
            derived_key.copy_from_slice(&output.as_bytes()[..AES_KEY_SIZE]);

            derived_key
        }

        /// Derive multiple encryption keys for different purposes
        ///
        /// **Note**: This is a legacy function. Use `KeyDerivationManager` for new code.
        ///
        /// # Arguments
        /// * `master_key` - The master key material
        /// * `contexts` - List of context strings for different keys
        /// * `salt` - Optional salt for additional entropy
        ///
        /// # Returns
        /// * `Vec<[u8; 32]>` - Multiple encryption keys derived from the same master key
        pub fn derive_multiple_keys(
            master_key: &[u8],
            contexts: &[&str],
            salt: Option<&[u8]>,
        ) -> Vec<[u8; AES_KEY_SIZE]> {
            contexts
                .iter()
                .map(|&context| derive_encryption_key(master_key, context, salt))
                .collect()
        }
    }

    /// Standard encryption contexts for different data types
    pub mod contexts {
        /// Context for general database atom encryption
        pub const ATOM_DATA: &str = "datafold_atom_encryption_v1";

        /// Context for schema metadata encryption
        pub const SCHEMA_METADATA: &str = "datafold_schema_encryption_v1";

        /// Context for index data encryption
        pub const INDEX_DATA: &str = "datafold_index_encryption_v1";

        /// Context for backup data encryption
        pub const BACKUP_DATA: &str = "datafold_backup_encryption_v1";

        /// Context for temporary data encryption
        pub const TEMP_DATA: &str = "datafold_temp_encryption_v1";

        /// Context for transform queue encryption
        pub const TRANSFORM_QUEUE: &str = "datafold_transform_queue_encryption_v1";

        /// Context for network message encryption
        pub const NETWORK_MESSAGES: &str = "datafold_network_encryption_v1";

        /// Context for configuration data encryption
        pub const CONFIG_DATA: &str = "datafold_config_encryption_v1";

        /// Get all standard contexts as a slice
        pub fn all_contexts() -> &'static [&'static str] {
            &[
                ATOM_DATA,
                SCHEMA_METADATA,
                INDEX_DATA,
                BACKUP_DATA,
                TEMP_DATA,
                TRANSFORM_QUEUE,
                NETWORK_MESSAGES,
                CONFIG_DATA,
            ]
        }
    }

    /// Integration utilities for connecting PBI 8 and Task 9-2 systems
    pub mod integration {
        use super::*;

        /// Create a complete encryption system from a CryptoConfig and master keypair
        ///
        /// This function bridges PBI 8's crypto infrastructure with Task 9-2's encryption
        /// utilities, providing a high-level interface for setting up encryption at rest.
        ///
        /// # Arguments
        /// * `crypto_config` - The crypto configuration from PBI 8
        /// * `master_keypair` - The Ed25519 master key pair
        /// * `contexts` - List of encryption contexts to create encryptors for
        ///
        /// # Returns
        /// * `Ok((KeyDerivationManager, HashMap<String, EncryptionAtRest>))` - Complete encryption system
        /// * `Err(CryptoError)` - If system creation fails
        pub fn create_encryption_system(
            crypto_config: &CryptoConfig,
            master_keypair: &MasterKeyPair,
            contexts: &[&str],
        ) -> CryptoResult<(KeyDerivationManager, HashMap<String, EncryptionAtRest>)> {
            // Validate crypto config
            crypto_config.validate().map_err(|e| CryptoError::KeyDerivation {
                message: format!("Invalid crypto config: {}", e),
            })?;

            // Create key derivation manager
            let key_manager = KeyDerivationManager::new(master_keypair, crypto_config)?;
            
            // Create encryptors for all requested contexts
            let encryptors = key_manager.create_multiple_encryptors(contexts, None)?;
            
            Ok((key_manager, encryptors))
        }

        /// Create encryption system with default contexts
        ///
        /// Creates encryptors for all standard DataFold contexts.
        ///
        /// # Arguments
        /// * `crypto_config` - The crypto configuration from PBI 8
        /// * `master_keypair` - The Ed25519 master key pair
        ///
        /// # Returns
        /// * `Ok((KeyDerivationManager, HashMap<String, EncryptionAtRest>))` - Complete encryption system
        /// * `Err(CryptoError)` - If system creation fails
        pub fn create_default_encryption_system(
            crypto_config: &CryptoConfig,
            master_keypair: &MasterKeyPair,
        ) -> CryptoResult<(KeyDerivationManager, HashMap<String, EncryptionAtRest>)> {
            create_encryption_system(crypto_config, master_keypair, contexts::all_contexts())
        }

        /// Test the complete encryption system with a round-trip operation
        ///
        /// # Arguments
        /// * `key_manager` - The key derivation manager
        /// * `encryptors` - Map of encryption contexts to encryptors
        ///
        /// # Returns
        /// * `Ok(())` - If all tests pass
        /// * `Err(CryptoError)` - If any test fails
        pub fn test_encryption_system(
            key_manager: &KeyDerivationManager,
            encryptors: &HashMap<String, EncryptionAtRest>,
        ) -> CryptoResult<()> {
            let test_data = b"DataFold encryption system integration test";
            
            // Test each encryptor
            for (context, encryptor) in encryptors {
                // Test round-trip encryption
                let encrypted = encryptor.encrypt(test_data)
                    .map_err(|e| CryptoError::KeyGeneration {
                        message: format!("Encryption failed for context '{}': {}", context, e),
                    })?;
                
                let decrypted = encryptor.decrypt(&encrypted)
                    .map_err(|e| CryptoError::Signature {
                        message: format!("Decryption failed for context '{}': {}", context, e),
                    })?;
                
                if decrypted != test_data {
                    return Err(CryptoError::Signature {
                        message: format!("Round-trip test failed for context '{}'", context),
                    });
                }
            }
            
            // Test key derivation manager fingerprint
            let _fingerprint = key_manager.master_key_fingerprint();
            
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_keypair;

    fn create_test_key() -> [u8; AES_KEY_SIZE] {
        let mut key = [0u8; AES_KEY_SIZE];
        for i in 0..AES_KEY_SIZE {
            key[i] = (i % 256) as u8;
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

    #[test]
    fn test_key_derivation() {
        let master_key = b"test_master_key_material_32bytes";
        let context = "test_context";
        let salt = Some(b"test_salt".as_slice());

        let derived_key1 = key_derivation::legacy::derive_encryption_key(master_key, context, salt);
        let derived_key2 = key_derivation::legacy::derive_encryption_key(master_key, context, salt);

        // Same inputs should produce same key
        assert_eq!(derived_key1, derived_key2);

        // Different context should produce different key
        let different_key = key_derivation::legacy::derive_encryption_key(master_key, "different_context", salt);
        assert_ne!(derived_key1, different_key);

        // No salt should produce different key
        let no_salt_key = key_derivation::legacy::derive_encryption_key(master_key, context, None);
        assert_ne!(derived_key1, no_salt_key);
    }

    #[test]
    fn test_multiple_key_derivation() {
        let master_key = b"test_master_key_material_32bytes";
        let contexts = &["context1", "context2", "context3"];
        let salt = Some(b"test_salt".as_slice());

        let keys = key_derivation::legacy::derive_multiple_keys(master_key, contexts, salt);
        assert_eq!(keys.len(), 3);

        // All keys should be different
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }

    #[test]
    fn test_integration_with_master_keypair() {
        // Test integration with the existing crypto system using legacy functions
        let master_keypair = generate_master_keypair().unwrap();
        let master_key_bytes = master_keypair.secret_key_bytes();

        // Derive an encryption key from the master key using legacy function
        let encryption_key = key_derivation::legacy::derive_encryption_key(
            &master_key_bytes,
            key_derivation::contexts::ATOM_DATA,
            None,
        );

        // Create encryptor with derived key
        let encryptor = EncryptionAtRest::new(encryption_key).unwrap();

        // Test encryption/decryption
        let test_data = b"Integration test with master keypair";
        let encrypted = encryptor.encrypt(test_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(test_data, &decrypted[..]);
        assert!(encryptor.self_test().is_ok());
        
        // Test with different contexts produce different keys
        let schema_key = key_derivation::legacy::derive_encryption_key(
            &master_key_bytes,
            key_derivation::contexts::SCHEMA_METADATA,
            None,
        );
        
        assert_ne!(encryption_key, schema_key);
    }

    #[test]
    fn test_key_derivation_manager_creation() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        
        // Test creation from master keypair
        let manager = key_derivation::KeyDerivationManager::new(&master_keys, &crypto_config)
            .expect("Should create key derivation manager");
        
        // Test creation from raw bytes
        let master_key_bytes = master_keys.secret_key_bytes();
        let manager2 = key_derivation::KeyDerivationManager::from_bytes(master_key_bytes, &crypto_config)
            .expect("Should create key derivation manager from bytes");
        
        // Both should produce the same fingerprint
        assert_eq!(manager.master_key_fingerprint(), manager2.master_key_fingerprint());
    }

    #[test]
    fn test_key_derivation_manager_key_derivation() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        let manager = key_derivation::KeyDerivationManager::new(&master_keys, &crypto_config).unwrap();
        
        // Test single key derivation
        let key1 = manager.derive_key(key_derivation::contexts::ATOM_DATA, None);
        let key2 = manager.derive_key(key_derivation::contexts::SCHEMA_METADATA, None);
        
        // Keys should be different for different contexts
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), AES_KEY_SIZE);
        assert_eq!(key2.len(), AES_KEY_SIZE);
        
        // Same context should produce same key
        let key1_repeat = manager.derive_key(key_derivation::contexts::ATOM_DATA, None);
        assert_eq!(key1, key1_repeat);
        
        // Different salt should produce different key
        let key1_salted = manager.derive_key(key_derivation::contexts::ATOM_DATA, Some(b"test_salt"));
        assert_ne!(key1, key1_salted);
    }

    #[test]
    fn test_key_derivation_manager_multiple_keys() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        let manager = key_derivation::KeyDerivationManager::new(&master_keys, &crypto_config).unwrap();
        
        let contexts = &[
            key_derivation::contexts::ATOM_DATA,
            key_derivation::contexts::SCHEMA_METADATA,
            key_derivation::contexts::BACKUP_DATA,
        ];
        
        let keys = manager.derive_multiple_keys(contexts, None);
        
        assert_eq!(keys.len(), 3);
        assert!(keys.contains_key(key_derivation::contexts::ATOM_DATA));
        assert!(keys.contains_key(key_derivation::contexts::SCHEMA_METADATA));
        assert!(keys.contains_key(key_derivation::contexts::BACKUP_DATA));
        
        // All keys should be different
        let key_values: Vec<_> = keys.values().collect();
        for i in 0..key_values.len() {
            for j in (i + 1)..key_values.len() {
                assert_ne!(key_values[i], key_values[j]);
            }
        }
    }

    #[test]
    fn test_key_derivation_manager_create_encryptor() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        let manager = key_derivation::KeyDerivationManager::new(&master_keys, &crypto_config).unwrap();
        
        // Test single encryptor creation
        let encryptor = manager.create_encryptor(key_derivation::contexts::ATOM_DATA, None)
            .expect("Should create encryptor");
        
        // Test encryption/decryption
        let test_data = b"test data for encryption";
        let encrypted = encryptor.encrypt(test_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(test_data, &decrypted[..]);
        
        // Test multiple encryptors creation
        let contexts = &[
            key_derivation::contexts::ATOM_DATA,
            key_derivation::contexts::SCHEMA_METADATA,
        ];
        
        let encryptors = manager.create_multiple_encryptors(contexts, None)
            .expect("Should create multiple encryptors");
        
        assert_eq!(encryptors.len(), 2);
        
        // Test each encryptor works
        for (context, encryptor) in &encryptors {
            let encrypted = encryptor.encrypt(test_data).unwrap();
            let decrypted = encryptor.decrypt(&encrypted).unwrap();
            assert_eq!(test_data, &decrypted[..], "Failed for context: {}", context);
        }
    }

    #[test]
    fn test_key_derivation_contexts() {
        let all_contexts = key_derivation::contexts::all_contexts();
        
        // Should have all expected contexts
        assert!(all_contexts.contains(&key_derivation::contexts::ATOM_DATA));
        assert!(all_contexts.contains(&key_derivation::contexts::SCHEMA_METADATA));
        assert!(all_contexts.contains(&key_derivation::contexts::INDEX_DATA));
        assert!(all_contexts.contains(&key_derivation::contexts::BACKUP_DATA));
        assert!(all_contexts.contains(&key_derivation::contexts::TEMP_DATA));
        assert!(all_contexts.contains(&key_derivation::contexts::TRANSFORM_QUEUE));
        assert!(all_contexts.contains(&key_derivation::contexts::NETWORK_MESSAGES));
        assert!(all_contexts.contains(&key_derivation::contexts::CONFIG_DATA));
        
        // Should have at least 8 contexts
        assert!(all_contexts.len() >= 8);
        
        // All contexts should be unique
        let mut unique_contexts = std::collections::HashSet::new();
        for context in all_contexts {
            assert!(unique_contexts.insert(context), "Duplicate context: {}", context);
        }
    }

    #[test]
    fn test_integration_create_encryption_system() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        
        let contexts = &[
            key_derivation::contexts::ATOM_DATA,
            key_derivation::contexts::SCHEMA_METADATA,
            key_derivation::contexts::BACKUP_DATA,
        ];
        
        let (manager, encryptors) = key_derivation::integration::create_encryption_system(
            &crypto_config,
            &master_keys,
            contexts,
        ).expect("Should create encryption system");
        
        assert_eq!(encryptors.len(), 3);
        
        // Test the system
        key_derivation::integration::test_encryption_system(&manager, &encryptors)
            .expect("System test should pass");
    }

    #[test]
    fn test_integration_create_default_encryption_system() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        
        let (manager, encryptors) = key_derivation::integration::create_default_encryption_system(
            &crypto_config,
            &master_keys,
        ).expect("Should create default encryption system");
        
        // Should have all standard contexts
        let all_contexts = key_derivation::contexts::all_contexts();
        assert_eq!(encryptors.len(), all_contexts.len());
        
        for context in all_contexts {
            assert!(encryptors.contains_key(*context), "Missing context: {}", context);
        }
        
        // Test the complete system
        key_derivation::integration::test_encryption_system(&manager, &encryptors)
            .expect("Default system test should pass");
    }

    #[test]
    fn test_key_derivation_with_different_configs() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys = generate_master_keypair().unwrap();
        
        // Test with different crypto configurations
        let config1 = CryptoConfig::with_random_key();
        let config2 = CryptoConfig::with_enhanced_security("test-passphrase".to_string());
        
        let manager1 = key_derivation::KeyDerivationManager::new(&master_keys, &config1).unwrap();
        let manager2 = key_derivation::KeyDerivationManager::new(&master_keys, &config2).unwrap();
        
        // Different configs should produce different derived keys
        let key1 = manager1.derive_key(key_derivation::contexts::ATOM_DATA, None);
        let key2 = manager2.derive_key(key_derivation::contexts::ATOM_DATA, None);
        
        assert_ne!(key1, key2, "Different configs should produce different keys");
        
        // But same master key fingerprint (since same master key)
        assert_eq!(manager1.master_key_fingerprint(), manager2.master_key_fingerprint());
    }

    #[test]
    fn test_key_derivation_security_properties() {
        use crate::config::crypto::CryptoConfig;
        
        let master_keys1 = generate_master_keypair().unwrap();
        let master_keys2 = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        
        let manager1 = key_derivation::KeyDerivationManager::new(&master_keys1, &crypto_config).unwrap();
        let manager2 = key_derivation::KeyDerivationManager::new(&master_keys2, &crypto_config).unwrap();
        
        // Different master keys should produce different derived keys
        let key1 = manager1.derive_key(key_derivation::contexts::ATOM_DATA, None);
        let key2 = manager2.derive_key(key_derivation::contexts::ATOM_DATA, None);
        
        assert_ne!(key1, key2, "Different master keys should produce different derived keys");
        
        // Different master key fingerprints
        assert_ne!(manager1.master_key_fingerprint(), manager2.master_key_fingerprint());
        
        // Keys should be uniformly distributed (basic check)
        let key = manager1.derive_key(key_derivation::contexts::ATOM_DATA, None);
        let zero_count = key.iter().filter(|&&b| b == 0).count();
        let ff_count = key.iter().filter(|&&b| b == 0xFF).count();
        
        // Should not be all zeros or all 0xFF (very low probability)
        assert!(zero_count < 32, "Key should not be all zeros");
        assert!(ff_count < 32, "Key should not be all 0xFF");
        assert!(zero_count < 16, "Too many zero bytes (poor distribution)");
        assert!(ff_count < 16, "Too many 0xFF bytes (poor distribution)");
    }

    #[test]
    fn test_legacy_key_derivation_compatibility() {
        let master_keys = generate_master_keypair().unwrap();
        let master_key_bytes = master_keys.secret_key_bytes();
        
        // Test legacy functions still work
        let contexts = &[
            key_derivation::contexts::ATOM_DATA,
            key_derivation::contexts::SCHEMA_METADATA,
            key_derivation::contexts::BACKUP_DATA,
        ];
        
        // Test single key derivation
        let key = key_derivation::legacy::derive_encryption_key(
            &master_key_bytes,
            key_derivation::contexts::ATOM_DATA,
            None,
        );
        
        let encryptor = EncryptionAtRest::new(key).unwrap();
        assert!(encryptor.self_test().is_ok());
        
        // Test multiple key derivation
        let keys = key_derivation::legacy::derive_multiple_keys(&master_key_bytes, contexts, None);
        assert_eq!(keys.len(), 3);
        
        // All keys should be different
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j]);
            }
        }
    }
}