use std::sync::Arc;
use serde_json::json;

use fold_db::schema::SchemaManager;
use fold_db::store::Store;
use fold_db::api::{
    QueryPayload, QueryItem,
    WritePayload, WriteItem,
    query, write,
};

fn setup_test_db() -> (Arc<SchemaManager>, Arc<Store>) {
    let schema_manager = Arc::new(SchemaManager::new());
    let store = Arc::new(Store::new(&format!(":memory:{:?}", std::thread::current().id())).unwrap());

    // Create test schema
    let mut schema = fold_db::schema::InternalSchema::new();
    schema.fields.insert("test_field".to_string(), "test-uuid".to_string());
    schema.fields.insert("id".to_string(), "id-uuid".to_string());
    schema.fields.insert("value".to_string(), "value-uuid".to_string());
    
    schema_manager.load_schema("test", schema).unwrap();

    (schema_manager, store)
}

#[test]
fn test_query_field() {
    let (schema_manager, store) = setup_test_db();

    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "test_field".to_string(),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let response = query(schema_manager, store, payload);
    assert!(response.results.len() == 1);
    // Verify no error in response
    assert!(response.results[0].result.is_null(), 
        "Expected null for unset field but got: {:?}", response.results[0].result);
}

#[test]
fn test_query_nonexistent_schema() {
    let (schema_manager, store) = setup_test_db();

    let payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "nonexistent".to_string(),
                field: "test_field".to_string(),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let response = query(schema_manager, store, payload);
    assert!(response.results.len() == 1);
    let error = response.results[0].result.as_object().unwrap()
        .get("error").unwrap().as_str().unwrap();
    assert_eq!(error, "schema not loaded");
}

#[test]
fn test_write_field() {
    let (schema_manager, store) = setup_test_db();

    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteField {
                schema: "test".to_string(),
                field: "test_field".to_string(),
                value: json!("new value"),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let response = write(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert_eq!(response.results[0].status, "ok");

    // Verify the write
    let query_payload = QueryPayload {
        queries: vec![
            QueryItem::Field {
                schema: "test".to_string(),
                field: "test_field".to_string(),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let query_response = query(schema_manager, store, query_payload);
    let result = &query_response.results[0].result;
    assert_eq!(result.get("value").unwrap(), &json!("new value"), 
        "Expected query to return the written value");
}

#[test]
fn test_write_collection() {
    let (schema_manager, store) = setup_test_db();

    let item = json!({"id": 1, "value": "test"});
    let payload = WritePayload {
        writes: vec![
            WriteItem::WriteCollection {
                schema: "test".to_string(),
                collection: "test_collection".to_string(),
                item: item.clone(),
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let response = write(schema_manager.clone(), store.clone(), payload);
    assert!(response.results.len() == 1);
    assert_eq!(response.results[0].status, "ok");

    // Verify the write
    let query_payload = QueryPayload {
        queries: vec![
            QueryItem::Collection {
                schema: "test".to_string(),
                collection: "test_collection".to_string(),
                sort: None,
                sort_field: None,
                limit: None,
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let query_response = query(schema_manager, store, query_payload);
    let items = query_response.results[0].result.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(&items[0], &item);
}

#[test]
fn test_sorted_collection() {
    let (schema_manager, store) = setup_test_db();

    // Write multiple items
    let items = vec![
        json!({"id": 1, "name": "Charlie"}),
        json!({"id": 2, "name": "Alice"}),
        json!({"id": 3, "name": "Bob"}),
    ];

    for item in &items {
            let payload = WritePayload {
                writes: vec![
                    WriteItem::WriteCollection {
                        schema: "test".to_string(),
                        collection: "test_collection".to_string(),
                        item: item.clone(),
                    }
                ],
                public_key: "test_key".to_string(),
                distance: Some(0),
            };
        let response = write(schema_manager.clone(), store.clone(), payload);
        assert_eq!(response.results[0].status, "ok");
    }

    // Test ascending sort
    let sort_field = "name".to_string();
    let sort_order = "asc".to_string();
    let query_payload = QueryPayload {
        queries: vec![
            QueryItem::Collection {
                schema: "test".to_string(),
                collection: "test_collection".to_string(),
                sort: Some(sort_order),
                sort_field: Some(sort_field.clone()),
                limit: None,
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let query_response = query(schema_manager.clone(), store.clone(), query_payload);
    let items = query_response.results[0].result.as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].get("name").unwrap(), "Alice");
    assert_eq!(items[1].get("name").unwrap(), "Bob");
    assert_eq!(items[2].get("name").unwrap(), "Charlie");

    // Test descending sort
    let query_payload = QueryPayload {
        queries: vec![
            QueryItem::Collection {
                schema: "test".to_string(),
                collection: "test_collection".to_string(),
                sort: Some("desc".to_string()),
                sort_field: Some(sort_field),
                limit: None,
            }
        ],
        public_key: "test_key".to_string(),
        distance: Some(0),
    };

    let query_response = query(schema_manager, store, query_payload);
    let items = query_response.results[0].result.as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].get("name").unwrap(), "Charlie");
    assert_eq!(items[1].get("name").unwrap(), "Bob");
    assert_eq!(items[2].get("name").unwrap(), "Alice");
}
