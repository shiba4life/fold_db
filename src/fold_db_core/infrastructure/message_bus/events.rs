//! Event type definitions for the message bus system
//!
//! This module contains all event types that can be published and subscribed to
//! through the message bus, including both notification events and request/response events.

use serde::{Deserialize, Serialize};
use serde_json::Value;

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

/// Request to register a transform with the TransformManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRegistrationRequest {
    /// The transform registration details
    pub registration: crate::schema::types::TransformRegistration,
    /// Correlation ID for tracking the request
    pub correlation_id: String,
}

/// Response to transform registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRegistrationResponse {
    /// Correlation ID for tracking the request
    pub correlation_id: String,
    /// Whether the registration was successful
    pub success: bool,
    /// Error message if registration failed
    pub error: Option<String>,
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
            Event::SystemInitializationRequest(_) => "SystemInitializationRequest",
            Event::SystemInitializationResponse(_) => "SystemInitializationResponse",
        }
    }
}

/// Trait for types that can be used as events in the message bus
pub trait EventType: Clone + Send + 'static {
    /// Get the unique type identifier for this event type
    fn type_id() -> &'static str;
}

// ========== EventType Implementations ==========

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

impl EventType for TransformRegistrationRequest {
    fn type_id() -> &'static str {
        "TransformRegistrationRequest"
    }
}

impl EventType for TransformRegistrationResponse {
    fn type_id() -> &'static str {
        "TransformRegistrationResponse"
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


impl EventType for Event {
    fn type_id() -> &'static str {
        "Event"
    }
}