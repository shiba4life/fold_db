use std::sync::{Arc, Mutex};

use fold_node::fold_db_core::transform_manager::types::TransformRunner;
use fold_node::fold_db_core::transform_orchestrator::TransformOrchestrator;
use tempfile::tempdir;
use sled;
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

fn create_orchestrator(manager: Arc<MockTransformManager>) -> (TransformOrchestrator, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let tree = db.open_tree("orchestrator").unwrap();
    (TransformOrchestrator::new(manager, tree), dir)
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
    let (orchestrator, _dir) = create_orchestrator(manager.clone());

    orchestrator.add_task("SchemaA", "field1", "h1").unwrap();

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
    let (orchestrator, _dir) = create_orchestrator(manager.clone());

    orchestrator.add_task("Schema", "a", "h2").unwrap();
    orchestrator.add_task("Schema", "b", "h2").unwrap();
    orchestrator.add_task("Schema", "c", "h2").unwrap();

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
    let (orchestrator, _dir) = create_orchestrator(manager.clone());

    orchestrator.add_task("SchemaA", "field", "h3").unwrap();
    assert_eq!(orchestrator.len().unwrap(), 1);

    orchestrator.process_queue();

    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 1);
    assert_eq!(exec[0], "SchemaB.other");
}

#[test]
fn duplicate_ids_are_deduped() {
    let mgr = MockTransformManager::new();
    let manager = Arc::new(mgr);
    let (orchestrator, _dir) = create_orchestrator(manager.clone());

    orchestrator.add_transform("T1", "h4").unwrap();
    orchestrator.add_transform("T1", "h4").unwrap();
    assert_eq!(orchestrator.len().unwrap(), 1);

    orchestrator.process_queue();
    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 1);
    assert_eq!(exec[0], "T1");
}

#[test]
fn processed_prevents_rerun() {
    let mgr = MockTransformManager::new();
    let manager = Arc::new(mgr);
    let (orchestrator, _dir) = create_orchestrator(manager.clone());

    orchestrator.add_transform("T2", "h5").unwrap();
    orchestrator.process_queue();

    // Queue again with same hash
    orchestrator.add_transform("T2", "h5").unwrap();
    orchestrator.process_queue();

    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 1);
    drop(exec);

    // New hash should trigger execution
    orchestrator.add_transform("T2", "h6").unwrap();
    orchestrator.process_queue();
    let exec = manager.executed.lock().unwrap();
    assert_eq!(exec.len(), 2);
}

#[test]
fn state_persists_on_disk() {
    let mgr = MockTransformManager::new();
    let manager = Arc::new(mgr);
    let dir = tempdir().unwrap();
    {
        let db = sled::open(dir.path()).unwrap();
        let tree = db.open_tree("orchestrator").unwrap();
        let orchestrator = TransformOrchestrator::new(manager.clone(), tree);
        orchestrator.add_transform("T3", "h7").unwrap();
        assert_eq!(orchestrator.len().unwrap(), 1);
    }

    let db = sled::open(dir.path()).unwrap();
    let tree = db.open_tree("orchestrator").unwrap();
    let orchestrator = TransformOrchestrator::new(manager.clone(), tree);
    assert_eq!(orchestrator.len().unwrap(), 1);
}
