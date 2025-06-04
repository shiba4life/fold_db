//! Test to verify that approved schemas loaded from disk can have their fields properly mapped
//! 
//! This test simulates the issue where approved schemas loaded during initialization
//! remain in the `available` HashMap but `map_fields()` only operates on the `schemas` HashMap.

use std::path::PathBuf;
use std::sync::Arc;
use fold_node::schema::core::{SchemaCore, SchemaState};
use fold_node::schema::Schema;
use fold_node::fold_db_core::infrastructure::message_bus::MessageBus;
use fold_node::db_operations::core::DbOps;

#[tokio::test]
async fn test_approved_schema_field_mapping_fix() {
    // Initialize test environment
    let temp_dir = tempfile::tempdir().unwrap();
    let schemas_dir = temp_dir.path().join("schemas");
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&schemas_dir).unwrap();
    std::fs::create_dir_all(&data_dir).unwrap();

    // Create a simple test schema
    let test_schema = Schema {
        name: "TestUser".to_string(),
        fields: vec![
            ("id".to_string(), fold_node::schema::types::field::variant::FieldVariant::Single(
                fold_node::schema::types::field::single_field::SingleField::new("String", None, None)
            )),
            ("name".to_string(), fold_node::schema::types::field::variant::FieldVariant::Single(
                fold_node::schema::types::field::single_field::SingleField::new("String", None, None)
            )),
        ].into_iter().collect(),
        relationships: std::collections::HashMap::new(),
    };

    // Save the schema to disk
    let schema_file = schemas_dir.join("TestUser.json");
    let schema_json = serde_json::to_string_pretty(&test_schema).unwrap();
    std::fs::write(&schema_file, schema_json).unwrap();

    // Initialize SchemaCore
    let message_bus = Arc::new(MessageBus::new());
    let db_ops = Arc::new(DbOps::new(&data_dir.to_string_lossy()).unwrap());
    let schema_core = SchemaCore::new(schemas_dir, db_ops, message_bus).unwrap();

    // Load the schema and approve it
    schema_core.discover_and_load_all_schemas().unwrap();
    schema_core.approve_schema("TestUser").unwrap();

    // Verify the schema is approved
    let approved_schemas = schema_core.list_schemas_by_state(SchemaState::Approved).unwrap();
    assert!(approved_schemas.contains(&"TestUser".to_string()), 
            "TestUser schema should be approved");

    // Test the fix: ensure approved schema can be mapped
    let result = schema_core.ensure_approved_schema_in_schemas("TestUser");
    assert!(result.is_ok(), "Should successfully move approved schema to schemas HashMap");

    // Test field mapping works after the fix
    let atom_refs_result = schema_core.map_fields("TestUser");
    assert!(atom_refs_result.is_ok(), "Field mapping should work for approved schema after fix");

    let atom_refs = atom_refs_result.unwrap();
    assert_eq!(atom_refs.len(), 2, "Should create atom refs for both fields (id and name)");

    println!("✅ Test passed: Approved schema field mapping fix is working correctly");
}

fn main() {
    println!("Run with: cargo test test_approved_schema_field_mapping_fix");
}