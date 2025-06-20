use log::info;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{FoldDbError, FoldDbResult, NetworkErrorKind};
use crate::fold_db_core::FoldDB;
use crate::network::{NetworkConfig, NetworkCore, PeerId};
use crate::security::{EncryptionManager, SecurityManager};

use super::DataFoldNode;
use super::config::NodeInfo;
use super::node::NetworkStatus;

impl DataFoldNode {
    /// Initialize the network layer
    pub async fn init_network(&mut self, network_config: NetworkConfig) -> FoldDbResult<()> {
        let network_core = NetworkCore::new(network_config)
            .await
            .map_err(|e| FoldDbError::Network(e.into()))?;

        let mut network_core = network_core;
        let db_clone = self.db.clone();

        network_core
            .schema_service_mut()
            .set_schema_check_callback(move |schema_names| {
                let db = match db_clone.lock() {
                    Ok(db) => db,
                    Err(_) => return Vec::new(),
                };

                schema_names
                    .iter()
                    .filter(|name| matches!(db.schema_manager.get_schema(name), Ok(Some(_))))
                    .cloned()
                    .collect()
            });

        let local_peer_id = network_core.local_peer_id();
        network_core.register_node_id(&self.node_id, local_peer_id);
        info!(
            "Registered node ID {} with peer ID {}",
            self.node_id, local_peer_id
        );

        self.network = Some(Arc::new(tokio::sync::Mutex::new(network_core)));

        Ok(())
    }

    /// Start the network service using the node configuration address
    pub async fn start_network(&self) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;
            let address = &self.config.network_listen_address;
            network
                .run(address)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Start the network service with a specific listen address
    pub async fn start_network_with_address(&self, listen_address: &str) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;
            network
                .run(listen_address)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Stop the network service
    pub async fn stop_network(&self) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let mut network_guard = network.lock().await;
            info!("Stopping network service");
            network_guard.stop();
            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Get a mutable reference to the network core
    pub async fn get_network_mut(&self) -> FoldDbResult<tokio::sync::MutexGuard<'_, NetworkCore>> {
        if let Some(network) = &self.network {
            Ok(network.lock().await)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Discover nodes on the local network using mDNS
    pub async fn discover_nodes(&self) -> FoldDbResult<Vec<PeerId>> {
        if let Some(network) = &self.network {
            let network_guard = network.lock().await;

            info!("Triggering mDNS discovery...");

            let known_peers: Vec<PeerId> = network_guard.known_peers().iter().cloned().collect();

            Ok(known_peers)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Get the list of known nodes
    pub async fn get_known_nodes(&self) -> FoldDbResult<HashMap<String, NodeInfo>> {
        if let Some(network) = &self.network {
            let network_guard = network.lock().await;

            let mut result = HashMap::new();
            for peer_id in network_guard.known_peers() {
                let peer_id_str = peer_id.to_string();

                if let Some(info) = self.trusted_nodes.get(&peer_id_str) {
                    result.insert(peer_id_str, info.clone());
                } else {
                    result.insert(
                        peer_id_str.clone(),
                        NodeInfo {
                            id: peer_id_str,
                            trust_distance: self.config.default_trust_distance,
                        },
                    );
                }
            }

            Ok(result)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Check which schemas are available on a remote peer
    pub async fn check_remote_schemas(
        &self,
        peer_id_str: &str,
        schema_names: Vec<String>,
    ) -> FoldDbResult<Vec<String>> {
        if let Some(network) = &self.network {
            let peer_id = peer_id_str.parse::<PeerId>().map_err(|e| {
                FoldDbError::Network(NetworkErrorKind::Connection(format!(
                    "Invalid peer ID: {}",
                    e
                )))
            })?;

            let mut network = network.lock().await;
            let result = network
                .check_schemas(peer_id, schema_names)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            Ok(result)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Forward a request to another node
    pub async fn forward_request(&self, peer_id: PeerId, request: Value) -> FoldDbResult<Value> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;

            let node_id = network
                .get_node_id_for_peer(&peer_id)
                .unwrap_or_else(|| peer_id.to_string());

            info!("Forwarding request to node {} (peer {})", node_id, peer_id);

            let response = network
                .forward_request(peer_id, request)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            info!("Received response from node {} (peer {})", node_id, peer_id);

            Ok(response)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Simple method to connect to another node
    pub async fn connect_to_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.add_trusted_node(node_id)
    }

    /// Retrieve basic network status information
    pub async fn get_network_status(&self) -> FoldDbResult<NetworkStatus> {
        let initialized = self.network.is_some();
        let connected_nodes_count = if let Some(network) = &self.network {
            let guard = network.lock().await;
            guard.known_peers().len()
        } else {
            0
        };
        Ok(NetworkStatus {
            node_id: self.node_id.clone(),
            initialized,
            connected_nodes_count,
        })
    }

    /// Restart the node by reinitializing all components
    pub async fn restart(&mut self) -> FoldDbResult<()> {
        info!("Restarting DataFoldNode...");

        if self.network.is_some() {
            info!("Stopping network service for restart");
            if let Err(e) = self.stop_network().await {
                log::warn!("Failed to stop network during restart: {}", e);
            }
        }

        let storage_path = self
            .config
            .storage_path
            .to_str()
            .ok_or_else(|| FoldDbError::Config("Invalid storage path".to_string()))?
            .to_string();

        info!("Closing existing database");
        let old_db = std::mem::replace(
            &mut self.db,
            Arc::new(Mutex::new(FoldDB::new(&format!("{}_temp", storage_path))?)),
        );

        drop(old_db);

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        info!("Reinitializing database");
        let new_db = Arc::new(Mutex::new(FoldDB::new(&storage_path)?));
        self.db = new_db;

        self.network = None;
        self.trusted_nodes.clear();

        let mut security_config = self.config.security_config.clone();
        if security_config.encrypt_at_rest && security_config.master_key.is_none() {
            security_config.master_key = Some(EncryptionManager::generate_master_key());
        }
        self.security_manager = Arc::new(
            SecurityManager::new(security_config)
                .map_err(|e| FoldDbError::SecurityError(e.to_string()))?,
        );

        info!("DataFoldNode restart completed successfully");
        Ok(())
    }

    /// Perform a soft restart that preserves network connections
    pub async fn soft_restart(&mut self) -> FoldDbResult<()> {
        info!("Performing soft restart of DataFoldNode...");

        let storage_path = self
            .config
            .storage_path
            .to_str()
            .ok_or_else(|| FoldDbError::Config("Invalid storage path".to_string()))?
            .to_string();

        info!("Closing existing database");
        let old_db = std::mem::replace(
            &mut self.db,
            Arc::new(Mutex::new(FoldDB::new(&format!("{}_temp", storage_path))?)),
        );

        drop(old_db);

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        info!("Reinitializing database");
        let new_db = Arc::new(Mutex::new(FoldDB::new(&storage_path)?));
        self.db = new_db;

        info!("DataFoldNode soft restart completed successfully");
        Ok(())
    }
}

