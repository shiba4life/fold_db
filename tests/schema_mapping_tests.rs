use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

mod test_helpers;
use test_helpers::{setup_test_db, cleanup_test_db, setup_and_allow_schema};
use test_helpers::schema_builder::{create_field_with_permissions, create_schema_with_fields, create_rename_mapper};
use test_helpers::operation_builder::{create_query, create_single_field_mutation};

#[test]
fn test_schema_mapping_and_data_propagation() {
    let (mut db, db_path) = setup_test_db();
    let test_key = "test_key".to_string();

    // Create source schema
    let source_field_uuid = Uuid::new_v4().to_string();
    let mut write_keys = HashMap::new();
    write_keys.insert(test_key.clone(), 1u8);
    
    let mut source_fields = HashMap::new();
    source_fields.insert(
        "user_name".to_string(),
        create_field_with_permissions(
            source_field_uuid.clone(),
            1, // read distance
            0, // write distance
            None, // explicit read keys
            Some(write_keys.clone()), // explicit write keys
        )
    );

    let source_schema = create_schema_with_fields(
        "source_schema".to_string(),
        source_fields,
        vec![] // no mappers
    );

    // Load and allow source schema
    db.load_schema(source_schema).expect("Failed to load source schema");
    setup_and_allow_schema(&mut db, "source_schema").expect("Failed to allow source schema");

    // Write initial data to source schema
    let source_mutation = create_single_field_mutation(
        "source_schema".to_string(),
        "user_name".to_string(),
        json!("initial_username"),
        test_key.clone(),
        1
    );
    assert!(db.write_schema(source_mutation).is_ok());

    // Create target schema with mapping
    let mut target_fields = HashMap::new();
    target_fields.insert(
        "username".to_string(),
        create_field_with_permissions(
            source_field_uuid.clone(), // Share the same atom UUID
            1, // read distance
            0, // write distance
            None, // explicit read keys
            Some(write_keys), // explicit write keys
        )
    );

    let target_schema = create_schema_with_fields(
        "target_schema".to_string(),
        target_fields,
        vec![
            create_rename_mapper(
                "source_schema".to_string(),
                "target_schema".to_string(),
                "user_name".to_string(),
                "username".to_string(),
            )
        ]
    );

    // Load and allow target schema
    db.load_schema(target_schema).expect("Failed to load target schema");
    setup_and_allow_schema(&mut db, "target_schema").expect("Failed to allow target schema");

    // Verify target schema received mapped data from source schema
    let target_query = create_query(
        "target_schema".to_string(),
        vec!["username".to_string()],
        test_key.clone(),
        1
    );
    
    let target_results = db.query_schema(target_query.clone());
    let target_results: HashMap<String, _> = target_query.fields.iter().cloned()
        .zip(target_results.into_iter())
        .collect();
    
    let username_result = target_results.get("username").unwrap();
    assert!(username_result.is_ok());
    assert_eq!(username_result.as_ref().unwrap(), &json!("initial_username"));

    cleanup_test_db(&db_path);
}
