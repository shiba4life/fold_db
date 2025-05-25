use std::collections::HashMap;

use crate::error::{FoldDbError, FoldDbResult};

use super::DataFoldNode;
use super::config::NodeInfo;

impl DataFoldNode {
    /// Adds a trusted node to the node's trusted nodes list.
    pub fn add_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.trusted_nodes.insert(
            node_id.to_string(),
            NodeInfo {
                id: node_id.to_string(),
                trust_distance: self.config.default_trust_distance,
            },
        );
        Ok(())
    }

    /// Removes a trusted node from the node's trusted nodes list.
    pub fn remove_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.trusted_nodes.remove(node_id);
        Ok(())
    }

    /// Gets the current list of trusted nodes and their trust distances.
    pub fn get_trusted_nodes(&self) -> &HashMap<String, NodeInfo> {
        &self.trusted_nodes
    }

    /// Allows operations on a schema and persists permission for this node.
    pub fn allow_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let mut db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.allow_schema(schema_name)?;
        drop(db);
        self.grant_schema_permission(schema_name)?;
        Ok(())
    }

    /// Grants schema permission for this node.
    pub fn grant_schema_permission(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let mut perms = db.get_schema_permissions(&self.node_id);
        if !perms.contains(&schema_name.to_string()) {
            perms.push(schema_name.to_string());
            db.set_schema_permissions(&self.node_id, &perms)
                .map_err(|e| FoldDbError::Config(format!("Failed to set permissions: {}", e)))?;
        }
        Ok(())
    }

    /// Revokes schema permission for this node.
    pub fn revoke_schema_permission(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let mut perms = db.get_schema_permissions(&self.node_id);
        perms.retain(|s| s != schema_name);
        db.set_schema_permissions(&self.node_id, &perms)
            .map_err(|e| FoldDbError::Config(format!("Failed to set permissions: {}", e)))?;
        Ok(())
    }

    /// Checks if this node has permission to access the given schema.
    pub fn check_schema_permission(&self, schema_name: &str) -> FoldDbResult<bool> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let perms = db.get_schema_permissions(&self.node_id);
        Ok(perms.contains(&schema_name.to_string()))
    }
}


