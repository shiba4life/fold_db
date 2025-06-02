//! Mutation generator for creating mutations from AI responses and JSON data

use crate::ingestion::{IngestionError, IngestionResult};
use crate::schema::types::{Mutation, MutationType};
use serde_json::Value;
use std::collections::HashMap;
use log::{info, warn, debug};

/// Service for generating mutations from AI responses and JSON data
pub struct MutationGenerator;

impl MutationGenerator {
    /// Create a new mutation generator
    pub fn new() -> Self {
        Self
    }

    /// Generate mutations from JSON data and mutation mappers
    pub fn generate_mutations(
        &self,
        schema_name: &str,
        json_data: &Value,
        mutation_mappers: &HashMap<String, String>,
        trust_distance: u32,
        pub_key: String,
    ) -> IngestionResult<Vec<Mutation>> {
        info!("Generating mutations for schema '{}' with {} mappers", schema_name, mutation_mappers.len());
        
        let mut mutations = Vec::new();
        let mut fields_and_values = HashMap::new();

        // Process each mutation mapper
        for (json_path, schema_path) in mutation_mappers {
            debug!("Processing mapper: {} -> {}", json_path, schema_path);
            
            // Extract value from JSON using the path
            match self.extract_value_from_json_path(json_data, json_path) {
                Ok(Some(value)) => {
                    // Parse the schema path to get the field name
                    let field_name = self.parse_schema_field_path(schema_path)?;
                    info!("Mapped {} = {:?} to field {}", json_path, value, field_name);
                    fields_and_values.insert(field_name, value);
                }
                Ok(None) => {
                    warn!("No value found at JSON path: {}", json_path);
                }
                Err(e) => {
                    warn!("Failed to extract value from path '{}': {}", json_path, e);
                }
            }
        }

        // If we have fields to mutate, create a mutation
        if !fields_and_values.is_empty() {
            let mutation = Mutation {
                schema_name: schema_name.to_string(),
                fields_and_values,
                pub_key,
                trust_distance,
                mutation_type: MutationType::Create,
            };
            mutations.push(mutation);
            info!("Created mutation with {} fields", mutations[0].fields_and_values.len());
        } else {
            warn!("No valid field mappings found, no mutations generated");
        }

        Ok(mutations)
    }

    /// Extract value from JSON using a dot-notation path
    fn extract_value_from_json_path(&self, json_data: &Value, path: &str) -> IngestionResult<Option<Value>> {
        let path_parts = self.parse_json_path(path)?;
        let mut current = json_data;

        for part in &path_parts {
            match part {
                JsonPathPart::Field(field_name) => {
                    if let Value::Object(obj) = current {
                        current = obj.get(field_name).unwrap_or(&Value::Null);
                    } else {
                        return Ok(None);
                    }
                }
                JsonPathPart::Index(index) => {
                    if let Value::Array(arr) = current {
                        current = arr.get(*index).unwrap_or(&Value::Null);
                    } else {
                        return Ok(None);
                    }
                }
            }

            if current == &Value::Null {
                return Ok(None);
            }
        }

        Ok(Some(current.clone()))
    }

    /// Parse a JSON path into components
    fn parse_json_path(&self, path: &str) -> IngestionResult<Vec<JsonPathPart>> {
        let mut parts = Vec::new();
        let mut current_field = String::new();
        let mut chars = path.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '.' => {
                    if !current_field.is_empty() {
                        parts.push(JsonPathPart::Field(current_field.clone()));
                        current_field.clear();
                    }
                }
                '[' => {
                    // Handle array index
                    if !current_field.is_empty() {
                        parts.push(JsonPathPart::Field(current_field.clone()));
                        current_field.clear();
                    }

                    let mut index_str = String::new();
                    for index_ch in chars.by_ref() {
                        if index_ch == ']' {
                            break;
                        }
                        index_str.push(index_ch);
                    }

                    let index = index_str.parse::<usize>()
                        .map_err(|_| IngestionError::path_parsing_error(
                            format!("Invalid array index '{}' in path '{}'", index_str, path)
                        ))?;
                    
                    parts.push(JsonPathPart::Index(index));
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }

        if !current_field.is_empty() {
            parts.push(JsonPathPart::Field(current_field));
        }

        Ok(parts)
    }

    /// Parse schema field path to extract the field name
    fn parse_schema_field_path(&self, schema_path: &str) -> IngestionResult<String> {
        // Expected format: "schema.field" or "schema.field[\"key\"]"
        
        // Split on the first dot to separate schema name from field path
        let parts: Vec<&str> = schema_path.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(IngestionError::field_mapping_error(
                format!("Invalid schema path format '{}', expected 'schema.field'", schema_path)
            ));
        }

        let field_part = parts[1];
        
        // Handle array/object access notation like field["key"]
        if let Some(bracket_pos) = field_part.find('[') {
            Ok(field_part[..bracket_pos].to_string())
        } else {
            Ok(field_part.to_string())
        }
    }

    /// Generate mutations for collection fields (arrays)
    pub fn generate_collection_mutations(
        &self,
        schema_name: &str,
        json_data: &Value,
        mutation_mappers: &HashMap<String, String>,
        trust_distance: u32,
        pub_key: String,
    ) -> IngestionResult<Vec<Mutation>> {
        info!("Generating collection mutations for schema '{}'", schema_name);
        
        let mut mutations = Vec::new();

        // Group mappers by collection field
        let mut collection_groups: HashMap<String, Vec<(String, String)>> = HashMap::new();
        
        for (json_path, schema_path) in mutation_mappers {
            // Check if this is a collection field (contains array index)
            if json_path.contains('[') && json_path.contains(']') {
                let _base_path = self.get_base_collection_path(json_path)?;
                let field_name = self.parse_schema_field_path(schema_path)?;
                
                collection_groups
                    .entry(field_name)
                    .or_default()
                    .push((json_path.clone(), schema_path.clone()));
            }
        }

        // Generate mutations for each collection
        for (field_name, mappers) in collection_groups {
            let collection_mutations = self.generate_collection_field_mutations(
                schema_name,
                &field_name,
                json_data,
                &mappers,
                trust_distance,
                pub_key.clone(),
            )?;
            mutations.extend(collection_mutations);
        }

        Ok(mutations)
    }

    /// Get the base path for a collection (remove array indices)
    fn get_base_collection_path(&self, path: &str) -> IngestionResult<String> {
        // Remove all array indices from the path
        let mut result = String::new();
        let chars = path.chars();
        let mut skip_until_bracket = false;

        for ch in chars {
            if ch == '[' {
                skip_until_bracket = true;
            } else if ch == ']' {
                skip_until_bracket = false;
            } else if !skip_until_bracket {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Generate mutations for a specific collection field
    fn generate_collection_field_mutations(
        &self,
        schema_name: &str,
        _field_name: &str,
        json_data: &Value,
        mappers: &[(String, String)],
        trust_distance: u32,
        pub_key: String,
    ) -> IngestionResult<Vec<Mutation>> {
        let mut mutations = Vec::new();

        // Extract the collection data
        let base_path = if let Some((first_path, _)) = mappers.first() {
            self.get_base_collection_path(first_path)?
        } else {
            return Ok(mutations);
        };

        if let Ok(Some(Value::Array(collection_data))) = self.extract_value_from_json_path(json_data, &base_path) {
            // Create a mutation for each item in the collection
            for (index, _item) in collection_data.iter().enumerate() {
                let mut fields_and_values = HashMap::new();
                
                // For each mapper, extract the value for this specific item
                for (json_path, schema_path) in mappers {
                    // Replace the array index in the path with the current index
                    let item_path = self.replace_array_index_in_path(json_path, index)?;
                    
                    if let Ok(Some(value)) = self.extract_value_from_json_path(json_data, &item_path) {
                        let field_name = self.parse_schema_field_path(schema_path)?;
                        fields_and_values.insert(field_name, value);
                    }
                }

                if !fields_and_values.is_empty() {
                    let mutation = Mutation {
                        schema_name: schema_name.to_string(),
                        fields_and_values,
                        pub_key: pub_key.clone(),
                        trust_distance,
                        mutation_type: MutationType::AddToCollection(index.to_string()),
                    };
                    mutations.push(mutation);
                }
            }
        }

        Ok(mutations)
    }

    /// Replace array index in a path with a specific index
    fn replace_array_index_in_path(&self, path: &str, new_index: usize) -> IngestionResult<String> {
        let mut result = String::new();
        let mut chars = path.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '[' {
                result.push(ch);
                // Skip the existing index
                for index_ch in chars.by_ref() {
                    if index_ch == ']' {
                        result.push_str(&new_index.to_string());
                        result.push(index_ch);
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

/// Parts of a JSON path
#[derive(Debug, Clone)]
enum JsonPathPart {
    Field(String),
    Index(usize),
}

impl Default for MutationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_path() {
        let generator = MutationGenerator::new();
        
        let result = generator.parse_json_path("user.name").unwrap();
        assert_eq!(result.len(), 2);
        
        let result = generator.parse_json_path("items[0].price").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_extract_value_from_json_path() {
        let generator = MutationGenerator::new();
        let json_data = json!({
            "user": {
                "name": "John",
                "items": [
                    {"price": 10.0},
                    {"price": 20.0}
                ]
            }
        });

        let result = generator.extract_value_from_json_path(&json_data, "user.name").unwrap();
        assert_eq!(result, Some(json!("John")));

        let result = generator.extract_value_from_json_path(&json_data, "user.items[0].price").unwrap();
        assert_eq!(result, Some(json!(10.0)));
    }

    #[test]
    fn test_parse_schema_field_path() {
        let generator = MutationGenerator::new();
        
        let result = generator.parse_schema_field_path("UserSchema.name").unwrap();
        assert_eq!(result, "name");

        let result = generator.parse_schema_field_path("ProductSchema.tags[\"category\"]").unwrap();
        assert_eq!(result, "tags");
    }

    #[test]
    fn test_generate_mutations() {
        let generator = MutationGenerator::new();
        let json_data = json!({
            "name": "John",
            "age": 30
        });

        let mut mappers = HashMap::new();
        mappers.insert("name".to_string(), "UserSchema.name".to_string());
        mappers.insert("age".to_string(), "UserSchema.age".to_string());

        let result = generator.generate_mutations(
            "UserSchema",
            &json_data,
            &mappers,
            0,
            "test-key".to_string(),
        ).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].fields_and_values.len(), 2);
    }
}