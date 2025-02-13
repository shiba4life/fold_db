use crate::permissions::types::policy::TrustDistance;
use crate::schema::types::SchemaError;
use crate::schema_interpreter::types::{JsonPermissionPolicy, JsonSchemaDefinition};
use std::collections::HashSet;

pub struct SchemaValidator;

impl SchemaValidator {
    /// Validates a JSON schema definition.
    ///
    /// # Errors
    /// Returns a `SchemaError` if:
    /// - The schema name is empty
    /// - Any field has invalid permissions
    /// - Any field has invalid payment configuration
    /// - Any schema mapper has invalid rules
    pub fn validate(schema: &JsonSchemaDefinition) -> crate::schema_interpreter::Result<()> {
        // Validate schema name
        if schema.name.is_empty() {
            return Err(SchemaError::InvalidField(
                "Schema name cannot be empty".to_string(),
            ));
        }

        // Validate fields
        for (field_name, field) in &schema.fields {
            if field_name.is_empty() {
                return Err(SchemaError::InvalidField(
                    "Field name cannot be empty".to_string(),
                ));
            }

            // Validate permissions
            Self::validate_permissions(&field.permission_policy)?;

            // Validate payment config
            if field.payment_config.base_multiplier <= 0.0 {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} base_multiplier must be positive"
                )));
            }

            if let Some(min_payment) = field.payment_config.min_payment {
                if min_payment == 0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_payment cannot be zero"
                    )));
                }
            }
        }

        // Validate schema mappers
        for mapper in &schema.schema_mappers {
            // Must have at least one source schema
            if mapper.source_schemas.is_empty() {
                return Err(SchemaError::InvalidField(
                    "Schema mapper must have at least one source schema".to_string(),
                ));
            }

            // Target schema must be specified
            if mapper.target_schema.is_empty() {
                return Err(SchemaError::InvalidField(
                    "Schema mapper must specify a target schema".to_string(),
                ));
            }

            // Check for duplicate source-target pairs
            let mut seen_pairs = HashSet::new();
            for source in &mapper.source_schemas {
                let pair = (source.clone(), mapper.target_schema.clone());
                if !seen_pairs.insert(pair.clone()) {
                    return Err(SchemaError::InvalidField(format!(
                        "Duplicate source-target pair: {source} -> {}",
                        mapper.target_schema
                    )));
                }
            }

            // Validate mapping rules
            let mut mapped_fields = HashSet::new();
            for rule in &mapper.rules {
                match rule {
                    crate::schema_interpreter::types::JsonMappingRule::Rename {
                        source_field,
                        target_field,
                    } => {
                        if source_field.is_empty() || target_field.is_empty() {
                            return Err(SchemaError::InvalidField(
                                "Rename rule must specify both source and target fields"
                                    .to_string(),
                            ));
                        }
                        if !mapped_fields.insert(source_field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Field {source_field} is mapped multiple times"
                            )));
                        }
                    }
                    crate::schema_interpreter::types::JsonMappingRule::Drop { field } => {
                        if field.is_empty() {
                            return Err(SchemaError::InvalidField(
                                "Drop rule must specify a field".to_string(),
                            ));
                        }
                        if !mapped_fields.insert(field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Field {field} is mapped multiple times"
                            )));
                        }
                    }
                    crate::schema_interpreter::types::JsonMappingRule::Map {
                        source_field,
                        target_field,
                        ..
                    } => {
                        if source_field.is_empty() || target_field.is_empty() {
                            return Err(SchemaError::InvalidField(
                                "Map rule must specify both source and target fields".to_string(),
                            ));
                        }
                        if !mapped_fields.insert(source_field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Field {source_field} is mapped multiple times"
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_permissions(
        policy: &JsonPermissionPolicy,
    ) -> crate::schema_interpreter::Result<()> {
        match policy.read {
            TrustDistance::Distance(_) => {}
            TrustDistance::NoRequirement => {}
        }

        match policy.write {
            TrustDistance::Distance(_) => {}
            TrustDistance::NoRequirement => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::config::TrustDistanceScaling;
    use crate::schema_interpreter::types::{
        JsonFieldPaymentConfig, JsonMappingRule, JsonSchemaField, JsonSchemaMapper,
    };
    use std::collections::HashMap;

    fn create_valid_schema() -> JsonSchemaDefinition {
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
            schema_mappers: vec![],
            payment_config: crate::fees::payment_config::SchemaPaymentConfig {
                base_multiplier: 1.0,
                min_payment_threshold: 0,
            },
        }
    }

    #[test]
    fn test_validate_valid_schema() {
        let schema = create_valid_schema();
        assert!(SchemaValidator::validate(&schema).is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let mut schema = create_valid_schema();
        schema.name = "".to_string();
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_validate_permissions() {
        let mut schema = create_valid_schema();
        let field = JsonSchemaField {
            permission_policy: JsonPermissionPolicy {
                read: TrustDistance::Distance(0), // Valid
                write: TrustDistance::NoRequirement,
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
        schema.fields.insert("test_field_2".to_string(), field);
        assert!(SchemaValidator::validate(&schema).is_ok());
    }

    #[test]
    fn test_validate_invalid_payment_config() {
        let mut schema = create_valid_schema();
        let field = JsonSchemaField {
            permission_policy: JsonPermissionPolicy {
                read: TrustDistance::NoRequirement,
                write: TrustDistance::Distance(1),
                explicit_read: None,
                explicit_write: None,
            },
            ref_atom_uuid: "test_uuid".to_string(),
            payment_config: JsonFieldPaymentConfig {
                base_multiplier: 0.0, // Invalid
                trust_distance_scaling: TrustDistanceScaling::None,
                min_payment: None,
            },
        };
        schema.fields.insert("invalid_field".to_string(), field);
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_validate_invalid_mapper() {
        let mut schema = create_valid_schema();
        schema.schema_mappers = vec![JsonSchemaMapper {
            source_schemas: vec![], // Invalid - empty source schemas
            target_schema: "target".to_string(),
            rules: vec![],
        }];
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_validate_duplicate_mapping() {
        let mut schema = create_valid_schema();
        schema.schema_mappers = vec![JsonSchemaMapper {
            source_schemas: vec!["source".to_string()],
            target_schema: "target".to_string(),
            rules: vec![
                JsonMappingRule::Map {
                    source_field: "field".to_string(),
                    target_field: "new_field".to_string(),
                },
                JsonMappingRule::Rename {
                    source_field: "field".to_string(), // Duplicate mapping
                    target_field: "other_field".to_string(),
                },
            ],
        }];
        assert!(SchemaValidator::validate(&schema).is_err());
    }
}
