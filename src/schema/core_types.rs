//! Core schema types and enums for the DataFold schema system
//!
//! This module contains the fundamental data structures and types used throughout
//! the schema system, including schema states, sources, loading reports, and the
//! core SchemaCore struct.

use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::{
    Field, FieldVariant, JsonSchemaDefinition, JsonSchemaField, Schema, SchemaError, SingleField,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Report of schema discovery and loading operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaLoadingReport {
    /// All schemas discovered from any source
    pub discovered_schemas: Vec<String>,
    /// Schemas currently loaded (approved state)
    pub loaded_schemas: Vec<String>,
    /// Schemas that failed to load with error messages
    pub failed_schemas: Vec<(String, String)>,
    /// Current state of all known schemas
    pub schema_states: HashMap<String, SchemaState>,
    /// Source where each schema was discovered
    pub loading_sources: HashMap<String, SchemaSource>,
    /// Timestamp of last discovery operation
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Source of a discovered schema
#[derive(Debug, Serialize, Deserialize)]
pub enum SchemaSource {
    /// Schema from available_schemas/ directory
    AvailableDirectory,
    /// Schema from data/schemas/ directory
    DataDirectory,
    /// Schema from previously saved state
    Persistence,
}

/// State of a schema within the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SchemaState {
    /// Schema discovered from files but not yet approved by user
    #[default]
    Available,
    /// Schema approved by user, can be queried, mutated, field-mapped and transforms run
    Approved,
    /// Schema blocked by user, cannot be queried or mutated but field-mapping and transforms still run
    Blocked,
}

/// Core schema management system that combines schema interpretation, validation, and management.
///
/// SchemaCore is responsible for:
/// - Loading and validating schemas from JSON
/// - Managing schema storage and persistence
/// - Handling schema field mappings
/// - Providing schema access and validation services
///
/// This unified component simplifies the schema system by combining the functionality
/// previously split across SchemaManager and SchemaInterpreter.
pub struct SchemaCore {
    /// Thread-safe storage for loaded schemas
    pub(crate) schemas: Mutex<HashMap<String, Schema>>,
    /// All schemas known to the system and their load state
    pub(crate) available: Mutex<HashMap<String, (Schema, SchemaState)>>,
    /// Unified database operations (required)
    pub(crate) db_ops: std::sync::Arc<crate::db_operations::DbOperations>,
    /// Schema directory path
    pub(crate) schemas_dir: PathBuf,
    /// Message bus for event-driven communication
    pub(crate) message_bus: Arc<MessageBus>,
}

impl SchemaCore {
    /// Creates a new SchemaCore with DbOperations (unified approach)
    pub fn new(
        path: &str,
        db_ops: std::sync::Arc<crate::db_operations::DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<Self, SchemaError> {
        let schemas_dir = PathBuf::from(path).join("schemas");

        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&schemas_dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to create schemas directory: {}",
                    e
                )));
            }
        }

        Ok(Self {
            schemas: Mutex::new(HashMap::new()),
            available: Mutex::new(HashMap::new()),
            db_ops,
            schemas_dir,
            message_bus,
        })
    }

    /// Creates a new SchemaCore for testing purposes with a temporary database
    pub fn new_for_testing(path: &str) -> Result<Self, SchemaError> {
        let db = sled::open(path)?;
        let db_ops = std::sync::Arc::new(crate::db_operations::DbOperations::new(db)?);
        let message_bus = Arc::new(MessageBus::new());
        Self::new(path, db_ops, message_bus)
    }

    /// Creates a default SchemaCore for testing purposes
    pub fn init_default() -> Result<Self, SchemaError> {
        Self::new_for_testing("data")
    }

    /// Gets the path for a schema file.
    pub fn schema_path(&self, schema_name: &str) -> PathBuf {
        self.schemas_dir.join(format!("{}.json", schema_name))
    }

    /// Gets the file path for a schema
    pub fn get_schema_path(&self, schema_name: &str) -> PathBuf {
        self.schema_path(schema_name)
    }

    /// Retrieves a schema by name.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Lists all schema names currently loaded.
    pub fn list_loaded_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.keys().cloned().collect())
    }

    /// Lists all schemas available on disk and their state.
    pub fn list_available_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(available.keys().cloned().collect())
    }

    /// Retrieve the persisted state for a schema if known.
    pub fn get_schema_state(&self, schema_name: &str) -> Option<SchemaState> {
        let available = self.available.lock().ok()?;
        available.get(schema_name).map(|(_, s)| *s)
    }

    /// Backwards compatible method for listing loaded schemas.
    pub fn list_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.list_loaded_schemas()
    }

    /// Checks if a schema exists in the manager.
    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.contains_key(schema_name))
    }

    /// Check if a schema can be queried (must be Approved)
    pub fn can_query_schema(&self, schema_name: &str) -> bool {
        matches!(
            self.get_schema_state(schema_name),
            Some(SchemaState::Approved)
        )
    }

    /// Check if a schema can be mutated (must be Approved)
    pub fn can_mutate_schema(&self, schema_name: &str) -> bool {
        matches!(
            self.get_schema_state(schema_name),
            Some(SchemaState::Approved)
        )
    }

    /// Get all available schemas (any state)
    pub fn list_all_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.list_available_schemas()
    }

    /// Load schema state from sled
    pub fn load_schema_state(&self) -> Result<HashMap<String, SchemaState>, SchemaError> {
        let states = self.load_states();
        Ok(states)
    }

    /// Load schema states using DbOperations
    pub fn load_states(&self) -> HashMap<String, SchemaState> {
        self.db_ops.get_all_schema_states().unwrap_or_default()
    }

    /// Persist all schema load states using DbOperations
    pub(crate) fn persist_states(&self) -> Result<(), SchemaError> {
        let available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        for (name, (_, state)) in available.iter() {
            self.db_ops.store_schema_state(name, *state)?;
        }

        Ok(())
    }

    /// Persists a schema using DbOperations
    pub(crate) fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        self.db_ops.store_schema(&schema.name, schema)
    }

    /// Updates the ref_atom_uuid for a specific field in a schema and persists it to disk.
    ///
    /// **CRITICAL: This is the ONLY method that should set ref_atom_uuid on field definitions**
    ///
    /// This method is the central point for managing ref_atom_uuid values to prevent
    /// "ghost ref_atom_uuid" issues where UUIDs exist but don't point to actual AtomRefs.
    ///
    /// **Proper Usage Pattern:**
    /// 1. Field manager methods (set_field_value, update_field) create AtomRef and return UUID
    /// 2. Mutation logic calls this method with the returned UUID
    /// 3. This method sets the UUID on the actual schema (not a clone)
    /// 4. This method persists the schema to disk immediately
    /// 5. This ensures ref_atom_uuid is only set when AtomRef actually exists
    ///
    /// **Why this prevents "ghost ref_atom_uuid" issues:**
    /// - Centralizes all ref_atom_uuid setting in one place
    /// - Always persists changes immediately to disk
    /// - Only called after AtomRef is confirmed to exist
    /// - Updates both in-memory and on-disk schema representations
    ///
    /// **DO NOT** set ref_atom_uuid directly on field definitions elsewhere in the code.
    pub fn update_field_ref_atom_uuid(
        &self,
        schema_name: &str,
        field_name: &str,
        ref_atom_uuid: String,
    ) -> Result<(), SchemaError> {
        info!(
            "🔧 UPDATE_FIELD_REF_ATOM_UUID START - schema: {}, field: {}, uuid: {}",
            schema_name, field_name, ref_atom_uuid
        );

        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        if let Some(schema) = schemas.get_mut(schema_name) {
            if let Some(field) = schema.fields.get_mut(field_name) {
                field.set_ref_atom_uuid(ref_atom_uuid.clone());
                info!(
                    "Field {}.{} ref_atom_uuid updated in memory",
                    schema_name, field_name
                );

                // Persist the updated schema to disk
                info!("Persisting updated schema {} to disk", schema_name);
                self.persist_schema(schema)?;
                info!(
                    "Schema {} persisted successfully with updated ref_atom_uuid",
                    schema_name
                );

                // Also update the available schemas map to keep it in sync
                let mut available = self.available.lock().map_err(|_| {
                    SchemaError::InvalidData("Failed to acquire available schemas lock".to_string())
                })?;

                if let Some((available_schema, _state)) = available.get_mut(schema_name) {
                    if let Some(available_field) = available_schema.fields.get_mut(field_name) {
                        available_field.set_ref_atom_uuid(ref_atom_uuid);
                        info!(
                            "Available schema {}.{} ref_atom_uuid updated",
                            schema_name, field_name
                        );
                    }
                }

                Ok(())
            } else {
                Err(SchemaError::InvalidField(format!(
                    "Field {} not found in schema {}",
                    field_name, schema_name
                )))
            }
        } else {
            Err(SchemaError::NotFound(format!(
                "Schema {} not found",
                schema_name
            )))
        }
    }
}