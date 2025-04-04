use fold_node::testing::{
    FieldPaymentConfig,
    TrustDistanceScaling,
    PermissionsPolicy,
    TrustDistance,
    SchemaField,
    Mutation,
    Query,
    Schema,
    MutationType,
    FieldType,
};
use fold_node::{DataFoldNode, NodeConfig};
use serde_json::json;
use tempfile::tempdir;
use std::collections::HashMap;

fn create_test_node() -> DataFoldNode {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    DataFoldNode::new(config).unwrap()
}

fn create_test_schema() -> Schema {
    let mut schema = Schema::new("user_profile".to_string());

    // Add name field
    let name_field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    );
    schema.add_field("name".to_string(), name_field);

    // Add email field
    let email_field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    );
    schema.add_field("email".to_string(), email_field);

    schema
}

#[test]
fn test_node_schema_operations() {
    let mut node = create_test_node();
    let schema = create_test_schema();

    // Test schema loading
    assert!(node.load_schema(schema.clone()).is_ok());

    // Test schema retrieval
    let retrieved_schema = node.get_schema("user_profile").unwrap();
    assert!(retrieved_schema.is_some());
    assert_eq!(retrieved_schema.unwrap().name, "user_profile");
}

#[test]
fn test_node_data_operations() {
    let mut node = create_test_node();
    let schema = create_test_schema();

    // Load schema
    node.load_schema(schema).unwrap();
    node.allow_schema("user_profile").unwrap();

    // Test mutation
    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![
            ("name".to_string(), json!("John Doe")),
            ("email".to_string(), json!("john@example.com")),
        ]
        .into_iter()
        .collect(),
    };

    assert!(node.mutate(mutation).is_ok());

    // Test query
    let query = Query {
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["name".to_string(), "email".to_string()],
        trust_distance: 1,
    };

    let results = node.query(query).unwrap();
    assert_eq!(results.len(), 2);

    // Verify results
    for result in results {
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value == json!("John Doe") || value == json!("john@example.com"));
    }
}

#[test]
fn test_trust_distance_handling() {
    let mut node = create_test_node();

    // Test add trusted node
    assert!(node.add_trusted_node("test_node").is_ok());

    // Verify default trust distance is applied to queries
    let schema = create_test_schema();
    node.load_schema(schema).unwrap();
    node.allow_schema("user_profile").unwrap();

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
    let schema = create_test_schema();

    // Load schema
    node.load_schema(schema).unwrap();
    node.allow_schema("user_profile").unwrap();

    // Get initial schema to see the field's ref_atom_uuid
    let initial_schema = node.get_schema("user_profile").unwrap().unwrap();
    let initial_name_field = initial_schema.fields.get("name").unwrap();
    println!("Initial name field ref_atom_uuid: {:?}", initial_name_field.get_ref_atom_uuid());

    // Create initial data
    let mutation1 = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("name".to_string(), json!("John Doe"))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation1).unwrap();

    // Get schema after first mutation to check ref_atom_uuid
    let schema_after_create = node.get_schema("user_profile").unwrap().unwrap();
    let name_field_after_create = schema_after_create.fields.get("name").unwrap();
    println!("Name field ref_atom_uuid after create: {:?}", name_field_after_create.get_ref_atom_uuid());

    // Query current value
    let query1 = Query {
        schema_name: "user_profile".to_string(),
        fields: vec!["name".to_string()],
        pub_key: "test_key".to_string(),
        trust_distance: 1,
    };
    let results1 = node.query(query1).unwrap();
    println!("Value after create: {:?}", results1[0]);

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
    let query2 = Query {
        schema_name: "user_profile".to_string(),
        fields: vec!["name".to_string()],
        pub_key: "test_key".to_string(),
        trust_distance: 1,
    };
    let results2 = node.query(query2).unwrap();
    println!("Value after update: {:?}", results2[0]);

    // Get schema after update to check ref_atom_uuid
    let schema_after_update = node.get_schema("user_profile").unwrap().unwrap();
    let name_field_after_update = schema_after_update.fields.get("name").unwrap();
    println!("Name field ref_atom_uuid after update: {:?}", name_field_after_update.get_ref_atom_uuid());

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
