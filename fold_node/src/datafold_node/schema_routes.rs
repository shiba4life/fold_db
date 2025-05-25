#[cfg(test)]
use super::sample_manager::SampleManager;
use super::http_server::AppState;
use super::http_helpers::with_node;
use crate::schema::Schema;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

/// List all schemas.
pub async fn list_schemas(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to list schemas");
    with_node(state, |node| {
        node.list_schemas()
            .map(|schemas| (StatusCode::OK, json!({"data": schemas})))
            .map_err(|e| {
                error!("Failed to list schemas: {}", e);
                e
            })
    })
    .await
}

/// Get a schema by name.
pub async fn get_schema(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    with_node(state, move |node| {
        match node.get_schema(&name)? {
            Some(schema) => Ok((StatusCode::OK, json!(schema))),
            None => Ok((StatusCode::NOT_FOUND, json!({"error": format!("Schema '{}' not found", name)}))),
        }
    })
    .await
}

/// Create a new schema.
pub async fn create_schema(schema: web::Json<Schema>, state: web::Data<AppState>) -> impl Responder {
    with_node(state, |node| {
        node.load_schema(schema.into_inner())
            .map(|_| (StatusCode::CREATED, json!({"success": true})))
    })
    .await
}

/// Update an existing schema.
pub async fn update_schema(path: web::Path<String>, schema: web::Json<Schema>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let schema_data = schema.into_inner();

    if schema_data.name != name {
        return HttpResponse::BadRequest().json(json!({"error": format!("Schema name '{}' does not match path '{}'", schema_data.name, name)}));
    }

    with_node(state, move |node| {
        let _ = node.unload_schema(&name);
        node.load_schema(schema_data)
            .map(|_| (StatusCode::OK, json!({"success": true})))
    })
    .await
}

/// Unload a schema so it is no longer active.
pub async fn unload_schema_route(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    with_node(state, move |node| {
        node.unload_schema(&name)
            .map(|_| (StatusCode::OK, json!({"success": true})))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::{DataFoldNode, config::NodeConfig};
    use actix_web::web;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_schemas_empty() {
        let dir = tempdir().unwrap();
        let config = NodeConfig::new(dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let state = web::Data::new(super::super::http_server::AppState {
            node: std::sync::Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager: SampleManager { schemas: Default::default(), queries: Default::default(), mutations: Default::default() }
        });
        use actix_web::test;
        let req = test::TestRequest::default().to_http_request();
        let resp = list_schemas(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}

