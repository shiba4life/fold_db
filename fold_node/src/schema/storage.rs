use crate::schema::types::{Schema, SchemaError};
use super::core::SchemaState;
use log::info;
use serde_json;
use sled::Tree;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Handles persistence of schemas and their states on disk
pub struct SchemaStorage {
    pub(crate) schemas_dir: PathBuf,
    pub(crate) schema_states_tree: Tree,
}

impl SchemaStorage {
    /// Create a new storage helper ensuring the schema directory exists
    pub fn new(schemas_dir: PathBuf, schema_states_tree: Tree) -> Result<Self, SchemaError> {
        if let Err(e) = fs::create_dir_all(&schemas_dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to create schemas directory: {}",
                    e
                )));
            }
        }
        Ok(Self { schemas_dir, schema_states_tree })
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

    /// Persists a schema to disk
    pub fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        let path = self.schema_path(&schema.name);

        info!("Persisting schema '{}' to {}", schema.name, path.display());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to create schema directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(schema)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize schema: {}", e)))?;

        fs::write(&path, json).map_err(|e| {
            SchemaError::InvalidData(format!(
                "Failed to write schema file: {}, path: {}",
                e,
                path.to_string_lossy()
            ))
        })?;

        info!("Schema '{}' persisted to disk", schema.name);
        Ok(())
    }
}

