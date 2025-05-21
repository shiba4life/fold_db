use crate::network::config::NetworkConfig;
use crate::network::error::{NetworkResult};
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_discover_peers_mdns_disabled() {
        let mut config = NetworkConfig::default();
        config.enable_mdns = false;
        let mut known_peers = HashSet::new();

        let peers = discover_peers(&config, &mut known_peers)
            .await
            .expect("discovery should succeed");

        assert!(peers.is_empty(), "expected no peers when mDNS disabled");
        assert!(known_peers.is_empty());
    }

    #[cfg(feature = "simulate-peers")]
    #[tokio::test]
    async fn test_discover_peers_with_simulated_peers() {
        let config = NetworkConfig::default();
        let mut known_peers = HashSet::new();

        let peers = discover_peers(&config, &mut known_peers)
            .await
            .expect("discovery should succeed");

        assert!(!peers.is_empty(), "expected simulated peers to be discovered");
        assert_eq!(peers.len(), known_peers.len());
    }
}
