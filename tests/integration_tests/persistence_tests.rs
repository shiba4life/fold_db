use fold_node::testing::{
    FieldPaymentConfig, FieldType, PermissionsPolicy, Schema, SchemaField,
    TrustDistance, TrustDistanceScaling, Mutation, MutationType, Query,
};
use fold_node::{DataFoldNode, NodeConfig};
use fold_node::transform::{Transform, TransformParser};
use serde_json::json;
use std::collections::HashMap;
use tempfile::tempdir;

fn create_persistence_schema() -> Schema {
    let mut schema = Schema::new("PersistSchema".to_string());
    // Input field
    let input_field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    );
    schema.add_field("input_field".to_string(), input_field);

    // Transform field that depends on input_field
    let parser = TransformParser::new();
    let expr = parser.parse_expression("input_field + 1").unwrap();
    let transform = Transform::new_with_expr(
        "input_field + 1".to_string(),
        expr,
        "PersistSchema.transform_field".to_string(),
    );
    let transform_field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    )
    .with_transform(transform);
    schema.add_field("transform_field".to_string(), transform_field);

    schema
}

#[test]
fn test_db_and_transform_persistence() {
    // Create first node with a temporary directory
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let mut node = DataFoldNode::new(config.clone()).unwrap();

    // Load schema and insert a value
    let schema = create_persistence_schema();
    node.load_schema(schema).unwrap();

    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "PersistSchema".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("input_field".to_string(), json!(42))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation).unwrap();

    let query = Query {
        schema_name: "PersistSchema".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["input_field".to_string()],
        trust_distance: 1,
    };
    let results = node.query(query).unwrap();
    assert_eq!(results[0].as_ref().unwrap(), &json!(42));

    // Drop the first node to flush data to disk
    drop(node);
    // Load a new node from the same directory
    let mut node2 = DataFoldNode::new(config.clone()).unwrap();
    let schema_path = dir.path().join("schemas").join("PersistSchema.json");
    assert!(schema_path.exists());
    fold_node::datafold_node::loader::load_schema_from_file(&schema_path, &mut node2).unwrap();

    // Verify schema exists and transform persisted
    let loaded_schema = node2.get_schema("PersistSchema").unwrap().unwrap();
    let field = loaded_schema.fields.get("transform_field").unwrap();
    assert!(field.get_transform().is_some(), "Transform should persist across reload");

    // Verify stored data persists
    let query2 = Query {
        schema_name: "PersistSchema".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["input_field".to_string()],
        trust_distance: 1,
    };
    let results2 = node2.query(query2).unwrap();
    assert_eq!(results2[0].as_ref().unwrap(), &json!(42));
}

