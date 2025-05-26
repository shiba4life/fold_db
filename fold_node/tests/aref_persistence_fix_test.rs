use fold_node::testing::{Field, SingleField, FieldVariant, PermissionsPolicy, FieldPaymentConfig, TrustDistanceScaling};
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
fn test_aref_persistence_fix_verification() {
    // This test verifies that the fix for aref persistence across reloads works correctly
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path().to_str().unwrap();
    
    // Step 1: Create FoldDB and load a schema without aref
    let generated_ref_uuid = {
        let mut db = FoldDB::new(test_path).unwrap();
        let schema = create_schema_without_aref("persistence_fix_test");
        
        // Verify the schema doesn't have a ref_atom_uuid initially
        assert!(schema.fields.get("test_field").unwrap().ref_atom_uuid().is_none());
        
        db.add_schema_available(schema).unwrap();
        db.approve_schema("persistence_fix_test").unwrap();
        
        // After loading, the schema should have a generated aref
        let loaded_schema = db.get_schema("persistence_fix_test").unwrap().unwrap();
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
    // This should automatically load schemas that were marked as Loaded
    // and properly map their arefs
    let db2 = FoldDB::new(test_path).unwrap();
    
    // Step 3: Check if the schema is loaded and has the same generated aref
    let reloaded_schema = db2.get_schema("persistence_fix_test").unwrap();
    
    assert!(reloaded_schema.is_some(), "Schema should be automatically loaded after FoldDB restart");
    
    let schema = reloaded_schema.unwrap();
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
        "Generated ref_atom_uuid should be preserved after FoldDB restart - this verifies the fix!"
    );
    
    println!("✅ Aref persistence fix verified successfully!");
}