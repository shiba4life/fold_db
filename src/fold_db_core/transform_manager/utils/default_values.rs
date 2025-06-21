use serde_json::Value as JsonValue;
use log::{info, warn};

use super::TransformUtils;

impl TransformUtils {
    /// Enhanced default value generation with comprehensive field mapping
    pub fn get_default_value_for_field(field_name: &str) -> JsonValue {
        info!("📊 Generating default value for field: {}", field_name);

        let default_value = match field_name {
            "input1" => JsonValue::Number(serde_json::Number::from(42)),
            "input2" => JsonValue::Number(serde_json::Number::from(24)),
            "value1" => JsonValue::Number(serde_json::Number::from(5)),
            "value2" => JsonValue::Number(serde_json::Number::from(10)),
            "weight" => JsonValue::Number(serde_json::Number::from(70)),
            "height" => JsonValue::Number(serde_json::Number::from_f64(1.75).unwrap_or(serde_json::Number::from(175))),
            "age" => JsonValue::Number(serde_json::Number::from(30)),
            "id" | "user_id" | "patient_id" => JsonValue::String("default_id".to_string()),
            "name" | "username" | "patient_name" => JsonValue::String("default_name".to_string()),
            "active" | "enabled" | "is_valid" => JsonValue::Bool(true),
            "disabled" | "inactive" | "is_deleted" => JsonValue::Bool(false),
            "score" | "rating" => JsonValue::Number(serde_json::Number::from(0)),
            "count" | "quantity" => JsonValue::Number(serde_json::Number::from(1)),
            "price" | "amount" => JsonValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
            _ => Self::infer_default_by_name_pattern(field_name),
        };

        info!("📊 Default value for '{}': {}", field_name, default_value);
        default_value
    }

    /// Infer default value based on field name patterns
    fn infer_default_by_name_pattern(field_name: &str) -> JsonValue {
        let lower_name = field_name.to_lowercase();

        match true {
            _ if lower_name.contains("count") || lower_name.contains("number") || lower_name.contains("value") => {
                JsonValue::Number(serde_json::Number::from(0))
            }
            _ if lower_name.contains("active") || lower_name.contains("enabled") || lower_name.contains("valid") => {
                JsonValue::Bool(false)
            }
            _ if lower_name.contains("list") || lower_name.contains("array") || lower_name.contains("tags") => {
                JsonValue::Array(vec![])
            }
            _ if lower_name.contains("config") || lower_name.contains("meta") || lower_name.contains("data") => {
                JsonValue::Object(serde_json::Map::new())
            }
            _ => JsonValue::String("default".to_string()),
        }
    }

    /// Infer type from field name for smart defaults
    pub fn infer_type_from_field_name(field_name: &str) -> &'static str {
        let lower_name = field_name.to_lowercase();
        match true {
            _ if lower_name.contains("id") || lower_name.contains("name") || lower_name.contains("email") => "string",
            _ if lower_name.contains("count") || lower_name.contains("age") || lower_name.contains("score") => "integer",
            _ if lower_name.contains("weight") || lower_name.contains("height") || lower_name.contains("price") => "float",
            _ if lower_name.contains("active") || lower_name.contains("enabled") || lower_name.contains("valid") => "boolean",
            _ if lower_name.contains("list") || lower_name.contains("items") || lower_name.contains("tags") => "array",
            _ if lower_name.contains("config") || lower_name.contains("settings") || lower_name.contains("meta") => "object",
            _ => "string",
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_value_helper() {
        assert_eq!(TransformUtils::get_default_value_for_field("input1"), JsonValue::Number(serde_json::Number::from(42)));
        assert_eq!(TransformUtils::get_default_value_for_field("active"), JsonValue::Bool(true));
    }

    #[test]
    fn test_infer_type_from_field_name() {
        assert_eq!(TransformUtils::infer_type_from_field_name("user_id"), "string");
        assert_eq!(TransformUtils::infer_type_from_field_name("age_count"), "integer");
        assert_eq!(TransformUtils::infer_type_from_field_name("weight_value"), "float");
        assert_eq!(TransformUtils::infer_type_from_field_name("is_active"), "boolean");
    }
}

