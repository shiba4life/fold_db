//! Key management command handlers
//! 
//! This module contains organized key management functionality broken down into
//! focused submodules for better maintainability and security.

// Sub-modules
pub mod backup;
pub mod config;
pub mod error;
pub mod extraction;
pub mod generation;
pub mod import_export;
pub mod rotation;
pub mod storage;
pub mod utils;

// Re-export types that are commonly used
pub use error::{KeyError, KeyResult};
pub use config::{KeyManagementConfig, get_default_config_path, initialize_key_management};

// Re-export all public handler functions for CLI command registry
pub use backup::{handle_backup_key, handle_restore_key};
pub use extraction::{handle_extract_public_key, handle_verify_key};
pub use generation::{handle_derive_from_master, handle_derive_key, handle_generate_key};
pub use import_export::{handle_export_key, handle_import_key};
pub use rotation::{handle_list_key_versions, handle_rotate_key};
pub use storage::{handle_delete_key, handle_list_keys, handle_retrieve_key, handle_store_key};

// Re-export utility functions that might be needed externally
pub use utils::{validate_key_id, format_and_output_key_with_index};

// Re-export types used in function signatures
pub use backup::KeyBackupFormat;
pub use generation::EnhancedKdfParams;
pub use import_export::{EnhancedKeyExportFormat, ExportKeyMetadata};
pub use storage::{KeyVersionMetadata, VersionedKeyStorageConfig};