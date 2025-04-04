use std::time::Duration;
use datafold_sdk::{
    DataFoldClient,
    types::NodeConnection,
};
use fold_db::{
    datafold_node::{DataFoldNode, TcpServer, config::NodeConfig},
    network::NetworkConfig,
};
use fold_client::{FoldClient, FoldClientConfig};
use std::path::PathBuf;
use serde_json::json;
use tokio::time::sleep;
use uuid::Uuid;

use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a log file
    let mut log_file = File::create("sandboxed_app_two_nodes.log")?;
    
    // Helper function to log to both console and file
    let mut log = |msg: &str| {
        println!("{}", msg);
        writeln!(log_file, "{}", msg).unwrap();
    };
    log("DataFold Sandboxed App - Two Node Example");
    log("=========================================");

    // Step 1: Create and start two nodes with different data
    log("\nSetting up two DataFold nodes...");
    
    // Create temporary directories for the nodes
    let node1_dir = PathBuf::from("test_data/sandboxed_two_node_example/node1/db");
    let node2_dir = PathBuf::from("test_data/sandboxed_two_node_example/node2/db");
    
    std::fs::create_dir_all(&node1_dir)?;
    std::fs::create_dir_all(&node2_dir)?;
    
    // Create node configs with different ports
    let node1_config = NodeConfig {
        storage_path: node1_dir,
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/9001".to_string(),
    };
    
    let node2_config = NodeConfig {
        storage_path: node2_dir,
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/9002".to_string(),
    };
    
    // Create the nodes
    let mut node1 = DataFoldNode::new(node1_config)?;
    let mut node2 = DataFoldNode::new(node2_config)?;
    
    log("Nodes created successfully");
    
    // Create network configs
    let network1_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/9001");
    let network2_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/9002");
    
    // Initialize the network layers
    node1.init_network(network1_config).await?;
    node2.init_network(network2_config).await?;
    
    // Start the network services
    node1.start_network_with_address("/ip4/127.0.0.1/tcp/9001").await?;
    log("Node 1 network service started");
    
    // Start the TCP server for Node 1 on a different port
    log("Starting TCP server for Node 1 on port 8001...");
    let tcp_server1 = TcpServer::new(node1.clone(), 8001).await?;
    let tcp_server1_handle = tokio::spawn(async move {
        if let Err(e) = tcp_server1.run().await {
            eprintln!("Node 1 TCP server error: {}", e);
        }
    });
    // We need to keep the handle to prevent the task from being dropped
    let _ = tcp_server1_handle;
    log("Node 1 TCP server started");
    
    // Wait a moment before starting the second node
    sleep(Duration::from_secs(1)).await;
    
    node2.start_network_with_address("/ip4/127.0.0.1/tcp/9002").await?;
    log("Node 2 network service started");
    
    // Start the TCP server for Node 2 on a different port
    log("Starting TCP server for Node 2 on port 8002...");
    let tcp_server2 = TcpServer::new(node2.clone(), 8002).await?;
    let tcp_server2_handle = tokio::spawn(async move {
        if let Err(e) = tcp_server2.run().await {
            eprintln!("Node 2 TCP server error: {}", e);
        }
    });
    // We need to keep the handle to prevent the task from being dropped
    let _ = tcp_server2_handle;
    log("Node 2 TCP server started");
    
    // Wait a moment to ensure both nodes are fully started
    sleep(Duration::from_secs(2)).await;
    
    log("Network services and TCP servers started");
    
    // Get the node IDs
    let node1_id = node1.get_node_id().to_string();
    let node2_id = node2.get_node_id().to_string();
    
    log(&format!("Node 1 ID: {}", node1_id));
    log(&format!("Node 2 ID: {}", node2_id));
    
    // Add trusted nodes
    node1.add_trusted_node(&node2_id)?;
    node2.add_trusted_node(&node1_id)?;
    
    // Get the actual PeerIds from the network cores
    let node1_peer_id;
    let node2_peer_id;
    
    {
        let network1 = node1.get_network_mut().await?;
        node1_peer_id = network1.local_peer_id();
    }
    
    {
        let network2 = node2.get_network_mut().await?;
        node2_peer_id = network2.local_peer_id();
    }
    
    log(&format!("Node 1 Peer ID: {}", node1_peer_id));
    log(&format!("Node 2 Peer ID: {}", node2_peer_id));
    
    // Manually add peers to simulate discovery
    {
        let mut network1 = node1.get_network_mut().await?;
        network1.add_known_peer(node2_peer_id);
        // Register the node ID to peer ID mapping
        network1.register_node_id(&node2_id, node2_peer_id);
    }
    
    {
        let mut network2 = node2.get_network_mut().await?;
        network2.add_known_peer(node1_peer_id);
        // Register the node ID to peer ID mapping
        network2.register_node_id(&node1_id, node1_peer_id);
    }
    
    // Wait for the nodes to discover each other
    sleep(Duration::from_secs(2)).await;
    
    log("Peers manually added");
    
    // Step 2: Create schemas on both nodes
    log("\nCreating schemas on both nodes...");
    
    // Create clients for both nodes using the TCP server ports
    let connection1 = NodeConnection::TcpSocket("127.0.0.1".to_string(), 8001);
    let client1 = DataFoldClient::with_connection(
        "schema-creator",
        "private-key-placeholder",
        "public-key-placeholder",
        connection1,
    );
    
    let connection2 = NodeConnection::TcpSocket("127.0.0.1".to_string(), 8002);
    let client2 = DataFoldClient::with_connection(
        "schema-creator",
        "private-key-placeholder",
        "public-key-placeholder",
        connection2,
    );
    
    // Create schemas using the client API
    log("Creating schemas using client API");
    
    // Create user schema
    log("Creating user schema");
    let user_schema = json!({
        "name": "user",
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        },
        "fields": {
            "id": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "username": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "full_name": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "bio": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "email": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            }
        }
    });
    
    // Create post schema
    log("Creating post schema");
    let post_schema = json!({
        "name": "post",
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        },
        "fields": {
            "id": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "title": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "content": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            },
            "author_id": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {
                        "NoRequirement": null
                    },
                    "write_policy": {
                        "Distance": 0
                    }
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                },
                "field_mappers": {}
            }
        }
    });
    
    // Create schemas on Node 1
    log("Creating schemas on Node 1");
    let request1 = client1.send_request(
        datafold_sdk::types::AppRequest::new(
            client1.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": user_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request1 {
        Ok(_) => log("User schema created on Node 1"),
        Err(e) => log(&format!("Error creating user schema on Node 1: {}", e)),
    }
    
    let request1 = client1.send_request(
        datafold_sdk::types::AppRequest::new(
            client1.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": post_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request1 {
        Ok(_) => log("Post schema created on Node 1"),
        Err(e) => log(&format!("Error creating post schema on Node 1: {}", e)),
    }
    
    // Create schemas on Node 2
    log("Creating schemas on Node 2");
    let request2 = client2.send_request(
        datafold_sdk::types::AppRequest::new(
            client2.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": user_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request2 {
        Ok(_) => log("User schema created on Node 2"),
        Err(e) => log(&format!("Error creating user schema on Node 2: {}", e)),
    }
    
    let request2 = client2.send_request(
        datafold_sdk::types::AppRequest::new(
            client2.get_app_id(),
            None,
            "create_schema",
            json!({ "schema": post_schema }),
            "private-key-placeholder",
        )
    ).await;
    
    match request2 {
        Ok(_) => log("Post schema created on Node 2"),
        Err(e) => log(&format!("Error creating post schema on Node 2: {}", e)),
    }
    
    log("Schemas created on both nodes");
    
    // Step 3: Add different data to each node
    log("\nAdding different data to each node...");
    
    // Add data to Node 1
    let alice_id = Uuid::new_v4().to_string();
    log(&format!("Adding user to Node 1: {}", alice_id));
    
    let alice_mutation = client1.send_request(
        datafold_sdk::types::AppRequest::new(
            client1.get_app_id(),
            None,
            "mutation",
            json!({
                "schema": "user",
                "mutation_type": "create",
                "data": {
                    "id": alice_id.clone(),
                    "username": "alice",
                    "full_name": "Alice Johnson",
                    "bio": "Node 1 user",
                    "email": "node1_user@example.com"
                }
            }),
            "private-key-placeholder",
        )
    ).await;
    
    match alice_mutation {
        Ok(_) => log("User added to Node 1"),
        Err(e) => log(&format!("Error adding user to Node 1: {}", e)),
    }
    
    let alice_post_id = Uuid::new_v4().to_string();
    log(&format!("Adding post to Node 1: {}", alice_post_id));
    
    let alice_post_mutation = client1.send_request(
        datafold_sdk::types::AppRequest::new(
            client1.get_app_id(),
            None,
            "mutation",
            json!({
                "schema": "post",
                "mutation_type": "create",
                "data": {
                    "id": alice_post_id.clone(),
                    "title": "Hello from Node 1",
                    "content": "This post is stored on Node 1",
                    "author_id": alice_id
                }
            }),
            "private-key-placeholder",
        )
    ).await;
    
    match alice_post_mutation {
        Ok(_) => log("Post added to Node 1"),
        Err(e) => log(&format!("Error adding post to Node 1: {}", e)),
    }
    
    // Add data to Node 2
    let bob_id = Uuid::new_v4().to_string();
    log(&format!("Adding user to Node 2: {}", bob_id));
    
    let bob_mutation = client2.send_request(
        datafold_sdk::types::AppRequest::new(
            client2.get_app_id(),
            None,
            "mutation",
            json!({
                "schema": "user",
                "mutation_type": "create",
                "data": {
                    "id": bob_id.clone(),
                    "username": "bob",
                    "full_name": "Bob Smith",
                    "bio": "Node 2 user",
                    "email": "node2_user@example.com"
                }
            }),
            "private-key-placeholder",
        )
    ).await;
    
    match bob_mutation {
        Ok(_) => log("User added to Node 2"),
        Err(e) => log(&format!("Error adding user to Node 2: {}", e)),
    }
    
    let bob_post_id = Uuid::new_v4().to_string();
    log(&format!("Adding post to Node 2: {}", bob_post_id));
    
    let bob_post_mutation = client2.send_request(
        datafold_sdk::types::AppRequest::new(
            client2.get_app_id(),
            None,
            "mutation",
            json!({
                "schema": "post",
                "mutation_type": "create",
                "data": {
                    "id": bob_post_id.clone(),
                    "title": "Hello from Node 2",
                    "content": "This post is stored on Node 2",
                    "author_id": bob_id
                }
            }),
            "private-key-placeholder",
        )
    ).await;
    
    match bob_post_mutation {
        Ok(_) => log("Post added to Node 2"),
        Err(e) => log(&format!("Error adding post to Node 2: {}", e)),
    }
    
    log("Data added to both nodes");
    
    // Step 4: Create FoldClient instances for each node
    log("\nCreating FoldClient instances...");
    
    // Create FoldClient for Node 1
    let mut config1 = FoldClientConfig::default();
    config1.allow_network_access = true;
    config1.allow_filesystem_access = true;
    config1.max_memory_mb = Some(512);
    config1.max_cpu_percent = Some(25);
    config1.node_tcp_address = Some(("127.0.0.1".to_string(), 8001));
    
    let mut fold_client1 = FoldClient::with_config(config1)?;
    log("FoldClient for Node 1 created");
    
    // Create FoldClient for Node 2
    let mut config2 = FoldClientConfig::default();
    config2.allow_network_access = true;
    config2.allow_filesystem_access = true;
    config2.max_memory_mb = Some(512);
    config2.max_cpu_percent = Some(25);
    config2.node_tcp_address = Some(("127.0.0.1".to_string(), 8002));
    
    let mut fold_client2 = FoldClient::with_config(config2)?;
    log("FoldClient for Node 2 created");
    
    // Start the FoldClients
    log("Starting FoldClient for Node 1...");
    fold_client1.start().await?;
    log("FoldClient for Node 1 started successfully");
    
    log("Starting FoldClient for Node 2...");
    fold_client2.start().await?;
    log("FoldClient for Node 2 started successfully");
    
    // Step 5: Register sandboxed apps
    log("\nRegistering sandboxed apps...");
    
    // Register app for Node 1
    let app1 = fold_client1.register_app(
        "Sandboxed App Node 1",
        &["list_schemas", "query", "mutation", "discover_nodes", "query_remote"]
    ).await?;
    log(&format!("App registered for Node 1 with ID: {}", app1.app_id));
    
    // Register app for Node 2
    let app2 = fold_client2.register_app(
        "Sandboxed App Node 2",
        &["list_schemas", "query", "mutation", "discover_nodes", "query_remote"]
    ).await?;
    log(&format!("App registered for Node 2 with ID: {}", app2.app_id));
    
    // Step 6: Get the path to the sandboxed_app example
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("examples");
    
    // Get the path to the fold_client examples
    let fold_client_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fold_client");
    
    // Build the sandboxed_app example
    log("Building sandboxed_app example...");
    std::process::Command::new("cargo")
        .args(&["build", "--example", "sandboxed_app"])
        .current_dir(&fold_client_dir)
        .status()
        .expect("Failed to build sandboxed_app example");
    
    // Get the path to the built example
    let sandboxed_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("examples")
        .join("sandboxed_app");
    
    // Check if the example exists
    if !sandboxed_app_path.exists() {
        log(&format!("Error: sandboxed_app example not found at {:?}", sandboxed_app_path));
        return Err("sandboxed_app example not found".into());
    }
    
    // Step 7: Launch sandboxed apps
    log("\nLaunching sandboxed apps...");
    
    // Launch app for Node 1
    log("Launching sandboxed app for Node 1...");
    fold_client1.launch_app(
        &app1.app_id,
        target_dir.join("sandboxed_app").to_str().unwrap(),
        &["--verbose", "--node", "1"]
    ).await?;
    log("Sandboxed app for Node 1 launched successfully");
    
    // Wait for the app to complete with a timeout
    log("Waiting for Node 1 app to complete (with 15 second timeout)...");
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    
    loop {
        let running = fold_client1.is_app_running(&app1.app_id).await?;
        
        if !running {
            log("Node 1 sandboxed app has completed");
            break;
        }
        
        if start_time.elapsed() > timeout {
            log("Timeout reached. Terminating Node 1 sandboxed app...");
            fold_client1.terminate_app(&app1.app_id).await?;
            log("Node 1 sandboxed app terminated");
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    // Launch app for Node 2
    log("Launching sandboxed app for Node 2...");
    fold_client2.launch_app(
        &app2.app_id,
        target_dir.join("sandboxed_app").to_str().unwrap(),
        &["--verbose", "--node", "2"]
    ).await?;
    log("Sandboxed app for Node 2 launched successfully");
    
    // Wait for the app to complete with a timeout
    log("Waiting for Node 2 app to complete (with 15 second timeout)...");
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    
    loop {
        let running = fold_client2.is_app_running(&app2.app_id).await?;
        
        if !running {
            log("Node 2 sandboxed app has completed");
            break;
        }
        
        if start_time.elapsed() > timeout {
            log("Timeout reached. Terminating Node 2 sandboxed app...");
            fold_client2.terminate_app(&app2.app_id).await?;
            log("Node 2 sandboxed app terminated");
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    // Step 8: Verify cross-node querying
    log("\nVerifying cross-node querying...");
    
    // Launch a special verification app for Node 1 that will query Node 2
    let verify_app = fold_client1.register_app(
        "Verification App",
        &["list_schemas", "query", "mutation", "discover_nodes", "query_remote"]
    ).await?;
    log(&format!("Verification app registered with ID: {}", verify_app.app_id));
    
    // Create a verification app project
    let verification_project_dir = PathBuf::from("test_data/sandboxed_two_node_example/verification_app");
    std::fs::create_dir_all(&verification_project_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "verification_app"
version = "0.1.0"
edition = "2021"

[dependencies]
fold_client = { path = "../../../fold_client" }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
dirs = "4.0"
"#;
    
    std::fs::write(verification_project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create src directory
    let src_dir = verification_project_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    
    // Create main.rs
    let main_rs = r#"use fold_client::ipc::client::{IpcClient, Result};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Verification App starting...");
    
    // Get the app ID and token from environment variables
    let app_id = env::var("FOLD_CLIENT_APP_ID").expect("FOLD_CLIENT_APP_ID not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN").expect("FOLD_CLIENT_APP_TOKEN not set");
    
    // Get the socket directory
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        PathBuf::from(dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.join(".datafold").join("sockets")
    };
    
    // Connect to the FoldClient
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;
    println!("Connected to FoldClient successfully");
    
    // Discover remote nodes
    println!("Discovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Discovered nodes: {:?}", nodes);
    
    if nodes.is_empty() {
        println!("No remote nodes found!");
        return Ok(());
    }
    
    // Get the first remote node ID
    let node_id = nodes[0].get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    
    println!("Querying remote node {}...", node_id);
    
    // Query posts on the remote node
    println!("Querying posts on remote node...");
    let remote_posts = client.query_remote(
        node_id,
        "post",
        &["id", "title", "content", "author_id"],
        None,
    ).await?;
    
    println!("Remote posts: {:?}", remote_posts);
    
    // Check if we found "Hello from Node 2" post
    let found_node2_post = remote_posts.iter().any(|post| {
        post.get("title").and_then(|v| v.as_str()) == Some("Hello from Node 2")
    });
    
    if found_node2_post {
        println!("SUCCESS: Found 'Hello from Node 2' post through cross-node query!");
    } else {
        println!("FAILURE: Did not find 'Hello from Node 2' post in remote query results.");
    }
    
    Ok(())
}
"#;
    
    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Build the verification app
    log("Building verification app...");
    std::process::Command::new("cargo")
        .args(&["build"])
        .current_dir(&verification_project_dir)
        .status()
        .expect("Failed to build verification app");
    
    // Get the path to the built verification app
    let verification_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("verification_app");
    
    // Launch the verification app
    log("Launching verification app...");
    fold_client1.launch_app(
        &verify_app.app_id,
        verification_app_path.to_str().unwrap(),
        &[]
    ).await?;
    log("Verification app launched successfully");
    
    // Wait for the verification app to complete with a timeout
    log("Waiting for verification app to complete (with 15 second timeout)...");
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    
    loop {
        let running = fold_client1.is_app_running(&verify_app.app_id).await?;
        
        if !running {
            log("Verification app has completed");
            break;
        }
        
        if start_time.elapsed() > timeout {
            log("Timeout reached. Terminating verification app...");
            fold_client1.terminate_app(&verify_app.app_id).await?;
            log("Verification app terminated");
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    // Step 9: Directly query Node 2 from Node 1 to demonstrate cross-node querying
    log("\nDirectly querying Node 2 from Node 1...");
    
    // Create a client for Node 1
    let connection = NodeConnection::TcpSocket("127.0.0.1".to_string(), 8001);
    let direct_client = DataFoldClient::with_connection(
        "direct-client",
        "private-key-placeholder",
        "public-key-placeholder",
        connection,
    );
    
    // Discover remote nodes
    log("Discovering remote nodes...");
    let nodes = direct_client.discover_nodes().await?;
    log(&format!("Discovered nodes: {:?}", nodes));
    
    if !nodes.is_empty() {
        let remote_node_id = &nodes[0].id;
        log(&format!("Querying posts on Node 2 (ID: {}) through Node 1...", remote_node_id));
        
        // Query posts on Node 2 through Node 1
        let query_result = direct_client.query_on_node("post", remote_node_id)
            .select(&["id", "title", "content", "author_id"])
            .execute()
            .await;
        
        match query_result {
            Ok(result) => {
                log("Remote post query results:");
                log(&format!("{:#?}", result.results));
                
                // Check if we found "Hello from Node 2" post
                let found_node2_post = result.results.iter().any(|value| {
                    if let Some(title_str) = value.as_str() {
                        return title_str == "Hello from Node 2";
                    }
                    false
                });
                
                if found_node2_post {
                    log("\n✅ SUCCESS: Found 'Hello from Node 2' post through cross-node query!");
                } else {
                    log("\n❌ FAILURE: Did not find 'Hello from Node 2' post in the query results.");
                }
            },
            Err(e) => {
                log(&format!("Error querying posts on Node 2 through Node 1: {}", e));
            }
        }
    } else {
        log("No remote nodes found. Make sure both nodes are running and peers are properly added.");
    }
    
    // Stop the FoldClients
    log("\nStopping FoldClients...");
    fold_client1.stop().await?;
    fold_client2.stop().await?;
    log("FoldClients stopped successfully");
    
    log("\nSandboxed App Two Node Example completed successfully!");
    
    Ok(())
}
