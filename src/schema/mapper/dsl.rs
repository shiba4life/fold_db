use super::types::MappingRule;
use crate::schema::SchemaError; // Updated to use re-exported type

/// Parse DSL string into mapping rules
/// Parse a mapping DSL string into a vector of mapping rules
///
/// # Errors
///
/// Returns a `SchemaError` if:
/// - The DSL syntax is invalid
/// - A mapping rule is malformed
/// - Required fields are missing
pub fn parse_mapping_dsl(dsl: &str) -> Result<Vec<MappingRule>, SchemaError> {
    let mut rules = Vec::new();

    for (i, line) in dsl.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        match tokens.first().map(|s| s.to_uppercase()) {
            Some(cmd) => {
                match cmd.as_str() {
                    "RENAME" => {
                        if tokens.len() != 4 || tokens[2].to_uppercase() != "TO" {
                            return Err(SchemaError::InvalidDSL(format!(
                                "Invalid RENAME syntax on line {}",
                                i + 1
                            )));
                        }
                        rules.push(MappingRule::Rename {
                            source_field: tokens[1].to_string(),
                            target_field: tokens[3].to_string(),
                        });
                    }
                    "DROP" => {
                        if tokens.len() != 2 {
                            return Err(SchemaError::InvalidDSL(format!(
                                "Invalid DROP syntax on line {}",
                                i + 1
                            )));
                        }
                        rules.push(MappingRule::Drop {
                            field: tokens[1].to_string(),
                        });
                    }
                    "MAP" => {
                        // Syntax: MAP source_field TO target_field [WITH function]
                        if tokens.len() < 4 || tokens[2].to_uppercase() != "TO" {
                            return Err(SchemaError::InvalidDSL(format!("Invalid MAP syntax on line {}. Expected: MAP source TO target [WITH function]", i + 1)));
                        }
                        let function = if tokens.len() > 4 {
                            if tokens[4].to_uppercase() != "WITH" {
                                return Err(SchemaError::InvalidDSL(format!(
                                    "Invalid MAP syntax on line {}. Expected WITH keyword",
                                    i + 1
                                )));
                            }
                            if tokens.len() != 6 {
                                return Err(SchemaError::InvalidDSL(format!("Invalid MAP syntax on line {}. Expected function name after WITH", i + 1)));
                            }
                            Some(tokens[5].to_string())
                        } else {
                            None
                        };
                        rules.push(MappingRule::Map {
                            source_field: tokens[1].to_string(),
                            target_field: tokens[3].to_string(),
                            function,
                        });
                    }
                    _ => {
                        return Err(SchemaError::InvalidDSL(format!(
                            "Unknown command on line {}",
                            i + 1
                        )))
                    }
                }
            }
            None => {
                return Err(SchemaError::InvalidDSL(format!(
                    "Empty command on line {}",
                    i + 1
                )))
            }
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
            MAP email_address TO email WITH to_lowercase
        "#;

        let rules = parse_mapping_dsl(dsl).unwrap();
        assert_eq!(rules.len(), 3);

        match &rules[0] {
            MappingRule::Rename {
                source_field,
                target_field,
            } => {
                assert_eq!(source_field, "username");
                assert_eq!(target_field, "displayName");
            }
            _ => panic!("Expected Rename rule"),
        }

        match &rules[1] {
            MappingRule::Drop { field } => {
                assert_eq!(field, "privateEmail");
            }
            _ => panic!("Expected Drop rule"),
        }

        match &rules[2] {
            MappingRule::Map {
                source_field,
                target_field,
                function,
            } => {
                assert_eq!(source_field, "email_address");
                assert_eq!(target_field, "email");
                assert_eq!(function, &Some("to_lowercase".to_string()));
            }
            _ => panic!("Expected Map rule"),
        }
    }
}
