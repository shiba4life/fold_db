//! Default value generation utilities for consistent value creation across the transform system.
//!
//! This module provides unified default value generation patterns to eliminate 
//! duplicate hardcoded value logic and provide consistent validation patterns.

use serde_json::Value as JsonValue;
use log::{info, warn};
use crate::schema::types::SchemaError;

/// Utility for generating default values with consistent patterns
pub struct DefaultValueHelper;

impl DefaultValueHelper {
    /// Get default value for a field based on its name with consistent mapping
    pub fn get_default_value_for_field(field_name: &str) -> JsonValue {
        info!("📊 Generating default value for field: {}", field_name);
        
        let default_value = match field_name {
            // Common input field names
            "input1" => JsonValue::Number(serde_json::Number::from(42)),
            "input2" => JsonValue::Number(serde_json::Number::from(24)),
            "value1" => JsonValue::Number(serde_json::Number::from(5)),
            "value2" => JsonValue::Number(serde_json::Number::from(10)),
            
            // Common calculation inputs
            "weight" => JsonValue::Number(serde_json::Number::from(70)),
            "height" => {
                if let Some(num) = serde_json::Number::from_f64(1.75) {
                    JsonValue::Number(num)
                } else {
                    JsonValue::Number(serde_json::Number::from(175)) // fallback to integer
                }
            },
            "age" => JsonValue::Number(serde_json::Number::from(30)),
            
            // Common identifiers
            "id" | "user_id" | "patient_id" => JsonValue::String("default_id".to_string()),
            "name" | "username" | "patient_name" => JsonValue::String("default_name".to_string()),
            
            // Common boolean flags
            "active" | "enabled" | "is_valid" => JsonValue::Bool(true),
            "disabled" | "inactive" | "is_deleted" => JsonValue::Bool(false),
            
            // Common numeric defaults
            "score" | "rating" => JsonValue::Number(serde_json::Number::from(0)),
            "count" | "quantity" => JsonValue::Number(serde_json::Number::from(1)),
            "price" | "amount" => JsonValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
            
            // Fallback for unknown fields
            _ => {
                warn!("⚠️ No specific default value defined for field '{}', using 0", field_name);
                JsonValue::Number(serde_json::Number::from(0))
            }
        };
        
        info!("📊 Default value for '{}': {}", field_name, default_value);
        default_value
    }

    /// Create a default mapping for transform inputs with validation
    pub fn create_default_mapping(
        field_names: &[String],
    ) -> Result<std::collections::HashMap<String, JsonValue>, SchemaError> {
        info!("🔧 Creating default mapping for {} fields", field_names.len());
        
        if field_names.is_empty() {
            warn!("⚠️ Empty field list provided for default mapping creation");
            return Ok(std::collections::HashMap::new());
        }
        
        let mut mapping = std::collections::HashMap::new();
        
        for field_name in field_names {
            if field_name.trim().is_empty() {
                warn!("⚠️ Skipping empty field name in default mapping");
                continue;
            }
            
            let default_value = Self::get_default_value_for_field(field_name);
            mapping.insert(field_name.clone(), default_value);
            info!("📋 Added default mapping: {} -> {}", field_name, mapping[field_name]);
        }
        
        info!("✅ Created default mapping with {} entries", mapping.len());
        Ok(mapping)
    }

    /// Get default value for a field with schema prefix handling
    pub fn get_default_value_with_schema_prefix(full_field_name: &str) -> JsonValue {
        info!("📊 Generating default value for field with schema prefix: {}", full_field_name);
        
        // Extract field name from schema.field format
        let field_name = if let Some(dot_pos) = full_field_name.find('.') {
            &full_field_name[dot_pos + 1..]
        } else {
            full_field_name
        };
        
        Self::get_default_value_for_field(field_name)
    }

    /// Validate and create default values for transform inputs
    pub fn validate_and_create_defaults(
        declared_inputs: &[String],
        analyzed_dependencies: &[String],
    ) -> Result<std::collections::HashMap<String, JsonValue>, SchemaError> {
        info!("🔍 Validating and creating defaults for transform inputs");
        info!("📋 Declared inputs: {:?}", declared_inputs);
        info!("📋 Analyzed dependencies: {:?}", analyzed_dependencies);
        
        // Use declared inputs if available, otherwise use analyzed dependencies
        let inputs_to_process = if declared_inputs.is_empty() {
            info!("📝 No declared inputs, using analyzed dependencies");
            analyzed_dependencies
        } else {
            info!("📝 Using declared inputs");
            declared_inputs
        };
        
        let mut default_mapping = std::collections::HashMap::new();
        
        for input_field in inputs_to_process {
            if input_field.trim().is_empty() {
                warn!("⚠️ Skipping empty input field");
                continue;
            }
            
            let default_value = Self::get_default_value_with_schema_prefix(input_field);
            default_mapping.insert(input_field.clone(), default_value);
            info!("📊 Default value for '{}': {}", input_field, default_mapping[input_field]);
        }
        
        info!("✅ Created validated default mapping with {} entries", default_mapping.len());
        Ok(default_mapping)
    }

    /// Get typed default value based on expected type
    pub fn get_typed_default_value(field_name: &str, expected_type: &str) -> JsonValue {
        info!("📊 Generating typed default value for field: {} (type: {})", field_name, expected_type);
        
        let default_value = match expected_type.to_lowercase().as_str() {
            "string" | "str" => JsonValue::String(format!("default_{}", field_name)),
            "number" | "integer" | "int" => JsonValue::Number(serde_json::Number::from(0)),
            "float" | "double" => JsonValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
            "boolean" | "bool" => JsonValue::Bool(false),
            "array" => JsonValue::Array(vec![]),
            "object" => JsonValue::Object(serde_json::Map::new()),
            "null" => JsonValue::Null,
            _ => {
                warn!("⚠️ Unknown type '{}' for field '{}', using field-based default", expected_type, field_name);
                Self::get_default_value_for_field(field_name)
            }
        };
        
        info!("📊 Typed default value for '{}' ({}): {}", field_name, expected_type, default_value);
        default_value
    }

    /// Check if a field name suggests a specific type
    pub fn infer_type_from_field_name(field_name: &str) -> &'static str {
        match field_name.to_lowercase().as_str() {
            name if name.contains("id") || name.contains("name") || name.contains("email") => "string",
            name if name.contains("count") || name.contains("age") || name.contains("score") => "integer",
            name if name.contains("weight") || name.contains("height") || name.contains("price") => "float",
            name if name.contains("active") || name.contains("enabled") || name.contains("valid") => "boolean",
            name if name.contains("list") || name.contains("items") || name.contains("tags") => "array",
            name if name.contains("config") || name.contains("settings") || name.contains("meta") => "object",
            _ => "string" // Default to string
        }
    }

    /// Create smart defaults based on field name analysis
    pub fn create_smart_defaults(
        field_names: &[String],
    ) -> Result<std::collections::HashMap<String, JsonValue>, SchemaError> {
        info!("🧠 Creating smart defaults for {} fields", field_names.len());
        
        let mut mapping = std::collections::HashMap::new();
        
        for field_name in field_names {
            if field_name.trim().is_empty() {
                continue;
            }
            
            let inferred_type = Self::infer_type_from_field_name(field_name);
            let default_value = Self::get_typed_default_value(field_name, inferred_type);
            mapping.insert(field_name.clone(), default_value);
            
            info!("🧠 Smart default for '{}' (inferred type: {}): {}", 
                field_name, inferred_type, mapping[field_name]);
        }
        
        info!("✅ Created smart defaults mapping with {} entries", mapping.len());
        Ok(mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_value_for_field() {
        // Test known fields
        assert_eq!(
            DefaultValueHelper::get_default_value_for_field("input1"),
            JsonValue::Number(serde_json::Number::from(42))
        );
        
        assert_eq!(
            DefaultValueHelper::get_default_value_for_field("weight"),
            JsonValue::Number(serde_json::Number::from(70))
        );
        
        assert_eq!(
            DefaultValueHelper::get_default_value_for_field("active"),
            JsonValue::Bool(true)
        );
        
        // Test unknown field
        assert_eq!(
            DefaultValueHelper::get_default_value_for_field("unknown_field"),
            JsonValue::Number(serde_json::Number::from(0))
        );
    }

    #[test]
    fn test_create_default_mapping() {
        let fields = vec!["input1".to_string(), "weight".to_string(), "active".to_string()];
        let result = DefaultValueHelper::create_default_mapping(&fields);
        
        assert!(result.is_ok());
        let mapping = result.unwrap();
        assert_eq!(mapping.len(), 3);
        assert!(mapping.contains_key("input1"));
        assert!(mapping.contains_key("weight"));
        assert!(mapping.contains_key("active"));
    }

    #[test]
    fn test_get_default_value_with_schema_prefix() {
        let result = DefaultValueHelper::get_default_value_with_schema_prefix("User.weight");
        assert_eq!(result, JsonValue::Number(serde_json::Number::from(70)));
        
        let result2 = DefaultValueHelper::get_default_value_with_schema_prefix("input1");
        assert_eq!(result2, JsonValue::Number(serde_json::Number::from(42)));
    }

    #[test]
    fn test_get_typed_default_value() {
        assert_eq!(
            DefaultValueHelper::get_typed_default_value("test", "string"),
            JsonValue::String("default_test".to_string())
        );
        
        assert_eq!(
            DefaultValueHelper::get_typed_default_value("test", "boolean"),
            JsonValue::Bool(false)
        );
        
        assert_eq!(
            DefaultValueHelper::get_typed_default_value("test", "array"),
            JsonValue::Array(vec![])
        );
    }

    #[test]
    fn test_infer_type_from_field_name() {
        assert_eq!(DefaultValueHelper::infer_type_from_field_name("user_id"), "string");
        assert_eq!(DefaultValueHelper::infer_type_from_field_name("age_count"), "integer");
        assert_eq!(DefaultValueHelper::infer_type_from_field_name("weight_value"), "float");
        assert_eq!(DefaultValueHelper::infer_type_from_field_name("is_active"), "boolean");
        assert_eq!(DefaultValueHelper::infer_type_from_field_name("items_list"), "array");
        assert_eq!(DefaultValueHelper::infer_type_from_field_name("config_data"), "object");
    }

    #[test]
    fn test_create_smart_defaults() {
        let fields = vec![
            "user_id".to_string(),
            "age_count".to_string(),
            "weight_value".to_string(),
            "is_active".to_string()
        ];
        
        let result = DefaultValueHelper::create_smart_defaults(&fields);
        assert!(result.is_ok());
        
        let mapping = result.unwrap();
        assert_eq!(mapping.len(), 4);
        
        // Check that smart defaults are applied based on field names
        assert!(mapping["user_id"].is_string());
        assert!(mapping["age_count"].is_number());
        assert!(mapping["weight_value"].is_number());
        assert!(mapping["is_active"].is_boolean());
    }
}