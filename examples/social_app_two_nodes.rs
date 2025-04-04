use std::time::Duration;
use datafold_sdk::{
    DataFoldClient,
    types::NodeConnection,
};
use fold_db::{
    datafold_node::{DataFoldNode, TcpServer, config::NodeConfig},
    network::NetworkConfig,
};
use std::path::PathBuf;
use serde_json::json;
use tokio::time::sleep;
use uuid::Uuid;

use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a log file
    let mut log_file = File::create("social_app_two_nodes.log")?;
    
    // Helper function to log to both console and file
    let mut log = |msg: &str| {
        println!("{}", msg);
        writeln!(log_file, "{}", msg).unwrap();
    };
    log("DataFold Social App - Two Node Example");
    log("======================================");

    // Step 1: Create and start two nodes with different data
    log("\nSetting up two DataFold nodes...");
    
    // Create temporary directories for the nodes
    let node1_dir = PathBuf::from("test_data/two_node_example/node1/db");
    let node2_dir = PathBuf::from("test_data/two_node_example/node2/db");
    
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
    
    // Add data to nodes using the client API
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
    
    // Step 4: Create a social app client that connects to Node 1
    log("\nCreating social app client connected to Node 1...");
    
    // Wait a moment to ensure nodes are ready
    sleep(Duration::from_secs(2)).await;
    
    // Create a client for the app with a TCP connection to Node 1
    let connection = NodeConnection::TcpSocket("127.0.0.1".to_string(), 8001);
    let client = DataFoldClient::with_connection(
        "social-app",
        "private-key-placeholder",
        "public-key-placeholder",
        connection,
    );
    
    log("Client created and connected to Node 1");
    
    // Step 5: Discover remote nodes
    log("\nDiscovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    log(&format!("Discovered nodes: {:?}", nodes));
    
    // Step 6: Query data from both nodes
    log("\nQuerying data from both nodes...");
    
    // Query local node (Node 1)
    log("\nQuerying users on Node 1 (local)...");
    match client.query("user")
        .select(&["id", "username", "full_name", "bio", "email"])
        .execute()
        .await {
            Ok(query_result) => {
                log("Local user query results:");
                log(&format!("{:#?}", query_result.results));
            },
            Err(e) => {
                log(&format!("Error querying users on Node 1: {}", e));
            }
        }
    
    log("\nQuerying posts on Node 1 (local)...");
    match client.query("post")
        .select(&["id", "title", "content", "author_id"])
        .execute()
        .await {
            Ok(query_result) => {
                log("Local post query results:");
                log(&format!("{:#?}", query_result.results));
            },
            Err(e) => {
                log(&format!("Error querying posts on Node 1: {}", e));
            }
        }
    
    // First, try to query Node 2 through Node 1
    if !nodes.is_empty() {
        let remote_node_id = &nodes[0].id;
        log(&format!("\nAttempting to query Node 2 (ID: {}) through Node 1...", remote_node_id));
        
        log("\nQuerying posts on Node 2 (through Node 1)...");
        match client.query_on_node("post", remote_node_id)
            .select(&["id", "title", "content", "author_id"])
            .execute()
            .await {
                Ok(query_result) => {
                    log("Remote post query results (through Node 1):");
                    log(&format!("{:#?}", query_result.results));
                    
                    // Simply check if the string "Hello from Node 2" is in the results
                    let found_node2_post = query_result.results.iter().any(|value| {
                        value.as_str() == Some("Hello from Node 2")
                    });
                    
                    if found_node2_post {
                        log("\nSuccessfully queried 'Hello from Node 2' post through Node 1!");
                    } else {
                        log("\nDid not find 'Hello from Node 2' post in the query results from Node 1.");
                        log("This suggests that cross-node querying might not be working correctly.");
                    }
                },
                Err(e) => {
                    log(&format!("Error querying posts on Node 2 through Node 1: {}", e));
                }
            }
    } else {
        log("\nNo remote nodes found. Make sure both nodes are running and peers are properly added.");
    }
    
    // Now, connect directly to Node 2 to verify its data
    log("\nConnecting directly to Node 2 to verify its data...");
    let connection2 = NodeConnection::TcpSocket("127.0.0.1".to_string(), 8002);
    let direct_client = DataFoldClient::with_connection(
        "direct-client",
        "private-key-placeholder",
        "public-key-placeholder",
        connection2,
    );
    
    log("\nQuerying posts on Node 2 (direct connection)...");
    match direct_client.query("post")
        .select(&["id", "title", "content", "author_id"])
        .execute()
        .await {
            Ok(query_result) => {
                log("Direct Node 2 post query results:");
                log(&format!("{:#?}", query_result.results));
                
                // Simply check if the string "Hello from Node 2" is in the results
                let found_node2_post = query_result.results.iter().any(|value| {
                    value.as_str() == Some("Hello from Node 2")
                });
                
                if found_node2_post {
                    log("\nConfirmed 'Hello from Node 2' post exists on Node 2!");
                    
                    // Now check if we were able to query it through Node 1
                    let remote_query_successful = !nodes.is_empty() && !query_result.results.is_empty();
                    
                    if remote_query_successful {
                        log("\nCross-node querying is working correctly! We can query Node 2's data through Node 1.");
                    } else {
                        log("\nNode 2 has the expected data, but cross-node querying isn't working correctly.");
                        log("\nTo get 'Hello from Node 2' post from Node 1, you would need to fix the cross-node querying implementation.");
                    }
                } else {
                    log("\nDid not find 'Hello from Node 2' post on Node 2 directly.");
                    log("\nThis suggests that the data wasn't properly added to Node 2.");
                }
            },
            Err(e) => {
                log(&format!("Error querying posts on Node 2 directly: {}", e));
                log("This is unexpected since Node 2's TCP server should be accessible.");
            }
        }
    
    log("\nTwo-Node Social App Example completed successfully!");
    
    Ok(())
}
