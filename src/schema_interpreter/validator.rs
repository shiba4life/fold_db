use crate::schema::types::SchemaError;
use crate::schema_interpreter::types::{JsonSchemaDefinition, JsonMappingRule};
use crate::fees::types::config::TrustDistanceScaling;
use crate::permissions::types::policy::TrustDistance;
use std::collections::HashSet;

pub struct SchemaValidator;

impl SchemaValidator {
    /// Validates a JSON schema definition
    pub fn validate(schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        Self::validate_schema_name(schema)?;
        Self::validate_fields(schema)?;
        Self::validate_mappers(schema)?;
        Self::validate_payment_config(schema)?;
        Ok(())
    }

    fn validate_schema_name(schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        if schema.name.is_empty() {
            return Err(SchemaError::InvalidField("Schema name cannot be empty".to_string()));
        }
        Ok(())
    }

    fn validate_fields(schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        if schema.fields.is_empty() {
            return Err(SchemaError::InvalidField("Schema must have at least one field".to_string()));
        }

        for (field_name, field) in &schema.fields {
            // Validate field name
            if field_name.is_empty() {
                return Err(SchemaError::InvalidField("Field name cannot be empty".to_string()));
            }

            // Validate ref_atom_uuid
            if field.ref_atom_uuid.is_empty() {
                return Err(SchemaError::InvalidField(
                    format!("Field {} ref_atom_uuid cannot be empty", field_name)
                ));
            }

            // Validate payment config
            Self::validate_field_payment_config(field_name, &field.payment_config)?;

            // Validate trust distances
            Self::validate_trust_distance(&field.permission_policy.read_policy, field_name, "read")?;
            Self::validate_trust_distance(&field.permission_policy.write_policy, field_name, "write")?;
        }

        Ok(())
    }

    fn validate_field_payment_config(
        field_name: &str,
        config: &crate::schema_interpreter::types::JsonFieldPaymentConfig,
    ) -> Result<(), SchemaError> {
        if config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(
                format!("Field {} base_multiplier must be positive", field_name)
            ));
        }

        match &config.trust_distance_scaling {
            TrustDistanceScaling::Linear { slope: _, intercept: _, min_factor } |
            TrustDistanceScaling::Exponential { base: _, scale: _, min_factor } => {
                if *min_factor < 1.0 {
                    return Err(SchemaError::InvalidField(
                        format!("Field {} min_factor must be >= 1.0", field_name)
                    ));
                }
            }
            TrustDistanceScaling::None => {}
        }

        if let Some(min_payment) = config.min_payment {
            if min_payment == 0 {
                return Err(SchemaError::InvalidField(
                    format!("Field {} min_payment must be positive if specified", field_name)
                ));
            }
        }

        Ok(())
    }

    fn validate_trust_distance(
        distance: &TrustDistance,
        field_name: &str,
        policy_type: &str,
    ) -> Result<(), SchemaError> {
        if let TrustDistance::Distance(d) = distance {
            if *d < 0 {
                return Err(SchemaError::InvalidField(
                    format!("Field {} {} distance must be non-negative", field_name, policy_type)
                ));
            }
        }
        Ok(())
    }

    fn validate_mappers(schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        let mut seen_source_target_pairs = HashSet::new();

        for mapper in &schema.schema_mappers {
            // Validate source schemas
            if mapper.source_schemas.is_empty() {
                return Err(SchemaError::InvalidField(
                    "Schema mapper must have at least one source schema".to_string()
                ));
            }

            // Check for duplicate source-target pairs
            for source_schema in &mapper.source_schemas {
                let pair = (source_schema.clone(), mapper.target_schema.clone());
                if !seen_source_target_pairs.insert(pair) {
                    return Err(SchemaError::InvalidField(
                        format!(
                            "Duplicate mapping from source schema {} to target schema {}",
                            source_schema, mapper.target_schema
                        )
                    ));
                }
            }

            // Validate mapping rules
            Self::validate_mapping_rules(schema, mapper)?;
        }

        Ok(())
    }

    fn validate_mapping_rules(
        schema: &JsonSchemaDefinition,
        mapper: &crate::schema_interpreter::types::JsonSchemaMapper,
    ) -> Result<(), SchemaError> {
        let mut mapped_fields = HashSet::new();

        for rule in &mapper.rules {
            match rule {
                JsonMappingRule::Rename { source_field, target_field } => {
                    if mapped_fields.contains(target_field) {
                        return Err(SchemaError::InvalidField(
                            format!("Field {} is mapped multiple times", target_field)
                        ));
                    }
                    mapped_fields.insert(target_field.clone());

                    // Validate target field exists in schema
                    if !schema.fields.contains_key(target_field) {
                        return Err(SchemaError::InvalidField(
                            format!("Target field {} does not exist in schema", target_field)
                        ));
                    }
                }
                JsonMappingRule::Drop { field: _ } => {
                    // No additional validation needed for drop rules
                }
                JsonMappingRule::Map { source_field: _, target_field, function: _ } => {
                    if mapped_fields.contains(target_field) {
                        return Err(SchemaError::InvalidField(
                            format!("Field {} is mapped multiple times", target_field)
                        ));
                    }
                    mapped_fields.insert(target_field.clone());

                    // Validate target field exists in schema
                    if !schema.fields.contains_key(target_field) {
                        return Err(SchemaError::InvalidField(
                            format!("Target field {} does not exist in schema", target_field)
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_payment_config(schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        if schema.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(
                "Schema base_multiplier must be positive".to_string()
            ));
        }

        if schema.payment_config.min_payment_threshold < 0 {
            return Err(SchemaError::InvalidField(
                "Schema min_payment_threshold must be non-negative".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_interpreter::types::{JsonSchemaField, JsonPermissionPolicy, JsonFieldPaymentConfig};
    use std::collections::HashMap;

    fn create_valid_schema() -> JsonSchemaDefinition {
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            JsonSchemaField {
                permission_policy: JsonPermissionPolicy {
                    read_policy: TrustDistance::NoRequirement,
                    write_policy: TrustDistance::Distance(0),
                    explicit_read_policy: None,
                    explicit_write_policy: None,
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
    fn test_valid_schema() {
        let schema = create_valid_schema();
        assert!(SchemaValidator::validate(&schema).is_ok());
    }

    #[test]
    fn test_invalid_schema_name() {
        let mut schema = create_valid_schema();
        schema.name = "".to_string();
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_invalid_field_name() {
        let mut schema = create_valid_schema();
        let field = schema.fields.remove("test_field").unwrap();
        schema.fields.insert("".to_string(), field);
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_invalid_base_multiplier() {
        let mut schema = create_valid_schema();
        schema.payment_config.base_multiplier = 0.0;
        assert!(SchemaValidator::validate(&schema).is_err());
    }

    #[test]
    fn test_invalid_min_factor() {
        let mut schema = create_valid_schema();
        let field = schema.fields.get_mut("test_field").unwrap();
        field.payment_config.trust_distance_scaling = TrustDistanceScaling::Linear {
            slope: 1.0,
            intercept: 1.0,
            min_factor: 0.5,
        };
        assert!(SchemaValidator::validate(&schema).is_err());
    }
}
