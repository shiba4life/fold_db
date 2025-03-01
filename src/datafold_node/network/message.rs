use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::schema::types::Query;
use crate::datafold_node::network::types::{SchemaInfo, NodeInfo, SerializableQueryResult};

/// Message types for node communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Query request to another node
    Query(QueryMessage),
    /// Response to a query request
    QueryResponse(QueryResponseMessage),
    /// Request to list available schemas
    ListSchemasRequest(ListSchemasRequestMessage),
    /// Response with available schemas
    SchemaListResponse(SchemaListResponseMessage),
    /// Node announcement message
    NodeAnnouncement(NodeAnnouncementMessage),
    /// Error message
    Error(ErrorMessage),
    /// Ping message to check connection health
    Ping(PingMessage),
    /// Pong response to a ping message
    Pong(PongMessage),
}

/// Message for querying data from another node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMessage {
    /// Unique identifier for this query
    pub query_id: Uuid,
    /// The query to execute
    pub query: Query,
    /// Trust proof for authentication
    pub trust_proof: TrustProof,
}

/// Message with the response to a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponseMessage {
    /// Unique identifier matching the original query
    pub query_id: Uuid,
    /// The result of the query
    pub result: SerializableQueryResult,
}

/// Message requesting available schemas from a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSchemasRequestMessage {
    /// Unique identifier for this request
    pub request_id: Uuid,
}

/// Message with the response to a schema list request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaListResponseMessage {
    /// Unique identifier matching the original request
    pub request_id: Uuid,
    /// List of available schemas
    pub schemas: Vec<SchemaInfo>,
}

/// Message announcing a node's presence on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnnouncementMessage {
    /// Information about the announcing node
    pub node_info: NodeInfo,
    /// Timestamp of the announcement
    pub timestamp: u64,
}

/// Message indicating an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// Error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Optional additional details
    pub details: Option<String>,
    /// ID of the message that caused the error, if applicable
    pub related_message_id: Option<Uuid>,
}

/// Message to check if a connection is alive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    /// Unique identifier for this ping
    pub ping_id: Uuid,
    /// Timestamp when the ping was sent
    pub timestamp: u64,
}

/// Response to a ping message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    /// Unique identifier matching the original ping
    pub ping_id: Uuid,
    /// Timestamp when the pong was sent
    pub timestamp: u64,
}

/// Proof of trust for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustProof {
    /// Public key of the requesting node
    pub public_key: String,
    /// Signature of the request
    pub signature: String,
    /// Trust distance claimed by the requesting node
    pub trust_distance: u32,
}

/// Error codes for network errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Invalid message format
    InvalidMessage = 1000,
    /// Authentication failed
    AuthenticationFailed = 1001,
    /// Trust validation failed
    TrustValidationFailed = 1002,
    /// Schema not found
    SchemaNotFound = 1003,
    /// Permission denied
    PermissionDenied = 1004,
    /// Query execution failed
    QueryFailed = 1005,
    /// Internal server error
    InternalError = 1006,
    /// Protocol error
    ProtocolError = 1007,
    /// Timeout error
    Timeout = 1008,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMessage => write!(f, "Invalid message"),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::TrustValidationFailed => write!(f, "Trust validation failed"),
            Self::SchemaNotFound => write!(f, "Schema not found"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::QueryFailed => write!(f, "Query failed"),
            Self::InternalError => write!(f, "Internal error"),
            Self::ProtocolError => write!(f, "Protocol error"),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}
