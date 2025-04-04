//! Node client for FoldClient
//!
//! This module provides the client-side implementation for communicating with the DataFold node.

use crate::auth::AuthManager;
use crate::node::{NodeConnection, NodeRequest};
use crate::Result;
use crate::FoldClientError;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};

/// Client for communicating with the DataFold node
pub struct NodeClient {
    /// Connection to the DataFold node
    connection: NodeConnection,
    /// Authentication manager
    auth_manager: Arc<AuthManager>,
}

impl NodeClient {
    /// Create a new node client
    pub fn new(connection: NodeConnection, auth_manager: Arc<AuthManager>) -> Self {
        Self {
            connection,
            auth_manager,
        }
    }

    /// Send a request to the DataFold node
    pub async fn send_request(
        &self,
        app_id: &str,
        operation: &str,
        params: Value,
    ) -> Result<Value> {
        // Create a canonical representation of the request for signing
        let request_json = serde_json::json!({
            "app_id": app_id,
            "operation": operation,
            "params": params,
        });

        // Sign the request
        let message = serde_json::to_string(&request_json)
            .map_err(|e| FoldClientError::Serialization(format!("Failed to serialize request: {}", e)))?;
        let signature = self.auth_manager.sign_message(message.as_bytes())?;

        // Create the request
        let request = NodeRequest {
            app_id: app_id.to_string(),
            operation: operation.to_string(),
            params,
            signature,
        };

        // Send the request to the node
        match &self.connection {
            NodeConnection::UnixSocket(path) => {
                self.send_request_unix(request, path).await
            }
            NodeConnection::TcpSocket(host, port) => {
                self.send_request_tcp(request, host, *port).await
            }
        }
    }

    /// Send a request to the DataFold node over a Unix socket
    async fn send_request_unix(
        &self,
        request: NodeRequest,
        path: &std::path::Path,
    ) -> Result<Value> {
        // Connect to the Unix socket
        let mut stream = UnixStream::connect(path)
            .await
            .map_err(|e| FoldClientError::Node(format!("Failed to connect to Unix socket: {}", e)))?;

        // Serialize the request
        let request_json = serde_json::json!({
            "app_id": request.app_id,
            "operation": request.operation,
            "params": request.params,
            "signature": base64::encode(&request.signature),
        });
        let request_bytes = serde_json::to_vec(&request_json)
            .map_err(|e| FoldClientError::Serialization(format!("Failed to serialize request: {}", e)))?;

        // Send the request length
        stream.write_u32(request_bytes.len() as u32).await
            .map_err(|e| FoldClientError::Node(format!("Failed to send request length: {}", e)))?;

        // Send the request
        stream.write_all(&request_bytes).await
            .map_err(|e| FoldClientError::Node(format!("Failed to send request: {}", e)))?;

        // Read the response length
        let response_len = stream.read_u32().await
            .map_err(|e| FoldClientError::Node(format!("Failed to read response length: {}", e)))? as usize;

        // Read the response
        let mut response_bytes = vec![0u8; response_len];
        stream.read_exact(&mut response_bytes).await
            .map_err(|e| FoldClientError::Node(format!("Failed to read response: {}", e)))?;

        // Deserialize the response
        let response: Value = serde_json::from_slice(&response_bytes)
            .map_err(|e| FoldClientError::Serialization(format!("Failed to deserialize response: {}", e)))?;

        // Check if the response contains an error
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            return Err(FoldClientError::Node(error.to_string()));
        }

        // Return the result
        Ok(response)
    }

    /// Send a request to the DataFold node over a TCP socket
    async fn send_request_tcp(
        &self,
        request: NodeRequest,
        host: &str,
        port: u16,
    ) -> Result<Value> {
        // Connect to the TCP socket
        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| FoldClientError::Node(format!("Failed to connect to TCP socket at {}: {}", addr, e)))?;

        // Serialize the request
        let request_json = serde_json::json!({
            "app_id": request.app_id,
            "operation": request.operation,
            "params": request.params,
            "signature": base64::encode(&request.signature),
        });
        let request_bytes = serde_json::to_vec(&request_json)
            .map_err(|e| FoldClientError::Serialization(format!("Failed to serialize request: {}", e)))?;

        // Send the request length
        stream.write_u32(request_bytes.len() as u32).await
            .map_err(|e| FoldClientError::Node(format!("Failed to send request length: {}", e)))?;

        // Send the request
        stream.write_all(&request_bytes).await
            .map_err(|e| FoldClientError::Node(format!("Failed to send request: {}", e)))?;

        // Read the response length
        let response_len = stream.read_u32().await
            .map_err(|e| FoldClientError::Node(format!("Failed to read response length: {}", e)))? as usize;

        // Read the response
        let mut response_bytes = vec![0u8; response_len];
        stream.read_exact(&mut response_bytes).await
            .map_err(|e| FoldClientError::Node(format!("Failed to read response: {}", e)))?;

        // Deserialize the response
        let response: Value = serde_json::from_slice(&response_bytes)
            .map_err(|e| FoldClientError::Serialization(format!("Failed to deserialize response: {}", e)))?;

        // Check if the response contains an error
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            return Err(FoldClientError::Node(error.to_string()));
        }

        // Return the result
        Ok(response)
    }
}
