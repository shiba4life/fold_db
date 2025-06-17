use crate::logging::LoggingSystem;
use actix_web::{web, HttpResponse, Responder, Result};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Serialize, Deserialize)]
pub struct LogLevelUpdate {
    pub feature: String,
    pub level: String,
}

#[derive(Serialize, Deserialize)]
pub struct LogConfigResponse {
    pub message: String,
    pub current_level: String,
}

/// List current logs (backward compatibility)
pub async fn list_logs() -> impl Responder {
    // Use the logging system's get_logs function
    let logs = crate::logging::get_logs();
    HttpResponse::Ok().json(logs)
}

/// Stream logs via Server-Sent Events (backward compatibility)
pub async fn stream_logs() -> impl Responder {
    // Try to get log stream from the logging system
    match crate::logging::subscribe() {
        Some(rx) => {
            let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
                match msg {
                    Ok(line) => Some(Ok::<web::Bytes, actix_web::Error>(web::Bytes::from(
                        format!("data: {}\n\n", line),
                    ))),
                    Err(_) => None,
                }
            });
            HttpResponse::Ok()
                .insert_header(("Content-Type", "text/event-stream"))
                .streaming(stream)
        }
        None => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Log streaming not available"})),
    }
}

/// Get current logging configuration
pub async fn get_config() -> Result<impl Responder> {
    if let Some(config) = LoggingSystem::get_config().await {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "config": config
        })))
    } else {
        let current_level = log::max_level().to_string();
        Ok(HttpResponse::Ok().json(LogConfigResponse {
            message: "Basic logging configuration".to_string(),
            current_level,
        }))
    }
}

/// Update feature-specific log level at runtime
pub async fn update_feature_level(
    level_update: web::Json<LogLevelUpdate>,
) -> Result<impl Responder> {
    let valid_levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
    if !valid_levels.contains(&level_update.level.as_str()) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Invalid log level: {}", level_update.level)
        })));
    }

    match LoggingSystem::update_feature_level(&level_update.feature, &level_update.level).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Updated {} log level to {}", level_update.feature, level_update.level)
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update log level: {}", e)
            })))
        }
    }
}

/// Reload logging configuration from file
pub async fn reload_config() -> Result<impl Responder> {
    match LoggingSystem::reload_config_from_file("config/logging.toml").await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Configuration reloaded successfully"
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Failed to reload configuration: {}", e)
        }))),
    }
}

/// Get available log features and their current levels
pub async fn get_features() -> Result<impl Responder> {
    if let Some(features) = LoggingSystem::get_features().await {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "features": features,
            "available_levels": ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]
        })))
    } else {
        let current_level = log::max_level().to_string();
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "features": {
                "transform": current_level,
                "network": current_level,
                "database": current_level,
                "schema": current_level,
                "query": current_level,
                "mutation": current_level,
                "permissions": current_level,
                "http_server": current_level,
                "tcp_server": current_level,
                "ingestion": current_level
            },
            "available_levels": ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]
        })))
    }
}
