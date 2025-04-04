//! Sandboxed application example for FoldClient
//!
//! This example demonstrates how a sandboxed application would use the
//! FoldClient's IPC mechanism to communicate with the DataFold node.

use fold_client::ipc::client::{IpcClient, Result};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Get the app ID and token from environment variables
    // These are set by the FoldClient when launching the app
    let app_id = env::var("FOLD_CLIENT_APP_ID")
        .expect("FOLD_CLIENT_APP_ID environment variable not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN")
        .expect("FOLD_CLIENT_APP_TOKEN environment variable not set");

    println!("Sandboxed App Example");
    println!("App ID: {}", app_id);

    // Get the socket directory from the environment or use the default
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        PathBuf::from(dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.join(".datafold").join("sockets")
    };

    // Connect to the FoldClient
    println!("Connecting to FoldClient...");
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;
    println!("Connected to FoldClient");

    // List available schemas
    println!("Listing schemas...");
    let schemas = client.list_schemas().await?;
    println!("Available schemas: {:?}", schemas);

    // If the "user" schema exists, query it
    if schemas.contains(&"user".to_string()) {
        println!("Querying user schema...");
        let users = client.query("user", &["id", "username", "full_name"], None).await?;
        println!("Users: {:?}", users);

        // Create a new user
        println!("Creating a new user...");
        let user_id = uuid::Uuid::new_v4().to_string();
        let user_data = json!({
            "id": user_id,
            "username": "sandboxed_user",
            "full_name": "Sandboxed User",
            "bio": "Created from a sandboxed app",
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        let result = client.create("user", user_data).await?;
        println!("User created with ID: {}", result);

        // Query the user we just created
        println!("Querying the new user...");
        let filter = Some(json!({
            "field": "username",
            "operator": "eq",
            "value": "sandboxed_user",
        }));
        let users = client.query("user", &["id", "username", "full_name", "bio"], filter).await?;
        println!("New user: {:?}", users);
    }

    // Discover remote nodes
    println!("Discovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Discovered nodes: {:?}", nodes);

    // If there are remote nodes, query them
    if !nodes.is_empty() {
        let node_id = nodes[0].get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        
        println!("Querying remote node {}...", node_id);
        
        if schemas.contains(&"user".to_string()) {
            let remote_users = client.query_remote(
                node_id,
                "user",
                &["id", "username", "full_name"],
                None,
            ).await?;
            println!("Remote users: {:?}", remote_users);
        }
    }

    println!("Sandboxed App Example completed successfully!");
    
    Ok(())
}
