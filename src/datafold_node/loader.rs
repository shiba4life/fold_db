use crate::schema::Schema;
use crate::datafold_node::DataFoldNode;
use std::fs;
use std::path::Path;

pub fn load_schema_from_file<P: AsRef<Path>>(path: P, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let schema_str = fs::read_to_string(path.as_ref())?;
    let schema: Schema = serde_json::from_str(&schema_str)?;
    node.load_schema(schema)?;
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
        };
        
        let mut node = DataFoldNode::new(config)?;
        load_schema_from_file(&schema_path, &mut node)?;
        Ok(())
    }
}
