use chrono::Utc;
use fold_node::atom::{Atom, AtomRef, AtomRefBehavior, AtomRefCollection};
use serde_json::json;

#[test]
fn test_atom_creation() {
    let content = json!({
        "name": "test",
        "value": 42
    });

    let atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        content.clone(),
    );

    assert_eq!(atom.source_schema_name(), "test_schema");
    assert_eq!(atom.source_pub_key(), "test_key");
    assert_eq!(atom.prev_atom_uuid(), None);
    assert_eq!(atom.content(), &content);
    assert!(!atom.uuid().is_empty());
    assert!(atom.created_at() <= Utc::now());
}

#[test]
fn test_atom_with_prev_reference() {
    let first_atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!({"version": 1}),
    );

    let second_atom = Atom::with_prev_version(first_atom.clone(), first_atom.uuid().to_string());

    assert_eq!(second_atom.prev_atom_uuid(), Some(&first_atom.uuid().to_string()));
}

#[test]
fn test_atom_ref_creation_and_update() {
    let atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!({"test": true}),
    );

    let atom_ref = AtomRef::new(atom.uuid().to_string(), "test_key".to_string());
    assert_eq!(atom_ref.get_atom_uuid(), &atom.uuid().to_string());

    let new_atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!({"test": false}),
    )
    .with_prev_version(atom.uuid().to_string());

    let mut updated_ref = atom_ref.clone();
    updated_ref.set_atom_uuid(new_atom.uuid().to_string());

    assert_eq!(updated_ref.get_atom_uuid(), &new_atom.uuid().to_string());
    assert!(updated_ref.updated_at() >= atom_ref.updated_at());
}

#[test]
fn test_atom_ref_collection() {
    let atoms: Vec<_> = (0..3)
        .map(|i| {
            Atom::new(
                "test_schema".to_string(),
                "test_key".to_string(),
                json!({ "index": i }),
            )
        })
        .collect();

    let mut collection = AtomRefCollection::new("test_key".to_string());
    collection.set_atom_uuid("0".to_string(), atoms[0].uuid().to_string());
    collection.set_atom_uuid("1".to_string(), atoms[1].uuid().to_string());
    collection.set_atom_uuid("2".to_string(), atoms[2].uuid().to_string());

    assert_eq!(collection.get_atom_uuid("0"), Some(&atoms[0].uuid().to_string()));
    assert_eq!(collection.get_atom_uuid("1"), Some(&atoms[1].uuid().to_string()));
    assert_eq!(collection.get_atom_uuid("2"), Some(&atoms[2].uuid().to_string()));

    assert_eq!(collection.remove_atom_uuid("1"), Some(atoms[1].uuid().to_string()));
    assert_eq!(collection.get_atom_uuid("1"), None);

    assert!(collection.updated_at() > Utc::now() - chrono::Duration::seconds(1));
}
