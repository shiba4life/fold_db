use super::mapper::MappingRule;
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
    /// Returns a `SchemaError::MappingError` if the schema lock cannot be acquired.
    pub fn load_schema(&self, schema: Schema) -> Result<(), SchemaError> {
        self.schemas
            .lock()
            .map_err(|_| SchemaError::MappingError("Failed to acquire schema lock".to_string()))?
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
            .map_err(|_| SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
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
            .map_err(|_| SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
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
            .map_err(|_| SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
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
            .map_err(|_| SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;
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
        self.load_schema(schema)
    }

    fn validate_mapping_rules(rules: &[MappingRule]) -> Result<(), SchemaError> {
        let mut field_operations: HashMap<String, &MappingRule> = HashMap::new();

        for rule in rules {
            let field_name = match rule {
                MappingRule::Map { source_field, .. }
                | MappingRule::Rename { source_field, .. } => source_field,
                MappingRule::Drop { field } => field,
            };

            if let Some(existing_rule) = field_operations.get(field_name) {
                return Err(SchemaError::MappingError(
                    format!("Conflicting rules for field {field_name}: Cannot apply {rule:?} after {existing_rule:?}")
                ));
            }

            field_operations.insert(field_name.clone(), rule);
        }

        Ok(())
    }

    /// Applies schema mappers to transform schemas.
    ///
    /// # Errors
    /// Returns:
    /// - `SchemaError::MappingError` if the schema lock cannot be acquired or if there are conflicting mapping rules
    /// - `SchemaError::NotFound` if source or target schemas are not found
    /// - `SchemaError::InvalidField` if source or target fields are not found
    pub fn apply_schema_mappers(&self, schema: &Schema) -> Result<(), SchemaError> {
        // First get a read lock to validate and prepare changes
        let schemas = self
            .schemas
            .lock()
            .map_err(|_| SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;

        let mut changes = Vec::new();

        // Collect all changes without holding the lock
        for mapper in &schema.schema_mappers {
            Self::validate_mapping_rules(&mapper.rules)?;

            let source_schema = schemas
                .get(&mapper.source_schema_name)
                .ok_or_else(|| {
                    SchemaError::NotFound(format!(
                        "Source schema not found: {source_schema_name}",
                        source_schema_name = mapper.source_schema_name
                    ))
                })?
                .clone();

            let mut target_schema = schemas
                .get(&mapper.target_schema_name)
                .ok_or_else(|| {
                    SchemaError::NotFound(format!(
                        "Target schema not found: {target_schema_name}",
                        target_schema_name = mapper.target_schema_name
                    ))
                })?
                .clone();

            for rule in &mapper.rules {
                match rule {
                    MappingRule::Rename {
                        source_field,
                        target_field,
                    } => {
                        let source_field_value =
                            source_schema.fields.get(source_field).ok_or_else(|| {
                                SchemaError::InvalidField(format!(
                                    "Source field not found: {source_field}"
                                ))
                            })?;

                        if !target_schema.fields.contains_key(target_field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Target field not found: {target_field}"
                            )));
                        }

                        let field = source_field_value.clone();
                        target_schema.fields.insert(target_field.clone(), field);
                    }
                    MappingRule::Drop { field } => {
                        if !target_schema.fields.contains_key(field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Field to drop not found: {field}"
                            )));
                        }
                        target_schema.fields.remove(field);
                    }
                    MappingRule::Map {
                        source_field,
                        target_field,
                        function,
                    } => {
                        let source_field_value =
                            source_schema.fields.get(source_field).ok_or_else(|| {
                                SchemaError::InvalidField(format!(
                                    "Source field not found: {source_field}"
                                ))
                            })?;

                        if !target_schema.fields.contains_key(target_field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Target field not found: {target_field}"
                            )));
                        }

                        let mut field = source_field_value.clone();
                        if let Some(func_name) = function {
                            match func_name.as_str() {
                                "to_lowercase" => {
                                    // Apply lowercase transformation to the field's atom value
                                    // This will be handled by the query system when retrieving the value
                                    field.ref_atom_uuid = format!("lowercase:{}", field.ref_atom_uuid);
                                }
                                _ => return Err(SchemaError::MappingError(
                                    format!("Unknown mapping function: {func_name}")
                                )),
                            }
                        }
                        target_schema.fields.insert(target_field.clone(), field);
                    }
                }
            }

            changes.push((target_schema.name.clone(), target_schema));
        }

        // Drop the read lock
        drop(schemas);

        // Apply changes with a write lock
        {
            let mut schemas = self.schemas.lock().map_err(|_| {
                SchemaError::MappingError("Failed to acquire schema lock".to_string())
            })?;
            for (name, schema) in changes {
                schemas.insert(name, schema);
            }
            // Lock is dropped at end of scope
        }

        Ok(())
    }
}
