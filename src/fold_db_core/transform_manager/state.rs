//! In-memory state management for the transform manager
//!
//! This module handles all in-memory state operations including:
//! - Transform registration cache
//! - Mapping tables between transforms and fields
//! - State query methods
//! - Persistence of state to database

use super::config::*;
use crate::db_operations::DbOperations;
use crate::fold_db_core::transform_manager::utils::TransformUtils;
use crate::schema::types::{SchemaError, Transform};
use log::{error, info};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// In-memory state container for transform management
pub struct TransformManagerState {
    /// In-memory cache of registered transforms
    pub registered_transforms: RwLock<HashMap<String, Transform>>,
    /// Maps atom reference UUIDs to the transforms that depend on them
    pub aref_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their dependent atom reference UUIDs
    pub transform_to_arefs: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to input field names keyed by atom ref UUID
    pub transform_input_names: RwLock<HashMap<String, HashMap<String, String>>>,
    /// Maps schema.field keys to transforms triggered by them
    pub field_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to the fields that trigger them
    pub transform_to_fields: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their output atom reference UUIDs
    pub transform_outputs: RwLock<HashMap<String, String>>,
}

impl Default for TransformManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformManagerState {
    /// Create a new empty state container
    pub fn new() -> Self {
        Self {
            registered_transforms: RwLock::new(HashMap::new()),
            aref_to_transforms: RwLock::new(HashMap::new()),
            transform_to_arefs: RwLock::new(HashMap::new()),
            transform_input_names: RwLock::new(HashMap::new()),
            field_to_transforms: RwLock::new(HashMap::new()),
            transform_to_fields: RwLock::new(HashMap::new()),
            transform_outputs: RwLock::new(HashMap::new()),
        }
    }

    /// Load persisted mappings from database into state
    pub fn load_persisted_mappings(
        &self,
        db_ops: &Arc<DbOperations>,
    ) -> Result<(), SchemaError> {
        info!("🔄 Loading persisted mappings from database");

        // Load each mapping type
        self.load_mapping(db_ops, AREF_TO_TRANSFORMS_KEY, &self.aref_to_transforms)?;
        self.load_mapping(db_ops, TRANSFORM_TO_AREFS_KEY, &self.transform_to_arefs)?;
        self.load_mapping(db_ops, TRANSFORM_INPUT_NAMES_KEY, &self.transform_input_names)?;
        self.load_mapping(db_ops, FIELD_TO_TRANSFORMS_KEY, &self.field_to_transforms)?;
        self.load_mapping(db_ops, TRANSFORM_TO_FIELDS_KEY, &self.transform_to_fields)?;
        self.load_mapping(db_ops, TRANSFORM_OUTPUTS_KEY, &self.transform_outputs)?;

        info!("✅ Successfully loaded all persisted mappings");
        Ok(())
    }

    /// Generic method to load a mapping from database
    fn load_mapping<T>(
        &self,
        db_ops: &Arc<DbOperations>,
        key: &str,
        target: &RwLock<T>,
    ) -> Result<(), SchemaError>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        match db_ops.get_transform_mapping(key) {
            Ok(Some(data)) => {
                let mapping = TransformUtils::deserialize_mapping(&data, key)?;
                let mut target_lock = TransformUtils::write_lock(target, key)?;
                *target_lock = mapping;
                info!("✅ Loaded mapping: {}", key);
            }
            Ok(None) => {
                info!("ℹ️ No persisted mapping found for: {}", key);
            }
            Err(e) => {
                error!("❌ Failed to load mapping {}: {}", key, e);
                return Err(e);
            }
        }
        Ok(())
    }

    /// Persist all mappings to database
    pub fn persist_mappings(&self, db_ops: &Arc<DbOperations>) -> Result<(), SchemaError> {
        info!("💾 Persisting all mappings to database");

        TransformUtils::store_mapping(db_ops, &self.aref_to_transforms, AREF_TO_TRANSFORMS_KEY, "aref_to_transforms")?;
        TransformUtils::store_mapping(db_ops, &self.transform_to_arefs, TRANSFORM_TO_AREFS_KEY, "transform_to_arefs")?;
        TransformUtils::store_mapping(db_ops, &self.transform_input_names, TRANSFORM_INPUT_NAMES_KEY, "transform_input_names")?;
        TransformUtils::store_mapping(db_ops, &self.field_to_transforms, FIELD_TO_TRANSFORMS_KEY, "field_to_transforms")?;
        TransformUtils::store_mapping(db_ops, &self.transform_to_fields, TRANSFORM_TO_FIELDS_KEY, "transform_to_fields")?;
        TransformUtils::store_mapping(db_ops, &self.transform_outputs, TRANSFORM_OUTPUTS_KEY, "transform_outputs")?;

        info!("✅ All mappings persisted successfully");
        Ok(())
    }

    /// Check if a transform exists in the registration cache
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let registered_transforms = TransformUtils::read_lock(&self.registered_transforms, "registered_transforms")?;
        Ok(registered_transforms.contains_key(transform_id))
    }

    /// Get all registered transforms
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        let registered_transforms = TransformUtils::read_lock(&self.registered_transforms, "registered_transforms")?;
        Ok(registered_transforms.clone())
    }

    /// Get all transforms that depend on the specified atom reference
    pub fn get_dependent_transforms(&self, aref_uuid: &str) -> Result<HashSet<String>, SchemaError> {
        let aref_to_transforms = TransformUtils::read_lock(&self.aref_to_transforms, "aref_to_transforms")?;
        Ok(aref_to_transforms
            .get(aref_uuid)
            .cloned()
            .unwrap_or_default())
    }

    /// Get all atom references that a transform depends on
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        let transform_to_arefs = TransformUtils::read_lock(&self.transform_to_arefs, "transform_to_arefs")?;
        Ok(transform_to_arefs
            .get(transform_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Get the output atom reference for a transform
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        let transform_outputs = TransformUtils::read_lock(&self.transform_outputs, "transform_outputs")?;
        Ok(transform_outputs.get(transform_id).cloned())
    }

    /// Get all transforms that should run when the specified field is updated
    pub fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let key = format!("{}.{}", schema_name, field_name);
        let field_to_transforms = TransformUtils::read_lock(&self.field_to_transforms, "field_to_transforms")?;

        let result = field_to_transforms.get(&key).cloned().unwrap_or_default();

        // DEBUG: Log field mapping lookup
        info!(
            "🔍 DEBUG TransformManagerState: Looking up transforms for '{}' - found {} transforms: {:?}",
            key,
            result.len(),
            result
        );

        // DEBUG: Log all field mappings for diagnostics if empty
        if result.is_empty() {
            info!("🔍 DEBUG TransformManagerState: Current field_to_transforms state:");
            for (field_key, transforms) in field_to_transforms.iter() {
                info!("  📋 '{}' -> {:?}", field_key, transforms);
            }
            if field_to_transforms.is_empty() {
                info!("⚠️ DEBUG TransformManagerState: No field mappings found in state!");
            }
        }

        Ok(result)
    }

    /// Update a transform registration in the state
    pub fn update_transform_registration(
        &self,
        transform_id: &str,
        transform: Transform,
        input_arefs: &[String],
        input_names: &[String],
        trigger_fields: &[String],
        output_aref: &str,
    ) -> Result<(), SchemaError> {
        // Update registered transforms
        {
            let mut registered_transforms = TransformUtils::write_lock(&self.registered_transforms, "registered_transforms")?;
            registered_transforms.insert(transform_id.to_string(), transform);
        }

        // Update output mapping
        {
            let mut transform_outputs = TransformUtils::write_lock(&self.transform_outputs, "transform_outputs")?;
            transform_outputs.insert(transform_id.to_string(), output_aref.to_string());
        }

        // Update input atom references
        {
            let mut transform_to_arefs = TransformUtils::write_lock(&self.transform_to_arefs, "transform_to_arefs")?;
            let aref_set: HashSet<String> = input_arefs.iter().cloned().collect();
            transform_to_arefs.insert(transform_id.to_string(), aref_set);
        }

        // Update input names mapping
        {
            let mut transform_input_names = TransformUtils::write_lock(&self.transform_input_names, "transform_input_names")?;
            let name_map: HashMap<String, String> = input_arefs
                .iter()
                .zip(input_names.iter())
                .map(|(aref, name)| (aref.clone(), name.clone()))
                .collect();
            transform_input_names.insert(transform_id.to_string(), name_map);
        }

        // Update field trigger mappings
        self.update_field_trigger_mappings(transform_id, trigger_fields)?;

        // Update reverse mapping (aref -> transforms)
        self.update_aref_to_transforms_mapping(transform_id, input_arefs)?;

        Ok(())
    }

    /// Update field trigger mappings for a transform
    fn update_field_trigger_mappings(
        &self,
        transform_id: &str,
        trigger_fields: &[String],
    ) -> Result<(), SchemaError> {
        let mut transform_to_fields = TransformUtils::write_lock(&self.transform_to_fields, "transform_to_fields")?;
        let mut field_to_transforms = TransformUtils::write_lock(&self.field_to_transforms, "field_to_transforms")?;

        let field_set: HashSet<String> = trigger_fields.iter().cloned().collect();
        info!(
            "🔍 DEBUG: Registering field mappings for transform '{}' with trigger_fields: {:?}",
            transform_id, trigger_fields
        );

        for field_key in trigger_fields {
            let set = field_to_transforms.entry(field_key.clone()).or_default();
            set.insert(transform_id.to_string());
            info!(
                "🔗 DEBUG: Registered field mapping '{}' -> transform '{}'",
                field_key, transform_id
            );
        }
        transform_to_fields.insert(transform_id.to_string(), field_set);

        // DEBUG: Log current field mappings state
        info!("🔍 DEBUG: Current field_to_transforms state after registration:");
        for (field_key, transforms) in field_to_transforms.iter() {
            info!("  📋 '{}' -> {:?}", field_key, transforms);
        }

        Ok(())
    }

    /// Update aref to transforms mapping
    fn update_aref_to_transforms_mapping(
        &self,
        transform_id: &str,
        input_arefs: &[String],
    ) -> Result<(), SchemaError> {
        let mut aref_to_transforms = TransformUtils::write_lock(&self.aref_to_transforms, "aref_to_transforms")?;

        for aref_uuid in input_arefs {
            let transform_set = aref_to_transforms.entry(aref_uuid.clone()).or_default();
            transform_set.insert(transform_id.to_string());
        }

        Ok(())
    }

    /// Remove a transform from all state mappings
    pub fn remove_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let found = {
            let mut registered_transforms = TransformUtils::write_lock(&self.registered_transforms, "registered_transforms")?;
            registered_transforms.remove(transform_id).is_some()
        };

        if found {
            // Remove from transform outputs
            {
                let mut transform_outputs = TransformUtils::write_lock(&self.transform_outputs, "transform_outputs")?;
                transform_outputs.remove(transform_id);
            }

            // Remove field mappings
            {
                let mut transform_to_fields = TransformUtils::write_lock(&self.transform_to_fields, "transform_to_fields")?;
                let mut field_to_transforms = TransformUtils::write_lock(&self.field_to_transforms, "field_to_transforms")?;

                if let Some(fields) = transform_to_fields.remove(transform_id) {
                    for field in fields {
                        if let Some(set) = field_to_transforms.get_mut(&field) {
                            set.remove(transform_id);
                            if set.is_empty() {
                                field_to_transforms.remove(&field);
                            }
                        }
                    }
                }
            }

            // Get the input arefs for this transform
            let input_arefs = {
                let mut transform_to_arefs = TransformUtils::write_lock(&self.transform_to_arefs, "transform_to_arefs")?;
                transform_to_arefs.remove(transform_id).unwrap_or_default()
            };

            // Remove input name mapping
            {
                let mut transform_input_names = TransformUtils::write_lock(&self.transform_input_names, "transform_input_names")?;
                transform_input_names.remove(transform_id);
            }

            // Update the reverse mapping (aref -> transforms)
            {
                let mut aref_to_transforms = TransformUtils::write_lock(&self.aref_to_transforms, "aref_to_transforms")?;

                for aref_uuid in input_arefs {
                    if let Some(transform_set) = aref_to_transforms.get_mut(&aref_uuid) {
                        transform_set.remove(transform_id);

                        // Remove the entry if the set is empty
                        if transform_set.is_empty() {
                            aref_to_transforms.remove(&aref_uuid);
                        }
                    }
                }
            }
        }

        Ok(found)
    }
}