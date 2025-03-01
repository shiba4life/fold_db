use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use serde_json;
use uuid::Uuid;

use crate::error::{FoldDbError, NetworkErrorKind};
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::message::{
    Message, PingMessage, PongMessage, ErrorMessage, ErrorCode
};
use crate::datafold_node::network::types::{NodeId, ConnectionState};

/// Manages a connection to a remote node
pub struct Connection {
    /// Unique identifier for the connected node
    node_id: NodeId,
    /// TCP stream for communication
    stream: TcpStream,
    /// Trust distance to this node
    trust_distance: u32,
    /// Last time activity was seen on this connection
    last_seen: Instant,
    /// Current state of the connection
    state: ConnectionState,
    /// Unique identifier for this connection
    connection_id: Uuid,
}

impl Connection {
    /// Creates a new connection from an existing TCP stream
    pub fn new(stream: TcpStream, node_id: NodeId) -> NetworkResult<Self> {
        // Set TCP options
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;

        Ok(Self {
            node_id,
            stream,
            trust_distance: 0,
            last_seen: Instant::now(),
            state: ConnectionState::Connected,
            connection_id: Uuid::new_v4(),
        })
    }

    /// Creates a new connection by connecting to a remote address
    pub fn connect(address: &str, node_id: NodeId, timeout: Duration) -> NetworkResult<Self> {
        let stream = TcpStream::connect(address)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to connect: {}", e))))?;
        
        let mut connection = Self::new(stream, node_id)?;
        connection.state = ConnectionState::Connecting;
        
        // Set connection timeout
        connection.stream.set_read_timeout(Some(timeout))?;
        connection.stream.set_write_timeout(Some(timeout))?;
        
        Ok(connection)
    }

    /// Sends a message to the connected node
    pub fn send_message(&mut self, message: Message) -> NetworkResult<()> {
        if self.state == ConnectionState::Closed || self.state == ConnectionState::Failed {
            return Err(FoldDbError::Network(NetworkErrorKind::Connection("Connection is closed".to_string())));
        }

        let serialized = serde_json::to_string(&message)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Message(format!("Failed to serialize message: {}", e))))?;
        
        // Send message length as 4-byte prefix
        let length = serialized.len() as u32;
        let length_bytes = length.to_be_bytes();
        
        self.stream.write_all(&length_bytes)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to send message length: {}", e))))?;
        
        // Send message content
        self.stream.write_all(serialized.as_bytes())
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to send message: {}", e))))?;
        
        self.stream.flush()
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to flush stream: {}", e))))?;
        
        self.last_seen = Instant::now();
        Ok(())
    }

    /// Receives a message from the connected node
    pub fn receive_message(&mut self) -> NetworkResult<Message> {
        if self.state == ConnectionState::Closed || self.state == ConnectionState::Failed {
            return Err(FoldDbError::Network(NetworkErrorKind::Connection("Connection is closed".to_string())));
        }

        // Read message length (4-byte prefix)
        let mut length_bytes = [0u8; 4];
        self.stream.read_exact(&mut length_bytes)
            .map_err(|e| {
                self.state = ConnectionState::Failed;
                FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to read message length: {}", e)))
            })?;
        
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        // Read message content
        let mut buffer = vec![0u8; length];
        self.stream.read_exact(&mut buffer)
            .map_err(|e| {
                self.state = ConnectionState::Failed;
                FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to read message content: {}", e)))
            })?;
        
        // Deserialize message
        let message = serde_json::from_slice::<Message>(&buffer)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Message(format!("Failed to deserialize message: {}", e))))?;
        
        self.last_seen = Instant::now();
        
        // Automatically respond to ping messages
        if let Message::Ping(ping) = &message {
            self.handle_ping(ping)?;
            // Continue and return the ping message anyway
        }
        
        Ok(message)
    }

    /// Handles a ping message by sending a pong response
    fn handle_ping(&mut self, ping: &PingMessage) -> NetworkResult<()> {
        let pong = Message::Pong(PongMessage {
            ping_id: ping.ping_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        
        self.send_message(pong)
    }

    /// Sends a ping message to check if the connection is alive
    pub fn ping(&mut self) -> NetworkResult<()> {
        let ping = Message::Ping(PingMessage {
            ping_id: Uuid::new_v4(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        
        self.send_message(ping)
    }

    /// Sends an error message to the connected node
    pub fn send_error(&mut self, code: ErrorCode, message: &str, details: Option<&str>, related_id: Option<Uuid>) -> NetworkResult<()> {
        let error = Message::Error(ErrorMessage {
            code,
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
            related_message_id: related_id,
        });
        
        self.send_message(error)
    }

    /// Validates the connected node's identity
    pub fn validate_node(&self) -> NetworkResult<()> {
        // In a real implementation, this would verify the node's public key
        // and check trust distance requirements
        Ok(())
    }

    /// Checks if the connection is healthy
    pub fn is_healthy(&self) -> bool {
        self.state == ConnectionState::Connected || self.state == ConnectionState::Ready
    }

    /// Closes the connection
    pub fn close(&mut self) -> NetworkResult<()> {
        self.state = ConnectionState::Closing;
        self.stream.shutdown(std::net::Shutdown::Both)
            .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Failed to close connection: {}", e))))?;
        self.state = ConnectionState::Closed;
        Ok(())
    }

    /// Gets the node ID for this connection
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Gets the connection ID
    pub fn connection_id(&self) -> Uuid {
        self.connection_id
    }

    /// Gets the connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Sets the connection state
    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    /// Gets the trust distance for this connection
    pub fn trust_distance(&self) -> u32 {
        self.trust_distance
    }

    /// Sets the trust distance for this connection
    pub fn set_trust_distance(&mut self, trust_distance: u32) {
        self.trust_distance = trust_distance;
    }

    /// Gets the time since the last activity on this connection
    pub fn time_since_last_seen(&self) -> Duration {
        self.last_seen.elapsed()
    }
}
