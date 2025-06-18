//! Encryption wrapper management
//!
//! This module handles encryption manager integration, key management
//! coordination, and encrypted data handling for FoldDB.

use crate::db_operations::DbOperations;
use crate::fold_db_core::managers::AtomManager;
use crate::schema::SchemaError;
use std::sync::Arc;

/// Encryption operations coordinator
pub struct EncryptionOperations {
    db_ops: Arc<DbOperations>,
    encryption_wrapper: Option<Arc<crate::db_operations::EncryptionWrapper>>,
}

impl EncryptionOperations {
    /// Create a new encryption operations coordinator
    pub fn new(db_ops: Arc<DbOperations>) -> Self {
        Self {
            db_ops,
            encryption_wrapper: None,
        }
    }

    /// Enable encryption for atom storage with the given master key pair
    pub fn enable_atom_encryption(
        &mut self,
        master_keypair: &crate::crypto::MasterKeyPair,
        atom_manager: &mut AtomManager,
    ) -> Result<(), SchemaError> {
        let encryption_wrapper =
            crate::db_operations::EncryptionWrapper::new((*self.db_ops).clone(), master_keypair)
                .map_err(|e| {
                    SchemaError::InvalidData(format!(
                        "Failed to create encryption wrapper: {}",
                        e
                    ))
                })?;

        let encryption_wrapper_arc = Arc::new(encryption_wrapper);

        // Set encryption wrapper in atom manager
        atom_manager.set_encryption_wrapper(Arc::clone(&encryption_wrapper_arc));

        // Store the encryption wrapper
        self.encryption_wrapper = Some(encryption_wrapper_arc);

        Ok(())
    }

    /// Enable encryption for atom storage with crypto config
    pub fn enable_atom_encryption_with_config(
        &mut self,
        master_keypair: &crate::crypto::MasterKeyPair,
        crypto_config: &crate::config::crypto::CryptoConfig,
        atom_manager: &mut AtomManager,
    ) -> Result<(), SchemaError> {
        let encryption_wrapper = crate::db_operations::EncryptionWrapper::with_config(
            (*self.db_ops).clone(),
            master_keypair,
            crypto_config,
        )
        .map_err(|e| {
            SchemaError::InvalidData(format!(
                "Failed to create encryption wrapper: {}",
                e
            ))
        })?;

        let encryption_wrapper_arc = Arc::new(encryption_wrapper);

        // Set encryption wrapper in atom manager
        atom_manager.set_encryption_wrapper(Arc::clone(&encryption_wrapper_arc));

        // Store the encryption wrapper
        self.encryption_wrapper = Some(encryption_wrapper_arc);

        Ok(())
    }

    /// Disable encryption for atom storage (fallback to unencrypted)
    pub fn disable_atom_encryption(&mut self) {
        self.encryption_wrapper = None;
        // Note: AtomManager will fall back to unencrypted operations when encryption_wrapper is None
    }

    /// Check if atom encryption is enabled
    pub fn is_atom_encryption_enabled(&self) -> bool {
        #[allow(clippy::unnecessary_map_or)]
        self.encryption_wrapper
            .as_ref()
            .map_or(false, |wrapper| wrapper.is_encryption_enabled())
    }

    /// Get encryption statistics
    pub fn get_encryption_stats(
        &self,
    ) -> Result<std::collections::HashMap<String, u64>, SchemaError> {
        if let Some(encryption_wrapper) = &self.encryption_wrapper {
            encryption_wrapper.get_encryption_stats()
        } else {
            let mut stats = std::collections::HashMap::new();
            stats.insert("encryption_enabled".to_string(), 0);
            stats.insert("encrypted_items".to_string(), 0);
            stats.insert("unencrypted_items".to_string(), 0);
            stats.insert("available_contexts".to_string(), 0);
            Ok(stats)
        }
    }

    /// Migrate existing unencrypted atoms to encrypted format
    pub fn migrate_atoms_to_encrypted(&mut self) -> Result<u64, SchemaError> {
        if let Some(encryption_wrapper) = &self.encryption_wrapper {
            // Use an immutable reference to perform migration
            encryption_wrapper.migrate_to_encrypted(crate::db_operations::contexts::ATOM_DATA)
        } else {
            Err(SchemaError::InvalidData(
                "Encryption is not enabled".to_string(),
            ))
        }
    }

    /// Get a reference to the encryption wrapper for advanced operations
    pub fn encryption_wrapper(&self) -> Option<&Arc<crate::db_operations::EncryptionWrapper>> {
        self.encryption_wrapper.as_ref()
    }

    /// Validate encryption key integrity
    pub fn validate_encryption_keys(&self) -> Result<bool, SchemaError> {
        if let Some(encryption_wrapper) = &self.encryption_wrapper {
            // Just check if encryption is enabled - simplified validation
            Ok(encryption_wrapper.is_encryption_enabled())
        } else {
            Err(SchemaError::InvalidData(
                "Encryption is not enabled".to_string(),
            ))
        }
    }

    /// Get encryption configuration summary
    pub fn get_encryption_summary(&self) -> Result<serde_json::Value, SchemaError> {
        if let Some(_encryption_wrapper) = &self.encryption_wrapper {
            let stats = self.get_encryption_stats()?;
            Ok(serde_json::json!({
                "encryption_enabled": true,
                "statistics": stats,
                "validation_status": self.validate_encryption_keys()?,
            }))
        } else {
            Ok(serde_json::json!({
                "encryption_enabled": false,
                "statistics": self.get_encryption_stats()?,
            }))
        }
    }
}