use crate::atom::{Atom, AtomRef};
use crate::schema::types::{Transform, SchemaError};
use crate::transform::TransformExecutor;
use super::transform_orchestrator::TransformRunner;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Callback function type for getting an atom by its reference UUID
pub type GetAtomFn = Arc<dyn Fn(&str) -> Result<Atom, Box<dyn std::error::Error>> + Send + Sync>;

/// Callback function type for creating a new atom
pub type CreateAtomFn = Arc<dyn Fn(&str, String, Option<String>, JsonValue, Option<crate::atom::AtomStatus>) -> Result<Atom, Box<dyn std::error::Error>> + Send + Sync>;

/// Callback function type for updating an atom reference
pub type UpdateAtomRefFn = Arc<dyn Fn(&str, String, String) -> Result<AtomRef, Box<dyn std::error::Error>> + Send + Sync>;

pub struct TransformManager {
    /// Tree for storing transforms
    transforms_tree: sled::Tree,
    /// In-memory cache of registered transforms
    registered_transforms: RwLock<HashMap<String, Transform>>,
    /// Maps atom reference UUIDs to the transforms that depend on them
    aref_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their dependent atom reference UUIDs
    transform_to_arefs: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps schema.field keys to transforms triggered by them
    field_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to the fields that trigger them
    transform_to_fields: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their output atom reference UUIDs
    transform_outputs: RwLock<HashMap<String, String>>,
    /// Callback for getting an atom by its reference UUID
    get_atom_fn: GetAtomFn,
    /// Callback for creating a new atom
    create_atom_fn: CreateAtomFn,
    /// Callback for updating an atom reference
    update_atom_ref_fn: UpdateAtomRefFn,
}

impl TransformManager {
    /// Creates a new TransformManager instance
    pub fn new(
        transforms_tree: sled::Tree,
        get_atom_fn: GetAtomFn,
        create_atom_fn: CreateAtomFn,
        update_atom_ref_fn: UpdateAtomRefFn,
    ) -> Self {
        // Load any persisted transforms
        let mut registered_transforms = HashMap::new();
        for (key, value) in transforms_tree.iter().flatten() {
            if let (Ok(field_key), Ok(transform)) = (
                String::from_utf8(key.to_vec()),
                serde_json::from_slice::<Transform>(&value),
            ) {
                registered_transforms.insert(field_key, transform);
            }
        }

        Self {
            transforms_tree,
            registered_transforms: RwLock::new(registered_transforms),
            aref_to_transforms: RwLock::new(HashMap::new()),
            transform_to_arefs: RwLock::new(HashMap::new()),
            field_to_transforms: RwLock::new(HashMap::new()),
            transform_to_fields: RwLock::new(HashMap::new()),
            transform_outputs: RwLock::new(HashMap::new()),
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
        }
    }

    /// Registers a transform with its input and output atom references.
    pub fn register_transform(
        &self,
        transform_id: String,
        mut transform: Transform,
        input_arefs: Vec<String>,
        trigger_fields: Vec<String>,
        output_aref: String,
    ) -> Result<(), SchemaError> {
        // Validate the transform
        TransformExecutor::validate_transform(&transform)?;
        
        // Set the transform's input dependencies and output reference
        transform.set_input_dependencies(input_arefs.clone());
        transform.set_output_reference(output_aref.clone());
        
        // Store the transform
        let transform_json = serde_json::to_vec(&transform)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize transform: {}", e)))?;
        
        self.transforms_tree
            .insert(transform_id.as_bytes(), transform_json)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store transform: {}", e)))?;
        self.transforms_tree
            .flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush transform tree: {}", e)))?;
        
        // Update in-memory cache
        {
            let mut registered_transforms = self.registered_transforms.write().unwrap();
            registered_transforms.insert(transform_id.clone(), transform);
        }
        
        // Register the output atom reference
        {
            let mut transform_outputs = self.transform_outputs.write().unwrap();
            transform_outputs.insert(transform_id.clone(), output_aref);
        }
        
        // Register the input atom references
        {
            let mut transform_to_arefs = self.transform_to_arefs.write().unwrap();
            let mut aref_set = HashSet::new();

            for aref_uuid in &input_arefs {
                aref_set.insert(aref_uuid.clone());
            }

            transform_to_arefs.insert(transform_id.clone(), aref_set);
        }

        // Register the fields that trigger this transform
        {
            let mut transform_to_fields = self.transform_to_fields.write().unwrap();
            let mut field_to_transforms = self.field_to_transforms.write().unwrap();

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
            let mut aref_to_transforms = self.aref_to_transforms.write().unwrap();
            
            for aref_uuid in input_arefs {
                let transform_set = aref_to_transforms
                    .entry(aref_uuid)
                    .or_default();
                transform_set.insert(transform_id.clone());
            }
        }
        
        Ok(())
    }

    /// Registers a transform with automatic input dependency detection.
    pub fn register_transform_auto(
        &self,
        transform_id: String,
        transform: Transform,
        output_aref: String,
    ) -> Result<(), SchemaError> {
        let dependencies = transform.analyze_dependencies().into_iter().collect::<Vec<String>>();
        let trigger_fields = Vec::new();
        self.register_transform(
            transform_id,
            transform,
            dependencies,
            trigger_fields,
            output_aref,
        )
    }

    /// Unregisters a transform.
    pub fn unregister_transform(&self, transform_id: &str) -> bool {
        // Remove from transforms tree and cache
        let found = if self.transforms_tree.remove(transform_id.as_bytes()).is_ok() {
            let mut registered_transforms = self.registered_transforms.write().unwrap();
            registered_transforms.remove(transform_id).is_some()
        } else {
            false
        };
        
        if found {
            // Remove from transform outputs
            {
                let mut transform_outputs = self.transform_outputs.write().unwrap();
                transform_outputs.remove(transform_id);
            }

            // Remove field mappings
            {
                let mut transform_to_fields = self.transform_to_fields.write().unwrap();
                let mut field_to_transforms = self.field_to_transforms.write().unwrap();

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
                let mut transform_to_arefs = self.transform_to_arefs.write().unwrap();
                transform_to_arefs.remove(transform_id).unwrap_or_default()
            };
            
            // Update the reverse mapping (aref -> transforms)
            {
                let mut aref_to_transforms = self.aref_to_transforms.write().unwrap();
                
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
        
        found
    }

    /// Handles an atom reference update by executing all dependent transforms.
    pub fn handle_atom_ref_update(&self, aref_uuid: &str) -> Vec<Result<JsonValue, SchemaError>> {
        let mut results = Vec::new();
        
        // Find all transforms that depend on this atom reference
        let transform_ids = {
            let aref_to_transforms = self.aref_to_transforms.read().unwrap();
            
            match aref_to_transforms.get(aref_uuid) {
                Some(transform_set) => transform_set.clone(),
                None => return results, // No dependent transforms
            }
        };
        
        // Execute each transform
        for transform_id in transform_ids {
            let result = self.execute_transform(&transform_id);
            results.push(result);
        }
        
        results
    }

    /// Executes a transform and updates its output atom reference.
    fn execute_transform(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        // Get the transform
        let transform = {
            let registered_transforms = self.registered_transforms.read().unwrap();
            match registered_transforms.get(transform_id) {
                Some(transform) => transform.clone(),
                None => return Err(SchemaError::InvalidField(format!("Transform not found: {}", transform_id))),
            }
        };
        
        // Create an input provider function that gets values from atom references
        let get_atom_fn = &self.get_atom_fn;
        let transform_to_arefs = self.transform_to_arefs.read().unwrap();
        let input_arefs = transform_to_arefs.get(transform_id).cloned().unwrap_or_default();
        
        let input_provider = move |input_name: &str| -> Result<JsonValue, Box<dyn std::error::Error>> {
            if input_arefs.contains(input_name) {
                let atom = (get_atom_fn)(input_name)?;
                Ok(atom.content().clone())
            } else {
                Err(format!("Input not found: {}", input_name).into())
            }
        };
        
        // Execute the transform with the input provider
        let result = TransformExecutor::execute_transform_with_provider(&transform, input_provider)?;
        
        // Update the output atom reference
        let output_aref = {
            let transform_outputs = self.transform_outputs.read().unwrap();
            
            match transform_outputs.get(transform_id) {
                Some(aref_uuid) => aref_uuid.clone(),
                None => return Err(SchemaError::InvalidField(format!("Transform output not found: {}", transform_id))),
            }
        };
        
        // Create a new atom with the transform result
        let atom = match (self.create_atom_fn)(
            "transform_result",
            "transform_system".to_string(),
            None,
            result.clone(),
            None,
        ) {
            Ok(atom) => atom,
            Err(e) => return Err(SchemaError::InvalidField(format!("Failed to create atom: {}", e))),
        };
        
        // Update the output atom reference
        match (self.update_atom_ref_fn)(
            &output_aref,
            atom.uuid().to_string(),
            "transform_system".to_string(),
        ) {
            Ok(_) => {},
            Err(e) => return Err(SchemaError::InvalidField(format!("Failed to update atom reference: {}", e))),
        }
        
        Ok(result)
    }

    /// Executes a registered transform immediately and updates its output.
    pub fn execute_transform_now(
        &self,
        transform_id: &str,
    ) -> Result<JsonValue, SchemaError> {
        self.execute_transform(transform_id)
    }

    /// Returns true if a transform with the given id is registered.
    pub fn transform_exists(&self, transform_id: &str) -> bool {
        let registered_transforms = self.registered_transforms.read().unwrap();
        registered_transforms.contains_key(transform_id)
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> HashMap<String, Transform> {
        let registered_transforms = self.registered_transforms.read().unwrap();
        registered_transforms.clone()
    }

    /// Gets all transforms that depend on the specified atom reference.
    pub fn get_dependent_transforms(&self, aref_uuid: &str) -> HashSet<String> {
        let aref_to_transforms = self.aref_to_transforms.read().unwrap();
        match aref_to_transforms.get(aref_uuid) {
            Some(transform_set) => transform_set.clone(),
            None => HashSet::new(),
        }
    }

    /// Gets all atom references that a transform depends on.
    pub fn get_transform_inputs(&self, transform_id: &str) -> HashSet<String> {
        let transform_to_arefs = self.transform_to_arefs.read().unwrap();
        match transform_to_arefs.get(transform_id) {
            Some(aref_set) => aref_set.clone(),
            None => HashSet::new(),
        }
    }

    /// Gets the output atom reference for a transform.
    pub fn get_transform_output(&self, transform_id: &str) -> Option<String> {
        let transform_outputs = self.transform_outputs.read().unwrap();
        transform_outputs.get(transform_id).cloned()
    }

    /// Gets all transforms that should run when the specified field is updated.
    pub fn get_transforms_for_field(&self, schema_name: &str, field_name: &str) -> HashSet<String> {
        let key = format!("{}.{}", schema_name, field_name);
        let field_to_transforms = self.field_to_transforms.read().unwrap();
        field_to_transforms.get(&key).cloned().unwrap_or_default()
    }

    /// Execute transforms for a specific schema field
pub fn execute_field_transforms(
        &self,
        schema_name: &str,
        field_name: &str,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        let transform_id = format!("{}.{}", schema_name, field_name);
        
        let registered_transforms = self.registered_transforms.read().unwrap();
        if let Some(transform) = registered_transforms.get(&transform_id) {
            let mut input_values = HashMap::new();
            input_values.insert("field_value".to_string(), value.clone());
            input_values.insert("field_key".to_string(), serde_json::Value::String(transform_id.clone()));

            if let Err(e) = TransformExecutor::execute_transform(transform, input_values) {
                eprintln!("Failed to execute transform for field {}: {}", transform_id, e);
            }
        }

        Ok(())
    }
}

impl TransformRunner for TransformManager {
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        TransformManager::execute_transform_now(self, transform_id)
    }

    fn transform_exists(&self, transform_id: &str) -> bool {
        TransformManager::transform_exists(self, transform_id)
    }

    fn get_transforms_for_field(&self, schema_name: &str, field_name: &str) -> HashSet<String> {
        TransformManager::get_transforms_for_field(self, schema_name, field_name)
    }
}
