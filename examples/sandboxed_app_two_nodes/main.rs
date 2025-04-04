use std::fs::File;
use std::io::Write;

/// Logger for the test
pub struct Logger {
    log_file: File,
}

impl Logger {
    /// Create a new logger
    pub fn new(log_file_path: &str) -> Result<Self, std::io::Error> {
        let log_file = File::create(log_file_path)?;
        Ok(Self { log_file })
    }
    
    /// Log a message to both console and file
    pub fn log(&mut self, msg: &str) {
        println!("{}", msg);
        writeln!(self.log_file, "{}", msg).unwrap();
    }
}

// Import modules
mod node_setup;
mod schema;
mod data;
mod client;

// No need to use modules directly since we're using qualified paths

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a logger
    let mut logger = Logger::new("sandboxed_app_two_nodes.log")?;
    
    logger.log("DataFold Sandboxed App - Two Node Example");
    logger.log("=========================================");
    
    // Step 1: Set up two nodes with network connectivity
    let (node1, node2) = node_setup::setup_two_nodes(&mut logger).await?;
    
    // Step 2: Create schemas on both nodes
    schema::create_schemas(&node1, &node2, &mut logger).await?;
    
    // Step 3: Add test data to both nodes
    let (_alice, _alice_post, _bob, _bob_post) = data::add_test_data(&node1, &node2, &mut logger).await?;
    
    // Step 4: Set up FoldClient instances
    let (mut fold_client1, mut fold_client2) = client::setup_fold_clients(&mut logger).await?;
    
    // Step 5: Launch sandboxed apps
    client::launch_sandboxed_apps(&mut fold_client1, &mut fold_client2, &mut logger).await?;
    
    // Step 6: Verify cross-node querying
    client::verify_cross_node_query(&fold_client1, &mut logger).await?;
    
    // Step 7: Clean up
    client::stop_fold_clients(&mut fold_client1, &mut fold_client2, &mut logger).await?;
    
    logger.log("\nSandboxed App Two Node Example completed successfully!");
    
    Ok(())
}
