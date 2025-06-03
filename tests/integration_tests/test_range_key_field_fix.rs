//! Test to verify the fix for the range key field AtomRefRange issue
//! 
//! This test verifies that both range_key and non-range_key fields properly
//! store their AtomRefRange entries using the range_key VALUE as the key,
//! not the field names within the object.

use fold_node::fold_db_core::FoldDB;
use fold_node::schema::types::field::{FieldVariant, RangeField};
use fold_node::schema::types::{Mutation, MutationType, Query, Schema};
use fold_node::testing::*;
use serde_json::json;
use std::collections::HashMap;

/// Create a test schema that reproduces the original issue scenario
fn create_test_schema_with_test_id_and_test_data() -> Schema {
    let mut schema = Schema::new_range("test_schema".to_string(), "test_id".to_string());

    let range_field1 = RangeField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );
    let range_field2 = RangeField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );

    schema.add_field("test_id".to_string(), FieldVariant::Range(range_field1));
    schema.add_field("test_data".to_string(), FieldVariant::Range(range_field2));

    schema
}

/// Create a mutation that reproduces the original issue scenario
fn create_test_mutation() -> Mutation {
    let mut fields = HashMap::new();
    // Range key field - this should work correctly (baseline)
    fields.insert("test_id".to_string(), json!("abc"));
    // Non-range key field with object content - this was broken before the fix
    fields.insert("test_data".to_string(), json!({"test_id": "abc", "value": "123"}));

    Mutation::new(
        "test_schema".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    )
}

#[test]
fn test_range_key_field_fix_both_fields_return_matches() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_test_schema_with_test_id_and_test_data();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("test_schema").unwrap();

    // Execute mutation
    let mutation = create_test_mutation();
    let write_result = fold_db.write_schema(mutation);
    assert!(
        write_result.is_ok(),
        "Mutation should succeed: {:?}",
        write_result
    );

    // Query test_id field (range_key field) - this should find 1 match
    let test_id_query = Query::new_with_filter(
        "test_schema".to_string(),
        vec!["test_id".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "test_id": "abc"
            }
        })),
    );

    let test_id_results = fold_db.query_schema(test_id_query);
    assert_eq!(test_id_results.len(), 1, "test_id field should return 1 result");
    
    let test_id_result = &test_id_results[0];
    assert!(
        test_id_result.is_ok(),
        "test_id query should succeed: {:?}",
        test_id_result
    );

    // Verify test_id returns grouped results with our range key
    let test_id_grouped = test_id_result.as_ref().unwrap();
    assert!(
        test_id_grouped.is_object(),
        "test_id result should be grouped object"
    );
    
    let test_id_obj = test_id_grouped.as_object().unwrap();
    assert!(
        test_id_obj.contains_key("abc"),
        "test_id should contain results grouped by range key 'abc'"
    );

    println!("✅ test_id field: Found 1 match for range key 'abc'");

    // Query test_data field (non-range_key field) - this should also find 1 match after the fix
    let test_data_query = Query::new_with_filter(
        "test_schema".to_string(),
        vec!["test_data".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "test_id": "abc"
            }
        })),
    );

    let test_data_results = fold_db.query_schema(test_data_query);
    assert_eq!(test_data_results.len(), 1, "test_data field should return 1 result");
    
    let test_data_result = &test_data_results[0];
    assert!(
        test_data_result.is_ok(),
        "test_data query should succeed: {:?}",
        test_data_result
    );

    // Verify test_data returns grouped results with our range key
    let test_data_grouped = test_data_result.as_ref().unwrap();
    assert!(
        test_data_grouped.is_object(),
        "test_data result should be grouped object"
    );
    
    let test_data_obj = test_data_grouped.as_object().unwrap();
    assert!(
        test_data_obj.contains_key("abc"),
        "test_data should contain results grouped by range key 'abc' - THIS WAS THE BUG!"
    );

    // Verify the content is the full object that was stored
    let abc_data = &test_data_obj["abc"];
    println!("🔍 test_data content structure: {:?}", abc_data);
    
    assert!(
        abc_data.is_object(),
        "test_data content should be an object, got: {:?}",
        abc_data
    );
    
    let abc_data_obj = abc_data.as_object().unwrap();
    println!("🔍 test_data object keys: {:?}", abc_data_obj.keys().collect::<Vec<_>>());
    
    // The result is grouped by field name, so we need to access the "test_data" field
    assert!(
        abc_data_obj.contains_key("test_data"),
        "Result should contain the test_data field. Got: {:?}",
        abc_data_obj
    );
    
    let test_data_content = &abc_data_obj["test_data"];
    assert!(
        test_data_content.is_string(),
        "test_data field content should be a string containing the extracted value, got: {:?}",
        test_data_content
    );
    
    assert_eq!(
        test_data_content.as_str().unwrap(),
        "123",
        "test_data should contain the extracted value '123' after filtering out non-schema fields"
    );

    println!("✅ test_data field: Found 1 match for range key 'abc' with extracted value content");
    println!("🎉 Fix verified: Both range_key and non-range_key fields return matches for the same range key, with proper content extraction!");
}

#[test]
fn test_range_key_field_fix_correct_atomrefrange_structure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_test_schema_with_test_id_and_test_data();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("test_schema").unwrap();

    // Execute mutation
    let mutation = create_test_mutation();
    fold_db.write_schema(mutation).unwrap();

    // Verify both fields can be queried with the same range key
    let both_fields_query = Query::new_with_filter(
        "test_schema".to_string(),
        vec!["test_id".to_string(), "test_data".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "test_id": "abc"
            }
        })),
    );

    let results = fold_db.query_schema(both_fields_query);
    assert_eq!(results.len(), 1, "Should return 1 grouped result containing both fields");
    
    let result = &results[0];
    assert!(result.is_ok(), "Query should succeed: {:?}", result);

    let grouped = result.as_ref().unwrap();
    assert!(grouped.is_object(), "Result should be grouped object");
    
    let grouped_obj = grouped.as_object().unwrap();
    assert!(
        grouped_obj.contains_key("abc"),
        "Should contain results grouped by range key 'abc'"
    );

    let abc_group = &grouped_obj["abc"];
    assert!(abc_group.is_object(), "Group should be an object");
    
    let abc_group_obj = abc_group.as_object().unwrap();
    assert!(
        abc_group_obj.contains_key("test_id") && abc_group_obj.contains_key("test_data"),
        "Group should contain both test_id and test_data fields"
    );

    println!("✅ Both fields successfully grouped under the same range key 'abc'");
    println!("🎯 AtomRefRange structure is correct: range_key VALUE is used as key, not field names");
}