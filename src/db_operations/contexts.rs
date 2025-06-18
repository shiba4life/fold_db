//! Encryption contexts for different data types
//!
//! This module defines the various encryption contexts used throughout the system
//! for different types of data. Each context provides isolated encryption keys
//! and ensures proper data separation.

/// Context for atom data encryption
pub const ATOM_DATA: &str = "atom_data";

/// Context for schema definition encryption
pub const SCHEMA_DATA: &str = "schema_data";

/// Context for metadata encryption
pub const METADATA: &str = "metadata";

/// Context for transform data encryption
pub const TRANSFORM_DATA: &str = "transform_data";

/// Context for orchestrator state encryption
pub const ORCHESTRATOR_STATE: &str = "orchestrator_state";

/// Context for permissions data encryption
pub const PERMISSIONS: &str = "permissions";

/// Context for schema state encryption
pub const SCHEMA_STATE: &str = "schema_state";

/// Get all available encryption contexts
pub fn all_contexts() -> &'static [&'static str] {
    &[
        ATOM_DATA,
        SCHEMA_DATA,
        METADATA,
        TRANSFORM_DATA,
        ORCHESTRATOR_STATE,
        PERMISSIONS,
        SCHEMA_STATE,
    ]
}

/// Validate that a context name is recognized
pub fn is_valid_context(context: &str) -> bool {
    all_contexts().contains(&context)
}

/// Get the default context for general-purpose encryption
pub fn default_context() -> &'static str {
    ATOM_DATA
}

/// Context validation utilities
pub struct ContextValidator;

impl ContextValidator {
    /// Validate context name length and format
    pub fn validate_context_name(context: &str) -> Result<(), String> {
        if context.is_empty() {
            return Err("Context name cannot be empty".to_string());
        }

        if context.len() > 64 {
            return Err(format!(
                "Context name too long: {} characters, maximum is 64",
                context.len()
            ));
        }

        // Check for valid characters (alphanumeric and underscore only)
        if !context.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Context name can only contain alphanumeric characters and underscores".to_string());
        }

        Ok(())
    }

    /// Validate that context is in the allowed list
    pub fn validate_known_context(context: &str) -> Result<(), String> {
        if !is_valid_context(context) {
            return Err(format!("Unknown context: '{}'", context));
        }
        Ok(())
    }

    /// Full context validation (name format + known context)
    pub fn validate_context(context: &str) -> Result<(), String> {
        Self::validate_context_name(context)?;
        Self::validate_known_context(context)?;
        Ok(())
    }
}