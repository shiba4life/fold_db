use crate::schema::types::{Schema, SchemaError};
use super::core::SchemaState;
use log::info;
use serde_json;
use sled::Tree;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Handles persistence of schemas and their states using sled
pub struct SchemaStorage {
    pub(crate) schemas_dir: PathBuf,
    pub(crate) schema_states_tree: Tree,
    pub(crate) schemas_tree: Tree,
}

impl SchemaStorage {
    /// Create a new storage helper ensuring the schema directory exists
    pub fn new(schemas_dir: PathBuf, schema_states_tree: Tree, schemas_tree: Tree) -> Result<Self, SchemaError> {
        if let Err(e) = fs::create_dir_all(&schemas_dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to create schemas directory: {}",
                    e
                )));
            }
        }
        Ok(Self { schemas_dir, schema_states_tree, schemas_tree })
    }

    /// Gets the path for a schema file
    pub fn schema_path(&self, schema_name: &str) -> PathBuf {
        self.schemas_dir.join(format!("{}.json", schema_name))
    }

    /// Persist all schema load states to the sled tree
    pub fn persist_states(&self, available: &HashMap<String, (Schema, SchemaState)>) -> Result<(), SchemaError> {
        info!("Persisting schema states to sled tree");
        // Remove stale entries
        for key in self.schema_states_tree.iter().keys() {
            let k = key?;
            if let Ok(name) = std::str::from_utf8(&k) {
                if !available.contains_key(name) {
                    self.schema_states_tree.remove(name)?;
                }
            }
        }

        for (name, (_, state)) in available.iter() {
            let bytes = serde_json::to_vec(state)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize state: {}", e)))?;
            self.schema_states_tree.insert(name.as_bytes(), bytes)?;
        }
        self.schema_states_tree.flush()?;
        info!("Schema states persisted successfully");
        Ok(())
    }

    /// Load schema states from the sled tree
    pub fn load_states(&self) -> HashMap<String, SchemaState> {
        let mut map = HashMap::new();
        info!("Loading states from sled tree...");
        for item in self.schema_states_tree.iter().flatten() {
            if let Ok(name) = String::from_utf8(item.0.to_vec()) {
                if let Ok(state) = serde_json::from_slice::<SchemaState>(&item.1) {
                    info!("Found schema state: {} = {:?}", name, state);
                    map.insert(name, state);
                }
            }
        }
        info!("Loaded {} schema states", map.len());
        map
    }

    /// Persists a schema to sled database
    pub fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        info!("Persisting schema '{}' to sled database", schema.name);
        
        let bytes = serde_json::to_vec(schema)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize schema: {}", e)))?;
        
        self.schemas_tree.insert(&schema.name, bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to persist schema to sled: {}", e)))?;
        
        info!("Schema '{}' persisted to sled database", schema.name);
        Ok(())
    }

    /// Loads a schema from sled database
    pub fn load_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        match self.schemas_tree.get(schema_name) {
            Ok(Some(bytes)) => {
                let schema: Schema = serde_json::from_slice(&bytes)
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to deserialize schema: {}", e)))?;
                Ok(Some(schema))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SchemaError::InvalidData(format!("Failed to load schema from sled: {}", e)))
        }
    }

    /// Lists all schema names stored in sled
    pub fn list_schema_names(&self) -> Result<Vec<String>, SchemaError> {
        let mut names = Vec::new();
        for key_result in self.schemas_tree.iter().keys() {
            let key = key_result.map_err(|e| SchemaError::InvalidData(format!("Failed to read schema key: {}", e)))?;
            let name = std::str::from_utf8(&key)
                .map_err(|e| SchemaError::InvalidData(format!("Invalid UTF-8 in schema name: {}", e)))?;
            names.push(name.to_string());
        }
        Ok(names)
    }
}

