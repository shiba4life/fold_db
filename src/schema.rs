use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

/// The internal schema maps field names to aref UUIDs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Default)]
pub struct InternalSchema {
    pub fields: HashMap<String, String>,
}

/// The SchemaManager holds all loaded internal schemas in a thread-safe manner.
pub struct SchemaManager {
    schemas: RwLock<HashMap<String, InternalSchema>>,
}

impl SchemaManager {
    /// Create a new SchemaManager instance with an empty schema map.
    pub fn new() -> Self {
        SchemaManager {
            schemas: RwLock::new(HashMap::new()),
        }
    }

    /// Loads a new schema or updates an existing one.
    /// - `schema_name`: The unique name for the schema.
    /// - `schema`: The InternalSchema object containing the field-to-aref mapping.
    pub fn load_schema(&self, schema_name: &str, schema: InternalSchema) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|_| "Failed to acquire write lock")?;
        schemas.insert(schema_name.to_string(), schema);
        Ok(())
    }

    /// Unloads an existing schema. Returns true if the schema existed and was removed.
    pub fn unload_schema(&self, schema_name: &str) -> Result<bool, String> {
        let mut schemas = self.schemas.write().map_err(|_| "Failed to acquire write lock")?;
        Ok(schemas.remove(schema_name).is_some())
    }

    /// Retrieves a clone of the schema if it exists.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<InternalSchema>, String> {
        let schemas = self.schemas.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Checks if a schema is loaded.
    pub fn is_loaded(&self, schema_name: &str) -> Result<bool, String> {
        let schemas = self.schemas.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(schemas.contains_key(schema_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_load_and_get() {
        let manager = SchemaManager::new();
        let mut fields = HashMap::new();
        fields.insert("username".to_string(), "aref-uuid-for-username".to_string());
        fields.insert("posts".to_string(), "aref-uuid-for-posts".to_string());
        let schema = InternalSchema { fields };
        
        assert!(manager.load_schema("social", schema.clone()).is_ok());
        let retrieved = manager.get_schema("social").unwrap().unwrap();
        assert_eq!(
            retrieved.fields.get("username"),
            Some(&"aref-uuid-for-username".to_string())
        );
    }

    #[test]
    fn test_schema_unload() {
        let manager = SchemaManager::new();
        let schema = InternalSchema {
            fields: HashMap::new(),
        };
        
        assert!(manager.load_schema("social", schema).is_ok());
        assert!(manager.is_loaded("social").unwrap());
        assert!(manager.unload_schema("social").unwrap());
        assert!(!manager.is_loaded("social").unwrap());
    }

    #[test]
    fn test_nonexistent_schema() {
        let manager = SchemaManager::new();
        assert!(manager.get_schema("nonexistent").unwrap().is_none());
        assert!(!manager.is_loaded("nonexistent").unwrap());
    }

    #[test]
    fn test_schema_update() {
        let manager = SchemaManager::new();
        let mut fields1 = HashMap::new();
        fields1.insert("field1".to_string(), "uuid1".to_string());
        let schema1 = InternalSchema { fields: fields1 };
        
        let mut fields2 = HashMap::new();
        fields2.insert("field1".to_string(), "uuid2".to_string());
        let schema2 = InternalSchema { fields: fields2 };
        
        assert!(manager.load_schema("test", schema1).is_ok());
        assert!(manager.load_schema("test", schema2).is_ok());
        
        let retrieved = manager.get_schema("test").unwrap().unwrap();
        assert_eq!(
            retrieved.fields.get("field1"),
            Some(&"uuid2".to_string())
        );
    }
}
