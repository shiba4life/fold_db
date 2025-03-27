use fold_db::{
    datafold_node::{DataFoldNode, config::NodeConfig},
    network::NetworkConfig,
    schema::Schema,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting schema checking example...");
    
    // Create temporary directories for the nodes
    let node1_dir = PathBuf::from("test_data/example_node1/db");
    let node2_dir = PathBuf::from("test_data/example_node2/db");
    
    std::fs::create_dir_all(&node1_dir)?;
    std::fs::create_dir_all(&node2_dir)?;
    
    // Create node configs
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
    
    // Create a test schema for node1
    let test_schema1 = Schema::new("user_profile".to_string());
    let test_schema2 = Schema::new("product_catalog".to_string());
    
    // Load the schemas into node1
    node1.load_schema(test_schema1)?;
    node1.load_schema(test_schema2)?;
    
    println!("Node 1 schemas loaded");
    
    // Create network configs with different ports
    let network1_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/9001")
        .with_mdns(true);
    
    let network2_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/9002")
        .with_mdns(true);
    
    // Initialize the network layers
    node1.init_network(network1_config).await?;
    node2.init_network(network2_config).await?;
    
    println!("Network layers initialized");
    
    // Start the network services
    node1.start_network_with_address("/ip4/127.0.0.1/tcp/9001").await?;
    node2.start_network_with_address("/ip4/127.0.0.1/tcp/9002").await?;
    
    println!("Network services started");
    println!("\nNote: In this simplified implementation, we're not actually discovering peers");
    println!("through mDNS. In a real implementation, libp2p would handle peer discovery.");
    println!("For this example, we're manually adding peers to the known peers list.\n");
    
    // Get the node IDs
    let node1_id = node1.get_node_id().to_string();
    let node2_id = node2.get_node_id().to_string();
    
    println!("Node 1 ID: {}", node1_id);
    println!("Node 2 ID: {}", node2_id);
    
    // Add trusted nodes
    node1.add_trusted_node(&node2_id)?;
    node2.add_trusted_node(&node1_id)?;
    
    // Convert node IDs to PeerIds for remote schema checking
    let node1_peer_id = libp2p::PeerId::random(); // In a real scenario, this would be derived from the node ID
    let node2_peer_id = libp2p::PeerId::random(); // In a real scenario, this would be derived from the node ID
    
    println!("Node 1 Peer ID: {}", node1_peer_id);
    println!("Node 2 Peer ID: {}", node2_peer_id);
    
    // For testing purposes, add these peer IDs to the known peers in the network core
    // In a real scenario, this would happen through mDNS discovery
    {
        let mut network1 = node1.get_network_mut().await?;
        network1.add_known_peer(node2_peer_id.clone());
    }
    
    {
        let mut network2 = node2.get_network_mut().await?;
        network2.add_known_peer(node1_peer_id.clone());
    }
    
    println!("Trusted nodes added");
    
    // Wait for mDNS discovery to work
    println!("Waiting for mDNS discovery...");
    sleep(Duration::from_secs(2)).await;
    
    // Check which schemas are available on node1 from node2
    println!("Checking schemas on Node 1 from Node 2...");
    let schemas_to_check = vec![
        "user_profile".to_string(),
        "product_catalog".to_string(),
        "non_existent_schema".to_string(),
    ];
    
    match node2.check_remote_schemas(&node1_peer_id.to_string(), schemas_to_check).await {
        Ok(available_schemas) => {
            println!("Available schemas on Node 1:");
            for schema in available_schemas {
                println!("  - {}", schema);
            }
        }
        Err(e) => {
            println!("Error checking schemas: {}", e);
        }
    }
    
    // Check which schemas are available on node2 from node1
    println!("\nChecking schemas on Node 2 from Node 1...");
    let schemas_to_check = vec![
        "user_profile".to_string(),
        "product_catalog".to_string(),
    ];
    
    match node1.check_remote_schemas(&node2_peer_id.to_string(), schemas_to_check).await {
        Ok(available_schemas) => {
            println!("Available schemas on Node 2:");
            for schema in available_schemas {
                println!("  - {}", schema);
            }
        }
        Err(e) => {
            println!("Error checking schemas: {}", e);
        }
    }
    
    println!("\nExample completed successfully!");
    
    Ok(())
}
