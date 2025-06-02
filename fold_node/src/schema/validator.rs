use super::{core::SchemaCore, types::{Field, Schema, SchemaError, JsonSchemaDefinition}};
use crate::transform::TransformExecutor;

/// Validates a [`Schema`] before it is loaded into the database.
///
/// The validator checks general schema formatting rules and verifies
/// that any transforms reference valid fields in other schemas.
pub struct SchemaValidator<'a> {
    core: &'a SchemaCore,
}

impl<'a> SchemaValidator<'a> {
    /// Create a new validator operating on the provided [`SchemaCore`].
    pub fn new(core: &'a SchemaCore) -> Self {
        Self { core }
    }

    /// Validate the given [`Schema`].
    pub fn validate(&self, schema: &Schema) -> Result<(), SchemaError> {
        if schema.name.is_empty() {
            return Err(SchemaError::InvalidField(
                "Schema name cannot be empty".to_string(),
            ));
        }

        // For RangeSchema, ensure the range_key is a field in the schema
        if let Some(range_key) = schema.range_key() {
            if !schema.fields.contains_key(range_key) {
                return Err(SchemaError::InvalidField(format!(
                    "RangeSchema range_key '{}' must be one of the schema's fields.",
                    range_key
                )));
            }
        }

        if schema.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(
                "Schema base_multiplier must be positive".to_string(),
            ));
        }

        for (field_name, field) in &schema.fields {
            if field.payment_config().base_multiplier <= 0.0 {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} base_multiplier must be positive",
                )));
            }

            if let Some(min) = field.payment_config().min_payment {
                if min == 0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_payment cannot be zero",
                    )));
                }
            }

            if let Some(transform) = field.transform() {
                // Basic syntax validation
                TransformExecutor::validate_transform(transform)?;

                // Validate inputs
                for input in &transform.inputs {
                    let (sname, fname) = input.split_once('.')
                        .ok_or_else(|| SchemaError::InvalidTransform(format!(
                            "Invalid input format {input} for field {field_name}",
                        )))?;

                    if sname == schema.name {
                        if fname == field_name {
                            return Err(SchemaError::InvalidTransform(format!(
                                "Transform input {input} cannot reference its own field",
                            )));
                        }
                        if !schema.fields.contains_key(fname) {
                            return Err(SchemaError::InvalidTransform(format!(
                                "Input {input} references unknown field",
                            )));
                        }
                    } else {
                        let src_schema = self
                            .core
                            .get_schema(sname)?
                            .ok_or_else(|| {
                                SchemaError::InvalidTransform(format!(
                                    "Schema {sname} not found for input {input}",
                                ))
                            })?;

                        if !src_schema.fields.contains_key(fname) {
                            return Err(SchemaError::InvalidTransform(format!(
                                "Input {input} references unknown field",
                            )));
                        }
                    }
                }

                // Validate output
                let (out_schema, out_field) = transform.output.split_once('.')
                    .ok_or_else(|| SchemaError::InvalidTransform(format!(
                        "Invalid output format {} for field {field_name}",
                        transform.output
                    )))?;

                if out_schema == schema.name {
                    if out_field != field_name {
                        return Err(SchemaError::InvalidTransform(format!(
                            "Transform output {} does not match field name {}",
                            transform.output, field_name
                        )));
                    }
                } else {
                    let target = self.core.get_schema(out_schema)?.ok_or_else(|| {
                        SchemaError::InvalidTransform(format!(
                            "Schema {out_schema} not found for output {out_schema}.{out_field}",
                        ))
                    })?;

                    if !target.fields.contains_key(out_field) {
                        return Err(SchemaError::InvalidTransform(format!(
                            "Output field {} not found in schema {}",
                            out_field, out_schema
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate a [`JsonSchemaDefinition`] before interpretation
    pub fn validate_json_schema(&self, schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        if schema.name.is_empty() {
            return Err(SchemaError::InvalidField(
                "Schema name cannot be empty".to_string(),
            ));
        }

        for (field_name, field) in &schema.fields {
            if field_name.is_empty() {
                return Err(SchemaError::InvalidField(
                    "Field name cannot be empty".to_string(),
                ));
            }

            if field.payment_config.base_multiplier <= 0.0 {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} base_multiplier must be positive",
                )));
            }

            for (mapper_key, mapper_value) in &field.field_mappers {
                if mapper_key.is_empty() || mapper_value.is_empty() {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} has invalid field mapper: empty key or value",
                    )));
                }
            }

            if let Some(min_payment) = field.payment_config.min_payment {
                if min_payment == 0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_payment cannot be zero",
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate a mutation for a RangeSchema.
    pub fn validate_range_schema_mutation(
        &self,
        schema: &crate::schema::types::Schema,
        mutation: &crate::schema::types::operations::Mutation,
    ) -> Result<(), crate::schema::types::SchemaError> {
        if let Some(range_key) = schema.range_key() {
            // 1. Ensure all fields are rangeFields
            for (field_name, field_def) in &schema.fields {
                if !matches!(field_def, crate::schema::types::field::FieldVariant::Range(_)) {
                    return Err(crate::schema::types::SchemaError::InvalidData(format!(
                        "All fields in a RangeSchema must be rangeFields. Field '{}' is not a rangeField.",
                        field_name
                    )));
                }
            }
            // 2. Ensure all values in fields_and_values contain the same range_key value
            let mut found_range_key_value: Option<&serde_json::Value> = None;
            for (field_name, value) in mutation.fields_and_values.iter() {
                // Value must be an object containing the range_key
                let obj = value.as_object().ok_or_else(|| crate::schema::types::SchemaError::InvalidData(format!(
                    "Value for field '{}' must be an object containing the range_key '{}'.", field_name, range_key
                )))?;
                let key_val = obj.get(range_key).ok_or_else(|| crate::schema::types::SchemaError::InvalidData(format!(
                    "Value for field '{}' must contain the range_key '{}'.", field_name, range_key
                )))?;
                if let Some(existing) = &found_range_key_value {
                    if existing != &key_val {
                        return Err(crate::schema::types::SchemaError::InvalidData(format!(
                            "All range_key values must match for RangeSchema. Field '{}' has a different value.", field_name
                        )));
                    }
                } else {
                    found_range_key_value = Some(key_val);
                }
            }
        }
        Ok(())
    }
}

