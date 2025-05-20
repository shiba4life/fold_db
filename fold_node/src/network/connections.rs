use crate::network::error::{NetworkError, NetworkResult};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Send a request to a node over a TCP connection.
///
/// This utility handles serialization and network I/O for forwarding
/// JSON messages between peers.
pub async fn send_request_to_node(
    mut stream: tokio::net::TcpStream,
    request: Value,
) -> NetworkResult<Value> {
    // Serialize the request
    let request_bytes = serde_json::to_vec(&request).map_err(|e| {
        NetworkError::ProtocolError(format!("Failed to serialize request: {}", e))
    })?;

    // Send the request length
    stream
        .write_u32(request_bytes.len() as u32)
        .await
        .map_err(|e| NetworkError::ConnectionError(format!("Failed to send request length: {}", e)))?;

    // Send the request
    stream
        .write_all(&request_bytes)
        .await
        .map_err(|e| NetworkError::ConnectionError(format!("Failed to send request: {}", e)))?;

    // Read the response length
    let response_len = stream
        .read_u32()
        .await
        .map_err(|e| NetworkError::ConnectionError(format!("Failed to read response length: {}", e)))?
        as usize;

    // Read the response
    let mut response_bytes = vec![0u8; response_len];
    stream
        .read_exact(&mut response_bytes)
        .await
        .map_err(|e| NetworkError::ConnectionError(format!("Failed to read response: {}", e)))?;

    // Deserialize the response
    let response = serde_json::from_slice(&response_bytes).map_err(|e| {
        NetworkError::ProtocolError(format!("Failed to deserialize response: {}", e))
    })?;

    Ok(response)
}
