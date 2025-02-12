use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

use crate::folddb::FoldDB;
use crate::schema::types::{Mutation, Query};
use crate::schema::{Schema, SchemaError};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    pub storage_path: PathBuf,
    pub default_trust_distance: u32,
}

#[derive(Clone)]
pub struct DataFoldNode {
    db: Arc<FoldDB>,
    config: NodeConfig,
}

#[derive(Debug)]
pub enum NodeError {
    DatabaseError(String),
    SchemaError(SchemaError),
    PermissionError(String),
    ConfigError(String),
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::SchemaError(err) => write!(f, "Schema error: {}", err),
            Self::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for NodeError {}

impl From<SchemaError> for NodeError {
    fn from(error: SchemaError) -> Self {
        NodeError::SchemaError(error)
    }
}

impl From<sled::Error> for NodeError {
    fn from(error: sled::Error) -> Self {
        NodeError::DatabaseError(error.to_string())
    }
}

pub type NodeResult<T> = Result<T, NodeError>;

impl DataFoldNode {
    /// Initialize a new node with the given configuration
    pub fn new(config: NodeConfig) -> NodeResult<Self> {
        let db = Arc::new(FoldDB::new(
            config
                .storage_path
                .to_str()
                .ok_or_else(|| NodeError::ConfigError("Invalid storage path".to_string()))?,
        )?);

        Ok(Self { db, config })
    }

    /// Load an existing database node
    pub fn load(config: NodeConfig) -> NodeResult<Self> {
        // For now, loading is same as creating new since FoldDB handles existing data
        Self::new(config)
    }

    /// Load a schema into the database
    pub fn load_schema(&mut self, schema: Schema) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .load_schema(schema)?;
        Ok(())
    }

    /// Get a schema by its ID
    pub fn get_schema(&self, schema_id: &str) -> NodeResult<Option<Schema>> {
        Ok(self.db.schema_manager.get_schema(schema_id)?)
    }

    /// Execute a query against the database
    pub fn query(&self, mut query: Query) -> NodeResult<Vec<Result<Value, SchemaError>>> {
        // Apply default trust distance if not set
        if query.trust_distance == 0 {
            query.trust_distance = self.config.default_trust_distance;
        }

        // Execute query and return results
        Ok(self.db.query_schema(query))
    }

    /// Execute a mutation on the database
    pub fn mutate(&mut self, mutation: Mutation) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .write_schema(mutation)?;
        Ok(())
    }

    /// Set the default trust distance for this node
    pub fn set_trust_distance(&mut self, distance: u32) -> NodeResult<()> {
        if distance == 0 {
            return Err(NodeError::ConfigError(
                "Trust distance must be greater than 0".to_string(),
            ));
        }
        self.config.default_trust_distance = distance;
        Ok(())
    }

    /// Get the version history for a specific atom reference
    pub fn get_history(&self, aref_uuid: &str) -> NodeResult<Vec<Value>> {
        let history = self
            .db
            .get_atom_history(aref_uuid)
            .map_err(|e| NodeError::DatabaseError(e.to_string()))?;

        Ok(history
            .into_iter()
            .map(|atom| atom.content().clone())
            .collect())
    }

    /// Allow operations on a schema
    pub fn allow_schema(&mut self, schema_name: &str) -> NodeResult<()> {
        Arc::get_mut(&mut self.db)
            .ok_or_else(|| NodeError::ConfigError("Cannot get mutable reference to database".into()))?
            .allow_schema(schema_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_config() -> NodeConfig {
        let dir = tempdir().unwrap();
        NodeConfig {
            storage_path: dir.path().to_path_buf(),
            default_trust_distance: 1,
        }
    }

    #[test]
    fn test_node_creation() {
        let config = create_test_config();
        let node = DataFoldNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_trust_distance_validation() {
        let config = create_test_config();
        let mut node = DataFoldNode::new(config).unwrap();

        assert!(node.set_trust_distance(0).is_err());
        assert!(node.set_trust_distance(1).is_ok());
    }

    // Add more tests as needed...
}
