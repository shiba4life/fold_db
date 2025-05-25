use crate::schema::types::{Transform, SchemaError};
use super::types::{CreateAtomFn, GetAtomFn, GetFieldFn, UpdateAtomRefFn, TransformRunner};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

pub(super) const AREF_TO_TRANSFORMS_KEY: &str = "map_aref_to_transforms";
pub(super) const TRANSFORM_TO_AREFS_KEY: &str = "map_transform_to_arefs";
pub(super) const TRANSFORM_INPUT_NAMES_KEY: &str = "map_transform_input_names";
pub(super) const FIELD_TO_TRANSFORMS_KEY: &str = "map_field_to_transforms";
pub(super) const TRANSFORM_TO_FIELDS_KEY: &str = "map_transform_to_fields";
pub(super) const TRANSFORM_OUTPUTS_KEY: &str = "map_transform_outputs";

pub struct TransformManager {
    /// Tree for storing transforms
    pub(super) transforms_tree: sled::Tree,
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
    /// Creates a new TransformManager instance
    pub fn new(
        transforms_tree: sled::Tree,
        get_atom_fn: GetAtomFn,
        create_atom_fn: CreateAtomFn,
        update_atom_ref_fn: UpdateAtomRefFn,
        get_field_fn: GetFieldFn,
    ) -> Self {
        // Load any persisted transforms
        let mut registered_transforms = HashMap::new();
        for (key, value) in transforms_tree.iter().flatten() {
            if let Ok(field_key) = String::from_utf8(key.to_vec()) {
                if field_key.starts_with("map_") {
                    continue;
                }
                if let Ok(transform) = serde_json::from_slice::<Transform>(&value) {
                    registered_transforms.insert(field_key, transform);
                }
            }
        }

        let aref_to_transforms = transforms_tree
            .get(AREF_TO_TRANSFORMS_KEY)
            .ok()
            .and_then(|v| v)
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();

        let transform_to_arefs = transforms_tree
            .get(TRANSFORM_TO_AREFS_KEY)
            .ok()
            .and_then(|v| v)
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();

        let transform_input_names = transforms_tree
            .get(TRANSFORM_INPUT_NAMES_KEY)
            .ok()
            .and_then(|v| v)
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();

        let field_to_transforms = transforms_tree
            .get(FIELD_TO_TRANSFORMS_KEY)
            .ok()
            .and_then(|v| v)
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();

        let transform_to_fields = transforms_tree
            .get(TRANSFORM_TO_FIELDS_KEY)
            .ok()
            .and_then(|v| v)
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();

        let transform_outputs = transforms_tree
            .get(TRANSFORM_OUTPUTS_KEY)
            .ok()
            .and_then(|v| v)
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();

        Self {
            transforms_tree,
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
        }
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
