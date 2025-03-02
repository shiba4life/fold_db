use std::sync::Arc;
use warp::{Rejection, Reply};
use serde::{Deserialize, Serialize};
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::app::AppManifest;

/// Handler for listing apps
pub async fn handle_list_apps(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.list_apps() {
        Ok(apps) => {
            let response = ListAppsResponse { apps };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to list apps: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Handler for registering an app
pub async fn handle_register_app(
    manifest: AppManifest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.register_app(manifest) {
        Ok(_) => {
            let response = SuccessResponse {
                message: "App registered successfully".to_string(),
            };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to register app: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Handler for starting an app
pub async fn handle_start_app(
    app_name: String,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.start_app(&app_name) {
        Ok(_) => {
            let response = SuccessResponse {
                message: format!("App '{}' started successfully", app_name),
            };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to start app: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Handler for stopping an app
pub async fn handle_stop_app(
    app_name: String,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.stop_app(&app_name) {
        Ok(_) => {
            let response = SuccessResponse {
                message: format!("App '{}' stopped successfully", app_name),
            };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to stop app: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Handler for unloading an app
pub async fn handle_unload_app(
    app_name: String,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.unload_app(&app_name) {
        Ok(_) => {
            let response = SuccessResponse {
                message: format!("App '{}' unloaded successfully", app_name),
            };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to unload app: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Handler for listing APIs
pub async fn handle_list_apis(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.list_apis() {
        Ok(apis) => {
            let response = ListApisResponse { apis };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to list APIs: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Handler for registering an API
pub async fn handle_register_api(
    api: RegisterApiRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    match node.register_api(&api.name, &api.version, &api.description) {
        Ok(_) => {
            let response = SuccessResponse {
                message: format!("API '{}' registered successfully", api.name),
            };
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to register API: {}", e),
            };
            Ok(warp::reply::json(&error))
        }
    }
}

/// Request for registering an API
#[derive(Debug, Deserialize)]
pub struct RegisterApiRequest {
    /// API name
    pub name: String,
    
    /// API version
    pub version: String,
    
    /// API description
    pub description: String,
}

/// Response for listing apps
#[derive(Debug, Serialize)]
pub struct ListAppsResponse {
    /// List of app names
    pub apps: Vec<String>,
}

/// Response for listing APIs
#[derive(Debug, Serialize)]
pub struct ListApisResponse {
    /// List of APIs
    pub apis: Vec<crate::datafold_node::app::api::ApiInfo>,
}

/// Success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    /// Success message
    pub message: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
}
