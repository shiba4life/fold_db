use fold_db::testing::{Schema, SchemaField, PermissionsPolicy};
use fold_db::{DataFoldNode, NodeConfig};
use fold_db::datafold_node::web_server::{WebServer, ApiSuccessResponse};
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use warp::{
    test::request,
    Filter,
    filters::BoxedFilter,
    Reply,
    Rejection,
};
use crate::test_data::{schema_test_data, test_helpers::operation_builder};

async fn create_test_server() -> Arc<tokio::sync::Mutex<DataFoldNode>> {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
    };
    Arc::new(tokio::sync::Mutex::new(DataFoldNode::new(config).unwrap()))
}

#[tokio::test]
async fn test_schema_field_mapping() {
    let node = create_test_server().await;
    
    // Create API endpoints
    let schema_api = warp::path!("api" / "schema")
        .and(warp::post())
        .and(warp::body::json())
        .and(fold_db::datafold_node::web_server::with_node(Arc::clone(&node)))
        .and_then(fold_db::datafold_node::web_server::handle_schema);

    let execute_api = warp::path!("api" / "execute")
        .and(warp::post())
        .and(warp::body::json())
        .and(fold_db::datafold_node::web_server::with_node(Arc::clone(&node)))
        .and_then(fold_db::datafold_node::web_server::handle_execute);

    // 1. Create and load the first user profile schema
    let user_profile = schema_test_data::create_user_profile_schema();
    let schema_response = request()
        .method("POST")
        .path("/api/schema")
        .json(&user_profile)
        .reply(&schema_api)
        .await;
    assert_eq!(schema_response.status(), 200);

    // 2. Create mutation to update username
    let mutation = json!({
        "operation": json!({
            "type": "mutation",
            "schema": "user_profile",
            "operation": "create",
            "data": {
                "username": "johndoe"
            }
        }).to_string()
    });

    let mutation_response = request()
        .method("POST")
        .path("/api/execute")
        .json(&mutation)
        .reply(&execute_api)
        .await;
    assert_eq!(mutation_response.status(), 200);

    // 3. Create and load the second schema with field mapping
    let mut user_profile2 = Schema::new("user_profile2".to_string());
    let mut field_mappings = std::collections::HashMap::new();
    field_mappings.insert("user_profile".to_string(), "username".to_string());
    
    let username_field = SchemaField::new(
        PermissionsPolicy::default(),
        schema_test_data::create_default_payment_config(),
        field_mappings,
    );
    user_profile2.add_field("username".to_string(), username_field);
    
    let schema2_response = request()
        .method("POST")
        .path("/api/schema")
        .json(&user_profile2)
        .reply(&schema_api)
        .await;
    assert_eq!(schema2_response.status(), 200);

    // 4. Query the mapped field
    let query = json!({
        "operation": json!({
            "type": "query",
            "schema": "user_profile2",
            "fields": ["username"],
            "filter": null
        }).to_string()
    });

    let query_response = request()
        .method("POST")
        .path("/api/execute")
        .json(&query)
        .reply(&execute_api)
        .await;
    assert_eq!(query_response.status(), 200);

    // 5. Verify the result
    let response_data: ApiSuccessResponse<serde_json::Value> = serde_json::from_slice(query_response.body()).unwrap();
    let data = response_data.data.as_object().unwrap();
    println!("data: {:?}", data);
    assert!(data.contains_key("username"));
    assert_eq!(data["username"].as_str().unwrap(), "johndoe");
}
