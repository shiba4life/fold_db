use fold_db::{DataFoldNode, NodeConfig, datafold_node::{WebServer, load_schema_from_file}};
use std::{fs, sync::Arc, path::{Path, PathBuf}};

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
    
    // Load schema if provided
    println!("Checking for schema...");
    if let Err(e) = load_schema_from_file("config/schema.json", &mut node) {
        eprintln!("Error loading schema: {}", e);
    } else {
        println!("Schema loaded successfully");
    }
    
    // Initialize app system
    println!("Initializing app system...");
    let apps_dir = Path::new("apps");
    if !apps_dir.exists() {
        fs::create_dir_all(apps_dir)?;
        println!("Created apps directory");
    }
    node.init_app_system(apps_dir)?;
    println!("App system initialized");
    
    // Register core APIs
    println!("Registering core APIs...");
    node.register_api("data", "1.0.0", "Data access API")?;
    node.register_api("schema", "1.0.0", "Schema management API")?;
    node.register_api("network", "1.0.0", "Network API")?;
    println!("Core APIs registered");
    
    // Load apps
    println!("Loading apps...");
    if let Ok(loaded_apps) = node.load_all_apps() {
        if loaded_apps.is_empty() {
            println!("No apps found");
        } else {
            println!("Loaded apps: {}", loaded_apps.join(", "));
        }
    } else {
        println!("Failed to load apps");
    }
    
    // Wrap in Arc<Mutex> and create web server
    println!("Creating web server...");
    let node = Arc::new(tokio::sync::Mutex::new(node));
    
    // Check if we should use Unix socket
    let use_unix_socket = std::env::var("USE_UNIX_SOCKET")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    
    let unix_socket_path = std::env::var("UNIX_SOCKET_PATH")
        .unwrap_or_else(|_| "/var/run/datafold.sock".to_string());
    
    let server = if use_unix_socket {
        println!("Using Unix socket at: {}", unix_socket_path);
        WebServer::new(node).with_unix_socket(PathBuf::from(unix_socket_path))
    } else {
        println!("Using TCP socket");
        WebServer::new(node)
    };
    
    let port = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    
    println!("Web server created, starting on port {}...", port);
    
    // Run the server and handle any errors
    match server.run(port).await {
        Ok(_) => println!("Web server stopped normally"),
        Err(e) => {
            eprintln!("Web server error: {}", e);
            eprintln!("Error details: {:?}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
