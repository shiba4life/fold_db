use fold_node::{
    datafold_node::{config::NodeConfig, DataFoldNode, DataFoldHttpServer},
};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Main entry point for the DataFold HTTP server.
///
/// This function starts a DataFold HTTP server that serves the UI and provides
/// REST API endpoints for schemas, queries, and mutations. It initializes the node,
/// loads configuration, and starts the HTTP server.
///
/// # Command-Line Arguments
///
/// * `--port <PORT>` - Port for the HTTP server (default: 9001)
///
/// # Environment Variables
///
/// * `NODE_CONFIG` - Path to the node configuration file (default: config/node_config.json)
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Errors
///
/// Returns an error if:
/// * The configuration file cannot be read or parsed
/// * The node cannot be initialized
/// * The HTTP server cannot be started
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting DataFold HTTP Server...");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut http_port = 9001; // Default HTTP port

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        http_port = p;
                    }
                }
                i += 2;
            }
            _ => i += 1,
        }
    }

    // Read node config from environment variable or default path
    let config_path =
        std::env::var("NODE_CONFIG").unwrap_or_else(|_| "config/node_config.json".to_string());
    println!("Loading config from: {}", config_path);

    // Create a default config if the file doesn't exist
    let config: NodeConfig = if let Ok(config_str) = fs::read_to_string(&config_path) {
        serde_json::from_str(&config_str)?
    } else {
        println!("Config file not found, using default config");
        NodeConfig::new(PathBuf::from("data"))
    };
    println!("Config loaded successfully");

    // Load or initialize node
    println!("Loading DataFold Node...");
    let node = DataFoldNode::load(config)?;
    println!("Node loaded successfully");

    // Print node ID for connecting
    println!("Node ID: {}", node.get_node_id());

    // Start the HTTP server
    println!("Starting HTTP server on port {}...", http_port);
    let bind_address = format!("127.0.0.1:{}", http_port);
    let http_server = DataFoldHttpServer::new(node, &bind_address).await?;

    // Run the HTTP server
    http_server.run().await?;

    Ok(())
}