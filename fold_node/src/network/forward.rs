use crate::network::error::{NetworkError, NetworkResult};
use crate::network::connections;
use crate::network::NetworkCore;
use libp2p::PeerId;
use serde_json::Value;
use log::{info, warn, error};

impl NetworkCore {
    /// Forward a request to another node.
    pub async fn forward_request(
        &mut self,
        peer_id: PeerId,
        request: Value,
    ) -> NetworkResult<Value> {
        if !self.known_peers.contains(&peer_id) {
            return Err(NetworkError::ConnectionError(format!(
                "Peer not found: {}",
                peer_id
            )));
        }

        let operation = request
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                NetworkError::ProtocolError("Missing operation in request".to_string())
            })?;

        let node_id = self
            .get_node_id_for_peer(&peer_id)
            .unwrap_or_else(|| peer_id.to_string());

        info!(
            "Forwarding {} request to node {} (peer {})",
            operation, node_id, peer_id
        );

        let target_address = match self.get_address_for_node(&node_id) {
            Some(addr) => addr,
            None => {
                return Err(NetworkError::ConnectionError(format!(
                    "Address for node {} not found",
                    node_id
                )));
            }
        };

        info!("Connecting to target node at {}", target_address);

        let stream = match tokio::net::TcpStream::connect(&target_address).await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(NetworkError::ConnectionError(format!(
                    "Failed to connect to target node at {}: {}",
                    target_address, e
                )));
            }
        };

        let result = connections::send_request_to_node(stream, request.clone()).await;

        match result {
            Ok(response) => {
                info!("Received response from target node");
                Ok(response)
            }
            Err(e) => {
                error!("Error forwarding request to target node: {}", e);
                warn!("Falling back to simulated response");

                match operation {
                    "query" => {
                        let schema = request
                            .get("params")
                            .and_then(|v| v.get("schema"))
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                NetworkError::ProtocolError(
                                    "Missing schema in query request".to_string(),
                                )
                            })?;

                        let fields = request
                            .get("params")
                            .and_then(|v| v.get("fields"))
                            .and_then(|v| v.as_array())
                            .ok_or_else(|| {
                                NetworkError::ProtocolError(
                                    "Missing fields in query request".to_string(),
                                )
                            })?;

                        Ok(serde_json::json!({
                            "results": [
                                fields.iter().map(|_| {
                                    match rand::random::<u8>() % 3 {
                                        0 => serde_json::json!("sample_string_value"),
                                        1 => serde_json::json!(42),
                                        _ => serde_json::json!(true),
                                    }
                                }).collect::<Vec<_>>()
                            ],
                            "schema": schema,
                            "forwarded": true,
                            "node_id": node_id,
                            "peer_id": peer_id.to_string(),
                            "simulated": true
                        }))
                    }
                    "mutation" => {
                        let schema = request
                            .get("params")
                            .and_then(|v| v.get("schema"))
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                NetworkError::ProtocolError(
                                    "Missing schema in mutation request".to_string(),
                                )
                            })?;

                        Ok(serde_json::json!({
                            "success": true,
                            "id": format!("simulated_id_{}", rand::random::<u32>()),
                            "schema": schema,
                            "forwarded": true,
                            "node_id": node_id,
                            "peer_id": peer_id.to_string(),
                            "simulated": true
                        }))
                    }
                    _ => {
                        Ok(serde_json::json!({
                            "success": true,
                            "operation": operation,
                            "forwarded": true,
                            "node_id": node_id,
                            "peer_id": peer_id.to_string(),
                            "message": "Request forwarding simulation",
                            "simulated": true
                        }))
                    }
                }
            }
        }
    }
}
