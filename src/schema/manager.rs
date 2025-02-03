use std::collections::HashMap;
use std::sync::RwLock;
use super::internal_schema::InternalSchema;

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
