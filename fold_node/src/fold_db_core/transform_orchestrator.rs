use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde_json::Value as JsonValue;
use log::{info, error};

use crate::schema::SchemaError;
use super::transform_manager::types::TransformRunner;

/// Orchestrates execution of transforms sequentially.
pub struct TransformOrchestrator {
    queue: Mutex<VecDeque<String>>,
    manager: Arc<dyn TransformRunner>,
}

impl TransformOrchestrator {
    pub fn new(manager: Arc<dyn TransformRunner>) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            manager,
        }
    }

    /// Add a task for the given schema and field.
    pub fn add_task(&self, schema_name: &str, field_name: &str) -> Result<(), SchemaError> {
        let ids = self.manager.get_transforms_for_field(schema_name, field_name)?;
        if ids.is_empty() {
            return Ok(());
        }
        let mut q = self
            .queue
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire queue lock".to_string()))?;
        for id in ids {
            q.push_back(id);
        }
        Ok(())
    }

    /// Add a transform directly to the queue by ID.
    pub fn add_transform(&self, transform_id: &str) -> Result<(), SchemaError> {
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
        q.push_back(transform_id.to_string());
        
        // Log queue state
        info!("Current queue length: {}", q.len());
        info!("Queue contents: {:?}", q);
        
        Ok(())
    }

    /// Process a single task from the queue.
    pub fn process_one(&self) -> Option<Result<JsonValue, SchemaError>> {
        let transform_id = {
            let mut q = self
                .queue
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire queue lock".to_string())).ok()?;
            q.pop_front()
        }?;
        Some(self.manager.execute_transform_now(&transform_id))
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
        let length = q.len();
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
        let empty = q.is_empty();
        info!("Queue is empty: {}", empty);
        Ok(empty)
    }
}
