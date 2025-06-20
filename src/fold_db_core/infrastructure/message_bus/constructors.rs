//! Convenience constructors for event types
//!
//! This module provides convenient constructor methods for all event types
//! to make event creation more ergonomic.

use super::events::*;
use serde_json::Value;

// ========== Core Event Constructors ==========

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

impl FieldValueQueryRequest {
    /// Create a new FieldValueQueryRequest
    pub fn new(
        correlation_id: String,
        schema_name: String,
        field_name: String,
        filter: Option<Value>,
    ) -> Self {
        Self {
            correlation_id,
            schema_name,
            field_name,
            filter,
        }
    }
}

impl FieldValueQueryResponse {
    /// Create a new FieldValueQueryResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        field_value: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            field_value,
            error,
        }
    }
}

impl AtomRefQueryRequest {
    /// Create a new AtomRefQueryRequest
    pub fn new(correlation_id: String, aref_uuid: String) -> Self {
        Self {
            correlation_id,
            aref_uuid,
        }
    }
}

impl AtomRefQueryResponse {
    /// Create a new AtomRefQueryResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        exists: bool,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            exists,
            error,
        }
    }
}

impl SchemaStatusRequest {
    /// Create a new SchemaStatusRequest
    pub fn new(correlation_id: String) -> Self {
        Self {
            correlation_id,
        }
    }
}

impl SchemaStatusResponse {
    /// Create a new SchemaStatusResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        status_data: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            status_data,
            error,
        }
    }
}

impl SchemaDiscoveryRequest {
    /// Create a new SchemaDiscoveryRequest
    pub fn new(correlation_id: String) -> Self {
        Self {
            correlation_id,
        }
    }
}

impl SchemaDiscoveryResponse {
    /// Create a new SchemaDiscoveryResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        report_data: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            report_data,
            error,
        }
    }
}

impl AtomRefGetRequest {
    /// Create a new AtomRefGetRequest
    pub fn new(correlation_id: String, aref_uuid: String) -> Self {
        Self {
            correlation_id,
            aref_uuid,
        }
    }
}

impl AtomRefGetResponse {
    /// Create a new AtomRefGetResponse
    pub fn new(
        correlation_id: String,
        success: bool,
        aref_data: Option<Value>,
        error: Option<String>,
    ) -> Self {
        Self {
            correlation_id,
            success,
            aref_data,
            error,
        }
    }
}


impl SystemInitializationRequest {
    /// Create a new SystemInitializationRequest
    pub fn new(
        correlation_id: String,
        db_path: String,
        orchestrator_config: Option<Value>,
    ) -> Self {
        Self {
            correlation_id,
            db_path,
            orchestrator_config,
        }
    }
}

impl SystemInitializationResponse {
    /// Create a new SystemInitializationResponse
    pub fn new(correlation_id: String, success: bool, error: Option<String>) -> Self {
        Self {
            correlation_id,
            success,
            error,
        }
    }
}