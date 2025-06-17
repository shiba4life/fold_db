//! Schema CRUD operations, state management, and loading/unloading functionality
//!
//! This module contains the operational logic for schema management including:
//! - CRUD operations (add, approve, block schemas)
//! - State management (schema states, status retrieval)
//! - Loading and unloading operations (schema discovery, disk operations)

use crate::atom::{AtomRef, AtomRefRange};
use crate::schema::core_types::{SchemaCore, SchemaLoadingReport, SchemaSource, SchemaState};
use crate::schema::types::{Field, Schema, SchemaError};
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

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

    /// Approve a schema for queries and mutations
    pub fn approve_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Approving schema '{}'", schema_name);

        // Check if schema exists in available
        let schema_to_approve = {
            let available = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            available.get(schema_name).map(|(schema, _)| schema.clone())
        };

        let schema = schema_to_approve
            .ok_or_else(|| SchemaError::NotFound(format!("Schema '{}' not found", schema_name)))?;

        info!(
            "Schema '{}' to approve has {} fields: {:?}",
            schema_name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        // Update both in-memory stores and persist immediately
        {
            let mut schemas = self.schemas.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            let mut available = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;

            // Add to active schemas
            schemas.insert(schema_name.to_string(), schema.clone());
            // Update state in available
            available.insert(schema_name.to_string(), (schema, SchemaState::Approved));
        }

        // Persist the state change immediately
        self.persist_states()?;

        // Ensure fields have proper ARefs assigned (persistence happens in map_fields)
        match super::parsing::map_fields(self, schema_name) {
            Ok(atom_refs) => {
                info!(
                    "Schema '{}' field mapping successful: created {} atom references with proper types",
                    schema_name, atom_refs.len()
                );

                // CRITICAL: Persist the schema with field assignments to sled
                match self.get_schema(schema_name) {
                    Ok(Some(updated_schema)) => {
                        if let Err(e) = self.persist_schema(&updated_schema) {
                            log::warn!(
                                "Failed to persist schema '{}' with field assignments: {}",
                                schema_name,
                                e
                            );
                        } else {
                            info!(
                                "✅ Schema '{}' with field assignments persisted to sled database",
                                schema_name
                            );
                        }
                    }
                    Ok(None) => {
                        log::warn!("Schema '{}' not found after field mapping", schema_name);
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to retrieve schema '{}' for persistence: {}",
                            schema_name,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                info!(
                    "Schema '{}' field mapping failed: {}. Schema approved but fields may not work correctly.",
                    schema_name, e
                );
            }
        }

        // Transforms are already registered during initial schema loading
        // TransformManager will auto-reload transforms when it receives the SchemaChanged event
        info!("✅ Transform registration handled by event-driven TransformManager reload");

        // Publish SchemaLoaded event for approval
        use crate::fold_db_core::infrastructure::message_bus::SchemaLoaded;
        let schema_loaded_event = SchemaLoaded::new(schema_name, "approved");
        if let Err(e) = self.message_bus.publish(schema_loaded_event) {
            log::warn!("Failed to publish SchemaLoaded event for approval: {}", e);
        }

        // Publish SchemaChanged event for approval
        use crate::fold_db_core::infrastructure::message_bus::SchemaChanged;
        let schema_changed_event = SchemaChanged::new(schema_name);
        if let Err(e) = self.message_bus.publish(schema_changed_event) {
            log::warn!("Failed to publish SchemaChanged event for approval: {}", e);
        }

        info!("Schema '{}' approved successfully", schema_name);
        Ok(())
    }

    /// Ensures an approved schema is present in the schemas HashMap for field mapping
    /// This is used during initialization to fix the issue where approved schemas
    /// loaded from disk remain in 'available' but map_fields() only looks in 'schemas'
    pub fn ensure_approved_schema_in_schemas(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!(
            "Ensuring approved schema '{}' is available in schemas HashMap",
            schema_name
        );

        // Check if schema is already in schemas HashMap
        {
            let schemas = self.schemas.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            if schemas.contains_key(schema_name) {
                info!("Schema '{}' already in schemas HashMap", schema_name);
                return Ok(());
            }
        }

        // Get the schema from available HashMap and verify it's approved
        let schema_to_move = {
            let available = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;

            if let Some((schema, state)) = available.get(schema_name) {
                if *state == SchemaState::Approved {
                    Some(schema.clone())
                } else {
                    return Err(SchemaError::InvalidData(format!(
                        "Schema '{}' is not in Approved state",
                        schema_name
                    )));
                }
            } else {
                return Err(SchemaError::NotFound(format!(
                    "Schema '{}' not found in available schemas",
                    schema_name
                )));
            }
        };

        // Move the schema to schemas HashMap
        if let Some(schema) = schema_to_move {
            let mut schemas = self.schemas.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;

            schemas.insert(schema_name.to_string(), schema);
            info!(
                "Successfully moved approved schema '{}' to schemas HashMap for field mapping",
                schema_name
            );
        }

        Ok(())
    }

    /// Block a schema from queries and mutations
    pub fn block_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Blocking schema '{}'", schema_name);

        // Remove from active schemas but keep in available
        {
            let mut schemas = self.schemas.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            schemas.remove(schema_name);
        }

        self.set_schema_state(schema_name, SchemaState::Blocked)?;

        // Publish SchemaChanged event for blocking
        use crate::fold_db_core::infrastructure::message_bus::SchemaChanged;
        let schema_changed_event = SchemaChanged::new(schema_name);
        if let Err(e) = self.message_bus.publish(schema_changed_event) {
            log::warn!("Failed to publish SchemaChanged event for blocking: {}", e);
        }

        info!("Schema '{}' blocked successfully", schema_name);
        Ok(())
    }

    /// Get schemas by state
    pub fn list_schemas_by_state(&self, state: SchemaState) -> Result<Vec<String>, SchemaError> {
        let available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        let schemas: Vec<String> = available
            .iter()
            .filter(|(_, (_, s))| *s == state)
            .map(|(name, _)| name.clone())
            .collect();

        Ok(schemas)
    }

    /// Sets the state for a schema and persists all schema states.
    pub fn set_schema_state(
        &self,
        schema_name: &str,
        state: SchemaState,
    ) -> Result<(), SchemaError> {
        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        if let Some((_, st)) = available.get_mut(schema_name) {
            *st = state;
        } else {
            return Err(SchemaError::NotFound(format!(
                "Schema {} not found",
                schema_name
            )));
        }
        drop(available);
        self.persist_states()
    }

    /// Mark a schema as Available (remove from active schemas but keep discoverable)
    pub fn set_available(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Setting schema '{}' to Available", schema_name);
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        schemas.remove(schema_name);
        drop(schemas);
        self.set_schema_state(schema_name, SchemaState::Available)?;
        info!("Schema '{}' marked as Available", schema_name);
        Ok(())
    }

    /// Unload schema from active memory and set to Available state (preserving field assignments)
    pub fn unload_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!(
            "Unloading schema '{}' from active memory and setting to Available",
            schema_name
        );
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        schemas.remove(schema_name);
        drop(schemas);
        self.set_schema_state(schema_name, SchemaState::Available)?;
        info!("Schema '{}' unloaded and marked as Available", schema_name);
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

    /// Get comprehensive schema status for UI
    pub fn get_schema_status(&self) -> Result<SchemaLoadingReport, SchemaError> {
        info!("📊 Getting schema status");

        let schema_states = self.load_states();
        let loaded_schemas = self
            .list_schemas_by_state(SchemaState::Approved)
            .unwrap_or_else(|_| Vec::new());

        // Get all known schemas from states
        let discovered_schemas: Vec<String> = schema_states.keys().cloned().collect();

        // Create loading sources map (we don't track this in current implementation)
        let loading_sources: HashMap<String, SchemaSource> = discovered_schemas
            .iter()
            .map(|name| (name.clone(), SchemaSource::Persistence))
            .collect();

        Ok(SchemaLoadingReport {
            discovered_schemas,
            loaded_schemas,
            failed_schemas: Vec::new(), // No failures in status check
            schema_states,
            loading_sources,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Persist a schema to disk in Available state.
    pub fn add_schema_available(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!(
            "Adding schema '{}' as Available with {} fields: {:?}",
            schema.name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        // Ensure any transforms on fields have the correct output schema
        super::transforms::fix_transform_outputs(self, &mut schema);

        // Validate the schema after fixing transform outputs
        let validator = super::validator::SchemaValidator::new(self);
        validator.validate(&schema)?;
        info!("Schema '{}' validation passed", schema.name);

        info!(
            "After fix_transform_outputs, schema '{}' has {} fields: {:?}",
            schema.name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        // Persist the updated schema
        self.persist_schema(&schema)?;

        let name = schema.name.clone();
        let state_to_use = {
            let mut available = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;

            // Check if schema already exists and preserve its state
            let existing_state = available.get(&name).map(|(_, state)| *state);
            let state_to_use = existing_state.unwrap_or(SchemaState::Available);

            available.insert(name.clone(), (schema, state_to_use));

            // If the existing state was Approved, also add to the active schemas
            if state_to_use == SchemaState::Approved {
                let mut schemas = self.schemas.lock().map_err(|_| {
                    SchemaError::InvalidData("Failed to acquire schema lock".to_string())
                })?;
                schemas.insert(name.clone(), available.get(&name).unwrap().0.clone());
            }

            state_to_use
        };

        // Persist state changes
        self.persist_states()?;
        info!(
            "Schema '{}' added with preserved state: {:?}",
            name, state_to_use
        );

        Ok(())
    }

    /// Add a new schema from JSON to the available_schemas directory with validation
    pub fn add_schema_to_available_directory(
        &self,
        json_content: &str,
        schema_name: Option<String>,
    ) -> Result<String, SchemaError> {
        info!("Adding new schema to available_schemas directory");

        // Parse and validate the JSON schema
        let json_schema = super::parsing::parse_and_validate_json_schema(self, json_content)?;
        let final_name = schema_name.unwrap_or_else(|| json_schema.name.clone());

        // Check for duplicates and conflicts using the dedicated module
        super::duplicate_detection::SchemaDuplicateDetector::check_schema_conflicts(
            &json_schema,
            &final_name,
            "available_schemas",
            |hash, exclude| super::utils::find_schema_by_hash(self, hash, exclude),
        )?;

        // Write schema to file with hash using the dedicated module
        super::file_operations::SchemaFileOperations::write_schema_to_file(
            &json_schema,
            &final_name,
            "available_schemas",
        )?;

        // Load schema into memory
        let schema = super::parsing::interpret_schema(self, json_schema)?;
        self.load_schema_internal(schema)?;

        info!(
            "Schema '{}' added to available schemas and ready for approval",
            final_name
        );
        Ok(final_name)
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