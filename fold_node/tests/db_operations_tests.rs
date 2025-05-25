use fold_node::db_operations::*;
use fold_node::atom::{Atom, AtomRef, AtomRefCollection, AtomRefRange, AtomRefBehavior};
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestStruct {
    value: String,
}

fn create_temp_db() -> DbOperations {
    let db = sled::Config::new().temporary(true).open().unwrap();
    DbOperations::new(db)
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
    let db_ops = DbOperations::new(db);

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
    let db_ops2 = DbOperations::new(db2);
    let stored_atom: Option<Atom> = db_ops2
        .get_item(&format!("atom:{}", atom.uuid()))
        .unwrap();
    let stored_aref: Option<AtomRef> = db_ops2.get_item("ref:ref_persist").unwrap();

    assert!(stored_atom.is_some());
    assert!(stored_aref.is_some());
    assert_eq!(stored_aref.unwrap().get_atom_uuid(), &atom.uuid().to_string());
}
