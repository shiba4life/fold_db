use fold_db::testing::{
    Schema, SchemaField, PermissionsPolicy, TrustDistance, 
    FieldPaymentConfig, TrustDistanceScaling
};
use fold_db::{DataFoldNode, NodeConfig};
use std::sync::Arc;
use tempfile::tempdir;
use warp::{
    test::request,
    Filter,
    filters::BoxedFilter,
    Reply,
    Rejection,
};
use fold_db::datafold_node::web_server::{WebServer, ApiSuccessResponse, ApiErrorResponse, handle_schema, with_node};
use std::collections::HashMap;
use uuid;
use std::convert::Infallible;

// Import test helpers
mod test_helpers;

async fn create_test_server() -> Arc<tokio::sync::Mutex<DataFoldNode>> {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        docker: fold_db::datafold_node::DockerConfig::default(),
    };
    Arc::new(tokio::sync::Mutex::new(DataFoldNode::new(config).unwrap()))
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

    schema
}

#[tokio::test]
async fn test_schema_loading_success() {
    let node = create_test_server().await;
    let schema = create_test_schema();
    
    // Create test filter
    let api = warp::path!("api" / "schema")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_node(Arc::clone(&node)))
        .and_then(handle_schema);

    // Make request
    let response = request()
        .method("POST")
        .path("/api/schema")
        .json(&schema)
        .reply(&api)
        .await;

    assert_eq!(response.status(), 200);
    
    // Verify response format
    let response_data: ApiSuccessResponse<Schema> = serde_json::from_slice(response.body()).unwrap();
    assert_eq!(response_data.data.name, schema.name);
    assert!(response_data.data.fields.contains_key("name"));
}

#[tokio::test]
async fn test_schema_loading_invalid_schema() {
    let node = create_test_server().await;
    
    // Create an invalid schema (empty name)
    let mut invalid_schema = create_test_schema();
    invalid_schema.name = "".to_string();

    // Create test filter
    let api = warp::path!("api" / "schema")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_node(Arc::clone(&node)))
        .and_then(handle_schema);

    // Make request
    let response = request()
        .method("POST")
        .path("/api/schema")
        .json(&invalid_schema)
        .reply(&api)
        .await;

    assert_eq!(response.status(), 200); // Note: Using 200 since we wrap errors in ApiResponse
    
    // Verify error response
    println!("Response body: {}", String::from_utf8_lossy(response.body()));
    let response_data: ApiErrorResponse = serde_json::from_slice(response.body()).unwrap();
    assert!(!response_data.error.is_empty());
    assert_eq!(response_data.error, "Schema name cannot be empty");
}

#[tokio::test]
async fn test_schema_loading_duplicate() {
    let node = create_test_server().await;
    let schema = create_test_schema();
    
    // Create test filter
    let api = warp::path!("api" / "schema")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_node(Arc::clone(&node)))
        .and_then(handle_schema);

    // First load should succeed
    let response1 = request()
        .method("POST")
        .path("/api/schema")
        .json(&schema)
        .reply(&api)
        .await;
    assert_eq!(response1.status(), 200);

    // Second load of same schema should fail
    let response2 = request()
        .method("POST")
        .path("/api/schema")
        .json(&schema)
        .reply(&api)
        .await;

    assert_eq!(response2.status(), 200); // Note: Using 200 since we wrap errors in ApiResponse
    
    // Verify error response
    println!("Response body: {}", String::from_utf8_lossy(response2.body()));
    let response_data: ApiErrorResponse = serde_json::from_slice(response2.body()).unwrap();
    assert!(!response_data.error.is_empty());
    assert_eq!(response_data.error, "Schema error: Schema already exists");
}

#[tokio::test]
async fn test_schema_loading_malformed_json() {
    let node = create_test_server().await;
    
    // Create test filter
    let api = warp::path!("api" / "schema")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_node(Arc::clone(&node)))
        .and_then(handle_schema);

    // Make request with malformed JSON
    let response = request()
        .method("POST")
        .path("/api/schema")
        .body(r#"{"invalid: json"#)
        .reply(&api)
        .await;

    assert_eq!(response.status(), 400); // Warp will return 400 for invalid JSON
}

#[tokio::test]
async fn test_schema_deletion() {
    let node = create_test_server().await;
    let schema = create_test_schema();
    
    // First load a schema
    let load_api = warp::path!("api" / "schema")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_node(Arc::clone(&node)))
        .and_then(handle_schema);

    let load_response = request()
        .method("POST")
        .path("/api/schema")
        .json(&schema)
        .reply(&load_api)
        .await;
    assert_eq!(load_response.status(), 200);

    // Create delete endpoint filter
    let delete_api = warp::path!("api" / "schema" / String)
        .and(warp::delete())
        .and(with_node(Arc::clone(&node)))
        .and_then(fold_db::datafold_node::web_server::handle_delete_schema);

    // Test deleting existing schema
    let delete_response = request()
        .method("DELETE")
        .path(&format!("/api/schema/{}", schema.name))
        .reply(&delete_api)
        .await;

    assert_eq!(delete_response.status(), 200);
    let response_data: ApiSuccessResponse<&str> = serde_json::from_slice(delete_response.body()).unwrap();
    assert_eq!(response_data.data, "Schema removed successfully");

    // Test deleting non-existent schema
    let delete_nonexistent = request()
        .method("DELETE")
        .path("/api/schema/nonexistent")
        .reply(&delete_api)
        .await;

    assert_eq!(delete_nonexistent.status(), 200);
    let error_response: ApiErrorResponse = serde_json::from_slice(delete_nonexistent.body()).unwrap();
    assert_eq!(error_response.error, "Schema not found");

    // Verify schema was actually deleted by trying to delete it again
    let delete_again = request()
        .method("DELETE")
        .path(&format!("/api/schema/{}", schema.name))
        .reply(&delete_api)
        .await;

    assert_eq!(delete_again.status(), 200);
    let error_response: ApiErrorResponse = serde_json::from_slice(delete_again.body()).unwrap();
    assert_eq!(error_response.error, "Schema not found");
}
