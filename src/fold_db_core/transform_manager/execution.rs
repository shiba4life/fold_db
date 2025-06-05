use super::manager::TransformManager;
use crate::fold_db_core::transform_manager::utils::*;
use crate::transform::executor::TransformExecutor;
use crate::schema::types::{Schema, SchemaError};
use crate::schema::types::field::common::Field;
use log::{info, error, warn};
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

impl TransformManager {

    /// Execute a single transform with input fetching and computation
    pub fn execute_single_transform(
        transform_id: &str,
        transform: &crate::schema::types::Transform,
        db_ops: &Arc<crate::db_operations::DbOperations>,
    ) -> Result<JsonValue, SchemaError> {
        info!("🚀 Executing single transform: {} with inputs: {:?}", transform_id, transform.get_inputs());
        info!("🔧 Transform logic: {}", transform.logic);
        
        // Fetch input values for all transform inputs
        let mut input_values = HashMap::new();
        info!("📥 Loading input values for transform '{}'...", transform_id);
        
        // Get inputs to process - either explicit inputs or analyze dependencies
        let inputs_to_process = if transform.get_inputs().is_empty() {
            info!("📝 No explicit inputs declared, analyzing transform logic for dependencies...");
            let dependencies = transform.analyze_dependencies();
            info!("🔍 Found dependencies: {:?}", dependencies);
            dependencies.into_iter().collect::<Vec<_>>()
        } else {
            transform.get_inputs().to_vec()
        };
        
        for input_field in inputs_to_process {
            info!("📥 Loading input {}...", input_field);
            
            // Parse input field as "Schema.field" or handle simple field names
            if let Some(dot_pos) = input_field.find('.') {
                let input_schema = &input_field[..dot_pos];
                let input_field_name = &input_field[dot_pos + 1..];
                
                match Self::fetch_field_value(db_ops, input_schema, input_field_name) {
                    Ok(value) => {
                        info!("📊 Input value {}: {}", input_field, value);
                        input_values.insert(input_field.clone(), value);
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to fetch input '{}', using default: {}", input_field, e);
                        // Use default value based on the error or field name
                        let default_value = DefaultValueHelper::get_default_value_for_field(input_field_name);
                        info!("📊 Using default value for {}: {}", input_field, default_value);
                        input_values.insert(input_field.clone(), default_value);
                    }
                }
            } else {
                // Handle simple field names without schema prefix
                warn!("⚠️ Simple field name '{}' without schema prefix, using default value", input_field);
                let default_value = DefaultValueHelper::get_default_value_for_field(&input_field);
                info!("📊 Using default value for {}: {}", input_field, default_value);
                input_values.insert(input_field.clone(), default_value);
            }
        }
        
        // Log complete set of inputs before computation
        info!("📊 Complete input set for computation:");
        for (key, value) in &input_values {
            info!("  📋 {}: {}", key, value);
        }
        
        // Execute the transform using TransformExecutor
        info!("🧮 Starting computation with logic: {}", transform.logic);
        let result = TransformExecutor::execute_transform(transform, input_values)?;
        
        info!("✨ Computation result: {}", result);
        info!("🎯 Transform '{}' computation complete: {}", transform_id, result);
        Ok(result)
    }
    
    /// Fetch field value from a specific schema
    fn fetch_field_value(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema_name: &str,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!("📥 Loading input {}.{}...", schema_name, field_name);
        info!("🔍 Fetching field value from database...");
        
        // Load schema
        info!("📋 Loading schema '{}'...", schema_name);
        let schema = db_ops.get_schema(schema_name)?
            .ok_or_else(|| {
                error!("❌ Schema '{}' not found", schema_name);
                SchemaError::InvalidData(format!("Schema '{}' not found", schema_name))
            })?;
        
        info!("✅ Schema '{}' loaded successfully", schema_name);
        
        // Get field value using existing helper
        info!("🔍 Looking up field '{}' in schema...", field_name);
        let result = Self::get_field_value_from_schema(db_ops, &schema, field_name);
        
        match &result {
            Ok(value) => {
                info!("📊 Input value {}.{}: {}", schema_name, field_name, value);
            }
            Err(e) => {
                error!("❌ Failed to load {}.{}: {}", schema_name, field_name, e);
            }
        }
        
        result
    }
    
    
    
    /// Generic result storage for any transform
    pub fn store_transform_result_generic(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        transform: &crate::schema::types::Transform,
        result: &JsonValue,
    ) -> Result<(), SchemaError> {
        info!("💾 Storing generic transform result: {} for output: {}", result, transform.get_output());
        
        // Parse output field as "Schema.field"
        if let Some(dot_pos) = transform.get_output().find('.') {
            let schema_name = &transform.get_output()[..dot_pos];
            let field_name = &transform.get_output()[dot_pos + 1..];
            
            info!("💾 Storing result {} to {}.{}", result, schema_name, field_name);
            
            // Create an atom with the computed result
            info!("🔧 Creating atom for result storage...");
            let atom = db_ops.create_atom(
                schema_name,
                "transform_system".to_string(), // System-generated result
                None, // No previous version
                result.clone(),
                None, // Active status
            )?;
            
            info!("✅ Created atom {} with result: {} for {}.{}", atom.uuid(), result, schema_name, field_name);
            info!("💾 Storing result {} to {}.{}", result, schema_name, field_name);
            
            // Update the field's ref_atom_uuid to point to this atom
            info!("🔗 Updating field reference to point to new atom...");
            Self::update_field_reference(db_ops, schema_name, field_name, atom.uuid())?;
            
            info!("✅ Result stored successfully with atom_id: {}", atom.uuid());
            Ok(())
        } else {
            Err(SchemaError::InvalidField(format!("Invalid output field format '{}', expected 'Schema.field'", transform.get_output())))
        }
    }
    
    /// Update a field's ref_atom_uuid to point to a new atom and create proper linking
    fn update_field_reference(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema_name: &str,
        field_name: &str,
        atom_uuid: &str,
    ) -> Result<(), SchemaError> {
        info!("🔗 Updating field reference: {}.{} -> atom {}", schema_name, field_name, atom_uuid);
        
        // 1. Load the schema
        let mut schema = db_ops.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::InvalidData(format!("Schema '{}' not found", schema_name)))?;
        
        // 2. Get the field
        let field = schema.fields.get_mut(field_name)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field '{}' not found in schema '{}'", field_name, schema_name)))?;
        
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
        let atom_ref = crate::atom::AtomRef::new(atom_uuid.to_string(), "transform_system".to_string());
        db_ops.store_item(&format!("ref:{}", ref_uuid), &atom_ref)?;
        
        info!("✅ Created/updated AtomRef {} -> atom {}", ref_uuid, atom_uuid);
        LoggingHelper::log_atom_ref_operation(&ref_uuid, atom_uuid, "creation");
        
        // Debug: Verify the atom was stored correctly
        match db_ops.get_item::<crate::atom::Atom>(&format!("atom:{}", atom_uuid)) {
            Ok(Some(stored_atom)) => {
                let content_str = stored_atom.content().to_string();
                LoggingHelper::log_verification_result("atom", atom_uuid, Some(&content_str));
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
                LoggingHelper::log_verification_result("AtomRef", &ref_uuid, Some(&format!("points to atom: {}", target_atom_uuid)));
                LoggingHelper::log_atom_ref_operation(&ref_uuid, target_atom_uuid, "verification");
                
                // Verify this is NOT pointing to itself (the bug we just fixed)
                if ref_uuid == *target_atom_uuid {
                    error!("❌ CRITICAL BUG: AtomRef {} is pointing to itself instead of data atom!", ref_uuid);
                } else {
                    info!("✅ VERIFIED: AtomRef {} correctly points to different atom {}", ref_uuid, target_atom_uuid);
                }
            }
            Ok(None) => {
                error!("❌ DEBUG: AtomRef {} was not found after storage!", ref_uuid);
            }
            Err(e) => {
                error!("❌ DEBUG: Error retrieving AtomRef {}: {}", ref_uuid, e);
            }
        }
        
        // 5. Save the updated schema
        db_ops.store_schema(schema_name, &schema)?;
        
        info!("✅ Updated schema '{}' with field '{}' pointing to atom {}", schema_name, field_name, atom_uuid);
        
        Ok(())
    }

    /// Get field value from a schema using database operations (consolidated implementation)
    fn get_field_value_from_schema(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema: &Schema,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        // Use the unified FieldValueResolver instead of duplicate implementation
        FieldValueResolver::resolve_field_value(db_ops, schema, field_name)
    }
}