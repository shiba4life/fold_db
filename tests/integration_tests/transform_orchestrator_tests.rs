use std::sync::{Arc, Mutex};

use fold_node::fold_db_core::transform_orchestrator::{TransformOrchestrator, TransformRunner};
use fold_node::schema::SchemaError;
use serde_json::json;

struct MockTransformManager {
    executed: Arc<Mutex<Vec<String>>>,
    exists: Arc<Mutex<Vec<String>>>,
}

impl MockTransformManager {
    fn new() -> Self {
        Self {
            executed: Arc::new(Mutex::new(Vec::new())),
            exists: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl TransformRunner for MockTransformManager {
    fn execute_transform_now(&self, transform_id: &str) -> Result<serde_json::Value, SchemaError> {
        self.executed.lock().unwrap().push(transform_id.to_string());
        Ok(json!(null))
    }

    fn transform_exists(&self, transform_id: &str) -> bool {
        self.exists.lock().unwrap().push(transform_id.to_string());
        true
    }
}

#[test]
fn field_update_adds_to_queue() {
    let manager = Arc::new(MockTransformManager::new());
    let orchestrator = TransformOrchestrator::new(manager.clone());

    orchestrator.add_task("SchemaA", "field1");

    assert_eq!(orchestrator.len(), 1);
    // transform_exists should have been called
    let calls = manager.exists.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "SchemaA.field1");
}

#[test]
fn sequential_processing_of_tasks() {
    let manager = Arc::new(MockTransformManager::new());
    let orchestrator = TransformOrchestrator::new(manager.clone());

    orchestrator.add_task("Schema", "a");
    orchestrator.add_task("Schema", "b");
    orchestrator.add_task("Schema", "c");

    orchestrator.process_queue();

    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 3);
    assert_eq!(exec[0], "Schema.a");
    assert_eq!(exec[1], "Schema.b");
    assert_eq!(exec[2], "Schema.c");
    assert_eq!(orchestrator.len(), 0);
}
