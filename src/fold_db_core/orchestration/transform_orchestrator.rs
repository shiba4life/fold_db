use serde::{Deserialize, Serialize};
use sled::Tree;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

use log::{error, info};
use serde_json::Value as JsonValue;

use crate::fold_db_core::transform_manager::types::TransformRunner;
use crate::fold_db_core::infrastructure::message_bus::{MessageBus, TransformExecuted, FieldValueSet};
use crate::schema::SchemaError;

/// Trait for adding transforms to a queue
pub trait TransformQueue {
    fn add_task(&self, schema_name: &str, field_name: &str, mutation_hash: &str) -> Result<(), SchemaError>;
    fn add_transform(&self, transform_id: &str, mutation_hash: &str) -> Result<(), SchemaError>;
}

/// Orchestrates execution of transforms sequentially.
#[derive(Debug, Serialize, Deserialize)]
struct QueueItem {
    id: String,
    mutation_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueueState {
    queue: VecDeque<QueueItem>,
    queued: HashSet<String>,
    processed: HashSet<String>,
}

pub struct TransformOrchestrator {
    queue: Mutex<QueueState>,
    manager: Arc<dyn TransformRunner>,
    tree: Tree,
    message_bus: Arc<MessageBus>,
    /// Thread handle for monitoring FieldValueSet events
    _field_value_consumer_thread: Option<thread::JoinHandle<()>>,
}

impl TransformOrchestrator {
    pub fn new(manager: Arc<dyn TransformRunner>, tree: Tree, message_bus: Arc<MessageBus>) -> Self {
        let state = tree
            .get("state")
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice::<QueueState>(&v).ok())
            .unwrap_or_else(|| QueueState {
                queue: VecDeque::new(),
                queued: HashSet::new(),
                processed: HashSet::new(),
            });

        // Set up direct monitoring of FieldValueSet events
        let field_value_consumer_thread = TransformOrchestrator::setup_field_value_monitoring(
            Arc::clone(&message_bus),
            Arc::clone(&manager),
            tree.clone(),
        );

        Self {
            queue: Mutex::new(state),
            manager,
            tree,
            message_bus,
            _field_value_consumer_thread: Some(field_value_consumer_thread),
        }
    }


    fn persist_state(&self) -> Result<(), SchemaError> {
        info!("💾 PERSIST_STATE START - saving orchestrator state to disk");
        
        let state = {
            info!("🔒 Acquiring queue lock for state persistence");
            let q = self.queue.lock().map_err(|e| {
                error!("❌ Failed to acquire queue lock for persistence: {}", e);
                SchemaError::InvalidData("Failed to acquire queue lock".to_string())
            })?;
            
            info!("📋 Current state to persist - queue length: {}, queued count: {}, processed count: {}",
                q.queue.len(), q.queued.len(), q.processed.len());
            info!("📋 Queue items: {:?}", q.queue);
            info!("📋 Queued set: {:?}", q.queued);
            info!("📋 Processed set: {:?}", q.processed);
            
            serde_json::to_vec(&*q).map_err(|e| {
                error!("❌ Failed to serialize orchestrator state: {}", e);
                SchemaError::InvalidData(format!("Failed to serialize state: {}", e))
            })?
        };
        
        info!("💾 Inserting state into tree (size: {} bytes)", state.len());
        self.tree.insert("state", state).map_err(|e| {
            error!("❌ Failed to insert orchestrator state into tree: {}", e);
            SchemaError::InvalidData(format!("Failed to persist orchestrator state: {}", e))
        })?;
        
        info!("💾 Flushing tree to disk");
        self.tree.flush().map_err(|e| {
            error!("❌ Failed to flush orchestrator state to disk: {}", e);
            SchemaError::InvalidData(format!("Failed to flush orchestrator state: {}", e))
        })?;
        
        info!("✅ PERSIST_STATE COMPLETE - state saved successfully");
        Ok(())
    }

    /// Set up monitoring of FieldValueSet events to directly add transforms to queue
    fn setup_field_value_monitoring(
        message_bus: Arc<MessageBus>,
        manager: Arc<dyn TransformRunner>,
        tree: Tree,
    ) -> thread::JoinHandle<()> {
        let mut field_value_consumer = message_bus.subscribe::<FieldValueSet>();
        
        thread::spawn(move || {
            info!("🔍 TransformOrchestrator: Starting direct monitoring of FieldValueSet events");
            
            loop {
                match field_value_consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(event) => {
                        info!(
                            "🎯 TransformOrchestrator: Field value set detected - field: {}, source: {}",
                            event.field, event.source
                        );
                        
                        // Parse schema.field from the field path
                        if let Some((schema_name, field_name)) = event.field.split_once('.') {
                            // Look up transforms for this field using the manager
                            match manager.get_transforms_for_field(schema_name, field_name) {
                                Ok(transform_ids) => {
                                    if !transform_ids.is_empty() {
                                        info!(
                                            "🔍 Found {} transforms for field {}: {:?}",
                                            transform_ids.len(), event.field, transform_ids
                                        );
                                        
                                        // Load current queue state from persistent storage
                                        let mut current_state = tree
                                            .get("state")
                                            .ok()
                                            .flatten()
                                            .and_then(|v| serde_json::from_slice::<QueueState>(&v).ok())
                                            .unwrap_or_else(|| QueueState {
                                                queue: VecDeque::new(),
                                                queued: HashSet::new(),
                                                processed: HashSet::new(),
                                            });
                                        
                                        // Add transforms directly to queue
                                        for transform_id in &transform_ids {
                                            let key = format!("{}|{}", transform_id, event.source);
                                            if current_state.queued.insert(key.clone()) {
                                                current_state.queue.push_back(QueueItem {
                                                    id: transform_id.clone(),
                                                    mutation_hash: event.source.clone(),
                                                });
                                                info!(
                                                    "✅ Added transform {} to queue for field {}",
                                                    transform_id, event.field
                                                );
                                            }
                                        }
                                        
                                        // Persist updated state
                                        if let Ok(state_bytes) = serde_json::to_vec(&current_state) {
                                            if let Err(e) = tree.insert("state", state_bytes) {
                                                error!("❌ Failed to persist queue state: {}", e);
                                            } else if let Err(e) = tree.flush() {
                                                error!("❌ Failed to flush queue state: {}", e);
                                            }
                                        }
                                        
                                        // Process transforms immediately using the manager
                                        info!("🚀 TransformOrchestrator: Auto-processing {} transforms after field update", transform_ids.len());
                                        for (index, transform_id) in transform_ids.iter().enumerate() {
                                            info!("🔧 Processing transform {}/{}: {}", index + 1, transform_ids.len(), transform_id);
                                            
                                            let execution_start = std::time::Instant::now();
                                            match manager.execute_transform_now(transform_id) {
                                                Ok(result) => {
                                                    let duration = execution_start.elapsed();
                                                    info!("✅ Transform {} executed successfully from FieldValueSet event in {:?}: {}",
                                                        transform_id, duration, result);
                                                    
                                                    // Mark as processed in persistent state
                                                    let key = format!("{}|{}", transform_id, event.source);
                                                    info!("📝 Marking transform as processed with key: {}", key);
                                                    current_state.processed.insert(key.clone());
                                                    
                                                    let items_before = current_state.queue.len();
                                                    current_state.queue.retain(|item| {
                                                        !(item.id == *transform_id && item.mutation_hash == event.source)
                                                    });
                                                    let items_after = current_state.queue.len();
                                                    info!("📋 Removed {} items from queue ({} -> {})",
                                                        items_before - items_after, items_before, items_after);
                                                    
                                                    current_state.queued.remove(&key);
                                                    info!("📋 Removed key from queued set: {}", key);
                                                    
                                                    // Publish TransformExecuted event for successful execution
                                                    let event = TransformExecuted::new(transform_id, "success");
                                                    if let Err(e) = message_bus.publish(event) {
                                                        error!("❌ Failed to publish TransformExecuted success event for {}: {}", transform_id, e);
                                                    } else {
                                                        info!("📢 Published TransformExecuted success event for: {}", transform_id);
                                                    }
                                                }
                                                Err(e) => {
                                                    let duration = execution_start.elapsed();
                                                    error!("❌ Transform {} failed during execution after {:?}: {}", transform_id, duration, e);
                                                    error!("❌ FieldValueSet execution error details: {:?}", e);
                                                    
                                                    // Publish TransformExecuted event for failed execution
                                                    let event = TransformExecuted::new(transform_id, "failed");
                                                    if let Err(publish_err) = message_bus.publish(event) {
                                                        error!("❌ Failed to publish TransformExecuted failure event for {}: {}", transform_id, publish_err);
                                                    } else {
                                                        info!("📢 Published TransformExecuted failure event for: {}", transform_id);
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Persist final state after processing
                                        if let Ok(state_bytes) = serde_json::to_vec(&current_state) {
                                            if let Err(e) = tree.insert("state", state_bytes) {
                                                error!("❌ Failed to persist final queue state: {}", e);
                                            } else if let Err(e) = tree.flush() {
                                                error!("❌ Failed to flush final queue state: {}", e);
                                            }
                                        }
                                    } else {
                                        info!(
                                            "ℹ️ No transforms found for field {}",
                                            event.field
                                        );
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "❌ Failed to get transforms for field {}: {}",
                                        event.field, e
                                    );
                                }
                            }
                        } else {
                            error!(
                                "❌ Invalid field format '{}' - expected 'schema.field'",
                                event.field
                            );
                        }
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

    /// Add a task for the given schema and field.
    pub fn add_task(
        &self,
        schema_name: &str,
        field_name: &str,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        let ids = self
            .manager
            .get_transforms_for_field(schema_name, field_name)?;
        info!(
            "Transforms queued for {}.{}: {:?}",
            schema_name, field_name, ids
        );
        if ids.is_empty() {
            return Ok(());
        }
        let mut q = self
            .queue
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire queue lock".to_string()))?;
        for id in ids {
            let key = format!("{}|{}", id, mutation_hash);
            if q.queued.insert(key.clone()) {
                q.queue.push_back(QueueItem {
                    id,
                    mutation_hash: mutation_hash.to_string(),
                });
            }
        }
        drop(q);
        self.persist_state()?;
        Ok(())
    }

    /// Add a transform directly to the queue by ID.
    pub fn add_transform(
        &self,
        transform_id: &str,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        info!("🚀 ADD_TRANSFORM START - attempting to add transform to queue: {}", transform_id);
        info!("📋 Transform details - ID: {}, mutation_hash: {}", transform_id, mutation_hash);

        // Verify the transform exists
        info!("🔍 Checking if transform exists: {}", transform_id);
        match self.manager.transform_exists(transform_id) {
            Ok(exists) => {
                if !exists {
                    error!("❌ Transform not found: {}", transform_id);
                    return Err(SchemaError::InvalidData(format!(
                        "Transform '{}' not found",
                        transform_id
                    )));
                } else {
                    info!("✅ Transform exists: {}", transform_id);
                }
            }
            Err(e) => {
                error!("❌ Error checking transform existence for {}: {}", transform_id, e);
                return Err(e);
            }
        }

        info!("🔒 Acquiring queue lock for transform: {}", transform_id);
        let mut q = self.queue.lock().map_err(|e| {
            error!("❌ Failed to acquire queue lock for {}: {}", transform_id, e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;

        info!("📋 Queue state before adding transform {} - length: {}, items: {:?}", transform_id, q.queue.len(), q.queue);
        info!("📋 Queued set before adding: {:?}", q.queued);
        info!("📋 Processed set before adding: {:?}", q.processed);

        let key = format!("{}|{}", transform_id, mutation_hash);
        info!("🔑 Generated key for transform: {}", key);
        
        if q.queued.insert(key.clone()) {
            q.queue.push_back(QueueItem {
                id: transform_id.to_string(),
                mutation_hash: mutation_hash.to_string(),
            });
            info!("✅ Successfully added transform {} to queue with key: {}", transform_id, key);
        } else {
            info!("ℹ️ Transform {} with key {} already in queue, skipping", transform_id, key);
        }

        info!("📋 Queue state after adding transform {} - length: {}, items: {:?}", transform_id, q.queue.len(), q.queue);
        info!("📋 Queued set after adding: {:?}", q.queued);

        drop(q);
        
        info!("💾 Persisting state after adding transform: {}", transform_id);
        if let Err(e) = self.persist_state() {
            error!("❌ Failed to persist state after adding transform {}: {:?}", transform_id, e);
            return Err(e);
        }
        info!("✅ State persisted successfully after adding transform: {}", transform_id);
        
        // Process the queue immediately after adding transforms
        info!("🔄 Triggering automatic queue processing after adding transform: {}", transform_id);
        self.process_queue();
        info!("🏁 Automatic queue processing completed for transform: {}", transform_id);

        info!("🏁 ADD_TRANSFORM COMPLETE - successfully added and processed transform: {}", transform_id);
        Ok(())
    }

    /// Process a single task from the queue.
    pub fn process_one(&self) -> Option<Result<JsonValue, SchemaError>> {
        info!("🔄 PROCESS_ONE START - checking queue for items to process");

        let (transform_id, mutation_hash, already_processed) = {
            info!("🔒 Acquiring queue lock for process_one");
            let mut q = self
                .queue
                .lock()
                .map_err(|e| {
                    error!("❌ Failed to acquire queue lock in process_one: {}", e);
                    SchemaError::InvalidData("Failed to acquire queue lock".to_string())
                })
                .ok()?;

            info!(
                "📋 Queue state before processing - length: {}, items: {:?}",
                q.queue.len(),
                q.queue
            );
            info!("📋 Queued set: {:?}", q.queued);
            info!("📋 Processed set: {:?}", q.processed);

            match q.queue.pop_front() {
                Some(item) => {
                    let key = format!("{}|{}", item.id, item.mutation_hash);
                    let processed = q.processed.contains(&key);
                    let was_in_queued = q.queued.remove(&key);
                    
                    info!(
                        "📤 Popped item from queue: {} (mutation_hash: {}, already_processed: {}, was_in_queued_set: {})",
                        item.id, item.mutation_hash, processed, was_in_queued
                    );
                    info!("🔑 Processing key: {}", key);
                    
                    (item.id, item.mutation_hash, processed)
                }
                None => {
                    info!("📭 Queue is empty, nothing to process");
                    return None;
                }
            }
        };

        info!("💾 Persisting orchestrator state before processing transform: {}", transform_id);
        if let Err(e) = self.persist_state() {
            error!("❌ Failed to persist state before processing {}: {:?}", transform_id, e);
            return Some(Err(SchemaError::InvalidData(
                "Failed to persist state".to_string(),
            )));
        }
        info!("✅ State persisted successfully before processing: {}", transform_id);

        if already_processed {
            info!("⏭️ Transform {} already processed, skipping execution", transform_id);
            return Some(Ok(serde_json::json!({
                "status": "skipped_already_processed",
                "transform_id": transform_id,
                "mutation_hash": mutation_hash
            })));
        }

        info!("🚀 EXECUTING TRANSFORM: {}", transform_id);
        info!("🔧 Calling execute_transform_now with transform_id: {}", transform_id);
        
        // Execute transform directly through TransformManager instance method
        // This ensures database operations are available for the transform
        let execution_start_time = std::time::Instant::now();
        let result = match self.manager.execute_transform_now(&transform_id) {
            Ok(execution_result) => {
                let duration = execution_start_time.elapsed();
                info!("✅ Transform {} executed successfully in {:?}: {}", transform_id, duration, execution_result);
                Ok(serde_json::json!({
                    "status": "executed_from_queue",
                    "transform_id": transform_id,
                    "result": execution_result,
                    "method": "direct_execution",
                    "duration_ms": duration.as_millis(),
                    "mutation_hash": mutation_hash
                }))
            }
            Err(e) => {
                let duration = execution_start_time.elapsed();
                error!("❌ Transform {} failed during execution after {:?}: {}", transform_id, duration, e);
                error!("❌ Execution error details: {:?}", e);
                Err(SchemaError::InvalidData(format!("Transform execution failed: {}", e)))
            }
        };

        // Handle the execution result
        match &result {
            Ok(value) => {
                info!("✅ Transform {} execution completed successfully", transform_id);
                info!("📊 Execution result details: {:?}", value);
                
                // Mark as processed and update queue state
                info!("📝 Marking transform {} as processed", transform_id);
                let process_key = format!("{}|{}", transform_id, mutation_hash);
                {
                    let mut q = self.queue.lock().expect("queue lock for marking processed");
                    q.processed.insert(process_key.clone());
                    info!("✅ Transform {} marked as processed with key: {}", transform_id, process_key);
                    info!("📋 Updated processed set: {:?}", q.processed);
                }
                
                info!("💾 Persisting state after successful transform execution: {}", transform_id);
                if let Err(e) = self.persist_state() {
                    error!("❌ Failed to persist state after successful transform {}: {:?}", transform_id, e);
                    return Some(Err(e));
                }
                info!("✅ State persisted after successful execution: {}", transform_id);

                // Publish TransformExecuted event for successful execution
                info!("📢 Publishing TransformExecuted success event for: {}", transform_id);
                let event = TransformExecuted::new(&transform_id, "success");
                if let Err(e) = self.message_bus.publish(event) {
                    error!("❌ Failed to publish TransformExecuted success event for {}: {}", transform_id, e);
                } else {
                    info!("✅ Published TransformExecuted success event for transform: {}", transform_id);
                }
            }
            Err(e) => {
                error!("❌ Transform {} execution failed", transform_id);
                error!("❌ Failure details: {:?}", e);
                
                // Publish TransformExecuted event for failed execution
                info!("📢 Publishing TransformExecuted failure event for: {}", transform_id);
                let event = TransformExecuted::new(&transform_id, "failed");
                if let Err(publish_err) = self.message_bus.publish(event) {
                    error!("❌ Failed to publish TransformExecuted failure event for {}: {}", transform_id, publish_err);
                } else {
                    info!("✅ Published TransformExecuted failure event for transform: {}", transform_id);
                }
            }
        }

        // Log final queue state after processing
        if let Ok(final_length) = self.len() {
            info!("📊 Queue length after processing {}: {}", transform_id, final_length);
        }

        info!(
            "🏁 PROCESS_ONE COMPLETE - transform: {}, success: {}",
            transform_id, result.is_ok()
        );
        Some(result)
    }

    /// Process all queued tasks sequentially.
    pub fn process_queue(&self) {
        info!("🔄 PROCESS_QUEUE START - beginning to process all queued transforms");
        
        // Check initial queue state
        let initial_queue_length = match self.len() {
            Ok(length) => {
                info!("📊 Initial queue length: {}", length);
                length
            }
            Err(e) => {
                error!("❌ Failed to get initial queue length: {:?}", e);
                return;
            }
        };

        if initial_queue_length == 0 {
            info!("📭 Queue is empty, nothing to process");
            return;
        }

        info!("🚀 Starting to process {} queued transforms", initial_queue_length);
        let mut processed_count = 0;
        let mut iteration_count = 0;

        loop {
            iteration_count += 1;
            info!("🔄 Processing iteration #{}", iteration_count);
            
            // Log queue state before each iteration
            if let Ok(current_length) = self.len() {
                info!("📊 Queue length at iteration #{}: {}", iteration_count, current_length);
            }

            match self.process_one() {
                Some(result) => {
                    processed_count += 1;
                    match result {
                        Ok(value) => {
                            info!("✅ Successfully processed transform #{} in iteration #{}: {:?}", processed_count, iteration_count, value);
                        }
                        Err(e) => {
                            error!("❌ Failed to process transform #{} in iteration #{}: {:?}", processed_count, iteration_count, e);
                        }
                    }
                }
                None => {
                    info!("📭 No more items in queue after iteration #{}", iteration_count);
                    break;
                }
            }

            // Safety check to prevent infinite loops
            if iteration_count > 100 {
                error!("❌ Breaking out of process_queue loop after {} iterations to prevent infinite loop", iteration_count);
                break;
            }
        }

        // Final queue state check
        let final_queue_length = match self.len() {
            Ok(length) => {
                info!("📊 Final queue length: {}", length);
                length
            }
            Err(e) => {
                error!("❌ Failed to get final queue length: {:?}", e);
                0
            }
        };

        info!(
            "🏁 PROCESS_QUEUE COMPLETE - processed {} transforms across {} iterations",
            processed_count, iteration_count
        );
        info!("📈 Queue processing stats - Initial: {}, Final: {}, Processed: {}",
            initial_queue_length, final_queue_length, processed_count);
        
        if final_queue_length > 0 {
            error!("⚠️ WARNING: Queue still contains {} items after processing", final_queue_length);
            if let Ok(remaining_transforms) = self.list_queued_transforms() {
                error!("⚠️ Remaining transforms in queue: {:?}", remaining_transforms);
            }
        }
    }

    /// List queued transform IDs without dequeuing or running them.
    pub fn list_queued_transforms(&self) -> Result<Vec<String>, SchemaError> {
        // info!("📋 LIST_QUEUED_TRANSFORMS - getting current queue contents");
        let q = self.queue.lock().map_err(|e| {
            error!("❌ Failed to acquire queue lock for listing transforms: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;
        
        let transform_ids: Vec<String> = q.queue.iter().map(|item| item.id.clone()).collect();
        // info!("📋 Found {} queued transforms: {:?}", transform_ids.len(), transform_ids);
        Ok(transform_ids)
    }

    /// Queue length, useful for tests.
    pub fn len(&self) -> Result<usize, SchemaError> {
        let q = self.queue.lock().map_err(|e| {
            error!("❌ Failed to acquire queue lock for length check: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;
        let length = q.queue.len();
        info!("📊 Current queue length: {}", length);
        Ok(length)
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> Result<bool, SchemaError> {
        let q = self.queue.lock().map_err(|e| {
            error!("❌ Failed to acquire queue lock for empty check: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;
        let empty = q.queue.is_empty();
        // info!("📊 Queue is empty: {}", empty);
        // if !empty {
        //     info!("📋 Queue contents: {:?}", q.queue);
        // }
        Ok(empty)
    }
}

impl TransformQueue for TransformOrchestrator {
    fn add_task(&self, schema_name: &str, field_name: &str, mutation_hash: &str) -> Result<(), SchemaError> {
        self.add_task(schema_name, field_name, mutation_hash)
    }

    fn add_transform(&self, transform_id: &str, mutation_hash: &str) -> Result<(), SchemaError> {
        self.add_transform(transform_id, mutation_hash)
    }
}
