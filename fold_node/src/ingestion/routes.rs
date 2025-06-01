//! HTTP route handlers for the ingestion API

use crate::ingestion::{IngestionConfig, IngestionResponse};
use crate::ingestion::core::IngestionRequest;
use crate::ingestion::simple_service::SimpleIngestionService;
use crate::datafold_node::http_server::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use serde::{Deserialize, Serialize};
use log::{info, error, warn};
use std::fs;
use std::path::Path;

/// Process JSON ingestion request
pub async fn process_json(
    request: web::Json<IngestionRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!("Received JSON ingestion request");

    // Try to create a simple ingestion service
    let service = match create_simple_ingestion_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to initialize ingestion service: {}", e);
            return HttpResponse::ServiceUnavailable().json(IngestionResponse::failure(vec![
                format!("Ingestion service not available: {}", e)
            ]));
        }
    };

    // Get a mutable reference to the node
    let mut node = state.node.lock().await;

    // Process the ingestion request
    match service.process_json_with_node(request.into_inner(), &mut *node).await {
        Ok(response) => {
            if response.success {
                info!("Ingestion completed successfully");
                HttpResponse::Ok().json(response)
            } else {
                error!("Ingestion failed: {:?}", response.errors);
                HttpResponse::InternalServerError().json(response)
            }
        }
        Err(e) => {
            error!("Ingestion processing failed: {}", e);
            HttpResponse::InternalServerError().json(IngestionResponse::failure(vec![
                format!("Processing failed: {}", e)
            ]))
        }
    }
}

/// Get ingestion status
pub async fn get_status(_state: web::Data<AppState>) -> impl Responder {
    info!("Received ingestion status request");

    match create_simple_ingestion_service().await {
        Ok(service) => {
            match service.get_status() {
                Ok(status) => {
                    info!("Returning ingestion status");
                    HttpResponse::Ok().json(status)
                }
                Err(e) => {
                    error!("Failed to get ingestion status: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get status: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            warn!("Ingestion service not available: {}", e);
            HttpResponse::ServiceUnavailable().json(json!({
                "error": format!("Ingestion service not available: {}", e),
                "enabled": false,
                "configured": false
            }))
        }
    }
}

/// Health check endpoint for ingestion service
pub async fn health_check(_state: web::Data<AppState>) -> impl Responder {
    match create_simple_ingestion_service().await {
        Ok(service) => {
            let status = service.get_status().unwrap_or_else(|_| json!({
                "enabled": false,
                "configured": false
            }));

            let is_healthy = status.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false) &&
                             status.get("configured").and_then(|v| v.as_bool()).unwrap_or(false);

            if is_healthy {
                HttpResponse::Ok().json(json!({
                    "status": "healthy",
                    "service": "ingestion",
                    "details": status
                }))
            } else {
                HttpResponse::ServiceUnavailable().json(json!({
                    "status": "unhealthy",
                    "service": "ingestion",
                    "details": status
                }))
            }
        }
        Err(e) => {
            HttpResponse::ServiceUnavailable().json(json!({
                "status": "unavailable",
                "service": "ingestion",
                "error": e.to_string()
            }))
        }
    }
}

/// Get ingestion configuration (without sensitive data)
pub async fn get_config(_state: web::Data<AppState>) -> impl Responder {
    info!("Received ingestion config request");

    // Use the allow_empty version to get current config status
    let config = IngestionConfig::from_env_allow_empty();
    let config_info = json!({
        "enabled": config.enabled,
        "model": config.openrouter_model,
        "auto_execute_mutations": config.auto_execute_mutations,
        "default_trust_distance": config.default_trust_distance,
        "api_key_configured": !config.openrouter_api_key.is_empty(),
        "configured": config.is_ready()
    });

    HttpResponse::Ok().json(config_info)
}

/// Validate JSON data without processing
pub async fn validate_json(
    request: web::Json<serde_json::Value>,
    _state: web::Data<AppState>,
) -> impl Responder {
    info!("Received JSON validation request");

    match create_simple_ingestion_service().await {
        Ok(service) => {
            match service.validate_input(&request.into_inner()) {
                Ok(()) => {
                    info!("JSON validation successful");
                    HttpResponse::Ok().json(json!({
                        "valid": true,
                        "message": "JSON data is valid for ingestion"
                    }))
                }
                Err(e) => {
                    info!("JSON validation failed: {}", e);
                    HttpResponse::BadRequest().json(json!({
                        "valid": false,
                        "error": format!("Validation failed: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            HttpResponse::ServiceUnavailable().json(json!({
                "valid": false,
                "error": format!("Ingestion service not available: {}", e)
            }))
        }
    }
}

/// OpenRouter configuration request/response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterConfigRequest {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterConfigResponse {
    pub api_key: String,
    pub model: String,
}

/// Get OpenRouter configuration
pub async fn get_openrouter_config(_state: web::Data<AppState>) -> impl Responder {
    info!("Received OpenRouter config request");

    // Use the allow_empty version to get current config without requiring API key
    let config = IngestionConfig::from_env_allow_empty();
    
    // Don't return the actual API key for security, just indicate if it's set
    let response = json!({
        "api_key": if config.openrouter_api_key.is_empty() { "" } else { "***configured***" },
        "model": config.openrouter_model
    });
    HttpResponse::Ok().json(response)
}

/// Save OpenRouter configuration
pub async fn save_openrouter_config(
    request: web::Json<OpenRouterConfigRequest>,
    _state: web::Data<AppState>,
) -> impl Responder {
    info!("Received OpenRouter config save request");

    let config = request.into_inner();
    
    match save_openrouter_config_to_file(&config) {
        Ok(()) => {
            info!("OpenRouter configuration saved successfully");
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Configuration saved successfully"
            }))
        }
        Err(e) => {
            error!("Failed to save OpenRouter config: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to save configuration: {}", e)
            }))
        }
    }
}

/// Load OpenRouter configuration from file
fn load_openrouter_config() -> Result<OpenRouterConfigResponse, Box<dyn std::error::Error>> {
    let config_path = get_config_file_path();
    
    if !config_path.exists() {
        return Ok(OpenRouterConfigResponse {
            api_key: String::new(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
        });
    }

    let content = fs::read_to_string(&config_path)?;
    let config: OpenRouterConfigResponse = serde_json::from_str(&content)?;
    Ok(config)
}

/// Save OpenRouter configuration to file
fn save_openrouter_config_to_file(config: &OpenRouterConfigRequest) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_file_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_response = OpenRouterConfigResponse {
        api_key: config.api_key.clone(),
        model: config.model.clone(),
    };

    let content = serde_json::to_string_pretty(&config_response)?;
    fs::write(&config_path, content)?;
    
    info!("OpenRouter config saved to: {:?}", config_path);
    Ok(())
}

/// Get the path to the OpenRouter configuration file
fn get_config_file_path() -> std::path::PathBuf {
    let config_dir = std::env::var("DATAFOLD_CONFIG_DIR")
        .unwrap_or_else(|_| "./config".to_string());
    
    Path::new(&config_dir).join("openrouter_config.json")
}

/// Create a simple ingestion service with potentially updated config
async fn create_simple_ingestion_service() -> Result<SimpleIngestionService, crate::ingestion::IngestionError> {
    // Try to load saved OpenRouter config and merge with environment
    let mut config = IngestionConfig::from_env()?;
    
    if let Ok(saved_config) = load_openrouter_config() {
        // Override with saved config if API key is provided
        if !saved_config.api_key.is_empty() {
            config.openrouter_api_key = saved_config.api_key;
        }
        config.openrouter_model = saved_config.model;
    }
    
    SimpleIngestionService::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::datafold_node::{DataFoldNode, NodeConfig};
    use std::sync::Arc;
    use tempfile::tempdir;

    async fn create_test_app_state() -> web::Data<AppState> {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::load(config).await.unwrap();

        web::Data::new(AppState {
            node: Arc::new(tokio::sync::Mutex::new(node)),
        })
    }

    #[actix_web::test]
    async fn test_get_status() {
        let app_state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/status", web::get().to(get_status))
        ).await;

        let req = test::TestRequest::get().uri("/status").to_request();
        let resp = test::call_service(&app, req).await;
        // Should return service unavailable if not configured
        assert!(resp.status().is_server_error() || resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_health_check() {
        let app_state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/health", web::get().to(health_check))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_server_error() || resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_config() {
        let app_state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/config", web::get().to(get_config))
        ).await;

        let req = test::TestRequest::get().uri("/config").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}