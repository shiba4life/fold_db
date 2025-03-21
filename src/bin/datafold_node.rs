use fold_db::{DataFoldNode, NodeConfig};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting DataFold Node...");
    
    // Read node config from environment variable or default path
    let config_path = std::env::var("NODE_CONFIG")
        .unwrap_or_else(|_| "config/node_config.json".to_string());
    println!("Loading config from: {}", config_path);
    
    let config_str = fs::read_to_string(&config_path)?;
    let config: NodeConfig = serde_json::from_str(&config_str)?;
    println!("Config loaded successfully");
    
    // Load or initialize node
    println!("Loading DataFold Node...");
    let node = DataFoldNode::load(config)?;
    println!("Node loaded successfully");
    
    // Schemas are loaded from disk during node initialization
    println!("Previously loaded schemas are available");
    
    // Keep the process running until interrupted
    println!("DataFold Node is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    
    Ok(())
}
