use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use datafold_sdk::{
    DataFoldClient,
    types::NodeConnection,
};
use fold_client::{FoldClient, FoldClientConfig};

use crate::Logger;

/// FoldClient test instance
pub struct TestFoldClient {
    pub client: FoldClient,
    pub app_id: String,
}

/// Set up FoldClient instances for both nodes
pub async fn setup_fold_clients(logger: &mut Logger) -> Result<(TestFoldClient, TestFoldClient), Box<dyn std::error::Error>> {
    logger.log("\nCreating FoldClient instances...");
    
    // Create FoldClient for Node 1
    let mut config1 = FoldClientConfig::default();
    config1.allow_network_access = true;
    config1.allow_filesystem_access = true;
    config1.max_memory_mb = Some(512);
    config1.max_cpu_percent = Some(25);
    config1.node_tcp_address = Some(("127.0.0.1".to_string(), 8001));
    
    let mut fold_client1 = FoldClient::with_config(config1)?;
    logger.log("FoldClient for Node 1 created");
    
    // Create FoldClient for Node 2
    let mut config2 = FoldClientConfig::default();
    config2.allow_network_access = true;
    config2.allow_filesystem_access = true;
    config2.max_memory_mb = Some(512);
    config2.max_cpu_percent = Some(25);
    config2.node_tcp_address = Some(("127.0.0.1".to_string(), 8002));
    
    let mut fold_client2 = FoldClient::with_config(config2)?;
    logger.log("FoldClient for Node 2 created");
    
    // Start the FoldClients
    logger.log("Starting FoldClient for Node 1...");
    fold_client1.start().await?;
    logger.log("FoldClient for Node 1 started successfully");
    
    logger.log("Starting FoldClient for Node 2...");
    fold_client2.start().await?;
    logger.log("FoldClient for Node 2 started successfully");
    
    // Register apps
    logger.log("\nRegistering sandboxed apps...");
    
    // Register app for Node 1
    let app1 = fold_client1.register_app(
        "Sandboxed App Node 1",
        &["list_schemas", "query", "mutation", "discover_nodes", "query_remote"]
    ).await?;
    logger.log(&format!("App registered for Node 1 with ID: {}", app1.app_id));
    
    // Register app for Node 2
    let app2 = fold_client2.register_app(
        "Sandboxed App Node 2",
        &["list_schemas", "query", "mutation", "discover_nodes", "query_remote"]
    ).await?;
    logger.log(&format!("App registered for Node 2 with ID: {}", app2.app_id));
    
    Ok((
        TestFoldClient { client: fold_client1, app_id: app1.app_id },
        TestFoldClient { client: fold_client2, app_id: app2.app_id }
    ))
}

/// Launch sandboxed apps for both nodes
pub async fn launch_sandboxed_apps(fold_client1: &mut TestFoldClient, fold_client2: &mut TestFoldClient, logger: &mut Logger) -> Result<(), Box<dyn std::error::Error>> {
    // Get the path to the sandboxed_app example
    let fold_client_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fold_client");
    
    // Build the sandboxed_app example
    logger.log("Building sandboxed_app example...");
    std::process::Command::new("cargo")
        .args(&["build", "--example", "sandboxed_app"])
        .current_dir(&fold_client_dir)
        .status()
        .expect("Failed to build sandboxed_app example");
    
    // Get the path to the built example
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("examples");
    
    let sandboxed_app_path = target_dir.join("sandboxed_app");
    
    // Check if the example exists
    if !sandboxed_app_path.exists() {
        logger.log(&format!("Error: sandboxed_app example not found at {:?}", sandboxed_app_path));
        return Err("sandboxed_app example not found".into());
    }
    
    // Launch apps
    logger.log("\nLaunching sandboxed apps...");
    
    // Launch app for Node 1
    logger.log("Launching sandboxed app for Node 1...");
    fold_client1.client.launch_app(
        &fold_client1.app_id,
        sandboxed_app_path.to_str().unwrap(),
        &["--verbose", "--node", "1"]
    ).await?;
    logger.log("Sandboxed app for Node 1 launched successfully");
    
    // Wait for the app to complete with a timeout
    logger.log("Waiting for Node 1 app to complete (with 15 second timeout)...");
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    
    loop {
        let running = fold_client1.client.is_app_running(&fold_client1.app_id).await?;
        
        if !running {
            logger.log("Node 1 sandboxed app has completed");
            break;
        }
        
        if start_time.elapsed() > timeout {
            logger.log("Timeout reached. Terminating Node 1 sandboxed app...");
            fold_client1.client.terminate_app(&fold_client1.app_id).await?;
            logger.log("Node 1 sandboxed app terminated");
            break;
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    // Launch app for Node 2
    logger.log("Launching sandboxed app for Node 2...");
    fold_client2.client.launch_app(
        &fold_client2.app_id,
        sandboxed_app_path.to_str().unwrap(),
        &["--verbose", "--node", "2"]
    ).await?;
    logger.log("Sandboxed app for Node 2 launched successfully");
    
    // Wait for the app to complete with a timeout
    logger.log("Waiting for Node 2 app to complete (with 15 second timeout)...");
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    
    loop {
        let running = fold_client2.client.is_app_running(&fold_client2.app_id).await?;
        
        if !running {
            logger.log("Node 2 sandboxed app has completed");
            break;
        }
        
        if start_time.elapsed() > timeout {
            logger.log("Timeout reached. Terminating Node 2 sandboxed app...");
            fold_client2.client.terminate_app(&fold_client2.app_id).await?;
            logger.log("Node 2 sandboxed app terminated");
            break;
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    Ok(())
}

/// Verify cross-node querying
pub async fn verify_cross_node_query(_fold_client1: &TestFoldClient, logger: &mut Logger) -> Result<(), Box<dyn std::error::Error>> {
    logger.log("\nDirectly querying Node 2 from Node 1...");
    
    // Create a client for Node 1
    let connection = NodeConnection::TcpSocket("127.0.0.1".to_string(), 8001);
    let direct_client = DataFoldClient::with_connection(
        "direct-client",
        "private-key-placeholder",
        "public-key-placeholder",
        connection,
    );
    
    // Discover remote nodes
    logger.log("Discovering remote nodes...");
    let nodes = direct_client.discover_nodes().await?;
    logger.log(&format!("Discovered nodes: {:?}", nodes));
    
    if !nodes.is_empty() {
        let remote_node_id = &nodes[0].id;
        logger.log(&format!("Querying posts on Node 2 (ID: {}) through Node 1...", remote_node_id));
        
        // Query posts on Node 2 through Node 1
        let query_result = direct_client.query_on_node("post", remote_node_id)
            .select(&["id", "title", "content", "author_id"])
            .execute()
            .await;
        
        match query_result {
            Ok(result) => {
                logger.log("Remote post query results:");
                logger.log(&format!("{:#?}", result.results));
                
                // Check if we found "Hello from Node 2" post
                let found_node2_post = result.results.iter().any(|value| {
                    if let Some(title_str) = value.as_str() {
                        return title_str == "Hello from Node 2";
                    }
                    false
                });
                
                if found_node2_post {
                    logger.log("\n✅ SUCCESS: Found 'Hello from Node 2' post through cross-node query!");
                } else {
                    logger.log("\n❌ FAILURE: Did not find 'Hello from Node 2' post in the query results.");
                }
            },
            Err(e) => {
                logger.log(&format!("Error querying posts on Node 2 through Node 1: {}", e));
            }
        }
    } else {
        logger.log("No remote nodes found. Make sure both nodes are running and peers are properly added.");
    }
    
    Ok(())
}

/// Stop FoldClient instances
pub async fn stop_fold_clients(fold_client1: &mut TestFoldClient, fold_client2: &mut TestFoldClient, logger: &mut Logger) -> Result<(), Box<dyn std::error::Error>> {
    logger.log("\nStopping FoldClients...");
    fold_client1.client.stop().await?;
    fold_client2.client.stop().await?;
    logger.log("FoldClients stopped successfully");
    
    Ok(())
}
