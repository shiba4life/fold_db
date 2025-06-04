//! # Internal Message Bus for FoldDB Core
//!
//! Provides a foundational event-driven messaging system for migrating fold_db_core
//! to an event-driven architecture. This module implements a simple pub/sub message bus
//! using Rust channels for internal communication between components.
//!
//! ## Design Goals
//! - Enable loose coupling between database components
//! - Support both synchronous and asynchronous event handling
//! - Provide a foundation for eventual migration to full event-driven architecture
//! - Maintain high performance with minimal overhead
//!
//! ## Usage Example
//! ```rust
//! use fold_node::fold_db_core::infrastructure::message_bus::{MessageBus, FieldValueSet};
//! use serde_json::json;
//!
//! let mut bus = MessageBus::new();
//!
//! // Register a consumer for field value events
//! let mut receiver = bus.subscribe::<FieldValueSet>();
//!
//! // Send an event
//! bus.publish(FieldValueSet {
//!     field: "user.name".to_string(),
//!     value: json!("Alice"),
//!     source: "mutation_engine".to_string(),
//! });
//!
//! // Receive the event
//! if let Ok(event) = receiver.try_recv() {
//!     println!("Received event: {:?}", event);
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::mpsc as async_mpsc;
use tokio::time::{Duration as AsyncDuration, timeout};

/// Errors that can occur within the message bus system
#[derive(Error, Debug)]
pub enum MessageBusError {
    /// Failed to send a message to subscribers
    #[error("Failed to send message: {reason}")]
    SendFailed { reason: String },
    
    /// Failed to register a consumer
    #[error("Failed to register consumer for event type: {event_type}")]
    RegistrationFailed { event_type: String },
    
    /// Channel is disconnected
    #[error("Channel disconnected for event type: {event_type}")]
    ChannelDisconnected { event_type: String },
}

/// Result type for message bus operations
pub type MessageBusResult<T> = Result<T, MessageBusError>;

// ========== Core Event Types ==========

/// Event indicating that a field value has been set
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueSet {
    /// The field that was set (e.g., "user.name", "schema.field_name")
    pub field: String,
    /// The value that was set
    pub value: Value,
    /// The source component that triggered this event
    pub source: String,
}

/// Event indicating that a new atom has been created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomCreated {
    /// Unique identifier for the created atom
    pub atom_id: String,
    /// The data contained in the atom
    pub data: Value,
}

/// Event indicating that an existing atom has been updated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomUpdated {
    /// Unique identifier for the updated atom
    pub atom_id: String,
    /// The updated data contained in the atom
    pub data: Value,
}

/// Event indicating that an AtomRef has been created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefCreated {
    /// Unique identifier for the created AtomRef
    pub aref_uuid: String,
    /// Type of AtomRef (Range, Collection, Single)
    pub aref_type: String,
    /// Field path where AtomRef was created
    pub field_path: String,
}

/// Event indicating that an AtomRef has been updated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefUpdated {
    /// Unique identifier for the updated AtomRef
    pub aref_uuid: String,
    /// Field path where AtomRef was updated
    pub field_path: String,
    /// Operation performed (add, update, delete)
    pub operation: String,
}

/// Event indicating that a schema has been loaded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaLoaded {
    /// Name of the schema that was loaded
    pub schema_name: String,
    /// Status of the load operation (success, failed, approved, blocked)
    pub status: String,
}

/// Event indicating that a transform has been executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformExecuted {
    /// Unique identifier for the executed transform
    pub transform_id: String,
    /// Result of the transform execution (success, failed, etc.)
    pub result: String,
}

/// Event indicating that a schema has been changed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaChanged {
    /// The schema that was changed
    pub schema: String,
}

/// Event indicating that a transform has been triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformTriggered {
    /// Unique identifier for the transform that was triggered
    pub transform_id: String,
}

/// Event indicating that a query has been executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryExecuted {
    /// Type of query executed (range_query, single_query, etc.)
    pub query_type: String,
    /// Schema being queried
    pub schema: String,
    /// Time taken to execute the query in milliseconds
    pub execution_time_ms: u64,
    /// Number of results returned
    pub result_count: usize,
}

/// Event indicating that a mutation has been executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MutationExecuted {
    /// Type of operation performed (create, update, delete)
    pub operation: String,
    /// Schema being mutated
    pub schema: String,
    /// Time taken to execute the mutation in milliseconds
    pub execution_time_ms: u64,
    /// Number of fields affected
    pub fields_affected: usize,
}

// ========== Request/Response Event Types ==========

/// Request to create a new atom
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomCreateRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name for the atom
    pub schema_name: String,
    /// Source public key
    pub source_pub_key: String,
    /// Previous atom UUID (optional)
    pub prev_atom_uuid: Option<String>,
    /// Content for the atom
    pub content: Value,
    /// Atom status (optional)
    pub status: Option<String>,
}

/// Response to atom creation request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomCreateResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Created atom UUID (if successful)
    pub atom_uuid: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Created atom data (if successful)
    pub atom_data: Option<Value>,
}

/// Request to update an atom
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomUpdateRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Atom UUID to update
    pub atom_uuid: String,
    /// Updated content
    pub content: Value,
    /// Source public key
    pub source_pub_key: String,
}

/// Response to atom update request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomUpdateResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to create an AtomRef
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefCreateRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// AtomRef UUID
    pub aref_uuid: String,
    /// Atom UUID to reference
    pub atom_uuid: String,
    /// Source public key
    pub source_pub_key: String,
    /// AtomRef type (Single, Collection, Range)
    pub aref_type: String,
}

/// Response to AtomRef creation request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefCreateResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to update an AtomRef
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefUpdateRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// AtomRef UUID to update
    pub aref_uuid: String,
    /// New atom UUID
    pub atom_uuid: String,
    /// Source public key
    pub source_pub_key: String,
    /// AtomRef type (Single, Collection, Range)
    pub aref_type: String,
    /// Additional data for Collection/Range types
    pub additional_data: Option<Value>,
}

/// Response to AtomRef update request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefUpdateResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to set a field value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueSetRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name
    pub schema_name: String,
    /// Field name
    pub field_name: String,
    /// Value to set
    pub value: Value,
    /// Source public key
    pub source_pub_key: String,
}

/// Response to field value set request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueSetResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// AtomRef UUID created/used
    pub aref_uuid: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to update a field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldUpdateRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name
    pub schema_name: String,
    /// Field name
    pub field_name: String,
    /// New value
    pub value: Value,
    /// Source public key
    pub source_pub_key: String,
}

/// Response to field update request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldUpdateResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// AtomRef UUID used
    pub aref_uuid: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to load a schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaLoadRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name to load
    pub schema_name: String,
}

/// Response to schema load request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaLoadResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Loaded schema data (if successful)
    pub schema_data: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to approve a schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaApprovalRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name to approve
    pub schema_name: String,
}

/// Response to schema approval request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaApprovalResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to get atom history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomHistoryRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// AtomRef UUID to get history for
    pub aref_uuid: String,
}

/// Response to atom history request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomHistoryResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// History data (if successful)
    pub history: Option<Vec<Value>>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to get latest atom
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomGetRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// AtomRef UUID to get latest atom for
    pub aref_uuid: String,
}

/// Response to get latest atom request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomGetResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Atom data (if successful)
    pub atom_data: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to query a field value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueQueryRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name
    pub schema_name: String,
    /// Field name
    pub field_name: String,
    /// Optional filter for the field value
    pub filter: Option<Value>,
}

/// Response to field value query request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueQueryResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Field value data (if successful)
    pub field_value: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to query if an AtomRef exists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefQueryRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// AtomRef UUID to check
    pub aref_uuid: String,
}

/// Response to AtomRef query request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefQueryResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Whether the AtomRef exists
    pub exists: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to trigger a transform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformTriggerRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name
    pub schema_name: String,
    /// Field name
    pub field_name: String,
    /// Mutation hash for the transform
    pub mutation_hash: String,
}

/// Response to transform trigger request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformTriggerResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to get schema status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaStatusRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
}

/// Response to schema status request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaStatusResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Schema status data (if successful)
    pub status_data: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to discover and load all schemas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDiscoveryRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
}

/// Response to schema discovery request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDiscoveryResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Discovery report data (if successful)
    pub report_data: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to get an AtomRef
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefGetRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// AtomRef UUID to get
    pub aref_uuid: String,
}

/// Response to AtomRef get request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefGetResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// AtomRef data (if successful)
    pub aref_data: Option<Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to update a collection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectionUpdateRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Schema name
    pub schema_name: String,
    /// Field name
    pub field_name: String,
    /// Operation type (add, update, delete)
    pub operation: String,
    /// Value for the operation
    pub value: Value,
    /// Source public key
    pub source_pub_key: String,
    /// Collection item ID (for update/delete operations)
    pub item_id: Option<String>,
}

/// Response to collection update request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectionUpdateResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}


/// Request to execute transforms in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformExecutionRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
}

/// Response to transform execution request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformExecutionResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Number of transforms executed
    pub transforms_executed: usize,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request for system initialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInitializationRequest {
    /// Correlation ID for matching request with response
    pub correlation_id: String,
    /// Database path
    pub db_path: String,
    /// Orchestrator configuration
    pub orchestrator_config: Option<Value>,
}

/// Response to system initialization request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInitializationResponse {
    /// Correlation ID matching the request
    pub correlation_id: String,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Unified event enumeration that encompasses all event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    /// Field value set event
    FieldValueSet(FieldValueSet),
    /// Atom created event
    AtomCreated(AtomCreated),
    /// Atom updated event
    AtomUpdated(AtomUpdated),
    /// AtomRef created event
    AtomRefCreated(AtomRefCreated),
    /// AtomRef updated event
    AtomRefUpdated(AtomRefUpdated),
    /// Schema loaded event
    SchemaLoaded(SchemaLoaded),
    /// Transform executed event
    TransformExecuted(TransformExecuted),
    /// Schema changed event
    SchemaChanged(SchemaChanged),
    /// Transform triggered event
    TransformTriggered(TransformTriggered),
    /// Query executed event
    QueryExecuted(QueryExecuted),
    /// Mutation executed event
    MutationExecuted(MutationExecuted),
    
    // Request/Response Events
    /// Atom create request
    AtomCreateRequest(AtomCreateRequest),
    /// Atom create response
    AtomCreateResponse(AtomCreateResponse),
    /// Atom update request
    AtomUpdateRequest(AtomUpdateRequest),
    /// Atom update response
    AtomUpdateResponse(AtomUpdateResponse),
    /// AtomRef create request
    AtomRefCreateRequest(AtomRefCreateRequest),
    /// AtomRef create response
    AtomRefCreateResponse(AtomRefCreateResponse),
    /// AtomRef update request
    AtomRefUpdateRequest(AtomRefUpdateRequest),
    /// AtomRef update response
    AtomRefUpdateResponse(AtomRefUpdateResponse),
    /// Field value set request
    FieldValueSetRequest(FieldValueSetRequest),
    /// Field value set response
    FieldValueSetResponse(FieldValueSetResponse),
    /// Field update request
    FieldUpdateRequest(FieldUpdateRequest),
    /// Field update response
    FieldUpdateResponse(FieldUpdateResponse),
    /// Schema load request
    SchemaLoadRequest(SchemaLoadRequest),
    /// Schema load response
    SchemaLoadResponse(SchemaLoadResponse),
    /// Schema approval request
    SchemaApprovalRequest(SchemaApprovalRequest),
    /// Schema approval response
    SchemaApprovalResponse(SchemaApprovalResponse),
    /// Atom history request
    AtomHistoryRequest(AtomHistoryRequest),
    /// Atom history response
    AtomHistoryResponse(AtomHistoryResponse),
    /// Atom get request
    AtomGetRequest(AtomGetRequest),
    /// Atom get response
    AtomGetResponse(AtomGetResponse),
    /// Field value query request
    FieldValueQueryRequest(FieldValueQueryRequest),
    /// Field value query response
    FieldValueQueryResponse(FieldValueQueryResponse),
    /// AtomRef query request
    AtomRefQueryRequest(AtomRefQueryRequest),
    /// AtomRef query response
    AtomRefQueryResponse(AtomRefQueryResponse),
    /// Transform trigger request
    TransformTriggerRequest(TransformTriggerRequest),
    /// Transform trigger response
    TransformTriggerResponse(TransformTriggerResponse),
    /// Transform execution request
    TransformExecutionRequest(TransformExecutionRequest),
    /// Transform execution response
    TransformExecutionResponse(TransformExecutionResponse),
    /// System initialization request
    SystemInitializationRequest(SystemInitializationRequest),
    /// System initialization response
    SystemInitializationResponse(SystemInitializationResponse),
}

impl Event {
    /// Get the event type as a string identifier
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::FieldValueSet(_) => "FieldValueSet",
            Event::AtomCreated(_) => "AtomCreated",
            Event::AtomUpdated(_) => "AtomUpdated",
            Event::AtomRefCreated(_) => "AtomRefCreated",
            Event::AtomRefUpdated(_) => "AtomRefUpdated",
            Event::SchemaLoaded(_) => "SchemaLoaded",
            Event::TransformExecuted(_) => "TransformExecuted",
            Event::SchemaChanged(_) => "SchemaChanged",
            Event::TransformTriggered(_) => "TransformTriggered",
            Event::QueryExecuted(_) => "QueryExecuted",
            Event::MutationExecuted(_) => "MutationExecuted",
            Event::AtomCreateRequest(_) => "AtomCreateRequest",
            Event::AtomCreateResponse(_) => "AtomCreateResponse",
            Event::AtomUpdateRequest(_) => "AtomUpdateRequest",
            Event::AtomUpdateResponse(_) => "AtomUpdateResponse",
            Event::AtomRefCreateRequest(_) => "AtomRefCreateRequest",
            Event::AtomRefCreateResponse(_) => "AtomRefCreateResponse",
            Event::AtomRefUpdateRequest(_) => "AtomRefUpdateRequest",
            Event::AtomRefUpdateResponse(_) => "AtomRefUpdateResponse",
            Event::FieldValueSetRequest(_) => "FieldValueSetRequest",
            Event::FieldValueSetResponse(_) => "FieldValueSetResponse",
            Event::FieldUpdateRequest(_) => "FieldUpdateRequest",
            Event::FieldUpdateResponse(_) => "FieldUpdateResponse",
            Event::SchemaLoadRequest(_) => "SchemaLoadRequest",
            Event::SchemaLoadResponse(_) => "SchemaLoadResponse",
            Event::SchemaApprovalRequest(_) => "SchemaApprovalRequest",
            Event::SchemaApprovalResponse(_) => "SchemaApprovalResponse",
            Event::AtomHistoryRequest(_) => "AtomHistoryRequest",
            Event::AtomHistoryResponse(_) => "AtomHistoryResponse",
            Event::AtomGetRequest(_) => "AtomGetRequest",
            Event::AtomGetResponse(_) => "AtomGetResponse",
            Event::FieldValueQueryRequest(_) => "FieldValueQueryRequest",
            Event::FieldValueQueryResponse(_) => "FieldValueQueryResponse",
            Event::AtomRefQueryRequest(_) => "AtomRefQueryRequest",
            Event::AtomRefQueryResponse(_) => "AtomRefQueryResponse",
            Event::TransformTriggerRequest(_) => "TransformTriggerRequest",
            Event::TransformTriggerResponse(_) => "TransformTriggerResponse",
            Event::TransformExecutionRequest(_) => "TransformExecutionRequest",
            Event::TransformExecutionResponse(_) => "TransformExecutionResponse",
            Event::SystemInitializationRequest(_) => "SystemInitializationRequest",
            Event::SystemInitializationResponse(_) => "SystemInitializationResponse",
        }
    }
}

// ========== Event Type Traits ==========

/// Trait for types that can be used as events in the message bus
pub trait EventType: Clone + Send + 'static {
    /// Get the unique type identifier for this event type
    fn type_id() -> &'static str;
}

impl EventType for FieldValueSet {
    fn type_id() -> &'static str {
        "FieldValueSet"
    }
}

impl EventType for AtomCreated {
    fn type_id() -> &'static str {
        "AtomCreated"
    }
}

impl EventType for AtomUpdated {
    fn type_id() -> &'static str {
        "AtomUpdated"
    }
}

impl EventType for AtomRefCreated {
    fn type_id() -> &'static str {
        "AtomRefCreated"
    }
}

impl EventType for AtomRefUpdated {
    fn type_id() -> &'static str {
        "AtomRefUpdated"
    }
}

impl EventType for SchemaLoaded {
    fn type_id() -> &'static str {
        "SchemaLoaded"
    }
}

impl EventType for TransformExecuted {
    fn type_id() -> &'static str {
        "TransformExecuted"
    }
}

impl EventType for SchemaChanged {
    fn type_id() -> &'static str {
        "SchemaChanged"
    }
}

impl EventType for TransformTriggered {
    fn type_id() -> &'static str {
        "TransformTriggered"
    }
}

impl EventType for QueryExecuted {
    fn type_id() -> &'static str {
        "QueryExecuted"
    }
}

impl EventType for MutationExecuted {
    fn type_id() -> &'static str {
        "MutationExecuted"
    }
}

// Request/Response Event Type implementations

impl EventType for AtomCreateRequest {
    fn type_id() -> &'static str {
        "AtomCreateRequest"
    }
}

impl EventType for AtomCreateResponse {
    fn type_id() -> &'static str {
        "AtomCreateResponse"
    }
}

impl EventType for AtomUpdateRequest {
    fn type_id() -> &'static str {
        "AtomUpdateRequest"
    }
}

impl EventType for AtomUpdateResponse {
    fn type_id() -> &'static str {
        "AtomUpdateResponse"
    }
}

impl EventType for AtomRefCreateRequest {
    fn type_id() -> &'static str {
        "AtomRefCreateRequest"
    }
}

impl EventType for AtomRefCreateResponse {
    fn type_id() -> &'static str {
        "AtomRefCreateResponse"
    }
}

impl EventType for AtomRefUpdateRequest {
    fn type_id() -> &'static str {
        "AtomRefUpdateRequest"
    }
}

impl EventType for AtomRefUpdateResponse {
    fn type_id() -> &'static str {
        "AtomRefUpdateResponse"
    }
}

impl EventType for FieldValueSetRequest {
    fn type_id() -> &'static str {
        "FieldValueSetRequest"
    }
}

impl EventType for FieldValueSetResponse {
    fn type_id() -> &'static str {
        "FieldValueSetResponse"
    }
}

impl EventType for FieldUpdateRequest {
    fn type_id() -> &'static str {
        "FieldUpdateRequest"
    }
}

impl EventType for FieldUpdateResponse {
    fn type_id() -> &'static str {
        "FieldUpdateResponse"
    }
}

impl EventType for SchemaLoadRequest {
    fn type_id() -> &'static str {
        "SchemaLoadRequest"
    }
}

impl EventType for SchemaLoadResponse {
    fn type_id() -> &'static str {
        "SchemaLoadResponse"
    }
}

impl EventType for SchemaApprovalRequest {
    fn type_id() -> &'static str {
        "SchemaApprovalRequest"
    }
}

impl EventType for SchemaApprovalResponse {
    fn type_id() -> &'static str {
        "SchemaApprovalResponse"
    }
}

impl EventType for AtomHistoryRequest {
    fn type_id() -> &'static str {
        "AtomHistoryRequest"
    }
}

impl EventType for AtomHistoryResponse {
    fn type_id() -> &'static str {
        "AtomHistoryResponse"
    }
}

impl EventType for AtomGetRequest {
    fn type_id() -> &'static str {
        "AtomGetRequest"
    }
}

impl EventType for AtomGetResponse {
    fn type_id() -> &'static str {
        "AtomGetResponse"
    }
}

impl EventType for FieldValueQueryRequest {
    fn type_id() -> &'static str {
        "FieldValueQueryRequest"
    }
}

impl EventType for FieldValueQueryResponse {
    fn type_id() -> &'static str {
        "FieldValueQueryResponse"
    }
}

impl EventType for AtomRefQueryRequest {
    fn type_id() -> &'static str {
        "AtomRefQueryRequest"
    }
}

impl EventType for AtomRefQueryResponse {
    fn type_id() -> &'static str {
        "AtomRefQueryResponse"
    }
}

impl EventType for TransformTriggerRequest {
    fn type_id() -> &'static str {
        "TransformTriggerRequest"
    }
}

impl EventType for TransformTriggerResponse {
    fn type_id() -> &'static str {
        "TransformTriggerResponse"
    }
}


impl EventType for TransformExecutionRequest {
    fn type_id() -> &'static str {
        "TransformExecutionRequest"
    }
}

impl EventType for TransformExecutionResponse {
    fn type_id() -> &'static str {
        "TransformExecutionResponse"
    }
}

impl EventType for SystemInitializationRequest {
    fn type_id() -> &'static str {
        "SystemInitializationRequest"
    }
}

impl EventType for SystemInitializationResponse {
    fn type_id() -> &'static str {
        "SystemInitializationResponse"
    }
}

impl EventType for CollectionUpdateRequest {
    fn type_id() -> &'static str {
        "CollectionUpdateRequest"
    }
}

impl EventType for CollectionUpdateResponse {
    fn type_id() -> &'static str {
        "CollectionUpdateResponse"
    }
}

impl EventType for Event {
    fn type_id() -> &'static str {
        "Event"
    }
}

// ========== Message Bus Implementation ==========

/// Consumer handle for receiving events of a specific type
pub struct Consumer<T: EventType> {
    receiver: Receiver<T>,
}

impl<T: EventType> Consumer<T> {
    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<T, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    /// Receive an event, blocking until one is available
    pub fn recv(&mut self) -> Result<T, mpsc::RecvError> {
        self.receiver.recv()
    }

    /// Get an iterator over received events
    pub fn iter(&mut self) -> mpsc::Iter<T> {
        self.receiver.iter()
    }

    /// Try to receive an event with a timeout
    pub fn recv_timeout(&mut self, timeout: std::time::Duration) -> Result<T, mpsc::RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}

/// Internal registry for managing event subscribers
struct SubscriberRegistry {
    // Using type erasure to store different channel senders
    // Key: event type name, Value: list of boxed senders
    subscribers: HashMap<String, Vec<Box<dyn std::any::Any + Send>>>,
}

impl SubscriberRegistry {
    fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    fn add_subscriber<T: EventType>(&mut self, sender: Sender<T>) {
        let type_id = T::type_id();
        let boxed_sender = Box::new(sender);
        
        self.subscribers
            .entry(type_id.to_string())
            .or_default()
            .push(boxed_sender);
    }

    fn get_subscribers<T: EventType>(&self) -> Vec<&Sender<T>> {
        let type_id = T::type_id();
        self.subscribers
            .get(type_id)
            .map(|senders| {
                senders
                    .iter()
                    .filter_map(|boxed| boxed.downcast_ref::<Sender<T>>())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Main message bus for event-driven communication
pub struct MessageBus {
    registry: Arc<Mutex<SubscriberRegistry>>,
}

impl MessageBus {
    /// Create a new message bus instance
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(SubscriberRegistry::new())),
        }
    }

    /// Subscribe to events of a specific type
    /// Returns a Consumer that can be used to receive events
    pub fn subscribe<T: EventType>(&self) -> Consumer<T> {
        let (sender, receiver) = mpsc::channel();
        
        let mut registry = self.registry.lock().unwrap();
        registry.add_subscriber(sender);
        
        Consumer { receiver }
    }

    /// Publish an event to all subscribers of that event type
    pub fn publish<T: EventType>(&self, event: T) -> MessageBusResult<()> {
        let registry = self.registry.lock().unwrap();
        let subscribers = registry.get_subscribers::<T>();
        
        if subscribers.is_empty() {
            // No subscribers for this event type - this is not an error
            return Ok(());
        }

        let mut failed_sends = 0;
        let total_subscribers = subscribers.len();

        for subscriber in subscribers {
            if subscriber.send(event.clone()).is_err() {
                failed_sends += 1;
            }
        }

        if failed_sends > 0 {
            return Err(MessageBusError::SendFailed {
                reason: format!("{} of {} subscribers failed to receive event", failed_sends, total_subscribers),
            });
        }

        Ok(())
    }

    /// Convenience method to publish a unified Event
    pub fn publish_event(&self, event: Event) -> MessageBusResult<()> {
        match event {
            Event::FieldValueSet(e) => self.publish(e),
            Event::AtomCreated(e) => self.publish(e),
            Event::AtomUpdated(e) => self.publish(e),
            Event::AtomRefCreated(e) => self.publish(e),
            Event::AtomRefUpdated(e) => self.publish(e),
            Event::SchemaLoaded(e) => self.publish(e),
            Event::TransformExecuted(e) => self.publish(e),
            Event::SchemaChanged(e) => self.publish(e),
            Event::TransformTriggered(e) => self.publish(e),
            Event::QueryExecuted(e) => self.publish(e),
            Event::MutationExecuted(e) => self.publish(e),
            Event::AtomCreateRequest(e) => self.publish(e),
            Event::AtomCreateResponse(e) => self.publish(e),
            Event::AtomUpdateRequest(e) => self.publish(e),
            Event::AtomUpdateResponse(e) => self.publish(e),
            Event::AtomRefCreateRequest(e) => self.publish(e),
            Event::AtomRefCreateResponse(e) => self.publish(e),
            Event::AtomRefUpdateRequest(e) => self.publish(e),
            Event::AtomRefUpdateResponse(e) => self.publish(e),
            Event::FieldValueSetRequest(e) => self.publish(e),
            Event::FieldValueSetResponse(e) => self.publish(e),
            Event::FieldUpdateRequest(e) => self.publish(e),
            Event::FieldUpdateResponse(e) => self.publish(e),
            Event::SchemaLoadRequest(e) => self.publish(e),
            Event::SchemaLoadResponse(e) => self.publish(e),
            Event::SchemaApprovalRequest(e) => self.publish(e),
            Event::SchemaApprovalResponse(e) => self.publish(e),
            Event::AtomHistoryRequest(e) => self.publish(e),
            Event::AtomHistoryResponse(e) => self.publish(e),
            Event::AtomGetRequest(e) => self.publish(e),
            Event::AtomGetResponse(e) => self.publish(e),
            Event::FieldValueQueryRequest(e) => self.publish(e),
            Event::FieldValueQueryResponse(e) => self.publish(e),
            Event::AtomRefQueryRequest(e) => self.publish(e),
            Event::AtomRefQueryResponse(e) => self.publish(e),
            Event::TransformTriggerRequest(e) => self.publish(e),
            Event::TransformTriggerResponse(e) => self.publish(e),
            Event::TransformExecutionRequest(e) => self.publish(e),
            Event::TransformExecutionResponse(e) => self.publish(e),
            Event::SystemInitializationRequest(e) => self.publish(e),
            Event::SystemInitializationResponse(e) => self.publish(e),
        }
    }

    /// Get the number of subscribers for a given event type
    pub fn subscriber_count<T: EventType>(&self) -> usize {
        let registry = self.registry.lock().unwrap();
        registry.get_subscribers::<T>().len()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Async Support ==========

/// Async consumer for event handling in async contexts
pub struct AsyncConsumer<T> {
    receiver: async_mpsc::UnboundedReceiver<T>,
}

impl AsyncConsumer<Event> {
    /// Create a new async consumer
    fn new(receiver: async_mpsc::UnboundedReceiver<Event>) -> Self {
        Self { receiver }
    }

    /// Async receive without blocking
    pub async fn recv(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }

    /// Async receive with timeout
    pub async fn recv_timeout(&mut self, duration: AsyncDuration) -> Result<Event, AsyncRecvError> {
        match timeout(duration, self.receiver.recv()).await {
            Ok(Some(event)) => Ok(event),
            Ok(None) => Err(AsyncRecvError::Disconnected),
            Err(_) => Err(AsyncRecvError::Timeout),
        }
    }

    /// Try to receive an event without waiting
    pub fn try_recv(&mut self) -> Result<Event, AsyncTryRecvError> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(event),
            Err(async_mpsc::error::TryRecvError::Empty) => Err(AsyncTryRecvError::Empty),
            Err(async_mpsc::error::TryRecvError::Disconnected) => Err(AsyncTryRecvError::Disconnected),
        }
    }

    /// Filter events to specific type
    pub async fn recv_filtered<T: EventType>(&mut self) -> Option<T> {
        while let Some(event) = self.recv().await {
            if let Some(typed_event) = self.extract_typed_event::<T>(event) {
                return Some(typed_event);
            }
        }
        None
    }

    /// Extract typed event from unified Event enum
    fn extract_typed_event<T: EventType>(&self, _event: Event) -> Option<T> {
        // This is a helper method to extract specific event types from the unified Event
        // Implementation depends on how we want to handle this conversion
        // For now, return None as this is a complex type conversion
        None
    }
}

/// Errors for async message reception
#[derive(Error, Debug, Clone)]
pub enum AsyncRecvError {
    #[error("Timeout while waiting for message")]
    Timeout,
    #[error("Channel disconnected")]
    Disconnected,
}

/// Errors for async try_recv
#[derive(Error, Debug, Clone)]
pub enum AsyncTryRecvError {
    #[error("No message available")]
    Empty,
    #[error("Channel disconnected")]
    Disconnected,
}

/// Trait for async event handlers
pub trait AsyncEventHandler<T: EventType>: Send + Sync {
    fn handle(&self, event: T) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;
}

/// Async subscriber registry for managing async event subscribers
struct AsyncSubscriberRegistry {
    // Use unified Event type for simplicity and type safety
    event_subscribers: HashMap<String, Vec<async_mpsc::UnboundedSender<Event>>>,
}

impl AsyncSubscriberRegistry {
    fn new() -> Self {
        Self {
            event_subscribers: HashMap::new(),
        }
    }

    fn add_subscriber(&mut self, event_type: String, sender: async_mpsc::UnboundedSender<Event>) {
        self.event_subscribers
            .entry(event_type)
            .or_default()
            .push(sender);
    }

    fn get_subscribers(&self, event_type: &str) -> Vec<&async_mpsc::UnboundedSender<Event>> {
        self.event_subscribers
            .get(event_type)
            .map(|senders| senders.iter().collect())
            .unwrap_or_default()
    }
}

/// Async message bus for event-driven communication
pub struct AsyncMessageBus {
    registry: Arc<tokio::sync::Mutex<AsyncSubscriberRegistry>>,
}

impl AsyncMessageBus {
    /// Create a new async message bus instance
    pub fn new() -> Self {
        Self {
            registry: Arc::new(tokio::sync::Mutex::new(AsyncSubscriberRegistry::new())),
        }
    }

    /// Subscribe to events of a specific type through unified Event enum
    pub async fn subscribe(&self, event_type: &str) -> AsyncConsumer<Event> {
        let (sender, receiver) = async_mpsc::unbounded_channel();
        
        let mut registry = self.registry.lock().await;
        registry.add_subscriber(event_type.to_string(), sender);
        
        AsyncConsumer::new(receiver)
    }

    /// Subscribe to all events
    pub async fn subscribe_all(&self) -> AsyncConsumer<Event> {
        let (sender, receiver) = async_mpsc::unbounded_channel();
        
        let mut registry = self.registry.lock().await;
        // Subscribe to all event types
        let event_types = [
            "FieldValueSet", "AtomCreated", "AtomUpdated", "AtomRefCreated",
            "AtomRefUpdated", "SchemaLoaded", "TransformExecuted", "SchemaChanged",
            "TransformTriggered", "QueryExecuted", "MutationExecuted"
        ];
        
        for event_type in &event_types {
            registry.add_subscriber(event_type.to_string(), sender.clone());
        }
        
        AsyncConsumer::new(receiver)
    }

    /// Publish an event (convenience method for individual event types)
    pub async fn publish_field_value_set(&self, event: FieldValueSet) -> MessageBusResult<()> {
        self.publish_event(Event::FieldValueSet(event)).await
    }

    pub async fn publish_atom_created(&self, event: AtomCreated) -> MessageBusResult<()> {
        self.publish_event(Event::AtomCreated(event)).await
    }

    pub async fn publish_query_executed(&self, event: QueryExecuted) -> MessageBusResult<()> {
        self.publish_event(Event::QueryExecuted(event)).await
    }

    pub async fn publish_mutation_executed(&self, event: MutationExecuted) -> MessageBusResult<()> {
        self.publish_event(Event::MutationExecuted(event)).await
    }

    /// Convenience method to publish a unified Event
    pub async fn publish_event(&self, event: Event) -> MessageBusResult<()> {
        let registry = self.registry.lock().await;
        let event_type = event.event_type();
        let subscribers = registry.get_subscribers(event_type);
        
        if subscribers.is_empty() {
            return Ok(());
        }

        let mut failed_sends = 0;
        let total_subscribers = subscribers.len();

        for subscriber in subscribers {
            if subscriber.send(event.clone()).is_err() {
                failed_sends += 1;
            }
        }

        if failed_sends > 0 {
            return Err(MessageBusError::SendFailed {
                reason: format!("{} of {} async subscribers failed to receive event", failed_sends, total_subscribers),
            });
        }

        Ok(())
    }

    /// Get the number of subscribers for a given event type
    pub async fn subscriber_count(&self, event_type: &str) -> usize {
        let registry = self.registry.lock().await;
        registry.get_subscribers(event_type).len()
    }
}

impl Default for AsyncMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Error Recovery and Event Sourcing ==========

/// Event with retry metadata for error recovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryableEvent {
    /// The original event
    pub event: Event,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Timestamp of original event
    pub timestamp: std::time::SystemTime,
    /// Error from last failure (if any)
    pub last_error: Option<String>,
}

impl RetryableEvent {
    /// Create a new retryable event
    pub fn new(event: Event, max_retries: u32) -> Self {
        Self {
            event,
            retry_count: 0,
            max_retries,
            timestamp: std::time::SystemTime::now(),
            last_error: None,
        }
    }

    /// Check if event can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count and set error
    pub fn increment_retry(&mut self, error: String) {
        self.retry_count += 1;
        self.last_error = Some(error);
    }

    /// Check if event should go to dead letter queue
    pub fn is_dead_letter(&self) -> bool {
        self.retry_count >= self.max_retries
    }
}

/// Dead letter event for events that couldn't be processed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeadLetterEvent {
    /// The original retryable event
    pub retryable_event: RetryableEvent,
    /// Timestamp when moved to dead letter queue
    pub dead_letter_timestamp: std::time::SystemTime,
    /// Reason for dead lettering
    pub reason: String,
}

impl DeadLetterEvent {
    /// Create a new dead letter event
    pub fn new(retryable_event: RetryableEvent, reason: String) -> Self {
        Self {
            retryable_event,
            dead_letter_timestamp: std::time::SystemTime::now(),
            reason,
        }
    }
}

/// Event history entry for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventHistoryEntry {
    /// Unique event ID
    pub event_id: String,
    /// The event data
    pub event: Event,
    /// Timestamp when event occurred
    pub timestamp: std::time::SystemTime,
    /// Source component that generated the event
    pub source: String,
    /// Event sequence number (for ordering)
    pub sequence_number: u64,
}

impl EventHistoryEntry {
    /// Create a new event history entry
    pub fn new(event: Event, source: String, sequence_number: u64) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event,
            timestamp: std::time::SystemTime::now(),
            source,
            sequence_number,
        }
    }
}

/// Enhanced message bus with error recovery and event sourcing
pub struct EnhancedMessageBus {
    /// Sync message bus
    sync_bus: Arc<MessageBus>,
    /// Async message bus
    async_bus: Arc<AsyncMessageBus>,
    /// Event history for event sourcing
    event_history: Arc<tokio::sync::Mutex<Vec<EventHistoryEntry>>>,
    /// Dead letter queue
    dead_letter_queue: Arc<tokio::sync::Mutex<Vec<DeadLetterEvent>>>,
    /// Retry queue
    retry_queue: Arc<tokio::sync::Mutex<Vec<RetryableEvent>>>,
    /// Event sequence counter
    sequence_counter: Arc<tokio::sync::Mutex<u64>>,
}

impl EnhancedMessageBus {
    /// Create a new enhanced message bus
    pub fn new() -> Self {
        Self {
            sync_bus: Arc::new(MessageBus::new()),
            async_bus: Arc::new(AsyncMessageBus::new()),
            event_history: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            dead_letter_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            retry_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            sequence_counter: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Get access to sync message bus
    pub fn sync_bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.sync_bus)
    }

    /// Get access to async message bus
    pub fn async_bus(&self) -> Arc<AsyncMessageBus> {
        Arc::clone(&self.async_bus)
    }

    /// Publish event with error recovery
    pub async fn publish_with_retry<T: EventType>(&self, event: T, max_retries: u32, source: String) -> MessageBusResult<()> {
        let unified_event = self.to_unified_event(event);
        let retryable_event = RetryableEvent::new(unified_event.clone(), max_retries);
        
        // Record in event history
        self.record_event_history(unified_event, source).await;
        
        // Try to publish
        match self.async_bus.publish_event(retryable_event.event.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Add to retry queue
                let mut retry_queue = self.retry_queue.lock().await;
                let mut retry_event = retryable_event;
                retry_event.increment_retry(e.to_string());
                retry_queue.push(retry_event);
                Err(e)
            }
        }
    }

    /// Process retry queue
    pub async fn process_retries(&self) -> usize {
        let mut retry_queue = self.retry_queue.lock().await;
        let mut processed = 0;
        let mut dead_letters = Vec::new();
        let mut successful_retries = Vec::new();

        for (index, retryable_event) in retry_queue.iter_mut().enumerate() {
            if !retryable_event.can_retry() {
                dead_letters.push((index, retryable_event.clone()));
                continue;
            }

            match self.async_bus.publish_event(retryable_event.event.clone()).await {
                Ok(_) => {
                    successful_retries.push(index);
                    processed += 1;
                }
                Err(e) => {
                    retryable_event.increment_retry(e.to_string());
                    if retryable_event.is_dead_letter() {
                        dead_letters.push((index, retryable_event.clone()));
                    }
                }
            }
        }

        // Move dead letters to dead letter queue
        if !dead_letters.is_empty() {
            let mut dlq = self.dead_letter_queue.lock().await;
            for (_, retryable_event) in &dead_letters {
                let dead_letter = DeadLetterEvent::new(
                    retryable_event.clone(),
                    "Max retries exceeded".to_string(),
                );
                dlq.push(dead_letter);
            }
        }

        // Remove processed events from retry queue
        let mut indices_to_remove: Vec<usize> = dead_letters.iter().map(|(i, _)| *i).collect();
        indices_to_remove.extend(successful_retries);
        indices_to_remove.sort_by(|a, b| b.cmp(a)); // Sort in reverse order

        for index in indices_to_remove {
            retry_queue.remove(index);
        }

        processed
    }

    /// Get event history for event sourcing
    pub async fn get_event_history(&self) -> Vec<EventHistoryEntry> {
        let history = self.event_history.lock().await;
        history.clone()
    }

    /// Get event history since a specific sequence number
    pub async fn get_event_history_since(&self, sequence_number: u64) -> Vec<EventHistoryEntry> {
        let history = self.event_history.lock().await;
        history
            .iter()
            .filter(|entry| entry.sequence_number > sequence_number)
            .cloned()
            .collect()
    }

    /// Replay events from history to reconstruct state
    pub async fn replay_events(&self, from_sequence: u64) -> MessageBusResult<usize> {
        let events = self.get_event_history_since(from_sequence).await;
        let mut replayed = 0;

        for entry in events {
            // Replay event through async bus
            self.async_bus.publish_event(entry.event).await?;
            replayed += 1;
        }

        Ok(replayed)
    }

    /// Get dead letter queue contents
    pub async fn get_dead_letters(&self) -> Vec<DeadLetterEvent> {
        let dlq = self.dead_letter_queue.lock().await;
        dlq.clone()
    }

    /// Clear dead letter queue
    pub async fn clear_dead_letters(&self) -> usize {
        let mut dlq = self.dead_letter_queue.lock().await;
        let count = dlq.len();
        dlq.clear();
        count
    }

    /// Get retry queue status
    pub async fn get_retry_queue_status(&self) -> (usize, usize) {
        let retry_queue = self.retry_queue.lock().await;
        let total = retry_queue.len();
        let ready_for_retry = retry_queue.iter().filter(|e| e.can_retry()).count();
        (total, ready_for_retry)
    }

    /// Record event in history
    async fn record_event_history(&self, event: Event, source: String) {
        let mut sequence_counter = self.sequence_counter.lock().await;
        *sequence_counter += 1;
        let sequence_number = *sequence_counter;
        drop(sequence_counter);

        let entry = EventHistoryEntry::new(event, source, sequence_number);
        
        let mut history = self.event_history.lock().await;
        history.push(entry);
    }

    /// Convert typed event to unified event (helper method)
    fn to_unified_event<T: EventType>(&self, _event: T) -> Event {
        // This is a simplified conversion - in practice you'd need proper trait bounds
        // or a more sophisticated event conversion system
        match std::any::type_name::<T>() {
            "fold_node::fold_db_core::message_bus::FieldValueSet" => {
                // For demo purposes - proper implementation would use proper casting
                Event::FieldValueSet(FieldValueSet::new("unknown", serde_json::Value::Null, "unknown"))
            }
            _ => Event::FieldValueSet(FieldValueSet::new("unknown", serde_json::Value::Null, "unknown"))
        }
    }
}

impl Default for EnhancedMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Convenience Constructors ==========

impl FieldValueSet {
    /// Create a new FieldValueSet event
    pub fn new(field: impl Into<String>, value: Value, source: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            value,
            source: source.into(),
        }
    }
}

impl AtomCreated {
    /// Create a new AtomCreated event
    pub fn new(atom_id: impl Into<String>, data: Value) -> Self {
        Self {
            atom_id: atom_id.into(),
            data,
        }
    }
}

impl AtomUpdated {
    /// Create a new AtomUpdated event
    pub fn new(atom_id: impl Into<String>, data: Value) -> Self {
        Self {
            atom_id: atom_id.into(),
            data,
        }
    }
}

impl AtomRefCreated {
    /// Create a new AtomRefCreated event
    pub fn new(aref_uuid: impl Into<String>, aref_type: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self {
            aref_uuid: aref_uuid.into(),
            aref_type: aref_type.into(),
            field_path: field_path.into(),
        }
    }
}

impl AtomRefUpdated {
    /// Create a new AtomRefUpdated event
    pub fn new(aref_uuid: impl Into<String>, field_path: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            aref_uuid: aref_uuid.into(),
            field_path: field_path.into(),
            operation: operation.into(),
        }
    }
}

impl SchemaLoaded {
    /// Create a new SchemaLoaded event
    pub fn new(schema_name: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            schema_name: schema_name.into(),
            status: status.into(),
        }
    }
}

impl TransformExecuted {
    /// Create a new TransformExecuted event
    pub fn new(transform_id: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            transform_id: transform_id.into(),
            result: result.into(),
        }
    }
}

impl SchemaChanged {
    /// Create a new SchemaChanged event
    pub fn new(schema: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
        }
    }
}

impl TransformTriggered {
    /// Create a new TransformTriggered event
    pub fn new(transform_id: impl Into<String>) -> Self {
        Self {
            transform_id: transform_id.into(),
        }
    }
}

impl QueryExecuted {
    /// Create a new QueryExecuted event
    pub fn new(
        query_type: impl Into<String>,
        schema: impl Into<String>,
        execution_time_ms: u64,
        result_count: usize,
    ) -> Self {
        Self {
            query_type: query_type.into(),
            schema: schema.into(),
            execution_time_ms,
            result_count,
        }
    }
}

impl MutationExecuted {
    /// Create a new MutationExecuted event
    pub fn new(
        operation: impl Into<String>,
        schema: impl Into<String>,
        execution_time_ms: u64,
        fields_affected: usize,
    ) -> Self {
        Self {
            operation: operation.into(),
            schema: schema.into(),
            execution_time_ms,
            fields_affected,
        }
    }
}

// ========== Request/Response Event Constructors ==========

impl AtomCreateRequest {
    /// Create a new AtomCreateRequest
    pub fn new(
        correlation_id: String,
        schema_name: String,
        source_pub_key: String,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            schema_name,
            source_pub_key,
            prev_atom_uuid,
            content,
            status,
        }
    }
}

impl AtomCreateResponse {
    /// Create a new AtomCreateResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        atom_uuid: Option<String>,
        error: Option<String>,
        atom_data: Option<Value>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            atom_uuid,
            error,
            atom_data,
        }
    }
}

impl AtomUpdateRequest {
    /// Create a new AtomUpdateRequest
    pub fn new(
        correlation_id: String,
        atom_uuid: String,
        content: Value,
        source_pub_key: String,
    ) -> Self {
        Self {
            correlation_id,
            atom_uuid,
            content,
            source_pub_key,
        }
    }
}

impl AtomUpdateResponse {
    /// Create a new AtomUpdateResponse
    pub fn new(correlation_id: String, success: bool, error: Option<String>) -> Self {
        Self {
            correlation_id,
            success,
            error,
        }
    }
}

impl AtomRefCreateRequest {
    /// Create a new AtomRefCreateRequest
    pub fn new(
        correlation_id: String,
        aref_uuid: String,
        atom_uuid: String,
        source_pub_key: String,
        aref_type: String,
    ) -> Self {
        Self {
            correlation_id,
            aref_uuid,
            atom_uuid,
            source_pub_key,
            aref_type,
        }
    }
}

impl AtomRefCreateResponse {
    /// Create a new AtomRefCreateResponse
    pub fn new(correlation_id: String, success: bool, error: Option<String>) -> Self {
        Self {
            correlation_id,
            success,
            error,
        }
    }
}

impl AtomRefUpdateRequest {
    /// Create a new AtomRefUpdateRequest
    pub fn new(
        correlation_id: String,
        aref_uuid: String,
        atom_uuid: String,
        source_pub_key: String,
        aref_type: String,
        additional_data: Option<Value>,
    ) -> Self {
        Self {
            correlation_id,
            aref_uuid,
            atom_uuid,
            source_pub_key,
            aref_type,
            additional_data,
        }
    }
}

impl AtomRefUpdateResponse {
    /// Create a new AtomRefUpdateResponse
    pub fn new(correlation_id: String, success: bool, error: Option<String>) -> Self {
        Self {
            correlation_id,
            success,
            error,
        }
    }
}

impl FieldValueSetRequest {
    /// Create a new FieldValueSetRequest
    pub fn new(
        correlation_id: String,
        schema_name: String,
        field_name: String,
        value: Value,
        source_pub_key: String,
    ) -> Self {
        Self {
            correlation_id,
            schema_name,
            field_name,
            value,
            source_pub_key,
        }
    }
}

impl FieldValueSetResponse {
    /// Create a new FieldValueSetResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        aref_uuid: Option<String>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            aref_uuid,
            error,
        }
    }
}

impl FieldUpdateRequest {
    /// Create a new FieldUpdateRequest
    pub fn new(
        correlation_id: String,
        schema_name: String,
        field_name: String,
        value: Value,
        source_pub_key: String,
    ) -> Self {
        Self {
            correlation_id,
            schema_name,
            field_name,
            value,
            source_pub_key,
        }
    }
}

impl FieldUpdateResponse {
    /// Create a new FieldUpdateResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        aref_uuid: Option<String>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            aref_uuid,
            error,
        }
    }
}

impl SchemaLoadRequest {
    /// Create a new SchemaLoadRequest
    pub fn new(correlation_id: String, schema_name: String) -> Self {
        Self {
            correlation_id,
            schema_name,
        }
    }
}

impl SchemaLoadResponse {
    /// Create a new SchemaLoadResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        schema_data: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            schema_data,
            error,
        }
    }
}

impl SchemaApprovalRequest {
    /// Create a new SchemaApprovalRequest
    pub fn new(correlation_id: String, schema_name: String) -> Self {
        Self {
            correlation_id,
            schema_name,
        }
    }
}

impl SchemaApprovalResponse {
    /// Create a new SchemaApprovalResponse
    pub fn new(correlation_id: String, success: bool, error: Option<String>) -> Self {
        Self {
            correlation_id,
            success,
            error,
        }
    }
}

impl AtomHistoryRequest {
    /// Create a new AtomHistoryRequest
    pub fn new(correlation_id: String, aref_uuid: String) -> Self {
        Self {
            correlation_id,
            aref_uuid,
        }
    }
}

impl AtomHistoryResponse {
    /// Create a new AtomHistoryResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        history: Option<Vec<Value>>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            history,
            error,
        }
    }
}

impl AtomGetRequest {
    /// Create a new AtomGetRequest
    pub fn new(correlation_id: String, aref_uuid: String) -> Self {
        Self {
            correlation_id,
            aref_uuid,
        }
    }
}

impl AtomGetResponse {
    /// Create a new AtomGetResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        atom_data: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            atom_data,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn test_field_value_set_event() {
        let event = FieldValueSet::new("user.name", json!("Alice"), "test_source");
        assert_eq!(event.field, "user.name");
        assert_eq!(event.value, json!("Alice"));
        assert_eq!(event.source, "test_source");
    }

    #[test]
    fn test_atom_created_event() {
        let event = AtomCreated::new("atom-123", json!({"key": "value"}));
        assert_eq!(event.atom_id, "atom-123");
        assert_eq!(event.data, json!({"key": "value"}));
    }

    #[test]
    fn test_message_bus_basic_pubsub() {
        let bus = MessageBus::new();
        let mut consumer = bus.subscribe::<FieldValueSet>();

        // Verify no events initially
        assert!(consumer.try_recv().is_err());

        // Publish an event
        let event = FieldValueSet::new("test.field", json!("test_value"), "test");
        bus.publish(event.clone()).unwrap();

        // Consumer should receive the event
        let received = consumer.try_recv().unwrap();
        assert_eq!(received, event);
    }

    #[test]
    fn test_multiple_consumers_same_event_type() {
        let bus = MessageBus::new();
        let mut consumer1 = bus.subscribe::<AtomCreated>();
        let mut consumer2 = bus.subscribe::<AtomCreated>();

        assert_eq!(bus.subscriber_count::<AtomCreated>(), 2);

        let event = AtomCreated::new("atom-456", json!({"data": "test"}));
        bus.publish(event.clone()).unwrap();

        // Both consumers should receive the event
        assert_eq!(consumer1.try_recv().unwrap(), event);
        assert_eq!(consumer2.try_recv().unwrap(), event);
    }

    #[test]
    fn test_different_event_types() {
        let bus = MessageBus::new();
        let mut field_consumer = bus.subscribe::<FieldValueSet>();
        let mut atom_consumer = bus.subscribe::<AtomCreated>();

        // Publish different event types
        let field_event = FieldValueSet::new("test.field", json!("value"), "source");
        let atom_event = AtomCreated::new("atom-789", json!({}));

        bus.publish(field_event.clone()).unwrap();
        bus.publish(atom_event.clone()).unwrap();

        // Each consumer should only receive their event type
        assert_eq!(field_consumer.try_recv().unwrap(), field_event);
        assert!(field_consumer.try_recv().is_err()); // No more events

        assert_eq!(atom_consumer.try_recv().unwrap(), atom_event);
        assert!(atom_consumer.try_recv().is_err()); // No more events
    }

    #[test]
    fn test_publish_event_unified() {
        let bus = MessageBus::new();
        let mut consumer = bus.subscribe::<SchemaChanged>();

        let schema_event = SchemaChanged::new("test_schema");
        let unified_event = Event::SchemaChanged(schema_event.clone());

        bus.publish_event(unified_event).unwrap();

        assert_eq!(consumer.try_recv().unwrap(), schema_event);
    }

    #[test]
    fn test_no_subscribers() {
        let bus = MessageBus::new();
        
        // Publishing to no subscribers should not fail
        let event = TransformTriggered::new("transform-123");
        bus.publish(event).unwrap();
    }

    #[test]
    fn test_event_type_ids() {
        assert_eq!(FieldValueSet::type_id(), "FieldValueSet");
        assert_eq!(AtomCreated::type_id(), "AtomCreated");
        assert_eq!(AtomUpdated::type_id(), "AtomUpdated");
        assert_eq!(SchemaChanged::type_id(), "SchemaChanged");
        assert_eq!(TransformTriggered::type_id(), "TransformTriggered");
        assert_eq!(QueryExecuted::type_id(), "QueryExecuted");
        assert_eq!(MutationExecuted::type_id(), "MutationExecuted");
    }

    #[test]
    fn test_event_serialization() {
        let event = FieldValueSet::new("test", json!("value"), "source");
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: FieldValueSet = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_consumer_recv_timeout() {
        let bus = MessageBus::new();
        let mut consumer = bus.subscribe::<AtomUpdated>();

        // Should timeout since no events are published
        let result = consumer.recv_timeout(Duration::from_millis(10));
        assert!(matches!(result, Err(mpsc::RecvTimeoutError::Timeout)));
    }

    #[test]
    fn test_unified_event_types() {
        let field_event = Event::FieldValueSet(FieldValueSet::new("field", json!("value"), "source"));
        let atom_event = Event::AtomCreated(AtomCreated::new("atom-id", json!({})));
        let query_event = Event::QueryExecuted(QueryExecuted::new("range_query", "test_schema", 150, 5));
        let mutation_event = Event::MutationExecuted(MutationExecuted::new("create", "test_schema", 75, 3));
        
        assert_eq!(field_event.event_type(), "FieldValueSet");
        assert_eq!(atom_event.event_type(), "AtomCreated");
        assert_eq!(query_event.event_type(), "QueryExecuted");
        assert_eq!(mutation_event.event_type(), "MutationExecuted");
    }

    #[test]
    fn test_query_executed_event() {
        let bus = MessageBus::new();
        let mut consumer = bus.subscribe::<QueryExecuted>();

        let event = QueryExecuted::new("range_query", "Product", 250, 10);
        bus.publish(event.clone()).unwrap();

        assert_eq!(consumer.try_recv().unwrap(), event);
        assert_eq!(event.query_type, "range_query");
        assert_eq!(event.schema, "Product");
        assert_eq!(event.execution_time_ms, 250);
        assert_eq!(event.result_count, 10);
    }

    #[test]
    fn test_mutation_executed_event() {
        let bus = MessageBus::new();
        let mut consumer = bus.subscribe::<MutationExecuted>();

        let event = MutationExecuted::new("update", "User", 125, 2);
        bus.publish(event.clone()).unwrap();

        assert_eq!(consumer.try_recv().unwrap(), event);
        assert_eq!(event.operation, "update");
        assert_eq!(event.schema, "User");
        assert_eq!(event.execution_time_ms, 125);
        assert_eq!(event.fields_affected, 2);
    }

    #[test]
    fn test_query_mutation_event_serialization() {
        let query_event = QueryExecuted::new("single_query", "Analytics", 75, 1);
        let serialized = serde_json::to_string(&query_event).unwrap();
        let deserialized: QueryExecuted = serde_json::from_str(&serialized).unwrap();
        assert_eq!(query_event, deserialized);

        let mutation_event = MutationExecuted::new("create", "Inventory", 100, 4);
        let serialized = serde_json::to_string(&mutation_event).unwrap();
        let deserialized: MutationExecuted = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mutation_event, deserialized);
    }
}