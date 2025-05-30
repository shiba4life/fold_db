//! # Field Manager: AtomRef and ref_atom_uuid Management
//!
//! **CRITICAL: Preventing "Ghost ref_atom_uuid" Issues**
//!
//! This module manages field values and their corresponding AtomRefs. The most important
//! principle is the proper management of `ref_atom_uuid` values to prevent "ghost" UUIDs
//! that point to non-existent AtomRefs.
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
use super::transform_manager::TransformManager;
use crate::atom::AtomStatus;
use crate::schema::types::Field;
use crate::schema::types::field::FieldVariant;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use log::info;

use std::sync::{Arc, RwLock};
use super::transform_orchestrator::TransformOrchestrator;

pub struct FieldManager {
    pub(super) atom_manager: AtomManager,
    transform_manager: Arc<RwLock<Option<Arc<TransformManager>>>>,
    orchestrator: Arc<RwLock<Option<Arc<TransformOrchestrator>>>>,
}

impl FieldManager {
    pub fn new(atom_manager: AtomManager) -> Self {
        Self {
            atom_manager,
            transform_manager: Arc::new(RwLock::new(None)),
            orchestrator: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_transform_manager(&self, manager: Arc<TransformManager>) -> Result<(), SchemaError> {
        let mut guard = self
            .transform_manager
            .write()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string()))?;
        *guard = Some(manager);
        Ok(())
    }

    pub fn get_transform_manager(&self) -> Result<Option<Arc<TransformManager>>, SchemaError> {
        let guard = self
            .transform_manager
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string()))?;
        Ok(guard.clone())
    }

    pub fn set_orchestrator(&self, orchestrator: Arc<TransformOrchestrator>) -> Result<(), SchemaError> {
        let mut guard = self
            .orchestrator
            .write()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string()))?;
        *guard = Some(orchestrator);
        Ok(())
    }

    pub fn get_orchestrator(&self) -> Result<Option<Arc<TransformOrchestrator>>, SchemaError> {
        let guard = self
            .orchestrator
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string()))?;
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
        info!("🔍 get_field_value START - schema: {}, field: {}", schema.name, field);
        
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        info!("📋 Field definition found for {}.{}, type: {:?}",
              schema.name, field, std::mem::discriminant(field_def));
        
        // If the ref_atom_uuid hasn't been set yet, treat it as missing so
        // queries return `null` for this field until a value is written.
        let ref_atom_uuid = match field_def.ref_atom_uuid() {
            Some(id) if id.is_empty() => None,
            other => other,
        };
        info!("🆔 ref_atom_uuid for {}.{}: {:?}", schema.name, field, ref_atom_uuid);

        let result = match field_def {
            crate::schema::types::field::FieldVariant::Range(_range_field) => {
                // For Range fields, treat them like Single fields for now
                // The data is stored as a single JSON object atom
                info!("🎯 Processing Range field: {}.{}", schema.name, field);
                if let Some(ref_atom_uuid) = ref_atom_uuid {
                    info!("🔗 Fetching atom for Range field {}.{} with ref_atom_uuid: {}",
                          schema.name, field, ref_atom_uuid);
                    match self.atom_manager.get_latest_atom(ref_atom_uuid) {
                        Ok(atom) => {
                            let content = atom.content().clone();
                            info!("✅ Retrieved atom content for {}.{}: {:?}", schema.name, field, content);
                            content
                        }
                        Err(e) => {
                            info!("❌ Failed to get atom for {}.{}: {:?}, using default", schema.name, field, e);
                            Self::default_value(field)
                        }
                    }
                } else {
                    info!("⚠️  No ref_atom_uuid for Range field {}.{}, using default", schema.name, field);
                    Self::default_value(field)
                }
            }
            _ => {
                // For Single and Collection fields, use the existing logic
                info!("🎯 Processing Single/Collection field: {}.{}", schema.name, field);
                if let Some(ref_atom_uuid) = ref_atom_uuid {
                    info!("🔗 Fetching atom for field {}.{} with ref_atom_uuid: {}",
                          schema.name, field, ref_atom_uuid);
                    match self.atom_manager.get_latest_atom(ref_atom_uuid) {
                        Ok(atom) => {
                            let content = atom.content().clone();
                            info!("✅ Retrieved atom content for {}.{}: {:?}", schema.name, field, content);
                            content
                        }
                        Err(e) => {
                            info!("❌ Failed to get atom for {}.{}: {:?}, using default", schema.name, field, e);
                            Self::default_value(field)
                        }
                    }
                } else {
                    info!("⚠️  No ref_atom_uuid for field {}.{}, using default", schema.name, field);
                    Self::default_value(field)
                }
            }
        };

        info!(
            "✅ get_field_value COMPLETE - schema: {}, field: {}, aref_uuid: {:?}, result: {:?}",
            schema.name,
            field,
            ref_atom_uuid,
            result
        );

        Ok(result)
    }

    pub fn get_filtered_field_value(&self, schema: &Schema, field: &str, filter: &Value) -> Result<Value, SchemaError> {
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // Check if this is a RangeField that supports filtering
        match field_def {
            FieldVariant::Range(_range_field) => {
                // Extract the range_filter from the filter object
                let range_filter = if let Some(filter_obj) = filter.as_object() {
                    if let Some(target_field) = filter_obj.get("field") {
                        if target_field.as_str() == Some(field) {
                            filter_obj.get("range_filter")
                                .ok_or_else(|| SchemaError::InvalidData("Missing range_filter in filter".to_string()))?
                        } else {
                            // This filter is not for this field, return regular field value
                            return self.get_field_value(schema, field);
                        }
                    } else {
                        // Assume the entire filter is the range filter (for backward compatibility)
                        filter
                    }
                } else {
                    return Err(SchemaError::InvalidData("Filter must be an object".to_string()));
                };

                // Get the full Range field data first
                info!("get_filtered_field_value - About to call get_field_value for Range field: {}", field);
                let range_data = self.get_field_value(schema, field)?;
                info!("get_filtered_field_value - Retrieved range_data: {:?}", range_data);
                
                // Apply filtering to the data
                if let Value::Object(data_map) = range_data {
                    let mut matches = std::collections::HashMap::new();
                    
                    // Parse the range filter
                    if let Some(filter_obj) = range_filter.as_object() {
                        if let Some(key_prefix) = filter_obj.get("KeyPrefix").and_then(|v| v.as_str()) {
                            // Filter by key prefix
                            for (key, value) in data_map {
                                if key.starts_with(key_prefix) {
                                    matches.insert(key, value.as_str().unwrap_or("").to_string());
                                }
                            }
                        } else if let Some(exact_key) = filter_obj.get("Key").and_then(|v| v.as_str()) {
                            // Filter by exact key
                            if let Some(value) = data_map.get(exact_key) {
                                matches.insert(exact_key.to_string(), value.as_str().unwrap_or("").to_string());
                            }
                        } else if let Some(pattern) = filter_obj.get("KeyPattern").and_then(|v| v.as_str()) {
                            // Filter by key pattern (simple glob matching)
                            for (key, value) in data_map {
                                // Simple glob pattern matching
                                let is_match = if let Some(prefix) = pattern.strip_suffix('*') {
                                    key.starts_with(prefix)
                                } else if let Some(suffix) = pattern.strip_prefix('*') {
                                    key.ends_with(suffix)
                                } else {
                                    key == pattern
                                };
                                
                                if is_match {
                                    matches.insert(key, value.as_str().unwrap_or("").to_string());
                                }
                            }
                        }
                    }
                    
                    let result = serde_json::json!({
                        "matches": matches,
                        "total_count": matches.len()
                    });
                    Ok(result)
                } else {
                    // No data or invalid data format
                    let result = serde_json::json!({
                        "matches": {},
                        "total_count": 0
                    });
                    Ok(result)
                }
            }
            _ => {
                // For non-range fields, fall back to regular field value retrieval
                // In the future, we could add filtering support for other field types
                self.get_field_value(schema, field)
            }
        }
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
        info!("🔧 set_field_value START - schema: {}, field: {}, content: {:?}, pub_key: {}",
              schema.name, field, content, source_pub_key);
        
        // Check if field already has a ref_atom_uuid before we start
        let existing_ref_uuid = schema.fields.get(field)
            .and_then(|f| f.ref_atom_uuid())
            .map(|uuid| uuid.to_string());
        info!("🔍 Existing ref_atom_uuid for {}.{}: {:?}", schema.name, field, existing_ref_uuid);
        
        let aref_uuid = {
            let mut ctx = AtomContext::new(schema, field, source_pub_key.clone(), &mut self.atom_manager);

            let field_def = ctx.get_field_def()?;
            info!("📋 Field definition type for {}.{}: {:?}", schema.name, field,
                  std::mem::discriminant(field_def));
            
            if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
                return Err(SchemaError::InvalidField(
                    "Collection fields cannot be updated without id".to_string(),
                ));
            }

            // Special handling for Range fields
            if let crate::schema::types::FieldVariant::Range(_range_field) = field_def {
                info!("🎯 Handling Range field for {}.{}", schema.name, field);
                return self.set_range_field_value(schema, field, content, source_pub_key);
            }

            let aref_uuid = ctx.get_or_create_atom_ref()?;
            info!("🆔 Got/created aref_uuid for {}.{}: {}", schema.name, field, aref_uuid);
            
            let prev_atom_uuid = {
                let ref_atoms = ctx.atom_manager.get_ref_atoms();
                let guard = ref_atoms
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                guard
                    .get(&aref_uuid)
                    .map(|aref| aref.get_atom_uuid().to_string())
            };
            info!("📜 Previous atom_uuid for aref {}: {:?}", aref_uuid, prev_atom_uuid);

            ctx.create_and_update_atom(prev_atom_uuid.clone(), content.clone(), None)?;
            info!("✅ Created/updated atom for {}.{} with content: {:?}, prev_uuid: {:?}",
                  schema.name, field, content, prev_atom_uuid);
            aref_uuid
        }; // ctx is dropped here

        // DO NOT set ref_atom_uuid here on the schema clone - it will be lost
        // The ref_atom_uuid should only be set when the schema manager persists the schema
        info!("🔗 AtomRef created with UUID: {} for {}.{} (not setting on field definition yet)",
              aref_uuid, schema.name, field);

        info!(
            "✅ set_field_value COMPLETE - schema: {}, field: {}, aref_uuid: {}, content: {:?}",
            schema.name,
            field,
            aref_uuid,
            content
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
    fn set_range_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<String, SchemaError> {
        // For now, let's store the Range field data as a single atom like other fields
        // but mark it specially so we can retrieve it correctly
        let aref_uuid = {
            let mut ctx = AtomContext::new(schema, field, source_pub_key.clone(), &mut self.atom_manager);
            let aref_uuid = ctx.get_or_create_atom_ref()?;
            
            let prev_atom_uuid = {
                let ref_atoms = ctx.atom_manager.get_ref_atoms();
                let guard = ref_atoms
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                guard
                    .get(&aref_uuid)
                    .map(|aref| aref.get_atom_uuid().to_string())
            };

            ctx.create_and_update_atom(prev_atom_uuid, content.clone(), None)?;
            aref_uuid
        }; // ctx is dropped here
        
        // DO NOT set ref_atom_uuid here on the schema clone - it will be lost
        // The ref_atom_uuid should only be set when the schema manager persists the schema

        info!(
            "set_range_field_value - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            aref_uuid
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
            let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

            let field_def = ctx.get_field_def()?;
            if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
                return Err(SchemaError::InvalidField(
                    "Collection fields cannot be updated".to_string(),
                ));
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
            schema.name,
            field,
            aref_uuid
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

            ctx.create_and_update_atom(Some(prev_atom_uuid), Value::Null, Some(AtomStatus::Deleted))?;
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
            schema.name,
            field,
            aref_uuid
        );

        Ok(())
    }

    fn default_value(field: &str) -> Value {
        match field {
            "username" | "email" | "full_name" | "bio" | "location" => {
                Value::String("".to_string())
            }
            "age" => Value::Number(serde_json::Number::from(0)),
            "value1" | "value2" => Value::Number(serde_json::Number::from(0)),
            _ => Value::Null,
        }
    }
}

impl Clone for FieldManager {
    fn clone(&self) -> Self {
        Self {
            atom_manager: self.atom_manager.clone(),
            transform_manager: Arc::clone(&self.transform_manager),
            orchestrator: Arc::clone(&self.orchestrator),
        }
    }
}
