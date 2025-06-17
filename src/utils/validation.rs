//! Validation utilities to consolidate duplicate validation patterns across the codebase
//!
//! This module provides common validation functions that were previously duplicated
//! throughout the codebase as .is_empty() checks and similar patterns.

use crate::schema::types::SchemaError;

/// Validation utilities for common patterns
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validates that a string is not empty, returning a SchemaError if it is
    pub fn require_non_empty_string(value: &str, field_name: &str) -> Result<(), SchemaError> {
        if value.is_empty() {
            return Err(SchemaError::InvalidField(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validates that a collection is not empty, returning a SchemaError if it is
    pub fn require_non_empty_collection<T>(
        collection: &[T],
        field_name: &str,
    ) -> Result<(), SchemaError> {
        if collection.is_empty() {
            return Err(SchemaError::InvalidField(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validates that an option contains a value
    pub fn require_some<'a, T>(
        option: &'a Option<T>,
        field_name: &str,
    ) -> Result<&'a T, SchemaError> {
        option
            .as_ref()
            .ok_or_else(|| SchemaError::InvalidField(format!("{} is required", field_name)))
    }

    /// Validates that a numeric value is positive
    pub fn require_positive(value: f64, field_name: &str) -> Result<(), SchemaError> {
        if value <= 0.0 {
            return Err(SchemaError::InvalidField(format!(
                "{} must be positive",
                field_name
            )));
        }
        Ok(())
    }

    /// Validates API key format (common pattern in ingestion configs)
    pub fn require_valid_api_key(api_key: &str, service_name: &str) -> Result<(), SchemaError> {
        Self::require_non_empty_string(api_key, &format!("{} API key", service_name))?;

        // Basic API key format validation
        if api_key.len() < 10 {
            return Err(SchemaError::InvalidField(format!(
                "{} API key appears to be too short",
                service_name
            )));
        }

        Ok(())
    }

    /// Validates field name format (schema.field)
    pub fn require_valid_field_name(field_name: &str) -> Result<(), SchemaError> {
        Self::require_non_empty_string(field_name, "Field name")?;

        if !field_name.contains('.') {
            return Err(SchemaError::InvalidField(
                "Field name must be in format 'schema.field'".to_string(),
            ));
        }

        let parts: Vec<&str> = field_name.split('.').collect();
        if parts.len() != 2 {
            return Err(SchemaError::InvalidField(
                "Field name must be in format 'schema.field'".to_string(),
            ));
        }

        Self::require_non_empty_string(parts[0], "Schema name")?;
        Self::require_non_empty_string(parts[1], "Field name")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_non_empty_string() {
        // Valid case
        assert!(ValidationUtils::require_non_empty_string("valid", "test").is_ok());

        // Invalid case
        assert!(ValidationUtils::require_non_empty_string("", "test").is_err());
    }

    #[test]
    fn test_require_valid_field_name() {
        // Valid cases
        assert!(ValidationUtils::require_valid_field_name("Schema.field").is_ok());

        // Invalid cases
        assert!(ValidationUtils::require_valid_field_name("").is_err());
        assert!(ValidationUtils::require_valid_field_name("no_dot").is_err());
        assert!(ValidationUtils::require_valid_field_name("too.many.dots").is_err());
        assert!(ValidationUtils::require_valid_field_name(".field").is_err());
        assert!(ValidationUtils::require_valid_field_name("schema.").is_err());
    }

    #[test]
    fn test_require_positive() {
        // Valid case
        assert!(ValidationUtils::require_positive(1.0, "test").is_ok());

        // Invalid cases
        assert!(ValidationUtils::require_positive(0.0, "test").is_err());
        assert!(ValidationUtils::require_positive(-1.0, "test").is_err());
    }
}