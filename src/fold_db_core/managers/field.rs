//! Pure Event-Driven FieldManager
//!
//! This is a completely event-driven version of FieldManager that communicates
//! only through request/response events, eliminating all direct method calls to AtomManager.

use crate::fold_db_core::services::field_retrieval::FieldRetrievalService;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus,
    FieldValueSetRequest, FieldValueSetResponse,
    FieldUpdateRequest, FieldUpdateResponse,
    AtomCreateRequest, AtomCreateResponse,
    AtomRefCreateRequest, AtomRefCreateResponse, AtomRefUpdateResponse,
    FieldValueSet
};
use crate::schema::{Schema, SchemaError};
use log::{info, warn, error};
use serde_json::Value;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use uuid::Uuid;


/// Re-export unified statistics from common_stats module
pub use crate::fold_db_core::shared::stats::EventDrivenFieldStats;

/// Response tracker for correlating requests with responses
#[derive(Debug)]
#[allow(dead_code)]
struct PendingRequest {
    correlation_id: String,
    created_at: Instant,
    response_sender: mpsc::Sender<ResponseResult>,
}

/// Response result for requests
#[derive(Debug, Clone)]
pub enum ResponseResult {
    AtomCreateResponse(AtomCreateResponse),
    AtomRefCreateResponse(AtomRefCreateResponse),
    AtomRefUpdateResponse(AtomRefUpdateResponse),
    FieldValueSetSuccess(String), // aref_uuid
    FieldUpdateSuccess(String),   // aref_uuid
    Error(String),
    Timeout,
}

/// Pure event-driven FieldManager that only communicates via events
#[allow(dead_code)]
pub struct FieldManager {
    retrieval_service: FieldRetrievalService,
    message_bus: Arc<MessageBus>,
    stats: Arc<Mutex<EventDrivenFieldStats>>,
    event_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
    pending_requests: Arc<Mutex<HashMap<String, PendingRequest>>>,
}

impl FieldManager {
    pub fn new(message_bus: Arc<MessageBus>) -> Self {
        let manager = Self {
            retrieval_service: FieldRetrievalService::new(Arc::clone(&message_bus)),
            message_bus: Arc::clone(&message_bus),
            stats: Arc::new(Mutex::new(EventDrivenFieldStats::new())),
            event_threads: Arc::new(Mutex::new(Vec::new())),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start pure event-driven processing
        manager.start_event_processing();
        manager
    }

    /// Start background event processing threads for request/response handling
    fn start_event_processing(&self) {
        info!("🚀 Starting FieldManager pure event processing");
        
        let mut threads = self.event_threads.lock().unwrap();
        
        // Thread 1: FieldValueSetRequest processing
        let field_set_thread = self.start_field_set_processing();
        threads.push(field_set_thread);
        
        // Thread 2: FieldUpdateRequest processing
        let field_update_thread = self.start_field_update_processing();
        threads.push(field_update_thread);
        
        // Thread 3: Response processing for all types
        let response_thread = self.start_response_processing();
        threads.push(response_thread);
        
        // Thread 4: Cleanup expired requests
        let cleanup_thread = self.start_cleanup_processing();
        threads.push(cleanup_thread);
        
        info!("✅ FieldManager started {} event processing threads", threads.len());
    }

    /// Process FieldValueSetRequest events
    fn start_field_set_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<FieldValueSetRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🔧 FieldValueSetRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_field_value_set_request(request) {
                            error!("❌ Error processing FieldValueSetRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ FieldValueSetRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process FieldUpdateRequest events
    fn start_field_update_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<FieldUpdateRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🔄 FieldUpdateRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_field_update_request(request) {
                            error!("❌ Error processing FieldUpdateRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ FieldUpdateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process all response events
    fn start_response_processing(&self) -> JoinHandle<()> {
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("📨 Response processor started");
            
            // Subscribe to all response types
            let mut atom_create_consumer = manager.message_bus.subscribe::<AtomCreateResponse>();
            let mut atomref_create_consumer = manager.message_bus.subscribe::<AtomRefCreateResponse>();
            let mut atomref_update_consumer = manager.message_bus.subscribe::<AtomRefUpdateResponse>();
            
            loop {
                let mut received_any = false;
                
                // Check AtomCreateResponse
                if let Ok(response) = atom_create_consumer.try_recv() {
                    manager.handle_atom_create_response(response);
                    received_any = true;
                }
                
                // Check AtomRefCreateResponse
                if let Ok(response) = atomref_create_consumer.try_recv() {
                    manager.handle_atomref_create_response(response);
                    received_any = true;
                }
                
                // Check AtomRefUpdateResponse
                if let Ok(response) = atomref_update_consumer.try_recv() {
                    manager.handle_atomref_update_response(response);
                    received_any = true;
                }
                
                if !received_any {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        })
    }

    /// Cleanup expired requests
    fn start_cleanup_processing(&self) -> JoinHandle<()> {
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🧹 Request cleanup processor started");
            
            loop {
                thread::sleep(Duration::from_secs(5));
                manager.cleanup_expired_requests();
            }
        })
    }

    /// Handle FieldValueSetRequest by orchestrating atom and atomref creation
    fn handle_field_value_set_request(&self, request: FieldValueSetRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 Processing FieldValueSetRequest for {}.{}", request.schema_name, request.field_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.field_sets_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // For now, create a simple atom and then an atomref
        // Step 1: Create atom
        let atom_correlation_id = Uuid::new_v4().to_string();
        let atom_request = AtomCreateRequest::new(
            atom_correlation_id.clone(),
            request.schema_name.clone(),
            request.source_pub_key.clone(),
            None,
            request.value.clone(),
            Some("Approved".to_string()),
        );

        // Set up response tracking
        let (response_sender, response_receiver) = mpsc::channel();
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(atom_correlation_id.clone(), PendingRequest {
                correlation_id: atom_correlation_id.clone(),
                created_at: Instant::now(),
                response_sender,
            });
        }

        // Send atom create request
        self.message_bus.publish(atom_request)?;
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_sent += 1;
        drop(stats);

        // Wait for atom creation response
        let atom_response = match response_receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(ResponseResult::AtomCreateResponse(response)) => response,
            Ok(other) => {
                let error_msg = format!("Unexpected response type: {:?}", other);
                let response = FieldValueSetResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(error_msg),
                );
                self.message_bus.publish(response)?;
                return Ok(());
            }
            Err(_) => {
                let mut stats = self.stats.lock().unwrap();
                stats.timeouts += 1;
                drop(stats);
                
                let response = FieldValueSetResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some("Timeout waiting for atom creation".to_string()),
                );
                self.message_bus.publish(response)?;
                return Ok(());
            }
        };

        if !atom_response.success {
            let response = FieldValueSetResponse::new(
                request.correlation_id,
                false,
                None,
                atom_response.error,
            );
            self.message_bus.publish(response)?;
            return Ok(());
        }

        // Step 2: Create atomref
        let atom_uuid = atom_response.atom_uuid.unwrap();
        let aref_uuid = Uuid::new_v4().to_string();
        let aref_correlation_id = Uuid::new_v4().to_string();
        
        let atomref_request = AtomRefCreateRequest::new(
            aref_correlation_id.clone(),
            aref_uuid.clone(),
            atom_uuid,
            request.source_pub_key,
            "Single".to_string(), // Default to Single type
        );

        // Set up response tracking for atomref
        let (aref_response_sender, aref_response_receiver) = mpsc::channel();
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(aref_correlation_id.clone(), PendingRequest {
                correlation_id: aref_correlation_id.clone(),
                created_at: Instant::now(),
                response_sender: aref_response_sender,
            });
        }

        // Send atomref create request
        self.message_bus.publish(atomref_request)?;
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_sent += 1;
        drop(stats);

        // Wait for atomref creation response
        let aref_response = match aref_response_receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(ResponseResult::AtomRefCreateResponse(response)) => response,
            Ok(other) => {
                let error_msg = format!("Unexpected response type: {:?}", other);
                let response = FieldValueSetResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(error_msg),
                );
                self.message_bus.publish(response)?;
                return Ok(());
            }
            Err(_) => {
                let mut stats = self.stats.lock().unwrap();
                stats.timeouts += 1;
                drop(stats);
                
                let response = FieldValueSetResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some("Timeout waiting for atomref creation".to_string()),
                );
                self.message_bus.publish(response)?;
                return Ok(());
            }
        };

        // Send final response
        let final_response = if aref_response.success {
            // Publish FieldValueSet event
            let field_path = format!("{}.{}", request.schema_name, request.field_name);
            let field_event = FieldValueSet::new(field_path, request.value, "event_driven_field_manager");
            if let Err(e) = self.message_bus.publish(field_event) {
                warn!("Failed to publish FieldValueSet event: {}", e);
            }
            
            FieldValueSetResponse::new(
                request.correlation_id,
                true,
                Some(aref_uuid),
                None,
            )
        } else {
            FieldValueSetResponse::new(
                request.correlation_id,
                false,
                None,
                aref_response.error,
            )
        };

        self.message_bus.publish(final_response)?;
        Ok(())
    }

    /// Handle FieldUpdateRequest by orchestrating atomref updates
    fn handle_field_update_request(&self, request: FieldUpdateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Processing FieldUpdateRequest for {}.{}", request.schema_name, request.field_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.field_updates_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // For simplicity, create a new atom and update the atomref
        // Step 1: Create new atom
        let atom_correlation_id = Uuid::new_v4().to_string();
        let atom_request = AtomCreateRequest::new(
            atom_correlation_id.clone(),
            request.schema_name.clone(),
            request.source_pub_key.clone(),
            None, // We'd need the previous atom UUID in a real implementation
            request.value.clone(),
            Some("Approved".to_string()),
        );

        // Set up response tracking
        let (response_sender, response_receiver) = mpsc::channel();
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(atom_correlation_id.clone(), PendingRequest {
                correlation_id: atom_correlation_id.clone(),
                created_at: Instant::now(),
                response_sender,
            });
        }

        // Send atom create request
        self.message_bus.publish(atom_request)?;
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_sent += 1;
        drop(stats);

        // Wait for atom creation response
        let atom_response = match response_receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(ResponseResult::AtomCreateResponse(response)) => response,
            Ok(other) => {
                let error_msg = format!("Unexpected response type: {:?}", other);
                let response = FieldUpdateResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(error_msg),
                );
                self.message_bus.publish(response)?;
                return Ok(());
            }
            Err(_) => {
                let mut stats = self.stats.lock().unwrap();
                stats.timeouts += 1;
                drop(stats);
                
                let response = FieldUpdateResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some("Timeout waiting for atom creation".to_string()),
                );
                self.message_bus.publish(response)?;
                return Ok(());
            }
        };

        if !atom_response.success {
            let response = FieldUpdateResponse::new(
                request.correlation_id,
                false,
                None,
                atom_response.error,
            );
            self.message_bus.publish(response)?;
            return Ok(());
        }

        // For now, just return success - in a real implementation we'd update the existing atomref
        let final_response = FieldUpdateResponse::new(
            request.correlation_id,
            true,
            atom_response.atom_uuid,
            None,
        );

        self.message_bus.publish(final_response)?;
        Ok(())
    }

    /// Handle AtomCreateResponse
    fn handle_atom_create_response(&self, response: AtomCreateResponse) {
        if let Some(pending) = self.pending_requests.lock().unwrap().remove(&response.correlation_id) {
            let _ = pending.response_sender.send(ResponseResult::AtomCreateResponse(response));
            
            let mut stats = self.stats.lock().unwrap();
            stats.responses_received += 1;
            drop(stats);
        }
    }

    /// Handle AtomRefCreateResponse
    fn handle_atomref_create_response(&self, response: AtomRefCreateResponse) {
        if let Some(pending) = self.pending_requests.lock().unwrap().remove(&response.correlation_id) {
            let _ = pending.response_sender.send(ResponseResult::AtomRefCreateResponse(response));
            
            let mut stats = self.stats.lock().unwrap();
            stats.responses_received += 1;
            drop(stats);
        }
    }

    /// Handle AtomRefUpdateResponse
    fn handle_atomref_update_response(&self, response: AtomRefUpdateResponse) {
        if let Some(pending) = self.pending_requests.lock().unwrap().remove(&response.correlation_id) {
            let _ = pending.response_sender.send(ResponseResult::AtomRefUpdateResponse(response));
            
            let mut stats = self.stats.lock().unwrap();
            stats.responses_received += 1;
            drop(stats);
        }
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
                let _ = request.response_sender.send(ResponseResult::Timeout);
                
                let mut stats = self.stats.lock().unwrap();
                stats.timeouts += 1;
                drop(stats);
            }
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> EventDrivenFieldStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get field value using retrieval service (backward compatibility)
    pub fn get_field_value(&self, _schema: &Schema, _field: &str) -> Result<Value, SchemaError> {
        info!("🔍 FieldManager::get_field_value - delegating to FieldRetrievalService");
        // For now, we need an AtomManager to get field values
        // In a fully event-driven system, this would also be event-based
        Err(SchemaError::InvalidData("Field retrieval not implemented in event-driven mode".to_string()))
    }

    /// Get field value with filter (backward compatibility)
    pub fn get_field_value_with_filter(
        &self,
        _schema: &Schema,
        _field: &str,
        _filter: &Value,
    ) -> Result<Value, SchemaError> {
        info!("🔄 FieldManager::get_field_value_with_filter - delegating to FieldRetrievalService");
        // For now, we need an AtomManager to get field values
        // In a fully event-driven system, this would also be event-based
        Err(SchemaError::InvalidData("Field retrieval with filter not implemented in event-driven mode".to_string()))
    }

    // ========== BACKWARD COMPATIBILITY METHODS ==========
    // These methods are provided for legacy code compatibility
    // They delegate to the event-driven system or return appropriate errors

}

impl Clone for FieldManager {
    fn clone(&self) -> Self {
        Self {
            retrieval_service: FieldRetrievalService::new_default(),
            message_bus: Arc::clone(&self.message_bus),
            stats: Arc::clone(&self.stats),
            event_threads: Arc::clone(&self.event_threads),
            pending_requests: Arc::clone(&self.pending_requests),
        }
    }
}
