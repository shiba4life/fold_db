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

    // Deserialize either a full `Schema` or a `JsonSchemaDefinition`
    let schema: Schema = match serde_json::from_str::<Schema>(&schema_str) {
        Ok(schema) => schema,
        Err(_) => {
            let json_schema: JsonSchemaDefinition = serde_json::from_str(&schema_str)?;
            let core = SchemaCore::init_default()?;
            core.interpret_schema(json_schema)?
        }
    };

    node.load_schema(schema)?;
    Ok(())
}


