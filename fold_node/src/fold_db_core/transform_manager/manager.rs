use crate::schema::types::{Transform, SchemaError, TransformRegistration};
use crate::db_operations::DbOperations;
use crate::transform::TransformExecutor;
use super::types::{CreateAtomFn, GetAtomFn, GetFieldFn, UpdateAtomRefFn, TransformRunner};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use log::info;

pub(super) const AREF_TO_TRANSFORMS_KEY: &str = "map_aref_to_transforms";
pub(super) const TRANSFORM_TO_AREFS_KEY: &str = "map_transform_to_arefs";
pub(super) const TRANSFORM_INPUT_NAMES_KEY: &str = "map_transform_input_names";
pub(super) const FIELD_TO_TRANSFORMS_KEY: &str = "map_field_to_transforms";
pub(super) const TRANSFORM_TO_FIELDS_KEY: &str = "map_transform_to_fields";
pub(super) const TRANSFORM_OUTPUTS_KEY: &str = "map_transform_outputs";

pub struct TransformManager {
    /// Tree for storing transforms (legacy)
    pub(super) transforms_tree: sled::Tree,
    /// Unified database operations (new)
    pub(super) db_ops: Option<std::sync::Arc<crate::db_operations::DbOperations>>,
    /// In-memory cache of registered transforms
    pub(super) registered_transforms: RwLock<HashMap<String, Transform>>,
    /// Maps atom reference UUIDs to the transforms that depend on them
    pub(super) aref_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their dependent atom reference UUIDs
    pub(super) transform_to_arefs: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to input field names keyed by atom ref UUID
    pub(super) transform_input_names: RwLock<HashMap<String, HashMap<String, String>>>,
    /// Maps schema.field keys to transforms triggered by them
    pub(super) field_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to the fields that trigger them
    pub(super) transform_to_fields: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their output atom reference UUIDs
    pub(super) transform_outputs: RwLock<HashMap<String, String>>,
    /// Callback for getting an atom by its reference UUID
    pub(super) get_atom_fn: GetAtomFn,
    /// Callback for creating a new atom
    pub(super) create_atom_fn: CreateAtomFn,
    /// Callback for updating an atom reference
    pub(super) update_atom_ref_fn: UpdateAtomRefFn,
    /// Callback for retrieving field values
    pub(super) get_field_fn: GetFieldFn,
}

impl TransformManager {
    /// Creates a new TransformManager instance with unified database operations
    pub fn new(
        db_ops: std::sync::Arc<crate::db_operations::DbOperations>,
        get_atom_fn: GetAtomFn,
        create_atom_fn: CreateAtomFn,
        update_atom_ref_fn: UpdateAtomRefFn,
        get_field_fn: GetFieldFn,
    ) -> Result<Self, SchemaError> {
        // Create a legacy tree for backward compatibility with existing code
        let transforms_tree = db_ops.db().open_tree("transforms")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open transforms tree: {}", e)))?;

        // Load any persisted transforms using unified operations
        let mut registered_transforms = HashMap::new();
        let transform_ids = db_ops.list_transforms()?;
        
        for transform_id in transform_ids {
            match db_ops.get_transform(&transform_id) {
                Ok(Some(transform)) => {
                    registered_transforms.insert(transform_id, transform);
                },
                Ok(None) => {
                    log::warn!("Transform '{}' not found in storage during initialization", transform_id);
                },
                Err(e) => {
                    log::error!("Failed to load transform '{}' during initialization: {}", transform_id, e);
                    return Err(e);
                }
            }
        }

        // Load mappings using unified operations only
        let (aref_to_transforms, transform_to_arefs, transform_input_names,
             field_to_transforms, transform_to_fields, transform_outputs) =
            Self::load_persisted_mappings_unified(&db_ops)?;

        Ok(Self {
            transforms_tree,
            db_ops: Some(db_ops),
            registered_transforms: RwLock::new(registered_transforms),
            aref_to_transforms: RwLock::new(aref_to_transforms),
            transform_to_arefs: RwLock::new(transform_to_arefs),
            transform_input_names: RwLock::new(transform_input_names),
            field_to_transforms: RwLock::new(field_to_transforms),
            transform_to_fields: RwLock::new(transform_to_fields),
            transform_outputs: RwLock::new(transform_outputs),
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
            get_field_fn,
        })
    }

    /// Returns true if a transform with the given id is registered.
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let registered_transforms = self
            .registered_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire registered_transforms lock".to_string(),
                )
            })?;
        Ok(registered_transforms.contains_key(transform_id))
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        let registered_transforms = self
            .registered_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire registered_transforms lock".to_string(),
                )
            })?;
        Ok(registered_transforms.clone())
    }

    /// Gets all transforms that depend on the specified atom reference.
    pub fn get_dependent_transforms(&self, aref_uuid: &str) -> Result<HashSet<String>, SchemaError> {
        let aref_to_transforms = self
            .aref_to_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire aref_to_transforms lock".to_string(),
                )
            })?;
        Ok(match aref_to_transforms.get(aref_uuid) {
            Some(transform_set) => transform_set.clone(),
            None => HashSet::new(),
        })
    }

    /// Gets all atom references that a transform depends on.
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        let transform_to_arefs = self
            .transform_to_arefs
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire transform_to_arefs lock".to_string(),
                )
            })?;
        Ok(match transform_to_arefs.get(transform_id) {
            Some(aref_set) => aref_set.clone(),
            None => HashSet::new(),
        })
    }

    /// Gets the output atom reference for a transform.
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        let transform_outputs = self
            .transform_outputs
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire transform_outputs lock".to_string(),
                )
            })?;
        Ok(transform_outputs.get(transform_id).cloned())
    }

    /// Gets all transforms that should run when the specified field is updated.
    pub fn get_transforms_for_field(&self, schema_name: &str, field_name: &str) -> Result<HashSet<String>, SchemaError> {
        let key = format!("{}.{}", schema_name, field_name);
        let field_to_transforms = self
            .field_to_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire field_to_transforms lock".to_string(),
                )
            })?;
        Ok(field_to_transforms.get(&key).cloned().unwrap_or_default())
    }

    /// Register transform using unified database operations
    pub fn register_transform_with_db_ops(
        &self,
        registration: TransformRegistration,
        db_ops: &Arc<DbOperations>,
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

        // Validate the transform
        TransformExecutor::validate_transform(&transform)?;

        // Set transform output field
        let output_field = format!("{}.{}", schema_name, field_name);
        let inputs_len = input_arefs.len();
        transform.set_output(output_field.clone());

        // Store transform using unified operations
        db_ops.store_transform(&transform_id, &transform)?;

        // Update in-memory cache
        {
            let mut registered_transforms = self
                .registered_transforms
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire registered_transforms lock".to_string(),
                    )
                })?;
            registered_transforms.insert(transform_id.clone(), transform);
        }

        // Register the output atom reference
        {
            let mut transform_outputs = self
                .transform_outputs
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_outputs lock".to_string(),
                    )
                })?;
            transform_outputs.insert(transform_id.clone(), output_aref.clone());
        }

        // Register the input atom references
        {
            let mut transform_to_arefs = self
                .transform_to_arefs
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_to_arefs lock".to_string(),
                    )
                })?;
            let mut aref_set = HashSet::new();
            for aref_uuid in &input_arefs {
                aref_set.insert(aref_uuid.clone());
            }
            transform_to_arefs.insert(transform_id.clone(), aref_set);
        }

        // Store mapping of input names to refs
        {
            let mut transform_input_names = self
                .transform_input_names
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_input_names lock".to_string(),
                    )
                })?;
            let mut map = HashMap::new();
            for (aref_uuid, name) in input_arefs.iter().zip(input_names.iter()) {
                map.insert(aref_uuid.clone(), name.clone());
            }
            transform_input_names.insert(transform_id.clone(), map);
        }

        // Register the fields that trigger this transform
        {
            let mut transform_to_fields = self
                .transform_to_fields
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_to_fields lock".to_string(),
                    )
                })?;
            let mut field_to_transforms = self
                .field_to_transforms
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire field_to_transforms lock".to_string(),
                    )
                })?;

            let mut field_set = HashSet::new();
            for field_key in &trigger_fields {
                field_set.insert(field_key.clone());
                let set = field_to_transforms.entry(field_key.clone()).or_default();
                set.insert(transform_id.clone());
            }

            transform_to_fields.insert(transform_id.clone(), field_set);
        }

        // Update the reverse mapping (aref -> transforms)
        {
            let mut aref_to_transforms = self
                .aref_to_transforms
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire aref_to_transforms lock".to_string(),
                    )
                })?;

            for aref_uuid in input_arefs {
                let transform_set = aref_to_transforms.entry(aref_uuid).or_default();
                transform_set.insert(transform_id.clone());
            }
        }

        info!(
            "Registered transform {} output {} with {} input references using unified operations",
            transform_id,
            output_field,
            inputs_len
        );

        // Persist mappings using unified operations
        self.persist_mappings_unified(db_ops)?;
        Ok(())
    }

    /// Persist mappings using unified operations when available
    pub fn persist_mappings_unified(&self, db_ops: &Arc<DbOperations>) -> Result<(), SchemaError> {
        // Store aref_to_transforms mapping
        {
            let map = self.aref_to_transforms.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire aref_to_transforms lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize aref_to_transforms: {}", e))
            })?;
            db_ops.store_transform_mapping(AREF_TO_TRANSFORMS_KEY, &json)?;
        }

        // Store transform_to_arefs mapping
        {
            let map = self.transform_to_arefs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_to_arefs lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_to_arefs: {}", e))
            })?;
            db_ops.store_transform_mapping(TRANSFORM_TO_AREFS_KEY, &json)?;
        }

        // Store transform_input_names mapping
        {
            let map = self.transform_input_names.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_input_names lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_input_names: {}", e))
            })?;
            db_ops.store_transform_mapping(TRANSFORM_INPUT_NAMES_KEY, &json)?;
        }

        // Store field_to_transforms mapping
        {
            let map = self.field_to_transforms.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire field_to_transforms lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize field_to_transforms: {}", e))
            })?;
            db_ops.store_transform_mapping(FIELD_TO_TRANSFORMS_KEY, &json)?;
        }

        // Store transform_to_fields mapping
        {
            let map = self.transform_to_fields.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_to_fields lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_to_fields: {}", e))
            })?;
            db_ops.store_transform_mapping(TRANSFORM_TO_FIELDS_KEY, &json)?;
        }

        // Store transform_outputs mapping
        {
            let map = self.transform_outputs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_outputs lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_outputs: {}", e))
            })?;
            db_ops.store_transform_mapping(TRANSFORM_OUTPUTS_KEY, &json)?;
        }

        Ok(())
    }

    /// Load persisted mappings using unified operations
    #[allow(clippy::type_complexity)]
    fn load_persisted_mappings_unified(
        db_ops: &Arc<DbOperations>,
    ) -> Result<(
        HashMap<String, HashSet<String>>,
        HashMap<String, HashSet<String>>,
        HashMap<String, HashMap<String, String>>,
        HashMap<String, HashSet<String>>,
        HashMap<String, HashSet<String>>,
        HashMap<String, String>,
    ), SchemaError> {
        // Load aref_to_transforms
        let aref_to_transforms = if let Some(data) = db_ops.get_transform_mapping(AREF_TO_TRANSFORMS_KEY)? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Load transform_to_arefs
        let transform_to_arefs = if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_TO_AREFS_KEY)? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Load transform_input_names
        let transform_input_names = if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_INPUT_NAMES_KEY)? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Load field_to_transforms
        let field_to_transforms = if let Some(data) = db_ops.get_transform_mapping(FIELD_TO_TRANSFORMS_KEY)? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Load transform_to_fields
        let transform_to_fields = if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_TO_FIELDS_KEY)? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Load transform_outputs
        let transform_outputs = if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_OUTPUTS_KEY)? {
            serde_json::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok((
            aref_to_transforms,
            transform_to_arefs,
            transform_input_names,
            field_to_transforms,
            transform_to_fields,
            transform_outputs,
        ))
    }

}

impl TransformRunner for TransformManager {
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        TransformManager::execute_transform_now(self, transform_id)
    }

    fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        TransformManager::transform_exists(self, transform_id)
    }

    fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        TransformManager::get_transforms_for_field(self, schema_name, field_name)
    }
}
