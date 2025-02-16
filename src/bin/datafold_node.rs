use fold_db::{DataFoldNode, NodeConfig, Schema, datafold_node::WebServer};
use std::{fs, sync::Arc};

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
    
    // Initialize node without Arc
    println!("Initializing DataFold Node...");
    let mut node = DataFoldNode::new(config)?;
    println!("Node initialized successfully");
    
    // Load schema if provided
    println!("Checking for schema...");
    match fs::read_to_string("config/schema.json") {
        Ok(schema_str) => {
            println!("Found schema.json, loading...");
            match serde_json::from_str::<Schema>(&schema_str) {
                Ok(schema) => {
                    match node.load_schema(schema) {
                        Ok(_) => println!("Schema loaded successfully"),
                        Err(e) => eprintln!("Error loading schema into node: {}", e),
                    }
                },
                Err(e) => eprintln!("Error parsing schema.json: {}", e),
            }
        },
        Err(e) => eprintln!("Error reading schema.json: {}", e),
    }
    
    // Wrap in Arc and create web server
    println!("Creating web server...");
    let node = Arc::new(node);
    let server = WebServer::new(node);
    println!("Web server created, starting on port 8080...");
    
    // Run the server and handle any errors
    match server.run(8080).await {
        Ok(_) => println!("Web server stopped normally"),
        Err(e) => {
            eprintln!("Web server error: {}", e);
            eprintln!("Error details: {:?}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
