use super::validator::SchemaValidator;
use crate::atom::{AtomRef, AtomRefRange};
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::{
    Field, FieldVariant, JsonSchemaDefinition, JsonSchemaField, Schema, SchemaError, SingleField,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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

    /// Gets the path for a schema file.
    pub fn schema_path(&self, schema_name: &str) -> PathBuf {
        self.schemas_dir.join(format!("{}.json", schema_name))
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


    /// Load a schema into memory and persist it to disk.
    /// This preserves existing schema state if it exists, otherwise defaults to Available.
    pub fn load_schema_internal(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!(
            "🔄 DEBUG: LOAD_SCHEMA_INTERNAL START - schema: '{}' with {} fields: {:?}",
            schema.name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        schema = self.resolve_persisted_schema(schema)?;

        self.log_field_refs(&schema);

        self.fix_transform_outputs(&mut schema);
        self.register_schema_transforms(&schema)?;

        self.persist_if_needed(&schema)?;

        self.update_state_and_memory(schema)?;

        Ok(())
    }

    fn resolve_persisted_schema(&self, schema: Schema) -> Result<Schema, SchemaError> {
        if let Ok(Some(persisted_schema)) = self.db_ops.get_schema(&schema.name) {
            info!(
                "📂 Found persisted schema for '{}' in database, using persisted version with field assignments",
                schema.name
            );
            Ok(persisted_schema)
        } else {
            info!("📋 No persisted schema found for '{}', using JSON version", schema.name);
            Ok(schema)
        }
    }

    fn log_field_refs(&self, schema: &Schema) {
        for (field_name, field) in &schema.fields {
            let ref_uuid = field
                .ref_atom_uuid()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "None".to_string());
            info!("📋 Field {}.{} has ref_atom_uuid: {}", schema.name, field_name, ref_uuid);
        }
    }

    fn persist_if_needed(&self, schema: &Schema) -> Result<(), SchemaError> {
        let should_persist = schema.fields.values().all(|f| f.ref_atom_uuid().is_none());
        if should_persist {
            self.persist_schema(schema)?;
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
        Ok(())
    }

    fn update_state_and_memory(&self, schema: Schema) -> Result<(), SchemaError> {
        let name = schema.name.clone();
        let existing_state = self.db_ops.get_schema_state(&name).unwrap_or(None);
        let schema_state = existing_state.unwrap_or(SchemaState::Available);

        info!(
            "Schema '{}' existing state: {:?}, using state: {:?}",
            name, existing_state, schema_state
        );

        {
            let mut all = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            all.insert(name.clone(), (schema, schema_state));
        }

        if existing_state.is_none() {
            self.set_schema_state(&name, SchemaState::Available)?;
            info!("Schema '{}' loaded and marked as Available (new schema)", name);
        } else {
            info!("Schema '{}' loaded with preserved state: {:?}", name, schema_state);
        }

        self.publish_schema_loaded(&name);
        Ok(())
    }

    fn publish_schema_loaded(&self, name: &str) {
        use crate::fold_db_core::infrastructure::message_bus::schema_events::SchemaLoaded;
        let schema_loaded_event = SchemaLoaded::new(name.to_string(), "loaded");
        if let Err(e) = self.message_bus.publish(schema_loaded_event) {
            log::warn!("Failed to publish SchemaLoaded event: {}", e);
        }
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
        match self.map_fields(schema_name) {
            Ok(atom_refs) => {
                info!(
                    "Schema '{}' field mapping successful: created {} atom references with proper types",
                    schema_name, atom_refs.len()
                );
                
                // CRITICAL: Persist the schema with field assignments to sled
                match self.get_schema(schema_name) {
                    Ok(Some(updated_schema)) => {
                        if let Err(e) = self.persist_schema(&updated_schema) {
                            log::warn!("Failed to persist schema '{}' with field assignments: {}", schema_name, e);
                        } else {
                            info!("✅ Schema '{}' with field assignments persisted to sled database", schema_name);
                        }
                    }
                    Ok(None) => {
                        log::warn!("Schema '{}' not found after field mapping", schema_name);
                    }
                    Err(e) => {
                        log::warn!("Failed to retrieve schema '{}' for persistence: {}", schema_name, e);
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
        use crate::fold_db_core::infrastructure::message_bus::schema_events::SchemaLoaded;
        let schema_loaded_event = SchemaLoaded::new(schema_name, "approved");
        if let Err(e) = self.message_bus.publish(schema_loaded_event) {
            log::warn!("Failed to publish SchemaLoaded event for approval: {}", e);
        }

        // Publish SchemaChanged event for approval
        use crate::fold_db_core::infrastructure::message_bus::schema_events::SchemaChanged;
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
        info!("Ensuring approved schema '{}' is available in schemas HashMap", schema_name);

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
                    return Err(SchemaError::InvalidData(
                        format!("Schema '{}' is not in Approved state", schema_name)
                    ));
                }
            } else {
                return Err(SchemaError::NotFound(
                    format!("Schema '{}' not found in available schemas", schema_name)
                ));
            }
        };

        // Move the schema to schemas HashMap
        if let Some(schema) = schema_to_move {
            let mut schemas = self.schemas.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            
            schemas.insert(schema_name.to_string(), schema);
            info!("Successfully moved approved schema '{}' to schemas HashMap for field mapping", schema_name);
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
        use crate::fold_db_core::infrastructure::message_bus::schema_events::SchemaChanged;
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

    /// Persist a schema to disk in Available state.
    pub fn add_schema_available(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!(
            "Adding schema '{}' as Available with {} fields: {:?}",
            schema.name,
            schema.fields.len(),
            schema.fields.keys().collect::<Vec<_>>()
        );

        // Ensure any transforms on fields have the correct output schema
        self.fix_transform_outputs(&mut schema);

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
        let json_schema = self.parse_and_validate_json_schema(json_content)?;
        let final_name = schema_name.unwrap_or_else(|| json_schema.name.clone());

        // Check for duplicates and conflicts using the dedicated module
        super::duplicate_detection::SchemaDuplicateDetector::check_schema_conflicts(
            &json_schema,
            &final_name,
            "available_schemas",
            |hash, exclude| self.find_schema_by_hash(hash, exclude),
        )?;

        // Write schema to file with hash using the dedicated module
        super::file_operations::SchemaFileOperations::write_schema_to_file(
            &json_schema,
            &final_name,
            "available_schemas",
        )?;

        // Load schema into memory
        let schema = self.interpret_schema(json_schema)?;
        self.load_schema_internal(schema)?;

        info!(
            "Schema '{}' added to available schemas and ready for approval",
            final_name
        );
        Ok(final_name)
    }

    /// Parse and validate JSON schema content
    fn parse_and_validate_json_schema(
        &self,
        json_content: &str,
    ) -> Result<super::types::JsonSchemaDefinition, SchemaError> {
        let json_schema: super::types::JsonSchemaDefinition = serde_json::from_str(json_content)
            .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {}", e)))?;

        let validator = super::validator::SchemaValidator::new(self);
        validator.validate_json_schema(&json_schema)?;
        info!("JSON schema validation passed for '{}'", json_schema.name);

        Ok(json_schema)
    }

    /// Find a schema with the same hash (for duplicate detection) in the specified directory
    /// Find a schema with the same hash (for duplicate detection)
    fn find_schema_by_hash(
        &self,
        target_hash: &str,
        exclude_name: &str,
    ) -> Result<Option<String>, SchemaError> {
        let available_schemas_dir = std::path::PathBuf::from("available_schemas");

        if let Ok(entries) = std::fs::read_dir(&available_schemas_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    // Skip the file we're trying to create
                    if let Some(file_stem) = path.file_stem() {
                        if file_stem == exclude_name {
                            continue;
                        }
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(schema_json) = serde_json::from_str::<serde_json::Value>(&content)
                        {
                            // Check if schema has a hash field
                            if let Some(existing_hash) =
                                schema_json.get("hash").and_then(|h| h.as_str())
                            {
                                if existing_hash == target_hash {
                                    if let Some(name) =
                                        schema_json.get("name").and_then(|n| n.as_str())
                                    {
                                        return Ok(Some(name.to_string()));
                                    }
                                }
                            } else {
                                // Calculate hash for schemas without hash field
                                if let Ok(calculated_hash) =
                                    super::hasher::SchemaHasher::calculate_hash(&schema_json)
                                {
                                    if calculated_hash == target_hash {
                                        if let Some(name) =
                                            schema_json.get("name").and_then(|n| n.as_str())
                                        {
                                            return Ok(Some(name.to_string()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Retrieves a schema by name.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Gets the file path for a schema
    pub fn get_schema_path(&self, schema_name: &str) -> PathBuf {
        self.schema_path(schema_name)
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
        info!("Unloading schema '{}' from active memory and setting to Available", schema_name);
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

    /// Maps fields between schemas based on their defined relationships.
    /// Returns a list of AtomRefs that need to be persisted in FoldDB.
    pub fn map_fields(&self, schema_name: &str) -> Result<Vec<AtomRef>, SchemaError> {
        info!("🔧 Starting field mapping for schema '{}'", schema_name);
        
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        // First collect all the source field ref_atom_uuids we need
        let mut field_mappings = Vec::new();
        if let Some(schema) = schemas.get(schema_name) {
            for (field_name, field) in &schema.fields {
                for (source_schema_name, source_field_name) in field.field_mappers() {
                    if let Some(source_schema) = schemas.get(source_schema_name) {
                        if let Some(source_field) = source_schema.fields.get(source_field_name) {
                            if let Some(ref_atom_uuid) = source_field.ref_atom_uuid() {
                                field_mappings.push((field_name.clone(), ref_atom_uuid.clone()));
                            }
                        }
                    }
                }
            }
        }
        drop(schemas); // Release the immutable lock

        // Now get a mutable lock to update the fields
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        let schema = schemas
            .get_mut(schema_name)
            .ok_or_else(|| SchemaError::InvalidData(format!("Schema {schema_name} not found")))?;

        // Apply the collected mappings
        for (field_name, ref_atom_uuid) in field_mappings {
            if let Some(field) = schema.fields.get_mut(&field_name) {
                field.set_ref_atom_uuid(ref_atom_uuid);
            }
        }

        let mut atom_refs = Vec::new();

        // For unmapped fields, create a new ref_atom_uuid and AtomRef
        // Only create new ARefs for fields that truly don't have them (None or empty)
        for field in schema.fields.values_mut() {
            let needs_new_aref = match field.ref_atom_uuid() {
                None => true,
                Some(uuid) => uuid.is_empty(),
            };

            if needs_new_aref {
                let ref_atom_uuid = Uuid::new_v4().to_string();

                // Create and store the appropriate atom reference type based on field type
                let key = format!("ref:{}", ref_atom_uuid);
                
                match field {
                    FieldVariant::Range(_) => {
                        // For range fields, create AtomRefRange
                        let atom_ref_range = AtomRefRange::new(ref_atom_uuid.clone());
                        if let Err(e) = self.db_ops.store_item(&key, &atom_ref_range) {
                            info!("Failed to persist AtomRefRange '{}': {}", ref_atom_uuid, e);
                        } else {
                            info!("✅ Persisted AtomRefRange: {}", key);
                        }
                        // Create a corresponding AtomRef for the return list
                        atom_refs.push(AtomRef::new(Uuid::new_v4().to_string(), "system".to_string()));
                    }
                    FieldVariant::Single(_) => {
                        // For single fields, create AtomRef
                        let atom_ref = AtomRef::new(Uuid::new_v4().to_string(), "system".to_string());
                        if let Err(e) = self.db_ops.store_item(&key, &atom_ref) {
                            info!("Failed to persist AtomRef '{}': {}", ref_atom_uuid, e);
                        } else {
                            info!("✅ Persisted AtomRef: {}", key);
                        }
                        atom_refs.push(atom_ref);
                    }
                };

                // Set the ref_atom_uuid in the field - this will be used as the key to find the AtomRef
                field.set_ref_atom_uuid(ref_atom_uuid);
            }
        }

        // Persist the updated schema
        self.persist_schema(schema)?;

        // Also update the available HashMap to keep it in sync
        let updated_schema = schema.clone();
        drop(schemas); // Release the schemas lock

        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        if let Some((_, state)) = available.get(schema_name) {
            let state = *state;
            available.insert(schema_name.to_string(), (updated_schema, state));
        }

        Ok(atom_refs)
    }

    /// Converts a JSON schema field to a FieldVariant.
    fn convert_field(json_field: JsonSchemaField) -> FieldVariant {
        let mut single_field = SingleField::new(
            json_field.permission_policy.into(),
            json_field.payment_config.into(),
            json_field.field_mappers,
        );

        if let Some(ref_atom_uuid) = json_field.ref_atom_uuid {
            single_field.set_ref_atom_uuid(ref_atom_uuid);
        }

        // Add transform if present
        if let Some(json_transform) = json_field.transform {
            single_field.set_transform(json_transform.into());
        }

        // For now, we'll create all fields as Single fields
        FieldVariant::Single(single_field)
    }

    /// Interprets a JSON schema definition and converts it to a Schema.
    pub fn interpret_schema(
        &self,
        json_schema: JsonSchemaDefinition,
    ) -> Result<Schema, SchemaError> {
        // First validate the JSON schema
        SchemaValidator::new(self).validate_json_schema(&json_schema)?;

        // Convert fields
        let mut fields = HashMap::new();
        for (field_name, json_field) in json_schema.fields {
            fields.insert(field_name, Self::convert_field(json_field));
        }

        // Create the schema
        Ok(Schema {
            name: json_schema.name,
            schema_type: json_schema.schema_type,
            fields,
            payment_config: json_schema.payment_config,
            hash: json_schema.hash,
        })
    }

    /// Interprets a JSON schema from a string and loads it as Available.
    pub fn load_schema_from_json(&self, json_str: &str) -> Result<(), SchemaError> {
        info!(
            "Parsing JSON schema from string, length: {}",
            json_str.len()
        );
        let json_schema: JsonSchemaDefinition = serde_json::from_str(json_str)
            .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {e}")))?;

        info!(
            "JSON schema parsed successfully, name: {}, fields: {:?}",
            json_schema.name,
            json_schema.fields.keys().collect::<Vec<_>>()
        );
        let schema = self.interpret_schema(json_schema)?;
        info!(
            "Schema interpreted successfully, name: {}, fields: {:?}",
            schema.name,
            schema.fields.keys().collect::<Vec<_>>()
        );
        self.load_schema_internal(schema)
    }

    /// Interprets a JSON schema from a file and loads it as Available.
    pub fn load_schema_from_file(&self, path: &str) -> Result<(), SchemaError> {
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| SchemaError::InvalidField(format!("Failed to read schema file: {e}")))?;

        info!(
            "Loading schema from file: {}, content length: {}",
            path,
            json_str.len()
        );
        self.load_schema_from_json(&json_str)
    }
}
