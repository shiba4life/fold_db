use crate::{Schema, DataFoldNode};
use std::fs;

pub fn load_schema_from_file(path: &str, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let schema_str = fs::read_to_string(path)?;
    let schema: Schema = serde_json::from_str(&schema_str)?;
    node.load_schema(schema)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeConfig;
    use crate::datafold_node::DockerConfig;
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
            docker: DockerConfig::default(),
        };
        
        let mut node = DataFoldNode::new(config)?;
        load_schema_from_file(schema_path.to_str().unwrap(), &mut node)?;
        Ok(())
    }
} 