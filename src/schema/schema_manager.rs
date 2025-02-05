use std::collections::HashMap;
use std::sync::Mutex;
use super::{Schema, SchemaError};  // Updated to use re-exported types

pub struct SchemaManager {
    schemas: Mutex<HashMap<String, Schema>>,
}

impl SchemaManager {
    pub fn new() -> Self {
        Self {
            schemas: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_schema(&self, schema: Schema) -> Result<(), SchemaError> {
        let mut schemas = self.schemas.lock().map_err(|_| 
            SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
        
        // Run schema transforms
        for transform in &schema.transforms {
            // TODO: Implement transform execution
            println!("Running transform: {}", transform);
        }
        
        schemas.insert(schema.name.clone(), schema);
        Ok(())
    }

    pub fn unload_schema(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let mut schemas = self.schemas.lock().map_err(|_| 
            SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.remove(schema_name).is_some())
    }

    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        let schemas = self.schemas.lock().map_err(|_| 
            SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    pub fn list_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let schemas = self.schemas.lock().map_err(|_| 
            SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.keys().cloned().collect())
    }

    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schemas = self.schemas.lock().map_err(|_| 
            SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.contains_key(schema_name))
    }

    pub fn is_loaded(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.schema_exists(schema_name)
    }

    pub fn load_schema_with_name(&self, name: &str, schema: Schema) -> Result<(), SchemaError> {
        if schema.name != name {
            return Err(SchemaError::InvalidData(format!(
                "Schema name mismatch: expected {}, got {}",
                name, schema.name
            )));
        }
        self.load_schema(schema)
    }
}
