use crate::schema::types::{Transform, SchemaError, TransformRegistration};
use crate::transform::TransformExecutor;
use super::types::{CreateAtomFn, GetAtomFn, GetFieldFn, UpdateAtomRefFn, TransformRunner};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use log::{error, info};

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
    /// Callback for retrieving field values
    get_field_fn: GetFieldFn,
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
            get_field_fn,
        }
    }

    /// Registers a transform and tracks its input and output atom references
    /// internally.  The provided `input_arefs` are used only for dependency
    /// tracking and are not persisted on the [`Transform`] itself.
    pub fn register_transform(&self, registration: TransformRegistration) -> Result<(), SchemaError> {
        let TransformRegistration {
            transform_id,
            mut transform,
            input_arefs,
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
            transform_outputs.insert(transform_id.clone(), output_aref);
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
            "Registered transform {} output {} with {} input references",
            transform_id,
            output_field,
            inputs_len
        );

        Ok(())
    }

    /// Registers a transform with automatic input dependency detection.
    pub fn register_transform_auto(
        &self,
        transform_id: String,
        transform: Transform,
        output_aref: String,
        schema_name: String,
        field_name: String,
    ) -> Result<(), SchemaError> {
        let dependencies = transform.analyze_dependencies().into_iter().collect::<Vec<String>>();
        let trigger_fields = Vec::new();
        let inputs_len = dependencies.len();
        let output_field = format!("{}.{}", schema_name, field_name);
        let tid = transform_id.clone();
        self.register_transform(
            TransformRegistration {
                transform_id,
                transform,
                input_arefs: dependencies,
                trigger_fields,
                output_aref,
                schema_name,
                field_name,
            }
        )?;
        info!(
            "Registered transform {} output {} with {} input references",
            tid,
            output_field,
            inputs_len
        );
        Ok(())
    }

    /// Unregisters a transform.
    pub fn unregister_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        // Remove from transforms tree and cache
        let found = if self.transforms_tree.remove(transform_id.as_bytes()).is_ok() {
            let mut registered_transforms = self
                .registered_transforms
                .write()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire registered_transforms lock".to_string(),
                    )
                })?;
            registered_transforms.remove(transform_id).is_some()
        } else {
            false
        };
        
        if found {
            // Remove from transform outputs
            {
                let mut transform_outputs = self
                    .transform_outputs
                    .write()
                    .map_err(|_| {
                        SchemaError::InvalidData(
                            "Failed to acquire transform_outputs lock".to_string(),
                        )
                    })?;
                transform_outputs.remove(transform_id);
            }

            // Remove field mappings
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
                let mut transform_to_arefs = self
                    .transform_to_arefs
                    .write()
                    .map_err(|_| {
                        SchemaError::InvalidData(
                            "Failed to acquire transform_to_arefs lock".to_string(),
                        )
                    })?;
                transform_to_arefs.remove(transform_id).unwrap_or_default()
            };
            
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

    /// Handles an atom reference update by executing all dependent transforms.
    pub fn handle_atom_ref_update(&self, aref_uuid: &str) -> Vec<Result<JsonValue, SchemaError>> {
        let mut results = Vec::new();
        
        // Find all transforms that depend on this atom reference
        let transform_ids = match self.aref_to_transforms.read() {
            Ok(map) => match map.get(aref_uuid) {
                Some(set) => set.clone(),
                None => return results,
            },
            Err(_) => {
                results.push(Err(SchemaError::InvalidData(
                    "Failed to acquire aref_to_transforms lock".to_string(),
                )));
                return results;
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
        info!("Executing transform {}", transform_id);

        let transform = self.fetch_registered_transform(transform_id)?;
        let input_values = self.gather_input_values(transform_id, &transform)?;

        let result = match TransformExecutor::execute_transform(&transform, input_values) {
            Ok(val) => {
                info!("Transform {} produced result: {:?}", transform_id, val);
                val
            }
            Err(e) => {
                error!("Transform {} failed: {}", transform_id, e);
                return Err(e);
            }
        };

        self.persist_transform_result(transform_id, result.clone())?;
        Ok(result)
    }

    /// Fetch a registered transform by id.
    fn fetch_registered_transform(&self, transform_id: &str) -> Result<Transform, SchemaError> {
        let registered_transforms = self
            .registered_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire registered_transforms lock".to_string(),
                )
            })?;
        match registered_transforms.get(transform_id) {
            Some(transform) => Ok(transform.clone()),
            None => Err(SchemaError::InvalidField(format!(
                "Transform not found: {}",
                transform_id
            ))),
        }
    }

    /// Gather input values for a transform.
    fn gather_input_values(
        &self,
        transform_id: &str,
        transform: &Transform,
    ) -> Result<HashMap<String, JsonValue>, SchemaError> {
        let get_atom_fn = &self.get_atom_fn;
        let transform_to_arefs = self
            .transform_to_arefs
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire transform_to_arefs lock".to_string(),
                )
            })?;
        let input_arefs = transform_to_arefs
            .get(transform_id)
            .cloned()
            .unwrap_or_default();

        let mut input_values = HashMap::new();

        let inputs = transform.get_inputs();
        if !inputs.is_empty() {
            for input in inputs {
                if let Some((schema, field)) = input.split_once('.') {
                    let val = (self.get_field_fn)(schema, field)?;
                    input_values.insert(input.clone(), val);
                }
            }
        }

        for aref in &input_arefs {
            let atom = (get_atom_fn)(aref).map_err(|e| {
                SchemaError::InvalidField(format!("Failed to get input '{}': {}", aref, e))
            })?;
            input_values.insert(aref.clone(), atom.content().clone());
        }

        info!("Input values for {}: {:?}", transform_id, input_values);
        Ok(input_values)
    }

    /// Persist the result of a transform to its output atom reference.
    fn persist_transform_result(
        &self,
        transform_id: &str,
        result: JsonValue,
    ) -> Result<(), SchemaError> {
        let output_aref = {
            let transform_outputs = self
                .transform_outputs
                .read()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_outputs lock".to_string(),
                    )
                })?;

            match transform_outputs.get(transform_id) {
                Some(aref_uuid) => aref_uuid.clone(),
                None => {
                    return Err(SchemaError::InvalidField(format!(
                        "Transform output not found: {}",
                        transform_id
                    )))
                }
            }
        };

        let atom = match (self.create_atom_fn)(
            "transform_result",
            "transform_system".to_string(),
            None,
            result.clone(),
            None,
        ) {
            Ok(atom) => atom,
            Err(e) => {
                return Err(SchemaError::InvalidField(format!("Failed to create atom: {}", e)))
            }
        };

        match (self.update_atom_ref_fn)(
            &output_aref,
            atom.uuid().to_string(),
            "transform_system".to_string(),
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(SchemaError::InvalidField(format!(
                    "Failed to update atom reference: {}",
                    e
                )))
            }
        }

        Ok(())
    }

    /// Executes a registered transform immediately and updates its output.
    pub fn execute_transform_now(
        &self,
        transform_id: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!("execute_transform_now called for {}", transform_id);
        let result = self.execute_transform(transform_id);
        match &result {
            Ok(val) => info!("Transform {} finished with result: {:?}", transform_id, val),
            Err(e) => error!("Transform {} failed: {}", transform_id, e),
        }
        result
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

    /// Execute transforms for a specific schema field
pub fn execute_field_transforms(
        &self,
        schema_name: &str,
        field_name: &str,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        let transform_id = format!("{}.{}", schema_name, field_name);
        
        let registered_transforms = self
            .registered_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData(
                    "Failed to acquire registered_transforms lock".to_string(),
                )
            })?;
        if let Some(transform) = registered_transforms.get(&transform_id) {
            let mut input_values = HashMap::new();
            input_values.insert("field_value".to_string(), value.clone());
            input_values.insert("field_key".to_string(), serde_json::Value::String(transform_id.clone()));

            if let Err(e) = TransformExecutor::execute_transform(transform, input_values) {
                error!("Failed to execute transform for field {}: {}", transform_id, e);
            }
        }

        Ok(())
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
