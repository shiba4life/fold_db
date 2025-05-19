use std::collections::{VecDeque, HashSet};
use std::sync::{Arc, Mutex};

use serde_json::Value as JsonValue;

use crate::schema::SchemaError;

/// Trait abstraction over transform execution for easier testing.
pub trait TransformRunner: Send + Sync {
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError>;
    fn transform_exists(&self, transform_id: &str) -> bool;
    fn get_transforms_for_field(&self, schema_name: &str, field_name: &str) -> HashSet<String>;
}

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
    pub fn add_task(&self, schema_name: &str, field_name: &str) {
        let ids = self.manager.get_transforms_for_field(schema_name, field_name);
        if ids.is_empty() {
            return;
        }

        let mut q = self.queue.lock().unwrap();
        for id in ids {
            q.push_back(id);
        }
    }

    /// Process a single task from the queue.
    pub fn process_one(&self) -> Option<Result<JsonValue, SchemaError>> {
        let transform_id = {
            let mut q = self.queue.lock().unwrap();
            q.pop_front()
        }?;
        Some(self.manager.execute_transform_now(&transform_id))
    }

    /// Process all queued tasks sequentially.
    pub fn process_queue(&self) {
        while self.process_one().is_some() {}
    }

    /// Queue length, useful for tests.
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}
