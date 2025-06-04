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
    let request_bytes = serde_json::to_vec(&request)
        .map_err(|e| NetworkError::ProtocolError(format!("Failed to serialize request: {}", e)))?;

    // Send the request length
    stream
        .write_u32(request_bytes.len() as u32)
        .await
        .map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to send request length: {}", e))
        })?;

    // Send the request
    stream
        .write_all(&request_bytes)
        .await
        .map_err(|e| NetworkError::ConnectionError(format!("Failed to send request: {}", e)))?;

    // Read the response length
    let response_len = stream.read_u32().await.map_err(|e| {
        NetworkError::ConnectionError(format!("Failed to read response length: {}", e))
    })? as usize;

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

use crate::network::NetworkCore;
use libp2p::PeerId;
use std::collections::HashSet;

impl NetworkCore {
    /// Register a node ID with a peer ID
    pub fn register_node_id(&mut self, node_id: &str, peer_id: PeerId) {
        self.node_to_peer_map.insert(node_id.to_string(), peer_id);
        self.peer_to_node_map.insert(peer_id, node_id.to_string());
    }

    /// Register the listening address for a node ID
    pub fn register_node_address(&mut self, node_id: &str, address: String) {
        self.node_to_address_map
            .insert(node_id.to_string(), address);
    }

    /// Get the listening address for a node ID
    pub fn get_address_for_node(&self, node_id: &str) -> Option<String> {
        self.node_to_address_map.get(node_id).cloned()
    }

    /// Get the peer ID for a node ID
    pub fn get_peer_id_for_node(&self, node_id: &str) -> Option<PeerId> {
        self.node_to_peer_map.get(node_id).cloned()
    }

    /// Get the node ID for a peer ID
    pub fn get_node_id_for_peer(&self, peer_id: &PeerId) -> Option<String> {
        self.peer_to_node_map.get(peer_id).cloned()
    }

    /// Add a known peer to the network
    pub fn add_known_peer(&mut self, peer_id: PeerId) {
        self.known_peers.insert(peer_id);
    }

    /// Get the set of known peers
    pub fn known_peers(&self) -> &HashSet<PeerId> {
        &self.known_peers
    }
}
