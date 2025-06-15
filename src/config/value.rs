//! Configuration value system for format-agnostic configuration storage
//!
//! This module provides the `ConfigValue` enum and related functionality for
//! handling configuration data in a format-agnostic way, with serialization
//! support for TOML and JSON formats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::config::error::{ConfigError, ConfigResult};

/// Format-agnostic configuration value that can represent any configuration data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// Null/None value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Object/table of key-value pairs
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Create a new string value
    pub fn string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
    }

    /// Create a new integer value
    pub fn integer(i: i64) -> Self {
        Self::Integer(i)
    }

    /// Create a new boolean value
    pub fn boolean(b: bool) -> Self {
        Self::Bool(b)
    }

    /// Create a new float value
    pub fn float(f: f64) -> Self {
        Self::Float(f)
    }

    /// Create a new array value
    pub fn array(values: Vec<ConfigValue>) -> Self {
        Self::Array(values)
    }

    /// Create a new object value
    pub fn object(map: HashMap<String, ConfigValue>) -> Self {
        Self::Object(map)
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::Null => "null",
            ConfigValue::Bool(_) => "boolean",
            ConfigValue::Integer(_) => "integer",
            ConfigValue::Float(_) => "float",
            ConfigValue::String(_) => "string",
            ConfigValue::Array(_) => "array",
            ConfigValue::Object(_) => "object",
        }
    }

    /// Check if this is a null value
    pub fn is_null(&self) -> bool {
        matches!(self, ConfigValue::Null)
    }

    /// Try to convert to boolean
    pub fn as_bool(&self) -> ConfigResult<bool> {
        match self {
            ConfigValue::Bool(b) => Ok(*b),
            _ => Err(ConfigError::validation(format!(
                "Expected boolean, found {}", self.type_name()
            ))),
        }
    }

    /// Try to convert to integer
    pub fn as_integer(&self) -> ConfigResult<i64> {
        match self {
            ConfigValue::Integer(i) => Ok(*i),
            ConfigValue::Float(f) if f.fract() == 0.0 => Ok(*f as i64),
            _ => Err(ConfigError::validation(format!(
                "Expected integer, found {}", self.type_name()
            ))),
        }
    }

    /// Try to convert to float
    pub fn as_float(&self) -> ConfigResult<f64> {
        match self {
            ConfigValue::Float(f) => Ok(*f),
            ConfigValue::Integer(i) => Ok(*i as f64),
            _ => Err(ConfigError::validation(format!(
                "Expected float, found {}", self.type_name()
            ))),
        }
    }

    /// Try to convert to string
    pub fn as_string(&self) -> ConfigResult<&str> {
        match self {
            ConfigValue::String(s) => Ok(s),
            _ => Err(ConfigError::validation(format!(
                "Expected string, found {}", self.type_name()
            ))),
        }
    }

    /// Try to convert to array
    pub fn as_array(&self) -> ConfigResult<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Ok(arr),
            _ => Err(ConfigError::validation(format!(
                "Expected array, found {}", self.type_name()
            ))),
        }
    }

    /// Try to convert to object
    pub fn as_object(&self) -> ConfigResult<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(obj) => Ok(obj),
            _ => Err(ConfigError::validation(format!(
                "Expected object, found {}", self.type_name()
            ))),
        }
    }

    /// Get mutable reference to array
    pub fn as_array_mut(&mut self) -> ConfigResult<&mut Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Ok(arr),
            _ => Err(ConfigError::validation(format!(
                "Expected array, found {}", self.type_name()
            ))),
        }
    }

    /// Get mutable reference to object
    pub fn as_object_mut(&mut self) -> ConfigResult<&mut HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(obj) => Ok(obj),
            _ => Err(ConfigError::validation(format!(
                "Expected object, found {}", self.type_name()
            ))),
        }
    }

    /// Get value at object key
    pub fn get(&self, key: &str) -> ConfigResult<&ConfigValue> {
        let obj = self.as_object()?;
        obj.get(key).ok_or_else(|| ConfigError::not_found(format!("Key '{}'", key)))
    }

    /// Get mutable value at object key
    pub fn get_mut(&mut self, key: &str) -> ConfigResult<&mut ConfigValue> {
        let obj = self.as_object_mut()?;
        obj.get_mut(key).ok_or_else(|| ConfigError::not_found(format!("Key '{}'", key)))
    }

    /// Set value at object key
    pub fn set(&mut self, key: String, value: ConfigValue) -> ConfigResult<()> {
        let obj = self.as_object_mut()?;
        obj.insert(key, value);
        Ok(())
    }

    /// Get value at array index
    pub fn get_index(&self, index: usize) -> ConfigResult<&ConfigValue> {
        let arr = self.as_array()?;
        arr.get(index).ok_or_else(|| ConfigError::not_found(format!("Index {}", index)))
    }

    /// Merge this value with another value
    /// Objects are merged recursively, arrays are replaced, scalars are replaced
    pub fn merge(mut self, other: ConfigValue) -> ConfigResult<ConfigValue> {
        match (&mut self, other) {
            (ConfigValue::Object(ref mut obj1), ConfigValue::Object(obj2)) => {
                for (key, value) in obj2 {
                    match obj1.get_mut(&key) {
                        Some(existing) => {
                            let merged = existing.clone().merge(value)?;
                            obj1.insert(key, merged);
                        }
                        None => {
                            obj1.insert(key, value);
                        }
                    }
                }
                Ok(self)
            }
            (_, other_value) => Ok(other_value),
        }
    }

    /// Deep clone this value
    pub fn deep_clone(&self) -> Self {
        self.clone()
    }

    /// Validate this configuration value against a schema
    pub fn validate(&self, _schema: &ConfigValueSchema) -> ConfigResult<()> {
        // TODO: Implement validation logic based on schema
        // For now, just return Ok
        Ok(())
    }

    /// Convert to TOML string
    pub fn to_toml_string(&self) -> ConfigResult<String> {
        // Convert to toml::Value first
        let toml_value = self.to_toml_value()?;
        toml::to_string(&toml_value).map_err(ConfigError::from)
    }

    /// Parse from TOML string
    pub fn from_toml_string(s: &str) -> ConfigResult<Self> {
        let toml_value: toml::Value = toml::from_str(s)?;
        Self::from_toml_value(toml_value)
    }

    /// Convert to toml::Value
    fn to_toml_value(&self) -> ConfigResult<toml::Value> {
        let value = match self {
            ConfigValue::Null => return Err(ConfigError::validation("TOML does not support null values")),
            ConfigValue::Bool(b) => toml::Value::Boolean(*b),
            ConfigValue::Integer(i) => toml::Value::Integer(*i),
            ConfigValue::Float(f) => toml::Value::Float(*f),
            ConfigValue::String(s) => toml::Value::String(s.clone()),
            ConfigValue::Array(arr) => {
                let toml_arr: Result<Vec<_>, _> = arr.iter()
                    .map(|v| v.to_toml_value())
                    .collect();
                toml::Value::Array(toml_arr?)
            }
            ConfigValue::Object(obj) => {
                let mut toml_table = toml::value::Table::new();
                for (k, v) in obj {
                    toml_table.insert(k.clone(), v.to_toml_value()?);
                }
                toml::Value::Table(toml_table)
            }
        };
        Ok(value)
    }

    /// Convert from toml::Value
    fn from_toml_value(value: toml::Value) -> ConfigResult<Self> {
        let config_value = match value {
            toml::Value::Boolean(b) => ConfigValue::Bool(b),
            toml::Value::Integer(i) => ConfigValue::Integer(i),
            toml::Value::Float(f) => ConfigValue::Float(f),
            toml::Value::String(s) => ConfigValue::String(s),
            toml::Value::Array(arr) => {
                let config_arr: Result<Vec<_>, _> = arr.into_iter()
                    .map(Self::from_toml_value)
                    .collect();
                ConfigValue::Array(config_arr?)
            }
            toml::Value::Table(table) => {
                let mut config_obj = HashMap::new();
                for (k, v) in table {
                    config_obj.insert(k, Self::from_toml_value(v)?);
                }
                ConfigValue::Object(config_obj)
            }
            toml::Value::Datetime(_) => {
                return Err(ConfigError::validation("TOML datetime values are not supported"));
            }
        };
        Ok(config_value)
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Null => write!(f, "null"),
            ConfigValue::Bool(b) => write!(f, "{}", b),
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(fl) => write!(f, "{}", fl),
            ConfigValue::String(s) => write!(f, "\"{}\"", s),
            ConfigValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ConfigValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Schema for validating configuration values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValueSchema {
    /// Expected type
    pub value_type: String,
    /// Whether the value is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<ConfigValue>,
    /// Validation rules
    pub validation: HashMap<String, ConfigValue>,
}

impl ConfigValueSchema {
    /// Create a new schema for a string value
    pub fn string(required: bool) -> Self {
        Self {
            value_type: "string".to_string(),
            required,
            default: None,
            validation: HashMap::new(),
        }
    }

    /// Create a new schema for an integer value
    pub fn integer(required: bool) -> Self {
        Self {
            value_type: "integer".to_string(),
            required,
            default: None,
            validation: HashMap::new(),
        }
    }

    /// Create a new schema for a boolean value
    pub fn boolean(required: bool) -> Self {
        Self {
            value_type: "boolean".to_string(),
            required,
            default: None,
            validation: HashMap::new(),
        }
    }

    /// Set default value
    pub fn with_default(mut self, default: ConfigValue) -> Self {
        self.default = Some(default);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_creation() {
        let val = ConfigValue::string("test");
        assert_eq!(val.as_string().unwrap(), "test");
        assert_eq!(val.type_name(), "string");
    }

    #[test]
    fn test_config_value_merge() {
        let mut obj1 = HashMap::new();
        obj1.insert("a".to_string(), ConfigValue::integer(1));
        obj1.insert("b".to_string(), ConfigValue::string("old"));

        let mut obj2 = HashMap::new();
        obj2.insert("b".to_string(), ConfigValue::string("new"));
        obj2.insert("c".to_string(), ConfigValue::boolean(true));

        let val1 = ConfigValue::object(obj1);
        let val2 = ConfigValue::object(obj2);

        let merged = val1.merge(val2).unwrap();
        let merged_obj = merged.as_object().unwrap();

        assert_eq!(merged_obj.get("a").unwrap().as_integer().unwrap(), 1);
        assert_eq!(merged_obj.get("b").unwrap().as_string().unwrap(), "new");
        assert_eq!(merged_obj.get("c").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_toml_serialization() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), ConfigValue::string("test"));
        obj.insert("count".to_string(), ConfigValue::integer(42));
        obj.insert("enabled".to_string(), ConfigValue::boolean(true));

        let val = ConfigValue::object(obj);
        let toml_str = val.to_toml_string().unwrap();
        
        assert!(toml_str.contains("name = \"test\""));
        assert!(toml_str.contains("count = 42"));
        assert!(toml_str.contains("enabled = true"));

        // Test round-trip
        let parsed = ConfigValue::from_toml_string(&toml_str).unwrap();
        assert_eq!(parsed, val);
    }

    #[test]
    fn test_type_conversion() {
        let val = ConfigValue::integer(42);
        assert_eq!(val.as_integer().unwrap(), 42);
        assert_eq!(val.as_float().unwrap(), 42.0);
        assert!(val.as_string().is_err());
    }
}