use fold_db::{DataFoldNode, NodeConfig, datafold_node::{WebServer, load_schema_from_file}};
use std::{fs, sync::Arc, path::Path};

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
