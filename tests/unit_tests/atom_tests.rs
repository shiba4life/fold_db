use chrono::Utc;
use fold_db::testing::{Atom, AtomRef, AtomRefBehavior};
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
    assert!(atom.uuid().len() > 0);
    assert!(atom.created_at() <= Utc::now());
}

#[test]
fn test_atom_with_prev_reference() {
    let first_atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!({"version": 1}),
    );

    let second_atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!({"version": 2}),
    ).with_prev_version(first_atom.uuid().to_string());

    assert_eq!(
        second_atom.prev_atom_uuid(),
        Some(&first_atom.uuid().to_string())
    );
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
    ).with_prev_version(atom.uuid().to_string());

    let mut updated_ref = atom_ref.clone();
    updated_ref.set_atom_uuid(new_atom.uuid().to_string());

    assert_eq!(
        updated_ref.get_atom_uuid(),
        &new_atom.uuid().to_string()
    );
    assert!(updated_ref.updated_at() >= atom_ref.updated_at());
}
