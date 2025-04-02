use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{AppSdkError, AppSdkResult};
use crate::types::{NodeConnection, AppRequest};

#[cfg(any(test, feature = "mock"))]
use crate::network_mock::NetworkMock;

/// Utility functions for network operations
pub struct NetworkUtils;

impl NetworkUtils {
    /// Send a request to the node
    pub async fn send_request(connection: &NodeConnection, request: AppRequest) -> AppSdkResult<Value> {
        match connection {
            NodeConnection::UnixSocket(path) => {
                // Special handling for mock connections in tests
                #[cfg(any(test, feature = "mock"))]
                if NetworkMock::is_mock_path(path) {
                    return NetworkMock::handle_mock_request(&request).await;
                }
                
                // Normal production code path
                let stream = tokio::net::UnixStream::connect(path)
                    .await
                    .map_err(|e| AppSdkError::Connection(format!("Failed to connect to Unix socket: {}", e)))?;
                
                Self::send_request_stream(stream, request).await
            }
            NodeConnection::SharedMemory(_) => {
                Err(AppSdkError::Connection("Shared memory connection not yet implemented".to_string()))
            }
            NodeConnection::NamedPipe(_path) => {
                #[cfg(target_family = "windows")]
                {
                    use tokio::net::windows::named_pipe::ClientOptions;
                    let client = ClientOptions::new()
                        .open(_path)
                        .map_err(|e| AppSdkError::Connection(format!("Failed to connect to named pipe: {}", e)))?;
                    
                    Self::send_request_stream(client, request).await
                }
                #[cfg(not(target_family = "windows"))]
                {
                    Err(AppSdkError::Connection("Named pipes are only supported on Windows".to_string()))
                }
            }
        }
    }


    /// Helper to send request over any async stream
    pub async fn send_request_stream<T>(mut stream: T, request: AppRequest) -> AppSdkResult<Value> 
    where
        T: AsyncReadExt + AsyncWriteExt + Unpin
    {
        // Serialize the request
        let request_bytes = serde_json::to_vec(&request)
            .map_err(|e| AppSdkError::Serialization(format!("Failed to serialize request: {}", e)))?;
        
        // Send the request length as u32 first
        stream.write_u32(request_bytes.len() as u32).await
            .map_err(|e| AppSdkError::Connection(format!("Failed to send request length: {}", e)))?;
        
        // Send the request
        stream.write_all(&request_bytes).await
            .map_err(|e| AppSdkError::Connection(format!("Failed to send request: {}", e)))?;
        
        // Read the response length
        let response_len = stream.read_u32().await
            .map_err(|e| AppSdkError::Connection(format!("Failed to read response length: {}", e)))? as usize;
        
        // Read the response
        let mut response_bytes = vec![0u8; response_len];
        stream.read_exact(&mut response_bytes).await
            .map_err(|e| AppSdkError::Connection(format!("Failed to read response: {}", e)))?;
        
        // Deserialize the response
        let result = serde_json::from_slice(&response_bytes)
            .map_err(|e| AppSdkError::Serialization(format!("Failed to deserialize response: {}", e)))?;
        
        Ok(result)
    }
}
