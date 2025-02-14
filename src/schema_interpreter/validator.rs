use crate::permissions::types::policy::TrustDistance;
use crate::schema::types::SchemaError;
use crate::schema_interpreter::types::{JsonPermissionPolicy, JsonSchemaDefinition};

pub struct SchemaValidator;

impl SchemaValidator {
    /// Validates a JSON schema definition.
    ///
    /// # Errors
    /// Returns a `SchemaError` if:
    /// - The schema name is empty
    /// - Any field has invalid permissions
    /// - Any field has invalid payment configuration
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

            // Validate field mappers
            for (mapper_key, mapper_value) in &field.field_mappers {
                if mapper_key.is_empty() || mapper_value.is_empty() {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} has invalid field mapper: empty key or value"
                    )));
                }
            }

            if let Some(min_payment) = field.payment_config.min_payment {
                if min_payment == 0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_payment cannot be zero"
                    )));
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
        JsonFieldPaymentConfig, JsonSchemaField,
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
            field_mappers: HashMap::new(),
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
            field_mappers: HashMap::new(),
        };
        schema.fields.insert("invalid_field".to_string(), field);
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_validate_invalid_field_mappers() {
        let mut schema = create_valid_schema();
        let mut field_mappers = HashMap::new();
        field_mappers.insert("".to_string(), "value".to_string()); // Empty key
        
        let field = JsonSchemaField {
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
            field_mappers,
        };
        schema.fields.insert("field_with_invalid_mapper".to_string(), field);
        assert!(SchemaValidator::validate(&schema).is_err());

        // Test empty value
        let mut schema = create_valid_schema();
        let mut field_mappers = HashMap::new();
        field_mappers.insert("key".to_string(), "".to_string()); // Empty value
        
        let field = JsonSchemaField {
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
            field_mappers,
        };
        schema.fields.insert("field_with_invalid_mapper".to_string(), field);
        assert!(SchemaValidator::validate(&schema).is_err());
    }
}
