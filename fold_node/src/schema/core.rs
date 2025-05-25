use crate::atom::AtomRef;
use crate::schema::types::{
    JsonSchemaDefinition, JsonSchemaField, Schema, SchemaError, Field, FieldVariant, SingleField,
};
use serde_json;
use serde::{Serialize, Deserialize};
use sled::Tree;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use log::{info, debug};
use uuid::Uuid;

/// State of a schema within the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaState {
    Loaded,
    Unloaded,
}

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
    /// All schemas known to the system and their load state
    available: Mutex<HashMap<String, (Schema, SchemaState)>>,
    /// Base directory for schema storage
    schemas_dir: PathBuf,
    /// Sled tree storing schema states
    schema_states_tree: Tree,
}

impl SchemaCore {
    /// Internal helper to create the schema directory and construct the struct.
    fn init_with_dir(schemas_dir: PathBuf, schema_states_tree: Tree) -> Result<Self, SchemaError> {
        if let Err(e) = fs::create_dir_all(&schemas_dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to create schemas directory: {}",
                    e
                )));
            }
        }

        Ok(Self {
            schemas: Mutex::new(HashMap::new()),
            available: Mutex::new(HashMap::new()),
            schemas_dir,
            schema_states_tree,
        })
    }

    /// Creates a new `SchemaCore` using the default `data/schemas` directory.
    #[must_use = "This returns a Result that should be handled"]
    pub fn init_default() -> Result<Self, SchemaError> {
        let db = sled::open("data")?;
        let tree = db.open_tree("schema_states")?;
        let schemas_dir = PathBuf::from("data/schemas");
        Self::init_with_dir(schemas_dir, tree)
    }

    /// Creates a new `SchemaCore` instance with a custom schemas directory.
    #[must_use = "This returns a Result containing the schema core that should be handled"]
    pub fn new(path: &str) -> Result<Self, SchemaError> {
        let db = sled::open(path)?;
        let tree = db.open_tree("schema_states")?;
        let schemas_dir = PathBuf::from(path).join("schemas");
        Self::init_with_dir(schemas_dir, tree)
    }

    /// Creates a new `SchemaCore` using an existing sled tree for schema states.
    pub fn new_with_tree(path: &str, schema_states_tree: Tree) -> Result<Self, SchemaError> {
        let schemas_dir = PathBuf::from(path).join("schemas");
        Self::init_with_dir(schemas_dir, schema_states_tree)
    }

    /// Gets the path for a schema file.
    fn schema_path(&self, schema_name: &str) -> PathBuf {
        self.schemas_dir.join(format!("{}.json", schema_name))
    }

    /// Persist all schema load states to the sled tree
    fn persist_states(&self) -> Result<(), SchemaError> {
        info!("Persisting schema states to sled tree");
        let available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        // Remove stale entries
        for key in self.schema_states_tree.iter().keys() {
            let k = key?;
            if let Ok(name) = std::str::from_utf8(&k) {
                if !available.contains_key(name) {
                    self.schema_states_tree.remove(name)?;
                }
            }
        }

        for (name, (_, state)) in available.iter() {
            let bytes = serde_json::to_vec(state)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize state: {}", e)))?;
            self.schema_states_tree.insert(name.as_bytes(), bytes)?;
        }
        self.schema_states_tree.flush()?;
        info!("Schema states persisted successfully");
        Ok(())
    }

    /// Load schema states from the sled tree
    fn load_states(&self) -> HashMap<String, SchemaState> {
        let mut map = HashMap::new();
        for item in self.schema_states_tree.iter().flatten() {
            if let Ok(name) = String::from_utf8(item.0.to_vec()) {
                if let Ok(state) = serde_json::from_slice::<SchemaState>(&item.1) {
                    map.insert(name, state);
                }
            }
        }
        map
    }

    /// Persists a schema to disk.
    fn persist_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        let path = self.schema_path(&schema.name);

        info!("Persisting schema '{}' to {}", schema.name, path.display());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to create schema directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(schema)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize schema: {}", e)))?;

        fs::write(&path, json).map_err(|e| {
            SchemaError::InvalidData(format!(
                "Failed to write schema file: {}, path: {}",
                e,
                path.to_string_lossy()
            ))
        })?;

        info!("Schema '{}' persisted to disk", schema.name);

        Ok(())
    }

    fn fix_transform_outputs(&self, schema: &mut Schema) {
        for (field_name, field) in schema.fields.iter_mut() {
            if let Some(transform) = field.transform() {
                let out_schema = transform.get_output();
                if out_schema.starts_with("test.") {
                    let mut new_transform = (*transform).clone();
                    new_transform.set_output(format!("{}.{}", schema.name, field_name));
                    field.set_transform(new_transform);
                }
            }
        }
    }

    /// Loads a schema into the manager and persists it to disk.
    pub fn load_schema(&self, mut schema: Schema) -> Result<(), SchemaError> {
        info!("Loading schema '{}'", schema.name);

        // Ensure any transforms on fields have the correct output schema
        self.fix_transform_outputs(&mut schema);

        // Persist the updated schema
        self.persist_schema(&schema)?;

        // Then add it to memory
        let name = schema.name.clone();
        {
            let mut loaded = self
                .schemas
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            loaded.insert(name.clone(), schema.clone());
        }
        {
            let mut all = self
                .available
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
            all.insert(name.clone(), (schema, SchemaState::Loaded));
        }

        // Persist state changes
        self.persist_states()?;
        info!("Schema '{}' loaded and state persisted", name);

        Ok(())
    }


    /// Retrieves a schema by name.
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.get(schema_name).cloned())
    }

    /// Updates the ref_atom_uuid for a specific field in a schema.
    pub fn update_field_ref_atom_uuid(&self, schema_name: &str, field_name: &str, ref_atom_uuid: String) -> Result<(), SchemaError> {
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        
        if let Some(schema) = schemas.get_mut(schema_name) {
            if let Some(field) = schema.fields.get_mut(field_name) {
                field.set_ref_atom_uuid(ref_atom_uuid);
                Ok(())
            } else {
                Err(SchemaError::InvalidField(format!("Field {} not found in schema {}", field_name, schema_name)))
            }
        } else {
            Err(SchemaError::NotFound(format!("Schema {} not found", schema_name)))
        }
    }

    /// Lists all schema names currently loaded.
    pub fn list_loaded_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.keys().cloned().collect())
    }

    /// Lists all schemas available on disk and their state.
    pub fn list_available_schemas(&self) -> Result<Vec<String>, SchemaError> {
        let available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(available.keys().cloned().collect())
    }

    /// Retrieve the persisted state for a schema if known.
    pub fn get_schema_state(&self, schema_name: &str) -> Option<SchemaState> {
        let available = self.available.lock().ok()?;
        available.get(schema_name).map(|(_, s)| *s)
    }

    /// Backwards compatible method for listing loaded schemas.
    pub fn list_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.list_loaded_schemas()
    }

    /// Checks if a schema exists in the manager.
    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        Ok(schemas.contains_key(schema_name))
    }

    /// Mark a schema as unloaded but keep it available in memory
    pub fn set_unloaded(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Unloading schema '{}'", schema_name);
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        schemas.remove(schema_name);
        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        if let Some((_, state)) = available.get_mut(schema_name) {
            *state = SchemaState::Unloaded;
            // persist state change
            drop(available);
            self.persist_states()?;
            info!("Schema '{}' marked as unloaded", schema_name);
            Ok(())
        } else {
            Err(SchemaError::NotFound(format!("Schema {schema_name} not found")))
        }
    }

    /// Unload a schema from memory without deleting its persisted file.
    pub fn unload_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        self.set_unloaded(schema_name)
    }

    /// Loads all schema files from the schemas directory and marks them as loaded
    /// if their persisted state is `Loaded`.
    pub fn load_schemas_from_disk(&self) -> Result<(), SchemaError> {
        let states = self.load_states();
        info!("Loading schemas from {}", self.schemas_dir.display());
        if let Ok(entries) = std::fs::read_dir(&self.schemas_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                        if schema_opt.is_none() {
                            if let Ok(json_schema) = serde_json::from_str::<JsonSchemaDefinition>(&contents) {
                                if let Ok(schema) = self.interpret_schema(json_schema) {
                                    schema_opt = Some(schema);
                                }
                            }
                        }
                        if let Some(mut schema) = schema_opt {
                            self.fix_transform_outputs(&mut schema);
                            let name = schema.name.clone();
                            let state = states.get(&name).copied().unwrap_or(SchemaState::Loaded);
                            {
                                let mut available = self.available.lock().map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
                                available.insert(name.clone(), (schema.clone(), state));
                            }
                            if state == SchemaState::Loaded {
                                let mut loaded = self.schemas.lock().map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
                                loaded.insert(name.clone(), schema);
                            }
                            info!("Loaded schema '{}' from disk", name);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads only schema states from disk without populating the loaded schema map.
    pub fn load_schema_states_from_disk(&self) -> Result<(), SchemaError> {
        let states = self.load_states();
        let mut available = self
            .available
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;
        for (name, state) in states {
            available.insert(name.clone(), (Schema::new(name), state));
        }
        Ok(())
    }

    /// Maps fields between schemas based on their defined relationships.
    /// Returns a list of AtomRefs that need to be persisted in FoldDB.
    pub fn map_fields(&self, schema_name: &str) -> Result<Vec<AtomRef>, SchemaError> {
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        // First collect all the source field ref_atom_uuids we need
        let mut field_mappings = Vec::new();
        if let Some(schema) = schemas.get(schema_name) {
            for (field_name, field) in &schema.fields {
                for (source_schema_name, source_field_name) in field.field_mappers() {
                    if let Some(source_schema) = schemas.get(source_schema_name) {
                        if let Some(source_field) = source_schema.fields.get(source_field_name) {
                            if let Some(ref_atom_uuid) = source_field.ref_atom_uuid() {
                                field_mappings.push((field_name.clone(), ref_atom_uuid.clone()));
                            }
                        }
                    }
                }
            }
        }
        drop(schemas); // Release the immutable lock

        // Now get a mutable lock to update the fields
        let mut schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

        let schema = schemas
            .get_mut(schema_name)
            .ok_or_else(|| SchemaError::InvalidData(format!("Schema {schema_name} not found")))?;

        // Apply the collected mappings
        for (field_name, ref_atom_uuid) in field_mappings {
            if let Some(field) = schema.fields.get_mut(&field_name) {
                field.set_ref_atom_uuid(ref_atom_uuid);
            }
        }

        let mut atom_refs = Vec::new();

        // For unmapped fields, create a new ref_atom_uuid and AtomRef
        for field in schema.fields.values_mut() {
            if field.ref_atom_uuid().is_none() {
                let ref_atom_uuid = Uuid::new_v4().to_string();

                // Create a new AtomRef for this field
                let atom_ref = match field {
                    FieldVariant::Collection(_) => {
                        // For collection fields, we'll create a placeholder AtomRef
                        // The actual collection will be created when data is added
                        AtomRef::new(Uuid::new_v4().to_string(), "system".to_string())
                    }
                    _ => {
                        // For single fields, create a normal AtomRef
                        AtomRef::new(Uuid::new_v4().to_string(), "system".to_string())
                    }
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

    /// Converts a JSON schema field to a FieldVariant.
    fn convert_field(json_field: JsonSchemaField) -> FieldVariant {
        let mut single_field = SingleField::new(
            json_field.permission_policy.into(),
            json_field.payment_config.into(),
            json_field.field_mappers,
        );
        
        if let Some(ref_atom_uuid) = json_field.ref_atom_uuid {
            single_field.set_ref_atom_uuid(ref_atom_uuid);
        }
        
        // Add transform if present
        if let Some(json_transform) = json_field.transform {
            single_field.set_transform(json_transform.into());
        }
        
        // For now, we'll create all fields as Single fields
        // TODO: Handle Collection and Range field types based on json_field.field_type
        FieldVariant::Single(single_field)
    }

    /// Interprets a JSON schema definition and converts it to a Schema.
    pub fn interpret_schema(
        &self,
        json_schema: JsonSchemaDefinition,
    ) -> Result<Schema, SchemaError> {
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
    use crate::fees::FieldPaymentConfig;

    use crate::permissions::types::policy::PermissionsPolicy;
    use crate::schema::types::field::FieldType;
    use crate::schema::types::{JsonSchemaDefinition, JsonSchemaField};
    use crate::schema::types::json_schema::{JsonFieldPaymentConfig, JsonPermissionPolicy};
    use crate::fees::{SchemaPaymentConfig, TrustDistanceScaling};
    use crate::permissions::types::policy::TrustDistance;
    use std::fs;

    fn cleanup_test_schema(name: &str) {
        let path = PathBuf::from("data/schemas").join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }

    fn create_test_field(
        ref_atom_uuid: Option<String>,
        field_mappers: HashMap<String, String>,
    ) -> FieldVariant {
        let mut single_field = SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            field_mappers,
        );
        if let Some(uuid) = ref_atom_uuid {
            single_field.set_ref_atom_uuid(uuid);
        }
        FieldVariant::Single(single_field)
    }

    fn build_json_schema(name: &str) -> JsonSchemaDefinition {
        let permission_policy = JsonPermissionPolicy {
            read: TrustDistance::Distance(0),
            write: TrustDistance::Distance(0),
            explicit_read: None,
            explicit_write: None,
        };
        let field = JsonSchemaField {
            permission_policy,
            ref_atom_uuid: Some("uuid".to_string()),
            payment_config: JsonFieldPaymentConfig {
                base_multiplier: 1.0,
                trust_distance_scaling: TrustDistanceScaling::None,
                min_payment: None,
            },
            field_mappers: HashMap::new(),
            field_type: FieldType::Single,
            transform: None,
        };
        let mut fields = HashMap::new();
        fields.insert("field".to_string(), field);
        JsonSchemaDefinition {
            name: name.to_string(),
            fields,
            payment_config: SchemaPaymentConfig::default(),
        }
    }

    #[test]
    fn test_schema_persistence() {
        let test_schema_name = "test_persistence_schema";
        cleanup_test_schema(test_schema_name); // Cleanup any leftover test files

        let core = SchemaCore::new("data").unwrap();

        // Create a test schema
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let schema = Schema::new(test_schema_name.to_string()).with_fields(fields);

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
            loaded_schema
                .fields
                .get("test_field")
                .unwrap()
                .ref_atom_uuid(),
            Some(&"test_uuid".to_string())
        );

        // Unload schema should keep file on disk
        core.unload_schema(test_schema_name).unwrap();
        assert!(schema_path.exists());

        cleanup_test_schema(test_schema_name);
    }

    #[test]
    fn test_map_fields_success() {
        let core = SchemaCore::new("data").unwrap();

        // Create source schema with a field that has a ref_atom_uuid
        let mut source_fields = HashMap::new();
        source_fields.insert(
            "source_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let source_schema = Schema::new("source_schema".to_string()).with_fields(source_fields);
        core.load_schema(source_schema).unwrap();

        // Create target schema with a field that maps to the source field
        let mut field_mappers = HashMap::new();
        field_mappers.insert("source_schema".to_string(), "source_field".to_string());
        let mut target_fields = HashMap::new();
        target_fields.insert(
            "target_field".to_string(),
            create_test_field(None, field_mappers),
        );
        let target_schema = Schema::new("target_schema".to_string()).with_fields(target_fields);
        core.load_schema(target_schema).unwrap();

        // Map fields
        core.map_fields("target_schema").unwrap();

        // Verify the mapping
        let mapped_schema = core.get_schema("target_schema").unwrap().unwrap();
        let mapped_field = mapped_schema.fields.get("target_field").unwrap();
        assert_eq!(
            mapped_field.ref_atom_uuid(),
            Some(&"test_uuid".to_string())
        );
    }

    #[test]
    fn test_validate_schema_valid() {
        let core = SchemaCore::new("data").unwrap();
        let schema = build_json_schema("valid");
        assert!(core.validate_schema(&schema).is_ok());
    }

    #[test]
    fn test_validate_schema_empty_name() {
        let core = SchemaCore::new("data").unwrap();
        let schema = build_json_schema("");
        let result = core.validate_schema(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg == "Schema name cannot be empty"));
    }

    #[test]
    fn test_validate_schema_empty_field_name() {
        let core = SchemaCore::new("data").unwrap();
        let mut schema = build_json_schema("valid");
        let field = schema.fields.remove("field").unwrap();
        schema.fields.insert("".to_string(), field);
        let result = core.validate_schema(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg == "Field name cannot be empty"));
    }

    #[test]
    fn test_validate_schema_invalid_mapper() {
        let core = SchemaCore::new("data").unwrap();
        let mut schema = build_json_schema("valid");
        if let Some(field) = schema.fields.get_mut("field") {
            field.field_mappers.insert(String::new(), "v".to_string());
        }
        let result = core.validate_schema(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg.contains("invalid field mapper")));
    }

    #[test]
    fn test_validate_schema_min_payment_zero() {
        let core = SchemaCore::new("data").unwrap();
        let mut schema = build_json_schema("valid");
        if let Some(field) = schema.fields.get_mut("field") {
            field.payment_config.min_payment = Some(0);
        }
        let result = core.validate_schema(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg.contains("min_payment cannot be zero")));
    }
}
