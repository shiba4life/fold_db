use fold_node::testing::{Field, SchemaCore};
use fold_node::fold_db_core::FoldDB;
use fold_node::db_operations::core::DbOperations;
use tempfile::tempdir;
use std::sync::Arc;
mod test_data;
use test_data::schema_test_data::*;

fn create_test_db_ops() -> Arc<DbOperations> {
    let db = sled::Config::new().temporary(true).open().unwrap();
    Arc::new(DbOperations::new(db).unwrap())
}

#[test]
fn test_schema_aref_preservation_on_reload() {
    // Create a temporary directory for test
    let test_dir = tempdir().unwrap();
    let db_ops = create_test_db_ops();
    let manager = SchemaCore::new(test_dir.path().to_str().unwrap(), db_ops).unwrap();

    // Create and load a test schema with a specific ref_atom_uuid
    let original_schema = create_test_schema("aref_preservation_test");
    let original_ref_uuid = original_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap()
        .clone();
    
    manager.add_schema_available(original_schema.clone()).unwrap();
    manager.approve_schema("aref_preservation_test").unwrap();

    // Unload the schema
    manager.unload_schema("aref_preservation_test").unwrap();
    
    // Reload the schema from disk
    manager.load_schemas_from_disk().unwrap();
    
    // Check if the schema is marked as loaded (it should preserve its previous state)
    let reloaded_schema = manager.get_schema("aref_preservation_test").unwrap();
    
    // The schema should be None because it was unloaded and load_schemas_from_disk
    // only loads schemas that were previously marked as Loaded
    assert!(reloaded_schema.is_none(), "Schema should remain unloaded after reload from disk");
    
    // Now explicitly load it again
    manager.add_schema_available(original_schema.clone()).unwrap();
    manager.approve_schema("aref_preservation_test").unwrap();
    let loaded_schema = manager.get_schema("aref_preservation_test").unwrap().unwrap();
    
    // The ref_atom_uuid should be preserved
    let reloaded_ref_uuid = loaded_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap();
    
    assert_eq!(
        &original_ref_uuid, 
        reloaded_ref_uuid,
        "ref_atom_uuid should be preserved after unload/reload cycle"
    );
}

#[test]
fn test_schema_state_preservation_on_reload() {
    // Create a temporary directory for test
    let test_dir = tempdir().unwrap();
    let db_path = test_dir.path().join("test.db");
    
    // Create first manager instance
    let db1 = sled::open(&db_path).unwrap();
    let db_ops1 = Arc::new(DbOperations::new(db1).unwrap());
    let manager = SchemaCore::new(test_dir.path().to_str().unwrap(), db_ops1).unwrap();

    // Create and load a test schema
    let schema = create_test_schema("state_preservation_test");
    let original_ref_uuid = schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap()
        .clone();
    
    manager.add_schema_available(schema.clone()).unwrap();
    manager.approve_schema("state_preservation_test").unwrap();
    
    // Drop the manager to simulate a restart
    drop(manager);
    
    // Create a new manager instance with the same database - this should reload schemas that were marked as Loaded
    let db2 = sled::open(&db_path).unwrap();
    let db_ops2 = Arc::new(DbOperations::new(db2).unwrap());
    let new_manager = SchemaCore::new(test_dir.path().to_str().unwrap(), db_ops2).unwrap();
    new_manager.load_schema_states_from_disk().unwrap();
    
    // The schema should be automatically loaded because it was previously in Loaded state
    let reloaded_schema = new_manager.get_schema("state_preservation_test").unwrap()
        .expect("Schema should be automatically loaded because it was previously in Loaded state");
    
    // The ref_atom_uuid should be preserved
    let reloaded_ref_uuid = reloaded_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap();
    
    assert_eq!(
        &original_ref_uuid, 
        reloaded_ref_uuid,
        "ref_atom_uuid should be preserved after manager restart"
    );
}

#[test]
fn test_folddb_aref_preservation_on_reload() {
    // This test specifically tests the FoldDB level where map_fields is called
    let test_dir = tempdir().unwrap();
    let mut db = FoldDB::new(test_dir.path().to_str().unwrap()).unwrap();

    // Create a test schema with a specific ref_atom_uuid
    let original_schema = create_test_schema("folddb_aref_test");
    let original_ref_uuid = original_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap()
        .clone();
    
    // Load the schema through FoldDB (this calls map_fields)
    db.add_schema_available(original_schema.clone()).unwrap();
    db.approve_schema("folddb_aref_test").unwrap();
    
    // Get the schema and check the ref_atom_uuid
    let loaded_schema = db.get_schema("folddb_aref_test").unwrap().unwrap();
    let loaded_ref_uuid = loaded_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap();
    
    assert_eq!(
        &original_ref_uuid,
        loaded_ref_uuid,
        "ref_atom_uuid should be preserved when loading through FoldDB"
    );
    
    // Unload the schema
    db.unload_schema("folddb_aref_test").unwrap();
    
    // Load it again - this is where the issue might occur
    db.add_schema_available(original_schema.clone()).unwrap();
    db.approve_schema("folddb_aref_test").unwrap();
    
    // Check if the ref_atom_uuid is still the same
    let reloaded_schema = db.get_schema("folddb_aref_test").unwrap().unwrap();
    let reloaded_ref_uuid = reloaded_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid()
        .unwrap();
    
    assert_eq!(
        &original_ref_uuid,
        reloaded_ref_uuid,
        "ref_atom_uuid should be preserved after unload/reload through FoldDB"
    );
}

#[test]
fn test_schema_without_ref_atom_uuid_gets_new_one() {
    // This test verifies that schemas without ref_atom_uuid get new ones assigned
    let test_dir = tempdir().unwrap();
    let db_ops = create_test_db_ops();
    let manager = SchemaCore::new(test_dir.path().to_str().unwrap(), db_ops).unwrap();

    // Create a schema without ref_atom_uuid
    use fold_node::testing::{SingleField, FieldVariant, PermissionsPolicy, FieldPaymentConfig, TrustDistanceScaling};
    use std::collections::HashMap;
    
    let mut new_schema = fold_node::testing::Schema::new("no_ref_uuid_test".to_string());
    let field = SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    // Don't set ref_atom_uuid - it should be None by default
    new_schema.add_field("test_field".to_string(), FieldVariant::Single(field));
    
    // Verify the field doesn't have a ref_atom_uuid
    assert!(new_schema.fields.get("test_field").unwrap().ref_atom_uuid().is_none());
    
    // Load the schema through SchemaCore (this should assign a ref_atom_uuid via map_fields)
    manager.add_schema_available(new_schema).unwrap();
    manager.approve_schema("no_ref_uuid_test").unwrap();
    
    // Get the schema and check that it now has a ref_atom_uuid
    let loaded_schema = manager.get_schema("no_ref_uuid_test").unwrap().unwrap();
    let assigned_ref_uuid = loaded_schema
        .fields
        .get("test_field")
        .unwrap()
        .ref_atom_uuid();
    
    assert!(assigned_ref_uuid.is_some(), "Field should have been assigned a ref_atom_uuid");
    assert!(!assigned_ref_uuid.unwrap().is_empty(), "ref_atom_uuid should not be empty");
}