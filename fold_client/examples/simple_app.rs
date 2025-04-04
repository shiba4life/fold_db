//! Simple example application for FoldClient
//!
//! This example demonstrates how to use the FoldClient to register an app,
//! launch it, and communicate with the DataFold node.

use fold_client::{FoldClient, FoldClientConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Create a FoldClient with a custom configuration
    let mut config = FoldClientConfig::default();
    config.allow_network_access = false;
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
    println!("Registering app...");
    let app = client.register_app("Simple App", &["list_schemas", "query", "mutation"]).await?;
    println!("App registered successfully with ID: {}", app.app_id);

    // Launch the app
    println!("Launching app...");
    client.launch_app(&app.app_id, "echo", &["Hello from sandboxed app!"]).await?;
    println!("App launched successfully");

    // Wait for a moment to let the app run
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check if the app is still running
    let running = client.is_app_running(&app.app_id).await?;
    println!("App is running: {}", running);

    // List running apps
    let running_apps = client.list_running_apps().await?;
    println!("Running apps: {:?}", running_apps);

    // Terminate the app
    println!("Terminating app...");
    client.terminate_app(&app.app_id).await?;
    println!("App terminated successfully");

    // Stop the FoldClient
    println!("Stopping FoldClient...");
    client.stop().await?;
    println!("FoldClient stopped successfully");

    Ok(())
}
