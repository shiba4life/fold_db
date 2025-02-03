Below is an updated design document and pseudocode for a SchemaMapper that supports multiple source schemas. In this design, a single SchemaMapper can pull data from several source schemas, applying mapping rules to each as needed to populate a new target schema. The DSL syntax remains similar but now the payload for a mapper may specify one or more source schemas.

Requirements
	1.	Mapping Rules and DSL:
	•	The DSL still supports commands like:
	•	RENAME <source_field> TO <target_field>
	•	DROP <field>
	•	ADD <target_field> "<value>"
	•	MAP <source_field> TO <target_field> USING <function>
	•	These commands are parsed into MappingRule variants.
	2.	SchemaMapper Structure with Multiple Sources:
	•	Update the SchemaMapper so it holds a vector of source schema names instead of a single source schema.
	•	It also holds a target schema name and a set of mapping rules that will be applied to data from all source schemas.
	•	When a transformation is requested, the mapper iterates over each source schema, applies the mapping rules to that source’s data, and then merges the results into the target schema. Merging may simply be a union of the transformed JSON objects (with priority given to fields produced by earlier sources if needed).
	3.	Apply Function:
	•	The apply method now accepts a HashMap of source data, where keys are source schema names and values are the corresponding JSON objects.
	•	For each source schema that is provided and included in the mapper’s list, it applies the mapping rules (as before) and collects the transformed output.
	•	After processing all sources, the outputs are merged together. If a field is produced by more than one source, the mapper can either choose the first non-null value or apply a merge strategy.
	4.	API Integration:
	•	The API endpoint (for loading a SchemaMapper) now accepts an array for source_schemas and a single target_schema along with a DSL string.
	•	At runtime, when the target schema is queried or being bootstrapped, the system retrieves data from each listed source schema, calls the mapper’s apply function with these data sets, and returns the merged result.

Pseudocode Example in Rust

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "rule", rename_all = "lowercase")]
pub enum MappingRule {
    Rename { source_field: String, target_field: String },
    Drop { field: String },
    Add { target_field: String, value: Value },
    Map { source_field: String, target_field: String, function: String },
}

/// The SchemaMapper now supports multiple source schemas.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaMapper {
    /// A list of source schema names.
    pub source_schemas: Vec<String>,
    /// The target schema name.
    pub target_schema: String,
    pub rules: Vec<MappingRule>,
    // Optionally, a priority field could be added for chaining multiple mappers.
}

impl SchemaMapper {
    /// Apply the mapping rules for each source's data and merge the results.
    /// The input is a mapping from source schema name to its JSON data.
    pub fn apply(&self, sources_data: &HashMap<String, Value>) -> Value {
        let mut merged_target = serde_json::Map::new();
        // Iterate over each source schema in the order specified.
        for src in &self.source_schemas {
            if let Some(source_data) = sources_data.get(src) {
                // Process each source_data using our mapping rules.
                if source_data.is_object() {
                    let mut data = source_data.clone();
                    let obj = data.as_object_mut().unwrap();
                    let mut transformed = serde_json::Map::new();
                    
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
                                if let Some(val) = obj.get(source_field).cloned() {
                                    // Simple function implementation: for "to_uppercase"
                                    if function == "to_uppercase" {
                                        if let Some(s) = val.as_str() {
                                            transformed.insert(target_field.clone(), json!(s.to_uppercase()));
                                        } else {
                                            transformed.insert(target_field.clone(), val);
                                        }
                                    } else {
                                        // Default: copy the value
                                        transformed.insert(target_field.clone(), val);
                                    }
                                }
                            },
                        }
                    }
                    // Merge remaining unmapped fields from the source
                    for (k, v) in obj.iter() {
                        transformed.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                    // Merge this transformed output into the overall target.
                    // In case of conflict, values from earlier sources have priority.
                    for (k, v) in transformed.into_iter() {
                        merged_target.entry(k).or_insert(v);
                    }
                }
            }
        }
        Value::Object(merged_target)
    }
}

/// DSL Parser: Converts DSL string into a Vec<MappingRule>
pub fn parse_mapping_dsl(dsl: &str) -> Result<Vec<MappingRule>, String> {
    let mut rules = Vec::new();
    for (i, line) in dsl.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        match tokens.get(0).map(|s| s.to_uppercase().as_str()) {
            Some("RENAME") => {
                if tokens.len() != 4 || tokens[2].to_uppercase() != "TO" {
                    return Err(format!("Syntax error on line {}: {}", i+1, trimmed));
                }
                rules.push(MappingRule::Rename {
                    source_field: tokens[1].to_string(),
                    target_field: tokens[3].to_string(),
                });
            },
            Some("DROP") => {
                if tokens.len() != 2 {
                    return Err(format!("Syntax error on line {}: {}", i+1, trimmed));
                }
                rules.push(MappingRule::Drop { field: tokens[1].to_string() });
            },
            Some("ADD") => {
                if tokens.len() < 3 {
                    return Err(format!("Syntax error on line {}: {}", i+1, trimmed));
                }
                let target_field = tokens[1].to_string();
                let value_str = tokens[2..].join(" ");
                let value_trimmed = value_str.trim_matches('"');
                rules.push(MappingRule::Add { target_field, value: json!(value_trimmed) });
            },
            Some("MAP") => {
                if tokens.len() != 6 || tokens[2].to_uppercase() != "TO" || tokens[4].to_uppercase() != "USING" {
                    return Err(format!("Syntax error on line {}: {}", i+1, trimmed));
                }
                rules.push(MappingRule::Map {
                    source_field: tokens[1].to_string(),
                    target_field: tokens[3].to_string(),
                    function: tokens[5].to_string(),
                });
            },
            _ => return Err(format!("Unknown command on line {}: {}", i+1, trimmed)),
        }
    }
    Ok(rules)
}

/// Example payload for loading a SchemaMapper via an API endpoint.
#[derive(Deserialize)]
struct SchemaMapperPayload {
    source_schemas: Vec<String>,
    target_schema: String,
    mapping_dsl: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_dsl_multiple_rules() {
        let dsl = r#"
            # This is a comment
            RENAME username TO displayName
            DROP privateEmail
            ADD status "active"
            MAP username TO displayName USING to_uppercase
        "#;
        let rules = parse_mapping_dsl(dsl).unwrap();
        assert_eq!(rules.len(), 4);
        if let MappingRule::Rename { source_field, target_field } = &rules[0] {
            assert_eq!(source_field, "username");
            assert_eq!(target_field, "displayName");
        }
    }

    #[test]
    fn test_schema_mapper_apply_multiple_sources() {
        let rules = vec![
            MappingRule::Rename { source_field: "username".to_string(), target_field: "displayName".to_string() },
            MappingRule::Drop { field: "privateEmail".to_string() },
            MappingRule::Add { target_field: "status".to_string(), value: json!("active") },
            MappingRule::Map { source_field: "username".to_string(), target_field: "displayName".to_string(), function: "to_uppercase".to_string() },
        ];
        let mapper = SchemaMapper {
            source_schemas: vec!["social".to_string(), "legacy_social".to_string()],
            target_schema: "public_social".to_string(),
            rules,
        };

        // Simulate source data from two source schemas.
        let mut sources = HashMap::new();
        sources.insert("social".to_string(), json!({
            "username": "alice",
            "privateEmail": "alice@example.com",
            "bio": "Hello, world!"
        }));
        sources.insert("legacy_social".to_string(), json!({
            "username": "alice_old",
            "privateEmail": "oldalice@example.com",
            "bio": "Old bio"
        }));

        let result = mapper.apply(&sources);
        // Expected result: For each source, mapping rules are applied and merged.
        // For demonstration, we assume that fields from "social" take priority.
        let expected = json!({
            "displayName": "ALICE",
            "status": "active",
            "bio": "Hello, world!"
        });
        assert_eq!(result, expected);
    }
}

Instructions for the AI Developer
	1.	Update the SchemaMapper Structure:
	•	Modify the SchemaMapper struct to include a field source_schemas: Vec<String> instead of a single source_schema.
	•	Keep target_schema: String and rules: Vec<MappingRule> as before.
	2.	DSL Parsing:
	•	Ensure the DSL parser remains unchanged. The DSL itself does not need to change to support multiple sources—this is handled by the payload of the SchemaMapper.
	3.	Apply Function:
	•	Update the apply method so that it accepts a HashMap mapping each source schema name to its JSON data.
	•	For each source schema in the mapper’s source_schemas, check if data is available, apply the mapping rules to transform it, and merge the results.
	•	When merging, if a field has already been set by an earlier source, do not overwrite it (or apply a defined merge strategy).
	4.	API Integration:
	•	Define an API payload (SchemaMapperPayload) that accepts a vector for source_schemas, a single target schema, and a DSL string.
	•	The API endpoint (e.g., POST /api/schema_mapper/load) should parse the DSL using parse_mapping_dsl, construct a SchemaMapper instance, and register it with the Schema Manager.
	5.	Testing:
	•	Write unit tests for both DSL parsing and the apply method with multiple source data inputs.
	•	Verify that the transformed output meets the expected result when multiple source schemas are provided.

This updated design provides a clear blueprint for implementing a SchemaMapper that supports multiple source schemas, using a simple DSL for mapping definitions. It details the necessary changes to the data structures, parsing logic, and the application of rules, and outlines how the mapper integrates into the API workflow for folddb.