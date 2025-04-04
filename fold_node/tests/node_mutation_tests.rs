use std::collections::HashMap;
use serde_json::json;
use tempfile::tempdir;

use fold_node::{DataFoldNode, datafold_node::config::NodeConfig};
use fold_node::testing::{
    Schema, SchemaField, Mutation, MutationType,
    PermissionsPolicy, TrustDistance,
    FieldPaymentConfig,
    FieldType,
};

fn setup_test_node() -> DataFoldNode {
    let test_dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: test_dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    
    let mut node = DataFoldNode::new(config).expect("Failed to create test node");
    
    // Create test schema
    let mut test_schema = Schema::new("TestProfile".to_string());
    
    // Add username field
    let username_field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(0),
            TrustDistance::Distance(0)
        ),
        FieldPaymentConfig::default(),
        HashMap::new(),
        Some(FieldType::Single),
    );
    test_schema.add_field(String::from("username"), username_field);
    
    // Add age field
    let age_field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(0),
            TrustDistance::Distance(0)
        ),
        FieldPaymentConfig::default(),
        HashMap::new(),
        Some(FieldType::Single),
    );
    test_schema.add_field(String::from("age"), age_field);
    
    node.load_schema(test_schema).expect("Failed to load schema");
    node.allow_schema("TestProfile").expect("Failed to allow schema");
    
    node
}

#[test]
fn test_create_mutation() {
    let mut node = setup_test_node();
    
    // Test successful creation
    let mut fields_and_values = HashMap::new();
    fields_and_values.insert("username".to_string(), json!("testuser"));
    fields_and_values.insert("age".to_string(), json!(25));
    
    let create_mutation = Mutation::new(
        "TestProfile".to_string(),
        fields_and_values,
        String::new(), // Empty pub_key for testing
        0, // Default trust distance
        MutationType::Create,
    );
    
    let result = node.mutate(create_mutation);
    assert!(result.is_ok());
    
    // Test creation with erroneous required field
    let mut invalid_fields = HashMap::new();
    invalid_fields.insert("blahblah".to_string(), json!(25));
    
    let invalid_create = Mutation::new(
        "TestProfile".to_string(),
        invalid_fields,
        String::new(),
        0,
        MutationType::Create,
    );
    
    let result = node.mutate(invalid_create);
    assert!(result.is_err());
}

#[test]
fn test_update_mutation() {
    let mut node = setup_test_node();
    
    // First create an entry
    let mut create_fields = HashMap::new();
    create_fields.insert("username".to_string(), json!("updatetest"));
    create_fields.insert("age".to_string(), json!(30));
    
    let create_mutation = Mutation::new(
        "TestProfile".to_string(),
        create_fields,
        String::new(),
        0,
        MutationType::Create,
    );
    
    node.mutate(create_mutation).expect("Failed to create test data");
    
    // Test successful update
    let mut update_fields = HashMap::new();
    update_fields.insert("username".to_string(), json!("updatetest"));
    update_fields.insert("age".to_string(), json!(31));
    
    let update_mutation = Mutation::new(
        "TestProfile".to_string(),
        update_fields,
        String::new(),
        0,
        MutationType::Update,
    );
    
    let result = node.mutate(update_mutation);
    assert!(result.is_ok());
}

#[test]
fn test_delete_mutation() {
    let mut node = setup_test_node();
    
    // First create an entry
    let mut create_fields = HashMap::new();
    create_fields.insert("username".to_string(), json!("deletetest"));
    create_fields.insert("age".to_string(), json!(40));
    
    let create_mutation = Mutation::new(
        "TestProfile".to_string(),
        create_fields,
        String::new(),
        0,
        MutationType::Create,
    );
    
    node.mutate(create_mutation).expect("Failed to create test data");

    let mut delete_fields = HashMap::new();
    delete_fields.insert("username".to_string(), json!("deletetest"));
    delete_fields.insert("age".to_string(), json!(40));
    
    // Test successful deletion
    let delete_mutation = Mutation::new(
        "TestProfile".to_string(),
        delete_fields.clone(),
        String::new(),
        0,
        MutationType::Delete,
    );
    
    let result = node.mutate(delete_mutation);
    assert!(result.is_ok());
}

#[test]
fn test_schema_validation() {
    let mut node = setup_test_node();
    
    // Test unknown field
    let mut unknown_fields = HashMap::new();
    unknown_fields.insert("username".to_string(), json!("testuser"));
    unknown_fields.insert("unknown_field".to_string(), json!("value"));
    
    let unknown_field = Mutation::new(
        "TestProfile".to_string(),
        unknown_fields,
        String::new(),
        0,
        MutationType::Create,
    );
    
    let result = node.mutate(unknown_field);
    assert!(result.is_err());
}
