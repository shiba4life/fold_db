use super::{core::SchemaCore, types::{Schema, SchemaError}};
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

        if schema.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(
                "Schema base_multiplier must be positive".to_string(),
            ));
        }

        for (field_name, field) in &schema.fields {
            if field.payment_config.base_multiplier <= 0.0 {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} base_multiplier must be positive",
                )));
            }

            if let Some(min) = field.payment_config.min_payment {
                if min == 0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_payment cannot be zero",
                    )));
                }
            }

            if let Some(transform) = &field.transform {
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
}

