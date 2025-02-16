use crate::schema::types::{Schema, SchemaError, SchemaField};
use crate::schema_interpreter::types::{JsonSchemaDefinition, JsonSchemaField};
use crate::schema_interpreter::validator::SchemaValidator;
use std::collections::HashMap;

/// Interprets and converts JSON schema definitions into FoldDB schemas.
/// 
/// The SchemaInterpreter is responsible for:
/// - Validating JSON schema definitions
/// - Converting JSON fields to FoldDB schema fields
/// - Handling schema loading from various sources
/// - Ensuring schema integrity and completeness
/// 
/// It provides a bridge between human-readable JSON schema definitions
/// and the internal schema representation used by FoldDB, ensuring that:
/// - All required fields are present
/// - Field configurations are valid
/// - Permissions are properly defined
/// - Payment requirements are correctly specified
#[derive(Default)]
pub struct SchemaInterpreter;

impl SchemaInterpreter {
    /// Creates a new schema interpreter.
    /// 
    /// The interpreter is stateless, so this method simply returns
    /// a new instance ready for schema interpretation.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Interprets a JSON schema definition and converts it to a FoldDB schema.
    /// 
    /// This method:
    /// 1. Validates the JSON schema structure
    /// 2. Converts each field definition
    /// 3. Assembles the complete schema
    /// 
    /// The conversion process ensures that all required components are present
    /// and properly configured, including:
    /// - Field definitions and types
    /// - Permission policies
    /// - Payment configurations
    /// - Field mappings
    /// 
    /// # Arguments
    /// 
    /// * `json_schema` - The JSON schema definition to interpret
    /// 
    /// # Returns
    /// 
    /// A Result containing the converted Schema or an error
    /// 
    /// # Errors
    /// 
    /// Returns a SchemaError if:
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

    /// Converts a JSON schema field definition to a FoldDB schema field.
    /// 
    /// This method handles the conversion of:
    /// - Permission policies
    /// - Payment configurations
    /// - Field references
    /// - Field mappings
    /// 
    /// # Arguments
    /// 
    /// * `json_field` - The JSON field definition to convert
    /// 
    /// # Returns
    /// 
    /// The converted SchemaField
    fn convert_field(json_field: JsonSchemaField) -> SchemaField {
        SchemaField::new(
            json_field.permission_policy.into(),
            json_field.payment_config.into(),
            json_field.field_mappers,
        )
        .with_ref_atom_uuid(json_field.ref_atom_uuid)
    }

    /// Interprets a JSON schema from a string.
    /// 
    /// This method:
    /// 1. Parses the JSON string
    /// 2. Validates the schema structure
    /// 3. Converts it to a FoldDB schema
    /// 
    /// # Arguments
    /// 
    /// * `json_str` - The JSON schema definition as a string
    /// 
    /// # Returns
    /// 
    /// A Result containing the converted Schema or an error
    /// 
    /// # Errors
    /// 
    /// Returns a SchemaError if:
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
    /// This method:
    /// 1. Reads the schema file
    /// 2. Parses the JSON content
    /// 3. Validates and converts the schema
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the JSON schema file
    /// 
    /// # Returns
    /// 
    /// A Result containing the converted Schema or an error
    /// 
    /// # Errors
    /// 
    /// Returns a SchemaError if:
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
                field_mappers: HashMap::new(),
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
            field_mappers: HashMap::new(),
        };

        let field = SchemaInterpreter::convert_field(json_field);
        assert_eq!(field.get_ref_atom_uuid(), Some("test_uuid".to_string()));
    }
}
