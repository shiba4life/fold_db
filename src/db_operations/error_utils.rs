//! Error handling utilities for database operations
//!
//! This module provides common error handling patterns to reduce code duplication
//! and ensure consistent error messages across the database layer.

use crate::schema::SchemaError;

/// Utility functions for common error handling patterns in database operations
pub struct ErrorUtils;

impl ErrorUtils {
    /// Creates a serialization error with consistent formatting
    pub fn serialization_error(context: &str, error: serde_json::Error) -> SchemaError {
        SchemaError::InvalidData(format!("Serialization failed for {}: {}", context, error))
    }

    /// Creates a deserialization error with consistent formatting
    pub fn deserialization_error(context: &str, error: serde_json::Error) -> SchemaError {
        SchemaError::InvalidData(format!("Deserialization failed for {}: {}", context, error))
    }

    /// Creates a database operation error with consistent formatting
    pub fn database_error(operation: &str, error: sled::Error) -> SchemaError {
        SchemaError::InvalidData(format!("Database {} failed: {}", operation, error))
    }

    /// Creates a tree operation error with consistent formatting
    pub fn tree_error(operation: &str, tree_name: &str, error: sled::Error) -> SchemaError {
        SchemaError::InvalidData(format!(
            "Tree {} operation on '{}' failed: {}",
            operation, tree_name, error
        ))
    }

    /// Creates a lock acquisition error with consistent formatting
    pub fn lock_error(resource: &str) -> SchemaError {
        SchemaError::InvalidData(format!("Failed to acquire lock for {}", resource))
    }

    /// Creates a not found error with consistent formatting
    pub fn not_found_error(resource_type: &str, identifier: &str) -> SchemaError {
        SchemaError::NotFound(format!("{} '{}' not found", resource_type, identifier))
    }

    /// Creates an invalid data error with consistent formatting
    pub fn invalid_data_error(context: &str, details: &str) -> SchemaError {
        SchemaError::InvalidData(format!("Invalid data in {}: {}", context, details))
    }

    /// Helper for converting sled errors in database operations
    pub fn from_sled_error(operation: &str) -> impl Fn(sled::Error) -> SchemaError + '_ {
        move |e| Self::database_error(operation, e)
    }

    /// Helper for converting serialization errors
    pub fn from_serialization_error(
        context: &str,
    ) -> impl Fn(serde_json::Error) -> SchemaError + '_ {
        move |e| Self::serialization_error(context, e)
    }

    /// Helper for converting deserialization errors
    pub fn from_deserialization_error(
        context: &str,
    ) -> impl Fn(serde_json::Error) -> SchemaError + '_ {
        move |e| Self::deserialization_error(context, e)
    }
}

/// Convenience macros for common error patterns
#[macro_export]
macro_rules! sled_error {
    ($operation:expr) => {
        |e| $crate::db_operations::error_utils::ErrorUtils::database_error($operation, e)
    };
}

#[macro_export]
macro_rules! serialize_error {
    ($context:expr) => {
        |e| $crate::db_operations::error_utils::ErrorUtils::serialization_error($context, e)
    };
}

#[macro_export]
macro_rules! deserialize_error {
    ($context:expr) => {
        |e| $crate::db_operations::error_utils::ErrorUtils::deserialization_error($context, e)
    };
}

#[macro_export]
macro_rules! lock_error {
    ($resource:expr) => {
        $crate::db_operations::error_utils::ErrorUtils::lock_error($resource)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let sled_err = sled::Error::Unsupported("test".to_string());
        let db_error = ErrorUtils::database_error("insert", sled_err);
        assert!(matches!(db_error, SchemaError::InvalidData(_)));

        // Test serialization error by creating one from invalid JSON
        let invalid_json = "invalid json";
        if let Err(json_err) = serde_json::from_str::<serde_json::Value>(invalid_json) {
            let ser_error = ErrorUtils::serialization_error("schema", json_err);
            assert!(matches!(ser_error, SchemaError::InvalidData(_)));
        }
    }

    #[test]
    fn test_not_found_error() {
        let error = ErrorUtils::not_found_error("Schema", "TestSchema");
        assert!(matches!(error, SchemaError::NotFound(_)));
    }

    #[test]
    fn test_lock_error() {
        let error = ErrorUtils::lock_error("schema_mutex");
        assert!(matches!(error, SchemaError::InvalidData(_)));
    }
}
