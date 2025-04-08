//! IPC module for FoldClient
//!
//! This module provides IPC (Inter-Process Communication) mechanisms for
//! communication between the FoldClient and sandboxed applications.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

pub mod client;
pub mod server;

/// Request from an app to the FoldClient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRequest {
    /// Unique identifier for the request
    pub request_id: String,
    /// App identifier
    pub app_id: String,
    /// App token
    pub token: String,
    /// Operation to perform
    pub operation: String,
    /// Operation parameters
    pub params: Value,
    /// Signature of the request
    pub signature: Option<String>,
}

impl AppRequest {
    /// Create a new app request
    pub fn new(app_id: &str, token: &str, operation: &str, params: Value) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            app_id: app_id.to_string(),
            token: token.to_string(),
            operation: operation.to_string(),
            params,
            signature: None,
        }
    }

    /// Sign the request
    pub fn sign(&mut self, signature: String) {
        self.signature = Some(signature);
    }
}

/// Response from the FoldClient to an app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppResponse {
    /// Unique identifier for the request
    pub request_id: String,
    /// Whether the request was successful
    pub success: bool,
    /// Result of the operation
    pub result: Option<Value>,
    /// Error message if the request failed
    pub error: Option<String>,
}

impl AppResponse {
    /// Create a new successful response
    pub fn success(request_id: &str, result: Value) -> Self {
        Self {
            request_id: request_id.to_string(),
            success: true,
            result: Some(result),
            error: None,
        }
    }

    /// Create a new error response
    pub fn error(request_id: &str, error: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            success: false,
            result: None,
            error: Some(error.to_string()),
        }
    }
}

/// Request from the FoldClient to the DataFold node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRequest {
    /// App identifier
    pub app_id: String,
    /// Operation to perform
    pub operation: String,
    /// Operation parameters
    pub params: Value,
    /// Signature of the request
    pub signature: String,
}

/// Response from the DataFold node to the FoldClient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Result of the operation
    pub result: Option<Value>,
    /// Error message if the request failed
    pub error: Option<String>,
}

/// Get the path to the IPC socket for an app
pub fn get_app_socket_path(app_socket_dir: &std::path::Path, app_id: &str) -> PathBuf {
    app_socket_dir.join(format!("{}.sock", app_id))
}

/// Get the path to the Docker volume mount for the IPC socket
pub fn get_docker_socket_mount(app_id: &str) -> String {
    format!("/tmp/fold_client_sockets/{}.sock", app_id)
}
