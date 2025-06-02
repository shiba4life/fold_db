use std::collections::HashMap;
use serde_json::json;
use fold_node::testing::*;
use fold_node::fold_db_core::FoldDB;
use fold_node::schema::types::{Query, Mutation, MutationType, Schema, SchemaError};
use fold_node::schema::types::field::{FieldVariant, RangeField};

/// Create a test range schema with user_id as range_key
fn create_test_range_schema() -> Schema {
    let mut schema = Schema::new_range("user_scores".to_string(), "user_id".to_string());
    
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
    
    schema.add_field("user_id".to_string(), FieldVariant::Range(range_field1));
    schema.add_field("score".to_string(), FieldVariant::Range(range_field2));
    
    schema
}

/// Create a mutation for range schema with consistent range_key values
fn create_test_range_mutation() -> Mutation {
    let mut fields = HashMap::new();
    // Range key field provides the raw value for to_range_schema_mutation()
    fields.insert("user_id".to_string(), json!(123));
    // Other range fields already have the range_key populated
    fields.insert("score".to_string(), json!({
        "123": {"points": 42}
    }));
    
    Mutation::new(
        "user_scores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    )
}

#[test]
fn test_schema_validate_range_filter_success() {
    let schema = create_test_range_schema();
    
    let valid_filter = json!({
        "range_filter": {
            "user_id": 123
        }
    });
    
    let result = schema.validate_range_filter(&valid_filter);
    assert!(result.is_ok(), "Valid range filter should pass validation");
}

#[test]
fn test_schema_validate_range_filter_missing_range_filter() {
    let schema = create_test_range_schema();
    
    let invalid_filter = json!({
        "other_filter": {
            "user_id": 123
        }
    });
    
    let result = schema.validate_range_filter(&invalid_filter);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("requires a 'range_filter'"));
        }
        _ => panic!("Expected InvalidData error for missing range_filter"),
    }
}

#[test]
fn test_schema_validate_range_filter_wrong_key() {
    let schema = create_test_range_schema();
    
    let invalid_filter = json!({
        "range_filter": {
            "wrong_key": 123
        }
    });
    
    let result = schema.validate_range_filter(&invalid_filter);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("requires filter key 'user_id'"));
        }
        _ => panic!("Expected InvalidData error for wrong key"),
    }
}

#[test]
fn test_schema_validate_range_filter_multiple_keys() {
    let schema = create_test_range_schema();
    
    let invalid_filter = json!({
        "range_filter": {
            "user_id": 123,
            "extra_key": "value"
        }
    });
    
    let result = schema.validate_range_filter(&invalid_filter);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("should only contain 'user_id'"));
        }
        _ => panic!("Expected InvalidData error for multiple keys"),
    }
}

#[test]
fn test_schema_validate_range_filter_non_range_schema() {
    let non_range_schema = Schema::new("regular_schema".to_string());
    
    let filter = json!({
        "range_filter": {
            "user_id": 123
        }
    });
    
    // Non-range schemas should pass validation (no validation needed)
    let result = non_range_schema.validate_range_filter(&filter);
    assert!(result.is_ok(), "Non-range schemas should not validate range filters");
}


#[test]
fn test_range_schema_mutation_validation_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // Add and approve the test schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();
    
    // Create valid range mutation
    let mutation = create_test_range_mutation();
    
    // This should succeed - validation happens inside write_schema
    let result = fold_db.write_schema(mutation);
    assert!(result.is_ok(), "Valid range schema mutation should succeed: {:?}", result);
}

#[test]
fn test_range_schema_mutation_validation_mixed_field_types() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // Create schema with mixed field types (should be invalid for Range schema)
    let mut schema = Schema::new_range("mixed_schema".to_string(), "user_id".to_string());
    
    let range_field = RangeField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(), 
        HashMap::new(),
    );
    let single_field = fold_node::schema::types::field::SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );
    
    schema.add_field("user_id".to_string(), FieldVariant::Range(range_field));
    schema.add_field("bad_field".to_string(), FieldVariant::Single(single_field));
    
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("mixed_schema").unwrap();
    
    // Create mutation for mixed schema
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!(123));
    fields.insert("bad_field".to_string(), json!({
        "some_key": "value"
    }));
    
    let mutation = Mutation::new(
        "mixed_schema".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );
    
    // This should fail due to mixed field types
    let result = fold_db.write_schema(mutation);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("all fields must be RangeFields"));
        }
        _ => panic!("Expected InvalidData error for mixed field types"),
    }
}

#[test]
fn test_range_schema_mutation_validation_missing_range_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // Add and approve the test schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();
    
    // Create mutation missing range_key field
    let mut fields = HashMap::new();
    fields.insert("score".to_string(), json!({
        "some_key": {"points": 42}
    })); // Missing user_id field entirely
    
    let mutation = Mutation::new(
        "user_scores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );
    
    // This should fail due to missing range_key
    let result = fold_db.write_schema(mutation);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("missing range_key field"));
        }
        _ => panic!("Expected InvalidData error for missing range_key"),
    }
}

#[test] 
fn test_query_routing_range_schema_with_range_filter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // Add and approve the test schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();
    
    // Create and execute mutation to populate data
    let mutation = create_test_range_mutation();
    fold_db.write_schema(mutation).unwrap();
    
    // Create query with range_filter - should route to range schema query
    let query = Query::new_with_filter(
        "user_scores".to_string(),
        vec!["score".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": 123
            }
        }))
    );
    
    // This should route to query_range_schema and return grouped results
    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");
    
    let result = &results[0];
    assert!(result.is_ok(), "Range schema query should succeed: {:?}", result);
    
    // The result should be a grouped object
    let grouped_result = result.as_ref().unwrap();
    assert!(grouped_result.is_object(), "Result should be a grouped object");
    
    let grouped_obj = grouped_result.as_object().unwrap();
    assert!(grouped_obj.contains_key("123"), "Should contain results grouped by user_id 123");
}

#[test]
fn test_query_routing_range_schema_without_range_filter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // Add and approve the test schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();
    
    // Create and execute mutation to populate data
    let mutation = create_test_range_mutation();
    fold_db.write_schema(mutation).unwrap();
    
    // Create query without range_filter - should fall back to field-by-field processing
    let query = Query::new_with_filter(
        "user_scores".to_string(),
        vec!["score".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "other_filter": {
                "some_key": "value"
            }
        }))
    );
    
    // This should fall back to original field-by-field processing
    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one field result");
    
    let result = &results[0];
    assert!(result.is_ok(), "Field query should succeed: {:?}", result);
}

#[test]
fn test_query_routing_non_range_schema() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // Add and approve a non-range schema
    let mut schema = Schema::new("regular_schema".to_string());
    let single_field = fold_node::schema::types::field::SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );
    schema.add_field("regular_field".to_string(), FieldVariant::Single(single_field));
    
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("regular_schema").unwrap();
    
    // Create query for non-range schema
    let query = Query::new_with_filter(
        "regular_schema".to_string(),
        vec!["regular_field".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": 123
            }
        }))
    );
    
    // This should fall back to original field-by-field processing
    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one field result");
}

#[test]
fn test_full_range_schema_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();
    
    // 1. Create and approve range schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();
    
    // 2. Create multiple mutations for different users
    let mutation1 = create_test_range_mutation();
    
    let mut mutation2_fields = HashMap::new();
    mutation2_fields.insert("user_id".to_string(), json!(456));
    mutation2_fields.insert("score".to_string(), json!({
        "456": {"points": 75}
    }));
    
    let mutation2 = Mutation::new(
        "user_scores".to_string(),
        mutation2_fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );
    
    // 3. Execute mutations
    fold_db.write_schema(mutation1).unwrap();
    fold_db.write_schema(mutation2).unwrap();
    
    // 4. Query for specific user (123)
    let query1 = Query::new_with_filter(
        "user_scores".to_string(),
        vec!["score".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": 123
            }
        }))
    );
    
    let results1 = fold_db.query_schema(query1);
    assert_eq!(results1.len(), 1);
    let result1 = results1[0].as_ref().unwrap();
    let grouped1 = result1.as_object().unwrap();
    assert!(grouped1.contains_key("123"));
    assert!(!grouped1.contains_key("456"));
    
    // 5. Query for different user (456)
    let query2 = Query::new_with_filter(
        "user_scores".to_string(),
        vec!["score".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": 456
            }
        }))
    );
    
    let results2 = fold_db.query_schema(query2);
    assert_eq!(results2.len(), 1);
    let result2 = results2[0].as_ref().unwrap();
    let grouped2 = result2.as_object().unwrap();
    assert!(grouped2.contains_key("456"));
    assert!(!grouped2.contains_key("123"));
}