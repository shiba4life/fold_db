// Core database operations
mod atom_operations;
pub mod core;
pub mod crypto_metadata;
pub mod encrypted_backup;
pub mod encrypted_backup_async;
pub mod encryption_wrapper;
pub mod encryption_wrapper_async;
pub mod error_utils;
pub mod key_rotation_operations;
mod metadata_operations;
mod orchestrator_operations;
mod schema_operations;
mod transform_operations;
mod utility_operations;

// Tests module

// Re-export the main DbOperations struct and error utilities
pub use core::DbOperations;
pub use crypto_metadata::CryptoMetadata;
pub use encrypted_backup::{
    BackupError, BackupMetadata, BackupMode, BackupOptions, BackupResult, EncryptedBackupManager,
    RestoreOptions, RestoreStats,
};
pub use encrypted_backup_async::{AsyncBackupConfig, AsyncEncryptedBackup, ProgressInfo};
pub use encryption_wrapper::{contexts, EncryptionWrapper, MigrationConfig, MigrationMode};
pub use encryption_wrapper_async::{
    AsyncEncryptionWrapper, AsyncWrapperConfig, AsyncWrapperMetrics,
};
pub use error_utils::ErrorUtils;
pub use key_rotation_operations::{
    KeyAssociation, KeyRotationRecord, KEY_ASSOCIATIONS_TREE, KEY_ROTATION_INDEX_TREE,
    KEY_ROTATION_RECORDS_TREE,
};
// Re-export RotationStatus from security_types module
pub use crate::security_types::RotationStatus;
