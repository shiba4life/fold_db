//! Database encryption wrapper layer for DataFold with comprehensive backward compatibility
//!
//! This module provides a transparent encryption layer for database operations,
//! integrating with the key derivation system and AES-256-GCM encryption utilities.
//! It supports different encryption contexts for different data types while maintaining
//! full backward compatibility with existing unencrypted data.
//!
//! ## Features
//!
//! * Transparent encryption/decryption for database operations
//! * Support for multiple encryption contexts (atoms, schemas, metadata)
//! * Comprehensive backward compatibility with unencrypted data
//! * Multiple migration modes: gradual, full, and read-only compatibility
//! * Automatic detection of mixed encrypted/unencrypted environments
//! * Advanced migration utilities with integrity validation
//! * Integration with key derivation system from Task 9-3
//! * Uses AES-256-GCM encryption utilities from Task 9-2
//! * Enhanced error handling for backward compatibility scenarios
//! * Minimal changes to existing API surface
//!
//! ## Migration Modes
//!
//! * **Read-only compatibility**: Read both encrypted and unencrypted data seamlessly
//! * **Gradual migration**: Encrypt new data while preserving existing unencrypted data
//! * **Full migration**: Convert all existing data to encrypted format
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::db_operations::{DbOperations, EncryptionWrapper, MigrationMode};
//! use datafold::crypto::generate_master_keypair;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create database operations
//! let db = sled::open("test_db")?;
//! let db_ops = DbOperations::new(db)?;
//!
//! // Create encryption wrapper with gradual migration mode
//! let master_keypair = generate_master_keypair()?;
//! let encryption_wrapper = EncryptionWrapper::with_migration_mode(
//!     db_ops, &master_keypair, MigrationMode::Gradual
//! )?;
//!
//! // Store encrypted data (transparent to caller)
//! let data = b"sensitive data";
//! encryption_wrapper.store_encrypted_item("test_key", data, "atom_data")?;
//!
//! // Retrieve and decrypt data (transparent to caller)
//! let retrieved: Vec<u8> = encryption_wrapper.get_encrypted_item("test_key", "atom_data")?.unwrap();
//! assert_eq!(data, &retrieved[..]);
//! # Ok(())
//! # }
//! ```

use super::core::DbOperations;
use super::migration::{MigrationConfig, MigrationMode, MigrationStatus, MigrationUtils};
use super::encrypted_data_format::EncryptedDataFormat;
use super::contexts;
use crate::config::crypto::CryptoConfig;
use crate::crypto::{CryptoError, CryptoResult, MasterKeyPair};
use crate::datafold_node::crypto::encryption_at_rest::{
    key_derivation::KeyDerivationManager, EncryptionAtRest,
};
use crate::schema::SchemaError;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// Database encryption wrapper that provides transparent encryption with comprehensive backward compatibility
pub struct EncryptionWrapper {
    /// The underlying database operations
    db_ops: DbOperations,
    /// Key derivation manager for generating encryption keys
    #[allow(dead_code)]
    key_manager: KeyDerivationManager,
    /// Cached encryptors for different contexts
    pub(crate) encryptors: HashMap<String, EncryptionAtRest>,
    /// Whether encryption is enabled for new data
    encryption_enabled: bool,
    /// Current migration mode for backward compatibility
    migration_mode: MigrationMode,
    /// Migration configuration
    migration_config: MigrationConfig,
}

impl EncryptionWrapper {
    /// Create a new encryption wrapper with a master key pair
    pub fn new(db_ops: DbOperations, master_keypair: &MasterKeyPair) -> CryptoResult<Self> {
        // Use default crypto config if none is available
        let crypto_config = CryptoConfig::default();
        let key_manager = KeyDerivationManager::new(master_keypair, &crypto_config)?;

        // Pre-create encryptors for all contexts
        let encryptors = key_manager.create_multiple_encryptors(contexts::all_contexts(), None)?;

        Ok(Self {
            db_ops,
            key_manager,
            encryptors,
            encryption_enabled: true,
            migration_mode: MigrationMode::default(),
            migration_config: MigrationConfig::default(),
        })
    }

    /// Create a new encryption wrapper with a crypto config
    pub fn with_config(
        db_ops: DbOperations,
        master_keypair: &MasterKeyPair,
        crypto_config: &CryptoConfig,
    ) -> CryptoResult<Self> {
        let key_manager = KeyDerivationManager::new(master_keypair, crypto_config)?;

        let encryptors = key_manager.create_multiple_encryptors(contexts::all_contexts(), None)?;

        Ok(Self {
            db_ops,
            key_manager,
            encryptors,
            encryption_enabled: true,
            migration_mode: MigrationMode::default(),
            migration_config: MigrationConfig::default(),
        })
    }

    /// Create a new encryption wrapper with encryption disabled (legacy mode)
    pub fn without_encryption(db_ops: DbOperations) -> Self {
        // Create dummy key manager and encryptors (won't be used)
        let dummy_master_key = [0u8; 32];
        let crypto_config = CryptoConfig::default();
        let key_manager = KeyDerivationManager::from_bytes(dummy_master_key, &crypto_config)
            .expect("Failed to create dummy key manager");

        Self {
            db_ops,
            key_manager,
            encryptors: HashMap::new(),
            encryption_enabled: false,
            migration_mode: MigrationMode::ReadOnlyCompatibility,
            migration_config: MigrationConfig::default(),
        }
    }

    /// Create a new encryption wrapper with a specific migration mode
    pub fn with_migration_mode(
        db_ops: DbOperations,
        master_keypair: &MasterKeyPair,
        migration_mode: MigrationMode,
    ) -> CryptoResult<Self> {
        let crypto_config = CryptoConfig::default();
        let key_manager = KeyDerivationManager::new(master_keypair, &crypto_config)?;

        let encryptors = key_manager.create_multiple_encryptors(contexts::all_contexts(), None)?;

        let encryption_enabled = MigrationUtils::can_encrypt_new_data(migration_mode);

        Ok(Self {
            db_ops,
            key_manager,
            encryptors,
            encryption_enabled,
            migration_mode,
            migration_config: MigrationConfig::default(),
        })
    }

    /// Create a new encryption wrapper with full migration configuration
    pub fn with_migration_config(
        db_ops: DbOperations,
        master_keypair: &MasterKeyPair,
        migration_config: MigrationConfig,
    ) -> CryptoResult<Self> {
        // Validate the migration configuration
        MigrationUtils::validate_config(&migration_config)
            .map_err(|e| CryptoError::InvalidInput(format!("Invalid migration config: {}", e)))?;

        let crypto_config = CryptoConfig::default();
        let key_manager = KeyDerivationManager::new(master_keypair, &crypto_config)?;

        let encryptors = key_manager.create_multiple_encryptors(contexts::all_contexts(), None)?;

        let encryption_enabled = MigrationUtils::can_encrypt_new_data(migration_config.mode);

        Ok(Self {
            db_ops,
            key_manager,
            encryptors,
            encryption_enabled,
            migration_mode: migration_config.mode,
            migration_config,
        })
    }

    /// Get a reference to the underlying database operations
    pub fn db_ops(&self) -> &DbOperations {
        &self.db_ops
    }

    /// Get a reference to the metadata tree for testing
    #[cfg(test)]
    pub fn metadata_tree(&self) -> &sled::Tree {
        &self.db_ops.metadata_tree
    }

    /// Get a reference to the schemas tree for testing
    #[cfg(test)]
    pub fn schemas_tree(&self) -> &sled::Tree {
        &self.db_ops.schemas_tree
    }

    /// Get a reference to the transforms tree for testing
    #[cfg(test)]
    pub fn transforms_tree(&self) -> &sled::Tree {
        &self.db_ops.transforms_tree
    }

    /// Check if encryption is enabled
    pub fn is_encryption_enabled(&self) -> bool {
        self.encryption_enabled
    }

    /// Enable or disable encryption for new data
    pub fn set_encryption_enabled(&mut self, enabled: bool) {
        self.encryption_enabled = enabled;
    }

    /// Get the current migration mode
    pub fn migration_mode(&self) -> MigrationMode {
        self.migration_mode
    }

    /// Set the migration mode
    pub fn set_migration_mode(&mut self, mode: MigrationMode) {
        self.migration_mode = mode;
        // Update encryption enabled based on mode
        self.encryption_enabled = MigrationUtils::can_encrypt_new_data(mode);
    }

    /// Get the current migration configuration
    pub fn migration_config(&self) -> &MigrationConfig {
        &self.migration_config
    }

    /// Update the migration configuration
    pub fn set_migration_config(&mut self, config: MigrationConfig) {
        self.migration_mode = config.mode;
        self.migration_config = config;
        // Update encryption enabled based on mode
        self.encryption_enabled = MigrationUtils::can_encrypt_new_data(self.migration_mode);
    }

    /// Store encrypted data with a specific context
    pub fn store_encrypted_item<T: Serialize>(
        &self,
        key: &str,
        item: &T,
        context: &str,
    ) -> Result<(), SchemaError> {
        // Handle different migration modes
        match self.migration_mode {
            MigrationMode::ReadOnlyCompatibility => {
                // In read-only mode, never encrypt new data
                log::debug!(
                    "Storing item '{}' as unencrypted in read-only compatibility mode",
                    key
                );
                return self.db_ops.store_item(key, item);
            }
            MigrationMode::Gradual => {
                // In gradual mode, encrypt new data if encryption is enabled
                if !self.encryption_enabled {
                    log::debug!(
                        "Storing item '{}' as unencrypted (encryption disabled)",
                        key
                    );
                    return self.db_ops.store_item(key, item);
                }
                log::debug!(
                    "Storing item '{}' as encrypted in gradual migration mode",
                    key
                );
            }
            MigrationMode::Full => {
                // In full mode, always encrypt if possible
                if !self.encryption_enabled {
                    log::warn!(
                        "Cannot encrypt item '{}' in full migration mode: encryption disabled",
                        key
                    );
                    return Err(SchemaError::InvalidData(
                        "Full migration mode requires encryption to be enabled".to_string(),
                    ));
                }
                log::debug!("Storing item '{}' as encrypted in full migration mode", key);
            }
        }

        // Serialize the item
        let serialized = serde_json::to_vec(item)
            .map_err(|e| SchemaError::InvalidData(format!("Serialization failed: {}", e)))?;

        // Get the encryptor for this context
        let encryptor = self.encryptors.get(context).ok_or_else(|| {
            SchemaError::InvalidData(format!("Unknown encryption context: {}", context))
        })?;

        // Encrypt the data
        let encrypted_data = encryptor
            .encrypt(&serialized)
            .map_err(|e| SchemaError::InvalidData(format!("Encryption failed: {}", e)))?;

        // Create encrypted data format
        let encrypted_format = EncryptedDataFormat::new(context.to_string(), encrypted_data)
            .map_err(|e| {
                SchemaError::InvalidData(format!("Failed to create encrypted format: {}", e))
            })?;

        // Serialize to bytes
        let encrypted_bytes = encrypted_format.to_bytes().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to serialize encrypted data: {}", e))
        })?;

        // Store raw bytes
        self.db_ops
            .db()
            .insert(key.as_bytes(), encrypted_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Database insert failed: {}", e)))?;

        self.db_ops
            .db()
            .flush()
            .map_err(|e| SchemaError::InvalidData(format!("Database flush failed: {}", e)))?;

        Ok(())
    }

    /// Retrieve and decrypt data with a specific context
    pub fn get_encrypted_item<T: DeserializeOwned>(
        &self,
        key: &str,
        context: &str,
    ) -> Result<Option<T>, SchemaError> {
        match self.db_ops.db().get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                // Check if this is encrypted data
                if EncryptedDataFormat::is_encrypted_data(&bytes) {
                    // Decrypt the data
                    let encrypted_format =
                        EncryptedDataFormat::from_bytes(&bytes).map_err(|e| {
                            SchemaError::InvalidData(format!(
                                "Failed to parse encrypted data: {}",
                                e
                            ))
                        })?;

                    // Verify context matches
                    if encrypted_format.context() != context {
                        return Err(SchemaError::InvalidData(format!(
                            "Context mismatch: expected '{}', found '{}'",
                            context, encrypted_format.context()
                        )));
                    }

                    // Get the encryptor for this context
                    let encryptor = self.encryptors.get(context).ok_or_else(|| {
                        SchemaError::InvalidData(format!("Unknown encryption context: {}", context))
                    })?;

                    // Decrypt the data
                    let decrypted_bytes = encryptor
                        .decrypt(encrypted_format.encrypted_data())
                        .map_err(|e| {
                            SchemaError::InvalidData(format!("Decryption failed: {}", e))
                        })?;

                    // Deserialize
                    let item = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
                        SchemaError::InvalidData(format!("Deserialization failed: {}", e))
                    })?;

                    Ok(Some(item))
                } else {
                    // Backward compatibility: try to deserialize as unencrypted JSON
                    match serde_json::from_slice(&bytes) {
                        Ok(item) => Ok(Some(item)),
                        Err(e) => {
                            // Enhanced error handling based on migration mode
                            match self.migration_mode {
                                MigrationMode::ReadOnlyCompatibility => {
                                    // In read-only mode, return None for corrupted data instead of error
                                    log::warn!(
                                        "Corrupted data in read-only mode for key '{}': {}",
                                        key,
                                        e
                                    );
                                    Ok(None)
                                }
                                _ => {
                                    // In strict mode (gradual/full), return error for corrupted data
                                    Err(SchemaError::InvalidData(format!(
                                        "Failed to deserialize data: {}",
                                        e
                                    )))
                                }
                            }
                        }
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SchemaError::InvalidData(format!(
                "Database retrieval failed: {}",
                e
            ))),
        }
    }

    /// Store encrypted data in a specific tree
    pub fn store_encrypted_in_tree<T: Serialize>(
        &self,
        tree: &sled::Tree,
        key: &str,
        item: &T,
        context: &str,
    ) -> Result<(), SchemaError> {
        if !self.encryption_enabled {
            // Fall back to unencrypted storage
            return self.db_ops.store_in_tree(tree, key, item);
        }

        // Serialize the item
        let serialized = serde_json::to_vec(item)
            .map_err(|e| SchemaError::InvalidData(format!("Serialization failed: {}", e)))?;

        // Get the encryptor for this context
        let encryptor = self.encryptors.get(context).ok_or_else(|| {
            SchemaError::InvalidData(format!("Unknown encryption context: {}", context))
        })?;

        // Encrypt the data
        let encrypted_data = encryptor
            .encrypt(&serialized)
            .map_err(|e| SchemaError::InvalidData(format!("Encryption failed: {}", e)))?;

        // Create encrypted data format
        let encrypted_format = EncryptedDataFormat::new(context.to_string(), encrypted_data)
            .map_err(|e| {
                SchemaError::InvalidData(format!("Failed to create encrypted format: {}", e))
            })?;

        // Serialize to bytes
        let encrypted_bytes = encrypted_format.to_bytes().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to serialize encrypted data: {}", e))
        })?;

        // Store raw bytes
        tree.insert(key.as_bytes(), encrypted_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Tree insert failed: {}", e)))?;

        tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Tree flush failed: {}", e)))?;

        Ok(())
    }

    /// Retrieve and decrypt data from a specific tree
    pub fn get_encrypted_from_tree<T: DeserializeOwned>(
        &self,
        tree: &sled::Tree,
        key: &str,
        context: &str,
    ) -> Result<Option<T>, SchemaError> {
        match tree.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                // Check if this is encrypted data
                if EncryptedDataFormat::is_encrypted_data(&bytes) {
                    // Decrypt the data
                    let encrypted_format =
                        EncryptedDataFormat::from_bytes(&bytes).map_err(|e| {
                            SchemaError::InvalidData(format!(
                                "Failed to parse encrypted data: {}",
                                e
                            ))
                        })?;

                    // Verify context matches
                    if encrypted_format.context() != context {
                        return Err(SchemaError::InvalidData(format!(
                            "Context mismatch: expected '{}', found '{}'",
                            context, encrypted_format.context()
                        )));
                    }

                    // Get the encryptor for this context
                    let encryptor = self.encryptors.get(context).ok_or_else(|| {
                        SchemaError::InvalidData(format!("Unknown encryption context: {}", context))
                    })?;

                    // Decrypt the data
                    let decrypted_bytes = encryptor
                        .decrypt(encrypted_format.encrypted_data())
                        .map_err(|e| {
                            SchemaError::InvalidData(format!("Decryption failed: {}", e))
                        })?;

                    // Deserialize
                    let item = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
                        SchemaError::InvalidData(format!("Deserialization failed: {}", e))
                    })?;

                    Ok(Some(item))
                } else {
                    // Enhanced backward compatibility: try to deserialize as unencrypted JSON
                    match serde_json::from_slice(&bytes) {
                        Ok(item) => {
                            if self.migration_mode == MigrationMode::ReadOnlyCompatibility {
                                log::debug!(
                                    "Successfully read unencrypted data from tree for key '{}'",
                                    key
                                );
                            }
                            Ok(Some(item))
                        }
                        Err(e) => {
                            // Enhanced error handling for backward compatibility
                            let error_msg = format!(
                                "Failed to deserialize data from tree for key '{}' (tried both encrypted and unencrypted formats): {}",
                                key, e
                            );

                            if self.migration_mode == MigrationMode::ReadOnlyCompatibility {
                                log::warn!("{}", error_msg);
                                // In read-only mode, return None instead of error for corrupted data
                                Ok(None)
                            } else {
                                Err(SchemaError::InvalidData(error_msg))
                            }
                        }
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                let error_msg = format!("Tree retrieval failed for key '{}': {}", key, e);
                if self.migration_mode == MigrationMode::ReadOnlyCompatibility {
                    log::error!("{}", error_msg);
                    Ok(None) // Graceful degradation in read-only mode
                } else {
                    Err(SchemaError::InvalidData(error_msg))
                }
            }
        }
    }

    /// List all keys in a tree (works with both encrypted and unencrypted data)
    pub fn list_keys_in_tree(&self, tree: &sled::Tree) -> Result<Vec<String>, SchemaError> {
        self.db_ops.list_keys_in_tree(tree)
    }

    /// Delete an item from a tree (works with both encrypted and unencrypted data)
    pub fn delete_from_tree(&self, tree: &sled::Tree, key: &str) -> Result<bool, SchemaError> {
        self.db_ops.delete_from_tree(tree, key)
    }

    /// Check if a key exists in a tree (works with both encrypted and unencrypted data)
    pub fn exists_in_tree(&self, tree: &sled::Tree, key: &str) -> Result<bool, SchemaError> {
        self.db_ops.exists_in_tree(tree, key)
    }

    /// Get statistics about encryption usage
    pub fn get_encryption_stats(&self) -> Result<HashMap<String, u64>, SchemaError> {
        let mut stats = HashMap::new();

        // Count encrypted vs unencrypted items in main database
        let mut encrypted_count = 0u64;
        let mut unencrypted_count = 0u64;

        for result in self.db_ops.db().iter() {
            let (_, value) = result.map_err(|e| {
                SchemaError::InvalidData(format!("Failed to iterate database: {}", e))
            })?;

            if EncryptedDataFormat::is_encrypted_data(&value) {
                encrypted_count += 1;
            } else {
                unencrypted_count += 1;
            }
        }

        let total_items = encrypted_count + unencrypted_count;
        let is_mixed = encrypted_count > 0 && unencrypted_count > 0;
        let is_fully_encrypted = total_items > 0 && unencrypted_count == 0;

        stats.insert("encrypted_items".to_string(), encrypted_count);
        stats.insert("unencrypted_items".to_string(), unencrypted_count);
        stats.insert("total_items".to_string(), total_items);
        stats.insert(
            "encryption_enabled".to_string(),
            if self.encryption_enabled { 1 } else { 0 },
        );
        stats.insert(
            "is_mixed_environment".to_string(),
            if is_mixed { 1 } else { 0 },
        );
        stats.insert(
            "is_fully_encrypted".to_string(),
            if is_fully_encrypted { 1 } else { 0 },
        );
        stats.insert("migration_mode".to_string(), self.migration_mode as u64);
        stats.insert(
            "available_contexts".to_string(),
            self.encryptors.len() as u64,
        );

        Ok(stats)
    }

    /// Get comprehensive migration status
    pub fn get_migration_status(&self) -> Result<MigrationStatus, SchemaError> {
        let mut encrypted_count = 0u64;
        let mut unencrypted_count = 0u64;

        // Count encrypted vs unencrypted items in main database
        for result in self.db_ops.db().iter() {
            let (_, value) = result.map_err(|e| {
                SchemaError::InvalidData(format!("Failed to iterate database: {}", e))
            })?;

            if EncryptedDataFormat::is_encrypted_data(&value) {
                encrypted_count += 1;
            } else {
                unencrypted_count += 1;
            }
        }

        Ok(MigrationStatus {
            total_items: encrypted_count + unencrypted_count,
            encrypted_items: encrypted_count,
            unencrypted_items: unencrypted_count,
            migration_mode: self.migration_mode,
            encryption_enabled: self.encryption_enabled,
            last_migration_at: None, // TODO: Track this in metadata
        })
    }

    /// Detect unencrypted data automatically
    pub fn detect_unencrypted_data(&self) -> Result<Vec<String>, SchemaError> {
        let mut unencrypted_keys = Vec::new();

        for result in self.db_ops.db().iter() {
            let (key, value) = result.map_err(|e| {
                SchemaError::InvalidData(format!("Failed to iterate database: {}", e))
            })?;

            if !EncryptedDataFormat::is_encrypted_data(&value) {
                let key_str = String::from_utf8_lossy(&key).to_string();
                unencrypted_keys.push(key_str);
            }
        }

        Ok(unencrypted_keys)
    }

    /// Perform batch migration with comprehensive validation
    pub fn perform_batch_migration(&self, config: &MigrationConfig) -> Result<u64, SchemaError> {
        // Validate configuration first
        MigrationUtils::validate_config(config)?;

        // Check if encryption is disabled (read-only compatibility mode)
        if !self.encryption_enabled {
            return Err(SchemaError::InvalidData(
                "Cannot perform migration in read-only compatibility mode".to_string(),
            ));
        }

        match config.mode {
            MigrationMode::ReadOnlyCompatibility => Err(SchemaError::InvalidData(
                "Cannot perform migration in read-only compatibility mode".to_string(),
            )),
            MigrationMode::Gradual => {
                log::info!("Performing gradual migration - new data will be encrypted");
                Ok(0) // No migration needed, just enable encryption for new data
            }
            MigrationMode::Full => {
                log::info!("Performing full migration - converting all data to encrypted format");
                self.migrate_to_encrypted_with_validation(&config.target_context, config)
            }
        }
    }

    /// Enhanced migration with validation and error handling
    fn migrate_to_encrypted_with_validation(
        &self,
        context: &str,
        config: &MigrationConfig,
    ) -> Result<u64, SchemaError> {
        if !self.encryption_enabled {
            return Err(SchemaError::InvalidData(
                "Cannot migrate to encrypted format when encryption is disabled".to_string(),
            ));
        }

        log::info!(
            "Starting migration to encrypted format with context: {}",
            context
        );

        let mut migrated_count = 0u64;
        let mut items_to_migrate = Vec::new();
        let mut batch_count = 0;

        // Collect items that need migration
        for result in self.db_ops.db().iter() {
            let (key, value) = result.map_err(|e| {
                SchemaError::InvalidData(format!("Failed to iterate database: {}", e))
            })?;

            // Skip already encrypted data
            if !EncryptedDataFormat::is_encrypted_data(&value) {
                let key_str = String::from_utf8_lossy(&key).to_string();
                items_to_migrate.push((key_str, value.to_vec()));

                // Process in batches to avoid memory issues
                if items_to_migrate.len() >= config.batch_size {
                    batch_count += 1;
                    log::debug!(
                        "Processing migration batch {} with {} items",
                        batch_count,
                        items_to_migrate.len()
                    );
                    migrated_count += self.migrate_batch(&items_to_migrate, context, config)?;
                    items_to_migrate.clear();
                }
            }
        }

        // Process remaining items
        if !items_to_migrate.is_empty() {
            batch_count += 1;
            log::debug!(
                "Processing final migration batch {} with {} items",
                batch_count,
                items_to_migrate.len()
            );
            migrated_count += self.migrate_batch(&items_to_migrate, context, config)?;
        }

        log::info!(
            "Migration completed successfully. Migrated {} items in {} batches",
            migrated_count,
            batch_count
        );
        Ok(migrated_count)
    }

    /// Migrate a batch of items with validation
    fn migrate_batch(
        &self,
        items: &[(String, Vec<u8>)],
        context: &str,
        config: &MigrationConfig,
    ) -> Result<u64, SchemaError> {
        let mut migrated_count = 0u64;

        // Get the encryptor for this context
        let encryptor = self.encryptors.get(context).ok_or_else(|| {
            SchemaError::InvalidData(format!("Unknown encryption context: {}", context))
        })?;

        for (key, unencrypted_bytes) in items {
            // Validate data integrity if requested
            if config.verify_integrity && MigrationUtils::validate_json_integrity(unencrypted_bytes).is_err() {
                log::warn!(
                    "Skipping migration of potentially corrupted item '{}'",
                    key
                );
                continue;
            }

            // Encrypt the data
            let encrypted_data = encryptor.encrypt(unencrypted_bytes).map_err(|e| {
                SchemaError::InvalidData(format!(
                    "Migration encryption failed for key '{}': {}",
                    key, e
                ))
            })?;

            // Create encrypted data format
            let encrypted_format = EncryptedDataFormat::new(context.to_string(), encrypted_data)
                .map_err(|e| {
                    SchemaError::InvalidData(format!(
                        "Failed to create encrypted format for key '{}': {}",
                        key, e
                    ))
                })?;

            // Serialize to bytes
            let encrypted_bytes = encrypted_format.to_bytes().map_err(|e| {
                SchemaError::InvalidData(format!(
                    "Failed to serialize encrypted data for key '{}': {}",
                    key, e
                ))
            })?;

            // Replace the unencrypted data
            self.db_ops
                .db()
                .insert(key.as_bytes(), encrypted_bytes)
                .map_err(|e| {
                    SchemaError::InvalidData(format!(
                        "Failed to replace data for key '{}': {}",
                        key, e
                    ))
                })?;

            migrated_count += 1;
        }

        // Flush changes after each batch
        self.db_ops.db().flush().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to flush after batch migration: {}", e))
        })?;

        Ok(migrated_count)
    }

    /// Validate data format consistency across the database
    pub fn validate_data_format_consistency(&self) -> Result<HashMap<String, u64>, SchemaError> {
        let mut validation_stats = HashMap::new();
        let mut encrypted_count = 0u64;
        let mut unencrypted_count = 0u64;
        let mut invalid_count = 0u64;
        let mut context_counts: HashMap<String, u64> = HashMap::new();

        for result in self.db_ops.db().iter() {
            let (key, value) = result.map_err(|e| {
                SchemaError::InvalidData(format!("Failed to iterate database: {}", e))
            })?;

            let key_str = String::from_utf8_lossy(&key).to_string();

            if EncryptedDataFormat::is_encrypted_data(&value) {
                // Try to parse encrypted data format
                match EncryptedDataFormat::from_bytes(&value) {
                    Ok(encrypted_format) => {
                        encrypted_count += 1;
                        *context_counts
                            .entry(encrypted_format.context().to_string())
                            .or_insert(0) += 1;
                    }
                    Err(e) => {
                        log::warn!("Invalid encrypted data format for key '{}': {}", key_str, e);
                        invalid_count += 1;
                    }
                }
            } else {
                // Check if unencrypted data is valid JSON
                match MigrationUtils::validate_json_integrity(&value) {
                    Ok(_) => unencrypted_count += 1,
                    Err(e) => {
                        log::warn!(
                            "Invalid JSON format for unencrypted key '{}': {}",
                            key_str, e
                        );
                        invalid_count += 1;
                    }
                }
            }
        }

        validation_stats.insert("encrypted_valid".to_string(), encrypted_count);
        validation_stats.insert("unencrypted_valid".to_string(), unencrypted_count);
        validation_stats.insert("invalid_format".to_string(), invalid_count);
        validation_stats.insert("total_contexts".to_string(), context_counts.len() as u64);

        // Add per-context counts
        for (context, count) in context_counts {
            validation_stats.insert(format!("context_{}", context), count);
        }

        Ok(validation_stats)
    }

    /// Migrate unencrypted data to encrypted format (original method)
    pub fn migrate_to_encrypted(&self, context: &str) -> Result<u64, SchemaError> {
        if !self.encryption_enabled {
            return Err(SchemaError::InvalidData(
                "Cannot migrate to encrypted format when encryption is disabled".to_string(),
            ));
        }

        let mut migrated_count = 0u64;
        let mut items_to_migrate = Vec::new();

        // Collect items that need migration
        for result in self.db_ops.db().iter() {
            let (key, value) = result.map_err(|e| {
                SchemaError::InvalidData(format!("Failed to iterate database: {}", e))
            })?;

            // Skip already encrypted data
            if !EncryptedDataFormat::is_encrypted_data(&value) {
                let key_str = String::from_utf8_lossy(&key).to_string();
                items_to_migrate.push((key_str, value.to_vec()));
            }
        }

        // Migrate each item
        for (key, unencrypted_bytes) in items_to_migrate {
            // Get the encryptor for this context
            let encryptor = self.encryptors.get(context).ok_or_else(|| {
                SchemaError::InvalidData(format!("Unknown encryption context: {}", context))
            })?;

            // Encrypt the data
            let encrypted_data = encryptor.encrypt(&unencrypted_bytes).map_err(|e| {
                SchemaError::InvalidData(format!("Migration encryption failed: {}", e))
            })?;

            // Create encrypted data format
            let encrypted_format = EncryptedDataFormat::new(context.to_string(), encrypted_data)
                .map_err(|e| {
                    SchemaError::InvalidData(format!(
                        "Failed to create encrypted format during migration: {}",
                        e
                    ))
                })?;

            // Serialize to bytes
            let encrypted_bytes = encrypted_format.to_bytes().map_err(|e| {
                SchemaError::InvalidData(format!(
                    "Failed to serialize encrypted data during migration: {}",
                    e
                ))
            })?;

            // Replace the unencrypted data
            self.db_ops
                .db()
                .insert(key.as_bytes(), encrypted_bytes)
                .map_err(|e| {
                    SchemaError::InvalidData(format!(
                        "Failed to replace data during migration: {}",
                        e
                    ))
                })?;

            migrated_count += 1;
        }

        // Flush changes
        self.db_ops.db().flush().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to flush after migration: {}", e))
        })?;

        Ok(migrated_count)
    }

    /// Test encryption functionality
    pub fn self_test(&self) -> Result<(), SchemaError> {
        if !self.encryption_enabled {
            return Ok(()); // Skip test if encryption disabled
        }

        // Test each encryption context
        for context in contexts::all_contexts() {
            let test_data = format!("Test data for context: {}", context);
            let test_key = format!("test_key_{}", context);

            // Store encrypted
            self.store_encrypted_item(&test_key, &test_data, context)?;

            // Retrieve and verify
            let retrieved: String = self
                .get_encrypted_item(&test_key, context)?
                .ok_or_else(|| {
                    SchemaError::InvalidData(format!(
                        "Failed to retrieve test data for context: {}",
                        context
                    ))
                })?;

            if retrieved != test_data {
                return Err(SchemaError::InvalidData(format!(
                    "Self-test failed for context '{}': data mismatch",
                    context
                )));
            }

            // Clean up
            self.db_ops.db().remove(test_key.as_bytes()).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to clean up test data: {}", e))
            })?;
        }

        Ok(())
    }
}
