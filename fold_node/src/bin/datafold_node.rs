use fold_node::{
    datafold_node::{DataFoldNode, TcpServer, config::NodeConfig},
    network::NetworkConfig,
};
use fold_client::FoldClient;
use std::fs;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting DataFold Node...");
    
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut port = 9000; // Default port
    let mut tcp_port = 9000; // Default TCP port
    let mut start_fold_client = true; // Default to starting the FoldClient
    
    // Simple argument parsing
    for i in 1..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                port = p;
            }
        }
        if args[i] == "--tcp-port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                tcp_port = p;
            }
        }
        if args[i] == "--no-fold-client" {
            start_fold_client = false;
        }
    }
    
    // Read node config from environment variable or default path
    let config_path = std::env::var("NODE_CONFIG")
        .unwrap_or_else(|_| "config/node_config.json".to_string());
    println!("Loading config from: {}", config_path);
    
    let config_str = fs::read_to_string(&config_path)?;
    let config: NodeConfig = serde_json::from_str(&config_str)?;
    println!("Config loaded successfully");
    
    // Load or initialize node
    println!("Loading DataFold Node...");
    let mut node = DataFoldNode::load(config)?;
    println!("Node loaded successfully");
    
    // Schemas are loaded from disk during node initialization
    println!("Previously loaded schemas are available");
    
    // Initialize network layer
    println!("Initializing network layer...");
    let listen_address = format!("/ip4/0.0.0.0/tcp/{}", port);
    let network_config = NetworkConfig::new(&listen_address)
        .with_mdns(true)
        .with_request_timeout(30)
        .with_max_connections(50)
        .with_keep_alive_interval(20)
        .with_max_message_size(1_000_000);
    
    node.init_network(network_config).await?;
    println!("Network layer initialized");
    
    // Start the network service
    println!("Starting network service on port {}...", port);
    node.start_network_with_address(&listen_address).await?;
    println!("Network service started");
    
    // Print node ID for connecting
    println!("Node ID: {}", node.get_node_id());
    println!("Other nodes can connect to this node using the Node ID above");
    
    // Start the TCP server
    println!("Starting TCP server on port {}...", tcp_port);
    let tcp_server = TcpServer::new(node.clone(), tcp_port).await?;
    
    // Run the TCP server in a separate task
    let tcp_server_handle = tokio::spawn(async move {
        if let Err(e) = tcp_server.run().await {
            eprintln!("TCP server error: {}", e);
        }
    });
    
    // Start the FoldClient if enabled
    let fold_client_handle = if start_fold_client {
        println!("Starting integrated FoldClient...");
        
        // Configure the FoldClient to connect to our local node
        let mut config = fold_client::FoldClientConfig::default();
        config.node_tcp_address = Some(("127.0.0.1".to_string(), tcp_port));
        
        // Ensure the socket directory exists
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let socket_dir = home_dir.join(".datafold").join("sockets");
        println!("Creating socket directory at {:?}", socket_dir);
        std::fs::create_dir_all(&socket_dir)?;
        config.app_socket_dir = socket_dir;
        
        // Create the app data directory
        let app_data_dir = home_dir.join(".datafold").join("app_data");
        println!("Creating app data directory at {:?}", app_data_dir);
        std::fs::create_dir_all(&app_data_dir)?;
        config.app_data_dir = app_data_dir;
        
        // Create a new FoldClient with our configuration
        let mut fold_client = FoldClient::with_config(config)?;
        
        // Start the FoldClient
        fold_client.start().await?;
        println!("FoldClient started successfully");
        
        // Spawn a task to run the FoldClient
        let handle = tokio::spawn(async move {
            // Keep the FoldClient running until the main task is cancelled
            tokio::signal::ctrl_c().await.ok();
            println!("Stopping FoldClient...");
            if let Err(e) = fold_client.stop().await {
                eprintln!("Error stopping FoldClient: {}", e);
            }
            println!("FoldClient stopped");
        });
        
        Some(handle)
    } else {
        println!("FoldClient integration disabled");
        None
    };
    
    // Keep the process running until interrupted
    println!("DataFold Node is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    
    // Cancel the TCP server task
    tcp_server_handle.abort();
    
    // Wait for the FoldClient to shut down if it was started
    if let Some(handle) = fold_client_handle {
        handle.abort();
    }
    
    Ok(())
}
