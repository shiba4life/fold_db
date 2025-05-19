use crate::datafold_node::DataFoldNode;
use crate::schema::{Schema, SchemaCore};
use crate::schema::types::JsonSchemaDefinition;
use serde_json;
use std::fs;
use std::path::Path;

/// Loads a schema from a JSON file into a DataFold node.
///
/// This function reads a schema definition from a JSON file, deserializes it,
/// and loads it into the provided DataFold node. The schema will be available
/// for queries and mutations after loading.
///
/// # Arguments
///
/// * `path` - Path to the schema JSON file
/// * `node` - The DataFold node to load the schema into
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Errors
///
/// Returns an error if:
/// * The file cannot be read
/// * The file content is not valid JSON
/// * The JSON does not represent a valid schema
/// * There is an error loading the schema into the node
///
/// # Examples
///
/// ```rust,no_run
/// use fold_node::datafold_node::{DataFoldNode, NodeConfig, loader::load_schema_from_file};
/// use std::path::PathBuf;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a node first
///     let config = NodeConfig::new(PathBuf::from("data"));
///     let mut node = DataFoldNode::new(config)?;
///     
///     // Load a schema from a file
///     load_schema_from_file("schemas/user_profile.json", &mut node)?;
///     Ok(())
/// }
/// ```
pub fn load_schema_from_file<P: AsRef<Path>>(
    path: P,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema_str = fs::read_to_string(path.as_ref())?;
    match serde_json::from_str::<Schema>(&schema_str) {
        Ok(schema) => {
            node.load_schema(schema)?;
        }
        Err(_) => {
            let json_schema: JsonSchemaDefinition = serde_json::from_str(&schema_str)?;
            let core = SchemaCore::default()?;
            let schema = core.interpret_schema(json_schema)?;
            node.load_schema(schema)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::config::NodeConfig;
    use tempfile::tempdir;

    #[test]
    fn test_load_schema_from_config() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = tempdir()?;
        let db_path = test_dir.path().join("test_db");

        // Create a test schema file
        let schema_path = test_dir.path().join("test_schema.json");
        let test_schema = r#"{
            "name": "test_schema",
            "fields": {},
            "payment_config": {
                "base_multiplier": 1.0,
                "min_payment_threshold": 0
            }
        }"#;
        fs::write(&schema_path, test_schema)?;

        let config = NodeConfig {
            storage_path: db_path.into(),
            default_trust_distance: 1,
            network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
        };

        let mut node = DataFoldNode::new(config)?;
        load_schema_from_file(&schema_path, &mut node)?;
        Ok(())
    }

    #[test]
    fn test_load_schema_with_transform_object() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = tempdir()?;
        let db_path = test_dir.path().join("test_db");

        // Create a schema that includes a transform object
        let schema_path = test_dir.path().join("transform_schema.json");
        let test_schema = r#"{
            "name": "transform_schema",
            "fields": {
                "computed": {
                    "permission_policy": {
                        "read_policy": { "Distance": 0 },
                        "write_policy": { "Distance": 0 },
                        "explicit_read_policy": null,
                        "explicit_write_policy": null
                    },
                    "ref_atom_uuid": "calc_uuid",
                    "payment_config": {
                        "base_multiplier": 1.0,
                        "trust_distance_scaling": { "None": null },
                        "min_payment": null
                    },
                    "field_mappers": {},
                    "field_type": "Single",
                    "transform": {
                        "logic": "4 + 5",
                        "reversible": false,
                        "signature": null,
                        "payment_required": false
                    }
                }
            },
            "payment_config": {
                "base_multiplier": 1.0,
                "min_payment_threshold": 0
            }
        }"#;
        fs::write(&schema_path, test_schema)?;

        let config = NodeConfig {
            storage_path: db_path.into(),
            default_trust_distance: 1,
            network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
        };

        let mut node = DataFoldNode::new(config)?;
        load_schema_from_file(&schema_path, &mut node)?;

        // Verify the schema was loaded with the transform
        let loaded_schema = node
            .get_schema("transform_schema")?
            .expect("schema not found");
        let field = loaded_schema
            .fields
            .get("computed")
            .expect("field not found");
        assert!(field.transform.is_some());
        assert_eq!(field.transform.as_ref().unwrap().logic, "4 + 5");

        Ok(())
    }
}
