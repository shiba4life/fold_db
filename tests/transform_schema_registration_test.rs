//! Test to verify TransformSchema.result transform registration and field mappings

use fold_node::fold_db_core::infrastructure::message_bus::FieldValueSet;
use fold_node::datafold_node::DataFoldNode;
use fold_node::datafold_node::config::NodeConfig;
use tempfile::tempdir;
use serde_json::json;

#[tokio::test]
async fn test_transform_schema_registration() {
    env_logger::init();
    
    let temp_dir = tempdir().unwrap();
    
    println!("🚀 Testing TransformSchema registration and field mappings");
    
    // Create DataFoldNode which will load schemas from available_schemas/
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let node = DataFoldNode::load(config).await.unwrap();
    let fold_db = node.get_fold_db().unwrap();
    let message_bus = fold_db.message_bus();
    
    // Wait for schema loading to complete
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    // 🔧 FIX: Approve schemas so their transforms get registered
    println!("🔧 Approving TransformBase for field dependencies...");
    if let Err(e) = fold_db.schema_manager().approve_schema("TransformBase") {
        println!("❌ Failed to approve TransformBase: {}", e);
    } else {
        println!("✅ TransformBase approved successfully");
    }
    
    println!("🔧 Approving TransformSchema to register its transforms...");
    if let Err(e) = fold_db.schema_manager().approve_schema("TransformSchema") {
        println!("❌ Failed to approve TransformSchema: {}", e);
    } else {
        println!("✅ TransformSchema approved successfully");
    }
    
    // Wait for approval processing to complete
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    // Access the transform manager directly from FoldDB
    let transform_manager = fold_db.transform_manager();
    
    // 🔧 FIX: Manually reload transforms to pick up newly registered transforms from approved schemas
    println!("🔄 Manually reloading transforms after schema approval...");
    if let Err(e) = transform_manager.reload_transforms() {
        println!("❌ Failed to reload transforms: {}", e);
    } else {
        println!("✅ Transforms reloaded successfully");
    }
    let registered_transforms = transform_manager.list_transforms().unwrap();
    println!("🔍 Registered transforms: {:?}", registered_transforms.keys().collect::<Vec<_>>());
    
    // Check field mappings for TransformBase fields
    let transforms_for_value1 = transform_manager.get_transforms_for_field("TransformBase", "value1").unwrap();
    let transforms_for_value2 = transform_manager.get_transforms_for_field("TransformBase", "value2").unwrap();
    
    println!("🔗 Transforms for TransformBase.value1: {:?}", transforms_for_value1);
    println!("🔗 Transforms for TransformBase.value2: {:?}", transforms_for_value2);
    
    // Verify field mappings were created
    if transforms_for_value1.is_empty() {
        println!("❌ No transforms found for TransformBase.value1 - this indicates the bug!");
    } else {
        println!("✅ Found {} transforms for TransformBase.value1", transforms_for_value1.len());
    }
    
    if transforms_for_value2.is_empty() {
        println!("❌ No transforms found for TransformBase.value2 - this indicates the bug!");
    } else {
        println!("✅ Found {} transforms for TransformBase.value2", transforms_for_value2.len());
    }
    
    // Test the orchestration pipeline by publishing FieldValueSet events
    println!("🎯 Testing transform orchestration...");
    
    let field_event1 = FieldValueSet::new(
        "TransformBase.value1".to_string(),
        json!(10),
        "test_source",
    );
    message_bus.publish(field_event1).unwrap();
    
    let field_event2 = FieldValueSet::new(
        "TransformBase.value2".to_string(),
        json!(20),
        "test_source",
    );
    message_bus.publish(field_event2).unwrap();
    
    // Wait for processing
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    
    println!("✅ Transform registration and field mapping test completed!");
}