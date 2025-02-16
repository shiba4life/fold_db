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

    #[test]
    fn test_load_schema_from_config() -> Result<(), Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config/node_config.json")?;
        let config: NodeConfig = serde_json::from_str(&config_str)?;
        let mut node = DataFoldNode::new(config)?;
        
        load_schema_from_file("config/schema.json", &mut node)?;
        Ok(())
    }
} 