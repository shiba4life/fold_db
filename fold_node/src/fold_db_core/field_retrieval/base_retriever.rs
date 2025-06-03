//! Base Field Retriever
//!
//! Provides common functionality shared across all field retrievers to eliminate duplication.

use crate::fold_db_core::atom_manager::AtomManager;
use crate::schema::types::field::FieldVariant;
use crate::schema::types::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

/// Base retriever providing common functionality for all field retrievers
pub struct BaseRetriever<'a> {
    pub atom_manager: &'a AtomManager,
}

impl<'a> BaseRetriever<'a> {
    pub fn new(atom_manager: &'a AtomManager) -> Self {
        Self { atom_manager }
    }

    /// Validates field exists and returns the field definition
    pub fn get_field_def<'b>(
        &self,
        schema: &'b Schema,
        field: &str,
    ) -> Result<&'b FieldVariant, SchemaError> {
        schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))
    }

    /// Validates field is of expected type
    pub fn validate_field_type(
        &self,
        field_def: &FieldVariant,
        expected_type: &str,
        field: &str,
    ) -> Result<(), SchemaError> {
        let matches = matches!(
            (field_def, expected_type),
            (FieldVariant::Single(_), "Single")
                | (FieldVariant::Range(_), "Range")
                | (FieldVariant::Collection(_), "Collection")
        );

        if !matches {
            return Err(SchemaError::InvalidField(format!(
                "Field {} is not a {} field",
                field, expected_type
            )));
        }

        Ok(())
    }

    /// Extracts ref_atom_uuid from field definition, handling empty strings as None
    pub fn get_ref_atom_uuid(
        &self,
        field_def: &FieldVariant,
        schema_name: &str,
        field: &str,
    ) -> Option<String> {
        let ref_atom_uuid = match field_def.ref_atom_uuid() {
            Some(id) if id.is_empty() => None,
            other => other.map(|s| s.to_string()),
        };
        info!(
            "🆔 ref_atom_uuid for {}.{}: {:?}",
            schema_name, field, ref_atom_uuid
        );
        ref_atom_uuid
    }

    /// Gets the default value for a field based on field name
    pub fn default_value_for_field(&self, field: &str) -> Value {
        match field {
            "username" | "email" | "full_name" | "bio" | "location" => {
                Value::String("".to_string())
            }
            "age" => Value::Number(serde_json::Number::from(0)),
            "value1" | "value2" => Value::Number(serde_json::Number::from(0)),
            _ => Value::Null,
        }
    }

    /// Common logging for field retrieval start
    pub fn log_retrieval_start(&self, retriever_type: &str, schema_name: &str, field: &str) {
        info!(
            "🔍 {}::get_value - schema: {}, field: {}",
            retriever_type, schema_name, field
        );
    }

    /// Common logging for missing ref_atom_uuid
    pub fn log_missing_ref_uuid(&self, field_type: &str, schema_name: &str, field: &str) {
        info!(
            "⚠️  No ref_atom_uuid for {} field {}.{}, using default",
            field_type, schema_name, field
        );
    }

    /// Common logging for successful retrieval
    pub fn log_successful_retrieval(&self, schema_name: &str, field: &str, result: &Value) {
        info!(
            "✅ Retrieved {} field content for {}.{}: {:?}",
            self.get_field_type_name(schema_name, field),
            schema_name,
            field,
            result
        );
    }

    /// Helper to get field type name for logging
    fn get_field_type_name(&self, _schema_name: &str, _field: &str) -> &str {
        // This could be enhanced to actually look up the field type if needed
        "field"
    }

    /// Common field value retrieval pattern - handles validation, ref_uuid extraction, and error cases
    /// Specific retrievers provide the data loading and conversion logic
    pub fn retrieve_field_value<F>(
        &self,
        schema: &Schema,
        field: &str,
        field_type: &str,
        load_and_convert_fn: F,
    ) -> Result<Value, SchemaError>
    where
        F: FnOnce(&str) -> Result<Value, SchemaError>,
    {
        let field_def = self.get_field_def(schema, field)?;
        self.validate_field_type(field_def, field_type, field)?;

        let ref_atom_uuid = self.get_ref_atom_uuid(field_def, &schema.name, field);

        if let Some(ref_atom_uuid) = ref_atom_uuid {
            info!(
                "🔗 Fetching {} data for field {}.{} with ref_atom_uuid: {}",
                field_type, schema.name, field, ref_atom_uuid
            );

            match load_and_convert_fn(&ref_atom_uuid) {
                Ok(result) => {
                    self.log_successful_retrieval(&schema.name, field, &result);
                    Ok(result)
                }
                Err(e) => {
                    info!(
                        "❌ Failed to load {} data for {}.{}: {:?}, using default",
                        field_type, schema.name, field, e
                    );
                    Ok(Value::Null)
                }
            }
        } else {
            self.log_missing_ref_uuid(field_type, &schema.name, field);
            Ok(Value::Null)
        }
    }
}
