use crate::{Schema, DataFoldNode};
use std::fs;
use std::path::{Path, PathBuf};

/// Loads a schema from a file into the DataFoldNode.
/// 
/// # Arguments
/// 
/// * `path` - Path to the schema file
/// * `node` - DataFoldNode instance to load the schema into
/// 
/// # Returns
/// 
/// Result indicating success or an error
pub fn load_schema_from_file<P: AsRef<Path>>(path: P, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let schema_str = fs::read_to_string(path)?;
    let schema: Schema = serde_json::from_str(&schema_str)?;
    node.load_schema(schema)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeConfig;
    use tempfile::tempdir;

    #[test]
    fn test_load_schema_from_config() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for test files
        let test_dir = tempdir()?;
        let config_path = test_dir.path().join("node_config.json");
        let schema_path = test_dir.path().join("schema.json");
        
        // Create test config
        let config = NodeConfig::default();
        fs::write(&config_path, serde_json::to_string(&config)?)?;
        
        // Create test schema
        let schema = Schema::new("test_schema".to_string());
        fs::write(&schema_path, serde_json::to_string(&schema)?)?;
        
        // Test loading
        let mut node = DataFoldNode::new(config)?;
        load_schema_from_file(&schema_path, &mut node)?;
        
        Ok(())
    }
}
