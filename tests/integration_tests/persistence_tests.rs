use fold_node::testing::{
    Field, FieldVariant, SingleField, Schema, Query,
};
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::schema::types::Transform;
use fold_node::transform::parser::TransformParser;
use fold_node::{DataFoldNode, NodeConfig};
use crate::test_data::test_helpers::node_operations::{load_and_allow, insert_value, query_value};
use serde_json::json;
use std::collections::HashMap;
use tempfile::tempdir;

fn create_persistence_schema() -> Schema {
    let mut schema = Schema::new("PersistSchema".to_string());
    // Input field
    let input_field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    ));
    schema.add_field("input_field".to_string(), input_field);

    // Transform field that depends on input_field
    let parser = TransformParser::new();
    let expr = parser.parse_expression("input_field + 1").unwrap();
    let transform = Transform::new_with_expr(
        "input_field + 1".to_string(),
        expr,
        "PersistSchema.transform_field".to_string(),
    );
    let mut transform_field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    transform_field.set_transform(transform);
    schema.add_field("transform_field".to_string(), FieldVariant::Single(transform_field));

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
    load_and_allow(&mut node, schema).unwrap();

    insert_value(&mut node, "PersistSchema", "input_field", json!(42)).unwrap();

    let result = query_value(&mut node, "PersistSchema", "input_field").unwrap();
    assert_eq!(result, json!(42));

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
    assert!(field.transform().is_some(), "Transform should persist across reload");

    // Verify stored data persists
    let query2 = Query {
        schema_name: "PersistSchema".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["input_field".to_string()],
        trust_distance: 1,
        filter: None,
    };
    let results2 = node2.query(query2).unwrap();
    assert_eq!(results2[0].as_ref().unwrap(), &json!(42));
}

