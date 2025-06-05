use super::manager::{
    AREF_TO_TRANSFORMS_KEY, TRANSFORM_TO_AREFS_KEY, TRANSFORM_INPUT_NAMES_KEY,
    FIELD_TO_TRANSFORMS_KEY, TRANSFORM_TO_FIELDS_KEY, TRANSFORM_OUTPUTS_KEY,
    TransformManager
};
use crate::db_operations::DbOperations;
use crate::schema::types::SchemaError;
use log::info;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

impl TransformManager {
    /// Persist mappings using event-driven operations only
    pub fn persist_mappings_direct(&self) -> Result<(), SchemaError> {
        // Store aref_to_transforms mapping
        {
            let map = self.aref_to_transforms.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire aref_to_transforms lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize aref_to_transforms: {}", e))
            })?;
            self.db_ops.store_transform_mapping(AREF_TO_TRANSFORMS_KEY, &json)?;
        }

        // Store transform_to_arefs mapping
        {
            let map = self.transform_to_arefs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_to_arefs lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_to_arefs: {}", e))
            })?;
            self.db_ops.store_transform_mapping(TRANSFORM_TO_AREFS_KEY, &json)?;
        }

        // Store transform_input_names mapping
        {
            let map = self.transform_input_names.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_input_names lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!(
                    "Failed to serialize transform_input_names: {}",
                    e
                ))
            })?;
            self.db_ops.store_transform_mapping(TRANSFORM_INPUT_NAMES_KEY, &json)?;
        }

        // Store field_to_transforms mapping
        {
            let map = self.field_to_transforms.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire field_to_transforms lock".to_string())
            })?;
            
            // DEBUG: Log what we're storing
            info!("🔍 DEBUG: Storing field_to_transforms mapping with {} entries:", map.len());
            for (field_key, transforms) in map.iter() {
                info!("  📋 Storing '{}' -> {:?}", field_key, transforms);
            }
            
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize field_to_transforms: {}", e))
            })?;
            self.db_ops.store_transform_mapping(FIELD_TO_TRANSFORMS_KEY, &json)?;
            info!("✅ DEBUG: Successfully stored field_to_transforms mapping to database");
        }

        // Store transform_to_fields mapping
        {
            let map = self.transform_to_fields.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_to_fields lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_to_fields: {}", e))
            })?;
            self.db_ops.store_transform_mapping(TRANSFORM_TO_FIELDS_KEY, &json)?;
        }

        // Store transform_outputs mapping
        {
            let map = self.transform_outputs.read().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire transform_outputs lock".to_string())
            })?;
            let json = serde_json::to_vec(&*map).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize transform_outputs: {}", e))
            })?;
            self.db_ops.store_transform_mapping(TRANSFORM_OUTPUTS_KEY, &json)?;
        }

        Ok(())
    }

    /// Load persisted mappings using direct database operations
    #[allow(clippy::type_complexity)]
    pub(super) fn load_persisted_mappings_direct(
        db_ops: &Arc<DbOperations>,
    ) -> Result<
        (
            HashMap<String, HashSet<String>>,
            HashMap<String, HashSet<String>>,
            HashMap<String, HashMap<String, String>>,
            HashMap<String, HashSet<String>>,
            HashMap<String, HashSet<String>>,
            HashMap<String, String>,
        ),
        SchemaError,
    > {
        // Load aref_to_transforms
        let aref_to_transforms =
            if let Some(data) = db_ops.get_transform_mapping(AREF_TO_TRANSFORMS_KEY)? {
                serde_json::from_slice(&data).unwrap_or_default()
            } else {
                HashMap::new()
            };

        // Load transform_to_arefs
        let transform_to_arefs =
            if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_TO_AREFS_KEY)? {
                serde_json::from_slice(&data).unwrap_or_default()
            } else {
                HashMap::new()
            };

        // Load transform_input_names
        let transform_input_names =
            if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_INPUT_NAMES_KEY)? {
                serde_json::from_slice(&data).unwrap_or_default()
            } else {
                HashMap::new()
            };

        // Load field_to_transforms
        let field_to_transforms =
            if let Some(data) = db_ops.get_transform_mapping(FIELD_TO_TRANSFORMS_KEY)? {
                let loaded_map: HashMap<String, HashSet<String>> = serde_json::from_slice(&data).unwrap_or_default();
                info!("🔍 DEBUG: Loaded field_to_transforms mapping from database with {} entries:", loaded_map.len());
                for (field_key, transforms) in &loaded_map {
                    info!("  📋 Loaded '{}' -> {:?}", field_key, transforms);
                }
                loaded_map
            } else {
                info!("🔍 DEBUG: No field_to_transforms mapping found in database - starting with empty map");
                HashMap::new()
            };

        // Load transform_to_fields
        let transform_to_fields =
            if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_TO_FIELDS_KEY)? {
                serde_json::from_slice(&data).unwrap_or_default()
            } else {
                HashMap::new()
            };

        // Load transform_outputs
        let transform_outputs =
            if let Some(data) = db_ops.get_transform_mapping(TRANSFORM_OUTPUTS_KEY)? {
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