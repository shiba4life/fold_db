use fold_node::fold_db_core::FoldDB;
use fold_node::schema::types::field::{FieldVariant, RangeField};
use fold_node::schema::types::{Mutation, MutationType, Query, Schema, SchemaError};
use fold_node::testing::*;
use serde_json::json;
use std::collections::HashMap;

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
    fields.insert(
        "score".to_string(),
        json!({
            "123": {"points": 42}
        }),
    );

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
    assert!(
        result.is_ok(),
        "Non-range schemas should not validate range filters"
    );
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
    assert!(
        result.is_ok(),
        "Valid range schema mutation should succeed: {:?}",
        result
    );
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

    // This should fail because RangeSchema cannot contain Single fields
    let schema_result = fold_db.add_schema_available(schema);
    assert!(
        schema_result.is_err(),
        "Adding mixed field types to RangeSchema should fail"
    );

    // Verify the error message indicates the issue
    let error_msg = format!("{:?}", schema_result.unwrap_err());
    assert!(
        error_msg.contains("ALL fields must be Range fields"),
        "Error should mention that all fields must be Range fields"
    );

    // Test ends here since schema creation should fail
    return;
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
    fields.insert(
        "score".to_string(),
        json!({
            "some_key": {"points": 42}
        }),
    ); // Missing user_id field entirely

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
            assert!(msg.contains("missing required range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidData error for missing range_key"),
    }
}

#[test]
fn test_range_schema_mutation_validation_null_range_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Add and approve the test schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();

    // Create mutation with null range_key field
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!(null)); // Null range_key
    fields.insert(
        "score".to_string(),
        json!({
            "points": 42
        }),
    );

    let mutation = Mutation::new(
        "user_scores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    // This should fail due to null range_key
    let result = fold_db.write_schema(mutation);
    assert!(result.is_err());

    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("null value for range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidData error for null range_key"),
    }
}

#[test]
fn test_range_schema_mutation_validation_empty_string_range_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Add and approve the test schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();

    // Create mutation with empty string range_key field
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("")); // Empty string range_key
    fields.insert(
        "score".to_string(),
        json!({
            "points": 42
        }),
    );

    let mutation = Mutation::new(
        "user_scores".to_string(),
        fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    // This should fail due to empty string range_key
    let result = fold_db.write_schema(mutation);
    assert!(result.is_err());

    match result.unwrap_err() {
        SchemaError::InvalidData(msg) => {
            assert!(msg.contains("empty string value for range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidData error for empty string range_key"),
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
        })),
    );

    // This should route to query_range_schema and return grouped results
    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1, "Should return one grouped result");

    let result = &results[0];
    assert!(
        result.is_ok(),
        "Range schema query should succeed: {:?}",
        result
    );

    // The result should be a grouped object
    let grouped_result = result.as_ref().unwrap();
    assert!(
        grouped_result.is_object(),
        "Result should be a grouped object"
    );

    let grouped_obj = grouped_result.as_object().unwrap();
    assert!(
        grouped_obj.contains_key("123"),
        "Should contain results grouped by user_id 123"
    );
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
        })),
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
    schema.add_field(
        "regular_field".to_string(),
        FieldVariant::Single(single_field),
    );

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
        })),
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
    mutation2_fields.insert(
        "score".to_string(),
        json!({
            "456": {"points": 75}
        }),
    );

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
        })),
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
        })),
    );

    let results2 = fold_db.query_schema(query2);
    assert_eq!(results2.len(), 1);
    let result2 = results2[0].as_ref().unwrap();
    let grouped2 = result2.as_object().unwrap();
    assert!(grouped2.contains_key("456"));
    assert!(!grouped2.contains_key("123"));
}

#[test]
fn test_range_schema_multiple_mutations_same_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // 1. Create and approve range schema
    let schema = create_test_range_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("user_scores").unwrap();

    // 2. Create multiple mutations for the same range key (user_id: 123)
    // With single-UUID-per-range-key behavior, only the latest mutation should be stored

    // First mutation - initial score
    let mut mutation1_fields = HashMap::new();
    mutation1_fields.insert("user_id".to_string(), json!(123));
    mutation1_fields.insert(
        "score".to_string(),
        json!({
            "123": {"points": 42, "level": "beginner"}
        }),
    );

    let mutation1 = Mutation::new(
        "user_scores".to_string(),
        mutation1_fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Create,
    );

    // Second mutation - update score for same user
    let mut mutation2_fields = HashMap::new();
    mutation2_fields.insert("user_id".to_string(), json!(123));
    mutation2_fields.insert(
        "score".to_string(),
        json!({
            "123": {"points": 75, "level": "intermediate"}
        }),
    );

    let mutation2 = Mutation::new(
        "user_scores".to_string(),
        mutation2_fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Update,
    );

    // Third mutation - another update for same user
    let mut mutation3_fields = HashMap::new();
    mutation3_fields.insert("user_id".to_string(), json!(123));
    mutation3_fields.insert(
        "score".to_string(),
        json!({
            "123": {"points": 100, "level": "advanced", "achievement": "gold"}
        }),
    );

    let mutation3 = Mutation::new(
        "user_scores".to_string(),
        mutation3_fields,
        "test_pubkey".to_string(),
        0,
        MutationType::Update,
    );

    // 3. Execute all mutations
    fold_db.write_schema(mutation1).unwrap();
    fold_db.write_schema(mutation2).unwrap();
    fold_db.write_schema(mutation3).unwrap();

    // 4. Query to verify all mutations are stored
    let query = Query::new_with_filter(
        "user_scores".to_string(),
        vec!["score".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": 123
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

    // 5. Verify the result contains all mutations
    let grouped_result = result.as_ref().unwrap();
    assert!(
        grouped_result.is_object(),
        "Result should be a grouped object"
    );

    let grouped_obj = grouped_result.as_object().unwrap();
    assert!(
        grouped_obj.contains_key("123"),
        "Should contain results grouped by user_id 123"
    );

    let user_data = &grouped_obj["123"];

    // Extract the score field which should contain only the latest mutation
    let score_data = user_data.get("score").expect("Should have score field");

    // With the new single-UUID-per-range-key behavior, only the last mutation should be stored
    if score_data.is_array() {
        let user_array = score_data.as_array().unwrap();
        assert_eq!(
            user_array.len(),
            1,
            "Should contain only the latest mutation for user 123 (single UUID per range key behavior)"
        );

        // Verify that we have the latest mutation (advanced level)
        let item = &user_array[0];
        if let Some(level) = item.get("level").and_then(|v| v.as_str()) {
            assert_eq!(level, "advanced", "Should contain only the latest (advanced) mutation");
            assert_eq!(item.get("points").and_then(|v| v.as_i64()), Some(100));
            assert!(item.get("achievement").is_some());
        } else {
            panic!("Expected level field in the mutation");
        }
    } else {
        // If it's a single object, verify it's the latest mutation
        if let Some(level) = score_data.get("level").and_then(|v| v.as_str()) {
            assert_eq!(level, "advanced", "Should contain only the latest (advanced) mutation");
            assert_eq!(score_data.get("points").and_then(|v| v.as_i64()), Some(100));
            assert!(score_data.get("achievement").is_some());
        } else {
            panic!("Expected level field in the mutation");
        }
    }
}

// ===================== ADDITIONAL COMPREHENSIVE TESTS =====================

/// Create a complex range schema matching UserScores.json structure
fn create_user_scores_schema() -> Schema {
    let mut schema = Schema::new_range("UserScores".to_string(), "user_id".to_string());

    for field_name in &[
        "user_id",
        "game_scores",
        "achievements",
        "player_statistics",
        "ranking_data",
    ] {
        let range_field = RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );
        schema.add_field(field_name.to_string(), FieldVariant::Range(range_field));
    }

    schema
}

#[test]
fn test_user_scores_schema_complex_mutations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup UserScores-like schema
    let schema = create_user_scores_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create complex mutation with realistic gaming data
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("gamer_alice"));

    fields.insert(
        "game_scores".to_string(),
        json!({
            "gamer_alice": {
                "tetris": 95000,
                "pacman": 15400,
                "chess": 1850,
                "snake": 12300,
                "pong": 999
            }
        }),
    );

    fields.insert(
        "achievements".to_string(),
        json!({
            "gamer_alice": {
                "first_win": "2024-01-15T10:30:00Z",
                "high_score_tetris": "2024-02-20T14:45:00Z",
                "speed_demon": "2024-03-10T16:20:00Z",
                "perfectionist": "2024-04-05T09:15:00Z",
                "grand_master": "2024-05-01T20:00:00Z"
            }
        }),
    );

    fields.insert(
        "player_statistics".to_string(),
        json!({
            "gamer_alice": {
                "total_games": 1247,
                "total_playtime_hours": 156.5,
                "win_rate": 0.78,
                "avg_session_minutes": 18.3,
                "favorite_game": "tetris",
                "longest_streak": 23,
                "current_streak": 5
            }
        }),
    );

    fields.insert(
        "ranking_data".to_string(),
        json!({
            "gamer_alice": {
                "global_rank": 1542,
                "regional_rank": 89,
                "tier": "Diamond",
                "rating": 2150,
                "rank_change": "+45",
                "last_updated": "2024-05-15T12:00:00Z"
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
        "Complex UserScores mutation should succeed: {:?}",
        result
    );

    // Query all fields for the user
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec![
            "game_scores".to_string(),
            "achievements".to_string(),
            "player_statistics".to_string(),
            "ranking_data".to_string(),
        ],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "gamer_alice"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1);
    let result = results[0].as_ref().unwrap();
    let grouped_obj = result.as_object().unwrap();
    let user_data = grouped_obj.get("gamer_alice").unwrap().as_object().unwrap();

    // Verify all complex data is preserved
    assert!(user_data.contains_key("game_scores"));
    assert!(user_data.contains_key("achievements"));
    assert!(user_data.contains_key("player_statistics"));
    assert!(user_data.contains_key("ranking_data"));

    // Verify specific data integrity
    let game_scores = user_data.get("game_scores").unwrap().as_object().unwrap();
    assert_eq!(game_scores.get("tetris").unwrap().as_i64(), Some(95000));

    let stats = user_data
        .get("player_statistics")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(stats.get("total_games").unwrap().as_i64(), Some(1247));
    assert_eq!(stats.get("win_rate").unwrap().as_f64(), Some(0.78));
}

#[test]
fn test_range_schema_query_performance_large_dataset() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_user_scores_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutations for many users
    for i in 0..100 {
        let user_id = format!("user_{:03}", i);
        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), json!(user_id.clone()));

        fields.insert(
            "game_scores".to_string(),
            json!({
                user_id.clone(): {
                    "tetris": (i * 1000) + 5000,
                    "pacman": (i * 100) + 2000,
                    "chess": 1000 + (i * 10)
                }
            }),
        );

        fields.insert(
            "player_statistics".to_string(),
            json!({
                user_id: {
                    "total_games": i * 10 + 50,
                    "win_rate": 0.5 + (i as f64 / 200.0),
                    "ranking": 5000 - (i * 10)
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

        fold_db.write_schema(mutation).unwrap();
    }

    // Query specific users and verify performance
    let test_cases = ["user_025", "user_050", "user_075"];

    for user_id in &test_cases {
        let query = Query::new_with_filter(
            "UserScores".to_string(),
            vec!["game_scores".to_string(), "player_statistics".to_string()],
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
            grouped_obj.contains_key(*user_id),
            "Should contain data for {}",
            user_id
        );
        assert_eq!(
            grouped_obj.len(),
            1,
            "Should only contain data for queried user"
        );
    }
}

#[test]
fn test_range_schema_different_data_types_comprehensive() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_user_scores_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create mutation with comprehensive data types
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), json!("data_types_test"));

    fields.insert(
        "game_scores".to_string(),
        json!({
            "data_types_test": {
                "integer_score": 12345,
                "float_score": 98.75,
                "string_score": "SSS+",
                "boolean_completed": true,
                "false_boolean": false,
                "null_value": null,
                "zero_value": 0,
                "negative_value": -500,
                "large_number": 999999999,
                "scientific_notation": 1.23e+6,
                "unicode_string": "🎮 Gaming Score! 日本語",
                "empty_string": "",
                "array_simple": [100, 200, 300],
                "array_mixed": [1, "two", 3.0, true, null],
                "array_nested": [[1, 2], [3, 4]],
                "object_nested": {
                    "level": 50,
                    "xp": 75000,
                    "skills": {
                        "speed": 95,
                        "accuracy": 88
                    }
                },
                "object_empty": {},
                "array_empty": []
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
        "Mutation with comprehensive data types should succeed: {:?}",
        result
    );

    // Query and verify all data types are preserved
    let query = Query::new_with_filter(
        "UserScores".to_string(),
        vec!["game_scores".to_string()],
        "test_pubkey".to_string(),
        0,
        Some(json!({
            "range_filter": {
                "user_id": "data_types_test"
            }
        })),
    );

    let results = fold_db.query_schema(query);
    assert_eq!(results.len(), 1);
    let result = results[0].as_ref().unwrap();
    let grouped_obj = result.as_object().unwrap();
    let user_data = grouped_obj
        .get("data_types_test")
        .unwrap()
        .as_object()
        .unwrap();
    let game_scores = user_data.get("game_scores").unwrap().as_object().unwrap();

    // Verify all data types
    assert_eq!(
        game_scores.get("integer_score").unwrap().as_i64(),
        Some(12345)
    );
    assert_eq!(
        game_scores.get("float_score").unwrap().as_f64(),
        Some(98.75)
    );
    assert_eq!(
        game_scores.get("string_score").unwrap().as_str(),
        Some("SSS+")
    );
    assert_eq!(
        game_scores.get("boolean_completed").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        game_scores.get("false_boolean").unwrap().as_bool(),
        Some(false)
    );
    assert!(game_scores.get("null_value").unwrap().is_null());
    assert_eq!(game_scores.get("zero_value").unwrap().as_i64(), Some(0));
    assert_eq!(
        game_scores.get("negative_value").unwrap().as_i64(),
        Some(-500)
    );
    assert_eq!(
        game_scores.get("unicode_string").unwrap().as_str(),
        Some("🎮 Gaming Score! 日本語")
    );
    assert_eq!(game_scores.get("empty_string").unwrap().as_str(), Some(""));

    // Verify arrays
    let array_simple = game_scores.get("array_simple").unwrap().as_array().unwrap();
    assert_eq!(array_simple.len(), 3);
    assert_eq!(array_simple[0].as_i64(), Some(100));

    let array_mixed = game_scores.get("array_mixed").unwrap().as_array().unwrap();
    assert_eq!(array_mixed.len(), 5);
    assert_eq!(array_mixed[0].as_i64(), Some(1));
    assert_eq!(array_mixed[1].as_str(), Some("two"));
    assert_eq!(array_mixed[2].as_f64(), Some(3.0));
    assert_eq!(array_mixed[3].as_bool(), Some(true));
    assert!(array_mixed[4].is_null());

    // Verify nested objects
    let nested_obj = game_scores
        .get("object_nested")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(nested_obj.get("level").unwrap().as_i64(), Some(50));
    let skills = nested_obj.get("skills").unwrap().as_object().unwrap();
    assert_eq!(skills.get("speed").unwrap().as_i64(), Some(95));

    // Verify empty containers
    assert!(game_scores
        .get("object_empty")
        .unwrap()
        .as_object()
        .unwrap()
        .is_empty());
    assert!(game_scores
        .get("array_empty")
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn test_range_schema_edge_cases_and_boundary_conditions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_user_scores_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Test edge cases for range_key values
    let edge_case_users = [
        "0",                              // Single character
        "123456789012345678901234567890", // Very long string
        "user with spaces",               // Spaces
        "user:with:colons",               // Special characters
        "user-with-dashes",               // Dashes
        "user_with_underscores",          // Underscores
        "user.with.dots",                 // Dots
        "用户名中文",                     // Unicode characters
        "🎮🎯🎲",                         // Emoji
        "MiXeD_CaSe_UsEr",                // Mixed case
        "123numeric_start",               // Starting with numbers
        "_underscore_start",              // Starting with underscore
        "trailing_underscore_",           // Ending with underscore
    ];

    for user_id in &edge_case_users {
        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), json!(*user_id));
        fields.insert(
            "game_scores".to_string(),
            json!({
                *user_id: {
                    "test_score": 1000,
                    "user_type": "edge_case"
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
            "Edge case user_id '{}' should work: {:?}",
            user_id,
            result
        );

        // Query to verify data was stored
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
            grouped_obj.contains_key(*user_id),
            "Should contain data for edge case user '{}'",
            user_id
        );
    }
}

#[test]
fn test_range_schema_concurrent_operations_stress_test() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_user_scores_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create many mutations for the same user (simulating concurrent updates)
    let user_id = "stress_test_user";
    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), json!(user_id));

        fields.insert(
            "game_scores".to_string(),
            json!({
                user_id: {
                    format!("session_{}", i): {
                        "score": i * 100,
                        "timestamp": format!("2024-01-01T{:02}:00:00Z", i % 24),
                        "session_id": i
                    }
                }
            }),
        );

        let mutation = Mutation::new(
            "UserScores".to_string(),
            fields,
            "test_pubkey".to_string(),
            0,
            if i == 0 {
                MutationType::Create
            } else {
                MutationType::Update
            },
        );

        let result = fold_db.write_schema(mutation);
        assert!(result.is_ok(), "Stress test mutation {} should succeed", i);
    }

    // Query and verify all data is accessible
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
    assert!(grouped_obj.contains_key(user_id));

    // Verify we can access data (structure may vary based on storage implementation)
    let user_data = grouped_obj.get(user_id).unwrap();
    assert!(!user_data.is_null(), "User data should not be null");
}

#[test]
fn test_range_schema_query_isolation_between_users() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let mut fold_db = FoldDB::new(db_path.to_str().unwrap()).unwrap();

    // Setup schema
    let schema = create_user_scores_schema();
    fold_db.add_schema_available(schema).unwrap();
    fold_db.approve_schema("UserScores").unwrap();

    // Create data for multiple users
    let users_data = [
        ("alice", "secret_alice_data", 9999),
        ("bob", "secret_bob_data", 8888),
        ("charlie", "secret_charlie_data", 7777),
    ];

    for (user_id, secret_data, score) in &users_data {
        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), json!(*user_id));
        fields.insert(
            "game_scores".to_string(),
            json!({
                *user_id: {
                    "secret_score": score,
                    "secret_data": secret_data,
                    "confidential_info": format!("This is private data for {}", user_id)
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

        fold_db.write_schema(mutation).unwrap();
    }

    // Query each user and verify isolation
    for (target_user, expected_secret, expected_score) in &users_data {
        let query = Query::new_with_filter(
            "UserScores".to_string(),
            vec!["game_scores".to_string()],
            "test_pubkey".to_string(),
            0,
            Some(json!({
                "range_filter": {
                    "user_id": target_user
                }
            })),
        );

        let results = fold_db.query_schema(query);
        assert_eq!(results.len(), 1);
        let result = results[0].as_ref().unwrap();
        let grouped_obj = result.as_object().unwrap();

        // Should only contain target user's data
        assert_eq!(grouped_obj.len(), 1, "Should only contain one user's data");
        assert!(
            grouped_obj.contains_key(*target_user),
            "Should contain target user"
        );

        // Verify no other users' data is present
        for (other_user, _, _) in &users_data {
            if other_user != target_user {
                assert!(
                    !grouped_obj.contains_key(*other_user),
                    "Should not contain other user '{}' when querying for '{}'",
                    other_user,
                    target_user
                );
            }
        }

        // Verify the correct data is returned
        let user_data = grouped_obj.get(*target_user).unwrap().as_object().unwrap();
        let game_scores = user_data.get("game_scores").unwrap().as_object().unwrap();
        assert_eq!(
            game_scores.get("secret_score").unwrap().as_i64(),
            Some(*expected_score as i64)
        );
        assert_eq!(
            game_scores.get("secret_data").unwrap().as_str(),
            Some(*expected_secret)
        );
    }
}
