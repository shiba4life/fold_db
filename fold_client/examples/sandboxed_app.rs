//! Sandboxed application example for FoldClient
//!
//! This example demonstrates how a sandboxed application would use the
//! FoldClient's IPC mechanism to communicate with the DataFold node.

use fold_client::ipc::client::{IpcClient, Result, IpcClientError};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Check for verbose flag
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.contains(&"--verbose".to_string());
    
    // Initialize the logger with a more verbose configuration
    if verbose {
        std::env::set_var("RUST_LOG", "trace");
    }
    env_logger::init();
    
    println!("Sandboxed App starting...");
    if verbose {
        println!("Verbose mode enabled");
        println!("Command line arguments: {:?}", args);
    }

    // Get the app ID and token from environment variables
    // These are set by the FoldClient when launching the app
    println!("Getting environment variables...");
    let app_id = match env::var("FOLD_CLIENT_APP_ID") {
        Ok(id) => {
            println!("Found FOLD_CLIENT_APP_ID: {}", id);
            id
        },
        Err(e) => {
            println!("Error getting FOLD_CLIENT_APP_ID: {}", e);
            return Err(IpcClientError::Auth("FOLD_CLIENT_APP_ID environment variable not set".to_string()));
        }
    };
    
    let token = match env::var("FOLD_CLIENT_APP_TOKEN") {
        Ok(token) => {
            println!("Found FOLD_CLIENT_APP_TOKEN");
            token
        },
        Err(e) => {
            println!("Error getting FOLD_CLIENT_APP_TOKEN: {}", e);
            return Err(IpcClientError::Auth("FOLD_CLIENT_APP_TOKEN environment variable not set".to_string()));
        }
    };

    println!("Sandboxed App Example");
    println!("App ID: {}", app_id);

    // Get the socket directory from the environment or use the default
    println!("Getting socket directory...");
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        println!("Found FOLD_CLIENT_SOCKET_DIR: {}", dir);
        PathBuf::from(dir)
    } else {
        println!("FOLD_CLIENT_SOCKET_DIR not set, using default");
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let socket_dir = home_dir.join(".datafold").join("sockets");
        println!("Default socket directory: {:?}", socket_dir);
        socket_dir
    };

    // Connect to the FoldClient
    println!("Connecting to FoldClient...");
    let mut client = match IpcClient::connect(&socket_dir, &app_id, &token).await {
        Ok(client) => {
            println!("Connected to FoldClient successfully");
            client
        },
        Err(e) => {
            println!("Error connecting to FoldClient: {}", e);
            return Err(e);
        }
    };

    // List available schemas
    println!("Listing schemas...");
    let schemas = match client.list_schemas().await {
        Ok(schemas) => {
            println!("Successfully listed schemas");
            schemas
        },
        Err(e) => {
            println!("Error listing schemas: {}", e);
            return Err(e);
        }
    };
    println!("Available schemas: {:?}", schemas);

    // If the "user" schema exists, query it
    if schemas.contains(&"user".to_string()) {
        println!("User schema found, querying it...");
        let users = match client.query("user", &["id", "username", "full_name"], None).await {
            Ok(users) => {
                println!("Successfully queried users");
                users
            },
            Err(e) => {
                println!("Error querying users: {}", e);
                Vec::new()
            }
        };
        println!("Users: {:?}", users);

        // Create a new user
        println!("Creating a new user...");
        let user_id = uuid::Uuid::new_v4().to_string();
        println!("Generated user ID: {}", user_id);
        let user_data = json!({
            "id": user_id,
            "username": "sandboxed_user",
            "full_name": "Sandboxed User",
            "bio": "Created from a sandboxed app",
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        println!("User data: {}", user_data);
        
        match client.create("user", user_data).await {
            Ok(result) => {
                println!("User created with ID: {}", result);
                
                // Query the user we just created
                println!("Querying the new user...");
                let filter = Some(json!({
                    "field": "username",
                    "operator": "eq",
                    "value": "sandboxed_user",
                }));
                match client.query("user", &["id", "username", "full_name", "bio"], filter).await {
                    Ok(users) => {
                        println!("Successfully queried new user");
                        println!("New user: {:?}", users);
                    },
                    Err(e) => {
                        println!("Error querying new user: {}", e);
                    }
                };
            },
            Err(e) => {
                println!("Error creating user: {}", e);
            }
        };
    } else {
        println!("User schema not found, skipping user operations");
    }

    // Discover remote nodes
    println!("Discovering remote nodes...");
    let nodes = match client.discover_nodes().await {
        Ok(nodes) => {
            println!("Successfully discovered nodes");
            nodes
        },
        Err(e) => {
            println!("Error discovering nodes: {}", e);
            Vec::new()
        }
    };
    println!("Discovered nodes: {:?}", nodes);

    // If there are remote nodes, query them
    if !nodes.is_empty() {
        println!("Found remote nodes, attempting to query");
        let node_id = nodes[0].get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        
        println!("Querying remote node {}...", node_id);
        
        if schemas.contains(&"user".to_string()) {
            println!("User schema exists, querying remote node for users");
            match client.query_remote(
                node_id,
                "user",
                &["id", "username", "full_name"],
                None,
            ).await {
                Ok(remote_users) => {
                    println!("Successfully queried remote users");
                    println!("Remote users: {:?}", remote_users);
                },
                Err(e) => {
                    println!("Error querying remote users: {}", e);
                }
            };
        } else {
            println!("User schema not found, skipping remote user query");
        }
    } else {
        println!("No remote nodes found");
    }

    println!("Sandboxed App Example completed successfully!");
    
    Ok(())
}
