use super::manager::{
    AREF_TO_TRANSFORMS_KEY, TRANSFORM_TO_AREFS_KEY, TRANSFORM_INPUT_NAMES_KEY,
    FIELD_TO_TRANSFORMS_KEY, TRANSFORM_TO_FIELDS_KEY, TRANSFORM_OUTPUTS_KEY,
    TransformManager
};
use super::utils::*;
use crate::db_operations::DbOperations;
use crate::schema::types::SchemaError;
use log::info;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

impl TransformManager {
    /// Persist mappings using event-driven operations only
    pub fn persist_mappings_direct(&self) -> Result<(), SchemaError> {
        // Store aref_to_transforms mapping
        SerializationHelper::store_mapping(
            &self.db_ops,
            &self.aref_to_transforms,
            AREF_TO_TRANSFORMS_KEY,
            "aref_to_transforms"
        )?;

        // Store transform_to_arefs mapping
        SerializationHelper::store_mapping(
            &self.db_ops,
            &self.transform_to_arefs,
            TRANSFORM_TO_AREFS_KEY,
            "transform_to_arefs"
        )?;

        // Store transform_input_names mapping
        SerializationHelper::store_mapping(
            &self.db_ops,
            &self.transform_input_names,
            TRANSFORM_INPUT_NAMES_KEY,
            "transform_input_names"
        )?;

        // Store field_to_transforms mapping (with debug logging)
        SerializationHelper::store_mapping_with_debug(
            &self.db_ops,
            &self.field_to_transforms,
            FIELD_TO_TRANSFORMS_KEY,
            "field_to_transforms"
        )?;

        // Store transform_to_fields mapping
        SerializationHelper::store_mapping(
            &self.db_ops,
            &self.transform_to_fields,
            TRANSFORM_TO_FIELDS_KEY,
            "transform_to_fields"
        )?;

        // Store transform_outputs mapping
        SerializationHelper::store_mapping(
            &self.db_ops,
            &self.transform_outputs,
            TRANSFORM_OUTPUTS_KEY,
            "transform_outputs"
        )?;

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
        // Load aref_to_transforms using unified helper
        let aref_to_transforms = SerializationHelper::load_mapping_or_default(
            db_ops,
            AREF_TO_TRANSFORMS_KEY,
            "aref_to_transforms"
        )?;

        // Load transform_to_arefs using unified helper
        let transform_to_arefs = SerializationHelper::load_mapping_or_default(
            db_ops,
            TRANSFORM_TO_AREFS_KEY,
            "transform_to_arefs"
        )?;

        // Load transform_input_names using unified helper
        let transform_input_names = SerializationHelper::load_mapping_or_default(
            db_ops,
            TRANSFORM_INPUT_NAMES_KEY,
            "transform_input_names"
        )?;

        // Load field_to_transforms with special debug logging
        let field_to_transforms = match db_ops.get_transform_mapping(FIELD_TO_TRANSFORMS_KEY)? {
            Some(data) => {
                let loaded_map: HashMap<String, HashSet<String>> = SerializationHelper::deserialize_mapping(&data, "field_to_transforms")?;
                info!("🔍 DEBUG: Loaded field_to_transforms mapping from database with {} entries:", loaded_map.len());
                for (field_key, transforms) in &loaded_map {
                    info!("  📋 Loaded '{}' -> {:?}", field_key, transforms);
                }
                loaded_map
            }
            None => {
                LoggingHelper::log_persistence_operation("field_to_transforms", "load", false);
                HashMap::new()
            }
        };

        // Load transform_to_fields using unified helper
        let transform_to_fields = SerializationHelper::load_mapping_or_default(
            db_ops,
            TRANSFORM_TO_FIELDS_KEY,
            "transform_to_fields"
        )?;

        // Load transform_outputs using unified helper
        let transform_outputs = SerializationHelper::load_mapping_or_default(
            db_ops,
            TRANSFORM_OUTPUTS_KEY,
            "transform_outputs"
        )?;

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