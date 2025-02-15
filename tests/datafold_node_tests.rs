 use fold_db::testing::{
    FieldPaymentConfig,
    TrustDistanceScaling,
    PermissionsPolicy,
    TrustDistance,
    SchemaField,
    Mutation,
    Query,
    Schema,
};
use fold_db::{DataFoldNode, NodeConfig};
use serde_json::json;
use tempfile::tempdir;
use uuid;
use std::collections::HashMap;

fn create_test_node() -> DataFoldNode {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        docker: fold_db::datafold_node::DockerConfig::default(),
    };
    DataFoldNode::new(config).unwrap()
}

fn create_test_schema() -> Schema {
    let mut schema = Schema::new("user_profile".to_string());

    // Add name field
    let name_field = SchemaField {
        ref_atom_uuid: Some(uuid::Uuid::new_v4().to_string()),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        payment_config: FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        field_mappers: HashMap::new(),
    };
    schema.add_field("name".to_string(), name_field);

    // Add email field
    let email_field = SchemaField {
        ref_atom_uuid: Some(uuid::Uuid::new_v4().to_string()),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        payment_config: FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        field_mappers: HashMap::new(),
    };
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

    // Create initial data
    let mutation1 = Mutation {
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("name".to_string(), json!("John Doe"))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation1).unwrap();

    // Update data
    let mutation2 = Mutation {
        schema_name: "user_profile".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("name".to_string(), json!("Jane Doe"))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation2).unwrap();

    // Get the schema to find the field's ref_atom_uuid
    let schema = node.get_schema("user_profile").unwrap().unwrap();
    let name_field = schema.fields.get("name").unwrap();

    // Get history using the actual ref_atom_uuid
    let history = node.get_history(name_field.ref_atom_uuid.as_ref().unwrap());
    assert!(history.is_ok());

    // Verify history contents
    let history = history.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0], json!("Jane Doe")); // Most recent first
    assert_eq!(history[1], json!("John Doe"));
}
