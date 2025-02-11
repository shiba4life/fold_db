use std::collections::HashMap;
use crate::schema::types::{Schema, SchemaError, SchemaField};
use crate::schema_interpreter::types::{JsonSchemaDefinition, JsonSchemaField};
use crate::schema_interpreter::validator::SchemaValidator;
use crate::schema::mapper::{SchemaMapper, MappingRule};

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
    pub fn interpret(&self, json_schema: JsonSchemaDefinition) -> crate::schema_interpreter::Result<Schema> {
        // First validate the JSON schema
        SchemaValidator::validate(&json_schema)?;

        // Convert fields
        let mut fields = HashMap::new();
        for (field_name, json_field) in json_schema.fields {
            fields.insert(field_name, Self::convert_field(json_field));
        }

        // Convert schema mappers
        let schema_mappers = json_schema.schema_mappers.into_iter()
            .flat_map(|mapper| {
                mapper.source_schemas.into_iter().map(move |source| {
                    SchemaMapper::new(
                        source,
                        mapper.target_schema.clone(),
                        mapper.rules.iter().map(|rule| MappingRule::from(rule.clone())).collect(),
                    )
                })
            })
            .collect();

        // Create the schema
        Ok(Schema {
            name: json_schema.name,
            fields,
            schema_mappers,
            payment_config: json_schema.payment_config,
        })
    }

    /// Converts a JSON schema field to a `FoldDB` schema field.
    fn convert_field(json_field: JsonSchemaField) -> SchemaField {
        SchemaField {
            permission_policy: json_field.permission_policy.into(),
            ref_atom_uuid: json_field.ref_atom_uuid,
            payment_config: json_field.payment_config.into(),
        }
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
    use crate::permissions::types::policy::TrustDistance;
    use crate::fees::types::config::TrustDistanceScaling;
    use crate::schema_interpreter::types::{JsonPermissionPolicy, JsonFieldPaymentConfig, JsonMappingRule, JsonSchemaMapper};

    fn create_test_json_schema(with_mapper: bool) -> JsonSchemaDefinition {
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
        
        // Add fields that will be mapped
        fields.insert(
            "old_field".to_string(),
            JsonSchemaField {
                permission_policy: JsonPermissionPolicy {
                    read: TrustDistance::NoRequirement,
                    write: TrustDistance::Distance(1),
                    explicit_read: None,
                    explicit_write: None,
                },
                ref_atom_uuid: "test_uuid_2".to_string(),
                payment_config: JsonFieldPaymentConfig {
                    base_multiplier: 1.0,
                    trust_distance_scaling: TrustDistanceScaling::None,
                    min_payment: None,
                },
            },
        );
        
        fields.insert(
            "new_field".to_string(),
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

        let schema_mappers = if with_mapper {
            vec![JsonSchemaMapper {
                source_schemas: vec!["source_schema".to_string()],
                target_schema: "target_schema".to_string(),
                rules: vec![
                    JsonMappingRule::Map {
                        source_field: "old_field".to_string(),
                        target_field: "new_field".to_string(),
                        function: Some("to_lowercase".to_string()),
                    }
                ],
            }]
        } else {
            vec![]
        };

        JsonSchemaDefinition {
            name: "test_schema".to_string(),
            fields,
            schema_mappers,
            payment_config: crate::fees::payment_config::SchemaPaymentConfig {
                base_multiplier: 1.0,
                min_payment_threshold: 0,
            },
        }
    }

    #[test]
    fn test_interpret_valid_schema() {
        let interpreter = SchemaInterpreter::new();
        let json_schema = create_test_json_schema(false);
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
        let interpreter = SchemaInterpreter::new();
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
        assert_eq!(field.ref_atom_uuid, "test_uuid");
    }

    #[test]
    fn test_interpret_schema_with_mapper() {
        let interpreter = SchemaInterpreter::new();
        let json_schema = create_test_json_schema(true);
        let result = interpreter.interpret(json_schema);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.schema_mappers.len(), 1);
        
        let mapper = &schema.schema_mappers[0];
        assert_eq!(mapper.source_schema_name, "source_schema");
        assert_eq!(mapper.target_schema_name, "target_schema");
        assert_eq!(mapper.rules.len(), 1);

        match &mapper.rules[0] {
            MappingRule::Map { source_field, target_field, function } => {
                assert_eq!(source_field, "old_field");
                assert_eq!(target_field, "new_field");
                assert_eq!(function, &Some("to_lowercase".to_string()));
            },
            _ => panic!("Expected Map rule"),
        }
    }
}
