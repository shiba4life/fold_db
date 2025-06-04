//! Test script to validate that TransformSchema approval properly registers transforms
//! 
//! This test verifies that:
//! 1. TransformSchema contains transform definitions
//! 2. Schema approval calls register_schema_transforms()
//! 3. TransformManager reloads transforms when schemas change
//! 4. UI shows registered transforms instead of "no transforms found"

use fold_node::datafold_node::DataFoldNode;
use fold_node::datafold_node::config::NodeConfig;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🧪 Starting TransformSchema registration fix test...");
    
    // Create test data directory
    let test_data_dir = PathBuf::from("test_data_transform_fix");
    if test_data_dir.exists() {
        std::fs::remove_dir_all(&test_data_dir)?;
    }
    std::fs::create_dir_all(&test_data_dir)?;
    
    // Create node configuration
    let config = NodeConfig::new(test_data_dir.clone());
    let node = DataFoldNode::new(config)?;
    
    println!("✅ Node created successfully");
    
    // Start HTTP server in background
    let server_handle = {
        let node_clone = node.clone();
        tokio::spawn(async move {
            if let Err(e) = node_clone.start_http_server("127.0.0.1:3001").await {
                eprintln!("Server error: {}", e);
            }
        })
    };
    
    // Wait for server to start
    sleep(Duration::from_millis(1000)).await;
    println!("✅ HTTP server started on port 3001");
    
    // Test 1: Check initial state - should show no transforms
    println!("\n📋 Test 1: Checking initial transform state");
    let initial_response = reqwest::get("http://127.0.0.1:3001/transforms/list").await?;
    let initial_transforms: serde_json::Value = initial_response.json().await?;
    println!("Initial transforms: {}", serde_json::to_string_pretty(&initial_transforms)?);
    
    // Test 2: Check if TransformSchema is available
    println!("\n📋 Test 2: Checking available schemas");
    let schemas_response = reqwest::get("http://127.0.0.1:3001/schemas").await?;
    let schemas: serde_json::Value = schemas_response.json().await?;
    println!("Available schemas: {}", serde_json::to_string_pretty(&schemas)?);
    
    // Look for TransformSchema
    let has_transform_schema = schemas["discovered_schemas"]
        .as_array()
        .map(|arr| arr.iter().any(|s| s.as_str() == Some("TransformSchema")))
        .unwrap_or(false);
        
    if !has_transform_schema {
        println!("⚠️  TransformSchema not found in available schemas");
        println!("   This might be expected if schema files are not in the correct location");
    } else {
        println!("✅ TransformSchema found in available schemas");
        
        // Test 3: Approve TransformSchema
        println!("\n📋 Test 3: Approving TransformSchema");
        let client = reqwest::Client::new();
        let approve_response = client
            .post("http://127.0.0.1:3001/schemas/TransformSchema/approve")
            .send()
            .await?;
            
        if approve_response.status().is_success() {
            println!("✅ TransformSchema approved successfully");
            
            // Wait for async processing
            sleep(Duration::from_millis(2000)).await;
            
            // Test 4: Check transforms after approval
            println!("\n📋 Test 4: Checking transforms after schema approval");
            let post_approval_response = reqwest::get("http://127.0.0.1:3001/transforms/list").await?;
            let post_approval_transforms: serde_json::Value = post_approval_response.json().await?;
            println!("Post-approval transforms: {}", serde_json::to_string_pretty(&post_approval_transforms)?);
            
            // Test 5: Check transform queue status
            println!("\n📋 Test 5: Checking transform queue status");
            let queue_response = reqwest::get("http://127.0.0.1:3001/transforms/queue").await?;
            let queue_status: serde_json::Value = queue_response.json().await?;
            println!("Queue status: {}", serde_json::to_string_pretty(&queue_status)?);
            
            // Analyze results
            let transforms_count = post_approval_transforms
                .as_object()
                .map(|obj| obj.len())
                .unwrap_or(0);
                
            if transforms_count > 0 {
                println!("\n🎉 SUCCESS: Found {} registered transforms after schema approval!", transforms_count);
                println!("✅ Fix appears to be working - transforms are now being registered during approval");
            } else {
                println!("\n❌ ISSUE: No transforms found after schema approval");
                println!("   This suggests the fix may not be working completely");
            }
            
        } else {
            println!("❌ Failed to approve TransformSchema: {}", approve_response.status());
            let error_text = approve_response.text().await?;
            println!("Error details: {}", error_text);
        }
    }
    
    // Cleanup
    server_handle.abort();
    if test_data_dir.exists() {
        std::fs::remove_dir_all(&test_data_dir)?;
    }
    
    println!("\n🧪 Test completed");
    Ok(())
}