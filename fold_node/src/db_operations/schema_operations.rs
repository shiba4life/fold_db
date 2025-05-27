use crate::schema::SchemaError;
use crate::schema::core::SchemaState;
use crate::schema::Schema;
use super::core::DbOperations;

impl DbOperations {
    /// Stores a schema state
    pub fn store_schema_state(&self, schema_name: &str, state: SchemaState) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(&state)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize schema state: {}", e)))?;
        self.schema_states_tree.insert(schema_name.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store schema state: {}", e)))?;
        self.schema_states_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush schema states: {}", e)))?;
        Ok(())
    }

    /// Gets a schema state
    pub fn get_schema_state(&self, schema_name: &str) -> Result<Option<SchemaState>, SchemaError> {
        if let Some(bytes) = self.schema_states_tree.get(schema_name.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get schema state: {}", e)))? {
            let state = serde_json::from_slice(&bytes)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to deserialize schema state: {}", e)))?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    /// Lists all schemas with a specific state
    pub fn list_schemas_by_state(&self, target_state: SchemaState) -> Result<Vec<String>, SchemaError> {
        let mut schemas = Vec::new();
        for result in self.schema_states_tree.iter() {
            let (key, value) = result
                .map_err(|e| SchemaError::InvalidData(format!("Failed to iterate schema states: {}", e)))?;
            let schema_name = String::from_utf8_lossy(&key).to_string();
            let state: SchemaState = serde_json::from_slice(&value)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to deserialize schema state: {}", e)))?;
            if state == target_state {
                schemas.push(schema_name);
            }
        }
        Ok(schemas)
    }

    /// Stores a schema definition
    pub fn store_schema(&self, schema_name: &str, schema: &Schema) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(schema)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize schema: {}", e)))?;
        self.schemas_tree.insert(schema_name.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store schema: {}", e)))?;
        self.schemas_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush schemas: {}", e)))?;
        Ok(())
    }

    /// Gets a schema definition
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        if let Some(bytes) = self.schemas_tree.get(schema_name.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get schema: {}", e)))? {
            let schema = serde_json::from_slice(&bytes)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to deserialize schema: {}", e)))?;
            Ok(Some(schema))
        } else {
            Ok(None)
        }
    }

    /// Lists all stored schemas
    pub fn list_all_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let mut schemas = Vec::new();
        for result in self.schemas_tree.iter() {
            let (key, _) = result
                .map_err(|e| SchemaError::InvalidData(format!("Failed to iterate schemas: {}", e)))?;
            let schema_name = String::from_utf8_lossy(&key).to_string();
            schemas.push(schema_name);
        }
        Ok(schemas)
    }
}