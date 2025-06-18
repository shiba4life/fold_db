//! Migration logic for database encryption wrapper
//!
//! This module provides migration functionality for transitioning between encrypted
//! and unencrypted data formats, supporting different migration modes and comprehensive
//! validation during the migration process.

use crate::schema::SchemaError;

/// Migration modes for backward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationMode {
    /// Read both encrypted and unencrypted data seamlessly, but don't encrypt new data
    ReadOnlyCompatibility,
    /// Encrypt new data while preserving existing unencrypted data
    Gradual,
    /// Convert all existing data to encrypted format
    Full,
}

impl Default for MigrationMode {
    fn default() -> Self {
        Self::Gradual
    }
}

/// Migration status information
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// Total number of items in the database
    pub total_items: u64,
    /// Number of encrypted items
    pub encrypted_items: u64,
    /// Number of unencrypted items
    pub unencrypted_items: u64,
    /// Current migration mode
    pub migration_mode: MigrationMode,
    /// Whether encryption is enabled for new data
    pub encryption_enabled: bool,
    /// Last migration timestamp (if any)
    pub last_migration_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl MigrationStatus {
    /// Calculate the encryption percentage
    pub fn encryption_percentage(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.encrypted_items as f64 / self.total_items as f64) * 100.0
        }
    }

    /// Check if migration is complete (all data encrypted)
    pub fn is_fully_encrypted(&self) -> bool {
        self.total_items > 0 && self.unencrypted_items == 0
    }

    /// Check if this is a mixed environment
    pub fn is_mixed_environment(&self) -> bool {
        self.encrypted_items > 0 && self.unencrypted_items > 0
    }
}

/// Migration configuration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Migration mode to use
    pub mode: MigrationMode,
    /// Batch size for migration operations
    pub batch_size: usize,
    /// Whether to verify data integrity during migration
    pub verify_integrity: bool,
    /// Whether to backup data before migration
    pub backup_before_migration: bool,
    /// Context to use for migrated data
    pub target_context: String,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            mode: MigrationMode::Gradual,
            batch_size: 100,
            verify_integrity: true,
            backup_before_migration: true,
            target_context: super::contexts::ATOM_DATA.to_string(),
        }
    }
}

/// Migration utilities and helper functions
pub struct MigrationUtils;

impl MigrationUtils {
    /// Validate if data integrity should be checked based on configuration
    pub fn should_verify_integrity(config: &MigrationConfig) -> bool {
        config.verify_integrity
    }

    /// Validate JSON integrity for unencrypted data
    pub fn validate_json_integrity(data: &[u8]) -> Result<(), SchemaError> {
        serde_json::from_slice::<serde_json::Value>(data)
            .map_err(|e| SchemaError::InvalidData(format!("Invalid JSON format: {}", e)))?;
        Ok(())
    }

    /// Check if migration mode allows encryption of new data
    pub fn can_encrypt_new_data(mode: MigrationMode) -> bool {
        matches!(mode, MigrationMode::Gradual | MigrationMode::Full)
    }

    /// Check if migration mode requires all data to be encrypted
    pub fn requires_all_encrypted(mode: MigrationMode) -> bool {
        matches!(mode, MigrationMode::Full)
    }

    /// Validate migration configuration
    pub fn validate_config(config: &MigrationConfig) -> Result<(), SchemaError> {
        if config.batch_size == 0 {
            return Err(SchemaError::InvalidData(
                "Batch size must be greater than 0".to_string(),
            ));
        }

        if config.target_context.is_empty() {
            return Err(SchemaError::InvalidData(
                "Target context cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}