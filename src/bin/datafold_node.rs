use fold_db::{DataFoldNode, NodeConfig, Schema};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read node config from environment variable or default path
    let config_path = std::env::var("NODE_CONFIG")
        .unwrap_or_else(|_| "/app/config/node_config.json".to_string());
    
    let config_str = fs::read_to_string(&config_path)?;
    let config: NodeConfig = serde_json::from_str(&config_str)?;
    
    // Initialize node
    let mut node = DataFoldNode::new(config)?;
    
    // Load schema from config directory
    let schema_path = "/app/config/schema.json";
    let schema_str = fs::read_to_string(schema_path)?;
    let schema: Schema = serde_json::from_str(&schema_str)?;
    
    node.load_schema(schema)?;
    
    // Start server loop
    println!("DataFold Node started on port 8080");
    
    // Keep the process running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
