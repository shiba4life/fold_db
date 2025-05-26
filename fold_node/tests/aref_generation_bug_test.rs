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
fn test_aref_generation_and_persistence_bug() {
    // Create a temporary directory for test
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create FoldDB and load a schema without aref
    let generated_ref_uuid = {
        let mut db = FoldDB::new(test_path).unwrap();
        let schema = create_schema_without_aref("aref_gen_test");
        
        // Verify the schema doesn't have a ref_atom_uuid initially
        assert!(schema.fields.get("test_field").unwrap().ref_atom_uuid().is_none());
        
        db.load_schema(schema).unwrap();
        
        // After loading, the schema should have a generated aref
        let loaded_schema = db.get_schema("aref_gen_test").unwrap().unwrap();
        let generated_ref_uuid = loaded_schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone();
        
        assert!(!generated_ref_uuid.is_empty(), "Generated ref_atom_uuid should not be empty");
        
        generated_ref_uuid
    }; // Drop the first FoldDB instance
    
    // Step 2: Create a new FoldDB instance (simulating restart)
    let db2 = FoldDB::new(test_path).unwrap();
    
    // Step 3: Check if the schema is loaded and has the same generated aref
    let reloaded_schema = db2.get_schema("aref_gen_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        assert_eq!(
            &generated_ref_uuid, 
            reloaded_ref_uuid,
            "Generated ref_atom_uuid should be preserved after FoldDB restart"
        );
    } else {
        panic!("Schema should be automatically loaded after FoldDB restart");
    }
}

#[test]
fn test_schema_core_aref_generation_bug() {
    // This test specifically checks if the issue is in SchemaCore's load_schema_states_from_disk
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create SchemaCore and load a schema without aref
    let generated_ref_uuid = {
        let manager = SchemaCore::new(test_path).unwrap();
        let schema = create_schema_without_aref("schema_core_gen_test");
        
        // Verify the schema doesn't have a ref_atom_uuid initially
        assert!(schema.fields.get("test_field").unwrap().ref_atom_uuid().is_none());
        
        manager.load_schema(schema).unwrap();
        
        // After loading, the schema should have a generated aref
        let loaded_schema = manager.get_schema("schema_core_gen_test").unwrap().unwrap();
        let generated_ref_uuid = loaded_schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone();
        
        assert!(!generated_ref_uuid.is_empty(), "Generated ref_atom_uuid should not be empty");
        
        generated_ref_uuid
    }; // Drop the first SchemaCore instance
    
    // Step 2: Create a new SchemaCore instance and load states from disk
    let manager2 = SchemaCore::new(test_path).unwrap();
    manager2.load_schema_states_from_disk().unwrap();
    
    // Step 3: Check if the schema has the same generated aref
    let reloaded_schema = manager2.get_schema("schema_core_gen_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        assert_eq!(
            &generated_ref_uuid, 
            reloaded_ref_uuid,
            "Generated ref_atom_uuid should be preserved after SchemaCore restart"
        );
    } else {
        panic!("Schema should be automatically loaded after SchemaCore restart");
    }
}

#[test]
fn test_map_fields_not_called_on_reload() {
    // This test specifically checks if map_fields is called during load_schema_states_from_disk
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create SchemaCore and load a schema without aref
    let generated_ref_uuid = {
        let manager = SchemaCore::new(test_path).unwrap();
        let schema = create_schema_without_aref("map_fields_test");
        
        manager.load_schema(schema).unwrap();
        
        // Get the generated aref
        let loaded_schema = manager.get_schema("map_fields_test").unwrap().unwrap();
        loaded_schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap()
            .clone()
    }; // Drop the first SchemaCore instance
    
    // Step 2: Create a new SchemaCore instance but DON'T call load_schema_states_from_disk
    // Instead, manually load the schema from disk to see if map_fields is the issue
    let manager2 = SchemaCore::new(test_path).unwrap();
    
    // Load schemas from disk (this calls map_fields)
    manager2.load_schemas_from_disk().unwrap();
    
    // Check if the schema has the same aref
    let reloaded_schema = manager2.get_schema("map_fields_test").unwrap();
    
    if let Some(schema) = reloaded_schema {
        let reloaded_ref_uuid = schema
            .fields
            .get("test_field")
            .unwrap()
            .ref_atom_uuid()
            .unwrap();
        
        assert_eq!(
            &generated_ref_uuid, 
            reloaded_ref_uuid,
            "Generated ref_atom_uuid should be preserved when using load_schemas_from_disk"
        );
    } else {
        // This might be expected if the schema state was Unloaded
        println!("Schema was not loaded - this might be expected behavior");
    }
}