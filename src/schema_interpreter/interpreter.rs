use crate::schema::types::{Schema, SchemaError, SchemaField};
use crate::schema_interpreter::types::{JsonSchemaDefinition, JsonSchemaField};
use crate::schema_interpreter::validator::SchemaValidator;
use std::collections::HashMap;

/// Interprets JSON schema definitions and converts them to `FoldDB` schemas.
#[derive(Default)]
pub struct SchemaInterpreter;

impl SchemaInterpreter {
    /// Creates a new schema interpreter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Interprets a JSON schema definition and converts it to a `FoldDB` schema.
    ///
    /// # Errors
    /// Returns a `SchemaError` if:
    /// - The schema validation fails
    /// - Any required fields are missing
    /// - Field configurations are invalid
    pub fn interpret(
        &self,
        json_schema: JsonSchemaDefinition,
    ) -> crate::schema_interpreter::Result<Schema> {
        // First validate the JSON schema
        SchemaValidator::validate(&json_schema)?;

        // Convert fields
        let mut fields = HashMap::new();
        for (field_name, json_field) in json_schema.fields {
            fields.insert(field_name, Self::convert_field(json_field));
        }

        // Create the schema
        Ok(Schema {
            name: json_schema.name,
            fields,
            payment_config: json_schema.payment_config,
        })
    }

    /// Converts a JSON schema field to a `FoldDB` schema field.
    fn convert_field(json_field: JsonSchemaField) -> SchemaField {
        SchemaField::new(
            json_field.permission_policy.into(),
            json_field.payment_config.into(),
        ).with_ref_atom_uuid(json_field.ref_atom_uuid)
    }

    /// Interprets a JSON schema from a string.
    ///
    /// # Errors
    /// Returns a `SchemaError` if:
    /// - The JSON string is invalid
    /// - The schema validation fails
    /// - Any required fields are missing
    /// - Field configurations are invalid
    pub fn interpret_str(&self, json_str: &str) -> crate::schema_interpreter::Result<Schema> {
        let json_schema: JsonSchemaDefinition = serde_json::from_str(json_str)
            .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {e}")))?;
        self.interpret(json_schema)
    }

    /// Interprets a JSON schema from a file.
    ///
    /// # Errors
    /// Returns a `SchemaError` if:
    /// - The file cannot be read
    /// - The file contains invalid JSON
    /// - The schema validation fails
    /// - Any required fields are missing
    /// - Field configurations are invalid
    pub fn interpret_file(&self, path: &str) -> crate::schema_interpreter::Result<Schema> {
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| SchemaError::InvalidField(format!("Failed to read schema file: {e}")))?;
        self.interpret_str(&json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::config::TrustDistanceScaling;
    use crate::permissions::types::policy::TrustDistance;
    use crate::schema_interpreter::types::{
        JsonFieldPaymentConfig, JsonPermissionPolicy,
    };

    fn create_test_json_schema() -> JsonSchemaDefinition {
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            JsonSchemaField {
                permission_policy: JsonPermissionPolicy {
                    read: TrustDistance::NoRequirement,
                    write: TrustDistance::Distance(1),
                    explicit_read: None,
                    explicit_write: None,
                },
                ref_atom_uuid: "test_uuid".to_string(),
                payment_config: JsonFieldPaymentConfig {
                    base_multiplier: 1.0,
                    trust_distance_scaling: TrustDistanceScaling::None,
                    min_payment: None,
                },
            },
        );

        JsonSchemaDefinition {
            name: "test_schema".to_string(),
            fields,
            payment_config: crate::fees::payment_config::SchemaPaymentConfig {
                base_multiplier: 1.0,
                min_payment_threshold: 0,
            },
        }
    }

    #[test]
    fn test_interpret_valid_schema() {
        let interpreter = SchemaInterpreter::new();
        let json_schema = create_test_json_schema();
        let result = interpreter.interpret(json_schema);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.name, "test_schema");
        assert!(schema.fields.contains_key("test_field"));
    }

    #[test]
    fn test_interpret_invalid_json() {
        let interpreter = SchemaInterpreter::new();
        let invalid_json = "invalid json";
        let result = interpreter.interpret_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_field_conversion() {
        let json_field = JsonSchemaField {
            permission_policy: JsonPermissionPolicy {
                read: TrustDistance::NoRequirement,
                write: TrustDistance::Distance(0),
                explicit_read: None,
                explicit_write: None,
            },
            ref_atom_uuid: "test_uuid".to_string(),
            payment_config: JsonFieldPaymentConfig {
                base_multiplier: 1.0,
                trust_distance_scaling: TrustDistanceScaling::None,
                min_payment: None,
            },
        };

        let field = SchemaInterpreter::convert_field(json_field);
        assert_eq!(field.ref_atom_uuid, Some("test_uuid".to_string()));
    }

}
