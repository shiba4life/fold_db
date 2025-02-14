use super::{Schema, SchemaError}; // Updated to use re-exported types
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SchemaManager {
    schemas: Mutex<HashMap<String, Schema>>,
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self {
            schemas: Mutex::new(HashMap::new()),
        }
    }
}

impl SchemaManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a schema into the manager.
    ///
    /// # Errors
    /// Returns a `SchemaError` if the schema lock cannot be acquired.
    pub fn load_schema(&self, schema: Schema) -> Result<(), SchemaError> {
        self.schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?
            .insert(schema.name.clone(), schema);
        Ok(())
    }

    /// Unloads a schema from the manager.
    ///
    /// # Errors
    /// Returns a `SchemaError::MappingError` if the schema lock cannot be acquired.
    pub fn unload_schema(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.remove(schema_name).is_some())
    }

    /// Retrieves a schema by name.
    ///
    /// # Errors
    /// Returns a `SchemaError::MappingError` if the schema lock cannot be acquired.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Lists all schema names.
    ///
    /// # Errors
    /// Returns a `SchemaError::MappingError` if the schema lock cannot be acquired.
    pub fn list_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.keys().cloned().collect())
    }

    /// Checks if a schema exists.
    ///
    /// # Errors
    /// Returns a `SchemaError::MappingError` if the schema lock cannot be acquired.
    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.contains_key(schema_name))
    }

    /// Checks if a schema is loaded.
    ///
    /// # Errors
    /// Returns a `SchemaError::MappingError` if the schema lock cannot be acquired.
    pub fn is_loaded(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.schema_exists(schema_name)
    }

    /// Loads a schema with a specific name.
    ///
    /// # Errors
    /// Returns:
    /// - `SchemaError::InvalidData` if the schema name doesn't match the provided name
    /// - `SchemaError::MappingError` if the schema lock cannot be acquired
    pub fn load_schema_with_name(&self, name: &str, schema: Schema) -> Result<(), SchemaError> {
        if schema.name != name {
            return Err(SchemaError::InvalidData(format!(
                "Schema name mismatch: expected {name}, got {schema_name}",
                schema_name = schema.name
            )));
        }
        self.load_schema(schema)?;
        Ok(())
    }

}
