use fold_db::{
    permissions::{PermissionWrapper, PermissionsPolicy},
    schema::Schema,
    schema::schema_manager::SchemaManager,
    schema::types::{Query, Mutation, SchemaField},
};
use std::collections::HashMap;
use serde_json::Value;

#[test]
fn test_permission_wrapper_query() {
    // Setup
    let wrapper = PermissionWrapper::new();
    let schema_manager = SchemaManager::new();
    
    // Create a test schema
    let mut fields = HashMap::new();
    let field = SchemaField::new(
        PermissionsPolicy::new(2, 0), // Allow read within trust distance 2
        "test_ref".to_string(),
    );
    fields.insert("test_field".to_string(), field);
    
    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        transforms: Vec::new(),
    };
    
    schema_manager.load_schema(schema).unwrap();

    // Test cases
    let test_cases = vec![
        // Should pass - trust distance within policy
        (Query {
            schema_name: "test_schema".to_string(),
            fields: vec!["test_field".to_string()],
            pub_key: "test_key".to_string(),
            trust_distance: 1,
        }, true),
        // Should fail - trust distance exceeds policy
        (Query {
            schema_name: "test_schema".to_string(),
            fields: vec!["test_field".to_string()],
            pub_key: "test_key".to_string(),
            trust_distance: 3,
        }, false),
    ];

    for (query, should_pass) in test_cases {
        let results = wrapper.check_query_permissions(&query, &schema_manager);
        assert_eq!(results.len(), 1); // Should have one result per field
        assert_eq!(results[0].allowed, should_pass);
    }
}

#[test]
fn test_permission_wrapper_mutation() {
    // Setup
    let wrapper = PermissionWrapper::new();
    let schema_manager = SchemaManager::new();
    
    // Create a test schema with explicit write permissions
    let mut fields = HashMap::new();
    let mut policy = PermissionsPolicy::new(0, 0);
    let mut explicit_counts = HashMap::new();
    explicit_counts.insert("allowed_key".to_string(), 1);
    policy.explicit_write_policy = Some(fold_db::permissions::types::policy::ExplicitCounts {
        counts_by_pub_key: explicit_counts,
    });
    let field = SchemaField::new(policy, "test_ref".to_string());
    fields.insert("test_field".to_string(), field);
    
    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        transforms: Vec::new(),
    };
    
    schema_manager.load_schema(schema).unwrap();

    // Test cases
    let test_cases = vec![
        // Should pass - has explicit write permission
        (Mutation {
            schema_name: "test_schema".to_string(),
            fields_and_values: {
                let mut map = HashMap::new();
                map.insert("test_field".to_string(), Value::Null);
                map
            },
            pub_key: "allowed_key".to_string(),
            trust_distance: 0,
        }, true),
        // Should fail - no write permission
        (Mutation {
            schema_name: "test_schema".to_string(),
            fields_and_values: {
                let mut map = HashMap::new();
                map.insert("test_field".to_string(), Value::Null);
                map
            },
            pub_key: "unauthorized_key".to_string(),
            trust_distance: 0,
        }, false),
    ];

    for (mutation, should_pass) in test_cases {
        let results = wrapper.check_mutation_permissions(&mutation, &schema_manager);
        assert_eq!(results.len(), 1); // Should have one result per field
        assert_eq!(results[0].allowed, should_pass);
    }
}
