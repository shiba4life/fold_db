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
use super::field_retrieval::FieldRetrievalService;
use super::transform_manager::TransformManager;
use crate::atom::AtomStatus;
use crate::schema::types::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

use super::transform_orchestrator::TransformOrchestrator;
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
            info!(
                "📋 Field definition type for {}.{}: {:?}",
                schema.name,
                field,
                std::mem::discriminant(field_def)
            );

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
    fn set_range_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<String, SchemaError> {
        // Range fields need special handling to populate the AtomRefRange properly
        info!("🔧 set_range_field_value - content: {:?}", content);

        let aref_uuid = {
            let mut ctx = AtomContext::new(
                schema,
                field,
                source_pub_key.clone(),
                &mut self.atom_manager,
            );
            let aref_uuid = ctx.get_or_create_atom_ref()?;

            // For range fields, we don't create a main atom with the full JSON content.
            // Instead, we directly process the range field content to create individual atoms
            // for each key-value pair and populate the AtomRefRange.

            // Clone content for Range field processing
            let content_for_range = content.clone();

            // Check if this field is the range_key field - it should be handled as primitive
            if let Some(range_key) = schema.range_key() {
                if field == range_key {
                    // For range_key field, store the primitive value directly as a single atom
                    info!(
                        "🔑 Processing range_key field '{}' with primitive value: {:?}",
                        field, content_for_range
                    );

                    let key_atom = self
                        .atom_manager
                        .create_atom(
                            &schema.name,
                            source_pub_key.clone(),
                            None,
                            content_for_range,
                            None,
                        )
                        .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

                    info!(
                        "✅ Created atom for range_key field '{}': {}",
                        field,
                        key_atom.uuid()
                    );

                    // For range_key fields, we don't need to update AtomRefRange with key-value pairs
                    // The primitive value is stored directly in the atom
                } else {
                    // For non-range_key fields, process as object with key-value pairs
                    if let Some(obj) = content_for_range.as_object() {
                        info!(
                            "📦 Range field '{}' has {} key-value pairs",
                            field,
                            obj.len()
                        );

                        // Get existing AtomRefRange to find previous atoms for each key
                        let existing_range = {
                            let ref_ranges = self.atom_manager.get_ref_ranges();
                            let ranges_guard = ref_ranges.lock().map_err(|_| {
                                SchemaError::InvalidData("Failed to lock ref_ranges".to_string())
                            })?;
                            ranges_guard.get(&aref_uuid).cloned()
                        };

                        for (key, value) in obj {
                            // Find the latest atom UUID for this key to maintain atom chain
                            let prev_atom_uuid = if let Some(ref range) = existing_range {
                                range
                                    .get_atom_uuids(key)
                                    .and_then(|uuids| uuids.last().cloned())
                            } else {
                                None
                            };

                            let prev_atom_uuid_for_log = prev_atom_uuid.clone();
                            info!(
                                "🔗 For key '{}', previous atom UUID: {:?}",
                                key, prev_atom_uuid_for_log
                            );

                            // Create a separate atom for each key-value pair with proper atom chaining
                            let key_atom = self
                                .atom_manager
                                .create_atom(
                                    &schema.name,
                                    source_pub_key.clone(),
                                    prev_atom_uuid, // Link to previous atom for this key
                                    value.clone(),
                                    None,
                                )
                                .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

                            info!("🔑 Created atom for key: {} -> value: {:?} -> atom: {} (aref_uuid: {}, prev: {:?})",
                                    key, value, key_atom.uuid(), aref_uuid, prev_atom_uuid_for_log);

                            self.atom_manager
                                .update_atom_ref_range(
                                    &aref_uuid,
                                    key_atom.uuid().to_string(),
                                    key.clone(),
                                    source_pub_key.clone(),
                                )
                                .map_err(|e| SchemaError::InvalidData(e.to_string()))?;
                        }
                        info!("✅ Finished creating atoms and updating AtomRefRange for all keys");
                    } else {
                        return Err(SchemaError::InvalidData(format!(
                            "Non-range_key field '{}' must be a JSON object with key-value pairs",
                            field
                        )));
                    }
                }
            } else {
                // Not a range schema, fall back to original validation
                if let Some(obj) = content_for_range.as_object() {
                    info!("📦 Range field has {} key-value pairs", obj.len());

                    // Get existing AtomRefRange to find previous atoms for each key
                    let existing_range = {
                        let ref_ranges = self.atom_manager.get_ref_ranges();
                        let ranges_guard = ref_ranges.lock().map_err(|_| {
                            SchemaError::InvalidData("Failed to lock ref_ranges".to_string())
                        })?;
                        ranges_guard.get(&aref_uuid).cloned()
                    };

                    for (key, value) in obj {
                        // Find the latest atom UUID for this key to maintain atom chain
                        let prev_atom_uuid = if let Some(ref range) = existing_range {
                            range
                                .get_atom_uuids(key)
                                .and_then(|uuids| uuids.last().cloned())
                        } else {
                            None
                        };

                        let prev_atom_uuid_for_log = prev_atom_uuid.clone();
                        info!(
                            "🔗 For key '{}', previous atom UUID: {:?}",
                            key, prev_atom_uuid_for_log
                        );

                        // Create a separate atom for each key-value pair with proper atom chaining
                        let key_atom = self
                            .atom_manager
                            .create_atom(
                                &schema.name,
                                source_pub_key.clone(),
                                prev_atom_uuid, // Link to previous atom for this key
                                value.clone(),
                                None,
                            )
                            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

                        info!("🔑 Created atom for key: {} -> value: {:?} -> atom: {} (aref_uuid: {}, prev: {:?})",
                                key, value, key_atom.uuid(), aref_uuid, prev_atom_uuid_for_log);

                        self.atom_manager
                            .update_atom_ref_range(
                                &aref_uuid,
                                key_atom.uuid().to_string(),
                                key.clone(),
                                source_pub_key.clone(),
                            )
                            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;
                    }
                    info!("✅ Finished creating atoms and updating AtomRefRange for all keys");
                } else {
                    return Err(SchemaError::InvalidData(
                        "Range field data must be a JSON object with key-value pairs".to_string(),
                    ));
                }
            }

            aref_uuid
        }; // ctx is dropped here

        // DO NOT set ref_atom_uuid here on the schema clone - it will be lost
        // The ref_atom_uuid should only be set when the schema manager persists the schema

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
                return self.set_range_field_value(schema, field, content, source_pub_key);
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
