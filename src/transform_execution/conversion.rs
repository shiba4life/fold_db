//! Centralized input/output conversion utilities for transform execution.
//!
//! This module consolidates all JSON value conversion and validation logic
//! that was previously duplicated across multiple modules.

use crate::schema::types::errors::SchemaError;
use crate::transform::ast::Value;
use log::{info, warn};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Centralized conversion utilities to eliminate duplicate conversion logic.
pub struct ConversionUtils;

impl ConversionUtils {
    /// Converts input values from HashMap<String, JsonValue> to HashMap<String, Value> for interpreter.
    /// 
    /// CONSOLIDATED: Replaces duplicate conversion logic in engine.rs and utils/mod.rs
    pub fn convert_input_values_to_interpreter(input_values: HashMap<String, JsonValue>) -> HashMap<String, Value> {
        let mut variables = HashMap::new();

        for (name, value) in input_values {
            // Handle both schema.field format and regular field names
            variables.insert(name.clone(), Value::from(value.clone()));

            // If the name contains a dot, it's in schema.field format
            if let Some((schema, field)) = name.split_once('.') {
                // Add both schema.field and field entries
                variables.insert(format!("{}.{}", schema, field), Value::from(value.clone()));
                variables.insert(field.to_string(), Value::from(value));
            }
        }

        variables
    }

    /// Converts a result value from interpreter Value to JsonValue.
    /// 
    /// CONSOLIDATED: Replaces duplicate conversion logic
    pub fn convert_interpreter_result_to_json(value: Value) -> Result<JsonValue, SchemaError> {
        Ok(JsonValue::from(value))
    }

    /// Validates and converts JsonValue with type checking.
    /// 
    /// CONSOLIDATED: Replaces validation logic from utils/mod.rs
    pub fn validate_and_convert(
        json_value: JsonValue,
        expected_type: &str,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!(
            "🔄 Converting field '{}' (expected type: {})",
            field_name, expected_type
        );

        let is_valid = match expected_type.to_lowercase().as_str() {
            "string" | "str" => json_value.is_string(),
            "number" | "integer" | "int" => json_value.is_number() && json_value.as_i64().is_some(),
            "float" | "double" => json_value.is_number() && json_value.as_f64().is_some(),
            "boolean" | "bool" => json_value.is_boolean(),
            "array" => json_value.is_array(),
            "object" => json_value.is_object(),
            "null" => json_value.is_null(),
            _ => {
                warn!(
                    "⚠️ Unknown expected type '{}' for field '{}', allowing any type",
                    expected_type, field_name
                );
                true
            }
        };

        if !is_valid {
            let error_msg = format!(
                "Type validation failed for field '{}': expected '{}', got '{:?}'",
                field_name, expected_type, json_value
            );
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!(
            "✅ Successfully validated and converted field '{}'",
            field_name
        );
        Ok(json_value)
    }

    /// Converts string to JsonValue with type inference.
    /// 
    /// CONSOLIDATED: Replaces duplicate type inference logic
    pub fn string_to_json_value(value_str: &str, infer_type: bool) -> JsonValue {
        if !infer_type {
            return JsonValue::String(value_str.to_string());
        }

        // Try to infer the type from the string content
        if value_str.eq_ignore_ascii_case("true") {
            return JsonValue::Bool(true);
        }

        if value_str.eq_ignore_ascii_case("false") {
            return JsonValue::Bool(false);
        }

        if value_str.eq_ignore_ascii_case("null") {
            return JsonValue::Null;
        }

        // Try to parse as integer
        if let Ok(int_val) = value_str.parse::<i64>() {
            return JsonValue::Number(serde_json::Number::from(int_val));
        }

        // Try to parse as float
        if let Ok(float_val) = value_str.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(float_val) {
                return JsonValue::Number(num);
            }
        }

        JsonValue::String(value_str.to_string())
    }

    /// Enhanced default value generation with comprehensive field mapping.
    /// 
    /// CONSOLIDATED: Replaces duplicate default value logic
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
            "height" => JsonValue::Number(
                serde_json::Number::from_f64(1.75).unwrap_or(serde_json::Number::from(175)),
            ),
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

            // Fallback for unknown fields based on heuristics
            _ => Self::infer_default_by_name_pattern(field_name),
        };

        info!("📊 Default value for '{}': {}", field_name, default_value);
        default_value
    }

    /// Infers default value based on field name patterns.
    /// 
    /// CONSOLIDATED: Replaces duplicate pattern matching logic
    fn infer_default_by_name_pattern(field_name: &str) -> JsonValue {
        let lower_name = field_name.to_lowercase();

        match true {
            _ if lower_name.contains("count")
                || lower_name.contains("number")
                || lower_name.contains("value") =>
            {
                JsonValue::Number(serde_json::Number::from(0))
            }
            _ if lower_name.contains("active")
                || lower_name.contains("enabled")
                || lower_name.contains("valid") =>
            {
                JsonValue::Bool(false)
            }
            _ if lower_name.contains("list")
                || lower_name.contains("array")
                || lower_name.contains("tags") =>
            {
                JsonValue::Array(vec![])
            }
            _ if lower_name.contains("config")
                || lower_name.contains("meta")
                || lower_name.contains("data") =>
            {
                JsonValue::Object(serde_json::Map::new())
            }
            _ => JsonValue::String("default".to_string()),
        }
    }

    /// Gets typed default value based on expected type.
    /// 
    /// CONSOLIDATED: Replaces duplicate type-based default generation
    pub fn get_typed_default_value(field_name: &str, expected_type: &str) -> JsonValue {
        info!(
            "📊 Generating typed default value for field: {} (type: {})",
            field_name, expected_type
        );

        let default_value = match expected_type.to_lowercase().as_str() {
            "string" | "str" => JsonValue::String(format!("default_{}", field_name)),
            "number" | "integer" | "int" => JsonValue::Number(serde_json::Number::from(0)),
            "float" | "double" => JsonValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
            "boolean" | "bool" => JsonValue::Bool(false),
            "array" => JsonValue::Array(vec![]),
            "object" => JsonValue::Object(serde_json::Map::new()),
            "null" => JsonValue::Null,
            _ => {
                warn!(
                    "⚠️ Unknown type '{}' for field '{}', using field-based default",
                    expected_type, field_name
                );
                Self::get_default_value_for_field(field_name)
            }
        };

        info!(
            "📊 Typed default value for '{}' ({}): {}",
            field_name, expected_type, default_value
        );
        default_value
    }

    /// Infers type from field name for smart defaults.
    /// 
    /// CONSOLIDATED: Replaces duplicate type inference logic
    pub fn infer_type_from_field_name(field_name: &str) -> &'static str {
        let lower_name = field_name.to_lowercase();
        match true {
            _ if lower_name.contains("id")
                || lower_name.contains("name")
                || lower_name.contains("email") =>
            {
                "string"
            }
            _ if lower_name.contains("count")
                || lower_name.contains("age")
                || lower_name.contains("score") =>
            {
                "integer"
            }
            _ if lower_name.contains("weight")
                || lower_name.contains("height")
                || lower_name.contains("price") =>
            {
                "float"
            }
            _ if lower_name.contains("active")
                || lower_name.contains("enabled")
                || lower_name.contains("valid") =>
            {
                "boolean"
            }
            _ if lower_name.contains("list")
                || lower_name.contains("items")
                || lower_name.contains("tags") =>
            {
                "array"
            }
            _ if lower_name.contains("config")
                || lower_name.contains("settings")
                || lower_name.contains("meta") =>
            {
                "object"
            }
            _ => "string", // Default to string
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_conversion() {
        let mut input = HashMap::new();
        input.insert("test.field".to_string(), JsonValue::String("value".to_string()));
        
        let converted = ConversionUtils::convert_input_values_to_interpreter(input);
        
        assert!(converted.contains_key("test.field"));
        assert!(converted.contains_key("field"));
    }

    #[test]
    fn test_validation() {
        let result = ConversionUtils::validate_and_convert(
            JsonValue::String("test".to_string()),
            "string",
            "test_field"
        );
        assert!(result.is_ok());
        
        let result = ConversionUtils::validate_and_convert(
            JsonValue::String("test".to_string()),
            "number",
            "test_field"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let value = ConversionUtils::get_default_value_for_field("weight");
        assert_eq!(value, JsonValue::Number(serde_json::Number::from(70)));
        
        let typed_value = ConversionUtils::get_typed_default_value("test", "boolean");
        assert_eq!(typed_value, JsonValue::Bool(false));
    }

    #[test]
    fn test_type_inference() {
        assert_eq!(ConversionUtils::infer_type_from_field_name("user_id"), "string");
        assert_eq!(ConversionUtils::infer_type_from_field_name("count"), "integer");
        assert_eq!(ConversionUtils::infer_type_from_field_name("price"), "float");
        assert_eq!(ConversionUtils::infer_type_from_field_name("active"), "boolean");
    }
}