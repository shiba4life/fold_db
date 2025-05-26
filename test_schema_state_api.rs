//! Test script to verify the new schema state management API

use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use fold_node::schema::core::SchemaState;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Schema State Management API...");
    
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let mut node = DataFoldNode::new(config)?;
    
    println!("✅ DataFoldNode created successfully");
    
    // Test 1: List available schemas (should be empty initially)
    let available_schemas = node.list_available_schemas()?;
    println!("📋 Available schemas: {:?}", available_schemas);
    
    // Test 2: List schemas by state
    let approved_schemas = node.list_schemas_by_state(SchemaState::Approved)?;
    println!("✅ Approved schemas: {:?}", approved_schemas);
    
    let blocked_schemas = node.list_schemas_by_state(SchemaState::Blocked)?;
    println!("🚫 Blocked schemas: {:?}", blocked_schemas);
    
    // Test 3: Try to get state of non-existent schema (should fail)
    match node.get_schema_state("NonExistentSchema") {
        Ok(_) => println!("❌ Expected error for non-existent schema"),
        Err(e) => println!("✅ Correctly failed for non-existent schema: {}", e),
    }
    
    // Test 4: Load a sample schema and test state transitions
    // First, let's check if there are any sample schemas available
    let schemas_with_state = node.list_schemas_with_state()?;
    println!("📊 Schemas with state: {:?}", schemas_with_state);
    
    if let Some((schema_name, current_state)) = schemas_with_state.iter().next() {
        println!("🔍 Testing with schema: {} (current state: {:?})", schema_name, current_state);
        
        // Test getting schema state
        let state = node.get_schema_state(schema_name)?;
        println!("📊 Schema '{}' state: {:?}", schema_name, state);
        
        // Test approving schema
        match node.approve_schema(schema_name) {
            Ok(()) => {
                println!("✅ Successfully approved schema '{}'", schema_name);
                let new_state = node.get_schema_state(schema_name)?;
                println!("📊 New state: {:?}", new_state);
            },
            Err(e) => println!("⚠️  Failed to approve schema '{}': {}", schema_name, e),
        }
        
        // Test blocking schema
        match node.block_schema(schema_name) {
            Ok(()) => {
                println!("🚫 Successfully blocked schema '{}'", schema_name);
                let new_state = node.get_schema_state(schema_name)?;
                println!("📊 New state: {:?}", new_state);
            },
            Err(e) => println!("⚠️  Failed to block schema '{}': {}", schema_name, e),
        }
    } else {
        println!("ℹ️  No schemas available for state transition testing");
    }
    
    println!("\n🎉 Schema State Management API test completed!");
    Ok(())
}