use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::error::{FoldDbError, NetworkErrorKind};
use crate::datafold_node::network::connection::Connection;
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::types::{
    NodeId, NodeInfo, NetworkConfig, ConnectionState
};

/// Manages connections to remote nodes
pub struct ConnectionManager {
    /// Map of known nodes by ID
    nodes: Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
    /// Map of active connections by node ID
    connections: Arc<Mutex<HashMap<NodeId, Arc<Mutex<Connection>>>>>,
    /// Network configuration
    config: NetworkConfig,
    /// TCP listener for incoming connections
    listener: Option<Arc<TcpListener>>,
    /// Whether the connection manager is running
    running: Arc<Mutex<bool>>,
    /// Local node ID
    local_node_id: NodeId,
}

impl ConnectionManager {
    /// Creates a new connection manager
    pub fn new(
        config: NetworkConfig,
        local_node_id: NodeId,
    ) -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
            config,
            listener: None,
            running: Arc::new(Mutex::new(false)),
            local_node_id,
        }
    }

    /// Starts the connection manager
    pub fn start(&mut self) -> NetworkResult<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        // Start TCP listener
        let listener = TcpListener::bind(self.config.listen_address)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to bind listener: {}", e))))?;
        
        listener.set_nonblocking(true)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to set non-blocking: {}", e))))?;
        
        let listener_arc = Arc::new(listener);
        self.listener = Some(Arc::clone(&listener_arc));
        
        // Start connection acceptor thread
        let running = Arc::clone(&self.running);
        let connections = Arc::clone(&self.connections);
        let nodes = Arc::clone(&self.nodes);
        let listener_clone = Arc::clone(&listener_arc);
        let local_node_id = self.local_node_id.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // Accept new connections
                match listener_clone.accept() {
                    Ok((stream, addr)) => {
                        println!("New connection from {}", addr);
                        
                        // Create a new connection
                        match Self::create_connection(stream, addr, &connections, &nodes, local_node_id.clone()) {
                            Ok(_) => {},
                            Err(e) => eprintln!("Error creating connection: {}", e),
                        }
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No new connections, continue
                    },
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
                
                // Sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(100));
            }
        });
        
        // Start connection monitor thread
        let running_clone = Arc::clone(&self.running);
        let connections_clone = Arc::clone(&self.connections);
        let config_clone = self.config.clone();
        
        thread::spawn(move || {
            while *running_clone.lock().unwrap() {
                // Check connection health
                Self::monitor_connections(&connections_clone, config_clone.connection_timeout);
                
                // Sleep before next check
                thread::sleep(Duration::from_secs(5));
            }
        });
        
        Ok(())
    }

    /// Stops the connection manager
    pub fn stop(&mut self) -> NetworkResult<()> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Ok(());
        }
        *running = false;
        drop(running);
        
        // Close all connections
        let mut connections = self.connections.lock().unwrap();
        for (_, connection) in connections.iter_mut() {
            let mut conn = connection.lock().unwrap();
            if let Err(e) = conn.close() {
                eprintln!("Error closing connection: {}", e);
            }
        }
        connections.clear();
        
        Ok(())
    }

    /// Creates a new connection from a TCP stream
    fn create_connection(
        stream: TcpStream,
        addr: SocketAddr,
        connections: &Arc<Mutex<HashMap<NodeId, Arc<Mutex<Connection>>>>>,
        nodes: &Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
        _local_node_id: NodeId,
    ) -> NetworkResult<()> {
        // Create temporary node ID based on address
        let temp_node_id = format!("temp-{}", addr);
        
        // Create connection
        let connection = Connection::new(stream, temp_node_id.clone())?;
        let connection_arc = Arc::new(Mutex::new(connection));
        
        // Add to connections map
        {
            let mut connections_lock = connections.lock().unwrap();
            connections_lock.insert(temp_node_id.clone(), Arc::clone(&connection_arc));
        }
        
        // Start a thread to handle the connection
        let connections_clone = Arc::clone(connections);
        let _nodes_clone = Arc::clone(nodes);
        
        thread::spawn(move || {
            // Wait for identification
            let node_id = {
                let mut connection = connection_arc.lock().unwrap();
                
                // Set connection state to ready
                connection.set_state(ConnectionState::Ready);
                
                // Return the node ID (using temporary ID for now)
                temp_node_id.clone()
            };
            
            // Update connection with correct node ID
            {
                let mut connections_lock = connections_clone.lock().unwrap();
                
                // Remove temporary connection
                connections_lock.remove(&temp_node_id);
                
                // Add with correct node ID
                connections_lock.insert(node_id.clone(), Arc::clone(&connection_arc));
            }
            
            // Monitor connection until it's closed
            while {
                let connection = connection_arc.lock().unwrap();
                connection.is_healthy()
            } {
                // Sleep briefly
                thread::sleep(Duration::from_millis(100));
            }
            
            // Remove from connections map when closed
            let mut connections_lock = connections_clone.lock().unwrap();
            connections_lock.remove(&node_id);
        });
        
        Ok(())
    }

    /// Connects to a node by ID
    pub fn connect_to_node(&self, node_id: &NodeId) -> NetworkResult<()> {
        // Check if already connected
        {
            let connections = self.connections.lock().unwrap();
            if connections.contains_key(node_id) {
                return Ok(());
            }
        }
        
        // Get node info
        let node_info = {
            let nodes = self.nodes.lock().unwrap();
            nodes.get(node_id).cloned().ok_or_else(|| {
                FoldDbError::Network(NetworkErrorKind::Connection(format!("Node {} not found", node_id)))
            })?
        };
        
        // Connect to the node
        let connection = Connection::connect(
            &node_info.address.to_string(),
            node_id.clone(),
            self.config.connection_timeout,
        )?;
        
        // Add to connections map
        let mut connections = self.connections.lock().unwrap();
        connections.insert(node_id.clone(), Arc::new(Mutex::new(connection)));
        
        Ok(())
    }

    /// Gets a connection by node ID
    pub fn get_connection(&self, node_id: &NodeId) -> NetworkResult<Arc<Mutex<Connection>>> {
        let connections = self.connections.lock().unwrap();
        connections.get(node_id).cloned().ok_or_else(|| {
            FoldDbError::Network(NetworkErrorKind::Connection(format!("Not connected to node {}", node_id)))
        })
    }

    /// Adds a node to the known nodes list
    pub fn add_node(&self, node_info: NodeInfo) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node_info.node_id.clone(), node_info);
    }

    /// Gets the list of connected nodes
    pub fn connected_nodes(&self) -> HashSet<NodeId> {
        let connections = self.connections.lock().unwrap();
        connections.keys().cloned().collect()
    }

    /// Gets the list of known nodes
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo> {
        self.nodes.lock().unwrap().clone()
    }

    /// Monitors connections and closes unhealthy ones
    fn monitor_connections(
        connections: &Arc<Mutex<HashMap<NodeId, Arc<Mutex<Connection>>>>>,
        timeout: Duration,
    ) {
        let mut to_remove = Vec::new();
        
        // Check each connection
        let connections_lock = connections.lock().unwrap();
        for (node_id, connection_arc) in connections_lock.iter() {
            let connection = connection_arc.lock().unwrap();
            
            // Check if connection is healthy
            if !connection.is_healthy() {
                to_remove.push(node_id.clone());
                continue;
            }
            
            // Check if connection has timed out
            if connection.time_since_last_seen() > timeout {
                to_remove.push(node_id.clone());
                continue;
            }
        }
        drop(connections_lock);
        
        // Remove unhealthy connections
        if !to_remove.is_empty() {
            let mut connections_lock = connections.lock().unwrap();
            for node_id in to_remove {
                connections_lock.remove(&node_id);
            }
        }
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            eprintln!("Error stopping connection manager: {}", e);
        }
    }
}
