use fold_node::fold_db_core::transform_manager::TransformManager;
use fold_node::schema::types::{Transform, TransformRegistration};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;

#[test]
fn mappings_persist_to_disk() {
    let dir = tempfile::tempdir().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let tree = db.open_tree("transforms").unwrap();

    let get_atom_fn = Arc::new(|_id: &str| -> Result<fold_node::atom::Atom, Box<dyn std::error::Error>> {
        Err("ni".into())
    });
    let create_atom_fn = Arc::new(|_s: &str, _p: String, _prev: Option<String>, _c: serde_json::Value, _st: Option<fold_node::atom::AtomStatus>| -> Result<fold_node::atom::Atom, Box<dyn std::error::Error>> {
        Err("ni".into())
    });
    let update_atom_ref_fn = Arc::new(|_a: &str, _u: String, _p: String| -> Result<fold_node::atom::AtomRef, Box<dyn std::error::Error>> {
        Err("ni".into())
    });
    let get_field_fn = Arc::new(|_s: &str, _f: &str| -> Result<serde_json::Value, fold_node::schema::SchemaError> {
        Ok(json!(null))
    });

    let manager = TransformManager::new(
        tree.clone(),
        get_atom_fn.clone(),
        create_atom_fn.clone(),
        update_atom_ref_fn.clone(),
        get_field_fn.clone(),
    );

    let registration = TransformRegistration {
        transform_id: "S.a".to_string(),
        transform: Transform::new("1 + 1".to_string(), "S.a".to_string()),
        input_arefs: vec!["R1".to_string()],
        input_names: vec!["S.input".to_string()],
        trigger_fields: vec!["S.input".to_string()],
        output_aref: "O1".to_string(),
        schema_name: "S".to_string(),
        field_name: "a".to_string(),
    };
    manager.register_transform(registration).unwrap();

    drop(manager);

    let tree = db.open_tree("transforms").unwrap();
    let manager2 = TransformManager::new(
        tree,
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
        get_field_fn,
    );

    assert_eq!(
        manager2.get_transform_output("S.a").unwrap().unwrap(),
        "O1"
    );
    assert_eq!(
        manager2.get_transform_inputs("S.a").unwrap(),
        vec!["R1".to_string()].into_iter().collect::<HashSet<_>>()
    );
    assert_eq!(
        manager2.get_dependent_transforms("R1").unwrap(),
        vec!["S.a".to_string()].into_iter().collect::<HashSet<_>>()
    );
    assert_eq!(
        manager2
            .get_transforms_for_field("S", "input")
            .unwrap(),
        vec!["S.a".to_string()].into_iter().collect::<HashSet<_>>()
    );
}
