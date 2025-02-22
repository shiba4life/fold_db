use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::test_data::test_helpers::operation_builder::{create_query, create_single_field_mutation};
use crate::test_data::test_helpers::schema_builder::{create_field_with_permissions, create_schema_with_fields};
use crate::test_data::test_helpers::{cleanup_test_db, setup_and_allow_schema, setup_test_db};

#[test]
fn test_schema_versioning_with_permissions() {
    let (mut db, db_path) = setup_test_db();
    let owner_key = "owner_key".to_string();
    let reader_key = "reader_key".to_string();

    // Create schema with versioned field
    let field_uuid = Uuid::new_v4().to_string();
    let mut write_keys = HashMap::new();
    write_keys.insert(owner_key.clone(), 1u8);

    let mut fields = HashMap::new();
    fields.insert(
        "versioned_field".to_string(),
        create_field_with_permissions(
            field_uuid.clone(),
            2, // Moderate read restriction
            0, // Only explicit writers
            None,
            Some(write_keys),
        ),
    );

    let schema = create_schema_with_fields(
        "test_schema".to_string(),
        fields,
    );

    // Load and allow schema
    db.load_schema(schema).expect("Failed to load schema");
    setup_and_allow_schema(&mut db, "test_schema").expect("Failed to allow schema");

    // Create multiple versions with owner key
    for i in 1..=3 {
        let mutation = create_single_field_mutation(
            "test_schema".to_string(),
            "versioned_field".to_string(),
            json!(format!("version {}", i)),
            owner_key.clone(),
            1,
        );
        assert!(db.write_schema(mutation).is_ok());
    }

    // Verify history access with appropriate trust distance
    let history = db
        .get_atom_history(&field_uuid)
        .expect("Failed to get history");
    assert_eq!(history.len(), 3);

    // Test reading latest version with different trust distances
    let trusted_query = create_query(
        "test_schema".to_string(),
        vec!["versioned_field".to_string()],
        reader_key.clone(),
        1, // Within trust distance
    );
    let trusted_results = db.query_schema(trusted_query.clone());
    let trusted_results: HashMap<String, _> = trusted_query
        .fields
        .iter()
        .cloned()
        .zip(trusted_results.into_iter())
        .collect();

    let versioned_result = trusted_results.get("versioned_field").unwrap();
    assert!(versioned_result.is_ok());
    assert_eq!(versioned_result.as_ref().unwrap(), &json!("version 3"));

    // Test reading with trust distance beyond limit
    let untrusted_query = create_query(
        "test_schema".to_string(),
        vec!["versioned_field".to_string()],
        reader_key,
        3, // Beyond trust distance
    );
    let untrusted_results = db.query_schema(untrusted_query.clone());
    let untrusted_results: HashMap<String, _> = untrusted_query
        .fields
        .iter()
        .cloned()
        .zip(untrusted_results.into_iter())
        .collect();

    assert!(untrusted_results.get("versioned_field").unwrap().is_err());

    cleanup_test_db(&db_path);
}
