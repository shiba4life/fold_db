//! Pure Event-Driven SchemaManager
//!
//! This is a completely event-driven version of SchemaManager that communicates
//! only through request/response events, eliminating all direct method calls.

use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus,
    SchemaLoadRequest, SchemaLoadResponse,
    SchemaApprovalRequest, SchemaApprovalResponse,
    SchemaLoaded, SchemaChanged
};
use crate::schema::core::{SchemaCore, SchemaState, SchemaLoadingReport};
use crate::schema::{Schema, SchemaError};
use log::{info, warn, error};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Re-export unified statistics from common_stats module
pub use crate::fold_db_core::shared::stats::EventDrivenSchemaStats;

/// Response tracker for correlating requests with responses
#[derive(Debug)]
#[allow(dead_code)]
struct PendingSchemaRequest {
    correlation_id: String,
    created_at: Instant,
    response_sender: mpsc::Sender<SchemaResponseResult>,
}

/// Response result for schema requests
#[derive(Debug, Clone)]
pub enum SchemaResponseResult {
    SchemaLoadResponse(SchemaLoadResponse),
    SchemaApprovalResponse(SchemaApprovalResponse),
    Error(String),
    Timeout,
}

/// Pure event-driven SchemaManager that only communicates via events
pub struct EventDrivenSchemaManager {
    schema_core: Arc<SchemaCore>,
    message_bus: Arc<MessageBus>,
    stats: Arc<Mutex<EventDrivenSchemaStats>>,
    event_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
    pending_requests: Arc<Mutex<HashMap<String, PendingSchemaRequest>>>,
}

impl EventDrivenSchemaManager {
    pub fn new(
        path: &str,
        db_ops: Arc<crate::db_operations::DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<Self, SchemaError> {
        let schema_core = Arc::new(SchemaCore::new(path, db_ops, Arc::clone(&message_bus))?);
        
        let manager = Self {
            schema_core,
            message_bus: Arc::clone(&message_bus),
            stats: Arc::new(Mutex::new(EventDrivenSchemaStats::new())),
            event_threads: Arc::new(Mutex::new(Vec::new())),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start pure event-driven processing
        manager.start_event_processing();
        Ok(manager)
    }

    /// Start background event processing threads for request/response handling
    fn start_event_processing(&self) {
        info!("🚀 Starting EventDrivenSchemaManager pure event processing");
        
        let mut threads = self.event_threads.lock().unwrap();
        
        // Thread 1: SchemaLoadRequest processing
        let schema_load_thread = self.start_schema_load_processing();
        threads.push(schema_load_thread);
        
        // Thread 2: SchemaApprovalRequest processing
        let schema_approval_thread = self.start_schema_approval_processing();
        threads.push(schema_approval_thread);
        
        // Thread 3: Cleanup expired requests
        let cleanup_thread = self.start_cleanup_processing();
        threads.push(cleanup_thread);
        
        info!("✅ EventDrivenSchemaManager started {} event processing threads", threads.len());
    }

    /// Process SchemaLoadRequest events
    fn start_schema_load_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<SchemaLoadRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("📋 SchemaLoadRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_schema_load_request(request) {
                            error!("❌ Error processing SchemaLoadRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ SchemaLoadRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process SchemaApprovalRequest events
    fn start_schema_approval_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<SchemaApprovalRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("✅ SchemaApprovalRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_schema_approval_request(request) {
                            error!("❌ Error processing SchemaApprovalRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ SchemaApprovalRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Cleanup expired requests
    fn start_cleanup_processing(&self) -> JoinHandle<()> {
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🧹 Schema request cleanup processor started");
            
            loop {
                thread::sleep(Duration::from_secs(5));
                manager.cleanup_expired_requests();
            }
        })
    }

    /// Handle SchemaLoadRequest by loading schema and publishing response
    fn handle_schema_load_request(&self, request: SchemaLoadRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("📋 Processing SchemaLoadRequest for schema: {}", request.schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // Attempt to load the schema
        let result = self.schema_core.get_schema(&request.schema_name);

        let response = match result {
            Ok(Some(schema)) => {
                // Convert schema to JSON value
                let schema_data = serde_json::to_value(&schema).map_err(|e| {
                    format!("Failed to serialize schema: {}", e)
                })?;
                
                // Publish SchemaLoaded event
                let schema_loaded = SchemaLoaded::new(&request.schema_name, "loaded");
                if let Err(e) = self.message_bus.publish(schema_loaded) {
                    warn!("Failed to publish SchemaLoaded event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.schemas_loaded += 1;
                drop(stats);
                
                SchemaLoadResponse::new(
                    request.correlation_id,
                    true,
                    Some(schema_data),
                    None,
                )
            }
            Ok(None) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                SchemaLoadResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(format!("Schema '{}' not found", request.schema_name)),
                )
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                SchemaLoadResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(e.to_string()),
                )
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Handle SchemaApprovalRequest by approving schema and publishing response
    fn handle_schema_approval_request(&self, request: SchemaApprovalRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("✅ Processing SchemaApprovalRequest for schema: {}", request.schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // Attempt to approve the schema
        let result = self.schema_core.approve_schema(&request.schema_name);

        let response = match result {
            Ok(_) => {
                // Publish SchemaChanged event
                let schema_changed = SchemaChanged::new(&request.schema_name);
                if let Err(e) = self.message_bus.publish(schema_changed) {
                    warn!("Failed to publish SchemaChanged event: {}", e);
                }
                
                // Publish SchemaLoaded event for approved status
                let schema_loaded = SchemaLoaded::new(&request.schema_name, "approved");
                if let Err(e) = self.message_bus.publish(schema_loaded) {
                    warn!("Failed to publish SchemaLoaded event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.schemas_approved += 1;
                drop(stats);
                
                SchemaApprovalResponse::new(
                    request.correlation_id,
                    true,
                    None,
                )
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                SchemaApprovalResponse::new(
                    request.correlation_id,
                    false,
                    Some(e.to_string()),
                )
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Cleanup expired requests
    fn cleanup_expired_requests(&self) {
        let mut pending = self.pending_requests.lock().unwrap();
        let now = Instant::now();
        let mut expired_keys = Vec::new();
        
        for (key, request) in pending.iter() {
            if now.duration_since(request.created_at) > Duration::from_secs(30) {
                expired_keys.push(key.clone());
            }
        }
        
        for key in expired_keys {
            if let Some(request) = pending.remove(&key) {
                let _ = request.response_sender.send(SchemaResponseResult::Timeout);
            }
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> EventDrivenSchemaStats {
        self.stats.lock().unwrap().clone()
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn get_schema(&self, _schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest to get schema info and check state via events
    pub fn can_query_schema(&self, _schema_name: &str) -> bool {
        warn!("Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls");
        false
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest to get schema info and check state via events
    pub fn can_mutate_schema(&self, _schema_name: &str) -> bool {
        warn!("Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls");
        false
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn get_schema_state(&self, _schema_name: &str) -> Option<SchemaState> {
        warn!("Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls");
        None
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn list_schemas_by_state(&self, _state: SchemaState) -> Result<Vec<String>, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema loading violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn load_schema_from_json(&self, _json_str: &str) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct field update violates event-driven architecture
    /// Use FieldUpdateRequest/FieldUpdateResponse events instead
    pub fn update_field_ref_atom_uuid(
        &self,
        _schema_name: &str,
        _field_name: &str,
        _ref_atom_uuid: String,
    ) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven FieldUpdateRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn get_schema_status(&self) -> Result<SchemaLoadingReport, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema approval violates event-driven architecture
    /// Use SchemaApprovalRequest/SchemaApprovalResponse events instead
    pub fn approve_schema(&self, _schema_name: &str) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaApprovalRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema blocking violates event-driven architecture
    /// Use SchemaApprovalRequest/SchemaApprovalResponse events instead
    pub fn block_schema(&self, _schema_name: &str) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaApprovalRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema initialization violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn initialize_schema_system(&self) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema loading violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn load_schema_states_from_disk(&self) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct field mapping violates event-driven architecture
    /// Use FieldUpdateRequest/FieldUpdateResponse events instead
    pub fn map_fields(&self, _schema_name: &str) -> Result<Vec<crate::atom::AtomRef>, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven FieldUpdateRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct schema access violates event-driven architecture
    /// Use SchemaLoadRequest/SchemaLoadResponse events instead
    pub fn schema_exists(&self, _schema_name: &str) -> Result<bool, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct method calls".to_string()
        ))
    }
}

impl Clone for EventDrivenSchemaManager {
    fn clone(&self) -> Self {
        Self {
            schema_core: Arc::clone(&self.schema_core),
            message_bus: Arc::clone(&self.message_bus),
            stats: Arc::clone(&self.stats),
            event_threads: Arc::clone(&self.event_threads),
            pending_requests: Arc::clone(&self.pending_requests),
        }
    }
}