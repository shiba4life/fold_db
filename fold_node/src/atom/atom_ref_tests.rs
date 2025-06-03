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
        assert_eq!(
            keys,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );

        assert_eq!(range.get_atom_uuid("b"), Some(&atoms[1].uuid().to_string()));
        assert_eq!(
            range.remove_atom_uuid("b"),
            Some(atoms[1].uuid().to_string())
        );
        assert!(range.get_atom_uuid("b").is_none());

        assert!(range.updated_at() > Utc::now() - chrono::Duration::seconds(1));
    }

    #[test]
    fn test_atom_ref_range_multiple_atoms_per_key() {
        let atoms: Vec<_> = (0..5)
            .map(|i| {
                Atom::new(
                    "test_schema".to_string(),
                    "test_key".to_string(),
                    json!({ "value": i, "type": format!("mutation_{}", i) }),
                )
            })
            .collect();

        let mut range = AtomRefRange::new("test_key".to_string());

        // Add multiple atoms to the same key - this should accumulate, not override
        range.set_atom_uuid("user_123".to_string(), atoms[0].uuid().to_string());
        range.set_atom_uuid("user_123".to_string(), atoms[1].uuid().to_string());
        range.set_atom_uuid("user_123".to_string(), atoms[2].uuid().to_string());

        // Add atoms to different keys
        range.set_atom_uuid("user_456".to_string(), atoms[3].uuid().to_string());
        range.set_atom_uuid("user_789".to_string(), atoms[4].uuid().to_string());

        // Verify that all atoms are stored for user_123
        let user_123_uuids = range.get_atom_uuids("user_123").unwrap();
        assert_eq!(user_123_uuids.len(), 3, "Should have 3 atoms for user_123");
        assert!(user_123_uuids.contains(&atoms[0].uuid().to_string()));
        assert!(user_123_uuids.contains(&atoms[1].uuid().to_string()));
        assert!(user_123_uuids.contains(&atoms[2].uuid().to_string()));

        // Verify backward compatibility with get_atom_uuid (returns first)
        assert_eq!(
            range.get_atom_uuid("user_123"),
            Some(&atoms[0].uuid().to_string())
        );

        // Verify other keys have single atoms
        let user_456_uuids = range.get_atom_uuids("user_456").unwrap();
        assert_eq!(user_456_uuids.len(), 1);
        assert_eq!(user_456_uuids[0], atoms[3].uuid().to_string());

        let user_789_uuids = range.get_atom_uuids("user_789").unwrap();
        assert_eq!(user_789_uuids.len(), 1);
        assert_eq!(user_789_uuids[0], atoms[4].uuid().to_string());

        // Test removal
        let removed_uuids = range.remove_atom_uuids("user_123").unwrap();
        assert_eq!(removed_uuids.len(), 3);
        assert!(range.get_atom_uuids("user_123").is_none());

        // Verify other keys still exist
        assert!(range.get_atom_uuids("user_456").is_some());
        assert!(range.get_atom_uuids("user_789").is_some());
    }
}
