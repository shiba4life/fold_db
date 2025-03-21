use std::collections::HashSet;
use crate::datafold_node::network::schema_protocol::{SchemaCodec, SchemaMessage};

/// Network behavior for the DataFold node
/// 
/// This is a simplified version of the network behavior that will be expanded
/// in future iterations to include more libp2p functionality.
#[derive(Debug, Clone)]
pub struct FoldDbBehaviour {
    /// Codec for serializing/deserializing schema messages
    pub schema_codec: SchemaCodec,
    /// Set of discovered peers
    pub discovered_peers: HashSet<String>,
}

impl FoldDbBehaviour {
    /// Create a new FoldDbBehaviour
    pub fn new() -> Self {
        Self {
            schema_codec: SchemaCodec,
            discovered_peers: HashSet::new(),
        }
    }
    
    /// Add a discovered peer
    pub fn add_discovered_peer(&mut self, peer_id: String) {
        self.discovered_peers.insert(peer_id);
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.discovered_peers.remove(peer_id);
    }
    
    /// Get the list of discovered peers
    pub fn get_discovered_peers(&self) -> HashSet<String> {
        self.discovered_peers.clone()
    }
}

/// Events emitted by the network behavior
#[derive(Debug)]
pub enum NetworkEvent {
    /// mDNS discovery event
    Mdns(MdnsEvent),
    /// Schema request-response event
    SchemaReqResp(SchemaReqRespEvent),
}

/// mDNS discovery events
#[derive(Debug)]
pub enum MdnsEvent {
    /// Discovered a new peer
    Discovered(Vec<(String, String)>), // (peer_id, address)
    /// A peer expired
    Expired(String), // peer_id
}

/// Schema request-response events
#[derive(Debug)]
pub enum SchemaReqRespEvent {
    /// Received a message
    Message {
        /// Peer ID
        peer: String,
        /// Message
        message: SchemaReqRespMessage,
    },
    /// Response sent
    ResponseSent {
        /// Peer ID
        peer: String,
    },
}

/// Schema request-response messages
#[derive(Debug)]
pub enum SchemaReqRespMessage {
    /// Request
    Request {
        /// Request ID
        request_id: u64,
        /// Request
        request: SchemaMessage,
        /// Response channel
        channel: u64,
    },
    /// Response
    Response {
        /// Request ID
        request_id: u64,
        /// Response
        response: SchemaMessage,
    },
}
