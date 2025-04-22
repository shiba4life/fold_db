use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

mod test_data;
use test_data::test_helpers::operation_builder::{create_mutation, create_query};
use test_data::test_helpers::schema_builder::{
    create_field_with_permissions, create_schema_with_fields,
};
use test_data::test_helpers::{cleanup_test_db, setup_and_allow_schema, setup_test_db};

#[test]
fn test_permission_based_access() {
    let (mut db, db_path) = setup_test_db();
    let owner_key = "owner_key".to_string();
    let reader_key = "reader_key".to_string();
    let unauthorized_key = "unauthorized_key".to_string();

    // Create schema with different permission levels for fields
    let mut fields = HashMap::new();

    // Public field - anyone can read, only owner can write
    let mut public_write_keys = HashMap::new();
    public_write_keys.insert(owner_key.clone(), 1u8);
    fields.insert(
        "public_field".to_string(),
        create_field_with_permissions(
            Uuid::new_v4().to_string(),
            5, // Very permissive read
            0, // Restrictive write
            None,
            Some(public_write_keys),
        ),
    );

    // Protected field - trusted users can read, explicit users can write
    let mut protected_read_keys = HashMap::new();
    protected_read_keys.insert(reader_key.clone(), 1u8);
    let mut protected_write_keys = HashMap::new();
    protected_write_keys.insert(owner_key.clone(), 1u8);
    fields.insert(
        "protected_field".to_string(),
        create_field_with_permissions(
            Uuid::new_v4().to_string(),
            2, // Moderate read restriction
            0, // Only explicit writers
            Some(protected_read_keys),
            Some(protected_write_keys),
        ),
    );

    let schema = create_schema_with_fields("test_schema".to_string(), fields);

    // Load and allow schema
    db.load_schema(schema).expect("Failed to load schema");
    setup_and_allow_schema(&mut db, "test_schema").expect("Failed to allow schema");

    // Test writing with owner key
    let mut owner_fields = HashMap::new();
    owner_fields.insert("public_field".to_string(), json!("public value"));
    owner_fields.insert("protected_field".to_string(), json!("protected value"));
    let owner_mutation = create_mutation(
        "test_schema".to_string(),
        owner_fields,
        owner_key.clone(),
        1,
    );
    assert!(db.write_schema(owner_mutation).is_ok());

    // Test reading with different keys and trust distances

    // Reader with explicit permission can read protected field
    let reader_query = create_query(
        "test_schema".to_string(),
        vec!["public_field".to_string(), "protected_field".to_string()],
        reader_key.clone(),
        3,
    );
    let reader_results = db.query_schema(reader_query.clone());
    let reader_results: HashMap<String, _> = reader_query
        .fields
        .iter()
        .cloned()
        .zip(reader_results)
        .collect();

    assert!(reader_results.get("public_field").unwrap().is_ok()); // Can read public field
    assert!(reader_results.get("protected_field").unwrap().is_ok()); // Can read protected field due to explicit permission

    // Unauthorized user can read public field but not protected field
    let unauth_query = create_query(
        "test_schema".to_string(),
        vec!["public_field".to_string(), "protected_field".to_string()],
        unauthorized_key.clone(),
        3,
    );
    let unauth_results = db.query_schema(unauth_query.clone());
    let unauth_results: HashMap<String, _> = unauth_query
        .fields
        .iter()
        .cloned()
        .zip(unauth_results)
        .collect();

    assert!(unauth_results.get("public_field").unwrap().is_ok()); // Can read public field
    assert!(unauth_results.get("protected_field").unwrap().is_err()); // Cannot read protected field

    // Test unauthorized write attempt
    let mut unauth_fields = HashMap::new();
    unauth_fields.insert("public_field".to_string(), json!("unauthorized write"));
    let unauth_mutation = create_mutation(
        "test_schema".to_string(),
        unauth_fields,
        unauthorized_key,
        1,
    );
    assert!(db.write_schema(unauth_mutation).is_err());

    cleanup_test_db(&db_path);
}
