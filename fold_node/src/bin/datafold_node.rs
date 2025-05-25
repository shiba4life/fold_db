use fold_node::{
    datafold_node::{load_node_config, DataFoldNode, TcpServer},
    network::NetworkConfig,
};

use clap::Parser;
use log::{info, error};

/// Command line options for the datafold node binary.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Port for the P2P network
    #[arg(long, default_value_t = 9000)]
    port: u16,

    /// Port for the TCP server
    #[arg(long, default_value_t = 9000)]
    tcp_port: u16,
}

/// Main entry point for the DataFold Node server.
///
/// This function starts a DataFold node server that listens for incoming
/// connections on the specified ports. It initializes the node, loads
/// configuration, sets up the network layer, and starts the TCP server.
///
/// # Command-Line Arguments
///
/// * `--port <PORT>` - Port for the P2P network (default: 9000)
/// * `--tcp-port <PORT>` - Port for the TCP server (default: 9000)
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
/// * The network layer cannot be initialized
/// * The TCP server cannot be started
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fold_node::web_logger::init().ok();
    info!("Starting DataFold Node...");

    // Parse command-line arguments using clap
    let Cli { port, tcp_port } = Cli::parse();

    // Load node configuration
    let config = load_node_config(None, Some(port));
    info!("Config loaded successfully");

    // Load or initialize node
    info!("Loading DataFold Node...");
    let mut node = DataFoldNode::load(config)?;
    info!("Node loaded successfully");

    // Schemas are loaded from disk during node initialization
    info!("Previously loaded schemas are available");

    // Initialize network layer
    info!("Initializing network layer...");
    let listen_address = format!("/ip4/0.0.0.0/tcp/{}", port);
    let network_config = NetworkConfig::new(&listen_address)
        .with_mdns(true)
        .with_request_timeout(30)
        .with_max_connections(50)
        .with_keep_alive_interval(20)
        .with_max_message_size(1_000_000);

    node.init_network(network_config).await?;
    info!("Network layer initialized");

    // Start the network service
    info!("Starting network service on port {}...", port);
    node.start_network_with_address(&listen_address).await?;
    info!("Network service started");

    // Print node ID for connecting
    info!("Node ID: {}", node.get_node_id());
    info!("Other nodes can connect to this node using the Node ID above");

    // Start the TCP server
    info!("Starting TCP server on port {}...", tcp_port);
    let tcp_server = TcpServer::new(node.clone(), tcp_port).await?;

    // Run the TCP server in a separate task
    let tcp_server_handle = tokio::spawn(async move {
        if let Err(e) = tcp_server.run().await {
            error!("TCP server error: {}", e);
        }
    });

    // Wait for the TCP server to start
    tcp_server_handle.await?;

    Ok(())
}

