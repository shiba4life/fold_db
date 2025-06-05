//! Conversion utilities for consistent JsonValue→Value conversion logic across the transform system.
//!
//! This module provides unified conversion patterns with consistent error handling
//! to eliminate duplicate conversion code throughout the transform manager.

use crate::schema::types::errors::SchemaError;
use serde_json::Value as JsonValue;
use log::{info, error, warn};

/// Utility for consistent JsonValue to Value conversion patterns
pub struct ConversionHelper;

impl ConversionHelper {
    /// Convert JsonValue to Value with unified error handling
    /// Consolidates the duplicate conversion patterns across multiple files
    pub fn json_to_value(json_value: JsonValue) -> Result<JsonValue, SchemaError> {
        // For now, JsonValue and Value are the same type (serde_json::Value)
        // This method provides a consistent interface for future type changes
        Ok(json_value)
    }

    /// Validate and convert JsonValue with type checking
    /// Combines validation and conversion into a single operation
    pub fn validate_and_convert(
        json_value: JsonValue,
        expected_type: &str,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!("🔄 Converting field '{}' (expected type: {})", field_name, expected_type);
        
        // Validate the type matches expectations
        let is_valid = match expected_type.to_lowercase().as_str() {
            "string" | "str" => json_value.is_string(),
            "number" | "integer" | "int" => json_value.is_number() && json_value.as_i64().is_some(),
            "float" | "double" => json_value.is_number() && json_value.as_f64().is_some(),
            "boolean" | "bool" => json_value.is_boolean(),
            "array" => json_value.is_array(),
            "object" => json_value.is_object(),
            "null" => json_value.is_null(),
            _ => {
                warn!("⚠️ Unknown expected type '{}' for field '{}', allowing any type", expected_type, field_name);
                true
            }
        };

        if !is_valid {
            let error_msg = format!(
                "Type validation failed for field '{}': expected '{}', got '{:?}'",
                field_name, expected_type, json_value
            );
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Successfully validated and converted field '{}'", field_name);
        Ok(json_value)
    }

    /// Convert string to JsonValue with type inference
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

        // Default to string if no other type matches
        JsonValue::String(value_str.to_string())
    }

    /// Extract string value from JsonValue with consistent error handling
    pub fn extract_string_value(json_value: &JsonValue, field_name: &str) -> Result<String, SchemaError> {
        match json_value.as_str() {
            Some(string_value) => {
                info!("📝 Extracted string value from field '{}': '{}'", field_name, string_value);
                Ok(string_value.to_string())
            }
            None => {
                let error_msg = format!("Field '{}' is not a string: {:?}", field_name, json_value);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Extract numeric value from JsonValue with consistent error handling
    pub fn extract_numeric_value(json_value: &JsonValue, field_name: &str) -> Result<f64, SchemaError> {
        match json_value.as_f64() {
            Some(numeric_value) => {
                info!("🔢 Extracted numeric value from field '{}': {}", field_name, numeric_value);
                Ok(numeric_value)
            }
            None => {
                let error_msg = format!("Field '{}' is not a number: {:?}", field_name, json_value);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Extract boolean value from JsonValue with consistent error handling
    pub fn extract_boolean_value(json_value: &JsonValue, field_name: &str) -> Result<bool, SchemaError> {
        match json_value.as_bool() {
            Some(bool_value) => {
                info!("✅ Extracted boolean value from field '{}': {}", field_name, bool_value);
                Ok(bool_value)
            }
            None => {
                let error_msg = format!("Field '{}' is not a boolean: {:?}", field_name, json_value);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Convert between different JsonValue variants with validation
    pub fn convert_json_value_type(
        json_value: JsonValue,
        target_type: &str,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!("🔄 Converting field '{}' to type '{}'", field_name, target_type);

        match target_type.to_lowercase().as_str() {
            "string" | "str" => {
                let string_repr = match json_value {
                    JsonValue::String(s) => s,
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    JsonValue::Null => "null".to_string(),
                    _ => serde_json::to_string(&json_value).map_err(|e| {
                        SchemaError::InvalidData(format!("Failed to convert to string: {}", e))
                    })?,
                };
                Ok(JsonValue::String(string_repr))
            }
            "number" | "integer" | "int" => {
                match json_value {
                    JsonValue::Number(n) => Ok(JsonValue::Number(n)),
                    JsonValue::String(s) => {
                        let parsed: i64 = s.parse().map_err(|e| {
                            SchemaError::InvalidData(format!("Cannot convert '{}' to integer: {}", s, e))
                        })?;
                        Ok(JsonValue::Number(serde_json::Number::from(parsed)))
                    }
                    _ => Err(SchemaError::InvalidData(format!("Cannot convert {:?} to integer", json_value)))
                }
            }
            "float" | "double" => {
                match json_value {
                    JsonValue::Number(n) => Ok(JsonValue::Number(n)),
                    JsonValue::String(s) => {
                        let parsed: f64 = s.parse().map_err(|e| {
                            SchemaError::InvalidData(format!("Cannot convert '{}' to float: {}", s, e))
                        })?;
                        let number = serde_json::Number::from_f64(parsed).ok_or_else(|| {
                            SchemaError::InvalidData(format!("Invalid float value: {}", parsed))
                        })?;
                        Ok(JsonValue::Number(number))
                    }
                    _ => Err(SchemaError::InvalidData(format!("Cannot convert {:?} to float", json_value)))
                }
            }
            "boolean" | "bool" => {
                match json_value {
                    JsonValue::Bool(b) => Ok(JsonValue::Bool(b)),
                    JsonValue::String(s) => {
                        match s.to_lowercase().as_str() {
                            "true" | "1" | "yes" => Ok(JsonValue::Bool(true)),
                            "false" | "0" | "no" => Ok(JsonValue::Bool(false)),
                            _ => Err(SchemaError::InvalidData(format!("Cannot convert '{}' to boolean", s)))
                        }
                    }
                    JsonValue::Number(n) => {
                        if let Some(int_val) = n.as_i64() {
                            Ok(JsonValue::Bool(int_val != 0))
                        } else {
                            Err(SchemaError::InvalidData(format!("Cannot convert number to boolean: {}", n)))
                        }
                    }
                    _ => Err(SchemaError::InvalidData(format!("Cannot convert {:?} to boolean", json_value)))
                }
            }
            _ => {
                warn!("⚠️ Unknown target type '{}', returning original value", target_type);
                Ok(json_value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_to_value() {
        let json_val = json!({"key": "value"});
        let result = ConversionHelper::json_to_value(json_val.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json_val);
    }

    #[test]
    fn test_validate_and_convert_string() {
        let json_val = JsonValue::String("test".to_string());
        let result = ConversionHelper::validate_and_convert(json_val.clone(), "string", "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json_val);
    }

    #[test]
    fn test_validate_and_convert_type_mismatch() {
        let json_val = JsonValue::String("test".to_string());
        let result = ConversionHelper::validate_and_convert(json_val, "number", "test_field");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_json_value_with_inference() {
        assert_eq!(ConversionHelper::string_to_json_value("true", true), JsonValue::Bool(true));
        assert_eq!(ConversionHelper::string_to_json_value("42", true), JsonValue::Number(serde_json::Number::from(42)));
        assert_eq!(ConversionHelper::string_to_json_value("3.14", true), JsonValue::Number(serde_json::Number::from_f64(3.14).unwrap()));
        assert_eq!(ConversionHelper::string_to_json_value("hello", true), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_extract_string_value() {
        let json_val = JsonValue::String("test_value".to_string());
        let result = ConversionHelper::extract_string_value(&json_val, "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");
    }

    #[test]
    fn test_extract_numeric_value() {
        let json_val = JsonValue::Number(serde_json::Number::from_f64(42.5).unwrap());
        let result = ConversionHelper::extract_numeric_value(&json_val, "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.5);
    }

    #[test]
    fn test_convert_json_value_type() {
        // String to number conversion
        let json_val = JsonValue::String("42".to_string());
        let result = ConversionHelper::convert_json_value_type(json_val, "integer", "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Number(serde_json::Number::from(42)));
    }
}