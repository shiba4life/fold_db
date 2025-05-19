use std::sync::{Arc, Mutex};

use fold_node::fold_db_core::transform_orchestrator::{TransformOrchestrator, TransformRunner};
use fold_node::schema::SchemaError;
use serde_json::json;

use std::collections::{HashMap, HashSet};

struct MockTransformManager {
    executed: Arc<Mutex<Vec<String>>>,
    lookup: Arc<Mutex<Vec<(String, String)>>>,
    field_map: HashMap<String, Vec<String>>,
}

impl MockTransformManager {
    fn new() -> Self {
        Self {
            executed: Arc::new(Mutex::new(Vec::new())),
            lookup: Arc::new(Mutex::new(Vec::new())),
            field_map: HashMap::new(),
        }
    }
}

impl TransformRunner for MockTransformManager {
    fn execute_transform_now(&self, transform_id: &str) -> Result<serde_json::Value, SchemaError> {
        self.executed.lock().unwrap().push(transform_id.to_string());
        Ok(json!(null))
    }

    fn transform_exists(&self, _transform_id: &str) -> Result<bool, SchemaError> {
        Ok(true)
    }

    fn get_transforms_for_field(&self, schema_name: &str, field_name: &str) -> Result<HashSet<String>, SchemaError> {
        self.lookup.lock().unwrap().push((schema_name.to_string(), field_name.to_string()));
        Ok(
            self.field_map
                .get(&format!("{}.{}", schema_name, field_name))
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
        )
    }
}

#[test]
fn field_update_adds_to_queue() {
    let mut mgr = MockTransformManager::new();
    mgr.field_map.insert("SchemaA.field1".to_string(), vec!["SchemaA.field1".to_string()]);
    let manager = Arc::new(mgr);
    let orchestrator = TransformOrchestrator::new(manager.clone());

    orchestrator.add_task("SchemaA", "field1").unwrap();

    assert_eq!(orchestrator.len().unwrap(), 1);
    let calls = manager.lookup.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], ("SchemaA".to_string(), "field1".to_string()));
}

#[test]
fn sequential_processing_of_tasks() {
    let mut mgr = MockTransformManager::new();
    mgr.field_map.insert("Schema.a".to_string(), vec!["Schema.a".to_string()]);
    mgr.field_map.insert("Schema.b".to_string(), vec!["Schema.b".to_string()]);
    mgr.field_map.insert("Schema.c".to_string(), vec!["Schema.c".to_string()]);
    let manager = Arc::new(mgr);
    let orchestrator = TransformOrchestrator::new(manager.clone());

    orchestrator.add_task("Schema", "a").unwrap();
    orchestrator.add_task("Schema", "b").unwrap();
    orchestrator.add_task("Schema", "c").unwrap();

    orchestrator.process_queue();

    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 3);
    assert_eq!(exec[0], "Schema.a");
    assert_eq!(exec[1], "Schema.b");
    assert_eq!(exec[2], "Schema.c");
    assert_eq!(orchestrator.len().unwrap(), 0);
}

#[test]
fn mapping_adds_specific_transform() {
    let mut mgr = MockTransformManager::new();
    mgr.field_map.insert("SchemaA.field".to_string(), vec!["SchemaB.other".to_string()]);
    let manager = Arc::new(mgr);
    let orchestrator = TransformOrchestrator::new(manager.clone());

    orchestrator.add_task("SchemaA", "field").unwrap();
    assert_eq!(orchestrator.len().unwrap(), 1);

    orchestrator.process_queue();

    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 1);
    assert_eq!(exec[0], "SchemaB.other");
}
