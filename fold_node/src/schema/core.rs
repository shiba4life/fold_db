use crate::atom::AtomRef;
use crate::schema::types::{
    JsonSchemaDefinition, JsonSchemaField, Schema, SchemaError, Field, FieldVariant, SingleField,
};
use serde_json;
use serde::{Serialize, Deserialize};
use sled::Tree;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use log::info;
use uuid::Uuid;
use super::storage::SchemaStorage;
use super::validator::SchemaValidator;

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
    schemas: Mutex<HashMap<String, Schema>>,
    /// All schemas known to the system and their load state
    available: Mutex<HashMap<String, (Schema, SchemaState)>>,
    /// Persistent storage helper (legacy)
    storage: SchemaStorage,
    /// Unified database operations (new)
    db_ops: Option<std::sync::Arc<crate::db_operations::DbOperations>>,
    /// Schema directory path
    #[allow(dead_code)]
    schemas_dir: PathBuf,
}

impl SchemaCore {
    /// Internal helper to create the schema directory and construct the struct.
    fn init_with_dir(schemas_dir: PathBuf, schema_states_tree: Tree, schemas_tree: Tree) -> Result<Self, SchemaError> {
        let storage = SchemaStorage::new(schemas_dir.clone(), schema_states_tree, schemas_tree)?;
        Ok(Self {
            schemas: Mutex::new(HashMap::new()),
            available: Mutex::new(HashMap::new()),
            storage,
            db_ops: None,
            schemas_dir,
        })
    }

    /// Creates a new `SchemaCore` using the default `data/schemas` directory.
    #[must_use = "This returns a Result that should be handled"]
    pub fn init_default() -> Result<Self, SchemaError> {
        let db = sled::open("data")?;
        let schema_states_tree = db.open_tree("schema_states")?;
        let schemas_tree = db.open_tree("schemas")?;
        let schemas_dir = PathBuf::from("data/schemas");
        Self::init_with_dir(schemas_dir, schema_states_tree, schemas_tree)
    }

    /// Creates a new `SchemaCore` instance with a custom schemas directory.
    #[must_use = "This returns a Result containing the schema core that should be handled"]
    pub fn new(path: &str) -> Result<Self, SchemaError> {
        let db = sled::open(path)?;
        let schema_states_tree = db.open_tree("schema_states")?;
        let schemas_tree = db.open_tree("schemas")?;
        let schemas_dir = PathBuf::from(path).join("schemas");
        Self::init_with_dir(schemas_dir, schema_states_tree, schemas_tree)
    }

    /// Creates a new `SchemaCore` using existing sled trees for schema states and schemas.
    pub fn new_with_trees(path: &str, schema_states_tree: Tree, schemas_tree: Tree) -> Result<Self, SchemaError> {
        let schemas_dir = PathBuf::from(path).join("schemas");
        Self::init_with_dir(schemas_dir, schema_states_tree, schemas_tree)
    }

    /// Creates a new `SchemaCore` using unified DbOperations (Phase 2 implementation)
    pub fn new_with_db_ops(path: &str, db_ops: std::sync::Arc<crate::db_operations::DbOperations>) -> Result<Self, SchemaError> {
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
            storage: SchemaStorage::new(schemas_dir.clone(),
                                      db_ops.db().open_tree("schema_states").map_err(|e| SchemaError::InvalidData(e.to_string()))?,
                                      db_ops.db().open_tree("schemas").map_err(|e| SchemaError::InvalidData(e.to_string()))?)?,
            db_ops: Some(db_ops),
            schemas_dir,
        })
    }

    /// Gets the path for a schema file.
    pub fn schema_path(&self, schema_name: &str) -> PathBuf {
        self.storage.schema_path(schema_name)
    }

    /// Persist all schema load states to the sled tree
    fn persist_states(&self) -> Result<(), SchemaError> {
        if let Some(_db_ops) = &self.db_ops {
            // Use unified operations
            self.persist_states_unified()
        } else {
            // Use legacy storage
            let available = self
                .available
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            self.storage.persist_states(&available)
        }
    }

    /// Load schema states from the sled tree
    pub fn load_states(&self) -> HashMap<String, SchemaState> {
        if self.db_ops.is_some() {
            // Use unified operations
            self.load_states_unified()
        } else {
            // Use legacy storage
            self.storage.load_states()
        }
    }

    /// Persists a schema to disk.
    fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        if self.db_ops.is_some() {
            // Use unified operations
            self.persist_schema_unified(schema)
        } else {
            // Use legacy storage
            self.storage.persist_schema(schema)
        }
    }

    fn fix_transform_outputs(&self, schema: &mut Schema) {
        for (field_name, field) in schema.fields.iter_mut() {
            if let Some(transform) = field.transform() {
                let out_schema = transform.get_output();
                if out_schema.starts_with("test.") {
                    let mut new_transform = (*transform).clone();
                    new_transform.set_output(format!("{}.{}", schema.name, field_name));
                    field.set_transform(new_transform);
                }
            }
        }
    }

    /// Load a schema into memory and persist it to disk.
    /// This creates the schema in Available state by default.
    pub fn load_schema_internal(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!("🔄 LOAD_SCHEMA_INTERNAL START - schema: '{}' with {} fields: {:?}", schema.name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());
        
        // Log ref_atom_uuid values for each field
        for (field_name, field) in &schema.fields {
            let ref_uuid = field.ref_atom_uuid().map(|s| s.to_string()).unwrap_or_else(|| "None".to_string());
            info!("📋 Field {}.{} has ref_atom_uuid: {}", schema.name, field_name, ref_uuid);
        }

        // Ensure any transforms on fields have the correct output schema
        self.fix_transform_outputs(&mut schema);
        info!("After fix_transform_outputs, schema '{}' has {} fields: {:?}", schema.name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());

        // Persist the updated schema
        self.persist_schema(&schema)?;
        info!("After persist_schema, schema '{}' has {} fields: {:?}", schema.name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());

        // Add to memory with Available state
        let name = schema.name.clone();
        {
            let mut all = self
                .available
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            all.insert(name.clone(), (schema, SchemaState::Available));
        }

        // Persist state changes
        self.set_schema_state(&name, SchemaState::Available)?;
        info!("Schema '{}' loaded and marked as Available", name);

        Ok(())
    }

    /// Approve a schema for queries and mutations
    pub fn approve_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Approving schema '{}'", schema_name);
        
        // Check if schema exists in available
        let schema_to_approve = {
            let available = self.available.lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            available.get(schema_name).map(|(schema, _)| schema.clone())
        };
        
        let schema = schema_to_approve.ok_or_else(|| {
            SchemaError::NotFound(format!("Schema '{}' not found", schema_name))
        })?;

        info!("Schema '{}' to approve has {} fields: {:?}", schema_name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());

        // Update both in-memory stores and persist immediately
        {
            let mut schemas = self.schemas.lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            let mut available = self.available.lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            
            // Add to active schemas
            schemas.insert(schema_name.to_string(), schema.clone());
            // Update state in available
            available.insert(schema_name.to_string(), (schema, SchemaState::Approved));
        }
        
        // Persist the state change immediately
        self.persist_states()?;
        
        // Ensure fields have proper ARefs assigned
        let _ = self.map_fields(schema_name);

        info!("Schema '{}' approved successfully", schema_name);
        Ok(())
    }

    /// Block a schema from queries and mutations
    pub fn block_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Blocking schema '{}'", schema_name);
        
        // Remove from active schemas but keep in available
        {
            let mut schemas = self.schemas.lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            schemas.remove(schema_name);
        }

        self.set_schema_state(schema_name, SchemaState::Blocked)?;
        info!("Schema '{}' blocked successfully", schema_name);
        Ok(())
    }

    /// Get schemas by state
    pub fn list_schemas_by_state(&self, state: SchemaState) -> Result<Vec<String>, SchemaError> {
        let available = self.available.lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        let schemas: Vec<String> = available
            .iter()
            .filter(|(_, (_, s))| *s == state)
            .map(|(name, _)| name.clone())
            .collect();
        
        Ok(schemas)
    }

    /// Discover schemas from the schemas directory
    pub fn discover_schemas_from_files(&self) -> Result<Vec<Schema>, SchemaError> {
        let mut discovered_schemas = Vec::new();
        
        info!("Discovering schemas from {}", self.storage.schemas_dir.display());
        if let Ok(entries) = std::fs::read_dir(&self.storage.schemas_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                        if schema_opt.is_none() {
                            if let Ok(json_schema) = serde_json::from_str::<JsonSchemaDefinition>(&contents) {
                                if let Ok(schema) = self.interpret_schema(json_schema) {
                                    schema_opt = Some(schema);
                                }
                            }
                        }
                        if let Some(mut schema) = schema_opt {
                            self.fix_transform_outputs(&mut schema);
                            let schema_name = schema.name.clone();
                            discovered_schemas.push(schema);
                            info!("Discovered schema '{}' from file", schema_name);
                        }
                    }
                }
            }
        }
        
        Ok(discovered_schemas)
    }

    /// Discover schemas from the available_schemas directory
    pub fn discover_available_schemas(&self) -> Result<Vec<Schema>, SchemaError> {
        let mut discovered_schemas = Vec::new();
        let available_schemas_dir = PathBuf::from("available_schemas");
        
        info!("Discovering available schemas from {}", available_schemas_dir.display());
        if let Ok(entries) = std::fs::read_dir(&available_schemas_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                        if schema_opt.is_none() {
                            if let Ok(json_schema) = serde_json::from_str::<JsonSchemaDefinition>(&contents) {
                                if let Ok(schema) = self.interpret_schema(json_schema) {
                                    schema_opt = Some(schema);
                                }
                            }
                        }
                        if let Some(mut schema) = schema_opt {
                            self.fix_transform_outputs(&mut schema);
                            let schema_name = schema.name.clone();
                            discovered_schemas.push(schema);
                            info!("Discovered available schema '{}' from file", schema_name);
                        }
                    }
                }
            }
        }
        
        Ok(discovered_schemas)
    }

    /// Load all schemas from the available_schemas directory into SchemaCore
    pub fn load_available_schemas_from_directory(&self) -> Result<(), SchemaError> {
        let discovered_schemas = self.discover_available_schemas()?;
        
        for schema in discovered_schemas {
            let schema_name = schema.name.clone();
            info!("Loading available schema '{}' into SchemaCore", schema_name);
            self.load_schema_internal(schema)?;
        }
        
        info!("Loaded {} schemas from available_schemas directory", self.list_available_schemas()?.len());
        Ok(())
    }

    // ========== CONSOLIDATED SCHEMA API ==========
    
    /// Fetch available schemas from files (both data/schemas and available_schemas directories)
    pub fn fetch_available_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let mut all_schemas = Vec::new();
        
        // Get schemas from the default data/schemas directory
        let discovered_default = self.discover_schemas_from_files()?;
        all_schemas.extend(discovered_default.into_iter().map(|s| s.name));
        
        // Get schemas from the available_schemas directory
        let discovered_available = self.discover_available_schemas()?;
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

    /// Load schema state from sled
    pub fn load_schema_state(&self) -> Result<HashMap<String, SchemaState>, SchemaError> {
        let states = self.load_states();
        Ok(states)
    }

    /// Load available schemas from sled and files
    pub fn load_available_schemas(&self) -> Result<(), SchemaError> {
        self.load_schemas_from_disk()
    }

    /// Check if a schema can be queried (must be Approved)
    pub fn can_query_schema(&self, schema_name: &str) -> bool {
        matches!(self.get_schema_state(schema_name), Some(SchemaState::Approved))
    }

    /// Check if a schema can be mutated (must be Approved)
    pub fn can_mutate_schema(&self, schema_name: &str) -> bool {
        matches!(self.get_schema_state(schema_name), Some(SchemaState::Approved))
    }

    /// Get all available schemas (any state)
    pub fn list_all_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.list_available_schemas()
    }

    /// Persist a schema to disk in Available state.
    pub fn add_schema_available(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!("Adding schema '{}' as Available with {} fields: {:?}", schema.name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());

        // Ensure any transforms on fields have the correct output schema
        self.fix_transform_outputs(&mut schema);

        info!("After fix_transform_outputs, schema '{}' has {} fields: {:?}", schema.name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());

        // Persist the updated schema
        self.persist_schema(&schema)?;

        let name = schema.name.clone();
        let state_to_use = {
            let mut available = self
                .available
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            
            // Check if schema already exists and preserve its state
            let existing_state = available.get(&name).map(|(_, state)| *state);
            let state_to_use = existing_state.unwrap_or(SchemaState::Available);
            
            available.insert(name.clone(), (schema, state_to_use));
            
            // If the existing state was Approved, also add to the active schemas
            if state_to_use == SchemaState::Approved {
                let mut schemas = self
                    .schemas
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
                schemas.insert(name.clone(), available.get(&name).unwrap().0.clone());
            }
            
            state_to_use
        };

        // Persist state changes
        self.persist_states()?;
        info!("Schema '{}' added with preserved state: {:?}", name, state_to_use);

        Ok(())
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
        self.storage.schema_path(schema_name)
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
    pub fn update_field_ref_atom_uuid(&self, schema_name: &str, field_name: &str, ref_atom_uuid: String) -> Result<(), SchemaError> {
        info!("🔧 UPDATE_FIELD_REF_ATOM_UUID START - schema: {}, field: {}, uuid: {}", schema_name, field_name, ref_atom_uuid);
        
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        if let Some(schema) = schemas.get_mut(schema_name) {
            if let Some(field) = schema.fields.get_mut(field_name) {
                field.set_ref_atom_uuid(ref_atom_uuid.clone());
                info!("Field {}.{} ref_atom_uuid updated in memory", schema_name, field_name);
                
                // Persist the updated schema to disk
                info!("Persisting updated schema {} to disk", schema_name);
                self.storage.persist_schema(schema)?;
                info!("Schema {} persisted successfully with updated ref_atom_uuid", schema_name);
                
                // Also update the available schemas map to keep it in sync
                let mut available = self
                    .available
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire available schemas lock".to_string()))?;
                
                if let Some((available_schema, _state)) = available.get_mut(schema_name) {
                    if let Some(available_field) = available_schema.fields.get_mut(field_name) {
                        available_field.set_ref_atom_uuid(ref_atom_uuid);
                        info!("Available schema {}.{} ref_atom_uuid updated", schema_name, field_name);
                    }
                }
                
                Ok(())
            } else {
                Err(SchemaError::InvalidField(format!("Field {} not found in schema {}", field_name, schema_name)))
            }
        } else {
            Err(SchemaError::NotFound(format!("Schema {} not found", schema_name)))
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
    pub fn set_schema_state(&self, schema_name: &str, state: SchemaState) -> Result<(), SchemaError> {
        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        if let Some((_, st)) = available.get_mut(schema_name) {
            *st = state;
        } else {
            return Err(SchemaError::NotFound(format!("Schema {} not found", schema_name)));
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

    /// Legacy method - now sets schema to Available state
    pub fn unload_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        self.set_available(schema_name)
    }

    /// Loads all schema files from both the schemas directory and available_schemas directory and restores their states.
    /// Schemas marked as Approved will be loaded into active memory.
    pub fn load_schemas_from_disk(&self) -> Result<(), SchemaError> {
        let states = self.load_states();
        
        // Load from default schemas directory
        info!("Loading schemas from {}", self.storage.schemas_dir.display());
        self.load_schemas_from_directory(&self.storage.schemas_dir, &states)?;
        
        // Load from available_schemas directory
        let available_schemas_dir = PathBuf::from("available_schemas");
        info!("Loading schemas from {}", available_schemas_dir.display());
        self.load_schemas_from_directory(&available_schemas_dir, &states)?;
        
        // Persist any changes to schema states from newly discovered schemas
        self.persist_states()?;

        Ok(())
    }

    /// Helper method to load schemas from a specific directory
    fn load_schemas_from_directory(&self, dir: &PathBuf, states: &HashMap<String, SchemaState>) -> Result<(), SchemaError> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                        if schema_opt.is_none() {
                            if let Ok(json_schema) = serde_json::from_str::<JsonSchemaDefinition>(&contents) {
                                if let Ok(schema) = self.interpret_schema(json_schema) {
                                    schema_opt = Some(schema);
                                }
                            }
                        }
                        if let Some(mut schema) = schema_opt {
                            self.fix_transform_outputs(&mut schema);
                            let name = schema.name.clone();
                            let state = states
                                .get(&name)
                                .copied()
                                .unwrap_or(SchemaState::Available);
                            {
                                let mut available = self.available.lock().map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
                                available.insert(name.clone(), (schema.clone(), state));
                            }
                            if state == SchemaState::Approved {
                                let mut loaded = self.schemas.lock().map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
                                loaded.insert(name.clone(), schema);
                                drop(loaded); // Release the lock before calling map_fields
                                
                                // Ensure fields have proper ARefs assigned
                                let _ = self.map_fields(&name);
                            }
                            info!("Loaded schema '{}' from {} with state: {:?}", name, dir.display(), state);
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
        info!("DEBUG: load_schema_states_from_disk called with {} states", states.len());
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
                match self.storage.load_schema(&name) {
                    Ok(Some(mut schema)) => {
                        info!("Auto-loading approved schema '{}' from sled with {} fields: {:?}", name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());
                        
                        // 🔄 Log ref_atom_uuid values during schema loading
                        info!("🔄 SCHEMA_LOAD - Loading schema '{}' with {} fields", name, schema.fields.len());
                        for (field_name, field_def) in &schema.fields {
                            use crate::schema::types::Field;
                            match field_def.ref_atom_uuid() {
                                Some(uuid) => info!("📋 Field {}.{} has ref_atom_uuid: {}", name, field_name, uuid),
                                None => info!("📋 Field {}.{} has ref_atom_uuid: None", name, field_name),
                            }
                        }
                        
                        self.fix_transform_outputs(&mut schema);
                        info!("After fix_transform_outputs, auto-loaded schema '{}' has {} fields: {:?}", name, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());
                        schemas.insert(name.clone(), schema.clone());
                        available.insert(name.clone(), (schema, state));
                        drop(schemas); // Release the lock before calling map_fields
                        drop(available); // Release the lock before calling map_fields
                        
                        // Ensure fields have proper ARefs assigned
                        let _ = self.map_fields(&name);
                        
                        // Re-acquire locks for the next iteration
                        available = self
                            .available
                            .lock()
                            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
                        schemas = self
                            .schemas
                            .lock()
                            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
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
                match self.storage.load_schema(&name) {
                    Ok(Some(mut schema)) => {
                        // 🔄 Log ref_atom_uuid values during schema loading (non-Approved)
                        info!("🔄 SCHEMA_LOAD - Loading schema '{}' (state: {:?}) with {} fields", name, state, schema.fields.len());
                        for (field_name, field_def) in &schema.fields {
                            use crate::schema::types::Field;
                            match field_def.ref_atom_uuid() {
                                Some(uuid) => info!("📋 Field {}.{} has ref_atom_uuid: {}", name, field_name, uuid),
                                None => info!("📋 Field {}.{} has ref_atom_uuid: None", name, field_name),
                            }
                        }
                        
                        self.fix_transform_outputs(&mut schema);
                        info!("Loading schema '{}' from sled with state {:?} and {} fields: {:?}", name, state, schema.fields.len(), schema.fields.keys().collect::<Vec<_>>());
                        available.insert(name.clone(), (schema, state));
                    }
                    Ok(None) => {
                        info!("Schema '{}' not found in sled, creating empty schema", name);
                        available.insert(name.clone(), (Schema::new(name), state));
                    }
                    Err(e) => {
                        info!("Failed to load schema '{}' from sled: {}, creating empty schema", name, e);
                        available.insert(name.clone(), (Schema::new(name), state));
                    }
                }
            }
        }
        Ok(())
    }

    /// Maps fields between schemas based on their defined relationships.
    /// Returns a list of AtomRefs that need to be persisted in FoldDB.
    pub fn map_fields(&self, schema_name: &str) -> Result<Vec<AtomRef>, SchemaError> {
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

                // Create a new AtomRef for this field
                let atom_ref = match field {
                    FieldVariant::Collection(_) => {
                        // For collection fields, we'll create a placeholder AtomRef
                        // The actual collection will be created when data is added
                        AtomRef::new(Uuid::new_v4().to_string(), "system".to_string())
                    }
                    _ => {
                        // For single fields, create a normal AtomRef
                        AtomRef::new(Uuid::new_v4().to_string(), "system".to_string())
                    }
                };

                // Add the AtomRef to the list to be persisted
                atom_refs.push(atom_ref);

                // Set the ref_atom_uuid in the field
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
        // TODO: Handle Collection and Range field types based on json_field.field_type
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
            fields,
            payment_config: json_schema.payment_config,
        })
    }

    /// Interprets a JSON schema from a string and loads it as Available.
    pub fn load_schema_from_json(&self, json_str: &str) -> Result<(), SchemaError> {
        info!("Parsing JSON schema from string, length: {}", json_str.len());
        let json_schema: JsonSchemaDefinition = serde_json::from_str(json_str)
            .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {e}")))?;

        info!("JSON schema parsed successfully, name: {}, fields: {:?}", json_schema.name, json_schema.fields.keys().collect::<Vec<_>>());
        let schema = self.interpret_schema(json_schema)?;
        info!("Schema interpreted successfully, name: {}, fields: {:?}", schema.name, schema.fields.keys().collect::<Vec<_>>());
        self.load_schema_internal(schema)
    }

    /// Interprets a JSON schema from a file and loads it as Available.
    pub fn load_schema_from_file(&self, path: &str) -> Result<(), SchemaError> {
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| SchemaError::InvalidField(format!("Failed to read schema file: {e}")))?;

        info!("Loading schema from file: {}, content length: {}", path, json_str.len());
        self.load_schema_from_json(&json_str)
    }

    // ========== UNIFIED DB OPERATIONS METHODS (Phase 2) ==========

    /// Uses unified DbOperations for schema state persistence (when available)
    fn persist_states_unified(&self) -> Result<(), SchemaError> {
        if let Some(db_ops) = &self.db_ops {
            let available = self
                .available
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            
            for (name, (_, state)) in available.iter() {
                db_ops.store_schema_state(name, *state)?;
            }
            Ok(())
        } else {
            // Fallback to legacy storage
            self.persist_states()
        }
    }

    /// Uses unified DbOperations for schema state loading (when available)
    fn load_states_unified(&self) -> HashMap<String, SchemaState> {
        if let Some(db_ops) = &self.db_ops {
            // Load all schema states using unified operations
            let mut states = HashMap::new();
            
            // Get all available schemas and their states
            let available = self.available.lock().ok();
            if let Some(available) = available {
                for schema_name in available.keys() {
                    if let Ok(Some(state)) = db_ops.get_schema_state(schema_name) {
                        states.insert(schema_name.clone(), state);
                    }
                }
            }
            
            // Also try to get states for any schemas we might not know about yet
            // This is a bit tricky since we need to know schema names first
            // For now, we'll rely on the available schemas we already know about
            states
        } else {
            // Fallback to legacy storage
            self.load_states()
        }
    }

    /// Uses unified DbOperations for schema persistence (when available)
    fn persist_schema_unified(&self, schema: &Schema) -> Result<(), SchemaError> {
        if let Some(db_ops) = &self.db_ops {
            db_ops.store_schema(&schema.name, schema)
        } else {
            // Fallback to legacy storage
            self.persist_schema(schema)
        }
    }

    /// Uses unified DbOperations for schema loading (when available)
    #[allow(dead_code)]
    fn load_schema_unified(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        if let Some(db_ops) = &self.db_ops {
            db_ops.get_schema(schema_name)
        } else {
            // Fallback to legacy storage
            self.storage.load_schema(schema_name)
        }
    }

    /// Uses unified DbOperations for listing schema names (when available)
    #[allow(dead_code)]
    fn list_schema_names_unified(&self) -> Result<Vec<String>, SchemaError> {
        if let Some(db_ops) = &self.db_ops {
            db_ops.list_all_schemas()
        } else {
            // Fallback to legacy storage
            self.storage.list_schema_names()
        }
    }

    /// Uses unified DbOperations for listing schemas by state (when available)
    pub fn list_schemas_by_state_unified(&self, state: SchemaState) -> Result<Vec<String>, SchemaError> {
        if let Some(db_ops) = &self.db_ops {
            db_ops.list_schemas_by_state(state)
        } else {
            // Fallback to existing implementation
            self.list_schemas_by_state(state)
        }
    }

    /// Sets schema state using unified DbOperations (when available)
    pub fn set_schema_state_unified(&self, schema_name: &str, state: SchemaState) -> Result<(), SchemaError> {
        if let Some(db_ops) = &self.db_ops {
            // Update in-memory state
            let mut available = self
                .available
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            if let Some((_, st)) = available.get_mut(schema_name) {
                *st = state;
            } else {
                return Err(SchemaError::NotFound(format!("Schema {} not found", schema_name)));
            }
            drop(available);
            
            // Persist using unified operations
            db_ops.store_schema_state(schema_name, state)
        } else {
            // Fallback to existing implementation
            self.set_schema_state(schema_name, state)
        }
    }

    /// Checks if unified DbOperations is available
    pub fn has_unified_db_ops(&self) -> bool {
        self.db_ops.is_some()
    }
}

