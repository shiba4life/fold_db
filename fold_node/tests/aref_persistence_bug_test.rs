use fold_node::testing::{Field, SchemaCore};
use fold_node::fold_db_core::FoldDB;
use tempfile::tempdir;
mod test_data;
use test_data::schema_test_data::*;

#[test]
fn test_aref_persistence_bug_reproduction() {
    // Create a temporary directory for test
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create FoldDB and load a schema
    let original_ref_uuid = {
        let mut db = FoldDB::new(test_path).unwrap();
        let schema = create_test_schema("aref_bug_test");
        let original_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone();
        
        db.add_schema_available(schema).unwrap();
        db.approve_schema("aref_bug_test").unwrap();
        
        // Verify the schema is loaded with the correct aref
        let loaded_schema = db.get_schema("aref_bug_test").unwrap().unwrap();
        let loaded_ref_uuid = loaded_schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        assert_eq!(&original_ref_uuid, loaded_ref_uuid, "Initial load should preserve ref_atom_uuid");
        
        original_ref_uuid
    }; // Drop the first FoldDB instance
    
    // Step 2: Create a new FoldDB instance (simulating restart)
    // This should automatically load schemas that were marked as Loaded
    let db2 = FoldDB::new(test_path).unwrap();
    
    // Step 3: Check if the schema is loaded and has the same aref
    let reloaded_schema = db2.get_schema("aref_bug_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        assert_eq!(
            &original_ref_uuid, 
            reloaded_ref_uuid,
            "ref_atom_uuid should be preserved after FoldDB restart"
        );
    } else {
        panic!("Schema should be automatically loaded after FoldDB restart");
    }
}

#[test]
fn test_schema_core_restart_without_map_fields() {
    // This test specifically checks if the issue is in SchemaCore's load_schema_states_from_disk
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create SchemaCore and load a schema
    let original_ref_uuid = {
        let manager = SchemaCore::new(test_path).unwrap();
        let schema = create_test_schema("schema_core_bug_test");
        let original_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone();
        
        manager.add_schema_available(schema).unwrap();
        manager.approve_schema("schema_core_bug_test").unwrap();
        original_ref_uuid
    }; // Drop the first SchemaCore instance
    
    // Step 2: Create a new SchemaCore instance and load states from disk
    let manager2 = SchemaCore::new(test_path).unwrap();
    manager2.load_schema_states_from_disk().unwrap();
    
    // Step 3: Check if the schema has the same aref
    let reloaded_schema = manager2.get_schema("schema_core_bug_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        assert_eq!(
            &original_ref_uuid, 
            reloaded_ref_uuid,
            "ref_atom_uuid should be preserved after SchemaCore restart"
        );
    } else {
        panic!("Schema should be automatically loaded after SchemaCore restart");
    }
}