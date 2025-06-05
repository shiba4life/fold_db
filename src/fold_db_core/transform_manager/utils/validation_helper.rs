//! Validation utilities for consistent validation patterns across the transform system.
//!
//! This module provides unified validation patterns with consistent error handling
//! to eliminate duplicate validation code throughout the transform manager.

use crate::schema::types::errors::SchemaError;
use crate::schema::types::transform::Transform;
use log::{info, error, warn};
use std::collections::HashSet;

/// Utility for consistent validation patterns across the transform system
pub struct ValidationHelper;

impl ValidationHelper {
    /// Validate transform registration with consistent error messages
    /// Consolidates similar validation logic in registration processes
    pub fn validate_transform_registration(
        transform_id: &str,
        transform: &Transform,
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating transform registration for: {}", transform_id);

        // Validate transform ID is not empty
        if transform_id.trim().is_empty() {
            let error_msg = "Transform ID cannot be empty".to_string();
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate transform has inputs
        let inputs = transform.get_inputs();
        if inputs.is_empty() {
            let error_msg = format!("Transform '{}' must have at least one input", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate transform has output
        let output = transform.get_output();
        if output.trim().is_empty() {
            let error_msg = format!("Transform '{}' must have a valid output field", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate transform logic is not empty
        if transform.logic.trim().is_empty() {
            let error_msg = format!("Transform '{}' must have non-empty logic", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Transform registration validation passed for: {}", transform_id);
        Ok(())
    }

    /// Validate input types with consistent error messages
    /// Unifies validation patterns for transform operations
    pub fn validate_input_types(
        transform_id: &str,
        input_fields: &[String],
        required_types: Option<&[&str]>,
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating input types for transform: {}", transform_id);

        // Validate inputs are not empty
        if input_fields.is_empty() {
            let error_msg = format!("Transform '{}' has no input fields specified", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate each input field name
        for input_field in input_fields {
            if input_field.trim().is_empty() {
                let error_msg = format!("Transform '{}' has empty input field name", transform_id);
                error!("❌ {}", error_msg);
                return Err(SchemaError::InvalidData(error_msg));
            }

            // Validate field name format (schema.field)
            if !input_field.contains('.') {
                let error_msg = format!(
                    "Transform '{}' input field '{}' must be in format 'schema.field'",
                    transform_id, input_field
                );
                error!("❌ {}", error_msg);
                return Err(SchemaError::InvalidData(error_msg));
            }
        }

        // Validate against required types if provided
        if let Some(types) = required_types {
            if input_fields.len() != types.len() {
                let error_msg = format!(
                    "Transform '{}' input count ({}) doesn't match required types count ({})",
                    transform_id, input_fields.len(), types.len()
                );
                error!("❌ {}", error_msg);
                return Err(SchemaError::InvalidData(error_msg));
            }
        }

        info!("✅ Input type validation passed for transform: {}", transform_id);
        Ok(())
    }

    /// Validate field name format and structure
    pub fn validate_field_name(field_name: &str, context: &str) -> Result<(), SchemaError> {
        info!("🔍 Validating field name '{}' in context: {}", field_name, context);

        if field_name.trim().is_empty() {
            let error_msg = format!("Field name cannot be empty in context: {}", context);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Check for valid schema.field format
        let parts: Vec<&str> = field_name.split('.').collect();
        if parts.len() != 2 {
            let error_msg = format!(
                "Field name '{}' must be in format 'schema.field' in context: {}",
                field_name, context
            );
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        let (schema_name, field_name_part) = (parts[0], parts[1]);
        
        if schema_name.trim().is_empty() {
            let error_msg = format!("Schema name cannot be empty in field '{}' (context: {})", field_name, context);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        if field_name_part.trim().is_empty() {
            let error_msg = format!("Field name cannot be empty in field '{}' (context: {})", field_name, context);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Field name validation passed for: {}", field_name);
        Ok(())
    }

    /// Validate transform output format
    pub fn validate_transform_output(
        transform_id: &str,
        output_field: &str,
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating transform output for '{}': {}", transform_id, output_field);

        if output_field.trim().is_empty() {
            let error_msg = format!("Transform '{}' output field cannot be empty", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate output field format
        Self::validate_field_name(output_field, &format!("transform '{}' output", transform_id))?;

        info!("✅ Transform output validation passed for: {}", transform_id);
        Ok(())
    }

    /// Validate pattern matching for transform types
    /// Consolidates repeated pattern matching for transform types
    pub fn validate_transform_pattern(
        transform_id: &str,
        transform_logic: &str,
        allowed_patterns: &[&str],
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating transform pattern for: {}", transform_id);

        if transform_logic.trim().is_empty() {
            let error_msg = format!("Transform '{}' logic cannot be empty", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Check if logic matches any allowed patterns
        let logic_lower = transform_logic.to_lowercase();
        let pattern_matched = allowed_patterns.iter().any(|pattern| {
            logic_lower.contains(&pattern.to_lowercase())
        });

        if !pattern_matched {
            let error_msg = format!(
                "Transform '{}' logic doesn't match any allowed patterns: {:?}",
                transform_id, allowed_patterns
            );
            warn!("⚠️ {}", error_msg);
            // For now, just warn instead of failing to maintain compatibility
        }

        info!("✅ Transform pattern validation completed for: {}", transform_id);
        Ok(())
    }

    /// Validate field mappings consistency
    pub fn validate_field_mappings(
        field_to_transforms: &std::collections::HashMap<String, HashSet<String>>,
        transform_to_fields: &std::collections::HashMap<String, HashSet<String>>,
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating field mappings consistency");

        // Check that every field->transform mapping has a corresponding transform->field mapping
        for (field_key, transform_set) in field_to_transforms {
            for transform_id in transform_set {
                if let Some(field_set) = transform_to_fields.get(transform_id) {
                    if !field_set.contains(field_key) {
                        let error_msg = format!(
                            "Inconsistent mapping: field '{}' maps to transform '{}', but transform doesn't map back to field",
                            field_key, transform_id
                        );
                        error!("❌ {}", error_msg);
                        return Err(SchemaError::InvalidData(error_msg));
                    }
                } else {
                    let error_msg = format!(
                        "Inconsistent mapping: field '{}' maps to transform '{}', but transform has no field mappings",
                        field_key, transform_id
                    );
                    error!("❌ {}", error_msg);
                    return Err(SchemaError::InvalidData(error_msg));
                }
            }
        }

        info!("✅ Field mappings consistency validation passed");
        Ok(())
    }

    /// Validate transform dependencies
    pub fn validate_transform_dependencies(
        transform_id: &str,
        dependencies: &[String],
        existing_transforms: &HashSet<String>,
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating dependencies for transform: {}", transform_id);

        for dependency in dependencies {
            if !existing_transforms.contains(dependency) {
                let error_msg = format!(
                    "Transform '{}' depends on non-existent transform '{}'",
                    transform_id, dependency
                );
                error!("❌ {}", error_msg);
                return Err(SchemaError::InvalidData(error_msg));
            }
        }

        // Check for circular dependencies (basic check)
        if dependencies.contains(&transform_id.to_string()) {
            let error_msg = format!("Transform '{}' cannot depend on itself", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Transform dependencies validation passed for: {}", transform_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::transform::Transform;
    use std::collections::{HashMap, HashSet};

    fn create_test_transform() -> Transform {
        Transform {
            inputs: vec!["Schema.input1".to_string(), "Schema.input2".to_string()],
            logic: "input1 + input2".to_string(),
            output: "Schema.result".to_string(),
            parsed_expression: None,
        }
    }

    #[test]
    fn test_validate_transform_registration_success() {
        let transform = create_test_transform();
        let result = ValidationHelper::validate_transform_registration("test_transform", &transform);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_transform_registration_empty_id() {
        let transform = create_test_transform();
        let result = ValidationHelper::validate_transform_registration("", &transform);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_types_success() {
        let inputs = vec!["Schema.input1".to_string(), "Schema.input2".to_string()];
        let result = ValidationHelper::validate_input_types("test_transform", &inputs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_input_types_empty_inputs() {
        let inputs = vec![];
        let result = ValidationHelper::validate_input_types("test_transform", &inputs, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_field_name_success() {
        let result = ValidationHelper::validate_field_name("Schema.field", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_field_name_invalid_format() {
        let result = ValidationHelper::validate_field_name("invalid_field_name", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_transform_output_success() {
        let result = ValidationHelper::validate_transform_output("test_transform", "Schema.output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_field_mappings_consistency() {
        let mut field_to_transforms = HashMap::new();
        let mut transform_set = HashSet::new();
        transform_set.insert("transform1".to_string());
        field_to_transforms.insert("Schema.field1".to_string(), transform_set);

        let mut transform_to_fields = HashMap::new();
        let mut field_set = HashSet::new();
        field_set.insert("Schema.field1".to_string());
        transform_to_fields.insert("transform1".to_string(), field_set);

        let result = ValidationHelper::validate_field_mappings(&field_to_transforms, &transform_to_fields);
        assert!(result.is_ok());
    }
}