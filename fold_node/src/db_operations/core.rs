use crate::schema::SchemaError;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// Enhanced database operations struct that provides unified access to all database operations.
/// This replaces the previous mixed approach of direct sled access and DbOperations.
#[derive(Clone)]
pub struct DbOperations {
    /// The underlying sled database instance
    db: sled::Db,
    /// Cached trees for performance
    pub(crate) metadata_tree: sled::Tree,
    pub(crate) permissions_tree: sled::Tree,
    pub(crate) transforms_tree: sled::Tree,
    pub(crate) orchestrator_tree: sled::Tree,
    pub(crate) schema_states_tree: sled::Tree,
    pub(crate) schemas_tree: sled::Tree,
}

impl DbOperations {
    /// Creates a new enhanced DbOperations instance with all required trees
    pub fn new(db: sled::Db) -> Result<Self, sled::Error> {
        let metadata_tree = db.open_tree("metadata")?;
        let permissions_tree = db.open_tree("node_id_schema_permissions")?;
        let transforms_tree = db.open_tree("transforms")?;
        let orchestrator_tree = db.open_tree("orchestrator_state")?;
        let schema_states_tree = db.open_tree("schema_states")?;
        let schemas_tree = db.open_tree("schemas")?;
        
        Ok(Self {
            db,
            metadata_tree,
            permissions_tree,
            transforms_tree,
            orchestrator_tree,
            schema_states_tree,
            schemas_tree,
        })
    }

    /// Gets a reference to the underlying database
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Generic function to store a serializable item in the database
    pub fn store_item<T: Serialize>(&self, key: &str, item: &T) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(item)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize item: {}", e)))?;

        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store item: {}", e)))?;

        // Ensure the data is durably written to disk
        self.db
            .flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush db: {}", e)))?;

        Ok(())
    }

    /// Generic function to retrieve a deserializable item from the database
    pub fn get_item<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SchemaError> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let item = serde_json::from_slice(&bytes).map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to deserialize item: {}", e))
                })?;
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SchemaError::InvalidData(format!(
                "Failed to retrieve item: {}",
                e
            ))),
        }
    }

    /// Lists all items with a given prefix
    pub fn list_items_with_prefix(&self, prefix: &str) -> Result<Vec<String>, SchemaError> {
        let mut items = Vec::new();
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, _) = result
                .map_err(|e| SchemaError::InvalidData(format!("Failed to scan prefix: {}", e)))?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            items.push(key_str);
        }
        Ok(items)
    }

    /// Gets database statistics
    pub fn get_stats(&self) -> Result<HashMap<String, u64>, SchemaError> {
        let mut stats = HashMap::new();
        
        // Count items in each tree
        stats.insert("atoms".to_string(), self.count_items_with_prefix("atom:")?);
        stats.insert("refs".to_string(), self.count_items_with_prefix("ref:")?);
        stats.insert("metadata".to_string(), self.metadata_tree.len() as u64);
        stats.insert("permissions".to_string(), self.permissions_tree.len() as u64);
        stats.insert("transforms".to_string(), self.transforms_tree.len() as u64);
        stats.insert("orchestrator".to_string(), self.orchestrator_tree.len() as u64);
        stats.insert("schema_states".to_string(), self.schema_states_tree.len() as u64);
        stats.insert("schemas".to_string(), self.schemas_tree.len() as u64);
        
        Ok(stats)
    }

    /// Counts items with a given prefix
    fn count_items_with_prefix(&self, prefix: &str) -> Result<u64, SchemaError> {
        let mut count = 0;
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            result.map_err(|e| SchemaError::InvalidData(format!("Failed to scan prefix: {}", e)))?;
            count += 1;
        }
        Ok(count)
    }
}