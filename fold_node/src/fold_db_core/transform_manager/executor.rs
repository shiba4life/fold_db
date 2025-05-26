use crate::schema::types::{Transform, SchemaError};
use crate::transform::TransformExecutor;
use super::manager::TransformManager;
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use log::{info, error};

impl TransformManager {
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
                // If the transform fails due to missing inputs, silently resolve with null
                // This allows the system to continue functioning even when dependencies are not ready
                info!("Transform {} failed (resolving silently): {}", transform_id, e);
                serde_json::Value::Null
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

        let name_map = {
            let transform_input_names = self
                .transform_input_names
                .read()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_input_names lock".to_string(),
                    )
                })?;
            transform_input_names
                .get(transform_id)
                .cloned()
                .unwrap_or_default()
        };

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
            match (get_atom_fn)(aref) {
                Ok(atom) => {
                    let key = name_map
                        .get(aref)
                        .cloned()
                        .unwrap_or_else(|| aref.clone());
                    input_values.insert(key, atom.content().clone());
                }
                Err(e) => {
                    // Silently skip missing AtomRefs - this allows transforms to execute
                    // even when some input dependencies are not yet available
                    info!("Skipping missing AtomRef '{}' for transform {}: {}", aref, transform_id, e);
                }
            }
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
        info!("persist_transform_result called for transform: {}", transform_id);
        
        let output_aref = {
            let transform_outputs = self
                .transform_outputs
                .read()
                .map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_outputs lock".to_string(),
                    )
                })?;

            info!("Available transform outputs: {:?}", transform_outputs.keys().collect::<Vec<_>>());
            
            match transform_outputs.get(transform_id) {
                Some(aref_uuid) => {
                    info!("Found output ARef for {}: {}", transform_id, aref_uuid);
                    aref_uuid.clone()
                },
                None => {
                    error!("Transform output not found for: {}", transform_id);
                    return Err(SchemaError::InvalidField(format!(
                        "Transform output not found: {}",
                        transform_id
                    )))
                }
            }
        };

        info!("Creating atom for transform result: {:?}", result);
        let atom = match (self.create_atom_fn)(
            "transform_result",
            "transform_system".to_string(),
            None,
            result.clone(),
            None,
        ) {
            Ok(atom) => {
                info!("Created atom with UUID: {}", atom.uuid());
                atom
            },
            Err(e) => {
                error!("Failed to create atom: {}", e);
                return Err(SchemaError::InvalidField(format!("Failed to create atom: {}", e)))
            }
        };

        info!("Updating atom reference {} with atom {}", output_aref, atom.uuid());
        match (self.update_atom_ref_fn)(
            &output_aref,
            atom.uuid().to_string(),
            "transform_system".to_string(),
        ) {
            Ok(_) => {
                info!("Successfully updated atom reference {} with result", output_aref);
            }
            Err(e) => {
                error!("Failed to update atom reference {}: {}", output_aref, e);
                return Err(SchemaError::InvalidField(format!(
                    "Failed to update atom reference: {}",
                    e
                )))
            }
        }

        info!("Transform result persisted successfully for {}", transform_id);
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
