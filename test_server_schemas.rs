use fold_node::datafold_node::{DataFoldNode, load_node_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _ = env_logger::init();
    
    println!("Testing server schema loading...");
    
    // Load node configuration (same as HTTP server)
    let config = load_node_config(None, None);
    println!("Config loaded: storage_path = {:?}", config.storage_path);
    
    // Try to create a new node (bypassing load)
    match DataFoldNode::new(config.clone()) {
        Ok(mut node) => {
            println!("✅ Node created successfully");
            println!("Node ID: {}", node.get_node_id());
            
            // Try to get schema status
            match node.get_schema_status() {
                Ok(status) => {
                    println!("✅ Schema status retrieved");
                    println!("Discovered schemas: {:?}", status.discovered_schemas);
                    println!("Loaded schemas: {:?}", status.loaded_schemas);
                },
                Err(e) => println!("❌ Failed to get schema status: {}", e),
            }
            
            // Try to refresh schemas
            match node.refresh_schemas() {
                Ok(report) => {
                    println!("✅ Schema refresh successful");
                    println!("Discovered: {:?}", report.discovered_schemas);
                    println!("Loaded: {:?}", report.loaded_schemas);
                    println!("Failed: {:?}", report.failed_schemas);
                },
                Err(e) => println!("❌ Failed to refresh schemas: {}", e),
            }
        },
        Err(e) => {
            println!("❌ Failed to create node: {}", e);
            
            // Try with a temporary directory
            println!("Trying with temporary directory...");
            let temp_dir = tempfile::tempdir()?;
            let temp_config = fold_node::datafold_node::NodeConfig::new(temp_dir.path().to_path_buf());
            
            match DataFoldNode::new(temp_config) {
                Ok(mut node) => {
                    println!("✅ Node created with temp directory");
                    println!("Node ID: {}", node.get_node_id());
                    
                    match node.refresh_schemas() {
                        Ok(report) => {
                            println!("✅ Schema refresh successful with temp dir");
                            println!("Discovered: {:?}", report.discovered_schemas);
                        },
                        Err(e) => println!("❌ Failed to refresh schemas with temp dir: {}", e),
                    }
                },
                Err(e) => println!("❌ Failed to create node even with temp directory: {}", e),
            }
        }
    }
    
    Ok(())
}