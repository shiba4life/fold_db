use tempfile::tempdir;
use serde_json::json;
use std::collections::HashMap;

use fold_node::fold_db_core::{
    atom_manager::AtomManager,
    collection_manager::CollectionManager,
    context::AtomContext,
    field_manager::FieldManager,
};
use fold_node::testing::{
    CollectionField,
    Field,
    FieldPaymentConfig,
    FieldType,
    PermissionsPolicy,
    Schema,
    SingleField,
    FieldVariant,
    SchemaError,
    TrustDistanceScaling,
};
use serde_json::Value;
use uuid::Uuid;

fn setup_managers() -> (FieldManager, CollectionManager, AtomManager) {
    let temp = tempdir().unwrap();
    let db = sled::open(temp.path()).unwrap();
    let db_ops = fold_node::db_operations::DbOperations::new(db).unwrap();
    let atom_manager = AtomManager::new(db_ops);
    let field_manager = FieldManager::new(atom_manager.clone());
    let collection_manager = CollectionManager::new(field_manager.clone());
    (field_manager, collection_manager, atom_manager)
}

fn create_single_field() -> FieldVariant {
    let mut field = SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_ref_atom_uuid(Uuid::new_v4().to_string());
    FieldVariant::Single(field)
}

fn create_collection_field() -> FieldVariant {
    let mut field = CollectionField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_ref_atom_uuid(Uuid::new_v4().to_string());
    FieldVariant::Collection(field)
}

#[test]
fn test_field_manager_updates() {
    let (mut field_manager, _, _) = setup_managers();

    let mut schema = Schema::new("test".to_string());
    schema.add_field("name".to_string(), create_single_field());

    field_manager
        .set_field_value(&mut schema, "name", json!("Alice"), "key".to_string())
        .unwrap();
    let value = field_manager.get_field_value(&schema, "name").unwrap();
    assert_eq!(value, json!("Alice"));

    let mut schema_clone = schema.clone();
    field_manager
        .update_field(&mut schema_clone, "name", json!("Bob"), "key".to_string())
        .unwrap();
    let value = field_manager.get_field_value(&schema_clone, "name").unwrap();
    assert_eq!(value, json!("Bob"));

    let mut schema_clone2 = schema_clone.clone();
    field_manager
        .delete_field(&mut schema_clone2, "name", "key".to_string())
        .unwrap();
    let value = field_manager.get_field_value(&schema_clone2, "name").unwrap();
    assert_eq!(value, Value::Null);
}

#[test]
fn test_field_manager_collection_error() {
    let (mut field_manager, _, _) = setup_managers();
    let mut schema = Schema::new("test".to_string());
    schema.add_field("items".to_string(), create_collection_field());

    let result = field_manager.set_field_value(
        &mut schema,
        "items",
        json!("bad"),
        "key".to_string(),
    );
    assert!(matches!(result, Err(SchemaError::InvalidField(_))));
}

#[test]
fn test_collection_manager_operations() {
    let (_field_manager, mut collection_manager, atom_manager) = setup_managers();

    let mut schema = Schema::new("test".to_string());
    schema.add_field("items".to_string(), create_collection_field());
    let aref_uuid = schema
        .fields
        .get("items")
        .unwrap()
        .ref_atom_uuid()
        .unwrap();

    collection_manager
        .add_collection_field_value(
            &schema,
            "items",
            json!("v1"),
            "key".to_string(),
            "1".to_string(),
        )
        .unwrap();

    collection_manager
        .update_collection_field_value(
            &schema,
            "items",
            json!("v2"),
            "key".to_string(),
            "1".to_string(),
        )
        .unwrap();

    let atom_uuid = {
        let collections = atom_manager.get_ref_collections();
        let col = collections.lock().unwrap();
        let col_ref = col.get(aref_uuid.as_str()).unwrap();
        col_ref.get_atom_uuid("1").unwrap().clone()
    };
    let atoms = atom_manager.get_atoms();
    let atom = atoms.lock().unwrap().get(&atom_uuid).unwrap().content().clone();
    assert_eq!(atom, json!("v2"));

    collection_manager
        .delete_collection_field_value(&schema, "items", "key".to_string(), "1".to_string())
        .unwrap();

    let atom_uuid = {
        let collections = atom_manager.get_ref_collections();
        let col = collections.lock().unwrap();
        let col_ref = col.get(aref_uuid.as_str()).unwrap();
        col_ref.get_atom_uuid("1").unwrap().clone()
    };
    let atoms = atom_manager.get_atoms();
    let atom = atoms.lock().unwrap().get(&atom_uuid).unwrap().content().clone();
    assert_eq!(atom, Value::Null);
}

#[test]
fn test_atom_context_type_validation() {
    let (_field_manager, _, mut atom_manager) = setup_managers();
    let schema = {
        let mut s = Schema::new("test".to_string());
        s.add_field("single".to_string(), create_single_field());
        s
    };
    let ctx = AtomContext::new(&schema, "single", "key".to_string(), &mut atom_manager);
    assert!(ctx.validate_field_type(FieldType::Single).is_ok());
    assert!(ctx.validate_field_type(FieldType::Collection).is_err());
    assert!(ctx.validate_field_type(FieldType::Range).is_err());
}

