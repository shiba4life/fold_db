use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use sled::Tree;

use serde_json::Value as JsonValue;
use log::{info, error};

use crate::schema::SchemaError;
use super::transform_manager::types::TransformRunner;

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
}

impl TransformOrchestrator {
    pub fn new(manager: Arc<dyn TransformRunner>, tree: Tree) -> Self {
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
        Self {
            queue: Mutex::new(state),
            manager,
            tree,
        }
    }

    fn persist_state(&self) -> Result<(), SchemaError> {
        let state = {
            let q = self
                .queue
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire queue lock".to_string()))?;
            serde_json::to_vec(&*q)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize state: {}", e)))?
        };
        self.tree
            .insert("state", state)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to persist orchestrator state: {}", e)))?;
        self.tree
            .flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush orchestrator state: {}", e)))?;
        Ok(())
    }

    /// Add a task for the given schema and field.
    pub fn add_task(
        &self,
        schema_name: &str,
        field_name: &str,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        let ids = self.manager.get_transforms_for_field(schema_name, field_name)?;
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
                q.queue.push_back(QueueItem { id, mutation_hash: mutation_hash.to_string() });
            }
        }
        drop(q);
        self.persist_state()?;
        Ok(())
    }

    /// Add a transform directly to the queue by ID.
    pub fn add_transform(&self, transform_id: &str, mutation_hash: &str) -> Result<(), SchemaError> {
        info!("Attempting to add transform to queue: {}", transform_id);
        
        // Verify the transform exists
        match self.manager.transform_exists(transform_id) {
            Ok(exists) => {
                if !exists {
                    error!("Transform not found: {}", transform_id);
                    return Err(SchemaError::InvalidData(format!("Transform '{}' not found", transform_id)));
                }
            }
            Err(e) => {
                error!("Error checking transform existence: {}", e);
                return Err(e);
            }
        }

        let mut q = self
            .queue
            .lock()
            .map_err(|e| {
                error!("Failed to acquire queue lock: {}", e);
                SchemaError::InvalidData("Failed to acquire queue lock".to_string())
            })?;

        info!("Adding transform {} to queue", transform_id);
        let key = format!("{}|{}", transform_id, mutation_hash);
        if q.queued.insert(key.clone()) {
            q.queue.push_back(QueueItem { id: transform_id.to_string(), mutation_hash: mutation_hash.to_string() });
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
            
            info!("📋 Queue state - length: {}, items: {:?}", q.queue.len(), q.queue);
            
            match q.queue.pop_front() {
                Some(item) => {
                    let key = format!("{}|{}", item.id, item.mutation_hash);
                    let processed = q.processed.contains(&key);
                    q.queued.remove(&key);
                    info!("📤 Popped item from queue: {} (mutation_hash: {}, already_processed: {})",
                          item.id, item.mutation_hash, processed);
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
            return Some(Err(SchemaError::InvalidData("Failed to persist state".to_string())));
        }

        if already_processed {
            info!("⏭️  Transform {} already processed, skipping", transform_id);
            return Some(Ok(JsonValue::Null));
        }

        info!("🚀 EXECUTING TRANSFORM: {}", transform_id);
        let result = self.manager.execute_transform_now(&transform_id);

        match &result {
            Ok(value) => {
                info!("✅ Transform {} executed successfully: {:?}", transform_id, value);
                let mut q = self
                    .queue
                    .lock()
                    .expect("queue lock");
                q.processed.insert(format!("{}|{}", transform_id, mutation_hash));
                drop(q);
                if let Err(e) = self.persist_state() {
                    error!("❌ Failed to persist state after successful transform: {:?}", e);
                    return Some(Err(e));
                }
            }
            Err(e) => {
                error!("❌ Transform {} failed: {:?}", transform_id, e);
            }
        }
        
        info!("🏁 PROCESS_ONE COMPLETE - transform: {}, result: {:?}", transform_id, result);
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
                Err(e) => error!("❌ Failed to process transform #{}: {:?}", processed_count, e),
            }
        }
        
        info!("🏁 PROCESS_QUEUE COMPLETE - processed {} transforms", processed_count);
    }

    /// Queue length, useful for tests.
    pub fn len(&self) -> Result<usize, SchemaError> {
        let q = self
            .queue
            .lock()
            .map_err(|e| {
                error!("Failed to acquire queue lock: {}", e);
                SchemaError::InvalidData("Failed to acquire queue lock".to_string())
            })?;
        let length = q.queue.len();
        info!("Queue length: {}", length);
        Ok(length)
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> Result<bool, SchemaError> {
        let q = self
            .queue
            .lock()
            .map_err(|e| {
                error!("Failed to acquire queue lock: {}", e);
                SchemaError::InvalidData("Failed to acquire queue lock".to_string())
            })?;
        let empty = q.queue.is_empty();
        info!("Queue is empty: {}", empty);
        Ok(empty)
    }
}
