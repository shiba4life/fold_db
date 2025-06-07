#[cfg(test)]
mod tests {
    use super::super::{Atom, AtomRefHash, AtomRefBehavior};
    use serde_json::json;

    #[test]
    fn test_atom_ref_hash_basic_ops() {
        let atoms: Vec<_> = (0..3)
            .map(|i| {
                Atom::new(
                    "test_schema".to_string(),
                    "test_key".to_string(),
                    json!({"index": i}),
                )
            })
            .collect();

        let mut hash = AtomRefHash::new("test_key".to_string());
        hash.set_atom_uuid("a".to_string(), atoms[0].uuid().to_string());
        hash.set_atom_uuid("b".to_string(), atoms[1].uuid().to_string());

        assert_eq!(hash.get_atom_uuid("a"), Some(&atoms[0].uuid().to_string()));
        assert!(hash.contains_key("b"));
        assert_eq!(hash.len(), 2);

        assert_eq!(hash.remove_atom_uuid("a"), Some(atoms[0].uuid().to_string()));
        assert!(!hash.contains_key("a"));

        hash.clear();
        assert!(hash.is_empty());
    }
}
