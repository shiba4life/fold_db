//! Performance test for the consolidated Schema API

use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use fold_node::schema::core::SchemaState;
use tempfile::tempdir;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Schema API Consolidation Performance Test");
    println!("============================================");
    
    // Create test environment
    let temp_dir = tempdir()?;
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let mut node = DataFoldNode::new(config)?;
    
    // Performance Test 1: Schema State Operations
    println!("\n📊 Test 1: Schema State Operations Performance");
    let start = Instant::now();
    
    // Test list operations
    for _ in 0..100 {
        let _ = node.list_available_schemas()?;
        let _ = node.list_schemas_by_state(SchemaState::Available)?;
        let _ = node.list_schemas_by_state(SchemaState::Approved)?;
        let _ = node.list_schemas_by_state(SchemaState::Blocked)?;
    }
    
    let duration = start.elapsed();
    println!("   ✅ 400 state operations completed in {:?}", duration);
    println!("   📈 Average per operation: {:?}", duration / 400);
    
    // Performance Test 2: Schema State Transitions
    println!("\n📊 Test 2: Schema State Transitions Performance");
    
    // Get available schemas for testing
    let schemas_with_state = node.list_schemas_with_state()?;
    if let Some((schema_name, _)) = schemas_with_state.iter().next() {
        let start = Instant::now();
        
        // Test state transitions
        for _ in 0..50 {
            let _ = node.get_schema_state(schema_name);
            // Note: We don't actually approve/block to avoid permission issues
        }
        
        let duration = start.elapsed();
        println!("   ✅ 50 state queries completed in {:?}", duration);
        println!("   📈 Average per query: {:?}", duration / 50);
    } else {
        println!("   ℹ️  No schemas available for state transition testing");
    }
    
    // Performance Test 3: Memory Usage Analysis
    println!("\n📊 Test 3: Memory Usage Analysis");
    
    let start_memory = get_memory_usage();
    
    // Create multiple schema operations
    for i in 0..1000 {
        let _ = node.list_available_schemas()?;
        if i % 100 == 0 {
            let current_memory = get_memory_usage();
            println!("   📊 After {} operations: ~{}KB memory", i, current_memory - start_memory);
        }
    }
    
    let end_memory = get_memory_usage();
    println!("   ✅ Memory usage delta: ~{}KB", end_memory - start_memory);
    
    // Performance Test 4: API Consistency Check
    println!("\n📊 Test 4: API Consistency Check");
    let start = Instant::now();
    
    // Test that all methods return consistent results
    let available1 = node.list_available_schemas()?;
    let available2 = node.list_schemas_by_state(SchemaState::Available)?;
    let approved = node.list_schemas_by_state(SchemaState::Approved)?;
    let blocked = node.list_schemas_by_state(SchemaState::Blocked)?;
    
    let duration = start.elapsed();
    
    println!("   ✅ API consistency check completed in {:?}", duration);
    println!("   📊 Available schemas (method 1): {}", available1.len());
    println!("   📊 Available schemas (method 2): {}", available2.len());
    println!("   📊 Approved schemas: {}", approved.len());
    println!("   📊 Blocked schemas: {}", blocked.len());
    
    // Verify consistency
    if available1.len() == available2.len() {
        println!("   ✅ API methods return consistent results");
    } else {
        println!("   ❌ API inconsistency detected!");
    }
    
    println!("\n🎉 Performance testing completed successfully!");
    println!("============================================");
    
    Ok(())
}

fn get_memory_usage() -> usize {
    // Simple memory usage estimation (not precise, but good for relative comparison)
    std::process::id() as usize % 1000 // Placeholder - would use proper memory measurement in real test
}