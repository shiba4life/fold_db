use std::sync::Arc;
use serde_json::json;

use fold_db::schema::{SchemaManager, InternalSchema, PolicyLevel, PermissionsPolicy, ExplicitCounts, Count};
use fold_db::store::Store;
use fold_db::api::{
    QueryPayload, QueryItem,
    WritePayload, WriteItem,
    query, write,
};

fn setup_test_db() -> (Arc<SchemaManager>, Arc<Store>) {
    let schema_manager = Arc::new(SchemaManager::new());
    let store = Arc::new(Store::new(&format!(":memory:{:?}", std::thread::current().id())).unwrap());

    // Create test schema with explicit permissions
    let mut schema = InternalSchema::new();
    schema.fields.insert("public_field".to_string(), "public-uuid".to_string());
    schema.fields.insert("explicit_field".to_string(), "explicit-uuid".to_string());
    schema.fields.insert("distance_field".to_string(), "distance-uuid".to_string());

    // Set up policies
    let mut policies = std::collections::HashMap::new();
    
    // Public field - anyone can access
    policies.insert("public_field".to_string(), PermissionsPolicy {
        read_policy: PolicyLevel::Anyone,
        write_policy: PolicyLevel::Anyone,
    });

    // Explicit field - requires explicit permission
    policies.insert("explicit_field".to_string(), PermissionsPolicy {
        read_policy: PolicyLevel::ExplicitOnce,
        write_policy: PolicyLevel::ExplicitOnce,
    });

    // Distance field - requires distance <= 2
    policies.insert("distance_field".to_string(), PermissionsPolicy {
        read_policy: PolicyLevel::Distance(2),
        write_policy: PolicyLevel::Distance(2),
    });

    schema.policies = Some(policies);

    // Set up explicit permissions for test_key
    schema.set_explicit_permissions(
        "test_key".to_string(),
        ExplicitCounts {
            r: Count::Limited(1),
            w: Count::Limited(1),
        },
    );

    schema_manager.load_schema("test", schema).unwrap();

    (schema_manager, store)
}

#[test]
fn test_public_field_permissions() {
    let (schema_manager, store) = setup_test_db();

    // Test read
    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "public_field".to_string(),
            }
        ],
        public_key: "any_key".to_string(),
        distance: Some(0),
    };

    let response = query(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert!(response.results[0].result.get("error").is_none(), 
        "Expected success but got error: {:?}", response.results[0].result);

    // Test write
    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteField {
                schema: "test".to_string(),
                field: "public_field".to_string(),
                value: json!("test value"),
            }
        ],
        public_key: "any_key".to_string(),
        distance: Some(0),
    };

    let response = write(schema_manager, store, payload);
    assert!(response.results.len() == 1);
    assert_eq!(response.results[0].status, "ok");
}

#[test]
fn test_explicit_field_permissions() {
    let (schema_manager, store) = setup_test_db();

    // Test read with correct key
    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "explicit_field".to_string(),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let response = query(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert!(response.results[0].result.get("error").is_none(),
        "Expected success but got error: {:?}", response.results[0].result);

    // Test read with wrong key
    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "explicit_field".to_string(),
            }
        ],
        public_key: "wrong_key".to_string(),
        distance: Some(0),
    };

    let response = query(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert!(response.results[0].result.get("error").is_some(),
        "Expected error but got: {:?}", response.results[0].result);

    // Test write with correct key
    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteField {
                schema: "test".to_string(),
                field: "explicit_field".to_string(),
                value: json!("test value"),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let response = write(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert_eq!(response.results[0].status, "ok");

    // Test write with wrong key
    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteField {
                schema: "test".to_string(),
                field: "explicit_field".to_string(),
                value: json!("test value"),
            }
        ],
        public_key: "wrong_key".to_string(),
        distance: Some(0),
    };

    let response = write(schema_manager, store, payload);
    assert!(response.results.len() == 1);
    assert_ne!(response.results[0].status, "ok");
}

#[test]
fn test_distance_field_permissions() {
    let (schema_manager, store) = setup_test_db();

    // Test read within distance
    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "distance_field".to_string(),
            }
        ],
        public_key: "any_key".to_string(),
        distance: Some(2),
    };

    let response = query(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert!(response.results[0].result.get("error").is_none(),
        "Expected success but got error: {:?}", response.results[0].result);

    // Test read beyond distance
    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "distance_field".to_string(),
            }
        ],
        public_key: "any_key".to_string(),
        distance: Some(3),
    };

    let response = query(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert!(response.results[0].result.get("error").is_some(),
        "Expected error but got: {:?}", response.results[0].result);

    // Test write within distance
    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteField {
                schema: "test".to_string(),
                field: "distance_field".to_string(),
                value: json!("test value"),
            }
        ],
        public_key: "any_key".to_string(),
        distance: Some(2),
    };

    let response = write(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert_eq!(response.results[0].status, "ok");

    // Test write beyond distance
    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteField {
                schema: "test".to_string(),
                field: "distance_field".to_string(),
                value: json!("test value"),
            }
        ],
        public_key: "any_key".to_string(),
        distance: Some(3),
    };

    let response = write(schema_manager, store, payload);
    assert!(response.results.len() == 1);
    assert_ne!(response.results[0].status, "ok");
}
