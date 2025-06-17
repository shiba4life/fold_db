//! Transform execution module
//!
//! This module handles:
//! - Individual transform execution
//! - Input fetching and computation
//! - Result storage and field updates
//! - Execution error handling

use super::config::*;
use crate::db_operations::DbOperations;
use crate::fold_db_core::transform_manager::utils::TransformUtils;
use crate::schema::types::field::common::Field;
use crate::schema::types::{Schema, SchemaError, Transform};
use crate::transform::executor::TransformExecutor;
use log::{error, info};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Transform execution manager
pub struct TransformExecutionManager {
    db_ops: Arc<DbOperations>,
}

impl TransformExecutionManager {
    /// Create a new execution manager
    pub fn new(db_ops: Arc<DbOperations>) -> Self {
        Self { db_ops }
    }

    /// Execute a transform with the provided inputs
    pub fn execute_transform(
        &self,
        transform_id: &str,
        transform: &Transform,
    ) -> Result<JsonValue, SchemaError> {
        info!("🚀 Executing transform '{}' with execution manager", transform_id);
        Self::execute_single_transform(transform_id, transform, &self.db_ops)
    }

    /// Store transform result using the execution manager
    pub fn store_result(
        &self,
        transform: &Transform,
        result: &JsonValue,
    ) -> Result<(), SchemaError> {
        Self::store_transform_result_generic(&self.db_ops, transform, result)
    }
}

/// Static execution methods (can be called without manager instance)
impl TransformExecutionManager {
    /// Execute a single transform with input fetching and computation
    pub fn execute_single_transform(
        _transform_id: &str,
        transform: &crate::schema::types::Transform,
        db_ops: &Arc<crate::db_operations::DbOperations>,
    ) -> Result<JsonValue, SchemaError> {
        let mut input_values = HashMap::new();
        let inputs_to_process = if transform.get_inputs().is_empty() {
            transform
                .analyze_dependencies()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            transform.get_inputs().to_vec()
        };
        for input_field in inputs_to_process {
            if let Some(dot_pos) = input_field.find('.') {
                let input_schema = &input_field[..dot_pos];
                let input_field_name = &input_field[dot_pos + 1..];
                let value = Self::fetch_field_value(db_ops, input_schema, input_field_name)
                    .unwrap_or_else(|_| {
                        TransformUtils::get_default_value_for_field(input_field_name)
                    });
                input_values.insert(input_field.clone(), value);
            } else {
                input_values.insert(
                    input_field.clone(),
                    TransformUtils::get_default_value_for_field(&input_field),
                );
            }
        }
        TransformExecutor::execute_transform(transform, input_values)
    }

    /// Fetch field value from a specific schema
    fn fetch_field_value(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema_name: &str,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        let schema = db_ops.get_schema(schema_name)?.ok_or_else(|| {
            SchemaError::InvalidData(format!("Schema '{}' not found", schema_name))
        })?;
        Self::get_field_value_from_schema(db_ops, &schema, field_name)
    }

    /// Generic result storage for any transform
    pub fn store_transform_result_generic(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        transform: &crate::schema::types::Transform,
        result: &JsonValue,
    ) -> Result<(), SchemaError> {
        if let Some(dot_pos) = transform.get_output().find('.') {
            let schema_name = &transform.get_output()[..dot_pos];
            let field_name = &transform.get_output()[dot_pos + 1..];
            let atom = db_ops.create_atom(
                schema_name,
                TRANSFORM_SYSTEM_ACTOR.to_string(),
                None,
                result.clone(),
                None,
            )?;
            Self::update_field_reference(db_ops, schema_name, field_name, atom.uuid())
        } else {
            Err(SchemaError::InvalidField(format!(
                "Invalid output field format '{}', expected 'Schema.field'",
                transform.get_output()
            )))
        }
    }

    /// Update a field's ref_atom_uuid to point to a new atom and create proper linking
    fn update_field_reference(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema_name: &str,
        field_name: &str,
        atom_uuid: &str,
    ) -> Result<(), SchemaError> {
        info!(
            "🔗 Updating field reference: {}.{} -> atom {}",
            schema_name, field_name, atom_uuid
        );

        // 1. Load the schema
        let mut schema = db_ops.get_schema(schema_name)?.ok_or_else(|| {
            SchemaError::InvalidData(format!("Schema '{}' not found", schema_name))
        })?;

        // 2. Get the field
        let field = schema.fields.get_mut(field_name).ok_or_else(|| {
            SchemaError::InvalidField(format!(
                "Field '{}' not found in schema '{}'",
                field_name, schema_name
            ))
        })?;

        // 3. Get or create new ref_atom_uuid for the field
        let ref_uuid = match field.ref_atom_uuid() {
            Some(existing_ref) => existing_ref.clone(),
            None => {
                // Create new UUID for the field reference
                let new_ref_uuid = uuid::Uuid::new_v4().to_string();
                field.set_ref_atom_uuid(new_ref_uuid.clone());
                new_ref_uuid
            }
        };

        // 4. Create/update AtomRef to point to the new atom
        let atom_ref =
            crate::atom::AtomRef::new(atom_uuid.to_string(), TRANSFORM_SYSTEM_ACTOR.to_string());
        db_ops.store_item(&format!("ref:{}", ref_uuid), &atom_ref)?;

        info!(
            "✅ Created/updated AtomRef {} -> atom {}",
            ref_uuid, atom_uuid
        );
        crate::fold_db_core::transform_manager::metrics::LoggingHelper::log_atom_ref_operation(&ref_uuid, atom_uuid, "creation");

        // Debug: Verify the atom was stored correctly
        match db_ops.get_item::<crate::atom::Atom>(&format!("atom:{}", atom_uuid)) {
            Ok(Some(stored_atom)) => {
                let content_str = stored_atom.content().to_string();
                crate::fold_db_core::transform_manager::metrics::LoggingHelper::log_verification_result("atom", atom_uuid, Some(&content_str));
            }
            Ok(None) => {
                error!("❌ DEBUG: Atom {} was not found after storage!", atom_uuid);
            }
            Err(e) => {
                error!("❌ DEBUG: Error retrieving atom {}: {}", atom_uuid, e);
            }
        }

        // Debug: Verify the AtomRef was stored correctly
        match db_ops.get_item::<crate::atom::AtomRef>(&format!("ref:{}", ref_uuid)) {
            Ok(Some(stored_ref)) => {
                let target_atom_uuid = stored_ref.get_atom_uuid();
                crate::fold_db_core::transform_manager::metrics::LoggingHelper::log_verification_result(
                    "AtomRef",
                    &ref_uuid,
                    Some(&format!("points to atom: {}", target_atom_uuid)),
                );
                crate::fold_db_core::transform_manager::metrics::LoggingHelper::log_atom_ref_operation(&ref_uuid, target_atom_uuid, "verification");

                // Verify this is NOT pointing to itself (the bug we just fixed)
                if ref_uuid == *target_atom_uuid {
                    error!(
                        "❌ CRITICAL BUG: AtomRef {} is pointing to itself instead of data atom!",
                        ref_uuid
                    );
                } else {
                    info!(
                        "✅ VERIFIED: AtomRef {} correctly points to different atom {}",
                        ref_uuid, target_atom_uuid
                    );
                }
            }
            Ok(None) => {
                error!(
                    "❌ DEBUG: AtomRef {} was not found after storage!",
                    ref_uuid
                );
            }
            Err(e) => {
                error!("❌ DEBUG: Error retrieving AtomRef {}: {}", ref_uuid, e);
            }
        }

        // 5. Save the updated schema
        db_ops.store_schema(schema_name, &schema)?;

        info!(
            "✅ Updated schema '{}' with field '{}' pointing to atom {}",
            schema_name, field_name, atom_uuid
        );

        Ok(())
    }

    /// Get field value from a schema using database operations (consolidated implementation)
    fn get_field_value_from_schema(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema: &Schema,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        // Use the unified field value resolver
        TransformUtils::resolve_field_value(db_ops, schema, field_name, None)
    }
}
