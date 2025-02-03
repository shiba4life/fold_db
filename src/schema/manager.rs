use std::collections::HashMap;
use std::sync::RwLock;
use serde_json::Value;
use super::internal_schema::InternalSchema;
use super::mapper::{SchemaMapper, parse_mapping_dsl};
use super::types::SchemaError;

/// The SchemaManager holds all loaded internal schemas and schema mappers in a thread-safe manner.
pub struct SchemaManager {
    schemas: RwLock<HashMap<String, InternalSchema>>,
    mappers: RwLock<HashMap<String, SchemaMapper>>,
}

impl SchemaManager {
    /// Create a new SchemaManager instance with empty schema and mapper maps.
    pub fn new() -> Self {
        SchemaManager {
            schemas: RwLock::new(HashMap::new()),
            mappers: RwLock::new(HashMap::new()),
        }
    }

    /// Load a new schema mapper or update an existing one.
    pub fn load_mapper(
        &self,
        target_schema: &str,
        source_schemas: Vec<String>,
        mapping_dsl: &str,
    ) -> Result<(), SchemaError> {
        let rules = parse_mapping_dsl(mapping_dsl)?;
        let mapper = SchemaMapper::new(source_schemas, target_schema.to_string(), rules);
        
        let mut mappers = self.mappers
            .write()
            .map_err(|_| SchemaError::MappingError("Failed to acquire write lock".to_string()))?;
        
        mappers.insert(target_schema.to_string(), mapper);
        Ok(())
    }

    /// Apply a schema mapper to transform data from source schemas to target schema.
    pub fn apply_mapper(
        &self,
        target_schema: &str,
        sources_data: &HashMap<String, Value>,
    ) -> Result<Value, SchemaError> {
        let mappers = self.mappers
            .read()
            .map_err(|_| SchemaError::MappingError("Failed to acquire read lock".to_string()))?;
        
        let mapper = mappers
            .get(target_schema)
            .ok_or_else(|| SchemaError::MappingError(format!("No mapper found for schema '{}'", target_schema)))?;
        
        mapper.apply(sources_data)
    }

    /// Remove a schema mapper. Returns true if the mapper existed and was removed.
    pub fn remove_mapper(&self, target_schema: &str) -> Result<bool, SchemaError> {
        let mut mappers = self.mappers
            .write()
            .map_err(|_| SchemaError::MappingError("Failed to acquire write lock".to_string()))?;
        
        Ok(mappers.remove(target_schema).is_some())
    }

    /// Loads a new schema or updates an existing one.
    /// - `schema_name`: The unique name for the schema.
    /// - `schema`: The InternalSchema object containing the field-to-aref mapping.
    pub fn load_schema(&self, schema_name: &str, schema: InternalSchema) -> Result<(), SchemaError> {
        let mut schemas = self.schemas
            .write()
            .map_err(|_| SchemaError::MappingError("Failed to acquire write lock".to_string()))?;
        schemas.insert(schema_name.to_string(), schema);
        Ok(())
    }

    /// Unloads an existing schema. Returns true if the schema existed and was removed.
    pub fn unload_schema(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let mut schemas = self.schemas
            .write()
            .map_err(|_| SchemaError::MappingError("Failed to acquire write lock".to_string()))?;
        Ok(schemas.remove(schema_name).is_some())
    }

    /// Retrieves a clone of the schema if it exists.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<InternalSchema>, SchemaError> {
        let schemas = self.schemas
            .read()
            .map_err(|_| SchemaError::MappingError("Failed to acquire read lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Checks if a schema is loaded.
    pub fn is_loaded(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schemas = self.schemas
            .read()
            .map_err(|_| SchemaError::MappingError("Failed to acquire read lock".to_string()))?;
        Ok(schemas.contains_key(schema_name))
    }
}
