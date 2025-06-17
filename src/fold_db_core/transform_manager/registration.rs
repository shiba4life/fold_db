//! Transform registration and discovery module
//!
//! This module handles:
//! - Transform loading from database
//! - Transform registration and validation
//! - Transform discovery and existence checks
//! - Registration state management

use super::config::*;
use super::state::TransformManagerState;
use crate::db_operations::DbOperations;
use crate::fold_db_core::transform_manager::utils::TransformUtils;
use crate::schema::types::{SchemaError, Transform, TransformRegistration};
use crate::transform::executor::TransformExecutor;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Transform registration manager
pub struct TransformRegistrationManager {
    db_ops: Arc<DbOperations>,
    state: Arc<TransformManagerState>,
}

impl TransformRegistrationManager {
    /// Create a new registration manager
    pub fn new(db_ops: Arc<DbOperations>, state: Arc<TransformManagerState>) -> Self {
        Self { db_ops, state }
    }

    /// Load transforms from database during initialization
    pub fn load_transforms_from_database(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        info!("📋 Loading transforms from database during initialization");

        let mut loaded_transforms = HashMap::new();
        let transform_ids = self.db_ops.list_transforms()?;

        for transform_id in transform_ids {
            match self.db_ops.get_transform(&transform_id) {
                Ok(Some(transform)) => {
                    info!(
                        "📋 Loading transform '{}' with inputs: {:?}, output: {}",
                        transform_id,
                        transform.get_inputs(),
                        transform.get_output()
                    );
                    loaded_transforms.insert(transform_id, transform);
                }
                Ok(None) => {
                    warn!(
                        "Transform '{}' not found in storage during initialization",
                        transform_id
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to load transform '{}' during initialization: {}",
                        transform_id, e
                    );
                    return Err(e);
                }
            }
        }

        // Load transforms into state
        {
            let mut registered_transforms = TransformUtils::write_lock(&self.state.registered_transforms, "registered_transforms")?;
            *registered_transforms = loaded_transforms.clone();
        }

        info!(
            "✅ Successfully loaded {} transforms from database",
            loaded_transforms.len()
        );
        Ok(loaded_transforms)
    }

    /// Reload transforms from database when changes occur
    pub fn reload_transforms(&self) -> Result<(), SchemaError> {
        info!("🔄 Reloading transforms from database");

        let transform_ids = self.db_ops.list_transforms()?;

        let mut registered_transforms = TransformUtils::write_lock(&self.state.registered_transforms, "registered_transforms")?;
        let mut field_to_transforms = TransformUtils::write_lock(&self.state.field_to_transforms, "field_to_transforms")?;
        let mut transform_to_fields = TransformUtils::write_lock(&self.state.transform_to_fields, "transform_to_fields")?;

        for transform_id in transform_ids {
            // Skip if transform is already loaded
            if registered_transforms.contains_key(&transform_id) {
                continue;
            }

            match self.db_ops.get_transform(&transform_id) {
                Ok(Some(transform)) => {
                    info!(
                        "📋 Loaded new transform '{}' with inputs: {:?}, output: {}",
                        transform_id,
                        transform.get_inputs(),
                        transform.get_output()
                    );
                    registered_transforms.insert(transform_id.clone(), transform.clone());

                    // Register field mappings for the new transform
                    for input in transform.get_inputs() {
                        field_to_transforms
                            .entry(input.clone())
                            .or_insert_with(HashSet::new)
                            .insert(transform_id.clone());
                        transform_to_fields
                            .entry(transform_id.clone())
                            .or_insert_with(HashSet::new)
                            .insert(input.to_string());
                    }
                }
                Ok(None) => {
                    warn!(
                        "Transform '{}' not found in storage during reload",
                        transform_id
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to load transform '{}' during reload: {}",
                        transform_id, e
                    );
                }
            }
        }

        info!("✅ Transform reload completed");
        Ok(())
    }

    /// Register a new transform with event-driven approach
    pub fn register_transform_event_driven(
        &self,
        registration: TransformRegistration,
    ) -> Result<(), SchemaError> {
        let TransformRegistration {
            transform_id,
            mut transform,
            input_arefs,
            input_names,
            trigger_fields,
            output_aref,
            schema_name,
            field_name,
        } = registration;

        info!(
            "📝 Registering transform '{}' with {} inputs, output: {}.{}",
            transform_id,
            input_arefs.len(),
            schema_name,
            field_name
        );

        // Validate transform before registration
        TransformUtils::validate_transform_registration(&transform_id, &transform)?;
        TransformExecutor::validate_transform(&transform)?;

        // Prepare transform
        let output_field = format!("{}.{}", schema_name, field_name);
        transform.set_output(output_field.clone());

        // Store transform in database
        self.db_ops.store_transform(&transform_id, &transform)?;

        // Update in-memory state
        self.state.update_transform_registration(
            &transform_id,
            transform,
            &input_arefs,
            &input_names,
            &trigger_fields,
            &output_aref,
        )?;

        info!(
            "✅ Registered transform '{}' with output '{}' and {} input references",
            transform_id, output_field, input_arefs.len()
        );

        // Persist state changes to database
        self.state.persist_mappings(&self.db_ops)?;
        Ok(())
    }

    /// Register transform with automatic dependency detection
    pub fn register_transform_auto(
        &self,
        transform_id: String,
        transform: Transform,
        output_aref: String,
        schema_name: String,
        field_name: String,
    ) -> Result<(), SchemaError> {
        info!(
            "📝 Auto-registering transform '{}' with automatic dependency detection",
            transform_id
        );

        // Analyze dependencies automatically
        let dependencies = transform
            .analyze_dependencies()
            .into_iter()
            .collect::<Vec<String>>();

        let trigger_fields = Vec::new(); // Auto-detection doesn't specify trigger fields
        let output_field = format!("{}.{}", schema_name, field_name);

        let registration = TransformRegistration {
            transform_id: transform_id.clone(),
            transform,
            input_arefs: dependencies.clone(),
            input_names: Vec::new(),
            trigger_fields,
            output_aref,
            schema_name,
            field_name,
        };

        self.register_transform_event_driven(registration)?;

        info!(
            "✅ Auto-registered transform '{}' with output '{}' and {} dependencies",
            transform_id, output_field, dependencies.len()
        );

        Ok(())
    }

    /// Unregister a transform and clean up all related state
    pub fn unregister_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        info!("🗑️ Unregistering transform '{}'", transform_id);

        // Remove from database
        let existed = self.db_ops.delete_transform(transform_id)?;

        // Remove from in-memory state
        let found = self.state.remove_transform(transform_id)?;

        if found {
            // Persist state changes
            self.state.persist_mappings(&self.db_ops)?;
            info!("✅ Successfully unregistered transform '{}'", transform_id);
        } else {
            info!("ℹ️ Transform '{}' was not found during unregistration", transform_id);
        }

        Ok(existed || found)
    }

    /// Check if a transform exists
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        self.state.transform_exists(transform_id)
    }

    /// List all registered transforms
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        self.state.list_transforms()
    }

    /// Get transforms that depend on a specific atom reference
    pub fn get_dependent_transforms(&self, aref_uuid: &str) -> Result<HashSet<String>, SchemaError> {
        self.state.get_dependent_transforms(aref_uuid)
    }

    /// Get atom references that a transform depends on
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        self.state.get_transform_inputs(transform_id)
    }

    /// Get the output atom reference for a transform
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        self.state.get_transform_output(transform_id)
    }

    /// Get all transforms that should run when a field is updated
    pub fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        self.state.get_transforms_for_field(schema_name, field_name)
    }

    /// Load persisted mappings from database
    pub fn load_persisted_mappings(&self) -> Result<(), SchemaError> {
        self.state.load_persisted_mappings(&self.db_ops)
    }

    /// Create a registration from static mapping data  
    pub fn load_persisted_mappings_static(
        db_ops: &Arc<DbOperations>,
    ) -> Result<(
        HashMap<String, HashSet<String>>,   // aref_to_transforms
        HashMap<String, HashSet<String>>,   // transform_to_arefs  
        HashMap<String, HashMap<String, String>>, // transform_input_names
        HashMap<String, HashSet<String>>,   // field_to_transforms
        HashMap<String, HashSet<String>>,   // transform_to_fields
        HashMap<String, String>,            // transform_outputs
    ), SchemaError> {
        info!("🔄 Loading persisted mappings using static method");

        let aref_to_transforms = Self::load_mapping_static(db_ops, AREF_TO_TRANSFORMS_KEY)?;
        let transform_to_arefs = Self::load_mapping_static(db_ops, TRANSFORM_TO_AREFS_KEY)?;
        let transform_input_names = Self::load_mapping_static(db_ops, TRANSFORM_INPUT_NAMES_KEY)?;
        let field_to_transforms = Self::load_mapping_static(db_ops, FIELD_TO_TRANSFORMS_KEY)?;
        let transform_to_fields = Self::load_mapping_static(db_ops, TRANSFORM_TO_FIELDS_KEY)?;
        let transform_outputs = Self::load_mapping_static(db_ops, TRANSFORM_OUTPUTS_KEY)?;

        info!("✅ Successfully loaded all persisted mappings using static method");
        
        Ok((
            aref_to_transforms,
            transform_to_arefs,
            transform_input_names,
            field_to_transforms,
            transform_to_fields,
            transform_outputs,
        ))
    }

    /// Generic static method to load a mapping from database
    fn load_mapping_static<T>(
        db_ops: &Arc<DbOperations>,
        key: &str,
    ) -> Result<T, SchemaError>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        match db_ops.get_transform_mapping(key) {
            Ok(Some(data)) => {
                let mapping = TransformUtils::deserialize_mapping(&data, key)?;
                info!("✅ Loaded mapping: {}", key);
                Ok(mapping)
            }
            Ok(None) => {
                info!("ℹ️ No persisted mapping found for: {}", key);
                Ok(T::default())
            }
            Err(e) => {
                error!("❌ Failed to load mapping {}: {}", key, e);
                Err(e)
            }
        }
    }
}