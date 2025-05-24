#[cfg(test)]
mod tests {
    use super::super::{Atom, AtomRef, AtomRefBehavior, AtomRefCollection, AtomRefRange};
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test_atom_ref_creation_and_update() {
        let atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            json!({"test": true}),
        );

        // Test single atom ref
        let atom_ref = AtomRef::new(atom.uuid().to_string(), "test_key".to_string());
        assert_eq!(atom_ref.get_atom_uuid(), &atom.uuid().to_string());

        let new_atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            json!({"test": false}),
        );

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

        // Test collection usage
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.set_atom_uuid("0".to_string(), atoms[0].uuid().to_string());
        collection.set_atom_uuid("1".to_string(), atoms[1].uuid().to_string());
        collection.set_atom_uuid("2".to_string(), atoms[2].uuid().to_string());

        assert_eq!(
            collection.get_atom_uuid("0"),
            Some(&atoms[0].uuid().to_string())
        );
        assert_eq!(
            collection.get_atom_uuid("1"),
            Some(&atoms[1].uuid().to_string())
        );
        assert_eq!(
            collection.get_atom_uuid("2"),
            Some(&atoms[2].uuid().to_string())
        );

        // Test removal
        assert_eq!(
            collection.remove_atom_uuid("1"),
            Some(atoms[1].uuid().to_string())
        );
        assert_eq!(collection.get_atom_uuid("1"), None);

        // Test behavior trait
        assert!(collection.updated_at() > Utc::now() - chrono::Duration::seconds(1));
    }

    #[test]
    fn test_atom_ref_range() {
        let atoms: Vec<_> = (0..3)
            .map(|i| {
                Atom::new(
                    "test_schema".to_string(),
                    "test_key".to_string(),
                    json!({ "index": i }),
                )
            })
            .collect();

        let mut range = AtomRefRange::new("test_key".to_string());
        range.set_atom_uuid("a".to_string(), atoms[0].uuid().to_string());
        range.set_atom_uuid("b".to_string(), atoms[1].uuid().to_string());
        range.set_atom_uuid("c".to_string(), atoms[2].uuid().to_string());

        let keys: Vec<_> = range.atom_uuids.keys().cloned().collect();
        assert_eq!(keys, vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        assert_eq!(range.get_atom_uuid("b"), Some(&atoms[1].uuid().to_string()));
        assert_eq!(
            range.remove_atom_uuid("b"),
            Some(atoms[1].uuid().to_string())
        );
        assert!(range.get_atom_uuid("b").is_none());

        assert!(range.updated_at() > Utc::now() - chrono::Duration::seconds(1));
    }
}