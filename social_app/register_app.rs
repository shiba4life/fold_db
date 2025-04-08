use fold_client::{FoldClient, FoldClientConfig};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the app name and permissions from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: register_app <app_name> <permissions>");
        println!("Example: register_app social-app list_schemas,query,mutation,discover_nodes");
        return Ok(());
    }

    let app_name = &args[1];
    let permissions: Vec<&str> = args[2].split(',').collect();

    // Configure the FoldClient to connect to the integrated FoldClient in the DataFold node
    let mut config = FoldClientConfig::default();
    config.node_tcp_address = Some(("127.0.0.1".to_string(), 9000));
    
    // Ensure the socket directory exists
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let socket_dir = home_dir.join(".datafold").join("sockets");
    config.app_socket_dir = socket_dir;
    
    // Create the app data directory
    let app_data_dir = home_dir.join(".datafold").join("app_data");
    config.app_data_dir = app_data_dir;
    
    // Create a new FoldClient with our configuration
    let mut client = FoldClient::with_config(config)?;
    
    // We don't need to start the FoldClient since we're connecting to the integrated one

    // Register the app
    let app = client.register_app(app_name, &permissions).await?;

    println!("App ID: {}", app.app_id);
    println!("App Token: {}", app.token);

    // Stop the FoldClient
    client.stop().await?;

    Ok(())
}
