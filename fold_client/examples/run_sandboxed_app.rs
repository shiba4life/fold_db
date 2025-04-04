//! Example to run the sandboxed_app example
//!
//! This example demonstrates how to use the FoldClient to register and launch
//! the sandboxed_app example.

use fold_client::{FoldClient, FoldClientConfig, Result};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Create a FoldClient with a custom configuration
    let mut config = FoldClientConfig::default();
    config.allow_network_access = true;
    config.allow_filesystem_access = true;
    config.max_memory_mb = Some(512);
    config.max_cpu_percent = Some(25);

    // Create the FoldClient
    let mut client = FoldClient::with_config(config)?;

    // Start the FoldClient
    println!("Starting FoldClient...");
    client.start().await?;
    println!("FoldClient started successfully");

    // Register a new app
    println!("Registering sandboxed_app...");
    let app = client.register_app("Sandboxed App", &["list_schemas", "query", "mutation", "discover_nodes", "query_remote"]).await?;
    println!("App registered successfully with ID: {}", app.app_id);

    // Get the path to the sandboxed_app example
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("examples");
    
    // Build the sandboxed_app example if it doesn't exist
    if !target_dir.join("sandboxed_app").exists() {
        println!("Building sandboxed_app example...");
        std::process::Command::new("cargo")
            .args(&["build", "--example", "sandboxed_app"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("Failed to build sandboxed_app example");
    }

    // Launch the sandboxed_app with verbose output
    println!("Launching sandboxed_app...");
    client.launch_app(&app.app_id, target_dir.join("sandboxed_app").to_str().unwrap(), &["--verbose"]).await?;
    println!("sandboxed_app launched successfully");

    // Wait for the app to complete with a timeout
    println!("Waiting for sandboxed_app to complete (with 30 second timeout)...");
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);
    
    loop {
        let running = client.is_app_running(&app.app_id).await?;
        println!("Is app running: {}", running);
        
        if !running {
            println!("sandboxed_app has completed");
            break;
        }
        
        if start_time.elapsed() > timeout {
            println!("Timeout reached. Terminating sandboxed_app...");
            client.terminate_app(&app.app_id).await?;
            println!("sandboxed_app terminated");
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Stop the FoldClient
    println!("Stopping FoldClient...");
    client.stop().await?;
    println!("FoldClient stopped successfully");

    Ok(())
}
