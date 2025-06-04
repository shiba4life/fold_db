//! Pure Event-Driven AtomManager
//!
//! This is a completely event-driven version of AtomManager that communicates
//! only through request/response events, eliminating all direct method calls.

use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomRefRange, AtomStatus};
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus,
    AtomCreateRequest, AtomCreateResponse,
    AtomUpdateRequest, AtomUpdateResponse,
    AtomRefCreateRequest, AtomRefCreateResponse,
    AtomRefUpdateRequest, AtomRefUpdateResponse,
    AtomCreated, AtomUpdated, AtomRefCreated, AtomRefUpdated
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use log::{info, warn, error};

/// Re-export unified statistics from shared stats module
pub use super::super::shared::EventDrivenAtomStats;

/// Pure event-driven AtomManager that only communicates via events
pub struct AtomManager {
    db_ops: Arc<DbOperations>,
    atoms: Arc<Mutex<HashMap<String, Atom>>>,
    ref_atoms: Arc<Mutex<HashMap<String, AtomRef>>>,
    ref_collections: Arc<Mutex<HashMap<String, AtomRefCollection>>>,
    ref_ranges: Arc<Mutex<HashMap<String, AtomRefRange>>>,
    message_bus: Arc<MessageBus>,
    stats: Arc<Mutex<EventDrivenAtomStats>>,
    event_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl AtomManager {
    pub fn new(db_ops: DbOperations, message_bus: Arc<MessageBus>) -> Self {
        let mut atoms = HashMap::new();
        let mut ref_atoms = HashMap::new();
        let mut ref_collections = HashMap::new();
        let mut ref_ranges = HashMap::new();

        // Load existing data from database
        for result in db_ops.db().iter().flatten() {
            let key_str = String::from_utf8_lossy(result.0.as_ref());
            let bytes = result.1.as_ref();

            if let Some(stripped) = key_str.strip_prefix("atom:") {
                if let Ok(atom) = serde_json::from_slice(bytes) {
                    atoms.insert(stripped.to_string(), atom);
                }
            } else if let Some(stripped) = key_str.strip_prefix("ref:") {
                if let Ok(atom_ref) = serde_json::from_slice::<AtomRef>(bytes) {
                    ref_atoms.insert(stripped.to_string(), atom_ref);
                } else if let Ok(collection) = serde_json::from_slice::<AtomRefCollection>(bytes) {
                    ref_collections.insert(stripped.to_string(), collection);
                } else if let Ok(range) = serde_json::from_slice::<AtomRefRange>(bytes) {
                    ref_ranges.insert(stripped.to_string(), range);
                }
            }
        }

        let manager = Self {
            db_ops: Arc::new(db_ops),
            atoms: Arc::new(Mutex::new(atoms)),
            ref_atoms: Arc::new(Mutex::new(ref_atoms)),
            ref_collections: Arc::new(Mutex::new(ref_collections)),
            ref_ranges: Arc::new(Mutex::new(ref_ranges)),
            message_bus: Arc::clone(&message_bus),
            stats: Arc::new(Mutex::new(EventDrivenAtomStats::new())),
            event_threads: Arc::new(Mutex::new(Vec::new())),
        };

        // Start pure event-driven processing
        manager.start_event_processing();
        manager
    }

    /// Start background event processing threads for request/response handling
    fn start_event_processing(&self) {
        info!("🚀 Starting AtomManager pure event processing");
        
        let mut threads = self.event_threads.lock().unwrap();
        
        // Thread 1: AtomCreateRequest processing
        let atom_create_thread = self.start_atom_create_processing();
        threads.push(atom_create_thread);
        
        // Thread 2: AtomUpdateRequest processing
        let atom_update_thread = self.start_atom_update_processing();
        threads.push(atom_update_thread);
        
        // Thread 3: AtomRefCreateRequest processing
        let atomref_create_thread = self.start_atomref_create_processing();
        threads.push(atomref_create_thread);
        
        // Thread 4: AtomRefUpdateRequest processing
        let atomref_update_thread = self.start_atomref_update_processing();
        threads.push(atomref_update_thread);
        
        
        info!("✅ AtomManager started {} event processing threads", threads.len());
    }

    /// Process AtomCreateRequest events
    fn start_atom_create_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomCreateRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("⚛️ AtomCreateRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atom_create_request(request) {
                            error!("❌ Error processing AtomCreateRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomCreateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process AtomUpdateRequest events
    fn start_atom_update_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomUpdateRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🔄 AtomUpdateRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atom_update_request(request) {
                            error!("❌ Error processing AtomUpdateRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomUpdateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process AtomRefCreateRequest events
    fn start_atomref_create_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomRefCreateRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🔗 AtomRefCreateRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atomref_create_request(request) {
                            error!("❌ Error processing AtomRefCreateRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomRefCreateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process AtomRefUpdateRequest events
    fn start_atomref_update_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomRefUpdateRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("🔄 AtomRefUpdateRequest processor started");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atomref_update_request(request) {
                            error!("❌ Error processing AtomRefUpdateRequest: {}", e);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomRefUpdateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Handle AtomCreateRequest by creating atom and publishing response
    fn handle_atom_create_request(&self, request: AtomCreateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 Processing AtomCreateRequest for schema: {}", request.schema_name);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        let result = self.db_ops.create_atom(
            &request.schema_name,
            request.source_pub_key.clone(),
            request.prev_atom_uuid.clone(),
            request.content.clone(),
            request.status.as_ref().and_then(|s| match s.as_str() {
                "Active" => Some(AtomStatus::Active),
                "Deleted" => Some(AtomStatus::Deleted),
                _ => None,
            }),
        );

        let response = match result {
            Ok(atom) => {
                // Store in memory cache
                self.atoms.lock().unwrap().insert(atom.uuid().to_string(), atom.clone());
                
                // Publish AtomCreated event
                let atom_created = AtomCreated::new(atom.uuid().to_string(), request.content.clone());
                if let Err(e) = self.message_bus.publish(atom_created) {
                    warn!("Failed to publish AtomCreated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atoms_created += 1;
                drop(stats);
                
                AtomCreateResponse::new(
                    request.correlation_id,
                    true,
                    Some(atom.uuid().to_string()),
                    None,
                    Some(request.content),
                )
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomCreateResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(e.to_string()),
                    None,
                )
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Handle AtomUpdateRequest by updating atom and publishing response
    fn handle_atom_update_request(&self, request: AtomUpdateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Processing AtomUpdateRequest for atom: {}", request.atom_uuid);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // For simplicity, we'll create a new atom with the updated content
        // In a real implementation, you might want to update the existing atom
        let atom = Atom::new(
            "default_schema".to_string(),
            request.source_pub_key.clone(),
            request.content.clone(),
        );
        let atom_uuid = atom.uuid().to_string();

        let result = self.db_ops.db().insert(
            format!("atom:{}", atom_uuid),
            serde_json::to_vec(&atom)?,
        );

        let response = match result {
            Ok(_) => {
                // Store in memory cache
                self.atoms.lock().unwrap().insert(atom_uuid.clone(), atom);
                
                // Publish AtomUpdated event
                let atom_updated = AtomUpdated::new(atom_uuid, request.content);
                if let Err(e) = self.message_bus.publish(atom_updated) {
                    warn!("Failed to publish AtomUpdated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atoms_updated += 1;
                drop(stats);
                
                AtomUpdateResponse::new(request.correlation_id, true, None)
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomUpdateResponse::new(request.correlation_id, false, Some(e.to_string()))
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Handle AtomRefCreateRequest by creating AtomRef and publishing response
    fn handle_atomref_create_request(&self, request: AtomRefCreateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔗 Processing AtomRefCreateRequest for type: {}", request.aref_type);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        let result: Result<(), Box<dyn std::error::Error>> = match request.aref_type.as_str() {
            "Single" => {
                let aref = self.db_ops.update_atom_ref(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_atoms.lock().unwrap().insert(request.aref_uuid.clone(), aref);
                Ok(())
            }
            "Collection" => {
                let collection = self.db_ops.update_atom_ref_collection(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    "default".to_string(), // Default ID
                    request.source_pub_key.clone(),
                )?;
                self.ref_collections.lock().unwrap().insert(request.aref_uuid.clone(), collection);
                Ok(())
            }
            "Range" => {
                let range = self.db_ops.update_atom_ref_range(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    "default".to_string(), // Default key
                    request.source_pub_key.clone(),
                )?;
                self.ref_ranges.lock().unwrap().insert(request.aref_uuid.clone(), range);
                Ok(())
            }
            _ => Err(format!("Unknown AtomRef type: {}", request.aref_type).into())
        };

        let response = match result {
            Ok(_) => {
                // Publish AtomRefCreated event
                let atomref_created = AtomRefCreated::new(
                    &request.aref_uuid,
                    &request.aref_type,
                    format!("{}:{}", request.aref_type.to_lowercase(), request.aref_uuid),
                );
                if let Err(e) = self.message_bus.publish(atomref_created) {
                    warn!("Failed to publish AtomRefCreated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atom_refs_created += 1;
                drop(stats);
                
                AtomRefCreateResponse::new(request.correlation_id, true, None)
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomRefCreateResponse::new(request.correlation_id, false, Some(e.to_string()))
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Handle AtomRefUpdateRequest by updating AtomRef and publishing response
    fn handle_atomref_update_request(&self, request: AtomRefUpdateRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Processing AtomRefUpdateRequest for: {}", request.aref_uuid);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        let result: Result<(), Box<dyn std::error::Error>> = match request.aref_type.as_str() {
            "Single" => {
                let aref = self.db_ops.update_atom_ref(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_atoms.lock().unwrap().insert(request.aref_uuid.clone(), aref);
                Ok(())
            }
            "Collection" => {
                let id = request.additional_data
                    .as_ref()
                    .and_then(|d| d.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let collection = self.db_ops.update_atom_ref_collection(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    id.to_string(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_collections.lock().unwrap().insert(request.aref_uuid.clone(), collection);
                Ok(())
            }
            "Range" => {
                let key = request.additional_data
                    .as_ref()
                    .and_then(|d| d.get("key"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let range = self.db_ops.update_atom_ref_range(
                    &request.aref_uuid,
                    request.atom_uuid.clone(),
                    key.to_string(),
                    request.source_pub_key.clone(),
                )?;
                self.ref_ranges.lock().unwrap().insert(request.aref_uuid.clone(), range);
                Ok(())
            }
            _ => Err(format!("Unknown AtomRef type: {}", request.aref_type).into())
        };

        let response = match result {
            Ok(_) => {
                // Publish AtomRefUpdated event
                let atomref_updated = AtomRefUpdated::new(
                    &request.aref_uuid,
                    format!("{}:{}", request.aref_type.to_lowercase(), request.aref_uuid),
                    "update",
                );
                if let Err(e) = self.message_bus.publish(atomref_updated) {
                    warn!("Failed to publish AtomRefUpdated event: {}", e);
                }
                
                let mut stats = self.stats.lock().unwrap();
                stats.atom_refs_updated += 1;
                drop(stats);
                
                AtomRefUpdateResponse::new(request.correlation_id, true, None)
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                AtomRefUpdateResponse::new(request.correlation_id, false, Some(e.to_string()))
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Get current statistics
    pub fn get_stats(&self) -> EventDrivenAtomStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get access to stored data (for backward compatibility/testing)
    pub fn get_atoms(&self) -> Arc<Mutex<HashMap<String, Atom>>> {
        Arc::clone(&self.atoms)
    }

    pub fn get_ref_atoms(&self) -> Arc<Mutex<HashMap<String, AtomRef>>> {
        Arc::clone(&self.ref_atoms)
    }

    pub fn get_ref_collections(&self) -> Arc<Mutex<HashMap<String, AtomRefCollection>>> {
        Arc::clone(&self.ref_collections)
    }

    pub fn get_ref_ranges(&self) -> Arc<Mutex<HashMap<String, AtomRefRange>>> {
        Arc::clone(&self.ref_ranges)
    }

    // DEPRECATED STUB METHODS - For backwards compatibility during migration to event-driven architecture
    // TODO: Replace all calls to these methods with event-driven communication
    
    /// DEPRECATED: Use AtomHistoryRequest event instead
    pub fn get_atom_history(&self, _aref_uuid: &str) -> Result<Vec<crate::atom::Atom>, Box<dyn std::error::Error>> {
        Err("Method deprecated: Use event-driven AtomHistoryRequest via message bus instead of direct method calls".into())
    }

    /// DEPRECATED: Use AtomGetRequest event instead
    pub fn get_latest_atom(&self, _aref_uuid: &str) -> Result<crate::atom::Atom, Box<dyn std::error::Error>> {
        Err("Method deprecated: Use event-driven AtomGetRequest via message bus instead of direct method calls".into())
    }

    /// DEPRECATED: Use AtomCreateRequest event instead
    pub fn create_atom(
        &self,
        schema_name: &str,
        source_pub_key: String,
        prev_atom_uuid: Option<String>,
        content: serde_json::Value,
        status: Option<crate::atom::AtomStatus>,
    ) -> Result<crate::atom::Atom, Box<dyn std::error::Error>> {
        self.db_ops.create_atom(schema_name, source_pub_key, prev_atom_uuid, content, status)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// DEPRECATED: Use AtomRefUpdateRequest event instead
    pub fn update_atom_ref(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<crate::atom::AtomRef, Box<dyn std::error::Error>> {
        self.db_ops.update_atom_ref(aref_uuid, atom_uuid, source_pub_key)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// DEPRECATED: Use AtomRefUpdateRequest event instead
    pub fn update_atom_ref_range(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        content_key: String,
        source_pub_key: String,
    ) -> Result<crate::atom::AtomRefRange, Box<dyn std::error::Error>> {
        self.db_ops.update_atom_ref_range(aref_uuid, atom_uuid, content_key, source_pub_key)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// DEPRECATED: Use AtomRefUpdateRequest event instead
    pub fn update_atom_ref_collection(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        id: String,
        source_pub_key: String,
    ) -> Result<crate::atom::AtomRefCollection, Box<dyn std::error::Error>> {
        self.db_ops.update_atom_ref_collection(aref_uuid, atom_uuid, id, source_pub_key)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    // Note: All direct method access has been removed to enforce pure event-driven architecture.
    // Components must now communicate exclusively through request/response events via the message bus.
    // Use AtomCreateRequest, AtomUpdateRequest, AtomRefUpdateRequest, etc. for all operations.
}

impl Clone for AtomManager {
    fn clone(&self) -> Self {
        Self {
            db_ops: Arc::clone(&self.db_ops),
            atoms: Arc::clone(&self.atoms),
            ref_atoms: Arc::clone(&self.ref_atoms),
            ref_collections: Arc::clone(&self.ref_collections),
            ref_ranges: Arc::clone(&self.ref_ranges),
            message_bus: Arc::clone(&self.message_bus),
            stats: Arc::clone(&self.stats),
            event_threads: Arc::clone(&self.event_threads),
        }
    }
}
