use crate::schema::SchemaError;
use serde::{de::DeserializeOwned, Serialize};
use super::core::DbOperations;

impl DbOperations {
    /// Stores orchestrator state
    pub fn store_orchestrator_state<T: Serialize>(&self, key: &str, state: &T) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(state)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize orchestrator state: {}", e)))?;
        self.orchestrator_tree.insert(key.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store orchestrator state: {}", e)))?;
        self.orchestrator_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush orchestrator state: {}", e)))?;
        Ok(())
    }

    /// Gets orchestrator state
    pub fn get_orchestrator_state<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SchemaError> {
        if let Some(bytes) = self.orchestrator_tree.get(key.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get orchestrator state: {}", e)))? {
            let state = serde_json::from_slice(&bytes)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to deserialize orchestrator state: {}", e)))?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
}