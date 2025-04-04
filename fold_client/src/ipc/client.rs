//! IPC client for applications
//!
//! This module provides the client-side IPC implementation for applications
//! to communicate with the FoldClient.

use crate::ipc::{AppRequest, AppResponse, get_app_socket_path};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Error type for IPC client operations
#[derive(Debug, thiserror::Error)]
pub enum IpcClientError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Request error: {0}")]
    Request(String),
}

/// Result type for IPC client operations
pub type Result<T> = std::result::Result<T, IpcClientError>;

/// IPC client for applications
pub struct IpcClient {
    /// App identifier
    app_id: String,
    /// App token
    token: String,
    /// Connection to the FoldClient
    stream: UnixStream,
}

impl IpcClient {
    /// Connect to the FoldClient
    pub async fn connect(app_socket_dir: &std::path::Path, app_id: &str, token: &str) -> Result<Self> {
        // Get the socket path for the app
        let socket_path = get_app_socket_path(app_socket_dir, app_id);

        // Connect to the socket
        let stream = UnixStream::connect(&socket_path)
            .await
            .map_err(|e| IpcClientError::Connection(format!("Failed to connect to socket: {}", e)))?;

        Ok(Self {
            app_id: app_id.to_string(),
            token: token.to_string(),
            stream,
        })
    }

    /// Send a request to the FoldClient
    pub async fn send_request(&mut self, operation: &str, params: Value) -> Result<Value> {
        // Create the request
        let request = AppRequest::new(&self.app_id, &self.token, operation, params);

        // Serialize the request
        let request_bytes = serde_json::to_vec(&request)
            .map_err(IpcClientError::Serialization)?;

        // Send the request length
        self.stream.write_u32(request_bytes.len() as u32).await?;

        // Send the request
        self.stream.write_all(&request_bytes).await?;

        // Read the response length
        let response_len = self.stream.read_u32().await? as usize;

        // Read the response
        let mut response_bytes = vec![0u8; response_len];
        self.stream.read_exact(&mut response_bytes).await?;

        // Deserialize the response
        let response: AppResponse = serde_json::from_slice(&response_bytes)
            .map_err(IpcClientError::Serialization)?;

        // Check if the request was successful
        if response.success {
            response.result.ok_or_else(|| {
                IpcClientError::Request("Response marked as success but no result provided".to_string())
            })
        } else {
            Err(IpcClientError::Request(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// List available schemas
    pub async fn list_schemas(&mut self) -> Result<Vec<String>> {
        let result = self.send_request("list_schemas", Value::Null).await?;
        let schemas = result
            .as_array()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(schemas)
    }

    /// Query data from a schema
    pub async fn query(
        &mut self,
        schema: &str,
        fields: &[&str],
        filter: Option<Value>,
    ) -> Result<Vec<Value>> {
        let params = serde_json::json!({
            "schema": schema,
            "fields": fields,
            "filter": filter,
        });

        let result = self.send_request("query", params).await?;
        let results = result
            .get("results")
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .as_array()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .clone();
        Ok(results)
    }

    /// Create data in a schema
    pub async fn create(&mut self, schema: &str, data: Value) -> Result<String> {
        let params = serde_json::json!({
            "schema": schema,
            "mutation_type": "create",
            "data": data,
        });

        let result = self.send_request("mutation", params).await?;
        let id = result
            .get("id")
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .as_str()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .to_string();
        Ok(id)
    }

    /// Update data in a schema
    pub async fn update(&mut self, schema: &str, data: Value) -> Result<bool> {
        let params = serde_json::json!({
            "schema": schema,
            "mutation_type": "update",
            "data": data,
        });

        let result = self.send_request("mutation", params).await?;
        let success = result
            .get("success")
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .as_bool()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?;
        Ok(success)
    }

    /// Delete data from a schema
    pub async fn delete(&mut self, schema: &str, id: &str) -> Result<bool> {
        let params = serde_json::json!({
            "schema": schema,
            "mutation_type": "delete",
            "data": {
                "id": id,
            },
        });

        let result = self.send_request("mutation", params).await?;
        let success = result
            .get("success")
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .as_bool()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?;
        Ok(success)
    }

    /// Discover remote nodes
    pub async fn discover_nodes(&mut self) -> Result<Vec<Value>> {
        let result = self.send_request("discover_nodes", Value::Null).await?;
        let nodes = result
            .as_array()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .clone();
        Ok(nodes)
    }

    /// Query data from a remote node
    pub async fn query_remote(
        &mut self,
        node_id: &str,
        schema: &str,
        fields: &[&str],
        filter: Option<Value>,
    ) -> Result<Vec<Value>> {
        let params = serde_json::json!({
            "node_id": node_id,
            "schema": schema,
            "fields": fields,
            "filter": filter,
        });

        let result = self.send_request("query_remote", params).await?;
        let results = result
            .get("results")
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .as_array()
            .ok_or_else(|| IpcClientError::Request("Invalid response format".to_string()))?
            .clone();
        Ok(results)
    }
}
