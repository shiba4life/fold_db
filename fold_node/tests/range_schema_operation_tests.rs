use fold_node::fold_db_core::FoldDB;
use fold_node::schema::types::field::{FieldVariant, RangeField};
use fold_node::schema::types::{Mutation, MutationType, Query, Schema, SchemaError};
use fold_node::testing::*;
use serde_json::json;
use std::collections::HashMap;

/// Create a test range schema with multiple range fields
fn create_comprehensive_range_schema() -> Schema {
    let mut schema = Schema::new_range("UserScores".to_string(), "user_id".to_string());

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
    let range_field3 = RangeField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );
    let range_field4 = RangeField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    );

    schema.add_field("user_id".to_string(), FieldVariant::Range(range_field1));
    schema.add_field("game_scores".to_string(), FieldVariant::Range(range_field2));
    schema.add_field(
        "achievements".to_string(),
        FieldVariant::Range(range_field3),
    );
    schema.add_field("statistics".to_string(), FieldVariant::Range(range_field4));

    schema
}

/// Create a mutation with multiple range entries for the same user
fn create_range_mutation_single_user(user_id: &str) -> Mutation {
    let mut fields = HashMap::new();

    // Range key field provides the raw value
    fields.insert("user_id".to_string(), json!(user_id));

    // Other range fields with the range_key populated
    fields.insert(
        "game_scores".to_string(),
        json!({
            user_id: {
                "tetris": 85000,
                "pacman": 12500,
                "chess": 1200
            }
        }),
    );

    fields.insert(
        "achievements".to_string(),
        json!({
            user_id: {
                "first_win": "2024-01-15",
                "high_score": "2024-02-20",
                "speed_demon": "2024-03-10"
            }
        }),
    );

    fields.insert(
        "statistics".to_string(),
        json!({
            user_id: {
                "total_games": 450,
                "win_rate": 0.75,
                "avg_session_time": 25.5
            }
        }),
    );

    Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    )
}

/// Create a mutation for updating existing data
fn create_range_mutation_update(user_id: &str) -> Mutation {
    let mut fields = HashMap::new();

    fields.insert("user_id".to_string(), json!(user_id));

    // Update with new/modified data
    fields.insert(
        "game_scores".to_string(),
        json!({
            user_id: {
                "tetris": 90000, // Updated score
                "snake": 8750,   // New game
            }
        }),
    );

    fields.insert(
        "achievements".to_string(),
        json!({
            user_id: {
                "perfectionist": "2024-04-01" // New achievement
            }
        }),
    );

    Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Update,
    )
}

// ===================== RANGE SCHEMA MUTATION TESTS =====================

#[test]
fn test_range_schema_create_single_user_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation for single user
    let mutation = create_range_mutation_single_user("user_123");

    let result = fold_db.write_schema(mutation);
    assert!(
        result.is_ok(),
        "Range schema mutation should succeed: {:?}",
        result
    );
}

#[test]
fn test_range_schema_create_multiple_users() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutations for multiple users
    let users = ["user_123", "user_456", "user_789"];

    for user_id in &users {
        let mutation = create_range_mutation_single_user(user_id);
        let result = fold_db.write_schema(mutation);
        assert!(
            result.is_ok(),
            "Range schema mutation for {} should succeed: {:?}",
            user_id,
            result
        );
    }
}

#[test]
fn test_range_schema_update_existing_user() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create initial data
    let initial_mutation = create_range_mutation_single_user("user_123");
    fold_db.write_schema(initial_mutation).unwrap();

    // Update existing data
    let update_mutation = create_range_mutation_update("user_123");
    let result = fold_db.write_schema(update_mutation);
    assert!(
        result.is_ok(),
        "Range schema update mutation should succeed: {:?}",
        result
    );
}

#[test]
fn test_range_schema_different_data_types() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation with various data types
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("mixed_types_user"));

    fields.insert(
        "game_scores".to_string(),
        json!({
            "mixed_types_user": {
                "integer_score": 12345,
                "float_score": 98.75,
                "string_score": "SSS+",
                "boolean_completed": true,
                "null_value": null,
                "array_scores": [100, 200, 300],
                "nested_object": {
                    "level": 50,
                    "xp": 75000
                }
            }
        }),
    );

    let mutation = Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    let result = fold_db.write_schema(mutation);
    assert!(
        result.is_ok(),
        "Range schema mutation with mixed data types should succeed: {:?}",
        result
    );
}

#[test]
fn test_range_schema_large_dataset_mutation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create large dataset mutation
    let mut game_scores = serde_json::Map::new();
    for i in 0..100 {
        game_scores.insert(format!("game_{}", i), json!(i * 100));
    }

    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("heavy_gamer"));
    fields.insert(
        "game_scores".to_string(),
        json!({
            "heavy_gamer": game_scores
        }),
    );

    let mutation = Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    let result = fold_db.write_schema(mutation);
    assert!(
        result.is_ok(),
        "Range schema mutation with large dataset should succeed: {:?}",
        result
    );
}

// ===================== RANGE SCHEMA QUERY TESTS =====================

#[test]
fn test_range_schema_query_specific_user() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema and data
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Add data for multiple users
    let users = ["user_123", "user_456", "user_789"];
    for user_id in &users {
        let mutation = create_range_mutation_single_user(user_id);
        fold_db.write_schema(mutation).unwrap();
    }

    // Query specific user
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string(), "achievements".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "user_456"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");

    let result = &results[0];
    assert!(
        result.is_ok(),
        "Range schema query should succeed: {:?}",
        result
    );

    let grouped_result = result.as_ref().unwrap();
    assert!(
        grouped_result.is_object(),
        "Result should be a grouped object"
    );

    let grouped_obj = grouped_result.as_object().unwrap();
    assert!(
        grouped_obj.contains_key("user_456"),
        "Should contain results for user_456"
    );
    assert!(
        !grouped_obj.contains_key("user_123"),
        "Should not contain results for user_123"
    );
    assert!(
        !grouped_obj.contains_key("user_789"),
        "Should not contain results for user_789"
    );
}

#[test]
fn test_range_schema_query_multiple_fields() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema and data
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    let mutation = create_range_mutation_single_user("test_user");
    fold_db.write_schema(mutation).unwrap();

    // Query multiple fields
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec![
            "game_scores".to_string(),
            "achievements".to_string(),
            "statistics".to_string(),
        ],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "test_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");

    let result = &results[0];
    assert!(
        result.is_ok(),
        "Multi-field range schema query should succeed: {:?}",
        result
    );

    let grouped_result = result.as_ref().unwrap();
    let grouped_obj = grouped_result.as_object().unwrap();
    let user_data = grouped_obj.get("test_user").unwrap().as_object().unwrap();

    // Verify all requested fields are present
    assert!(
        user_data.contains_key("game_scores"),
        "Should contain game_scores"
    );
    assert!(
        user_data.contains_key("achievements"),
        "Should contain achievements"
    );
    assert!(
        user_data.contains_key("statistics"),
        "Should contain statistics"
    );
}

#[test]
fn test_range_schema_query_single_field() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema and data
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    let mutation = create_range_mutation_single_user("single_field_user");
    fold_db.write_schema(mutation).unwrap();

    // Query single field
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["achievements".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "single_field_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");

    let result = &results[0];
    assert!(
        result.is_ok(),
        "Single field range schema query should succeed: {:?}",
        result
    );

    let grouped_result = result.as_ref().unwrap();
    let grouped_obj = grouped_result.as_object().unwrap();
    let user_data = grouped_obj
        .get("single_field_user")
        .unwrap()
        .as_object()
        .unwrap();

    // Verify only requested field is present
    assert!(
        user_data.contains_key("achievements"),
        "Should contain achievements"
    );
    assert!(
        !user_data.contains_key("game_scores"),
        "Should not contain game_scores"
    );
    assert!(
        !user_data.contains_key("statistics"),
        "Should not contain statistics"
    );
}

#[test]
fn test_range_schema_query_nonexistent_user() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema and data
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    let mutation = create_range_mutation_single_user("existing_user");
    fold_db.write_schema(mutation).unwrap();

    // Query nonexistent user
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "nonexistent_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");

    let result = &results[0];
    assert!(
        result.is_ok(),
        "Query for nonexistent user should succeed but return empty: {:?}",
        result
    );

    let grouped_result = result.as_ref().unwrap();
    let grouped_obj = grouped_result.as_object().unwrap();
    assert!(
        grouped_obj.is_empty() || !grouped_obj.contains_key("nonexistent_user"),
        "Should not contain data for nonexistent user"
    );
}

#[test]
fn test_range_schema_query_after_update() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema and data
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create initial data
    let initial_mutation = create_range_mutation_single_user("update_test_user");
    fold_db.write_schema(initial_mutation).unwrap();

    // Update data
    let update_mutation = create_range_mutation_update("update_test_user");
    fold_db.write_schema(update_mutation).unwrap();

    // Query updated data
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string(), "achievements".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "update_test_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");

    let result = &results[0];
    assert!(
        result.is_ok(),
        "Query after update should succeed: {:?}",
        result
    );

    let grouped_result = result.as_ref().unwrap();
    let grouped_obj = grouped_result.as_object().unwrap();
    let user_data = grouped_obj
        .get("update_test_user")
        .unwrap()
        .as_object()
        .unwrap();

    // Verify updated data is present
    let game_scores = user_data.get("game_scores").unwrap().as_object().unwrap();
    assert!(
        game_scores.contains_key("tetris"),
        "Should contain updated tetris score"
    );
    assert!(
        game_scores.contains_key("snake"),
        "Should contain new snake score"
    );

    let achievements = user_data.get("achievements").unwrap().as_object().unwrap();
    assert!(
        achievements.contains_key("perfectionist"),
        "Should contain new achievement"
    );
}

// ===================== RANGE SCHEMA ERROR HANDLING TESTS =====================

#[test]
fn test_range_schema_query_missing_range_filter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Query without range_filter
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "other_filter": {
                "user_id": "test_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one result");

    // Should fall back to field-by-field processing
    let result = &results[0];
    assert!(
        result.is_ok(),
        "Should fall back to field query: {:?}",
        result
    );
}

#[test]
fn test_range_schema_query_invalid_range_filter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Query with invalid range_filter (wrong key)
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "wrong_key": "test_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one result");

    let result = &results[0];
    assert!(
        result.is_err(),
        "Query with invalid range_filter should fail"
    );

    match result.as_ref().unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("requires filter key 'user_id'"));
        }
        _ => panic!("Expected InvalidData error for invalid range_filter"),
    }
}

#[test]
fn test_range_schema_mutation_missing_range_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation without range_key
    let mut fields = HashMap::new();
    fields.insert(
        "game_scores".to_string(),
        json!({
            "some_user": {"tetris": 1000}
        }),
    );
    // Missing user_id field

    let mutation = Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    let result = fold_db.write_schema(mutation);
    assert!(result.is_err(), "Mutation missing range_key should fail");

    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("missing required range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidData error for missing range_key"),
    }
}

#[test]
fn test_range_schema_mutation_null_range_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation with null range_key
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!(null));
    fields.insert(
        "game_scores".to_string(),
        json!({
            "some_user": {"tetris": 1000}
        }),
    );

    let mutation = Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    let result = fold_db.write_schema(mutation);
    assert!(result.is_err(), "Mutation with null range_key should fail");

    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("null value for range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidData error for null range_key"),
    }
}

#[test]
fn test_range_schema_mutation_empty_string_range_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation with empty string range_key
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("   ")); // Whitespace only
    fields.insert(
        "game_scores".to_string(),
        json!({
            "some_user": {"tetris": 1000}
        }),
    );

    let mutation = Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    let result = fold_db.write_schema(mutation);
    assert!(
        result.is_err(),
        "Mutation with empty string range_key should fail"
    );

    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("empty string value for range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidData error for empty string range_key"),
    }
}

// ===================== INTEGRATION TESTS =====================

#[test]
fn test_end_to_end_range_schema_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // 1. Create and approve range schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // 2. Create initial data for multiple users
    let users = ["alice", "bob", "charlie"];
    for user_id in &users {
        let mutation = create_range_mutation_single_user(user_id);
        fold_db.write_schema(mutation).unwrap();
    }

    // 3. Update one user's data
    let update_mutation = create_range_mutation_update("alice");
    fold_db.write_schema(update_mutation).unwrap();

    // 4. Query specific user and verify data
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string(), "achievements".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "alice"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1);
    let result = results[0].as_ref().unwrap();
    let grouped_obj = result.as_object().unwrap();

    // Verify alice's data includes both original and updated data
    let alice_data = grouped_obj.get("alice").unwrap().as_object().unwrap();
    let game_scores = alice_data.get("game_scores").unwrap().as_object().unwrap();
    assert!(game_scores.contains_key("tetris")); // Updated
    assert!(game_scores.contains_key("snake")); // Added
    assert!(game_scores.contains_key("pacman")); // Original

    let achievements = alice_data.get("achievements").unwrap().as_object().unwrap();
    assert!(achievements.contains_key("perfectionist")); // Added
    assert!(achievements.contains_key("first_win")); // Original

    // 5. Query different user and verify isolation
    let bob_query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "bob"
            }
        })),
    );

    let bob_results = fold_db.query_schema(bob_query);
    let bob_result = bob_results[0].as_ref().unwrap();
    let bob_grouped_obj = bob_result.as_object().unwrap();
    let bob_data = bob_grouped_obj.get("bob").unwrap().as_object().unwrap();
    let bob_scores = bob_data.get("game_scores").unwrap().as_object().unwrap();

    // Bob should not have alice's updated data
    assert!(!bob_scores.contains_key("snake"));
    assert!(bob_scores.contains_key("tetris")); // But should have original data
}

#[test]
fn test_range_schema_concurrent_mutations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Simulate concurrent mutations for different users
    for i in 0..10 {
        let user_id = format!("user_{}", i);
        let mutation = create_range_mutation_single_user(&user_id);
        let result = fold_db.write_schema(mutation);
        assert!(result.is_ok(), "Concurrent mutation {} should succeed", i);
    }

    // Verify all users can be queried
    for i in 0..10 {
        let user_id = format!("user_{}", i);
        let query = Query::new_with_filter(
            "UserScores".to_string(),
            vec!["game_scores".to_string()],
            "test_pubkey".to_string(),
            0,
            Some(json!({
                "range_filter": {
                    "user_id": user_id
                }
            })),
        );

        let results = fold_db.query_schema(query);
        assert_eq!(results.len(), 1);
        let result = results[0].as_ref().unwrap();
        let grouped_obj = result.as_object().unwrap();
        assert!(
            grouped_obj.contains_key(&user_id),
            "Should contain data for {}",
            user_id
        );
    }
}

#[test]
fn test_range_schema_stress_test_large_mutations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_comprehensive_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation with large amounts of data
    let mut large_game_scores = serde_json::Map::new();
    for i in 0..1000 {
        large_game_scores.insert(
            format!("game_{:04}", i),
            json!({
                "score": i * 100,
                "level": i / 10,
                "timestamp": format!("2024-01-01T{:02}:00:00Z", i % 24)
            }),
        );
    }

    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("stress_test_user"));
    fields.insert(
        "game_scores".to_string(),
        json!({
            "stress_test_user": large_game_scores
        }),
    );

    let mutation = Mutation::new(
        "UserScores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    let result = fold_db.write_schema(mutation);
    assert!(
        result.is_ok(),
        "Large mutation should succeed: {:?}",
        result
    );

    // Query the large dataset
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "stress_test_user"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1);
    let result = results[0].as_ref().unwrap();
    let grouped_obj = result.as_object().unwrap();
    let user_data = grouped_obj
        .get("stress_test_user")
        .unwrap()
        .as_object()
        .unwrap();
    let game_scores = user_data.get("game_scores").unwrap().as_object().unwrap();

    assert_eq!(
        game_scores.len(),
        1000,
        "Should contain all 1000 game scores"
    );
}
