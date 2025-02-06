use fold_db::{
    permissions::{PermissionWrapper, PermissionsPolicy},
    permissions::types::policy::TrustDistance,
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
        PermissionsPolicy::new(
            TrustDistance::Distance(2), // Allow read within trust distance 2
            TrustDistance::Distance(0)
        ),
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
        let result = wrapper.check_query_field_permission(&query, "test_field", &schema_manager);
        assert_eq!(result.allowed, should_pass);
    }
}

#[test]
fn test_permission_wrapper_no_requirement() {
    // Setup
    let wrapper = PermissionWrapper::new();
    let schema_manager = SchemaManager::new();
    
    // Create a test schema with no distance requirement
    let mut fields = HashMap::new();
    let field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::NoRequirement, // No distance requirement for reads
            TrustDistance::Distance(0)
        ),
        "test_ref".to_string(),
    );
    fields.insert("test_field".to_string(), field);
    
    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        transforms: Vec::new(),
    };
    
    schema_manager.load_schema(schema).unwrap();

    // Test cases with varying trust distances - all should pass due to NoRequirement
    let test_distances = vec![0, 1, 5, 10, 100];
    
    for distance in test_distances {
        let query = Query {
            schema_name: "test_schema".to_string(),
            fields: vec!["test_field".to_string()],
            pub_key: "test_key".to_string(),
            trust_distance: distance,
        };
        
        let result = wrapper.check_query_field_permission(&query, "test_field", &schema_manager);
        assert!(result.allowed, "Query with distance {} should be allowed with NoRequirement", distance);
    }
}

#[test]
fn test_permission_wrapper_mutation() {
    // Setup
    let wrapper = PermissionWrapper::new();
    let schema_manager = SchemaManager::new();
    
    // Create a test schema with explicit write permissions
    let mut fields = HashMap::new();
    let mut policy = PermissionsPolicy::new(
        TrustDistance::Distance(0),
        TrustDistance::Distance(0)
    );
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
        let result = wrapper.check_mutation_field_permission(&mutation, "test_field", &schema_manager);
        assert_eq!(result.allowed, should_pass);
    }
}
