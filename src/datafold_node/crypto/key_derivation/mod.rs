//! Enhanced BLAKE3-based key derivation for DataFold encryption at rest
//!
//! This module provides secure key derivation functionality that integrates with
//! PBI 8's master key infrastructure, supporting multiple encryption contexts
//! and comprehensive error handling.

use super::encryption_core::{EncryptionAtRest, AES_KEY_SIZE};
use crate::config::crypto::{CryptoConfig, MasterKeyConfig};
use crate::crypto::{CryptoError, CryptoResult, MasterKeyPair};
use blake3::Hasher;
use std::collections::HashMap;

pub mod legacy;
pub mod contexts;

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
    pub fn new(
        master_keypair: &MasterKeyPair,
        crypto_config: &CryptoConfig,
    ) -> CryptoResult<Self> {
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
    pub fn from_bytes(
        master_key_bytes: [u8; 32],
        crypto_config: &CryptoConfig,
    ) -> CryptoResult<Self> {
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
    pub fn create_encryptor(
        &self,
        context: &str,
        salt: Option<&[u8]>,
    ) -> CryptoResult<EncryptionAtRest> {
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
        crypto_config
            .validate()
            .map_err(|e| CryptoError::KeyDerivation {
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
            let encrypted =
                encryptor
                    .encrypt(test_data)
                    .map_err(|e| CryptoError::KeyGeneration {
                        message: format!("Encryption failed for context '{}': {}", context, e),
                    })?;

            let decrypted =
                encryptor
                    .decrypt(&encrypted)
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_keypair;

    #[test]
    fn test_key_derivation() {
        let master_key = b"test_master_key_material_32bytes";
        let context = "test_context";
        let salt = Some(b"test_salt".as_slice());

        let derived_key1 = legacy::derive_encryption_key(master_key, context, salt);
        let derived_key2 = legacy::derive_encryption_key(master_key, context, salt);

        // Same inputs should produce same key
        assert_eq!(derived_key1, derived_key2);

        // Different context should produce different key
        let different_key =
            legacy::derive_encryption_key(master_key, "different_context", salt);
        assert_ne!(derived_key1, different_key);

        // No salt should produce different key
        let no_salt_key = legacy::derive_encryption_key(master_key, context, None);
        assert_ne!(derived_key1, no_salt_key);
    }

    #[test]
    fn test_multiple_key_derivation() {
        let master_key = b"test_master_key_material_32bytes";
        let contexts = &["context1", "context2", "context3"];
        let salt = Some(b"test_salt".as_slice());

        let keys = legacy::derive_multiple_keys(master_key, contexts, salt);
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
        let encryption_key = legacy::derive_encryption_key(
            &master_key_bytes,
            contexts::ATOM_DATA,
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
        let schema_key = legacy::derive_encryption_key(
            &master_key_bytes,
            contexts::SCHEMA_METADATA,
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
        let manager = KeyDerivationManager::new(&master_keys, &crypto_config)
            .expect("Should create key derivation manager");

        // Test creation from raw bytes
        let master_key_bytes = master_keys.secret_key_bytes();
        let manager2 =
            KeyDerivationManager::from_bytes(master_key_bytes, &crypto_config)
                .expect("Should create key derivation manager from bytes");

        // Both should produce the same fingerprint
        assert_eq!(
            manager.master_key_fingerprint(),
            manager2.master_key_fingerprint()
        );
    }

    #[test]
    fn test_key_derivation_manager_key_derivation() {
        use crate::config::crypto::CryptoConfig;

        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        let manager =
            KeyDerivationManager::new(&master_keys, &crypto_config).unwrap();

        // Test single key derivation
        let key1 = manager.derive_key(contexts::ATOM_DATA, None);
        let key2 = manager.derive_key(contexts::SCHEMA_METADATA, None);

        // Keys should be different for different contexts
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), AES_KEY_SIZE);
        assert_eq!(key2.len(), AES_KEY_SIZE);

        // Same context should produce same key
        let key1_repeat = manager.derive_key(contexts::ATOM_DATA, None);
        assert_eq!(key1, key1_repeat);

        // Different salt should produce different key
        let key1_salted =
            manager.derive_key(contexts::ATOM_DATA, Some(b"test_salt"));
        assert_ne!(key1, key1_salted);
    }

    #[test]
    fn test_key_derivation_manager_multiple_keys() {
        use crate::config::crypto::CryptoConfig;

        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();
        let manager =
            KeyDerivationManager::new(&master_keys, &crypto_config).unwrap();

        let contexts = &[
            contexts::ATOM_DATA,
            contexts::SCHEMA_METADATA,
            contexts::BACKUP_DATA,
        ];

        let keys = manager.derive_multiple_keys(contexts, None);

        assert_eq!(keys.len(), 3);
        assert!(keys.contains_key(contexts::ATOM_DATA));
        assert!(keys.contains_key(contexts::SCHEMA_METADATA));
        assert!(keys.contains_key(contexts::BACKUP_DATA));

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
        let manager =
            KeyDerivationManager::new(&master_keys, &crypto_config).unwrap();

        // Test single encryptor creation
        let encryptor = manager
            .create_encryptor(contexts::ATOM_DATA, None)
            .expect("Should create encryptor");

        // Test encryption/decryption
        let test_data = b"test data for encryption";
        let encrypted = encryptor.encrypt(test_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(test_data, &decrypted[..]);

        // Test multiple encryptors creation
        let contexts = &[
            contexts::ATOM_DATA,
            contexts::SCHEMA_METADATA,
        ];

        let encryptors = manager
            .create_multiple_encryptors(contexts, None)
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
        let all_contexts = contexts::all_contexts();

        // Should have all expected contexts
        assert!(all_contexts.contains(&contexts::ATOM_DATA));
        assert!(all_contexts.contains(&contexts::SCHEMA_METADATA));
        assert!(all_contexts.contains(&contexts::INDEX_DATA));
        assert!(all_contexts.contains(&contexts::BACKUP_DATA));
        assert!(all_contexts.contains(&contexts::TEMP_DATA));
        assert!(all_contexts.contains(&contexts::TRANSFORM_QUEUE));
        assert!(all_contexts.contains(&contexts::NETWORK_MESSAGES));
        assert!(all_contexts.contains(&contexts::CONFIG_DATA));

        // Should have at least 8 contexts
        assert!(all_contexts.len() >= 8);

        // All contexts should be unique
        let mut unique_contexts = std::collections::HashSet::new();
        for context in all_contexts {
            assert!(
                unique_contexts.insert(context),
                "Duplicate context: {}",
                context
            );
        }
    }

    #[test]
    fn test_integration_create_encryption_system() {
        use crate::config::crypto::CryptoConfig;

        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();

        let contexts = &[
            contexts::ATOM_DATA,
            contexts::SCHEMA_METADATA,
            contexts::BACKUP_DATA,
        ];

        let (manager, encryptors) = integration::create_encryption_system(
            &crypto_config,
            &master_keys,
            contexts,
        )
        .expect("Should create encryption system");

        assert_eq!(encryptors.len(), 3);

        // Test the system
        integration::test_encryption_system(&manager, &encryptors)
            .expect("System test should pass");
    }

    #[test]
    fn test_integration_create_default_encryption_system() {
        use crate::config::crypto::CryptoConfig;

        let master_keys = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();

        let (manager, encryptors) = integration::create_default_encryption_system(
            &crypto_config,
            &master_keys,
        )
        .expect("Should create default encryption system");

        // Should have all standard contexts
        let all_contexts = contexts::all_contexts();
        assert_eq!(encryptors.len(), all_contexts.len());

        for context in all_contexts {
            assert!(
                encryptors.contains_key(*context),
                "Missing context: {}",
                context
            );
        }

        // Test the complete system
        integration::test_encryption_system(&manager, &encryptors)
            .expect("Default system test should pass");
    }

    #[test]
    fn test_key_derivation_with_different_configs() {
        use crate::config::crypto::CryptoConfig;

        let master_keys = generate_master_keypair().unwrap();

        // Test with different crypto configurations
        let config1 = CryptoConfig::with_random_key();
        let config2 = CryptoConfig::with_enhanced_security("test-passphrase".to_string());

        let manager1 = KeyDerivationManager::new(&master_keys, &config1).unwrap();
        let manager2 = KeyDerivationManager::new(&master_keys, &config2).unwrap();

        // Different configs should produce different derived keys
        let key1 = manager1.derive_key(contexts::ATOM_DATA, None);
        let key2 = manager2.derive_key(contexts::ATOM_DATA, None);

        assert_ne!(
            key1, key2,
            "Different configs should produce different keys"
        );

        // But same master key fingerprint (since same master key)
        assert_eq!(
            manager1.master_key_fingerprint(),
            manager2.master_key_fingerprint()
        );
    }

    #[test]
    fn test_key_derivation_security_properties() {
        use crate::config::crypto::CryptoConfig;

        let master_keys1 = generate_master_keypair().unwrap();
        let master_keys2 = generate_master_keypair().unwrap();
        let crypto_config = CryptoConfig::with_random_key();

        let manager1 =
            KeyDerivationManager::new(&master_keys1, &crypto_config).unwrap();
        let manager2 =
            KeyDerivationManager::new(&master_keys2, &crypto_config).unwrap();

        // Different master keys should produce different derived keys
        let key1 = manager1.derive_key(contexts::ATOM_DATA, None);
        let key2 = manager2.derive_key(contexts::ATOM_DATA, None);

        assert_ne!(
            key1, key2,
            "Different master keys should produce different derived keys"
        );

        // Different master key fingerprints
        assert_ne!(
            manager1.master_key_fingerprint(),
            manager2.master_key_fingerprint()
        );

        // Keys should be uniformly distributed (basic check)
        let key = manager1.derive_key(contexts::ATOM_DATA, None);
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
            contexts::ATOM_DATA,
            contexts::SCHEMA_METADATA,
            contexts::BACKUP_DATA,
        ];

        // Test single key derivation
        let key = legacy::derive_encryption_key(
            &master_key_bytes,
            contexts::ATOM_DATA,
            None,
        );

        let encryptor = EncryptionAtRest::new(key).unwrap();
        assert!(encryptor.self_test().is_ok());

        // Test multiple key derivation
        let keys = legacy::derive_multiple_keys(&master_key_bytes, contexts, None);
        assert_eq!(keys.len(), 3);

        // All keys should be different
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j]);
            }
        }
    }
}