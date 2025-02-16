use fold_db::testing::{
    SchemaPaymentConfig,
    FieldPaymentConfig,
    TrustDistanceScaling,
    TrustDistance,
    PermissionWrapper,
    PermissionsPolicy,
    SchemaManager,
    Mutation,
    Query,
    SchemaField,
    Schema,
    ExplicitCounts,
};
use serde_json::Value;
use std::collections::HashMap;

fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

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
            TrustDistance::Distance(0),
        ),
        create_default_payment_config(),
        HashMap::new(),
    ).with_ref_atom_uuid("test_ref".to_string())
    .with_field_mappers(HashMap::new());
    fields.insert("test_field".to_string(), field);

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        payment_config: SchemaPaymentConfig::default(),
    };

    schema_manager.load_schema(schema).unwrap();

    // Test cases
    let test_cases = vec![
        // Should pass - trust distance within policy
        (
            Query {
                schema_name: "test_schema".to_string(),
                fields: vec!["test_field".to_string()],
                pub_key: "test_key".to_string(),
                trust_distance: 1,
            },
            true,
        ),
        // Should fail - trust distance exceeds policy
        (
            Query {
                schema_name: "test_schema".to_string(),
                fields: vec!["test_field".to_string()],
                pub_key: "test_key".to_string(),
                trust_distance: 3,
            },
            false,
        ),
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
            TrustDistance::Distance(0),
        ),
        create_default_payment_config(),
        HashMap::new(),
    ).with_ref_atom_uuid("test_ref".to_string())
    .with_field_mappers(HashMap::new());
    fields.insert("test_field".to_string(), field);

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        payment_config: SchemaPaymentConfig::default(),
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
        assert!(
            result.allowed,
            "Query with distance {} should be allowed with NoRequirement",
            distance
        );
    }
}

#[test]
fn test_permission_wrapper_mutation() {
    // Setup
    let wrapper = PermissionWrapper::new();
    let schema_manager = SchemaManager::new();

    // Create a test schema with both explicit write permissions and trust distance
    let mut fields = HashMap::new();
    eprintln!("Setting up test with write policy: TrustDistance::Distance(2)");
    let mut policy = PermissionsPolicy::new(
        TrustDistance::Distance(0),
        TrustDistance::Distance(2), // Allow writes within trust distance 2
    );
    let mut explicit_counts = HashMap::new();
    explicit_counts.insert("allowed_key".to_string(), 1);
    policy.explicit_write_policy = Some(ExplicitCounts {
        counts_by_pub_key: explicit_counts,
    });
    let field = SchemaField::new(
        policy,
        create_default_payment_config(),
        HashMap::new(),
    ).with_ref_atom_uuid("test_ref".to_string())
    .with_field_mappers(HashMap::new());
    fields.insert("test_field".to_string(), field);

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        payment_config: SchemaPaymentConfig::default(),
    };

    schema_manager.load_schema(schema).unwrap();

    // Test cases for both success and failure scenarios
    let test_cases = vec![
        // Should pass - has explicit permission and within trust distance
        (
            Mutation {
                schema_name: "test_schema".to_string(),
                fields_and_values: {
                    let mut map = HashMap::new();
                    map.insert("test_field".to_string(), Value::Null);
                    map
                },
                pub_key: "allowed_key".to_string(),
                trust_distance: 1,
            },
            true,
        ),
        // Should fail - untrusted key and exceeds trust distance
        (
            Mutation {
                schema_name: "test_schema".to_string(),
                fields_and_values: {
                    let mut map = HashMap::new();
                    map.insert("test_field".to_string(), Value::Null);
                    map
                },
                pub_key: "untrusted_key".to_string(),
                trust_distance: 3,
            },
            false,
        ),
    ];

    for (mutation, should_pass) in test_cases {
        eprintln!("\nTesting mutation:");
        eprintln!("  pub_key: {}", mutation.pub_key);
        eprintln!("  trust_distance: {}", mutation.trust_distance);
        eprintln!("  expected: {}", should_pass);

        let result =
            wrapper.check_mutation_field_permission(&mutation, "test_field", &schema_manager);
        eprintln!("  got: {}", result.allowed);
        if let Some(err) = &result.error {
            eprintln!("  error: {:?}", err);
        }

        assert_eq!(
            result.allowed, should_pass,
            "Mutation permission check failed for pub_key: {}, trust_distance: {}",
            mutation.pub_key, mutation.trust_distance
        );
    }
}
