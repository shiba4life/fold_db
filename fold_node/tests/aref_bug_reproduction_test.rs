use fold_node::testing::{Field, SchemaCore, SingleField, FieldVariant, PermissionsPolicy, FieldPaymentConfig, TrustDistanceScaling};
use fold_node::fold_db_core::FoldDB;
use tempfile::tempdir;
use std::collections::HashMap;

fn create_schema_without_aref(name: &str) -> fold_node::testing::Schema {
    let mut schema = fold_node::testing::Schema::new(name.to_string());
    let field = SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    // Don't set ref_atom_uuid - it should be None by default
    schema.add_field("test_field".to_string(), FieldVariant::Single(field));
    schema
}

#[test]
fn test_aref_bug_reproduction_with_folddb_restart() {
    // This test reproduces the exact scenario where arefs get unlinked:
    // 1. Load a schema without arefs (they get generated)
    // 2. Restart FoldDB (which calls load_schema_states_from_disk)
    // 3. Check if the arefs are still there
    
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create FoldDB and load a schema without aref
    let generated_ref_uuid = {
        let mut db = FoldDB::new(test_path).unwrap();
        let schema = create_schema_without_aref("bug_reproduction_test");
        
        // Verify the schema doesn't have a ref_atom_uuid initially
        assert!(schema.fields.get("test_field").unwrap().ref_atom_uuid().is_none());
        
        db.add_schema_available(schema).unwrap();
        db.approve_schema("bug_reproduction_test").unwrap();
        
        // After loading, the schema should have a generated aref
        let loaded_schema = db.get_schema("bug_reproduction_test").unwrap().unwrap();
        let generated_ref_uuid = loaded_schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone();
        
        assert!(!generated_ref_uuid.is_empty(), "Generated ref_atom_uuid should not be empty");
        println!("Generated ref_atom_uuid: {}", generated_ref_uuid);
        
        generated_ref_uuid
    }; // Drop the first FoldDB instance to simulate restart
    
    // Step 2: Create a new FoldDB instance (simulating restart)
    // This internally calls load_schema_states_from_disk which should preserve arefs
    let db2 = FoldDB::new(test_path).unwrap();
    
    // Step 3: Check if the schema is loaded and has the same generated aref
    let reloaded_schema = db2.get_schema("bug_reproduction_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        println!("Reloaded ref_atom_uuid: {}", reloaded_ref_uuid);
        
        assert_eq!(
            &generated_ref_uuid, 
            reloaded_ref_uuid,
            "Generated ref_atom_uuid should be preserved after FoldDB restart. This was the bug!"
        );
    } else {
        panic!("Schema should be automatically loaded after FoldDB restart");
    }
}

#[test]
fn test_schema_core_direct_load_states_bug() {
    // This test directly tests the load_schema_states_from_disk method
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create SchemaCore and load a schema without aref
    let generated_ref_uuid = {
        let manager = SchemaCore::new(test_path).unwrap();
        let schema = create_schema_without_aref("direct_load_states_test");
        
        manager.add_schema_available(schema).unwrap();
        manager.approve_schema("direct_load_states_test").unwrap();
        
        // Get the generated aref
        let loaded_schema = manager.get_schema("direct_load_states_test").unwrap().unwrap();
        let generated_ref_uuid = loaded_schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone();
        
        println!("Generated ref_atom_uuid: {}", generated_ref_uuid);
        generated_ref_uuid
    }; // Drop the first SchemaCore instance
    
    // Step 2: Create a new SchemaCore instance and directly call load_schema_states_from_disk
    let manager2 = SchemaCore::new(test_path).unwrap();
    manager2.load_schema_states_from_disk().unwrap();
    
    // Step 3: Check if the schema has the same generated aref
    let reloaded_schema = manager2.get_schema("direct_load_states_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        println!("Reloaded ref_atom_uuid: {}", reloaded_ref_uuid);
        
        assert_eq!(
            &generated_ref_uuid, 
            reloaded_ref_uuid,
            "Generated ref_atom_uuid should be preserved after calling load_schema_states_from_disk. This was the bug!"
        );
    } else {
        panic!("Schema should be loaded after calling load_schema_states_from_disk");
    }
}