use fold_node::{
    datafold_node::{config::NodeConfig, DataFoldNode, DataFoldHttpServer},
};
use std::fs;
use clap::Parser;
use env_logger;
use log::{info, error};

/// Command line options for the HTTP server binary.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Port for the HTTP server
    #[arg(long, default_value_t = 9001)]
    port: u16,
}

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
    env_logger::init();
    info!("Starting DataFold HTTP Server...");

    // Parse command-line arguments using clap
    let Cli { port: http_port } = Cli::parse();

    // Read node config from environment variable or default path
    let config_path =
        std::env::var("NODE_CONFIG").unwrap_or_else(|_| "config/node_config.json".to_string());
    info!("Loading config from: {}", config_path);

    // Create a default config if the file doesn't exist
    let config: NodeConfig = if let Ok(config_str) = fs::read_to_string(&config_path) {
        serde_json::from_str(&config_str)?
    } else {
        println!("Config file not found, using default config");
        NodeConfig::default()
    };
    info!("Config loaded successfully");

    // Load or initialize node
    info!("Loading DataFold Node...");
    let node = DataFoldNode::load(config)?;
    info!("Node loaded successfully");

    // Print node ID for connecting
    info!("Node ID: {}", node.get_node_id());

    // Start the HTTP server
    info!("Starting HTTP server on port {}...", http_port);
    let bind_address = format!("127.0.0.1:{}", http_port);
    let http_server = DataFoldHttpServer::new(node, &bind_address).await?;

    // Run the HTTP server
    http_server.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn defaults() {
        let cli = Cli::parse_from(["test"]);
        assert_eq!(cli.port, 9001);
    }

    #[test]
    fn custom_port() {
        let cli = Cli::parse_from(["test", "--port", "8000"]);
        assert_eq!(cli.port, 8000);
    }
}