use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::EventType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomCreateRequest {
    pub correlation_id: String,
    pub schema_name: String,
    pub source_pub_key: String,
    pub prev_atom_uuid: Option<String>,
    pub content: Value,
    pub status: Option<String>,
}

impl EventType for AtomCreateRequest {
    fn type_id() -> &'static str { "AtomCreateRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomCreateResponse {
    pub correlation_id: String,
    pub success: bool,
    pub atom_uuid: Option<String>,
    pub error: Option<String>,
    pub atom_data: Option<Value>,
}

impl EventType for AtomCreateResponse {
    fn type_id() -> &'static str { "AtomCreateResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomUpdateRequest {
    pub correlation_id: String,
    pub atom_uuid: String,
    pub content: Value,
    pub source_pub_key: String,
}

impl EventType for AtomUpdateRequest {
    fn type_id() -> &'static str { "AtomUpdateRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomUpdateResponse {
    pub correlation_id: String,
    pub success: bool,
    pub error: Option<String>,
}

impl EventType for AtomUpdateResponse {
    fn type_id() -> &'static str { "AtomUpdateResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefCreateRequest {
    pub correlation_id: String,
    pub aref_uuid: String,
    pub atom_uuid: String,
    pub source_pub_key: String,
    pub aref_type: String,
}

impl EventType for AtomRefCreateRequest {
    fn type_id() -> &'static str { "AtomRefCreateRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefCreateResponse {
    pub correlation_id: String,
    pub success: bool,
    pub error: Option<String>,
}

impl EventType for AtomRefCreateResponse {
    fn type_id() -> &'static str { "AtomRefCreateResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefUpdateRequest {
    pub correlation_id: String,
    pub aref_uuid: String,
    pub atom_uuid: String,
    pub source_pub_key: String,
    pub aref_type: String,
    pub additional_data: Option<Value>,
}

impl EventType for AtomRefUpdateRequest {
    fn type_id() -> &'static str { "AtomRefUpdateRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefUpdateResponse {
    pub correlation_id: String,
    pub success: bool,
    pub error: Option<String>,
}

impl EventType for AtomRefUpdateResponse {
    fn type_id() -> &'static str { "AtomRefUpdateResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueSetRequest {
    pub correlation_id: String,
    pub schema_name: String,
    pub field_name: String,
    pub value: Value,
    pub source_pub_key: String,
}

impl EventType for FieldValueSetRequest {
    fn type_id() -> &'static str { "FieldValueSetRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueSetResponse {
    pub correlation_id: String,
    pub success: bool,
    pub aref_uuid: Option<String>,
    pub error: Option<String>,
}

impl EventType for FieldValueSetResponse {
    fn type_id() -> &'static str { "FieldValueSetResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldUpdateRequest {
    pub correlation_id: String,
    pub schema_name: String,
    pub field_name: String,
    pub value: Value,
    pub source_pub_key: String,
}

impl EventType for FieldUpdateRequest {
    fn type_id() -> &'static str { "FieldUpdateRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldUpdateResponse {
    pub correlation_id: String,
    pub success: bool,
    pub aref_uuid: Option<String>,
    pub error: Option<String>,
}

impl EventType for FieldUpdateResponse {
    fn type_id() -> &'static str { "FieldUpdateResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaLoadRequest {
    pub correlation_id: String,
    pub schema_name: String,
}

impl EventType for SchemaLoadRequest {
    fn type_id() -> &'static str { "SchemaLoadRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaLoadResponse {
    pub correlation_id: String,
    pub success: bool,
    pub schema_data: Option<Value>,
    pub error: Option<String>,
}

impl EventType for SchemaLoadResponse {
    fn type_id() -> &'static str { "SchemaLoadResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaApprovalRequest {
    pub correlation_id: String,
    pub schema_name: String,
}

impl EventType for SchemaApprovalRequest {
    fn type_id() -> &'static str { "SchemaApprovalRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaApprovalResponse {
    pub correlation_id: String,
    pub success: bool,
    pub error: Option<String>,
}

impl EventType for SchemaApprovalResponse {
    fn type_id() -> &'static str { "SchemaApprovalResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomHistoryRequest {
    pub correlation_id: String,
    pub aref_uuid: String,
}

impl EventType for AtomHistoryRequest {
    fn type_id() -> &'static str { "AtomHistoryRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomHistoryResponse {
    pub correlation_id: String,
    pub success: bool,
    pub history: Option<Vec<Value>>,
    pub error: Option<String>,
}

impl EventType for AtomHistoryResponse {
    fn type_id() -> &'static str { "AtomHistoryResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomGetRequest {
    pub correlation_id: String,
    pub aref_uuid: String,
}

impl EventType for AtomGetRequest {
    fn type_id() -> &'static str { "AtomGetRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomGetResponse {
    pub correlation_id: String,
    pub success: bool,
    pub atom_data: Option<Value>,
    pub error: Option<String>,
}

impl EventType for AtomGetResponse {
    fn type_id() -> &'static str { "AtomGetResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueQueryRequest {
    pub correlation_id: String,
    pub schema_name: String,
    pub field_name: String,
    pub filter: Option<Value>,
}

impl EventType for FieldValueQueryRequest {
    fn type_id() -> &'static str { "FieldValueQueryRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueQueryResponse {
    pub correlation_id: String,
    pub success: bool,
    pub field_value: Option<Value>,
    pub error: Option<String>,
}

impl EventType for FieldValueQueryResponse {
    fn type_id() -> &'static str { "FieldValueQueryResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefQueryRequest {
    pub correlation_id: String,
    pub aref_uuid: String,
}

impl EventType for AtomRefQueryRequest {
    fn type_id() -> &'static str { "AtomRefQueryRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefQueryResponse {
    pub correlation_id: String,
    pub success: bool,
    pub exists: bool,
    pub error: Option<String>,
}

impl EventType for AtomRefQueryResponse {
    fn type_id() -> &'static str { "AtomRefQueryResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaStatusRequest {
    pub correlation_id: String,
}

impl EventType for SchemaStatusRequest {
    fn type_id() -> &'static str { "SchemaStatusRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaStatusResponse {
    pub correlation_id: String,
    pub success: bool,
    pub status_data: Option<Value>,
    pub error: Option<String>,
}

impl EventType for SchemaStatusResponse {
    fn type_id() -> &'static str { "SchemaStatusResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDiscoveryRequest {
    pub correlation_id: String,
}

impl EventType for SchemaDiscoveryRequest {
    fn type_id() -> &'static str { "SchemaDiscoveryRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDiscoveryResponse {
    pub correlation_id: String,
    pub success: bool,
    pub report_data: Option<Value>,
    pub error: Option<String>,
}

impl EventType for SchemaDiscoveryResponse {
    fn type_id() -> &'static str { "SchemaDiscoveryResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefGetRequest {
    pub correlation_id: String,
    pub aref_uuid: String,
}

impl EventType for AtomRefGetRequest {
    fn type_id() -> &'static str { "AtomRefGetRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefGetResponse {
    pub correlation_id: String,
    pub success: bool,
    pub aref_data: Option<Value>,
    pub error: Option<String>,
}

impl EventType for AtomRefGetResponse {
    fn type_id() -> &'static str { "AtomRefGetResponse" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInitializationRequest {
    pub correlation_id: String,
    pub db_path: String,
    pub orchestrator_config: Option<Value>,
}

impl EventType for SystemInitializationRequest {
    fn type_id() -> &'static str { "SystemInitializationRequest" }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInitializationResponse {
    pub correlation_id: String,
    pub success: bool,
    pub error: Option<String>,
}

impl EventType for SystemInitializationResponse {
    fn type_id() -> &'static str { "SystemInitializationResponse" }
}

