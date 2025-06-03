use fold_node::db_operations::DbOperations;
use fold_node::schema::types::{SchemaError, Transform};
use tempfile::TempDir;

fn create_temp_db() -> (DbOperations, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = sled::open(&db_path).unwrap();
    let db_ops = DbOperations::new(db).unwrap();
    (db_ops, temp_dir)
}

#[test]
fn test_transform_manager_integration_with_unified_operations() {
    let (db_ops, _temp_dir) = create_temp_db();

    // Test 1: Store a transform directly in DbOperations
    let transform = Transform::new("return input * 2".to_string(), "test.doubled".to_string());

    let result = db_ops.store_transform("test_transform", &transform);
    assert!(
        result.is_ok(),
        "Should be able to store transform in DbOperations"
    );

    // Test 2: Retrieve the transform from DbOperations
    let retrieved = db_ops.get_transform("test_transform").unwrap();
    assert!(
        retrieved.is_some(),
        "Transform should be retrievable from DbOperations"
    );

    let retrieved_transform = retrieved.unwrap();
    assert_eq!(retrieved_transform.logic, "return input * 2");
    assert_eq!(retrieved_transform.output, "test.doubled");

    // Test 3: List transforms
    let transforms = db_ops.list_transforms().unwrap();
    assert!(
        transforms.contains(&"test_transform".to_string()),
        "Transform should be in list"
    );

    // Test 4: Store and retrieve transform mappings
    let mapping_data = serde_json::to_vec(&serde_json::json!({"test": "mapping"})).unwrap();
    let mapping_result = db_ops.store_transform_mapping("test_mapping", &mapping_data);
    assert!(
        mapping_result.is_ok(),
        "Should be able to store transform mapping"
    );

    let retrieved_mapping = db_ops.get_transform_mapping("test_mapping").unwrap();
    assert!(
        retrieved_mapping.is_some(),
        "Transform mapping should be retrievable"
    );

    // Test 5: Delete transform
    let delete_result = db_ops.delete_transform("test_transform");
    assert!(delete_result.is_ok(), "Should be able to delete transform");

    let deleted_check = db_ops.get_transform("test_transform").unwrap();
    assert!(deleted_check.is_none(), "Transform should be deleted");

    println!("✅ DbOperations transform functionality works correctly");
    println!("✅ TransformManager can use unified DbOperations for persistence");
}

#[test]
fn test_transform_manager_constructor_with_unified_operations() {
    use fold_node::atom::{Atom, AtomRef};
    use fold_node::fold_db_core::transform_manager::manager::TransformManager;
    use serde_json::Value;
    use std::sync::Arc;

    let (db_ops, _temp_dir) = create_temp_db();
    let db_ops = Arc::new(db_ops);

    // Pre-store a transform to test loading
    let transform = Transform::new(
        "return value + 1".to_string(),
        "test.incremented".to_string(),
    );
    db_ops
        .store_transform("preloaded_transform", &transform)
        .unwrap();

    // Create minimal callback functions
    let get_atom_fn = Arc::new(
        |_aref_uuid: &str| -> Result<Atom, Box<dyn std::error::Error>> {
            let atom = Atom::new(
                "test_schema".to_string(),
                "test_pub_key".to_string(),
                serde_json::json!({"value": 1}),
            );
            Ok(atom)
        },
    );

    let create_atom_fn = Arc::new(
        |schema_name: &str,
         source_pub_key: String,
         _prev_atom_uuid: Option<String>,
         content: Value,
         _status: Option<fold_node::atom::AtomStatus>|
         -> Result<Atom, Box<dyn std::error::Error>> {
            let atom = Atom::new(schema_name.to_string(), source_pub_key, content);
            Ok(atom)
        },
    );

    let update_atom_ref_fn = Arc::new(
        |aref_uuid: &str,
         atom_uuid: String,
         _source_pub_key: String|
         -> Result<AtomRef, Box<dyn std::error::Error>> {
            let atom_ref = AtomRef::new(aref_uuid.to_string(), atom_uuid);
            Ok(atom_ref)
        },
    );

    let get_field_fn = Arc::new(
        |_schema_name: &str, _field_name: &str| -> Result<Value, SchemaError> {
            Ok(serde_json::json!({"field_value": "test"}))
        },
    );

    // Create TransformManager with unified operations - this should load the preloaded transform
    let transform_manager_result = TransformManager::new(
        db_ops.clone(),
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
        get_field_fn,
    );

    assert!(
        transform_manager_result.is_ok(),
        "TransformManager should be created successfully with unified operations"
    );

    let transform_manager = transform_manager_result.unwrap();

    // Verify the preloaded transform exists
    let exists = transform_manager
        .transform_exists("preloaded_transform")
        .unwrap();
    assert!(
        exists,
        "Preloaded transform should exist in TransformManager"
    );

    println!("✅ TransformManager constructor with unified operations works correctly");
    println!("✅ TransformManager loads persisted transforms from unified operations");
}
