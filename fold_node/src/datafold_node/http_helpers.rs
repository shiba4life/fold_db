use actix_web::{http::StatusCode, web, HttpResponse};
use serde_json::json;

use super::http_server::AppState;
use crate::{datafold_node::DataFoldNode, error::FoldDbResult};

/// Execute a closure with a locked node and return standardized JSON.
pub async fn with_node<F>(state: web::Data<AppState>, func: F) -> HttpResponse
where
    F: FnOnce(&mut DataFoldNode) -> FoldDbResult<(StatusCode, serde_json::Value)>,
{
    let mut node = state.node.lock().await;
    match func(&mut node) {
        Ok((status, value)) => HttpResponse::build(status).json(value),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
