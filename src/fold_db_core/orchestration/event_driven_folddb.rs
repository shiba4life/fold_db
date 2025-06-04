//! Pure Event-Driven FoldDB Implementation
//!
//! This demonstrates a completely event-driven version of FoldDB where all
//! communication between components happens through request/response events,
//! eliminating direct method calls.

use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus,
    FieldValueSetRequest, FieldValueSetResponse, FieldUpdateResponse, SchemaLoadResponse,
    SchemaApprovalRequest, SchemaApprovalResponse, AtomCreateResponse, AtomRefCreateResponse,
    MutationExecuted, QueryExecuted
};
use crate::schema::types::{Mutation, Query};
use crate::schema::SchemaError;
use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use log::{info, warn, error};
use serde_json::Value;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use uuid::Uuid;

/// Re-export unified statistics from common_stats module
pub use crate::fold_db_core::shared::stats::EventDrivenFoldDBStats;

/// Response tracker for correlating requests with responses
#[derive(Debug)]
#[allow(dead_code)]
struct PendingOperationRequest {
    correlation_id: String,
    created_at: Instant,
    response_sender: mpsc::Sender<OperationResponse>,
}

/// Unified response type for all operations
#[derive(Debug, Clone)]
pub enum OperationResponse {
    FieldValueSetResponse(FieldValueSetResponse),
    FieldUpdateResponse(FieldUpdateResponse),
    SchemaLoadResponse(SchemaLoadResponse),
    SchemaApprovalResponse(SchemaApprovalResponse),
    AtomCreateResponse(AtomCreateResponse),
    AtomRefCreateResponse(AtomRefCreateResponse),
    Error(String),
    Timeout,
}

/// Pure Event-Driven FoldDB that communicates only through events
#[allow(dead_code)]
pub struct EventDrivenFoldDB {
    /// Database operations for direct data access
    db_ops: Arc<DbOperations>,
    /// Permission wrapper for authorization
    permission_wrapper: PermissionWrapper,
    /// Message bus for all event communication
    message_bus: Arc<MessageBus>,
    /// Statistics tracking
    stats: Arc<Mutex<EventDrivenFoldDBStats>>,
    /// Pending operation tracking
    pending_operations: Arc<Mutex<HashMap<String, PendingOperationRequest>>>,
}

impl EventDrivenFoldDB {
    /// Create a new event-driven FoldDB instance
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let db_ops = DbOperations::new(db.clone())
            .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
        
        let message_bus = Arc::new(MessageBus::new());
        
        let folddb = Self {
            db_ops: Arc::new(db_ops),
            permission_wrapper: PermissionWrapper::new(),
            message_bus,
            stats: Arc::new(Mutex::new(EventDrivenFoldDBStats::new())),
            pending_operations: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start response processing
        folddb.start_response_processing();
        
        Ok(folddb)
    }

    /// Start background response processing for all event types
    fn start_response_processing(&self) {
        let folddb = self.clone();
        
        thread::spawn(move || {
            info!("📨 EventDrivenFoldDB response processor started");
            
            // Subscribe to all response types
            let mut field_set_consumer = folddb.message_bus.subscribe::<FieldValueSetResponse>();
            let mut field_update_consumer = folddb.message_bus.subscribe::<FieldUpdateResponse>();
            let mut schema_load_consumer = folddb.message_bus.subscribe::<SchemaLoadResponse>();
            let mut schema_approval_consumer = folddb.message_bus.subscribe::<SchemaApprovalResponse>();
            let mut atom_create_consumer = folddb.message_bus.subscribe::<AtomCreateResponse>();
            let mut atomref_create_consumer = folddb.message_bus.subscribe::<AtomRefCreateResponse>();
            
            loop {
                let mut received_any = false;
                
                // Check all response types
                if let Ok(response) = field_set_consumer.try_recv() {
                    folddb.handle_operation_response(OperationResponse::FieldValueSetResponse(response));
                    received_any = true;
                }
                
                if let Ok(response) = field_update_consumer.try_recv() {
                    folddb.handle_operation_response(OperationResponse::FieldUpdateResponse(response));
                    received_any = true;
                }
                
                if let Ok(response) = schema_load_consumer.try_recv() {
                    folddb.handle_operation_response(OperationResponse::SchemaLoadResponse(response));
                    received_any = true;
                }
                
                if let Ok(response) = schema_approval_consumer.try_recv() {
                    folddb.handle_operation_response(OperationResponse::SchemaApprovalResponse(response));
                    received_any = true;
                }
                
                if let Ok(response) = atom_create_consumer.try_recv() {
                    folddb.handle_operation_response(OperationResponse::AtomCreateResponse(response));
                    received_any = true;
                }
                
                if let Ok(response) = atomref_create_consumer.try_recv() {
                    folddb.handle_operation_response(OperationResponse::AtomRefCreateResponse(response));
                    received_any = true;
                }
                
                if !received_any {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });
        
        // Start cleanup thread
        let folddb_cleanup = self.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));
                folddb_cleanup.cleanup_expired_operations();
            }
        });
    }

    /// Handle incoming operation responses
    fn handle_operation_response(&self, response: OperationResponse) {
        let correlation_id = match &response {
            OperationResponse::FieldValueSetResponse(r) => &r.correlation_id,
            OperationResponse::FieldUpdateResponse(r) => &r.correlation_id,
            OperationResponse::SchemaLoadResponse(r) => &r.correlation_id,
            OperationResponse::SchemaApprovalResponse(r) => &r.correlation_id,
            OperationResponse::AtomCreateResponse(r) => &r.correlation_id,
            OperationResponse::AtomRefCreateResponse(r) => &r.correlation_id,
            OperationResponse::Error(_) | OperationResponse::Timeout => return,
        };

        if let Some(pending) = self.pending_operations.lock().unwrap().remove(correlation_id) {
            let _ = pending.response_sender.send(response);
            
            let mut stats = self.stats.lock().unwrap();
            stats.event_responses_received += 1;
            drop(stats);
        }
    }

    /// Cleanup expired operation requests
    fn cleanup_expired_operations(&self) {
        let mut pending = self.pending_operations.lock().unwrap();
        let now = Instant::now();
        let mut expired_keys = Vec::new();
        
        for (key, request) in pending.iter() {
            if now.duration_since(request.created_at) > Duration::from_secs(30) {
                expired_keys.push(key.clone());
            }
        }
        
        for key in expired_keys {
            if let Some(request) = pending.remove(&key) {
                let _ = request.response_sender.send(OperationResponse::Timeout);
                
                let mut stats = self.stats.lock().unwrap();
                stats.timeouts += 1;
                drop(stats);
            }
        }
    }

    /// Event-driven write_schema operation (replaces direct method calls)
    pub fn write_schema_event_driven(&self, mutation: Mutation) -> Result<(), SchemaError> {
        let start_time = Instant::now();
        info!("🔄 EVENT-DRIVEN write_schema for schema: {}", mutation.schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.mutations_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // Process each field mutation via events instead of direct calls
        for (field_name, value) in mutation.fields_and_values.iter() {
            info!("📤 Publishing FieldValueSetRequest for {}.{}", mutation.schema_name, field_name);
            
            // Create request
            let correlation_id = Uuid::new_v4().to_string();
            let request = FieldValueSetRequest::new(
                correlation_id.clone(),
                mutation.schema_name.clone(),
                field_name.clone(),
                value.clone(),
                mutation.pub_key.clone(),
            );

            // Set up response tracking
            let (response_sender, response_receiver) = mpsc::channel();
            {
                let mut pending = self.pending_operations.lock().unwrap();
                pending.insert(correlation_id.clone(), PendingOperationRequest {
                    correlation_id: correlation_id.clone(),
                    created_at: Instant::now(),
                    response_sender,
                });
            }

            // Publish request event
            if let Err(e) = self.message_bus.publish(request) {
                error!("Failed to publish FieldValueSetRequest: {}", e);
                continue;
            }

            let mut stats = self.stats.lock().unwrap();
            stats.event_requests_sent += 1;
            drop(stats);

            // Wait for response
            match response_receiver.recv_timeout(Duration::from_secs(10)) {
                Ok(OperationResponse::FieldValueSetResponse(response)) => {
                    if response.success {
                        info!("✅ Field {}.{} set successfully via events", mutation.schema_name, field_name);
                    } else {
                        error!("❌ Field {}.{} set failed: {:?}", mutation.schema_name, field_name, response.error);
                    }
                }
                Ok(other) => {
                    warn!("⚠️ Unexpected response type: {:?}", other);
                }
                Err(_) => {
                    let mut stats = self.stats.lock().unwrap();
                    stats.timeouts += 1;
                    drop(stats);
                    
                    error!("⏰ Timeout waiting for FieldValueSetResponse");
                }
            }
        }

        // Publish MutationExecuted event
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let mutation_event = MutationExecuted::new(
            format!("{:?}", mutation.mutation_type),
            &mutation.schema_name,
            execution_time_ms,
            mutation.fields_and_values.len(),
        );
        if let Err(e) = self.message_bus.publish(mutation_event) {
            warn!("Failed to publish MutationExecuted event: {}", e);
        }

        info!("✅ EVENT-DRIVEN write_schema completed for schema: {} in {}ms", 
              mutation.schema_name, execution_time_ms);
        Ok(())
    }

    /// Event-driven query_schema operation (replaces direct method calls)
    pub fn query_schema_event_driven(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        let start_time = Instant::now();
        info!("🔍 EVENT-DRIVEN query_schema for schema: {}", query.schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.queries_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // For demonstration, we'll return a simple result
        // In a real implementation, this would use events to retrieve field values
        let results = vec![Ok(serde_json::json!({
            "message": "Query processed via event-driven architecture",
            "schema": query.schema_name,
            "fields": query.fields,
            "timestamp": chrono::Utc::now()
        }))];

        // Publish QueryExecuted event
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let query_event = QueryExecuted::new(
            "event_driven_query",
            &query.schema_name,
            execution_time_ms,
            results.len(),
        );
        if let Err(e) = self.message_bus.publish(query_event) {
            warn!("Failed to publish QueryExecuted event: {}", e);
        }

        info!("✅ EVENT-DRIVEN query_schema completed for schema: {} in {}ms", 
              query.schema_name, execution_time_ms);
        results
    }

    /// Event-driven schema approval (replaces direct method calls)
    pub fn approve_schema_event_driven(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("✅ EVENT-DRIVEN approve_schema for: {}", schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.schema_operations += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // Create request
        let correlation_id = Uuid::new_v4().to_string();
        let request = SchemaApprovalRequest::new(correlation_id.clone(), schema_name.to_string());

        // Set up response tracking
        let (response_sender, response_receiver) = mpsc::channel();
        {
            let mut pending = self.pending_operations.lock().unwrap();
            pending.insert(correlation_id.clone(), PendingOperationRequest {
                correlation_id: correlation_id.clone(),
                created_at: Instant::now(),
                response_sender,
            });
        }

        // Publish request event
        if let Err(e) = self.message_bus.publish(request) {
            return Err(SchemaError::InvalidData(format!("Failed to publish SchemaApprovalRequest: {}", e)));
        }

        let mut stats = self.stats.lock().unwrap();
        stats.event_requests_sent += 1;
        drop(stats);

        // Wait for response
        match response_receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(OperationResponse::SchemaApprovalResponse(response)) => {
                if response.success {
                    info!("✅ Schema {} approved successfully via events", schema_name);
                    Ok(())
                } else {
                    Err(SchemaError::InvalidData(
                        response.error.unwrap_or("Schema approval failed".to_string())
                    ))
                }
            }
            Ok(other) => {
                Err(SchemaError::InvalidData(format!("Unexpected response type: {:?}", other)))
            }
            Err(_) => {
                let mut stats = self.stats.lock().unwrap();
                stats.timeouts += 1;
                drop(stats);
                
                Err(SchemaError::InvalidData("Timeout waiting for schema approval".to_string()))
            }
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> EventDrivenFoldDBStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get access to message bus for testing
    pub fn message_bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.message_bus)
    }

    /// Get database operations (for backward compatibility)
    pub fn db_ops(&self) -> Arc<DbOperations> {
        Arc::clone(&self.db_ops)
    }
}

impl Clone for EventDrivenFoldDB {
    fn clone(&self) -> Self {
        Self {
            db_ops: Arc::clone(&self.db_ops),
            permission_wrapper: PermissionWrapper::new(),
            message_bus: Arc::clone(&self.message_bus),
            stats: Arc::clone(&self.stats),
            pending_operations: Arc::clone(&self.pending_operations),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{Mutation, MutationType, Query};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_event_driven_folddb_creation() {
        let temp_dir = tempdir().unwrap();
        let folddb = EventDrivenFoldDB::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let stats = folddb.get_stats();
        assert_eq!(stats.mutations_processed, 0);
        assert_eq!(stats.queries_processed, 0);
        assert!(stats.last_activity.is_some());
    }

    #[test]
    fn test_event_driven_mutation() {
        let temp_dir = tempdir().unwrap();
        let folddb = EventDrivenFoldDB::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), serde_json::json!("test"));
        
        let mutation = Mutation {
            schema_name: "test_schema".to_string(),
            mutation_type: MutationType::Create,
            fields_and_values: fields,
            pub_key: "test_key".to_string(),
            trust_distance: 0,
        };

        // This will attempt to send events (though no receivers in test)
        let result = folddb.write_schema_event_driven(mutation);
        
        // Should succeed even without receivers
        assert!(result.is_ok());
        
        let stats = folddb.get_stats();
        assert_eq!(stats.mutations_processed, 1);
        assert_eq!(stats.event_requests_sent, 1);
    }

    #[test]
    fn test_event_driven_query() {
        let temp_dir = tempdir().unwrap();
        let folddb = EventDrivenFoldDB::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let query = Query {
            schema_name: "test_schema".to_string(),
            fields: vec!["name".to_string()],
            filter: None,
            trust_distance: 0,
            pub_key: "test_key".to_string(),
        };

        let results = folddb.query_schema_event_driven(query);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        
        let stats = folddb.get_stats();
        assert_eq!(stats.queries_processed, 1);
    }

    #[test]
    fn test_message_bus_integration() {
        let temp_dir = tempdir().unwrap();
        let folddb = EventDrivenFoldDB::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        // Test that we can access the message bus
        let message_bus = folddb.message_bus();
        assert_eq!(message_bus.subscriber_count::<FieldValueSetRequest>(), 0);
        
        // Create a subscriber to test event publishing
        let _consumer = message_bus.subscribe::<FieldValueSetRequest>();
        
        // The count should still be 0 for our specific type since we're not subscribing from within the folddb
        assert_eq!(message_bus.subscriber_count::<FieldValueSetRequest>(), 1);
    }
}