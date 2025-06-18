//! Standard encryption contexts for different data types

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

/// Context management utilities
pub mod utils {
    use super::*;

    /// Validate that a context string is one of the standard contexts
    pub fn is_standard_context(context: &str) -> bool {
        all_contexts().contains(&context)
    }

    /// Get a human-readable description for a context
    pub fn context_description(context: &str) -> Option<&'static str> {
        match context {
            ATOM_DATA => Some("General database atom encryption"),
            SCHEMA_METADATA => Some("Schema metadata encryption"),
            INDEX_DATA => Some("Index data encryption"),
            BACKUP_DATA => Some("Backup data encryption"),
            TEMP_DATA => Some("Temporary data encryption"),
            TRANSFORM_QUEUE => Some("Transform queue encryption"),
            NETWORK_MESSAGES => Some("Network message encryption"),
            CONFIG_DATA => Some("Configuration data encryption"),
            _ => None,
        }
    }
}