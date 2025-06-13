//! Cryptographic metadata storage and retrieval for DataFold databases
//!
//! This module provides secure storage and retrieval of cryptographic metadata,
//! including master public keys, algorithm information, and crypto-related
//! configuration data within the database metadata system.

use super::core::DbOperations;
use crate::crypto::{CryptoError, CryptoResult, PublicKey};
use crate::schema::SchemaError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Version constant for crypto metadata schema
const CRYPTO_METADATA_VERSION: u32 = 1;

/// Key prefix for crypto metadata in the metadata tree
const CRYPTO_METADATA_PREFIX: &str = "crypto_";

/// Specific key for master public key storage
const MASTER_PUBLIC_KEY_KEY: &str = "crypto_master_public_key";

/// Specific key for crypto metadata version
const CRYPTO_VERSION_KEY: &str = "crypto_version";

/// Comprehensive cryptographic metadata stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoMetadata {
    /// Version of the crypto metadata schema
    pub version: u32,
    /// Master public key for the database
    pub master_public_key: PublicKey,
    /// Signature algorithm used (e.g., "Ed25519")
    pub signature_algorithm: String,
    /// Key derivation method used (e.g., "Random", "Argon2id")
    pub key_derivation_method: String,
    /// Timestamp when crypto was initialized
    pub created_at: DateTime<Utc>,
    /// Additional metadata for future extensibility
    pub additional_metadata: HashMap<String, String>,
    /// Checksum for integrity verification
    pub checksum: String,
}

impl CryptoMetadata {
    /// Create new crypto metadata with the given public key
    pub fn new(master_public_key: PublicKey, key_derivation_method: String) -> CryptoResult<Self> {
        let created_at = Utc::now();
        let signature_algorithm = "Ed25519".to_string();
        let additional_metadata = HashMap::new();

        let mut metadata = Self {
            version: CRYPTO_METADATA_VERSION,
            master_public_key,
            signature_algorithm,
            key_derivation_method,
            created_at,
            additional_metadata,
            checksum: String::new(), // Will be computed below
        };

        // Compute checksum for integrity verification
        metadata.checksum = metadata.compute_checksum()?;

        Ok(metadata)
    }

    /// Compute integrity checksum for the metadata
    pub fn compute_checksum(&self) -> CryptoResult<String> {
        use sha2::{Digest, Sha256};
        use std::collections::BTreeMap;

        // Create a deterministic serialization with sorted additional_metadata
        let sorted_additional_metadata: BTreeMap<String, String> = self
            .additional_metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Create a serializable structure with sorted metadata
        let serializable_data = serde_json::json!({
            "version": self.version,
            "master_public_key": self.master_public_key,
            "signature_algorithm": self.signature_algorithm,
            "key_derivation_method": self.key_derivation_method,
            "created_at": self.created_at,
            "additional_metadata": sorted_additional_metadata
        });

        let serialized = serde_json::to_vec(&serializable_data).map_err(|e| {
            CryptoError::InvalidInput(format!("Failed to serialize metadata for checksum: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();

        Ok(hex::encode(result))
    }

    /// Verify the integrity of the metadata
    pub fn verify_integrity(&self) -> CryptoResult<bool> {
        let computed_checksum = self.compute_checksum()?;
        Ok(computed_checksum == self.checksum)
    }

    /// Add additional metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) -> CryptoResult<()> {
        self.additional_metadata.insert(key, value);
        self.checksum = self.compute_checksum()?;
        Ok(())
    }
}

/// Extension trait for DbOperations to handle crypto metadata
impl DbOperations {
    /// Store crypto metadata in the database
    pub fn store_crypto_metadata(&self, metadata: &CryptoMetadata) -> Result<(), SchemaError> {
        // Verify integrity before storing
        metadata.verify_integrity().map_err(|e| {
            SchemaError::InvalidData(format!("Crypto metadata integrity check failed: {}", e))
        })?;

        // Store the complete metadata
        self.store_in_tree(&self.metadata_tree, MASTER_PUBLIC_KEY_KEY, metadata)?;

        // Store version separately for quick access
        self.store_in_tree(&self.metadata_tree, CRYPTO_VERSION_KEY, &metadata.version)?;

        Ok(())
    }

    /// Retrieve crypto metadata from the database
    pub fn get_crypto_metadata(&self) -> Result<Option<CryptoMetadata>, SchemaError> {
        match self.get_from_tree(&self.metadata_tree, MASTER_PUBLIC_KEY_KEY)? {
            Some(metadata) => {
                let metadata: CryptoMetadata = metadata;

                // Verify integrity after retrieval
                metadata.verify_integrity().map_err(|e| {
                    SchemaError::InvalidData(format!(
                        "Retrieved crypto metadata failed integrity check: {}",
                        e
                    ))
                })?;

                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    /// Get just the master public key (commonly used operation)
    pub fn get_master_public_key(&self) -> Result<Option<PublicKey>, SchemaError> {
        match self.get_crypto_metadata()? {
            Some(metadata) => Ok(Some(metadata.master_public_key)),
            None => Ok(None),
        }
    }

    /// Check if crypto metadata exists
    pub fn has_crypto_metadata(&self) -> Result<bool, SchemaError> {
        self.exists_in_tree(&self.metadata_tree, MASTER_PUBLIC_KEY_KEY)
    }

    /// Get crypto metadata version
    pub fn get_crypto_version(&self) -> Result<Option<u32>, SchemaError> {
        self.get_from_tree(&self.metadata_tree, CRYPTO_VERSION_KEY)
    }

    /// Update additional metadata in existing crypto metadata
    pub fn update_crypto_additional_metadata(
        &self,
        key: String,
        value: String,
    ) -> Result<(), SchemaError> {
        match self.get_crypto_metadata()? {
            Some(mut metadata) => {
                metadata.add_metadata(key, value).map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to update additional metadata: {}", e))
                })?;
                self.store_crypto_metadata(&metadata)
            }
            None => Err(SchemaError::InvalidData(
                "No crypto metadata exists to update".to_string(),
            )),
        }
    }

    /// Delete crypto metadata (for testing or migration purposes)
    pub fn delete_crypto_metadata(&self) -> Result<bool, SchemaError> {
        let key_deleted = self.delete_from_tree(&self.metadata_tree, MASTER_PUBLIC_KEY_KEY)?;
        let _version_deleted = self.delete_from_tree(&self.metadata_tree, CRYPTO_VERSION_KEY)?;
        Ok(key_deleted)
    }

    /// List all crypto-related metadata keys
    pub fn list_crypto_metadata_keys(&self) -> Result<Vec<String>, SchemaError> {
        let all_keys = self.list_keys_in_tree(&self.metadata_tree)?;
        let crypto_keys: Vec<String> = all_keys
            .into_iter()
            .filter(|key| key.starts_with(CRYPTO_METADATA_PREFIX))
            .collect();
        Ok(crypto_keys)
    }

    /// Get crypto metadata statistics
    pub fn get_crypto_metadata_stats(&self) -> Result<HashMap<String, String>, SchemaError> {
        let mut stats = HashMap::new();

        match self.get_crypto_metadata()? {
            Some(metadata) => {
                stats.insert("crypto_enabled".to_string(), "true".to_string());
                stats.insert("version".to_string(), metadata.version.to_string());
                stats.insert(
                    "signature_algorithm".to_string(),
                    metadata.signature_algorithm.clone(),
                );
                stats.insert(
                    "key_derivation_method".to_string(),
                    metadata.key_derivation_method.clone(),
                );
                stats.insert("created_at".to_string(), metadata.created_at.to_rfc3339());
                stats.insert(
                    "additional_entries".to_string(),
                    metadata.additional_metadata.len().to_string(),
                );
                stats.insert(
                    "integrity_verified".to_string(),
                    metadata.verify_integrity().unwrap_or(false).to_string(),
                );
            }
            None => {
                stats.insert("crypto_enabled".to_string(), "false".to_string());
            }
        }

        Ok(stats)
    }

    /// Migrate crypto metadata to a new version (for future schema updates)
    pub fn migrate_crypto_metadata(&self, _target_version: u32) -> Result<(), SchemaError> {
        // For now, just verify current metadata exists and is valid
        match self.get_crypto_metadata()? {
            Some(metadata) => {
                if metadata.version == CRYPTO_METADATA_VERSION {
                    Ok(())
                } else {
                    Err(SchemaError::InvalidData(format!(
                        "Crypto metadata migration from version {} not implemented",
                        metadata.version
                    )))
                }
            }
            None => Err(SchemaError::InvalidData(
                "No crypto metadata to migrate".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_keypair;
    use tempfile::tempdir;

    fn create_test_db_ops() -> DbOperations {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        DbOperations::new(db).unwrap()
    }

    #[test]
    fn test_crypto_metadata_creation() {
        let keypair = generate_master_keypair().unwrap();
        let metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Random".to_string()).unwrap();

        assert_eq!(metadata.version, CRYPTO_METADATA_VERSION);
        assert_eq!(metadata.signature_algorithm, "Ed25519");
        assert_eq!(metadata.key_derivation_method, "Random");
        assert!(!metadata.checksum.is_empty());
        assert!(metadata.verify_integrity().unwrap());
    }

    #[test]
    fn test_store_and_retrieve_crypto_metadata() {
        let db_ops = create_test_db_ops();
        let keypair = generate_master_keypair().unwrap();
        let metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Argon2id".to_string()).unwrap();

        // Store metadata
        assert!(db_ops.store_crypto_metadata(&metadata).is_ok());

        // Retrieve metadata
        let retrieved = db_ops.get_crypto_metadata().unwrap().unwrap();
        assert_eq!(retrieved.version, metadata.version);
        assert_eq!(
            retrieved.master_public_key.to_bytes(),
            metadata.master_public_key.to_bytes()
        );
        assert_eq!(
            retrieved.key_derivation_method,
            metadata.key_derivation_method
        );
        assert!(retrieved.verify_integrity().unwrap());
    }

    #[test]
    fn test_get_master_public_key() {
        let db_ops = create_test_db_ops();
        let keypair = generate_master_keypair().unwrap();
        let public_key = keypair.public_key().clone();
        let metadata = CryptoMetadata::new(public_key.clone(), "Random".to_string()).unwrap();

        // Initially no key
        assert!(db_ops.get_master_public_key().unwrap().is_none());

        // Store and retrieve
        db_ops.store_crypto_metadata(&metadata).unwrap();
        let retrieved_key = db_ops.get_master_public_key().unwrap().unwrap();
        assert_eq!(retrieved_key.to_bytes(), public_key.to_bytes());
    }

    #[test]
    fn test_has_crypto_metadata() {
        let db_ops = create_test_db_ops();

        // Initially no metadata
        assert!(!db_ops.has_crypto_metadata().unwrap());

        // Store metadata
        let keypair = generate_master_keypair().unwrap();
        let metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Random".to_string()).unwrap();
        db_ops.store_crypto_metadata(&metadata).unwrap();

        // Now has metadata
        assert!(db_ops.has_crypto_metadata().unwrap());
    }

    #[test]
    fn test_crypto_metadata_integrity() {
        let keypair = generate_master_keypair().unwrap();
        let mut metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Random".to_string()).unwrap();

        // Initially valid
        assert!(metadata.verify_integrity().unwrap());

        // Corrupt checksum
        metadata.checksum = "invalid".to_string();
        assert!(!metadata.verify_integrity().unwrap());
    }

    #[test]
    fn test_update_additional_metadata() {
        let db_ops = create_test_db_ops();
        let keypair = generate_master_keypair().unwrap();
        let metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Random".to_string()).unwrap();

        // Store initial metadata
        db_ops.store_crypto_metadata(&metadata).unwrap();

        // Update additional metadata
        db_ops
            .update_crypto_additional_metadata("test_key".to_string(), "test_value".to_string())
            .unwrap();

        // Verify update
        let updated_metadata = db_ops.get_crypto_metadata().unwrap().unwrap();
        assert_eq!(
            updated_metadata.additional_metadata.get("test_key"),
            Some(&"test_value".to_string())
        );
        assert!(updated_metadata.verify_integrity().unwrap());
    }

    #[test]
    fn test_crypto_metadata_stats() {
        let db_ops = create_test_db_ops();

        // No crypto metadata
        let stats = db_ops.get_crypto_metadata_stats().unwrap();
        assert_eq!(stats.get("crypto_enabled"), Some(&"false".to_string()));

        // With crypto metadata
        let keypair = generate_master_keypair().unwrap();
        let metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Argon2id".to_string()).unwrap();
        db_ops.store_crypto_metadata(&metadata).unwrap();

        let stats = db_ops.get_crypto_metadata_stats().unwrap();
        assert_eq!(stats.get("crypto_enabled"), Some(&"true".to_string()));
        assert_eq!(
            stats.get("signature_algorithm"),
            Some(&"Ed25519".to_string())
        );
        assert_eq!(
            stats.get("key_derivation_method"),
            Some(&"Argon2id".to_string())
        );
        assert_eq!(stats.get("integrity_verified"), Some(&"true".to_string()));
    }

    #[test]
    fn test_delete_crypto_metadata() {
        let db_ops = create_test_db_ops();
        let keypair = generate_master_keypair().unwrap();
        let metadata =
            CryptoMetadata::new(keypair.public_key().clone(), "Random".to_string()).unwrap();

        // Store and verify exists
        db_ops.store_crypto_metadata(&metadata).unwrap();
        assert!(db_ops.has_crypto_metadata().unwrap());

        // Delete and verify gone
        assert!(db_ops.delete_crypto_metadata().unwrap());
        assert!(!db_ops.has_crypto_metadata().unwrap());
    }
}
