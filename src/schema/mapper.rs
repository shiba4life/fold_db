use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::schema::types::SchemaError;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "rule", rename_all = "lowercase")]
pub enum MappingRule {
    Rename { source_field: String, target_field: String },
    Drop { field: String },
    Add { target_field: String, value: Value },
    Map { source_field: String, target_field: String, function: String },
}

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

/// Parse DSL string into mapping rules
pub fn parse_mapping_dsl(dsl: &str) -> Result<Vec<MappingRule>, SchemaError> {
    let mut rules = Vec::new();
    
    for (i, line) in dsl.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        match tokens.get(0).map(|s| s.to_uppercase()) {
            Some(cmd) => {
                match cmd.as_str() {
                    "RENAME" => {
                        if tokens.len() != 4 || tokens[2].to_uppercase() != "TO" {
                            return Err(SchemaError::InvalidDSL(format!("Invalid RENAME syntax on line {}", i + 1)));
                        }
                        rules.push(MappingRule::Rename {
                            source_field: tokens[1].to_string(),
                            target_field: tokens[3].to_string(),
                        });
                    },
                    "DROP" => {
                        if tokens.len() != 2 {
                            return Err(SchemaError::InvalidDSL(format!("Invalid DROP syntax on line {}", i + 1)));
                        }
                        rules.push(MappingRule::Drop {
                            field: tokens[1].to_string(),
                        });
                    },
                    "ADD" => {
                        if tokens.len() < 3 {
                            return Err(SchemaError::InvalidDSL(format!("Invalid ADD syntax on line {}", i + 1)));
                        }
                        let target_field = tokens[1].to_string();
                        let value_str = tokens[2..].join(" ");
                        let value_trimmed = value_str.trim_matches('"');
                        rules.push(MappingRule::Add {
                            target_field,
                            value: json!(value_trimmed),
                        });
                    },
                    "MAP" => {
                        if tokens.len() != 6 || tokens[2].to_uppercase() != "TO" || tokens[4].to_uppercase() != "USING" {
                            return Err(SchemaError::InvalidDSL(format!("Invalid MAP syntax on line {}", i + 1)));
                        }
                        rules.push(MappingRule::Map {
                            source_field: tokens[1].to_string(),
                            target_field: tokens[3].to_string(),
                            function: tokens[5].to_string(),
                        });
                    },
                    _ => return Err(SchemaError::InvalidDSL(format!("Unknown command on line {}", i + 1))),
                }
            },
            None => return Err(SchemaError::InvalidDSL(format!("Empty command on line {}", i + 1))),
        }
    }

    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dsl() {
        let dsl = r#"
            # Comment line
            RENAME username TO displayName
            DROP privateEmail
            ADD status "active"
            MAP name TO upperName USING to_uppercase
        "#;

        let rules = parse_mapping_dsl(dsl).unwrap();
        assert_eq!(rules.len(), 4);

        match &rules[0] {
            MappingRule::Rename { source_field, target_field } => {
                assert_eq!(source_field, "username");
                assert_eq!(target_field, "displayName");
            },
            _ => panic!("Expected Rename rule"),
        }
    }

    #[test]
    fn test_schema_mapper_apply() {
        let rules = vec![
            MappingRule::Rename {
                source_field: "username".to_string(),
                target_field: "displayName".to_string(),
            },
            MappingRule::Drop {
                field: "privateEmail".to_string(),
            },
            MappingRule::Add {
                target_field: "status".to_string(),
                value: json!("active"),
            },
            MappingRule::Map {
                source_field: "name".to_string(),
                target_field: "upperName".to_string(),
                function: "to_uppercase".to_string(),
            },
        ];

        let mapper = SchemaMapper::new(
            vec!["profile".to_string(), "legacy".to_string()],
            "public_profile".to_string(),
            rules,
        );

        let mut sources = HashMap::new();
        sources.insert("profile".to_string(), json!({
            "username": "john_doe",
            "privateEmail": "john@example.com",
            "name": "John Doe",
            "bio": "Hello!"
        }));
        sources.insert("legacy".to_string(), json!({
            "username": "old_john",
            "privateEmail": "old@example.com",
            "name": "John Old",
            "bio": "Old bio"
        }));

        let result = mapper.apply(&sources).unwrap();
        let expected = json!({
            "displayName": "john_doe",
            "status": "active",
            "upperName": "JOHN DOE"
        });

        assert_eq!(result, expected);
    }
}
