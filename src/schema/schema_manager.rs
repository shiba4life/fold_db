use std::collections::HashMap;
use std::sync::Mutex;
use super::{Schema, SchemaError};  // Updated to use re-exported types
use super::mapper::MappingRule;
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

    fn validate_mapping_rules(rules: &[MappingRule]) -> Result<(), SchemaError> {
        let mut field_operations: HashMap<String, &MappingRule> = HashMap::new();
        
        for rule in rules {
            let field_name = match rule {
                MappingRule::Map { field_name } => field_name,
                MappingRule::Drop { field } => field,
                MappingRule::Rename { source_field, .. } => source_field,
            };
            
            if let Some(existing_rule) = field_operations.get(field_name) {
                return Err(SchemaError::MappingError(format!(
                    "Conflicting rules for field {}: Cannot apply {:?} after {:?}",
                    field_name, rule, existing_rule
                )));
            }
            
            field_operations.insert(field_name.clone(), rule);
        }
        
        Ok(())
    }

    pub fn apply_schema_mappers(&self, schema: &Schema) -> Result<(), SchemaError> {
        let mut schemas = self.schemas.lock().map_err(|_| 
            SchemaError::MappingError("Failed to acquire schema lock".to_string()))?;

        for mapper in &schema.schema_mappers {
            // Validate rules before applying
            Self::validate_mapping_rules(&mapper.rules)?;

            // Get source schema
            let source_schema = schemas.get(&mapper.source_schema_name)
                .ok_or_else(|| SchemaError::NotFound(format!(
                    "Source schema not found: {}", mapper.source_schema_name
                )))?.clone();
            
            // Get target schema
            let mut target_schema = schemas.get(&mapper.target_schema_name)
                .ok_or_else(|| SchemaError::NotFound(format!(
                    "Target schema not found: {}", mapper.target_schema_name
                )))?.clone();

            // Apply mapping rules
            for rule in &mapper.rules {
                match rule {
                    MappingRule::Rename { source_field, target_field } => {
                        let source_field_value = source_schema.fields.get(source_field)
                            .ok_or_else(|| SchemaError::InvalidField(format!(
                                "Source field not found: {}", source_field
                            )))?;
                        
                        if !target_schema.fields.contains_key(target_field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Target field not found: {}", target_field
                            )));
                        }
                        
                        let field = source_field_value.clone();
                        target_schema.fields.insert(target_field.clone(), field);
                    }
                    MappingRule::Drop { field } => {
                        if !target_schema.fields.contains_key(field) {
                            return Err(SchemaError::InvalidField(format!(
                                "Field to drop not found: {}", field
                            )));
                        }
                        target_schema.fields.remove(field);
                    }
                    MappingRule::Map { field_name } => {
                        let source_field_value = source_schema.fields.get(field_name)
                            .ok_or_else(|| SchemaError::InvalidField(format!(
                                "Source field not found: {}", field_name
                            )))?;
                        
                        if !target_schema.fields.contains_key(field_name) {
                            return Err(SchemaError::InvalidField(format!(
                                "Target field not found: {}", field_name
                            )));
                        }
                        
                        let field = source_field_value.clone();
                        target_schema.fields.insert(field_name.clone(), field);
                    }
                }
            }
            
            // Save the modified target schema back to the HashMap
            schemas.insert(target_schema.name.clone(), target_schema);
        }
        Ok(())
    }
}
