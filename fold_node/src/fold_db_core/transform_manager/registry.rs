use crate::schema::types::{Transform, SchemaError, TransformRegistration};
use crate::transform::TransformExecutor;
use super::manager::{
    AREF_TO_TRANSFORMS_KEY,
    TRANSFORM_TO_AREFS_KEY,
    TRANSFORM_INPUT_NAMES_KEY,
    FIELD_TO_TRANSFORMS_KEY,
    TRANSFORM_TO_FIELDS_KEY,
    TRANSFORM_OUTPUTS_KEY,
    TransformManager,
};
use std::collections::{HashMap, HashSet};
use log::info;

impl TransformManager {
    /// Registers a transform and tracks its input and output atom references
    /// internally.  The provided `input_arefs` are used only for dependency
    /// tracking and are not persisted on the [`Transform`] itself.
    pub fn register_transform(&self, registration: TransformRegistration) -> Result<(), SchemaError> {
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
            "Registered transform {} output {} with {} input references",
            transform_id,
            output_field,
            inputs_len
        );
        self.persist_mappings()?;
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
                input_names: Vec::new(),
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

            // Remove input name mapping
            {
                let mut transform_input_names = self
                    .transform_input_names
                    .write()
                    .map_err(|_| {
                        SchemaError::InvalidData(
                            "Failed to acquire transform_input_names lock".to_string(),
                        )
                    })?;
                transform_input_names.remove(transform_id);
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

        if found {
            self.persist_mappings()?;
        }

        Ok(found)
    }
    fn persist_mappings(&self) -> Result<(), SchemaError> {
        {
            let map = self.aref_to_transforms.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire aref_to_transforms lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize aref_to_transforms: {}", e))
            })?;
            self.transforms_tree.insert(AREF_TO_TRANSFORMS_KEY.as_bytes(), json)?;
        }

        {
            let map = self.transform_to_arefs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_to_arefs lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_to_arefs: {}", e))
            })?;
            self.transforms_tree.insert(TRANSFORM_TO_AREFS_KEY.as_bytes(), json)?;
        }

        {
            let map = self.transform_input_names.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_input_names lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_input_names: {}", e))
            })?;
            self.transforms_tree.insert(TRANSFORM_INPUT_NAMES_KEY.as_bytes(), json)?;
        }

        {
            let map = self.field_to_transforms.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire field_to_transforms lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize field_to_transforms: {}", e))
            })?;
            self.transforms_tree.insert(FIELD_TO_TRANSFORMS_KEY.as_bytes(), json)?;
        }

        {
            let map = self.transform_to_fields.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_to_fields lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_to_fields: {}", e))
            })?;
            self.transforms_tree.insert(TRANSFORM_TO_FIELDS_KEY.as_bytes(), json)?;
        }

        {
            let map = self.transform_outputs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_outputs lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_outputs: {}", e))
            })?;
            self.transforms_tree.insert(TRANSFORM_OUTPUTS_KEY.as_bytes(), json)?;
        }

        self.transforms_tree.flush()?;
        Ok(())
    }

}
