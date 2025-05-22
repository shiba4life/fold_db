use serde_json::json;
use tempfile::tempdir;
use uuid::Uuid;

use fold_node::db_operations::DbOperations;
use fold_node::fold_db_core::atom_manager::AtomManager;

#[test]
fn test_atom_history_retrieval() {
    let dir = tempdir().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let db_ops = DbOperations::new(db);
    let manager = AtomManager::new(db_ops);

    // create initial atom and reference
    let atom1 = manager
        .create_atom("schema", "key".to_string(), None, json!(1), None)
        .unwrap();
    let aref_uuid = Uuid::new_v4().to_string();
    manager
        .update_atom_ref(&aref_uuid, atom1.uuid().to_string(), "key".to_string())
        .unwrap();

    // create second version
    let atom2 = manager
        .create_atom(
            "schema",
            "key".to_string(),
            Some(atom1.uuid().to_string()),
            json!(2),
            None,
        )
        .unwrap();
    manager
        .update_atom_ref(&aref_uuid, atom2.uuid().to_string(), "key".to_string())
        .unwrap();

    // create third version
    let atom3 = manager
        .create_atom(
            "schema",
            "key".to_string(),
            Some(atom2.uuid().to_string()),
            json!(3),
            None,
        )
        .unwrap();
    manager
        .update_atom_ref(&aref_uuid, atom3.uuid().to_string(), "key".to_string())
        .unwrap();

    // retrieve history
    let history = manager.get_atom_history(&aref_uuid).unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].uuid(), atom3.uuid());
    assert_eq!(history[1].uuid(), atom2.uuid());
    assert_eq!(history[2].uuid(), atom1.uuid());
}

#[test]
fn test_in_memory_caches_and_reinit() {
    let dir = tempdir().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let db_clone = db.clone();
    let db_ops = DbOperations::new(db);
    let manager = AtomManager::new(db_ops);

    let atom1 = manager
        .create_atom("schema", "key".to_string(), None, json!(1), None)
        .unwrap();
    let aref_uuid = Uuid::new_v4().to_string();
    manager
        .update_atom_ref(&aref_uuid, atom1.uuid().to_string(), "key".to_string())
        .unwrap();

    let atom2 = manager
        .create_atom(
            "schema",
            "key".to_string(),
            Some(atom1.uuid().to_string()),
            json!(2),
            None,
        )
        .unwrap();
    manager
        .update_atom_ref(&aref_uuid, atom2.uuid().to_string(), "key".to_string())
        .unwrap();

    // check cache contents
    assert_eq!(manager.get_atoms().lock().unwrap().len(), 2);
    assert_eq!(manager.get_ref_atoms().lock().unwrap().len(), 1);
    let latest_uuid = manager
        .get_ref_atoms()
        .lock()
        .unwrap()
        .get(&aref_uuid)
        .unwrap()
        .get_atom_uuid()
        .clone();
    assert_eq!(latest_uuid, atom2.uuid().to_string());

    // flush data for good measure
    db_clone.flush().unwrap();
}

#[test]
fn test_reference_updates() {
    let dir = tempdir().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let db_ops = DbOperations::new(db);
    let manager = AtomManager::new(db_ops);

    let atom1 = manager
        .create_atom("schema", "key".to_string(), None, json!(1), None)
        .unwrap();
    let aref_uuid = Uuid::new_v4().to_string();
    manager
        .update_atom_ref(&aref_uuid, atom1.uuid().to_string(), "key".to_string())
        .unwrap();
    let ref_atoms = manager.get_ref_atoms();
    assert_eq!(
        ref_atoms
            .lock()
            .unwrap()
            .get(&aref_uuid)
            .unwrap()
            .get_atom_uuid(),
        &atom1.uuid().to_string()
    );

    let atom2 = manager
        .create_atom(
            "schema",
            "key".to_string(),
            Some(atom1.uuid().to_string()),
            json!(2),
            None,
        )
        .unwrap();
    manager
        .update_atom_ref(&aref_uuid, atom2.uuid().to_string(), "key".to_string())
        .unwrap();
    assert_eq!(
        ref_atoms
            .lock()
            .unwrap()
            .get(&aref_uuid)
            .unwrap()
            .get_atom_uuid(),
        &atom2.uuid().to_string()
    );

    // collection update
    let collection_uuid = Uuid::new_v4().to_string();
    let col_atom1 = manager
        .create_atom("schema", "key".to_string(), None, json!("a"), None)
        .unwrap();
    manager
        .update_atom_ref_collection(
            &collection_uuid,
            col_atom1.uuid().to_string(),
            "0".to_string(),
            "key".to_string(),
        )
        .unwrap();
    let col_map = manager.get_ref_collections();
    assert_eq!(
        col_map
            .lock()
            .unwrap()
            .get(&collection_uuid)
            .unwrap()
            .get_atom_uuid("0")
            .unwrap(),
        &col_atom1.uuid().to_string()
    );

    let col_atom2 = manager
        .create_atom(
            "schema",
            "key".to_string(),
            Some(col_atom1.uuid().to_string()),
            json!("b"),
            None,
        )
        .unwrap();
    manager
        .update_atom_ref_collection(
            &collection_uuid,
            col_atom2.uuid().to_string(),
            "0".to_string(),
            "key".to_string(),
        )
        .unwrap();
    assert_eq!(
        col_map
            .lock()
            .unwrap()
            .get(&collection_uuid)
            .unwrap()
            .get_atom_uuid("0")
            .unwrap(),
        &col_atom2.uuid().to_string()
    );

    // range update
    let range_uuid = Uuid::new_v4().to_string();
    let range_atom1 = manager
        .create_atom("schema", "key".to_string(), None, json!("r1"), None)
        .unwrap();
    manager
        .update_atom_ref_range(
            &range_uuid,
            range_atom1.uuid().to_string(),
            "a".to_string(),
            "key".to_string(),
        )
        .unwrap();
    let range_map = manager.get_ref_ranges();
    assert_eq!(
        range_map
            .lock()
            .unwrap()
            .get(&range_uuid)
            .unwrap()
            .get_atom_uuid("a")
            .unwrap(),
        &range_atom1.uuid().to_string()
    );
}

