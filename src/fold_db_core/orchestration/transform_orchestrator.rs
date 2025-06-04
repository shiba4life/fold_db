use serde::{Deserialize, Serialize};
use sled::Tree;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

use log::{error, info};
use serde_json::Value as JsonValue;

use crate::fold_db_core::transform_manager::types::TransformRunner;
use crate::fold_db_core::infrastructure::message_bus::{MessageBus, TransformTriggered, TransformExecuted, FieldValueSet};
use crate::schema::SchemaError;

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

        // Set up automatic transform triggering based on field changes
        let field_value_consumer_thread = Self::setup_field_value_monitoring(
            Arc::clone(&message_bus),
            Arc::clone(&manager),
        );

        Self {
            queue: Mutex::new(state),
            manager,
            tree,
            message_bus,
            _field_value_consumer_thread: Some(field_value_consumer_thread),
        }
    }

    /// Set up monitoring of FieldValueSet events to automatically trigger transforms
    fn setup_field_value_monitoring(
        message_bus: Arc<MessageBus>,
        manager: Arc<dyn TransformRunner>,
    ) -> thread::JoinHandle<()> {
        let mut field_value_consumer = message_bus.subscribe::<FieldValueSet>();
        
        thread::spawn(move || {
            info!("🔍 TransformOrchestrator: Starting automatic transform triggering based on field changes");
            
            loop {
                match field_value_consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(event) => {
                        info!(
                            "🎯 TransformOrchestrator: Field value set detected - field: {}, source: {}",
                            event.field, event.source
                        );
                        
                        // Parse schema.field from the field path
                        if let Some((schema_name, field_name)) = event.field.split_once('.') {
                            // Check if any transforms should be triggered for this field
                            match manager.get_transforms_for_field(schema_name, field_name) {
                                Ok(transform_ids) => {
                                    if !transform_ids.is_empty() {
                                        info!(
                                            "🚀 TransformOrchestrator: Found {} transforms triggered by {}.{}: {:?}",
                                            transform_ids.len(), schema_name, field_name, transform_ids
                                        );
                                        
                                        // Publish TransformTriggered events for each transform
                                        for transform_id in transform_ids {
                                            let triggered_event = TransformTriggered::new(&transform_id);
                                            if let Err(e) = message_bus.publish(triggered_event) {
                                                error!(
                                                    "Failed to publish TransformTriggered event for {}: {}",
                                                    transform_id, e
                                                );
                                            } else {
                                                info!(
                                                    "📢 TransformOrchestrator: Published TransformTriggered event for {}",
                                                    transform_id
                                                );
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to get transforms for field {}.{}: {}",
                                        schema_name, field_name, e
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

    fn persist_state(&self) -> Result<(), SchemaError> {
        let state = {
            let q = self.queue.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire queue lock".to_string())
            })?;
            serde_json::to_vec(&*q).map_err(|e| {
                SchemaError::InvalidData(format!("Failed to serialize state: {}", e))
            })?
        };
        self.tree.insert("state", state).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to persist orchestrator state: {}", e))
        })?;
        self.tree.flush().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to flush orchestrator state: {}", e))
        })?;
        Ok(())
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
        info!("Attempting to add transform to queue: {}", transform_id);

        // Verify the transform exists
        match self.manager.transform_exists(transform_id) {
            Ok(exists) => {
                if !exists {
                    error!("Transform not found: {}", transform_id);
                    return Err(SchemaError::InvalidData(format!(
                        "Transform '{}' not found",
                        transform_id
                    )));
                }
            }
            Err(e) => {
                error!("Error checking transform existence: {}", e);
                return Err(e);
            }
        }

        let mut q = self.queue.lock().map_err(|e| {
            error!("Failed to acquire queue lock: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;

        info!("Adding transform {} to queue", transform_id);
        let key = format!("{}|{}", transform_id, mutation_hash);
        if q.queued.insert(key.clone()) {
            q.queue.push_back(QueueItem {
                id: transform_id.to_string(),
                mutation_hash: mutation_hash.to_string(),
            });
        }

        // Log queue state
        info!("Current queue length: {}", q.queue.len());
        info!("Queue contents: {:?}", q.queue);

        drop(q);
        self.persist_state()?;

        Ok(())
    }

    /// Process a single task from the queue.
    pub fn process_one(&self) -> Option<Result<JsonValue, SchemaError>> {
        info!("🔄 PROCESS_ONE START - checking queue for items to process");

        let (transform_id, mutation_hash, already_processed) = {
            let mut q = self
                .queue
                .lock()
                .map_err(|_| {
                    error!("❌ Failed to acquire queue lock in process_one");
                    SchemaError::InvalidData("Failed to acquire queue lock".to_string())
                })
                .ok()?;

            info!(
                "📋 Queue state - length: {}, items: {:?}",
                q.queue.len(),
                q.queue
            );

            match q.queue.pop_front() {
                Some(item) => {
                    let key = format!("{}|{}", item.id, item.mutation_hash);
                    let processed = q.processed.contains(&key);
                    q.queued.remove(&key);
                    info!(
                        "📤 Popped item from queue: {} (mutation_hash: {}, already_processed: {})",
                        item.id, item.mutation_hash, processed
                    );
                    (item.id, item.mutation_hash, processed)
                }
                None => {
                    info!("📭 Queue is empty, nothing to process");
                    return None;
                }
            }
        };

        info!("💾 Persisting orchestrator state before processing");
        if let Err(e) = self.persist_state() {
            error!("❌ Failed to persist state: {:?}", e);
            return Some(Err(SchemaError::InvalidData(
                "Failed to persist state".to_string(),
            )));
        }

        if already_processed {
            info!("⏭️  Transform {} already processed, skipping", transform_id);
            return Some(Ok(JsonValue::Null));
        }

        info!("🚀 EXECUTING TRANSFORM: {}", transform_id);
        let result = self.manager.execute_transform_now(&transform_id);

        match &result {
            Ok(value) => {
                info!(
                    "✅ Transform {} executed successfully: {:?}",
                    transform_id, value
                );
                let mut q = self.queue.lock().expect("queue lock");
                q.processed
                    .insert(format!("{}|{}", transform_id, mutation_hash));
                drop(q);
                if let Err(e) = self.persist_state() {
                    error!(
                        "❌ Failed to persist state after successful transform: {:?}",
                        e
                    );
                    return Some(Err(e));
                }

                // Publish TransformExecuted event for successful execution
                let event = TransformExecuted::new(&transform_id, "success");
                if let Err(e) = self.message_bus.publish(event) {
                    error!("Failed to publish TransformExecuted event for {}: {}", transform_id, e);
                } else {
                    info!("📢 Published TransformExecuted success event for transform: {}", transform_id);
                }
            }
            Err(e) => {
                error!("❌ Transform {} failed: {:?}", transform_id, e);
                
                // Publish TransformExecuted event for failed execution
                let event = TransformExecuted::new(&transform_id, "failed");
                if let Err(publish_err) = self.message_bus.publish(event) {
                    error!("Failed to publish TransformExecuted event for {}: {}", transform_id, publish_err);
                } else {
                    info!("📢 Published TransformExecuted failure event for transform: {}", transform_id);
                }
            }
        }

        info!(
            "🏁 PROCESS_ONE COMPLETE - transform: {}, result: {:?}",
            transform_id, result
        );
        Some(result)
    }

    /// Process all queued tasks sequentially.
    pub fn process_queue(&self) {
        info!("🔄 PROCESS_QUEUE START - beginning to process all queued transforms");
        let mut processed_count = 0;

        while let Some(result) = self.process_one() {
            processed_count += 1;
            match result {
                Ok(value) => info!("✅ Processed transform #{}: {:?}", processed_count, value),
                Err(e) => error!(
                    "❌ Failed to process transform #{}: {:?}",
                    processed_count, e
                ),
            }
        }

        info!(
            "🏁 PROCESS_QUEUE COMPLETE - processed {} transforms",
            processed_count
        );
    }

    /// List queued transform IDs without dequeuing or running them.
    pub fn list_queued_transforms(&self) -> Result<Vec<String>, SchemaError> {
        let q = self.queue.lock().map_err(|e| {
            error!("Failed to acquire queue lock: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;
        Ok(q.queue.iter().map(|item| item.id.clone()).collect())
    }

    /// Queue length, useful for tests.
    pub fn len(&self) -> Result<usize, SchemaError> {
        let q = self.queue.lock().map_err(|e| {
            error!("Failed to acquire queue lock: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;
        let length = q.queue.len();
        info!("Queue length: {}", length);
        Ok(length)
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> Result<bool, SchemaError> {
        let q = self.queue.lock().map_err(|e| {
            error!("Failed to acquire queue lock: {}", e);
            SchemaError::InvalidData("Failed to acquire queue lock".to_string())
        })?;
        let empty = q.queue.is_empty();
        info!("Queue is empty: {}", empty);
        Ok(empty)
    }
}
