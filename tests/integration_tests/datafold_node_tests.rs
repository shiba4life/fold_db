use fold_node::testing::{Mutation, MutationType, Query};
use serde_json::json;
use crate::test_data::test_helpers::create_test_node;
use crate::test_data::test_helpers::node_operations::{
    load_and_allow, insert_value, query_value,
};
use crate::test_data::schema_test_data::create_basic_user_profile_schema;

#[test]
fn test_node_schema_operations() {
    let mut node = create_test_node();
    let schema = create_basic_user_profile_schema();

    assert!(load_and_allow(&mut node, schema.clone()).is_ok());

    // Test schema retrieval
    let retrieved_schema = node.get_schema("user_profile").unwrap();
    assert!(retrieved_schema.is_some());
    assert_eq!(retrieved_schema.unwrap().name, "user_profile");
}

#[test]
fn test_node_data_operations() {
    let mut node = create_test_node();
    let schema = create_basic_user_profile_schema();
    load_and_allow(&mut node, schema).unwrap();

    // Test mutation
    insert_value(&mut node, "user_profile", "name", json!("John Doe")).unwrap();
    insert_value(&mut node, "user_profile", "email", json!("john@example.com"))
        .unwrap();

    // Test query
    let results = vec![
        query_value(&mut node, "user_profile", "name").unwrap(),
        query_value(&mut node, "user_profile", "email").unwrap(),
    ];
    assert_eq!(results.len(), 2);

    for value in results {
        assert!(value == json!("John Doe") || value == json!("john@example.com"));
    }
}

#[test]
fn test_trust_distance_handling() {
    let mut node = create_test_node();

    // Test add trusted node
    assert!(node.add_trusted_node("test_node").is_ok());

    // Verify default trust distance is applied to queries
    let schema = create_basic_user_profile_schema();
    load_and_allow(&mut node, schema).unwrap();

    let query = Query {
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["name".to_string()],
        trust_distance: 0, // Should be replaced with default
    };

    let results = node.query(query).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_version_history() {
    let mut node = create_test_node();
    let schema = create_basic_user_profile_schema();
    load_and_allow(&mut node, schema).unwrap();

    // Get initial schema to see the field's ref_atom_uuid
    let initial_schema = node.get_schema("user_profile").unwrap().unwrap();
    let initial_name_field = initial_schema.fields.get("name").unwrap();
    println!(
        "Initial name field ref_atom_uuid: {:?}",
        initial_name_field.get_ref_atom_uuid()
    );

    // Create initial data
    insert_value(&mut node, "user_profile", "name", json!("John Doe")).unwrap();

    // Get schema after first mutation to check ref_atom_uuid
    let schema_after_create = node.get_schema("user_profile").unwrap().unwrap();
    let name_field_after_create = schema_after_create.fields.get("name").unwrap();
    println!(
        "Name field ref_atom_uuid after create: {:?}",
        name_field_after_create.get_ref_atom_uuid()
    );

    // Query current value
    let result1 = query_value(&mut node, "user_profile", "name").unwrap();
    println!("Value after create: {:?}", result1);

    // Update data
    let mutation2 = Mutation {
        mutation_type: MutationType::Update,
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("name".to_string(), json!("Jane Doe"))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation2).unwrap();

    // Query updated value
    let result2 = query_value(&mut node, "user_profile", "name").unwrap();
    println!("Value after update: {:?}", result2);

    // Get schema after update to check ref_atom_uuid
    let schema_after_update = node.get_schema("user_profile").unwrap().unwrap();
    let name_field_after_update = schema_after_update.fields.get("name").unwrap();
    println!(
        "Name field ref_atom_uuid after update: {:?}",
        name_field_after_update.get_ref_atom_uuid()
    );

    // Get history using the actual ref_atom_uuid
    let history = node.get_history(&name_field_after_update.get_ref_atom_uuid().unwrap());
    println!("History result: {:?}", history);

    assert!(history.is_ok());

    // Verify history contents
    let history = history.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0], json!("Jane Doe")); // Most recent first
    assert_eq!(history[1], json!("John Doe"));
}
