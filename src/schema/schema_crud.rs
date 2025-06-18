//! Schema CRUD operations
//!
//! This module handles Create, Read, Update, and Delete operations for schemas including:
//! - Add, approve, block schema operations
//! - Schema state transitions
//! - Schema persistence and validation

use crate::schema::core_types::{SchemaCore, SchemaLoadingReport, SchemaSource, SchemaState};
use crate::schema::types::{Schema, SchemaError};
use log::info;
use std::collections::HashMap;

impl SchemaCore {
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
}