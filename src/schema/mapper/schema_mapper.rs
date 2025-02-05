use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::schema::SchemaError;  // Updated to use re-exported type
use super::types::MappingRule;

/// SchemaMapper supports mapping data from multiple source schemas to a target schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaMapper {
    /// List of source schema names
    pub source_schemas: Vec<String>,
    /// Target schema name
    pub target_schema: String,
    /// Mapping rules to apply
    pub rules: Vec<MappingRule>,
}

impl SchemaMapper {
    /// Create a new SchemaMapper
    pub fn new(source_schemas: Vec<String>, target_schema: String, rules: Vec<MappingRule>) -> Self {
        Self {
            source_schemas,
            target_schema,
            rules,
        }
    }

    /// Apply mapping rules to transform data from source schemas to target schema format
    pub fn apply(&self, sources_data: &HashMap<String, Value>) -> Result<Value, SchemaError> {
        let mut merged_target = serde_json::Map::new();

        // Process each source schema in order
        for src in &self.source_schemas {
            if let Some(source_data) = sources_data.get(src) {
                if !source_data.is_object() {
                    return Err(SchemaError::InvalidData(format!("Source data for '{}' must be an object", src)));
                }

                let mut data = source_data.clone();
                let obj = data.as_object_mut().unwrap();
                let mut transformed = serde_json::Map::new();

                // Apply each mapping rule
                for rule in &self.rules {
                    match rule {
                        MappingRule::Rename { source_field, target_field } => {
                            if let Some(val) = obj.remove(source_field) {
                                transformed.insert(target_field.clone(), val);
                            }
                        },
                        MappingRule::Drop { field } => {
                            obj.remove(field);
                        },
                        MappingRule::Add { target_field, value } => {
                            transformed.insert(target_field.clone(), value.clone());
                        },
                        MappingRule::Map { source_field, target_field, function } => {
                            if let Some(val) = obj.get(source_field) {
                                let transformed_val = match function.as_str() {
                                    "to_uppercase" => {
                                        if let Some(s) = val.as_str() {
                                            json!(s.to_uppercase())
                                        } else {
                                            val.clone()
                                        }
                                    },
                                    "to_lowercase" => {
                                        if let Some(s) = val.as_str() {
                                            json!(s.to_lowercase())
                                        } else {
                                            val.clone()
                                        }
                                    },
                                    _ => val.clone(),
                                };
                                transformed.insert(target_field.clone(), transformed_val);
                            }
                        },
                    }
                }

                // We don't merge unmapped fields - only include fields that were explicitly mapped

                // Merge transformed output into overall target
                // Fields from earlier sources have priority
                for (k, v) in transformed.into_iter() {
                    merged_target.entry(k).or_insert(v);
                }
            }
        }

        Ok(Value::Object(merged_target))
    }
}
