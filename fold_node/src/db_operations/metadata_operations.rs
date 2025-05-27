use crate::schema::SchemaError;
use super::core::DbOperations;
use uuid::Uuid;

impl DbOperations {
    /// Retrieves or generates and persists the node identifier
    pub fn get_node_id(&self) -> Result<String, SchemaError> {
        if let Some(bytes) = self.metadata_tree.get("node_id")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get node_id: {}", e)))? {
            let id = String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| String::new());
            if !id.is_empty() {
                return Ok(id);
            }
        }
        let new_id = Uuid::new_v4().to_string();
        self.set_node_id(&new_id)?;
        Ok(new_id)
    }

    /// Sets the node identifier
    pub fn set_node_id(&self, node_id: &str) -> Result<(), SchemaError> {
        self.metadata_tree.insert("node_id", node_id.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to set node_id: {}", e)))?;
        self.metadata_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush metadata: {}", e)))?;
        Ok(())
    }

    /// Retrieves the list of permitted schemas for the given node
    pub fn get_schema_permissions(&self, node_id: &str) -> Result<Vec<String>, SchemaError> {
        if let Some(bytes) = self.permissions_tree.get(node_id)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get permissions: {}", e)))? {
            if let Ok(list) = serde_json::from_slice::<Vec<String>>(&bytes) {
                return Ok(list);
            }
        }
        Ok(Vec::new())
    }

    /// Sets the permitted schemas for the given node
    pub fn set_schema_permissions(&self, node_id: &str, schemas: &[String]) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(schemas)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize permissions: {}", e)))?;
        self.permissions_tree.insert(node_id, bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store permissions: {}", e)))?;
        self.permissions_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush permissions: {}", e)))?;
        Ok(())
    }
}