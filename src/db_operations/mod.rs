// Core database operations
mod atom_operations;
pub mod core;
pub mod crypto_metadata;
pub mod encrypted_backup;
pub mod encrypted_backup_async;
pub mod encryption_wrapper;
pub mod encryption_wrapper_async;
pub mod error_utils;
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
    EncryptedBackupManager, BackupMode, BackupOptions, RestoreOptions,
    BackupMetadata, BackupResult, RestoreStats, BackupError
};
pub use encrypted_backup_async::{AsyncEncryptedBackup, AsyncBackupConfig, ProgressInfo};
pub use encryption_wrapper::{EncryptionWrapper, contexts, MigrationMode, MigrationConfig};
pub use encryption_wrapper_async::{AsyncEncryptionWrapper, AsyncWrapperConfig, AsyncWrapperMetrics};
pub use error_utils::ErrorUtils;
