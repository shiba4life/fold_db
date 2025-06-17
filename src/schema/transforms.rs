//! Transform registration, mapping, and field transformation functionality
//!
//! This module contains the logic for:
//! - Transform registration with TransformManager
//! - Field-to-transform mappings
//! - Transform output fixing
//! - Schema transform auto-registration

use crate::schema::core_types::SchemaCore;
use crate::schema::types::{Field, Schema, SchemaError};
use log::info;

/// Ensure any transforms on fields have the correct output schema
pub fn fix_transform_outputs(schema_core: &SchemaCore, schema: &mut Schema) {
    for (field_name, field) in schema.fields.iter_mut() {
        if let Some(transform) = field.transform() {
            let out_schema = transform.get_output();
            if out_schema.starts_with("test.") {
                let mut new_transform = (*transform).clone();
                new_transform.set_output(format!("{}.{}", schema.name, field_name));
                field.set_transform(new_transform);
            }
        }
    }
}

/// Auto-register field transforms with TransformManager during schema loading
pub fn register_schema_transforms(
    schema_core: &SchemaCore,
    schema: &Schema,
) -> Result<(), SchemaError> {
    info!(
        "🔧 DEBUG: Auto-registering transforms for schema: {}",
        schema.name
    );
    info!(
        "🔍 DEBUG: Schema has {} fields to check for transforms",
        schema.fields.len()
    );

    for (field_name, field) in &schema.fields {
        info!(
            "🔍 DEBUG: Checking field '{}.{}' for transforms",
            schema.name, field_name
        );
        if let Some(transform) = field.transform() {
            info!(
                "📋 Found transform on field {}.{}: inputs={:?}, logic={}, output={}",
                schema.name,
                field_name,
                transform.get_inputs(),
                transform.logic,
                transform.get_output()
            );

            let transform_id = format!("{}.{}", schema.name, field_name);

            // Store the transform in the database so it can be loaded by TransformManager
            if let Err(e) = schema_core.db_ops.store_transform(&transform_id, transform) {
                log::error!("Failed to store transform {}: {}", transform_id, e);
                continue;
            }

            info!("✅ Stored transform {} for auto-registration", transform_id);

            // 🛠️ FIX: Create field-to-transform mappings for TransformOrchestrator
            // This is the missing piece - we need to map each input field to this transform
            for input_field in transform.get_inputs() {
                info!(
                    "🔗 Creating field mapping: '{}' → '{}' transform",
                    input_field, transform_id
                );

                // Store field mapping in database for TransformManager to load
                if let Err(e) =
                    store_field_to_transform_mapping(schema_core, input_field, &transform_id)
                {
                    log::error!(
                        "Failed to store field mapping '{}' → '{}': {}",
                        input_field,
                        transform_id,
                        e
                    );
                } else {
                    info!(
                        "✅ Stored field mapping: '{}' → '{}' transform",
                        input_field, transform_id
                    );
                }
            }
        }
    }

    Ok(())
}

/// Store field-to-transform mapping in database for TransformManager to load
pub fn store_field_to_transform_mapping(
    schema_core: &SchemaCore,
    field_key: &str,
    transform_id: &str,
) -> Result<(), SchemaError> {
    // Use the same key format as TransformManager
    const FIELD_TO_TRANSFORMS_KEY: &str = "map_field_to_transforms";

    // Load existing mappings using the correct method
    let mut field_mappings: std::collections::HashMap<
        String,
        std::collections::HashSet<String>,
    > = if let Some(data) = schema_core.db_ops.get_transform_mapping(FIELD_TO_TRANSFORMS_KEY)? {
        serde_json::from_slice(&data).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    // Add this mapping
    field_mappings
        .entry(field_key.to_string())
        .or_default()
        .insert(transform_id.to_string());

    // Store updated mappings using the correct method
    let json = serde_json::to_vec(&field_mappings).map_err(|e| {
        SchemaError::InvalidData(format!("Failed to serialize field mappings: {}", e))
    })?;
    schema_core
        .db_ops
        .store_transform_mapping(FIELD_TO_TRANSFORMS_KEY, &json)?;

    info!(
        "💾 Updated field mappings in database: {} fields mapped",
        field_mappings.len()
    );

    Ok(())
}

impl SchemaCore {
    /// Ensure any transforms on fields have the correct output schema
    pub(crate) fn fix_transform_outputs(&self, schema: &mut Schema) {
        fix_transform_outputs(self, schema)
    }

    /// Auto-register field transforms with TransformManager during schema loading
    pub(crate) fn register_schema_transforms(&self, schema: &Schema) -> Result<(), SchemaError> {
        register_schema_transforms(self, schema)
    }

    /// Store field-to-transform mapping in database for TransformManager to load
    pub(crate) fn store_field_to_transform_mapping(
        &self,
        field_key: &str,
        transform_id: &str,
    ) -> Result<(), SchemaError> {
        store_field_to_transform_mapping(self, field_key, transform_id)
    }
}