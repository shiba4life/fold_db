//! UI Range Filter Test
//!
//! Tests that range filtering works correctly from the UI query path,
//! simulating the exact query structure sent by the UI.

use log::info;
use serde_json::json;
use std::collections::HashMap;
use fold_node::{
    fold_db_core::FoldDB,
    schema::{
        types::{
            operations::{Mutation, MutationType, Query},
            field::{FieldVariant, RangeField},
            Schema,
        },
    },
    permissions::types::policy::PermissionsPolicy,
    fees::types::config::FieldPaymentConfig,
};
use tempfile::tempdir;

#[test]
fn test_ui_range_filter_query_path() {
    env_logger::init();
    info!("🧪 TEST: UI Range Filter Query Path");
    info!("   Testing that UI range filter queries work correctly");

    // Create temporary database
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_str().expect("Invalid path");
    
    let core = FoldDB::new(temp_path).expect("Failed to create FoldDB");

    // Create TestRangeSchema
    let mut schema = Schema::new_range("TestRangeSchema".to_string(), "test_id".to_string());
    let range_field = RangeField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );
    
    schema.fields.insert("test_id".to_string(), FieldVariant::Range(range_field.clone()));
    schema.fields.insert("test_data".to_string(), FieldVariant::Range(range_field));

    // Register and approve schema
    let mut core_mut = core;
    core_mut.add_schema_available(schema).expect("Failed to add schema");
    core_mut.approve_schema("TestRangeSchema").expect("Failed to approve schema");

    // Create mutations for test data
    info!("📝 Storing test data...");
    
    // Store data for range key '1'
    let mut fields1 = std::collections::HashMap::new();
    fields1.insert("test_id".to_string(), json!("1"));
    fields1.insert("test_data".to_string(), json!("a"));
    
    let mutation1 = Mutation::new(
        "TestRangeSchema".to_string(),
        fields1,
        "web-ui".to_string(),
        0,
        MutationType::Create,
    );
    
    core_mut.write_schema(mutation1).expect("Failed to store mutation for key '1'");

    // Store data for range key '2'
    let mut fields2 = std::collections::HashMap::new();
    fields2.insert("test_id".to_string(), json!("2"));
    fields2.insert("test_data".to_string(), json!("b"));
    
    let mutation2 = Mutation::new(
        "TestRangeSchema".to_string(),
        fields2,
        "web-ui".to_string(),
        0,
        MutationType::Create,
    );
    
    core_mut.write_schema(mutation2).expect("Failed to store mutation for key '2'");

    info!("📋 Data storage completed, testing UI query path...");

    // Test 1: Query with range filter for key "1" (exactly like UI sends)
    info!("🔍 Testing UI query with range filter for key '1'");
    
    let ui_query = Query::new_with_filter(
        "TestRangeSchema".to_string(),
        vec!["test_data".to_string(), "test_id".to_string()],
        "web-ui".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "test_id": "1"
            }
        }))
    );

    let result1 = core_mut.query(ui_query).expect("Query for key '1' failed");
    info!("📋 UI Query result for key '1': {}", serde_json::to_string_pretty(&result1).unwrap());

    // Verify that we got the correct data for key '1'
    let test_data_result = result1.get("test_data").expect("test_data field missing");
    let expected_key_1_data = json!({"1": "a"});
    assert_eq!(test_data_result, &expected_key_1_data, "UI query for key '1' returned incorrect test_data");

    let test_id_result = result1.get("test_id").expect("test_id field missing");
    let expected_key_1_id = json!({"1": "1"});
    assert_eq!(test_id_result, &expected_key_1_id, "UI query for key '1' returned incorrect test_id");

    info!("✅ UI Query for key '1' PASSED: Correct data returned");

    // Test 2: Query with range filter for key "2" (exactly like UI sends)
    info!("🔍 Testing UI query with range filter for key '2'");
    
    let ui_query_2 = Query::new_with_filter(
        "TestRangeSchema".to_string(),
        vec!["test_data".to_string(), "test_id".to_string()],
        "web-ui".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "test_id": "2"
            }
        }))
    );

    let result2 = core_mut.query(ui_query_2).expect("Query for key '2' failed");
    info!("📋 UI Query result for key '2': {}", serde_json::to_string_pretty(&result2).unwrap());

    // Verify that we got the correct data for key '2'
    let test_data_result_2 = result2.get("test_data").expect("test_data field missing");
    let expected_key_2_data = json!({"2": "b"});
    assert_eq!(test_data_result_2, &expected_key_2_data, "UI query for key '2' returned incorrect test_data");

    let test_id_result_2 = result2.get("test_id").expect("test_id field missing");
    let expected_key_2_id = json!({"2": "2"});
    assert_eq!(test_id_result_2, &expected_key_2_id, "UI query for key '2' returned incorrect test_id");

    info!("✅ UI Query for key '2' PASSED: Correct data returned");

    // Test 3: Query without filter (should return all keys)
    info!("🔍 Testing UI query without range filter (should return all keys)");
    
    let query_all = Query::new(
        "TestRangeSchema".to_string(),
        vec!["test_data".to_string()],
        "web-ui".to_string(),
        0,
    );

    let result_all = core_mut.query(query_all).expect("Query without filter failed");
    info!("📋 UI Query result without filter: {}", serde_json::to_string_pretty(&result_all).unwrap());

    // Verify that we got data for all keys
    let test_data_all = result_all.get("test_data").expect("test_data field missing");
    
    // Should contain both keys
    assert!(test_data_all.get("1").is_some(), "All-keys query missing data for key '1'");
    assert!(test_data_all.get("2").is_some(), "All-keys query missing data for key '2'");
    assert_eq!(test_data_all.get("1").unwrap(), &json!("a"), "All-keys query incorrect data for key '1'");
    assert_eq!(test_data_all.get("2").unwrap(), &json!("b"), "All-keys query incorrect data for key '2'");

    info!("✅ UI Query without filter PASSED: All keys returned correctly");

    info!("✅ ALL UI Range Filter Tests PASSED!");
}