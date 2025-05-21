use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use serde_json::Value as JsonValue;
use log::{info, error};

use crate::schema::SchemaError;
use super::transform_manager::types::TransformRunner;

/// Orchestrates execution of transforms sequentially.
#[derive(Debug)]
struct QueueItem {
    id: String,
    mutation_hash: String,
}

struct QueueState {
    queue: VecDeque<QueueItem>,
    queued: HashSet<String>,
    processed: HashSet<String>,
}

pub struct TransformOrchestrator {
    queue: Mutex<QueueState>,
    manager: Arc<dyn TransformRunner>,
}

impl TransformOrchestrator {
    pub fn new(manager: Arc<dyn TransformRunner>) -> Self {
        Self {
            queue: Mutex::new(QueueState {
                queue: VecDeque::new(),
                queued: HashSet::new(),
                processed: HashSet::new(),
            }),
            manager,
        }
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
        
        Ok(())
    }

    /// Process a single task from the queue.
    pub fn process_one(&self) -> Option<Result<JsonValue, SchemaError>> {
        let (transform_id, mutation_hash, already_processed) = {
            let mut q = self
                .queue
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire queue lock".to_string()))
                .ok()?;
            match q.queue.pop_front() {
                Some(item) => {
                    let key = format!("{}|{}", item.id, item.mutation_hash);
                    let processed = q.processed.contains(&key);
                    q.queued.remove(&key);
                    (item.id, item.mutation_hash, processed)
                }
                None => return None,
            }
        };

        if already_processed {
            return Some(Ok(JsonValue::Null));
        }

        info!("Executing transform: {}", transform_id);
        let result = self.manager.execute_transform_now(&transform_id);

        if result.is_ok() {
            let mut q = self
                .queue
                .lock()
                .expect("queue lock");
            q.processed.insert(format!("{}|{}", transform_id, mutation_hash));
        }
        info!("Result for transform {}: {:?}", transform_id, result);
        Some(result)
    }

    /// Process all queued tasks sequentially.
    pub fn process_queue(&self) {
        while self.process_one().is_some() {}
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
