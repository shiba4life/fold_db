// Core database operations
mod atom_operations;
pub mod core;
pub mod error_utils;
mod metadata_operations;
mod orchestrator_operations;
mod schema_operations;
mod transform_operations;
mod utility_operations;

// Migration modules for transitioning from legacy crypto
pub mod migration;
pub mod contexts;

// Legacy compatibility modules (temporary for compilation)
pub mod encryption_wrapper;
pub use encryption_wrapper::encryption_wrapper_async;

// Tests module
#[cfg(test)]
pub mod tests;

// Re-export the main DbOperations struct and error utilities
pub use core::{DbOperations, KeyAssociation, KeyRotationRecord, RotationStatistics};
pub use error_utils::ErrorUtils;

// Re-export migration modules
pub use migration::{MigrationConfig, MigrationMode, MigrationStatus};

// Re-export RotationStatus from security_types module
pub use crate::security_types::RotationStatus;

// Re-export compatibility items
pub use encryption_wrapper::{EncryptionWrapper, AsyncEncryptionWrapper, AsyncWrapperConfig};

// Legacy compatibility notes:
// - crypto_metadata, encrypted_backup modules have been removed
// - All database encryption functionality is now handled by unified_crypto::database
// - Key rotation is now handled by unified_crypto::keys::KeyRotationManager
