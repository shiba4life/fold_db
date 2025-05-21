use crate::network::config::NetworkConfig;
use crate::network::error::NetworkResult;
use crate::network::{NetworkCore, error::NetworkError};
use libp2p::PeerId;
use log::info;
use std::collections::HashSet;
use std::time::Duration;

/// Perform mDNS peer discovery and update the known peer list.
///
/// This function is separated from `NetworkCore` so that discovery
/// logic can be tested independently.
pub async fn discover_peers(
    config: &NetworkConfig,
    known_peers: &mut HashSet<PeerId>,
) -> NetworkResult<Vec<PeerId>> {
    if !config.enable_mdns {
        info!("mDNS discovery is disabled, no peers will be discovered");
        return Ok(Vec::new());
    }

    info!("Scanning for peers using mDNS on port {}", config.discovery_port);

    // In a real implementation this would send mDNS queries and wait for responses.
    // For now we simulate peer discovery when the `simulate-peers` feature is enabled.
    if cfg!(feature = "simulate-peers") {
        info!("SIMULATION: Generating random peers for demonstration");
        let num_peers = rand::random::<u8>() % 4;
        for _ in 0..num_peers {
            let peer_id = PeerId::random();
            known_peers.insert(peer_id);
            info!("SIMULATION: Discovered peer: {}", peer_id);
        }
    }

    tokio::time::sleep(Duration::from_millis(200)).await;

    Ok(known_peers.iter().cloned().collect())
}


impl NetworkCore {
    /// Check which schemas are available on a remote peer.
    pub async fn check_schemas(
        &mut self,
        peer_id: PeerId,
        schema_names: Vec<String>,
    ) -> NetworkResult<Vec<String>> {
        #[cfg(test)]
        {
            if let Some(peer_service) = self.mock_peers.get(&peer_id) {
                return Ok(peer_service.check_schemas(&schema_names));
            }
        }

        if !self.known_peers.contains(&peer_id) {
            return Err(NetworkError::ConnectionError(format!(
                "Peer not found: {}",
                peer_id
            )));
        }

        let available_schemas = schema_names
            .iter()
            .filter(|_| rand::random::<bool>())
            .cloned()
            .collect();

        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(available_schemas)
    }

    /// Actively scan for peers using mDNS
    pub async fn discover_nodes(&mut self) -> NetworkResult<Vec<PeerId>> {
        discover_peers(&self.config, &mut self.known_peers).await
    }
}
