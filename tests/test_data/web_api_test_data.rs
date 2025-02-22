use fold_db::{DataFoldNode, NodeConfig};
use std::sync::Arc;
use tempfile::tempdir;
use warp::{
    test::request,
    Filter,
    filters::BoxedFilter,
    Reply,
    Rejection,
    http::Response,
    hyper::body::Bytes,
};
use fold_db::datafold_node::web_server::{WebServer, ApiSuccessResponse, ApiErrorResponse, handle_schema, with_node};
use serde_json::json;

pub async fn create_test_server() -> Arc<tokio::sync::Mutex<DataFoldNode>> {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
    };
    Arc::new(tokio::sync::Mutex::new(DataFoldNode::new(config).unwrap()))
}

pub fn create_test_mutation(schema_name: &str, data: serde_json::Value) -> serde_json::Value {
    json!({
        "operation": json!({
            "type": "mutation",
            "schema": schema_name,
            "operation": "create",
            "data": data
        }).to_string()
    })
}

pub fn create_test_query(schema_name: &str, fields: Vec<&str>) -> serde_json::Value {
    json!({
        "operation": json!({
            "type": "query",
            "schema": schema_name,
            "fields": fields,
            "filter": null
        }).to_string()
    })
}

pub async fn load_schema_request<T: Reply + Send>(
    api: impl Filter<Extract = (T,)> + Clone + Send + Sync + 'static,
    schema: &fold_db::testing::Schema
) -> Response<Bytes> {
    request()
        .method("POST")
        .path("/api/schema")
        .json(schema)
        .reply(&api)
        .await
}

pub async fn execute_request<T: Reply + Send>(
    api: impl Filter<Extract = (T,)> + Clone + Send + Sync + 'static,
    operation: serde_json::Value
) -> Response<Bytes> {
    request()
        .method("POST")
        .path("/api/execute")
        .json(&operation)
        .reply(&api)
        .await
}

pub async fn delete_schema_request<T: Reply + Send>(
    api: impl Filter<Extract = (T,)> + Clone + Send + Sync + 'static,
    schema_name: &str
) -> Response<Bytes> {
    request()
        .method("DELETE")
        .path(&format!("/api/schema/{}", schema_name))
        .reply(&api)
        .await
}
