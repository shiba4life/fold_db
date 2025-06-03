//! # Field Manager: AtomRef and ref_atom_uuid Management
//!
//! **CRITICAL: Preventing "Ghost ref_atom_uuid" Issues**
//!
//! This module manages field values and their corresponding AtomRefs. The most important
//! principle is the proper management of `ref_atom_uuid` values to prevent "ghost" UUIDs
//! that point to non-existent AtomRefs.
//!
//! ## Range Schema Agnostic Design
//!
//! **IMPORTANT**: The FieldManager is completely agnostic about range schema logic.
//! It only handles field types (Range, Collection, etc.) and knows nothing about
//! range_key semantics or range schema transformations.
//!
//! **Range Schema Processing Flow:**
//! 1. **Schema Operations Layer**: Transforms range schema mutations using
//!    [`to_range_schema_mutation()`](../../schema/types/operations.rs:124) to ensure
//!    all AtomRefRange keys will be range_key VALUES instead of field names
//! 2. **Field Manager Layer**: Processes the pre-transformed data using standard
//!    field type logic - no knowledge of range schemas needed
//! 3. **Result**: Consistent AtomRefRange key structure across all range fields
//!
//! This separation ensures that:
//! - Range schema logic is centralized in the schema operations layer
//! - Field manager remains simple and focused on field type handling
//! - Changes to range schema behavior don't require field manager modifications
//!
//! ## The Problem: Ghost ref_atom_uuid
//!
//! A "ghost ref_atom_uuid" occurs when:
//! 1. A field definition has a ref_atom_uuid value
//! 2. But no corresponding AtomRef exists in the atom manager
//! 3. Queries fail with "AtomRef not found" errors
//! 4. This happens when ref_atom_uuid is set on schema clones that get discarded
//!
//! ## The Solution: Proper ref_atom_uuid Management Pattern
//!
//! **Field Manager Methods (set_field_value, update_field, etc.):**
//! - Create AtomRef in atom manager
//! - Return the UUID to caller
//! - DO NOT set ref_atom_uuid on field definition
//!
//! **Mutation Logic:**
//! - Call field manager method to get UUID
//! - Use schema_manager.update_field_ref_atom_uuid() to set and persist UUID
//! - This is the ONLY place where ref_atom_uuid should be set
//!
//! **Schema Manager:**
//! - Sets ref_atom_uuid on actual schema (not clone)
//! - Immediately persists schema to disk
//! - Ensures ref_atom_uuid is only set when AtomRef exists
//!
//! ## Rules to Prevent Ghost ref_atom_uuid:
//!
//! 1. **NEVER** set ref_atom_uuid directly on field definitions in field manager
//! 2. **ALWAYS** return UUID from field manager methods
//! 3. **ONLY** use schema_manager.update_field_ref_atom_uuid() to set ref_atom_uuid
//! 4. **ENSURE** AtomRef exists before setting ref_atom_uuid
//! 5. **PERSIST** schema immediately after setting ref_atom_uuid

use super::atom_manager::AtomManager;
use super::context::AtomContext;
use super::field_retrieval::FieldRetrievalService;
use super::transform_manager::TransformManager;
use crate::atom::AtomStatus;
use crate::schema::types::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

use super::transform_orchestrator::TransformOrchestrator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct FieldManager {
    pub(super) atom_manager: AtomManager,
    retrieval_service: FieldRetrievalService,
    transform_manager: Arc<RwLock<Option<Arc<TransformManager>>>>,
    orchestrator: Arc<RwLock<Option<Arc<TransformOrchestrator>>>>,
}

impl FieldManager {
    pub fn new(atom_manager: AtomManager) -> Self {
        Self {
            atom_manager,
            retrieval_service: FieldRetrievalService::new(),
            transform_manager: Arc::new(RwLock::new(None)),
            orchestrator: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_transform_manager(&self, manager: Arc<TransformManager>) -> Result<(), SchemaError> {
        let mut guard = self.transform_manager.write().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string())
        })?;
        *guard = Some(manager);
        Ok(())
    }

    pub fn get_transform_manager(&self) -> Result<Option<Arc<TransformManager>>, SchemaError> {
        let guard = self.transform_manager.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string())
        })?;
        Ok(guard.clone())
    }

    pub fn set_orchestrator(
        &self,
        orchestrator: Arc<TransformOrchestrator>,
    ) -> Result<(), SchemaError> {
        let mut guard = self.orchestrator.write().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string())
        })?;
        *guard = Some(orchestrator);
        Ok(())
    }

    pub fn get_orchestrator(&self) -> Result<Option<Arc<TransformOrchestrator>>, SchemaError> {
        let guard = self.orchestrator.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string())
        })?;
        Ok(guard.clone())
    }

    pub fn get_or_create_atom_ref(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: &str,
    ) -> Result<String, SchemaError> {
        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key.to_string(),
            &mut self.atom_manager,
        );
        ctx.get_or_create_atom_ref()
    }

    pub fn get_field_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        info!("🔍 FieldManager::get_field_value - delegating to FieldRetrievalService");
        self.retrieval_service
            .get_field_value(&self.atom_manager, schema, field)
    }

    /// Get field value with optional filtering using the field's native filtering capabilities.
    ///
    /// This method delegates to the FieldRetrievalService which handles field type detection
    /// and applies appropriate filtering logic for each field type.
    pub fn get_field_value_with_filter(
        &self,
        schema: &Schema,
        field: &str,
        filter: &Value,
    ) -> Result<Value, SchemaError> {
        info!("🔄 FieldManager::get_field_value_with_filter - delegating to FieldRetrievalService");
        self.retrieval_service.get_field_value_with_filter(
            &self.atom_manager,
            schema,
            field,
            filter,
        )
    }

    /// Sets a field value and creates the corresponding AtomRef.
    ///
    /// **CRITICAL: ref_atom_uuid Management**
    ///
    /// This method creates an AtomRef in the atom manager and returns the UUID.
    /// It does NOT set the ref_atom_uuid on the field definition - that must be done
    /// by the caller using the schema manager to ensure proper persistence.
    ///
    /// **Why this pattern prevents "ghost ref_atom_uuid" issues:**
    /// - Field definitions start with ref_atom_uuid = None
    /// - AtomRef is created in atom manager with a UUID
    /// - UUID is returned to caller (mutation logic)
    /// - Caller uses schema manager to set and persist ref_atom_uuid
    /// - This ensures ref_atom_uuid is only set when AtomRef actually exists
    ///
    /// **DO NOT** set ref_atom_uuid directly on the schema parameter - it's often
    /// a clone that gets discarded, leading to "ghost" UUIDs that point to nothing.
    ///
    /// Returns: The UUID of the created AtomRef that should be persisted via schema manager
    pub fn set_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<String, SchemaError> {
        info!(
            "🔧 set_field_value START - schema: {}, field: {}, content: {:?}, pub_key: {}",
            schema.name, field, content, source_pub_key
        );

        // Check if field already has a ref_atom_uuid before we start
        let existing_ref_uuid = schema
            .fields
            .get(field)
            .and_then(|f| f.ref_atom_uuid())
            .map(|uuid| uuid.to_string());
        info!(
            "🔍 Existing ref_atom_uuid for {}.{}: {:?}",
            schema.name, field, existing_ref_uuid
        );

        let aref_uuid = {
            let mut ctx = AtomContext::new(
                schema,
                field,
                source_pub_key.clone(),
                &mut self.atom_manager,
            );

            let field_def = ctx.get_field_def()?;

            if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
                return Err(SchemaError::InvalidField(
                    "Collection fields cannot be updated without id".to_string(),
                ));
            }

            // Special handling for Range fields
            if let crate::schema::types::FieldVariant::Range(_range_field) = field_def {
                info!("🎯 Handling Range field for {}.{}", schema.name, field);
                // Range fields now handle individual key-value pairs
                // content should be a JSON object, iterate over its key-value pairs

                // // if the length is greater than 1, create a new atom ref
                // if content.as_object().unwrap().len() > 1 {
                //     return Err(SchemaError::InvalidData(format!(
                //         "Range field {} expects a single key-value pair, got: {:?}", field, content
                //     )));
                // }

                let aref_uuid = ctx.get_or_create_atom_ref()?;

                if let Value::Object(map) = &content {
                    let mut last_uuid = String::new();
                    for (key, value) in map {
                        let uuid = self.set_range_field_value(schema, field, key, value.clone(), aref_uuid.clone(), source_pub_key.clone())?;
                        last_uuid = uuid;
                    }
                    return Ok(last_uuid);
                } else {
                    return Err(SchemaError::InvalidData(format!(
                        "Range field {} expects JSON object, got: {:?}", field, content
                    )));
                }
            }

            let aref_uuid = ctx.get_or_create_atom_ref()?;
            info!(
                "🆔 Got/created aref_uuid for {}.{}: {}",
                schema.name, field, aref_uuid
            );

            let prev_atom_uuid = {
                let ref_atoms = ctx.atom_manager.get_ref_atoms();
                let guard = ref_atoms.lock().map_err(|_| {
                    SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string())
                })?;
                guard
                    .get(&aref_uuid)
                    .map(|aref| aref.get_atom_uuid().to_string())
            };
            info!(
                "📜 Previous atom_uuid for aref {}: {:?}",
                aref_uuid, prev_atom_uuid
            );

            ctx.create_and_update_atom(prev_atom_uuid.clone(), content.clone(), None)?;
            info!(
                "✅ Created/updated atom for {}.{} with content: {:?}, prev_uuid: {:?}",
                schema.name, field, content, prev_atom_uuid
            );
            aref_uuid
        }; // ctx is dropped here

        // DO NOT set ref_atom_uuid here on the schema clone - it will be lost
        // The ref_atom_uuid should only be set when the schema manager persists the schema
        info!(
            "🔗 AtomRef created with UUID: {} for {}.{} (not setting on field definition yet)",
            aref_uuid, schema.name, field
        );

        info!(
            "✅ set_field_value COMPLETE - schema: {}, field: {}, aref_uuid: {}, content: {:?}",
            schema.name, field, aref_uuid, content
        );

        Ok(aref_uuid)
    }

    /// Sets a range field value and creates the corresponding AtomRefRange.
    ///
    /// **CRITICAL: ref_atom_uuid Management**
    ///
    /// This method creates an AtomRefRange in the atom manager and returns the UUID.
    /// It does NOT set the ref_atom_uuid on the field definition - that must be done
    /// by the caller using the schema manager to ensure proper persistence.
    ///
    /// **Why this pattern prevents "ghost ref_atom_uuid" issues:**
    /// - Field definitions start with ref_atom_uuid = None
    /// - AtomRefRange is created in atom manager with a UUID
    /// - UUID is returned to caller (mutation logic)
    /// - Caller uses schema manager to set and persist ref_atom_uuid
    /// - This ensures ref_atom_uuid is only set when AtomRefRange actually exists
    ///
    /// **DO NOT** set ref_atom_uuid directly on the schema parameter - it's often
    /// a clone that gets discarded, leading to "ghost" UUIDs that point to nothing.
    ///
    /// Returns: The UUID of the created AtomRefRange that should be persisted via schema manager
    pub fn set_range_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content_key: &str,
        content_value: Value,
        aref_uuid: String,
        source_pub_key: String,
    ) -> Result<String, SchemaError> {
        // Range fields need special handling to populate the AtomRefRange properly
        info!("🔧 set_range_field_value - key: {}, value: {:?}", content_key, content_value);


        let mut ctx = AtomContext::new(
                schema,
                field,
                source_pub_key.clone(),
                &mut self.atom_manager,
            );

            let prev_atom_uuid = {
                let ref_atoms = ctx.atom_manager.get_ref_atoms();
                let guard = ref_atoms.lock().map_err(|_| {
                    SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string())
                })?;
                guard
                    .get(&aref_uuid)
                    .map(|aref| aref.get_atom_uuid().to_string())
            };

            ctx.create_and_update_range_atom(prev_atom_uuid.clone(), content_key, content_value, None)?;

        info!(
            "set_range_field_value - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name, field, aref_uuid
        );

        Ok(aref_uuid)
    }


    /// Updates a field value and manages the corresponding AtomRef.
    ///
    /// **CRITICAL: ref_atom_uuid Management**
    ///
    /// This method updates an existing AtomRef or creates a new one if needed.
    /// It does NOT set the ref_atom_uuid on the field definition - that must be done
    /// by the caller using the schema manager to ensure proper persistence.
    ///
    /// **Why this pattern prevents "ghost ref_atom_uuid" issues:**
    /// - Uses existing ref_atom_uuid if field already has one
    /// - Creates new AtomRef if field doesn't have ref_atom_uuid yet
    /// - Returns UUID to caller for proper persistence via schema manager
    /// - Never sets ref_atom_uuid on schema clones that get discarded
    ///
    /// **DO NOT** set ref_atom_uuid directly on the schema parameter - it's often
    /// a clone that gets discarded, leading to "ghost" UUIDs that point to nothing.
    ///
    /// Returns: The UUID of the AtomRef that should be persisted via schema manager
    pub fn update_field(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<String, SchemaError> {
        let aref_uuid = {
            let mut ctx = AtomContext::new(
                schema,
                field,
                source_pub_key.clone(),
                &mut self.atom_manager,
            );

            let field_def = ctx.get_field_def()?;
            if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
                return Err(SchemaError::InvalidField(
                    "Collection fields cannot be updated".to_string(),
                ));
            }

            // Special handling for Range fields (same as set_field_value)
            if let crate::schema::types::FieldVariant::Range(_range_field) = field_def {
                info!(
                    "🎯 Handling Range field update for {}.{}",
                    schema.name, field
                );

                let aref_uuid = ctx.get_or_create_atom_ref()?;
                
                // Range fields now handle individual key-value pairs
                // content should be a JSON object, iterate over its key-value pairs
                if let Value::Object(map) = &content {
                    let mut last_uuid = String::new();
                    for (key, value) in map {
                        last_uuid = self.set_range_field_value(schema, field, key, value.clone(), aref_uuid.clone(), source_pub_key.clone())?;
                    }
                    return Ok(last_uuid);
                } else {
                    return Err(SchemaError::InvalidData(format!(
                        "Range field {} expects JSON object, got: {:?}", field, content
                    )));
                }
            }

            let aref_uuid = ctx.get_or_create_atom_ref()?;
            let prev_atom_uuid = ctx.get_prev_atom_uuid(&aref_uuid)?;

            ctx.create_and_update_atom(Some(prev_atom_uuid), content.clone(), None)?;
            aref_uuid
        }; // ctx is dropped here

        // DO NOT set ref_atom_uuid here on the schema clone - it will be lost
        // The ref_atom_uuid should only be set when the schema manager persists the schema

        info!(
            "update_field - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name, field, aref_uuid
        );

        Ok(aref_uuid)
    }

    pub fn delete_field(
        &mut self,
        schema: &mut Schema,
        field: &str,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let aref_uuid = {
            let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

            let field_def = ctx.get_field_def()?;
            if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
                return Err(SchemaError::InvalidField(
                    "Collection fields cannot be deleted without id".to_string(),
                ));
            }

            let aref_uuid = ctx.get_or_create_atom_ref()?;
            let prev_atom_uuid = ctx.get_prev_atom_uuid(&aref_uuid)?;

            ctx.create_and_update_atom(
                Some(prev_atom_uuid),
                Value::Null,
                Some(AtomStatus::Deleted),
            )?;
            aref_uuid
        }; // ctx is dropped here

        // Update the field definition with the aref_uuid if it doesn't have one
        // We do this after dropping the context to avoid borrow conflicts
        if let Some(field_def) = schema.fields.get_mut(field) {
            if field_def.ref_atom_uuid().is_none() {
                field_def.set_ref_atom_uuid(aref_uuid.clone());
            }
        }

        info!(
            "delete_field - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name, field, aref_uuid
        );

        Ok(())
    }
}

impl Clone for FieldManager {
    fn clone(&self) -> Self {
        Self {
            atom_manager: self.atom_manager.clone(),
            retrieval_service: FieldRetrievalService::new(),
            transform_manager: Arc::clone(&self.transform_manager),
            orchestrator: Arc::clone(&self.orchestrator),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_operations::DbOperations;
    use crate::schema::types::field::RangeField;
    use crate::schema::types::FieldVariant;
    use crate::permissions::types::policy::PermissionsPolicy;
    use crate::fees::types::config::FieldPaymentConfig;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_field_manager() -> FieldManager {
        let sled_db = sled::Config::new().temporary(true).open().unwrap();
        let db_ops = DbOperations::new(sled_db).unwrap();
        let atom_manager = AtomManager::new(db_ops);
        FieldManager::new(atom_manager)
    }

    fn create_test_schema_with_range_field() -> Schema {
        let mut schema = Schema::new("TestSchema".to_string());
        
        // Create a range field with proper parameters
        let range_field = RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );
        
        schema.add_field("test_range_field".to_string(), FieldVariant::Range(range_field));

        schema
    }

    #[test]
    fn test_set_range_field_value_with_context() {
        let mut field_manager = create_test_field_manager();
        let mut schema = create_test_schema_with_range_field();
        
        // Test data - a JSON object with key-value pairs for the range field
        let content = json!({
            "key1": "value1",
            "key2": {"nested": "value2"}
        });
        
        let source_pub_key = "test_pub_key".to_string();
        let field_name = "test_range_field";

        let aref_uuid = "abcdef".to_string();

        if let Value::Object(map) = &content {
            for (key, value) in map {
                let result = field_manager.set_range_field_value(
                    &mut schema,
                    field_name,
                    key,
                    value.clone(),
                    aref_uuid.clone(),
                    source_pub_key.clone(),
                );
                assert!(result.is_ok(), "Function should succeed");
                let aref_uuid = result.unwrap();
                assert!(!aref_uuid.is_empty(), "Should return a non-empty UUID");

                // Verify that atoms were created for each key
                let ref_ranges = field_manager.atom_manager.get_ref_ranges();
                let ranges_guard = ref_ranges.lock().unwrap();
                let range = ranges_guard.get(&aref_uuid).unwrap();

                // print the keys in the range
                println!("keys in range: {:?}", range.atom_uuids.keys());
                
                assert!(range.get_atom_uuid(key).is_some(), "key should have an atom");
            }
        }


    }

    #[test]
    fn test_set_range_field_value_with_context_invalid_data() {
        let mut field_manager = create_test_field_manager();
        let mut schema = create_test_schema_with_range_field();
        
        // Test with non-object JSON (should fail)
        let content = json!("not an object");
        let source_pub_key = "test_pub_key".to_string();
        let field_name = "test_range_field";

        let aref_uuid = "abcdef".to_string();

        if let Value::Object(map) = &content {
            for (key, value) in map {
                let result = field_manager.set_range_field_value(
                    &mut schema,
                    field_name,
                    key,
                    value.clone(),
                    aref_uuid.clone(),
                    source_pub_key.clone(),
                );
                assert!(result.is_err(), "Function should fail with non-object data");
                assert!(result.unwrap_err().to_string().contains("JSON object"));
            }
        }
    }

    #[test]
    fn test_set_range_field_value_with_context_empty_object() {
        let mut field_manager = create_test_field_manager();
        let mut schema = create_test_schema_with_range_field();
        
        // Test with empty JSON object
        let content = json!({});
        let source_pub_key = "test_pub_key".to_string();
        let field_name = "test_range_field";

        let aref_uuid = "abcdef".to_string();

        if let Value::Object(map) = &content {
            for (key, value) in map {
                let result = field_manager.set_range_field_value(
                    &mut schema,
                    field_name,
                    key,
                    value.clone(),
                    aref_uuid.clone(),
                    source_pub_key.clone(),
                );
                assert!(result.is_ok(), "Function should succeed with empty object");
                let aref_uuid = result.unwrap();
                assert!(!aref_uuid.is_empty(), "Should return a non-empty UUID");
            }
        }
    }
}
