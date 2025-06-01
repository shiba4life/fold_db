use fold_node::testing::Schema;
use fold_node::schema::types::SchemaType;

#[test]
fn test_new_range_sets_range_key() {
    let schema = Schema::new_range("range_schema".to_string(), "user_id".to_string());
    assert_eq!(schema.name, "range_schema");
    match schema.schema_type {
        SchemaType::Range { ref range_key } => assert_eq!(range_key, "user_id"),
        _ => panic!("Expected Range variant"),
    }
    assert!(schema.fields.is_empty());
}

#[test]
fn test_new_range_serialization_preserves_key() {
    let schema = Schema::new_range("serialize_schema".to_string(), "custom_key".to_string());
    let json = serde_json::to_string(&schema).expect("serialize failed");
    let deserialized: Schema = serde_json::from_str(&json).expect("deserialize failed");
    match deserialized.schema_type {
        SchemaType::Range { ref range_key } => assert_eq!(range_key, "custom_key"),
        _ => panic!("Expected Range variant"),
    }
}
