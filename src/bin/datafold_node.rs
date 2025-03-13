use fold_db::{DataFoldNode, NodeConfig, datafold_node::{UiServer, AppServer}};
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
    
    // Load or initialize node without Arc
    println!("Loading DataFold Node...");
    let mut node = DataFoldNode::load(config)?;
    println!("Node loaded successfully");
    
    // Schemas are loaded from disk during node initialization
    println!("Previously loaded schemas are available");
    
    // Wrap in Arc<Mutex> and create servers
    println!("Creating servers...");
    let node = Arc::new(tokio::sync::Mutex::new(node));
    
    // Create UI server
    let ui_server = UiServer::new(Arc::clone(&node));
    
    // Create App server
    let app_server = AppServer::new(Arc::clone(&node));
    
    // Run both servers in separate tasks
    println!("Starting servers...");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut port = 3000;
    
    // Check for --port argument
    for i in 1..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                port = p;
                println!("Using port {} as base port", port);
            }
        }
    }
    
    // Calculate UI and App server ports
    let ui_port = port + 1;
    let app_port = port;
    
    println!("UI server will use port {}", ui_port);
    println!("App server will use port {}", app_port);
    
    // Run UI server in a separate task
    let ui_handle = tokio::spawn(async move {
        match ui_server.run(ui_port).await {
            Ok(_) => println!("UI server stopped normally"),
            Err(e) => {
                eprintln!("UI server error: {}", e);
                eprintln!("Error details: {:?}", e);
            }
        }
    });
    
    // Run App server in a separate task
    let app_handle = tokio::spawn(async move {
        match app_server.run(app_port).await {
            Ok(_) => println!("App server stopped normally"),
            Err(e) => {
                eprintln!("App server error: {}", e);
                eprintln!("Error details: {:?}", e);
            }
        }
    });
    
    // Wait for both servers to complete
    let _ = tokio::try_join!(ui_handle, app_handle);
    
    Ok(())
}
