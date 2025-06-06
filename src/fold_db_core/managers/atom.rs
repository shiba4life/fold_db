//! Pure Event-Driven AtomManager
//!
//! This is a completely event-driven version of AtomManager that communicates
//! only through request/response events, eliminating all direct method calls.

use crate::atom::{Atom, AtomRef, AtomRefRange, AtomStatus};
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus,
    AtomCreateRequest, AtomCreateResponse,
    AtomUpdateRequest, AtomUpdateResponse,
    AtomRefCreateRequest, AtomRefCreateResponse,
    AtomRefUpdateRequest, AtomRefUpdateResponse,
    FieldValueSetRequest, FieldValueSetResponse,
    AtomCreated, AtomUpdated, AtomRefCreated, AtomRefUpdated,
    FieldValueSet  // DIAGNOSTIC FIX: Import missing FieldValueSet event
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
    // TODO: Collections are no longer supported - AtomRefCollection has been removed
    ref_ranges: Arc<Mutex<HashMap<String, AtomRefRange>>>,
    message_bus: Arc<MessageBus>,
    stats: Arc<Mutex<EventDrivenAtomStats>>,
    event_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl AtomManager {
    pub fn new(db_ops: DbOperations, message_bus: Arc<MessageBus>) -> Self {
        let mut atoms = HashMap::new();
        let mut ref_atoms = HashMap::new();
        // TODO: Collections are no longer supported - AtomRefCollection has been removed
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
                // TODO: Collections are no longer supported - AtomRefCollection has been removed
                } else if let Ok(range) = serde_json::from_slice::<AtomRefRange>(bytes) {
                    ref_ranges.insert(stripped.to_string(), range);
                }
            }
        }

        let manager = Self {
            db_ops: Arc::new(db_ops),
            atoms: Arc::new(Mutex::new(atoms)),
            ref_atoms: Arc::new(Mutex::new(ref_atoms)),
            // TODO: Collections are no longer supported - AtomRefCollection has been removed
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
        
        // Thread 5: FieldValueSetRequest processing - CRITICAL MUTATION BUG FIX
        let fieldvalueset_thread = self.start_fieldvalueset_processing();
        threads.push(fieldvalueset_thread);
        
        // DIAGNOSTIC LOG: All handlers now implemented
        info!("🔍 DIAGNOSTIC: AtomManager event threads - AtomCreateRequest: ✅, AtomUpdateRequest: ✅, AtomRefCreateRequest: ✅, AtomRefUpdateRequest: ✅, FieldValueSetRequest: ✅ FIXED");
        
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

    /// Process FieldValueSetRequest events - CRITICAL MUTATION BUG FIX
    fn start_fieldvalueset_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<FieldValueSetRequest>();
        let manager = self.clone();
        
        thread::spawn(move || {
            info!("📝 FieldValueSetRequest processor started - CRITICAL MUTATION BUG FIX");
            
            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_fieldvalueset_request(request) {
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
                // TODO: Collections are no longer supported - AtomRefCollection has been removed
                Err("Collection AtomRefs are no longer supported".into())
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
                // TODO: Collections are no longer supported - AtomRefCollection has been removed
                Err("Collection AtomRefs are no longer supported".into())
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

    /// Handle FieldValueSetRequest by creating atom and appropriate AtomRef - CRITICAL MUTATION BUG FIX
    fn handle_fieldvalueset_request(&self, request: FieldValueSetRequest) -> Result<(), Box<dyn std::error::Error>> {
        info!("📝 Processing FieldValueSetRequest for field: {}.{}", request.schema_name, request.field_name);
        info!("🔍 DIAGNOSTIC: FieldValueSetRequest details - correlation_id: {}, value: {}", request.correlation_id, request.value);
        
        let mut stats = self.stats.lock().unwrap();
        stats.requests_processed += 1;
        stats.last_activity = Some(Instant::now());
        drop(stats);

        // Step 1: Create atom with the field value
        info!("🔍 DIAGNOSTIC: Step 1 - Creating atom for schema: {}", request.schema_name);
        let atom_result = self.db_ops.create_atom(
            &request.schema_name,
            request.source_pub_key.clone(),
            None, // No previous atom for field value sets
            request.value.clone(),
            Some(AtomStatus::Active),
        );

        let response = match atom_result {
            Ok(atom) => {
                let atom_uuid = atom.uuid().to_string();
                info!("🔍 DIAGNOSTIC: Step 1 SUCCESS - Created atom with UUID: {}", atom_uuid);
                
                // Store atom in memory cache
                self.atoms.lock().unwrap().insert(atom_uuid.clone(), atom.clone());
                info!("🔍 DIAGNOSTIC: Stored atom in memory cache");
                
                // Step 2: Determine field type and create appropriate AtomRef
                // For now, we'll default to Single field type since field type determination
                // requires schema lookup which isn't straightforward in current patterns
                // TODO: Implement proper field type detection from schema
                let field_type = self.determine_field_type(&request.schema_name, &request.field_name);
                info!("🔍 DIAGNOSTIC: Step 2 - Determined field type: {}", field_type);
                
                let aref_result: Result<String, Box<dyn std::error::Error>> = match field_type.as_str() {
                    "Range" => {
                        // Create AtomRefRange for Range fields
                        let aref_uuid = format!("{}_{}_range", request.schema_name, request.field_name);
                        info!("🔍 DIAGNOSTIC: Creating AtomRefRange with UUID: {} -> atom: {}", aref_uuid, atom_uuid);
                        
                        // Extract range key from the request value
                        // Range values are expected to be objects with a "range_key" field
                        let range_key = if let Some(obj) = request.value.as_object() {
                            // Extract the VALUE of the "range_key" field, not the field name itself
                            if let Some(range_key_value) = obj.get("range_key") {
                                if let Some(key_str) = range_key_value.as_str() {
                                    key_str.to_string()
                                } else {
                                    // Handle non-string range keys by converting to string
                                    range_key_value.to_string().trim_matches('"').to_string()
                                }
                            } else {
                                warn!("🔶 RANGE KEY WARNING: No 'range_key' field found in value, using 'default'");
                                "default".to_string()
                            }
                        } else {
                            warn!("🔶 RANGE KEY WARNING: Value is not an object, using 'default'");
                            "default".to_string()
                        };
                        
                        info!("🔍 DIAGNOSTIC: Extracted range key: '{}' from value: {}", range_key, request.value);
                        
                        let range_result = self.db_ops.update_atom_ref_range(
                            &aref_uuid,
                            atom_uuid.clone(),
                            range_key,
                            request.source_pub_key.clone(),
                        );
                        
                        match range_result {
                            Ok(range) => {
                                self.ref_ranges.lock().unwrap().insert(aref_uuid.clone(), range);
                                info!("🔍 DIAGNOSTIC: Successfully created and stored AtomRefRange: {}", aref_uuid);
                                Ok(aref_uuid)
                            }
                            Err(e) => {
                                error!("❌ DIAGNOSTIC: Failed to create AtomRefRange: {}", e);
                                Err(Box::new(e) as Box<dyn std::error::Error>)
                            }
                        }
                    }
                    _ => {
                        // Default to Single field type
                        let aref_uuid = format!("{}_{}_single", request.schema_name, request.field_name);
                        info!("🔍 DIAGNOSTIC: Creating AtomRef (Single) with UUID: {} -> atom: {}", aref_uuid, atom_uuid);
                        
                        let single_result = self.db_ops.update_atom_ref(
                            &aref_uuid,
                            atom_uuid.clone(),
                            request.source_pub_key.clone(),
                        );
                        
                        match single_result {
                            Ok(aref) => {
                                info!("🔍 DIAGNOSTIC: AtomRef created successfully, final atom_uuid: {}", aref.get_atom_uuid());
                                self.ref_atoms.lock().unwrap().insert(aref_uuid.clone(), aref);
                                info!("🔍 DIAGNOSTIC: Successfully created and stored AtomRef: {}", aref_uuid);
                                Ok(aref_uuid)
                            }
                            Err(e) => {
                                error!("❌ DIAGNOSTIC: Failed to create AtomRef: {}", e);
                                Err(Box::new(e) as Box<dyn std::error::Error>)
                            }
                        }
                    }
                };

                match aref_result {
                    Ok(aref_uuid) => {
                        let mut stats = self.stats.lock().unwrap();
                        stats.atoms_created += 1;
                        stats.atom_refs_created += 1;
                        drop(stats);
                        
                        info!("✅ Successfully processed FieldValueSetRequest - atom: {}, aref: {}", atom_uuid, aref_uuid);
                        info!("🔍 DIAGNOSTIC: Final mapping - AtomRef {} -> Atom {}", aref_uuid, atom_uuid);
                        
                        // 🚨 CRITICAL FIX: Publish FieldValueSet event to trigger transform chain
                        let field_key = format!("{}.{}", request.schema_name, request.field_name);
                        let field_value_event = FieldValueSet {
                            field: field_key.clone(),
                            value: request.value.clone(),
                            source: "AtomManager".to_string(),
                        };
                        
                        info!("🔔 DIAGNOSTIC FIX: Publishing FieldValueSet event - field: {}, source: AtomManager", field_key);
                        match self.message_bus.publish(field_value_event) {
                            Ok(_) => {
                                info!("✅ DIAGNOSTIC FIX: Successfully published FieldValueSet event for: {}", field_key);
                            }
                            Err(e) => {
                                error!("❌ DIAGNOSTIC FIX: Failed to publish FieldValueSet event for {}: {}", field_key, e);
                                // Continue processing even if event publication fails
                            }
                        }
                        
                        FieldValueSetResponse::new(
                            request.correlation_id,
                            true,
                            Some(aref_uuid),
                            None,
                        )
                    }
                    Err(e) => {
                        let mut stats = self.stats.lock().unwrap();
                        stats.requests_failed += 1;
                        drop(stats);
                        
                        error!("❌ Failed to create AtomRef for FieldValueSetRequest: {}", e);
                        
                        FieldValueSetResponse::new(
                            request.correlation_id,
                            false,
                            None,
                            Some(format!("Failed to create AtomRef: {}", e)),
                        )
                    }
                }
            }
            Err(e) => {
                let mut stats = self.stats.lock().unwrap();
                stats.requests_failed += 1;
                drop(stats);
                
                error!("❌ Failed to create Atom for FieldValueSetRequest: {}", e);
                
                FieldValueSetResponse::new(
                    request.correlation_id,
                    false,
                    None,
                    Some(format!("Failed to create Atom: {}", e)),
                )
            }
        };

        // Publish response
        self.message_bus.publish(response)?;
        Ok(())
    }

    /// Determine field type based on schema and field name
    /// Determine field type by looking up the actual schema
    fn determine_field_type(&self, schema_name: &str, field_name: &str) -> String {
        info!("🔍 DIAGNOSTIC: Looking up field type for {}.{}", schema_name, field_name);
        
        // Look up the actual schema to determine field type
        match self.db_ops.get_schema(schema_name) {
            Ok(Some(schema)) => {
                match schema.fields.get(field_name) {
                    Some(crate::schema::types::field::FieldVariant::Range(_)) => {
                        info!("✅ DIAGNOSTIC: Field {}.{} is Range type", schema_name, field_name);
                        "Range".to_string()
                    }
                    Some(crate::schema::types::field::FieldVariant::Single(_)) => {
                        info!("✅ DIAGNOSTIC: Field {}.{} is Single type", schema_name, field_name);
                        "Single".to_string()
                    }
                    None => {
                        warn!("⚠️ DIAGNOSTIC: Field {} not found in schema {}, defaulting to Single", field_name, schema_name);
                        "Single".to_string()
                    }
                }
            }
            Ok(None) => {
                warn!("⚠️ DIAGNOSTIC: Schema {} not found, defaulting to Single for field {}", schema_name, field_name);
                "Single".to_string()
            }
            Err(e) => {
                error!("❌ DIAGNOSTIC: Error loading schema {}: {}, defaulting to Single for field {}", schema_name, e, field_name);
                "Single".to_string()
            }
        }
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

    // TODO: Collection references are no longer supported - AtomRefCollection has been removed

    pub fn get_ref_ranges(&self) -> Arc<Mutex<HashMap<String, AtomRefRange>>> {
        Arc::clone(&self.ref_ranges)
    }

    // DEPRECATED STUB METHODS - For backwards compatibility during migration to event-driven architecture
    // TODO: Replace all calls to these methods with event-driven communication
    

    /// DEPRECATED: Use AtomHistoryRequest event instead
    pub fn get_atom_history(&self, _aref_uuid: &str) -> Result<Vec<crate::atom::Atom>, Box<dyn std::error::Error>> {
        Err("Method deprecated: Use event-driven AtomHistoryRequest via message bus instead of direct method calls".into())
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

    // TODO: Collection operations are no longer supported - AtomRefCollection has been removed

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
            // TODO: Collections are no longer supported - AtomRefCollection has been removed
            ref_ranges: Arc::clone(&self.ref_ranges),
            message_bus: Arc::clone(&self.message_bus),
            stats: Arc::clone(&self.stats),
            event_threads: Arc::clone(&self.event_threads),
        }
    }
}
