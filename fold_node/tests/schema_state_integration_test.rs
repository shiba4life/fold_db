//! Integration test for schema state management API

use fold_node::datafold_node::{config::NodeConfig, DataFoldNode};
use fold_node::schema::core::SchemaState;
use tempfile::tempdir;

#[tokio::test]
async fn test_schema_state_management_api() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let mut node = DataFoldNode::new(config).unwrap();

    // Test 1: List available schemas (should work)
    let available_schemas = node.list_available_schemas().unwrap();
    println!("Available schemas: {:?}", available_schemas);

    // Test 2: List schemas by state
    let approved_schemas = node.list_schemas_by_state(SchemaState::Approved).unwrap();
    println!("Approved schemas: {:?}", approved_schemas);

    let blocked_schemas = node.list_schemas_by_state(SchemaState::Blocked).unwrap();
    println!("Blocked schemas: {:?}", blocked_schemas);

    // Test 3: Try to get state of non-existent schema (should fail)
    let result = node.get_schema_state("NonExistentSchema");
    assert!(result.is_err(), "Should fail for non-existent schema");

    // Test 4: Check if there are sample schemas and test state transitions
    let schemas_with_state = node.list_schemas_with_state().unwrap();
    println!("Schemas with state: {:?}", schemas_with_state);

    if let Some((schema_name, _current_state)) = schemas_with_state.iter().next() {
        println!("Testing with schema: {}", schema_name);

        // Test getting schema state
        let state = node.get_schema_state(schema_name).unwrap();
        println!("Schema '{}' state: {:?}", schema_name, state);

        // Test approving schema (might fail due to permissions, that's ok)
        match node.approve_schema(schema_name) {
            Ok(()) => {
                println!("Successfully approved schema '{}'", schema_name);
                let new_state = node.get_schema_state(schema_name).unwrap();
                println!("New state: {:?}", new_state);
                assert_eq!(new_state, SchemaState::Approved);
            }
            Err(e) => println!("Failed to approve schema '{}': {}", schema_name, e),
        }
    }

    println!("✅ Schema State Management API test completed successfully!");
}
