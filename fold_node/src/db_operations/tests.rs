#[cfg(test)]
mod tests {
    use crate::db_operations::DbOperations;
    use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomRefRange, AtomRefBehavior};
    use crate::schema::core::SchemaState;
    use crate::schema::types::transform::Transform;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        value: String,
    }

    fn create_temp_db() -> DbOperations {
        let db = sled::Config::new().temporary(true).open().unwrap();
        DbOperations::new(db).unwrap()
    }

    #[test]
    fn test_store_and_get_item() {
        let db_ops = create_temp_db();
        let item = TestStruct { value: "hello".to_string() };
        db_ops.store_item("key1", &item).unwrap();
        let retrieved: Option<TestStruct> = db_ops.get_item("key1").unwrap();
        assert_eq!(retrieved, Some(item));
    }

    #[test]
    fn test_create_atom_persists() {
        let db_ops = create_temp_db();
        let content = json!({"field": 1});
        let atom = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, content.clone(), None)
            .unwrap();
        let stored: Option<Atom> = db_ops.get_item(&format!("atom:{}", atom.uuid())).unwrap();
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().content(), &content);
    }

    #[test]
    fn test_update_atom_ref_persists() {
        let db_ops = create_temp_db();
        let atom1 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"v": 1}), None)
            .unwrap();
        let mut aref = db_ops
            .update_atom_ref("ref1", atom1.uuid().to_string(), "owner".to_string())
            .unwrap();
        assert_eq!(aref.get_atom_uuid(), &atom1.uuid().to_string());

        let atom2 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"v": 2}), None)
            .unwrap();
        aref = db_ops
            .update_atom_ref("ref1", atom2.uuid().to_string(), "owner".to_string())
            .unwrap();

        let stored: Option<AtomRef> = db_ops.get_item("ref:ref1").unwrap();
        let stored = stored.unwrap();
        assert_eq!(stored.uuid(), aref.uuid());
        assert_eq!(stored.get_atom_uuid(), &atom2.uuid().to_string());
    }

    #[test]
    fn test_update_atom_ref_collection_persists() {
        let db_ops = create_temp_db();
        let atom1 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"idx": 1}), None)
            .unwrap();
        let atom2 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"idx": 2}), None)
            .unwrap();

        let mut collection = db_ops
            .update_atom_ref_collection(
                "col1",
                atom1.uuid().to_string(),
                "a".to_string(),
                "owner".to_string(),
            )
            .unwrap();
        assert_eq!(collection.get_atom_uuid("a"), Some(&atom1.uuid().to_string()));

        collection = db_ops
            .update_atom_ref_collection(
                "col1",
                atom2.uuid().to_string(),
                "b".to_string(),
                "owner".to_string(),
            )
            .unwrap();

        let stored: Option<AtomRefCollection> = db_ops.get_item("ref:col1").unwrap();
        let stored = stored.unwrap();
        assert_eq!(stored.uuid(), collection.uuid());
        assert_eq!(stored.get_atom_uuid("a"), Some(&atom1.uuid().to_string()));
        assert_eq!(stored.get_atom_uuid("b"), Some(&atom2.uuid().to_string()));
    }

    #[test]
    fn test_update_atom_ref_range_persists() {
        let db_ops = create_temp_db();
        let atom1 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"idx": 1}), None)
            .unwrap();
        let atom2 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"idx": 2}), None)
            .unwrap();

        let mut range = db_ops
            .update_atom_ref_range(
                "range1",
                atom1.uuid().to_string(),
                "a".to_string(),
                "owner".to_string(),
            )
            .unwrap();
        assert_eq!(range.get_atom_uuid("a"), Some(&atom1.uuid().to_string()));

        range = db_ops
            .update_atom_ref_range(
                "range1",
                atom2.uuid().to_string(),
                "b".to_string(),
                "owner".to_string(),
            )
            .unwrap();

        let stored: Option<AtomRefRange> = db_ops.get_item("ref:range1").unwrap();
        let stored = stored.unwrap();
        assert_eq!(stored.uuid(), range.uuid());
        assert_eq!(stored.get_atom_uuid("a"), Some(&atom1.uuid().to_string()));
        assert_eq!(stored.get_atom_uuid("b"), Some(&atom2.uuid().to_string()));
    }

    #[test]
    fn test_persistence_across_reopen() {
        // Use a temporary directory so the DB persists across instances
        let dir = tempfile::tempdir().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let db_ops = DbOperations::new(db).unwrap();

        let atom = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"v": 1}), None)
            .unwrap();
        let _aref = db_ops
            .update_atom_ref("ref_persist", atom.uuid().to_string(), "owner".to_string())
            .unwrap();

        // Drop first instance to close the database
        drop(db_ops);

        // Re-open the database and verify the items exist
        let db2 = sled::open(dir.path()).unwrap();
        let db_ops2 = DbOperations::new(db2).unwrap();
        let stored_atom: Option<Atom> = db_ops2
            .get_item(&format!("atom:{}", atom.uuid()))
            .unwrap();
        let stored_aref: Option<AtomRef> = db_ops2.get_item("ref:ref_persist").unwrap();

        assert!(stored_atom.is_some());
        assert!(stored_aref.is_some());
        assert_eq!(stored_aref.unwrap().get_atom_uuid(), &atom.uuid().to_string());
    }

    #[test]
    fn test_unified_metadata_operations() {
        let db_ops = create_temp_db();
        
        // Test node_id operations
        let node_id = "test-node-123";
        db_ops.set_node_id(node_id).unwrap();
        assert_eq!(db_ops.get_node_id().unwrap(), node_id);
        
        // Test permissions operations
        let schemas = vec!["Schema1".to_string(), "Schema2".to_string()];
        db_ops.set_schema_permissions(node_id, &schemas).unwrap();
        assert_eq!(db_ops.get_schema_permissions(node_id).unwrap(), schemas);
    }

    #[test]
    fn test_unified_schema_operations() {
        let db_ops = create_temp_db();
        
        // Test schema state operations
        db_ops.store_schema_state("TestSchema", SchemaState::Approved).unwrap();
        assert_eq!(
            db_ops.get_schema_state("TestSchema").unwrap(),
            Some(SchemaState::Approved)
        );
        
        let approved_schemas = db_ops.list_schemas_by_state(SchemaState::Approved).unwrap();
        assert!(approved_schemas.contains(&"TestSchema".to_string()));
    }

    #[test]
    fn test_unified_transform_operations() {
        let db_ops = create_temp_db();
        
        // Create a test transform
        let transform = Transform::new(
            "return x + 1".to_string(),
            "test.output".to_string(),
        );
        
        // Store and retrieve transform
        db_ops.store_transform("test_transform", &transform).unwrap();
        let retrieved = db_ops.get_transform("test_transform").unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_transform = retrieved.unwrap();
        assert_eq!(retrieved_transform.logic, "return x + 1");
        assert_eq!(retrieved_transform.output, "test.output");
        
        // Test listing transforms
        let transforms = db_ops.list_transforms().unwrap();
        assert!(transforms.contains(&"test_transform".to_string()));
        
        // Test deleting transform
        db_ops.delete_transform("test_transform").unwrap();
        assert!(db_ops.get_transform("test_transform").unwrap().is_none());
    }

    #[test]
    fn test_unified_orchestrator_operations() {
        let db_ops = create_temp_db();
        
        // Test storing and retrieving orchestrator state
        let state = json!({"status": "running", "queue_size": 5});
        db_ops.store_orchestrator_state("queue_state", &state).unwrap();
        
        let retrieved: Option<serde_json::Value> = db_ops.get_orchestrator_state("queue_state").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), state);
    }

    #[test]
    fn test_batch_operations() {
        let db_ops = create_temp_db();
        
        // Test batch store
        let items = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
            ("key3".to_string(), "value3".to_string()),
        ];
        db_ops.batch_store(&items).unwrap();
        
        // Test batch get
        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let results: Vec<Option<String>> = db_ops.batch_get(&keys).unwrap();
        
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Some("value1".to_string()));
        assert_eq!(results[1], Some("value2".to_string()));
        assert_eq!(results[2], Some("value3".to_string()));
    }

    #[test]
    fn test_database_statistics() {
        let db_ops = create_temp_db();
        
        // Create some test data
        let atom = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"test": 1}), None)
            .unwrap();
        db_ops
            .update_atom_ref("ref1", atom.uuid().to_string(), "owner".to_string())
            .unwrap();
        db_ops.set_node_id("test-node").unwrap();
        db_ops.store_schema_state("TestSchema", SchemaState::Approved).unwrap();
        
        // Get statistics
        let stats = db_ops.get_stats().unwrap();
        
        assert!(stats.contains_key("atoms"));
        assert!(stats.contains_key("refs"));
        assert!(stats.contains_key("metadata"));
        assert!(stats.contains_key("schema_states"));
        
        // Verify counts
        assert_eq!(stats["atoms"], 1);
        assert_eq!(stats["refs"], 1);
        assert_eq!(stats["metadata"], 1);
        assert_eq!(stats["schema_states"], 1);
    }

    #[test]
    fn test_unified_operations_integration() {
        let db_ops = create_temp_db();
        
        // Test a complete workflow using unified operations
        
        // 1. Set up metadata
        let node_id = "integration-test-node";
        db_ops.set_node_id(node_id).unwrap();
        db_ops.set_schema_permissions(node_id, &["TestSchema".to_string()]).unwrap();
        
        // 2. Set up schema
        db_ops.store_schema_state("TestSchema", SchemaState::Approved).unwrap();
        
        // 3. Create atoms and refs
        let atom = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"value": 42}), None)
            .unwrap();
        db_ops
            .update_atom_ref("test_ref", atom.uuid().to_string(), "owner".to_string())
            .unwrap();
        
        // 4. Set up transforms
        let transform = Transform::new(
            "return value * 2".to_string(),
            "TestSchema.doubled_value".to_string(),
        );
        db_ops.store_transform("doubler", &transform).unwrap();
        
        // 5. Set up orchestrator state
        let orch_state = json!({"active_transforms": ["doubler"]});
        db_ops.store_orchestrator_state("active_state", &orch_state).unwrap();
        
        // 6. Verify everything is accessible through unified interface
        assert_eq!(db_ops.get_node_id().unwrap(), node_id);
        assert_eq!(db_ops.get_schema_permissions(node_id).unwrap(), vec!["TestSchema".to_string()]);
        assert_eq!(db_ops.get_schema_state("TestSchema").unwrap(), Some(SchemaState::Approved));
        
        let retrieved_atom: Option<Atom> = db_ops.get_item(&format!("atom:{}", atom.uuid())).unwrap();
        assert!(retrieved_atom.is_some());
        
        let retrieved_ref: Option<AtomRef> = db_ops.get_item("ref:test_ref").unwrap();
        assert!(retrieved_ref.is_some());
        
        assert!(db_ops.get_transform("doubler").unwrap().is_some());
        
        let retrieved_orch_state: Option<serde_json::Value> = db_ops.get_orchestrator_state("active_state").unwrap();
        assert_eq!(retrieved_orch_state.unwrap(), orch_state);
        
        // 7. Verify statistics reflect all operations
        let stats = db_ops.get_stats().unwrap();
        assert!(stats["atoms"] >= 1);
        assert!(stats["refs"] >= 1);
        assert!(stats["metadata"] >= 1);
        assert!(stats["schema_states"] >= 1);
        assert!(stats["transforms"] >= 1);
        assert!(stats["orchestrator"] >= 1);
    }

    #[test]
    fn test_schema_core_integration_with_unified_db_ops() {
        use crate::schema::{SchemaCore, Schema};
        use crate::schema::core::SchemaState;
        use crate::schema::types::schema::default_schema_type;
        use crate::fees::payment_config::SchemaPaymentConfig;
        use std::sync::Arc;
        use std::collections::HashMap;
        
        let db_ops = Arc::new(create_temp_db());
        let schema_core = SchemaCore::new("test_path", db_ops.clone()).unwrap();
        
        // SchemaCore now always uses DbOperations - no need to check
        // The fact that it was created successfully means it's working
        
        // Create a test schema and add it to SchemaCore
        let test_schema = Schema {
            name: "TestSchema".to_string(),
            schema_type: default_schema_type(),
            fields: HashMap::new(),
            payment_config: SchemaPaymentConfig::default(),
            hash: None,
        };
        
        // Add the schema as available first
        schema_core.add_schema_available(test_schema).unwrap();
        
        // Now test schema state operations through SchemaCore using unified DbOperations
        schema_core.set_schema_state("TestSchema", SchemaState::Approved).unwrap();
        
        // Verify the state was stored using unified operations
        let state = db_ops.get_schema_state("TestSchema").unwrap();
        assert_eq!(state, Some(SchemaState::Approved));
        
        // Test listing schemas by state through unified operations
        let approved_schemas = schema_core.list_schemas_by_state(SchemaState::Approved).unwrap();
        assert!(approved_schemas.contains(&"TestSchema".to_string()));
    }
}