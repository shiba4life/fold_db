use crate::schema::types::{Schema, SchemaError, SchemaField, JsonSchemaDefinition, JsonSchemaField};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use serde_json;
use uuid::Uuid;
use crate::schema::types::fields::FieldType;
use crate::atom::AtomRef;

/// Core schema management system that combines schema interpretation, validation, and management.
/// 
/// SchemaCore is responsible for:
/// - Loading and validating schemas from JSON
/// - Managing schema storage and persistence
/// - Handling schema field mappings
/// - Providing schema access and validation services
/// 
/// This unified component simplifies the schema system by combining the functionality
/// previously split across SchemaManager and SchemaInterpreter.
pub struct SchemaCore {
    /// Thread-safe storage for loaded schemas
    schemas: Mutex<HashMap<String, Schema>>,
    /// Base directory for schema storage
    schemas_dir: PathBuf,
}

impl Default for SchemaCore {
    fn default() -> Self {
        let schemas_dir = PathBuf::from("data/schemas");
        if let Err(e) = fs::create_dir_all(&schemas_dir) {
            // Ignore AlreadyExists error, but panic on other errors
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                panic!("Failed to create schemas directory: {}", e);
            }
        }
        
        Self {
            schemas: Mutex::new(HashMap::new()),
            schemas_dir,
        }
    }
}

impl SchemaCore {
    /// Creates a new SchemaCore instance with a custom schemas directory.
    #[must_use]
    pub fn new(path: &str) -> Self {
        let schemas_dir = PathBuf::from(path).join("schemas");
        if let Err(e) = fs::create_dir_all(&schemas_dir) {
            // Ignore AlreadyExists error, but panic on other errors
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                panic!("Failed to create schemas directory: {}", e);
            }
        }
        
        Self {
            schemas: Mutex::new(HashMap::new()),
            schemas_dir,
        }
    }

    /// Gets the path for a schema file.
    fn schema_path(&self, schema_name: &str) -> PathBuf {
        self.schemas_dir.join(format!("{}.json", schema_name))
    }

    /// Persists a schema to disk.
    fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        let path = self.schema_path(&schema.name);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| SchemaError::InvalidData(format!(
                    "Failed to create schema directory: {}", e
                )))?;
        }
        
        let json = serde_json::to_string_pretty(schema)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize schema: {}", e)))?;
        
        fs::write(&path, json)
            .map_err(|e| SchemaError::InvalidData(format!(
                "Failed to write schema file: {}, path: {}", e, path.to_string_lossy()
            )))?;   
        
        Ok(())
    }

    /// Loads a schema into the manager and persists it to disk.
    pub fn load_schema(&self, schema: Schema) -> Result<(), SchemaError> {
        // First persist the schema
        self.persist_schema(&schema)?;
        
        // Then add it to memory
        self.schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?
            .insert(schema.name.clone(), schema);
        
        Ok(())
    }

    /// Unloads a schema from the manager and removes its persistent storage.
    pub fn unload_schema(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        let was_present = schemas.remove(schema_name).is_some();
        
        if was_present {
            // Remove the schema file if it exists
            let path = self.schema_path(schema_name);
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to delete schema file: {}", e)))?;
            }
        }
        
        Ok(was_present)
    }

    /// Retrieves a schema by name.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Lists all schema names currently loaded.
    pub fn list_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.keys().cloned().collect())
    }

    /// Checks if a schema exists in the manager.
    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.contains_key(schema_name))
    }

    /// Loads all schema files from the schemas directory.
    pub fn load_schemas_from_disk(&self) -> Result<(), SchemaError> {
        if let Ok(entries) = std::fs::read_dir(&self.schemas_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                            if let Ok(schema) = serde_json::from_str(&contents) {
                                let _ = self.load_schema(schema);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Maps fields between schemas based on their defined relationships.
    /// Returns a list of AtomRefs that need to be persisted in FoldDB.
    pub fn map_fields(&self, schema_name: &str) -> Result<Vec<AtomRef>, SchemaError> {
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
                            if let Some(ref_atom_uuid) = source_field.get_ref_atom_uuid() {
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
                field.set_ref_atom_uuid(ref_atom_uuid);
            }
        }

        let mut atom_refs = Vec::new();

        // For unmapped fields, create a new ref_atom_uuid and AtomRef
        for (_, field) in &mut schema.fields {
            if field.get_ref_atom_uuid().is_none() {
                let ref_atom_uuid = Uuid::new_v4().to_string();
                
                // Create a new AtomRef for this field
                let atom_ref = if field.field_type() == &FieldType::Collection {
                    // For collection fields, we'll create a placeholder AtomRef
                    // The actual collection will be created when data is added
                    AtomRef::new(Uuid::new_v4().to_string(), "system".to_string())
                } else {
                    // For single fields, create a normal AtomRef
                    AtomRef::new(Uuid::new_v4().to_string(), "system".to_string())
                };
                
                // Add the AtomRef to the list to be persisted
                atom_refs.push(atom_ref);
                
                // Set the ref_atom_uuid in the field
                field.set_ref_atom_uuid(ref_atom_uuid);
            }
        }
        
        // Persist the updated schema
        self.persist_schema(schema)?;
        
        Ok(atom_refs)
    }

    /// Validates a JSON schema definition.
    fn validate_schema(&self, schema: &JsonSchemaDefinition) -> Result<(), SchemaError> {
        // Validate schema name
        if schema.name.is_empty() {
            return Err(SchemaError::InvalidField(
                "Schema name cannot be empty".to_string(),
            ));
        }

        // Validate fields
        for (field_name, field) in &schema.fields {
            if field_name.is_empty() {
                return Err(SchemaError::InvalidField(
                    "Field name cannot be empty".to_string(),
                ));
            }

            // Validate payment config
            if field.payment_config.base_multiplier <= 0.0 {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} base_multiplier must be positive"
                )));
            }

            // Validate field mappers
            for (mapper_key, mapper_value) in &field.field_mappers {
                if mapper_key.is_empty() || mapper_value.is_empty() {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} has invalid field mapper: empty key or value"
                    )));
                }
            }

            if let Some(min_payment) = field.payment_config.min_payment {
                if min_payment == 0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_payment cannot be zero"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Converts a JSON schema field to a SchemaField.
    fn convert_field(json_field: JsonSchemaField) -> SchemaField {
        SchemaField::new(
            json_field.permission_policy.into(),
            json_field.payment_config.into(),
            json_field.field_mappers,
            Some(json_field.field_type),
        )
        .with_ref_atom_uuid(json_field.ref_atom_uuid)
    }

    /// Interprets a JSON schema definition and converts it to a Schema.
    pub fn interpret_schema(&self, json_schema: JsonSchemaDefinition) -> Result<Schema, SchemaError> {
        // First validate the JSON schema
        self.validate_schema(&json_schema)?;

        // Convert fields
        let mut fields = HashMap::new();
        for (field_name, json_field) in json_schema.fields {
            fields.insert(field_name, Self::convert_field(json_field));
        }

        // Create the schema
        Ok(Schema {
            name: json_schema.name,
            fields,
            payment_config: json_schema.payment_config,
        })
    }

    /// Interprets a JSON schema from a string and loads it.
    pub fn load_schema_from_json(&self, json_str: &str) -> Result<(), SchemaError> {
        let json_schema: JsonSchemaDefinition = serde_json::from_str(json_str)
            .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {e}")))?;
        
        let schema = self.interpret_schema(json_schema)?;
        self.load_schema(schema)
    }

    /// Interprets a JSON schema from a file and loads it.
    pub fn load_schema_from_file(&self, path: &str) -> Result<(), SchemaError> {
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| SchemaError::InvalidField(format!("Failed to read schema file: {e}")))?;
        
        self.load_schema_from_json(&json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::config::FieldPaymentConfig;
    
    use crate::permissions::types::policy::PermissionsPolicy;
    use crate::schema::types::fields::{SchemaField, FieldType};
    use std::fs;

    fn cleanup_test_schema(name: &str) {
        let path = PathBuf::from("data/schemas").join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }

    fn create_test_field(ref_atom_uuid: Option<String>, field_mappers: HashMap<String, String>) -> SchemaField {
        let mut field = SchemaField::new(
            PermissionsPolicy::default(), 
            FieldPaymentConfig::default(), 
            field_mappers,
            Some(FieldType::Single),
        );
        if let Some(uuid) = ref_atom_uuid {
            field = field.with_ref_atom_uuid(uuid);
        }
        field
    }

    #[test]
    fn test_schema_persistence() {
        let test_schema_name = "test_persistence_schema";
        cleanup_test_schema(test_schema_name); // Cleanup any leftover test files
        
        let core = SchemaCore::new("data");
        
        // Create a test schema
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let schema = Schema::new(test_schema_name.to_string())
            .with_fields(fields);
        
        // Load and persist schema
        core.load_schema(schema.clone()).unwrap();
        
        // Verify file exists
        let schema_path = core.schema_path(test_schema_name);
        assert!(schema_path.exists());
        
        // Read and verify content
        let content = fs::read_to_string(&schema_path).unwrap();
        let loaded_schema: Schema = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded_schema.name, test_schema_name);
        assert_eq!(
            loaded_schema.fields.get("test_field").unwrap().get_ref_atom_uuid(),
            Some("test_uuid".to_string())
        );
        
        // Test unload removes file
        core.unload_schema(test_schema_name).unwrap();
        assert!(!schema_path.exists());
        
        cleanup_test_schema(test_schema_name);
    }

    #[test]
    fn test_map_fields_success() {
        let core = SchemaCore::new("data");
        
        // Create source schema with a field that has a ref_atom_uuid
        let mut source_fields = HashMap::new();
        source_fields.insert(
            "source_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let source_schema = Schema::new("source_schema".to_string())
            .with_fields(source_fields);
        core.load_schema(source_schema).unwrap();

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
        core.load_schema(target_schema).unwrap();

        // Map fields
        core.map_fields("target_schema").unwrap();

        // Verify the mapping
        let mapped_schema = core.get_schema("target_schema").unwrap().unwrap();
        let mapped_field = mapped_schema.fields.get("target_field").unwrap();
        assert_eq!(mapped_field.get_ref_atom_uuid(), Some("test_uuid".to_string()));
    }
}
