use super::manager::TransformManager;
use crate::schema::types::{SchemaError, Transform, TransformRegistration};
use log::info;

impl TransformManager {
    /// Registers a transform and tracks its input and output atom references
    /// internally.  The provided `input_arefs` are used only for dependency
    /// tracking and are not persisted on the [`Transform`] itself.
    /// UNIFIED: Register transform using event-driven database operations
    /// This replaces both register_transform() and register_transform_event_driven()
    pub fn register_transform(
        &self,
        registration: TransformRegistration,
    ) -> Result<(), SchemaError> {
        // Delegate to event-driven implementation
        self.register_transform_event_driven(registration)
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
        let dependencies = transform
            .analyze_dependencies()
            .into_iter()
            .collect::<Vec<String>>();
        let trigger_fields = Vec::new();
        let inputs_len = dependencies.len();
        let output_field = format!("{}.{}", schema_name, field_name);
        let tid = transform_id.clone();
        self.register_transform(TransformRegistration {
            transform_id,
            transform,
            input_arefs: dependencies,
            input_names: Vec::new(),
            trigger_fields,
            output_aref,
            schema_name,
            field_name,
        })?;
        info!(
            "Registered transform {} output {} with {} input references",
            tid, output_field, inputs_len
        );
        Ok(())
    }

    /// Unregisters a transform using direct database operations.
    pub fn unregister_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        // Use direct database operations for consistency with other components
        let _existed = self.db_ops.delete_transform(transform_id)?;

        // Remove from in-memory cache
        let found = {
            let mut registered_transforms = self.registered_transforms.write().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
            })?;
            registered_transforms.remove(transform_id).is_some()
        };

        if found {
            // Remove from transform outputs
            {
                let mut transform_outputs = self.transform_outputs.write().map_err(|_| {
                    SchemaError::InvalidData("Failed to acquire transform_outputs lock".to_string())
                })?;
                transform_outputs.remove(transform_id);
            }

            // Remove field mappings
            {
                let mut transform_to_fields = self.transform_to_fields.write().map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_to_fields lock".to_string(),
                    )
                })?;
                let mut field_to_transforms = self.field_to_transforms.write().map_err(|_| {
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
                let mut transform_to_arefs = self.transform_to_arefs.write().map_err(|_| {
                    SchemaError::InvalidData(
                        "Failed to acquire transform_to_arefs lock".to_string(),
                    )
                })?;
                transform_to_arefs.remove(transform_id).unwrap_or_default()
            };

            // Remove input name mapping
            {
                let mut transform_input_names =
                    self.transform_input_names.write().map_err(|_| {
                        SchemaError::InvalidData(
                            "Failed to acquire transform_input_names lock".to_string(),
                        )
                    })?;
                transform_input_names.remove(transform_id);
            }

            // Update the reverse mapping (aref -> transforms)
            {
                let mut aref_to_transforms = self.aref_to_transforms.write().map_err(|_| {
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
            self.persist_mappings_direct()?;
        }

        Ok(found)
    }
}
