use serde_json::json;
use crate::schema::types::SchemaError;
use super::types::MappingRule;

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
}
