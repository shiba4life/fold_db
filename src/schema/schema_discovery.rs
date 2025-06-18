//! Schema discovery and loading operations
//!
//! This module handles schema discovery from disk and loading operations including:
//! - Schema discovery from disk
//! - Loading and unloading operations
//! - Directory scanning functionality

use crate::schema::core_types::{SchemaCore, SchemaState};
use crate::schema::types::{Field, Schema, SchemaError};
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;

impl SchemaCore {
    /// Load a schema into memory and persist it to disk.
    /// This preserves existing schema state if it exists, otherwise defaults to Available.
    pub fn load_schema_internal(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!(
            "🔄 DEBUG: LOAD_SCHEMA_INTERNAL START - schema: '{}' with {} fields: {:?}",
            schema.name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        // Check if there's already a persisted schema in the database
        // If so, use that instead of the JSON version to preserve field assignments
        if let Ok(Some(persisted_schema)) = self.db_ops.get_schema(&schema.name) {
            info!(
                "📂 Found persisted schema for '{}' in database, using persisted version with field assignments",
                schema.name
            );
            schema = persisted_schema;
        } else {
            info!(
                "📋 No persisted schema found for '{}', using JSON version",
                schema.name
            );
        }

        // Log ref_atom_uuid values for each field
        for (field_name, field) in &schema.fields {
            let ref_uuid = field
                .ref_atom_uuid()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "None".to_string());
            info!(
                "📋 Field {}.{} has ref_atom_uuid: {}",
                schema.name, field_name, ref_uuid
            );
        }

        // Ensure any transforms on fields have the correct output schema
        super::transforms::fix_transform_outputs(self, &mut schema);

        // Auto-register field transforms with TransformManager
        info!(
            "🔧 DEBUG: About to call register_schema_transforms for schema: {}",
            schema.name
        );
        super::transforms::register_schema_transforms(self, &schema)?;
        info!(
            "After fix_transform_outputs, schema '{}' has {} fields: {:?}",
            schema.name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        // Only persist if we're using the JSON version (don't overwrite good database version)
        let should_persist = schema
            .fields
            .values()
            .all(|field| field.ref_atom_uuid().is_none());
        if should_persist {
            self.persist_schema(&schema)?;
            info!(
                "After persist_schema, schema '{}' has {} fields: {:?}",
                schema.name,
                schema.fields.len(),
                schema.fields.keys().collect::<Vec<_>>()
            );
        } else {
            info!(
                "Skipping persist_schema for '{}' - using persisted version with field assignments",
                schema.name
            );
        }

        // Check for existing schema state, preserve it if it exists
        let name = schema.name.clone();
        let existing_state = self.db_ops.get_schema_state(&name).unwrap_or(None);
        let schema_state = existing_state.unwrap_or(SchemaState::Available);

        info!(
            "Schema '{}' existing state: {:?}, using state: {:?}",
            name, existing_state, schema_state
        );

        // Add to memory with preserved or default state
        {
            let mut all = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            all.insert(name.clone(), (schema, schema_state));
        }

        // Only persist state changes if we're using the default Available state
        // (existing states are already persisted)
        if existing_state.is_none() {
            self.set_schema_state(&name, SchemaState::Available)?;
            info!(
                "Schema '{}' loaded and marked as Available (new schema)",
                name
            );
        } else {
            info!(
                "Schema '{}' loaded with preserved state: {:?}",
                name, schema_state
            );
        }

        // Publish SchemaLoaded event
        use crate::fold_db_core::infrastructure::message_bus::SchemaLoaded;
        let schema_loaded_event = SchemaLoaded::new(name.clone(), "loaded");
        if let Err(e) = self.message_bus.publish(schema_loaded_event) {
            log::warn!("Failed to publish SchemaLoaded event: {}", e);
        }

        Ok(())
    }

    /// Loads all schema files from both the schemas directory and available_schemas directory and restores their states.
    /// Schemas marked as Approved will be loaded into active memory.
    pub fn load_schemas_from_disk(&self) -> Result<(), SchemaError> {
        let states = self.load_states();

        // Load from default schemas directory
        info!("Loading schemas from {}", self.schemas_dir.display());
        self.load_schemas_from_directory(&self.schemas_dir, &states)?;

        // Load from available_schemas directory
        let available_schemas_dir = PathBuf::from("available_schemas");
        info!("Loading schemas from {}", available_schemas_dir.display());
        self.load_schemas_from_directory(&available_schemas_dir, &states)?;

        // Persist any changes to schema states from newly discovered schemas
        self.persist_states()?;

        Ok(())
    }

    /// Helper method to load schemas from a specific directory
    pub(crate) fn load_schemas_from_directory(
        &self,
        dir: &PathBuf,
        states: &HashMap<String, SchemaState>,
    ) -> Result<(), SchemaError> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                        if schema_opt.is_none() {
                            if let Ok(json_schema) =
                                serde_json::from_str::<super::types::JsonSchemaDefinition>(&contents)
                            {
                                if let Ok(schema) = super::parsing::interpret_schema(self, json_schema) {
                                    schema_opt = Some(schema);
                                }
                            }
                        }
                        if let Some(mut schema) = schema_opt {
                            super::transforms::fix_transform_outputs(self, &mut schema);
                            let name = schema.name.clone();
                            let state =
                                states.get(&name).copied().unwrap_or(SchemaState::Available);
                            {
                                let mut available = self.available.lock().map_err(|_| {
                                    SchemaError::InvalidData(
                                        "Failed to acquire schema lock".to_string(),
                                    )
                                })?;
                                available.insert(name.clone(), (schema.clone(), state));
                            }
                            if state == SchemaState::Approved {
                                let mut loaded = self.schemas.lock().map_err(|_| {
                                    SchemaError::InvalidData(
                                        "Failed to acquire schema lock".to_string(),
                                    )
                                })?;
                                loaded.insert(name.clone(), schema);
                                drop(loaded); // Release the lock before calling map_fields

                                // Ensure fields have proper ARefs assigned
                                let _ = super::parsing::map_fields(self, &name);
                            }
                            info!(
                                "Loaded schema '{}' from {} with state: {:?}",
                                name,
                                dir.display(),
                                state
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads schema states from sled and loads schemas that are marked as Approved.
    pub fn load_schema_states_from_disk(&self) -> Result<(), SchemaError> {
        let states = self.load_states();
        info!("Loading schema states from sled: {:?}", states);
        info!(
            "DEBUG: load_schema_states_from_disk called with {} states",
            states.len()
        );
        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        for (name, state) in states {
            info!("DEBUG: Processing schema '{}' with state {:?}", name, state);
            if state == SchemaState::Approved {
                // Load the actual schema from sled database into active memory
                match self.db_ops.get_schema(&name) {
                    Ok(Some(mut schema)) => {
                        info!(
                            "Auto-loading approved schema '{}' from sled with {} fields: {:?}",
                            name,
                            schema.fields.len(),
                            schema.fields.keys().collect::<Vec<_>>()
                        );

                        // 🔄 Log ref_atom_uuid values during schema loading
                        info!(
                            "🔄 SCHEMA_LOAD - Loading schema '{}' with {} fields",
                            name,
                            schema.fields.len()
                        );
                        for (field_name, field_def) in &schema.fields {
                            use crate::schema::types::Field;
                            match field_def.ref_atom_uuid() {
                                Some(uuid) => info!(
                                    "📋 Field {}.{} has ref_atom_uuid: {}",
                                    name, field_name, uuid
                                ),
                                None => info!(
                                    "📋 Field {}.{} has ref_atom_uuid: None",
                                    name, field_name
                                ),
                            }
                        }

                        super::transforms::fix_transform_outputs(self, &mut schema);
                        info!("After fix_transform_outputs, auto-loaded schema '{}' has {} fields: {:?}", name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());
                        schemas.insert(name.clone(), schema.clone());
                        available.insert(name.clone(), (schema, state));
                        drop(schemas); // Release the lock before calling map_fields
                        drop(available); // Release the lock before calling map_fields

                        // Ensure fields have proper ARefs assigned
                        let _ = super::parsing::map_fields(self, &name);

                        // Re-acquire locks for the next iteration
                        available = self.available.lock().map_err(|_| {
                            SchemaError::InvalidData("Failed to acquire schema lock".to_string())
                        })?;
                        schemas = self.schemas.lock().map_err(|_| {
                            SchemaError::InvalidData("Failed to acquire schema lock".to_string())
                        })?;
                    }
                    Ok(None) => {
                        info!("Schema '{}' not found in sled, creating empty schema", name);
                        available.insert(name.clone(), (Schema::new(name), SchemaState::Available));
                    }
                    Err(e) => {
                        info!("Failed to load schema '{}' from sled: {}", name, e);
                        available.insert(name.clone(), (Schema::new(name), SchemaState::Available));
                    }
                }
            } else {
                // Load the actual schema from sled for non-Approved states too
                match self.db_ops.get_schema(&name) {
                    Ok(Some(mut schema)) => {
                        // 🔄 Log ref_atom_uuid values during schema loading (non-Approved)
                        info!(
                            "🔄 SCHEMA_LOAD - Loading schema '{}' (state: {:?}) with {} fields",
                            name,
                            state,
                            schema.fields.len()
                        );
                        for (field_name, field_def) in &schema.fields {
                            use crate::schema::types::Field;
                            match field_def.ref_atom_uuid() {
                                Some(uuid) => info!(
                                    "📋 Field {}.{} has ref_atom_uuid: {}",
                                    name, field_name, uuid
                                ),
                                None => info!(
                                    "📋 Field {}.{} has ref_atom_uuid: None",
                                    name, field_name
                                ),
                            }
                        }

                        super::transforms::fix_transform_outputs(self, &mut schema);
                        info!(
                            "Loading schema '{}' from sled with state {:?} and {} fields: {:?}",
                            name,
                            state,
                            schema.fields.len(),
                            schema.fields.keys().collect::<Vec<_>>()
                        );
                        available.insert(name.clone(), (schema, state));
                    }
                    Ok(None) => {
                        info!("Schema '{}' not found in sled, creating empty schema", name);
                        available.insert(name.clone(), (Schema::new(name), state));
                    }
                    Err(e) => {
                        info!(
                            "Failed to load schema '{}' from sled: {}, creating empty schema",
                            name, e
                        );
                        available.insert(name.clone(), (Schema::new(name), state));
                    }
                }
            }
        }
        Ok(())
    }

    /// Load available schemas from sled and files
    pub fn load_available_schemas(&self) -> Result<(), SchemaError> {
        self.load_schemas_from_disk()
    }

    /// Initialize schema system - called during node startup
    pub fn initialize_schema_system(&self) -> Result<(), SchemaError> {
        info!("🚀 Initializing schema system");
        super::utils::discover_and_load_all_schemas(self)?;
        info!("✅ Schema system initialized successfully");
        Ok(())
    }

    /// Fetch available schemas from files (both data/schemas and available_schemas directories)
    /// DEPRECATED: Use discover_and_load_all_schemas() instead
    pub fn fetch_available_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let mut all_schemas = Vec::new();

        // Get schemas from the default data/schemas directory
        let discovered_default = super::utils::discover_schemas_from_files(self)?;
        all_schemas.extend(discovered_default.into_iter().map(|s| s.name));

        // Get schemas from the available_schemas directory
        let discovered_available = super::utils::discover_available_schemas(self)?;
        all_schemas.extend(discovered_available.into_iter().map(|s| s.name));

        // Remove duplicates while preserving order
        let mut unique_schemas = Vec::new();
        for schema_name in all_schemas {
            if !unique_schemas.contains(&schema_name) {
                unique_schemas.push(schema_name);
            }
        }

        Ok(unique_schemas)
    }

    /// Load all schemas from the available_schemas directory into SchemaCore
    pub fn load_available_schemas_from_directory(&self) -> Result<(), SchemaError> {
        let discovered_schemas = super::utils::discover_available_schemas(self)?;

        for schema in discovered_schemas {
            let schema_name = schema.name.clone();
            info!("Loading available schema '{}' into SchemaCore", schema_name);
            self.load_schema_internal(schema)?;
        }

        info!(
            "Loaded {} schemas from available_schemas directory",
            self.list_available_schemas()?.len()
        );
        Ok(())
    }
}