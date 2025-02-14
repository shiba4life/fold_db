use super::{Schema, SchemaError}; // Updated to use re-exported types
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SchemaManager {
    schemas: Mutex<HashMap<String, Schema>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::config::FieldPaymentConfig;
    use crate::permissions::types::policy::PermissionsPolicy;
    use crate::schema::types::fields::SchemaField;

    fn create_test_field(ref_atom_uuid: Option<String>, field_mappers: HashMap<String, String>) -> SchemaField {
        let mut field = SchemaField::new(PermissionsPolicy::default(), FieldPaymentConfig::default())
            .with_field_mappers(field_mappers);
        if let Some(uuid) = ref_atom_uuid {
            field = field.with_ref_atom_uuid(uuid);
        }
        field
    }

    #[test]
    fn test_map_fields_success() {
        let manager = SchemaManager::new();
        
        // Create source schema with a field that has a ref_atom_uuid
        let mut source_fields = HashMap::new();
        source_fields.insert(
            "source_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let source_schema = Schema::new("source_schema".to_string())
            .with_fields(source_fields);
        manager.load_schema(source_schema).unwrap();

        // Create target schema with a field that maps to the source field
        let mut field_mappers = HashMap::new();
        field_mappers.insert("source_schema".to_string(), "source_field".to_string());
        let mut target_fields = HashMap::new();
        target_fields.insert(
            "target_field".to_string(),
            create_test_field(None, field_mappers),
        );
        let target_schema = Schema::new("target_schema".to_string())
            .with_fields(target_fields);
        manager.load_schema(target_schema).unwrap();

        // Map fields
        manager.map_fields("target_schema").unwrap();

        // Verify the mapping
        let mapped_schema = manager.get_schema("target_schema").unwrap().unwrap();
        let mapped_field = mapped_schema.fields.get("target_field").unwrap();
        assert_eq!(mapped_field.ref_atom_uuid, Some("test_uuid".to_string()));
    }

    #[test]
    fn test_map_fields_source_schema_not_exists() {
        let manager = SchemaManager::new();
        
        // Create target schema with a field that maps to a non-existent schema
        let mut field_mappers = HashMap::new();
        field_mappers.insert("nonexistent_schema".to_string(), "source_field".to_string());
        let mut target_fields = HashMap::new();
        target_fields.insert(
            "target_field".to_string(),
            create_test_field(None, field_mappers),
        );
        let target_schema = Schema::new("target_schema".to_string())
            .with_fields(target_fields);
        manager.load_schema(target_schema).unwrap();

        // Map fields - should handle gracefully
        manager.map_fields("target_schema").unwrap();

        // Verify the field was not mapped
        let mapped_schema = manager.get_schema("target_schema").unwrap().unwrap();
        let mapped_field = mapped_schema.fields.get("target_field").unwrap();
        assert_eq!(mapped_field.ref_atom_uuid, None);
    }

    #[test]
    fn test_map_fields_source_field_not_exists() {
        let manager = SchemaManager::new();
        
        // Create source schema without the mapped field
        let source_schema = Schema::new("source_schema".to_string());
        manager.load_schema(source_schema).unwrap();

        // Create target schema with a field that maps to a non-existent field
        let mut field_mappers = HashMap::new();
        field_mappers.insert("source_schema".to_string(), "nonexistent_field".to_string());
        let mut target_fields = HashMap::new();
        target_fields.insert(
            "target_field".to_string(),
            create_test_field(None, field_mappers),
        );
        let target_schema = Schema::new("target_schema".to_string())
            .with_fields(target_fields);
        manager.load_schema(target_schema).unwrap();

        // Map fields - should handle gracefully
        manager.map_fields("target_schema").unwrap();

        // Verify the field was not mapped
        let mapped_schema = manager.get_schema("target_schema").unwrap().unwrap();
        let mapped_field = mapped_schema.fields.get("target_field").unwrap();
        assert_eq!(mapped_field.ref_atom_uuid, None);
    }
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

    pub fn map_fields(&self, schema_name: &str) -> Result<(), SchemaError> {
        let schemas = self.schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        // First collect all the source field ref_atom_uuids we need
        let mut field_mappings = Vec::new();
        if let Some(schema) = schemas.get(schema_name) {
            for (field_name, field) in &schema.fields {
                for (source_schema_name, source_field_name) in &field.field_mappers {
                    if let Some(source_schema) = schemas.get(source_schema_name) {
                        if let Some(source_field) = source_schema.fields.get(source_field_name) {
                            if let Some(ref_atom_uuid) = &source_field.ref_atom_uuid {
                                field_mappings.push((
                                    field_name.clone(),
                                    ref_atom_uuid.clone()
                                ));
                            }
                        }
                    }
                }
            }
        }
        drop(schemas); // Release the immutable lock

        // Now get a mutable lock to update the fields
        let mut schemas = self.schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        let schema = schemas.get_mut(schema_name)
            .ok_or_else(|| SchemaError::InvalidData(format!("Schema {schema_name} not found")))?;
            
        // Apply the collected mappings
        for (field_name, ref_atom_uuid) in field_mappings {
            if let Some(field) = schema.fields.get_mut(&field_name) {
                field.ref_atom_uuid = Some(ref_atom_uuid);
            }
        }
        Ok(())
    }

}
