use std::collections::HashSet;
use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json;

use crate::error::{FoldDbError, NetworkErrorKind};
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::message::{Message, NodeAnnouncementMessage};
use crate::datafold_node::network::types::{NodeId, NodeInfo, NodeCapabilities};

/// Configuration for node discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Port to use for discovery broadcasts
    pub discovery_port: u16,
    /// Interval for sending node announcements
    pub announcement_interval: Duration,
    /// Whether to enable automatic node discovery
    pub enable_discovery: bool,
    /// Local node information
    pub local_node_info: NodeInfo,
}

/// Handles node discovery on the network
pub struct NodeDiscovery {
    /// UDP socket for discovery broadcasts
    socket: UdpSocket,
    /// Set of known node IDs
    known_nodes: Arc<Mutex<HashSet<NodeId>>>,
    /// Configuration for discovery
    config: DiscoveryConfig,
    /// Last time an announcement was sent
    last_announcement: Instant,
    /// Whether the discovery service is running
    running: Arc<Mutex<bool>>,
}

impl NodeDiscovery {
    /// Creates a new node discovery service
    pub fn new(config: DiscoveryConfig) -> NetworkResult<Self> {
        // Bind to the discovery port
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", config.discovery_port))
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Discovery(format!("Failed to bind discovery socket: {}", e))))?;
        
        // Enable broadcast
        socket.set_broadcast(true)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Discovery(format!("Failed to set broadcast: {}", e))))?;
        
        // Set read timeout
        socket.set_read_timeout(Some(Duration::from_secs(1)))
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Discovery(format!("Failed to set read timeout: {}", e))))?;
        
        Ok(Self {
            socket,
            known_nodes: Arc::new(Mutex::new(HashSet::new())),
            config,
            last_announcement: Instant::now() - Duration::from_secs(3600), // Force immediate announcement
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Starts the discovery service in a background thread
    pub fn start(&mut self) -> NetworkResult<()> {
        if !self.config.enable_discovery {
            return Ok(());
        }

        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let socket_clone = self.socket.try_clone()
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Discovery(format!("Failed to clone socket: {}", e))))?;
        
        let known_nodes = Arc::clone(&self.known_nodes);
        // Clone config only if we need it
        let _config = self.config.clone();
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            
            while *running.lock().unwrap() {
                // Check for incoming announcements
                match socket_clone.recv_from(&mut buffer) {
                    Ok((size, src)) => {
                        if let Err(e) = Self::handle_announcement(&buffer[..size], src, &known_nodes) {
                            eprintln!("Error handling announcement: {}", e);
                        }
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Timeout, continue
                    },
                    Err(e) => {
                        eprintln!("Error receiving discovery message: {}", e);
                    }
                }

                // Sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(100));
            }
        });

        Ok(())
    }

    /// Stops the discovery service
    pub fn stop(&mut self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Handles an incoming node announcement
    fn handle_announcement(
        data: &[u8], 
        src: SocketAddr, 
        known_nodes: &Arc<Mutex<HashSet<NodeId>>>
    ) -> NetworkResult<()> {
        // Deserialize the announcement
        let message = serde_json::from_slice::<Message>(data)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Message(format!("Failed to deserialize announcement: {}", e))))?;
        
        if let Message::NodeAnnouncement(announcement) = message {
            // Update the node's address if it doesn't match the source
            let mut node_info = announcement.node_info;
            if node_info.address != src {
                node_info.address = src;
            }
            
            // Add to known nodes
            let mut nodes = known_nodes.lock().unwrap();
            nodes.insert(node_info.node_id.clone());
            
            // In a real implementation, we would store the node info in a more persistent way
            // and notify interested parties about the new node
        }
        
        Ok(())
    }

    /// Announces this node's presence on the network
    pub fn announce_presence(&mut self) -> NetworkResult<()> {
        if !self.config.enable_discovery {
            return Ok(());
        }

        // Check if it's time to send another announcement
        let now = Instant::now();
        if now.duration_since(self.last_announcement) < self.config.announcement_interval {
            return Ok(());
        }
        
        self.last_announcement = now;
        
        // Create announcement message
        let announcement = Message::NodeAnnouncement(NodeAnnouncementMessage {
            node_info: self.config.local_node_info.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        
        // Serialize the announcement
        let data = serde_json::to_vec(&announcement)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Message(format!("Failed to serialize announcement: {}", e))))?;
        
        // Broadcast to the discovery port
        let broadcast_addr = format!("255.255.255.255:{}", self.config.discovery_port);
        self.socket.send_to(&data, broadcast_addr)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Discovery(format!("Failed to send announcement: {}", e))))?;
        
        Ok(())
    }

    /// Finds nodes on the network
    pub fn find_nodes(&mut self) -> NetworkResult<Vec<NodeInfo>> {
        if !self.config.enable_discovery {
            return Ok(Vec::new());
        }

        // Announce presence to trigger responses
        self.announce_presence()?;
        
        // In a real implementation, we would wait for responses and collect node info
        // For now, we'll just return an empty list
        Ok(Vec::new())
    }

    /// Gets the set of known node IDs
    pub fn known_nodes(&self) -> HashSet<NodeId> {
        self.known_nodes.lock().unwrap().clone()
    }

    /// Creates a local node info structure
    pub fn create_local_node_info(
        node_id: NodeId,
        address: SocketAddr,
        trust_distance: u32,
        public_key: Option<String>,
    ) -> NodeInfo {
        NodeInfo {
            node_id,
            address,
            trust_distance,
            public_key,
            capabilities: NodeCapabilities {
                supports_query: true,
                supports_schema_listing: true,
            },
        }
    }
}

impl Drop for NodeDiscovery {
    fn drop(&mut self) {
        self.stop();
    }
}
