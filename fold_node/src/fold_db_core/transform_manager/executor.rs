use super::manager::TransformManager;
use crate::schema::types::{SchemaError, Transform};
use crate::transform::TransformExecutor;
use log::{error, info};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

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
        info!("🚀 EXECUTING TRANSFORM: {}", transform_id);

        let transform = self.fetch_registered_transform(transform_id)?;
        info!(
            "📋 Transform fetched: {} with logic: {}",
            transform_id, transform.logic
        );

        let input_values = self.gather_input_values(transform_id, &transform)?;
        info!(
            "📥 Input values gathered for {}: {:?}",
            transform_id, input_values
        );

        let result = match TransformExecutor::execute_transform(&transform, input_values.clone()) {
            Ok(val) => {
                info!(
                    "✅ Transform {} SUCCEEDED with result: {:?}",
                    transform_id, val
                );
                val
            }
            Err(e) => {
                // If the transform fails due to missing inputs, silently resolve with null
                // This allows the system to continue functioning even when dependencies are not ready
                info!(
                    "❌ Transform {} FAILED (resolving silently): {}",
                    transform_id, e
                );
                info!(
                    "🔍 Failed transform details - inputs: {:?}, logic: {}",
                    input_values, transform.logic
                );
                serde_json::Value::Null
            }
        };

        info!(
            "💾 Persisting transform result for {}: {:?}",
            transform_id, result
        );
        self.persist_transform_result(transform_id, result.clone())?;
        info!("✅ Transform {} execution COMPLETE", transform_id);
        Ok(result)
    }

    /// Fetch a registered transform by id.
    fn fetch_registered_transform(&self, transform_id: &str) -> Result<Transform, SchemaError> {
        let registered_transforms = self.registered_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
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
        info!("📥 GATHERING INPUT VALUES for transform: {}", transform_id);

        let get_atom_fn = &self.get_atom_fn;
        let transform_to_arefs = self.transform_to_arefs.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire transform_to_arefs lock".to_string())
        })?;
        let input_arefs = transform_to_arefs
            .get(transform_id)
            .cloned()
            .unwrap_or_default();
        info!("🔗 Input arefs for {}: {:?}", transform_id, input_arefs);

        let name_map = {
            let transform_input_names = self.transform_input_names.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_input_names lock".to_string())
            })?;
            transform_input_names
                .get(transform_id)
                .cloned()
                .unwrap_or_default()
        };
        info!("📋 Name map for {}: {:?}", transform_id, name_map);

        let mut input_values = HashMap::new();

        let inputs = transform.get_inputs();
        info!("📝 Transform inputs for {}: {:?}", transform_id, inputs);

        if !inputs.is_empty() {
            for input in inputs {
                info!("🔍 Processing input: {}", input);
                if let Some((schema, field)) = input.split_once('.') {
                    info!("🎯 Fetching field value for {}.{}", schema, field);
                    match (self.get_field_fn)(schema, field) {
                        Ok(val) => {
                            info!("✅ Got field value for {}: {:?}", input, val);
                            input_values.insert(input.clone(), val);
                        }
                        Err(e) => {
                            info!("❌ Failed to get field value for {}: {:?}", input, e);
                            return Err(e);
                        }
                    }
                } else {
                    info!("⚠️  Invalid input format (no dot): {}", input);
                }
            }
        } else {
            info!("📭 No inputs defined for transform: {}", transform_id);
        }

        info!(
            "🔗 Processing {} arefs for transform: {}",
            input_arefs.len(),
            transform_id
        );
        for aref in &input_arefs {
            info!("🔍 Processing aref: {}", aref);
            match (get_atom_fn)(aref) {
                Ok(atom) => {
                    let key = name_map.get(aref).cloned().unwrap_or_else(|| aref.clone());
                    let content = atom.content().clone();
                    info!(
                        "✅ Got atom content for aref {}: key={}, content={:?}",
                        aref, key, content
                    );
                    input_values.insert(key, content);
                }
                Err(e) => {
                    // Silently skip missing AtomRefs - this allows transforms to execute
                    // even when some input dependencies are not yet available
                    info!(
                        "⚠️  Skipping missing AtomRef '{}' for transform {}: {}",
                        aref, transform_id, e
                    );
                }
            }
        }

        info!(
            "✅ FINAL input values for {}: {:?}",
            transform_id, input_values
        );
        Ok(input_values)
    }

    /// Persist the result of a transform to its output atom reference.
    fn persist_transform_result(
        &self,
        transform_id: &str,
        result: JsonValue,
    ) -> Result<(), SchemaError> {
        info!(
            "persist_transform_result called for transform: {}",
            transform_id
        );

        let output_aref = {
            let transform_outputs = self.transform_outputs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_outputs lock".to_string())
            })?;

            info!(
                "Available transform outputs: {:?}",
                transform_outputs.keys().collect::<Vec<_>>()
            );

            match transform_outputs.get(transform_id) {
                Some(aref_uuid) => {
                    info!("Found output ARef for {}: {}", transform_id, aref_uuid);
                    aref_uuid.clone()
                }
                None => {
                    error!("Transform output not found for: {}", transform_id);
                    return Err(SchemaError::InvalidField(format!(
                        "Transform output not found: {}",
                        transform_id
                    )));
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
            }
            Err(e) => {
                error!("Failed to create atom: {}", e);
                return Err(SchemaError::InvalidField(format!(
                    "Failed to create atom: {}",
                    e
                )));
            }
        };

        info!(
            "Updating atom reference {} with atom {}",
            output_aref,
            atom.uuid()
        );
        match (self.update_atom_ref_fn)(
            &output_aref,
            atom.uuid().to_string(),
            "transform_system".to_string(),
        ) {
            Ok(_) => {
                info!(
                    "Successfully updated atom reference {} with result",
                    output_aref
                );
            }
            Err(e) => {
                error!("Failed to update atom reference {}: {}", output_aref, e);
                return Err(SchemaError::InvalidField(format!(
                    "Failed to update atom reference: {}",
                    e
                )));
            }
        }

        info!(
            "Transform result persisted successfully for {}",
            transform_id
        );
        Ok(())
    }

    /// Executes a registered transform immediately and updates its output.
    pub fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        info!(
            "🎯 EXECUTE_TRANSFORM_NOW START - transform_id: {}",
            transform_id
        );

        // Check if transform is registered
        let transform_exists = self
            .registered_transforms
            .read()
            .map_err(|_| {
                SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
            })?
            .contains_key(transform_id);

        if !transform_exists {
            error!("❌ Transform {} is not registered!", transform_id);
            return Err(SchemaError::InvalidData(format!(
                "Transform {} not found",
                transform_id
            )));
        }

        info!(
            "✅ Transform {} is registered, proceeding with execution",
            transform_id
        );
        let result = self.execute_transform(transform_id);

        match &result {
            Ok(val) => info!(
                "✅ EXECUTE_TRANSFORM_NOW SUCCESS - transform: {}, result: {:?}",
                transform_id, val
            ),
            Err(e) => error!(
                "❌ EXECUTE_TRANSFORM_NOW FAILED - transform: {}, error: {}",
                transform_id, e
            ),
        }

        info!(
            "🏁 EXECUTE_TRANSFORM_NOW COMPLETE - transform: {}",
            transform_id
        );
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

        let registered_transforms = self.registered_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
        })?;
        if let Some(transform) = registered_transforms.get(&transform_id) {
            let mut input_values = HashMap::new();
            input_values.insert("field_value".to_string(), value.clone());
            input_values.insert(
                "field_key".to_string(),
                serde_json::Value::String(transform_id.clone()),
            );

            if let Err(e) = TransformExecutor::execute_transform(transform, input_values) {
                error!(
                    "Failed to execute transform for field {}: {}",
                    transform_id, e
                );
            }
        }

        Ok(())
    }
}
