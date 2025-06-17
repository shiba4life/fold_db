//! Schema validation logic and validator interface
//!
//! This module contains the validation functionality for schemas including:
//! - Schema validation interface
//! - JSON schema validation
//! - Field validation logic
//! - Schema integrity checks

use crate::schema::core_types::SchemaCore;
use crate::schema::types::{Field, JsonSchemaDefinition, Schema, SchemaError};

/// Schema validator that provides validation services for schemas
pub struct SchemaValidator<'a> {
    schema_core: &'a SchemaCore,
}

impl<'a> SchemaValidator<'a> {
    /// Create a new schema validator
    pub fn new(schema_core: &'a SchemaCore) -> Self {
        Self { schema_core }
    }

    /// Validate a schema definition
    pub fn validate(&self, schema: &Schema) -> Result<(), SchemaError> {
        // Validate schema name
        if schema.name.is_empty() {
            return Err(SchemaError::InvalidField("Schema name cannot be empty".to_string()));
        }

        // Validate fields
        if schema.fields.is_empty() {
            return Err(SchemaError::InvalidField("Schema must have at least one field".to_string()));
        }

        // Validate each field
        for (field_name, field) in &schema.fields {
            if field_name.is_empty() {
                return Err(SchemaError::InvalidField("Field name cannot be empty".to_string()));
            }

            // Additional field validation can be added here
            self.validate_field(field_name, field)?;
        }

        Ok(())
    }

    /// Validate a JSON schema definition
    pub fn validate_json_schema(&self, json_schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        // Validate schema name
        if json_schema.name.is_empty() {
            return Err(SchemaError::InvalidField("Schema name cannot be empty".to_string()));
        }

        // Validate fields
        if json_schema.fields.is_empty() {
            return Err(SchemaError::InvalidField("Schema must have at least one field".to_string()));
        }

        // Validate each field
        for (field_name, field) in &json_schema.fields {
            if field_name.is_empty() {
                return Err(SchemaError::InvalidField("Field name cannot be empty".to_string()));
            }

            // Additional JSON field validation can be added here
            self.validate_json_field(field_name, field)?;
        }

        // Validate schema type if specified
        // (Schema type validation logic can be added here)

        Ok(())
    }

    /// Validate an individual field
    fn validate_field(&self, field_name: &str, _field: &crate::schema::types::FieldVariant) -> Result<(), SchemaError> {
        // Basic field name validation
        if field_name.contains('.') {
            return Err(SchemaError::InvalidField(
                format!("Field name '{}' cannot contain dots", field_name)
            ));
        }

        if field_name.starts_with('_') {
            return Err(SchemaError::InvalidField(
                format!("Field name '{}' cannot start with underscore", field_name)
            ));
        }

        // Additional field validation logic can be added here
        // For example: validate field mappers, transforms, permissions, etc.

        Ok(())
    }

    /// Validate a JSON field definition
    fn validate_json_field(&self, field_name: &str, _json_field: &crate::schema::types::JsonSchemaField) -> Result<(), SchemaError> {
        // Basic field name validation
        if field_name.contains('.') {
            return Err(SchemaError::InvalidField(
                format!("Field name '{}' cannot contain dots", field_name)
            ));
        }

        if field_name.starts_with('_') {
            return Err(SchemaError::InvalidField(
                format!("Field name '{}' cannot start with underscore", field_name)
            ));
        }

        // Additional JSON field validation logic can be added here
        // For example: validate field type, permissions, payment config, etc.

        Ok(())
    }

    /// Validate schema dependencies and relationships
    pub fn validate_schema_dependencies(&self, schema: &Schema) -> Result<(), SchemaError> {
        // Check field mappings reference valid schemas and fields
        for (field_name, field) in &schema.fields {
            for (source_schema_name, source_field_name) in field.field_mappers() {
                // Check if source schema exists
                match self.schema_core.get_schema(source_schema_name) {
                    Ok(Some(source_schema)) => {
                        // Check if source field exists
                        if !source_schema.fields.contains_key(source_field_name) {
                            return Err(SchemaError::InvalidField(format!(
                                "Field '{}' in schema '{}' references non-existent field '{}' in schema '{}'",
                                field_name, schema.name, source_field_name, source_schema_name
                            )));
                        }
                    }
                    Ok(None) => {
                        return Err(SchemaError::InvalidField(format!(
                            "Field '{}' in schema '{}' references non-existent schema '{}'",
                            field_name, schema.name, source_schema_name
                        )));
                    }
                    Err(e) => {
                        return Err(SchemaError::InvalidField(format!(
                            "Error checking schema dependency: {}",
                            e
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate transform definitions in schema
    pub fn validate_transforms(&self, schema: &Schema) -> Result<(), SchemaError> {
        for (field_name, field) in &schema.fields {
            if let Some(transform) = field.transform() {
                // Validate transform inputs reference valid fields
                for input_field in transform.get_inputs() {
                    if !input_field.contains('.') {
                        return Err(SchemaError::InvalidField(format!(
                            "Transform input '{}' in field '{}' must be in format 'schema.field'",
                            input_field, field_name
                        )));
                    }

                    let parts: Vec<&str> = input_field.split('.').collect();
                    if parts.len() != 2 {
                        return Err(SchemaError::InvalidField(format!(
                            "Transform input '{}' in field '{}' must be in format 'schema.field'",
                            input_field, field_name
                        )));
                    }
                }

                // Validate transform output
                let output = transform.get_output();
                if output.is_empty() {
                    return Err(SchemaError::InvalidField(format!(
                        "Transform output for field '{}' cannot be empty",
                        field_name
                    )));
                }

                // Validate transform logic is not empty
                if transform.logic.trim().is_empty() {
                    return Err(SchemaError::InvalidField(format!(
                        "Transform logic for field '{}' cannot be empty",
                        field_name
                    )));
                }
            }
        }

        Ok(())
    }
}

impl SchemaCore {
    /// Get a validator instance for this schema core
    pub fn validator(&self) -> SchemaValidator {
        SchemaValidator::new(self)
    }

    /// Validate a schema using the built-in validator
    pub fn validate_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        let validator = self.validator();
        validator.validate(schema)?;
        validator.validate_transforms(schema)?;
        Ok(())
    }

    /// Validate a JSON schema using the built-in validator
    pub fn validate_json_schema(&self, json_schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        let validator = self.validator();
        validator.validate_json_schema(json_schema)
    }

    /// Validate schema dependencies and relationships
    pub fn validate_schema_dependencies(&self, schema: &Schema) -> Result<(), SchemaError> {
        let validator = self.validator();
        validator.validate_schema_dependencies(schema)
    }
}