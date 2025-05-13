use fold_node::atom::{Atom, AtomRef, AtomRefBehavior, AtomStatus};
use fold_node::schema::transform::{TransformRegistry, GetAtomFn, CreateAtomFn, UpdateAtomRefFn};
use fold_node::schema::types::Transform;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Helper struct to store atoms and atom references for testing
struct TestStore {
    atoms: Arc<Mutex<HashMap<String, Atom>>>,
    atom_refs: Arc<Mutex<HashMap<String, AtomRef>>>,
}

// Helper function to set up a test environment
fn setup_test_env() -> (Arc<TestStore>, Arc<TransformRegistry>) {
    // Create atoms
    let atom1 = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!(5),
    );
    
    let atom2 = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!(10),
    );
    
    // Create atom references
    let aref1 = AtomRef::new(atom1.uuid().to_string(), "test_key".to_string());
    let aref2 = AtomRef::new(atom2.uuid().to_string(), "test_key".to_string());
    let output_aref = AtomRef::new("dummy".to_string(), "test_key".to_string());
    
    // Create store
    let atoms = Arc::new(Mutex::new(HashMap::new()));
    atoms.lock().unwrap().insert(atom1.uuid().to_string(), atom1);
    atoms.lock().unwrap().insert(atom2.uuid().to_string(), atom2);
    
    let atom_refs = Arc::new(Mutex::new(HashMap::new()));
    atom_refs.lock().unwrap().insert("input1".to_string(), aref1);
    atom_refs.lock().unwrap().insert("input2".to_string(), aref2);
    atom_refs.lock().unwrap().insert("output".to_string(), output_aref);
    
    let store = Arc::new(TestStore { atoms, atom_refs });
    
    // Create callback functions
    let store_clone = Arc::clone(&store);
    let get_atom_fn: GetAtomFn = Arc::new(move |aref_uuid: &str| -> Result<Atom, Box<dyn std::error::Error>> {
        let atom_refs_guard = store_clone.atom_refs.lock().unwrap();
        if let Some(aref) = atom_refs_guard.get(aref_uuid) {
            let atoms_guard = store_clone.atoms.lock().unwrap();
            if let Some(atom) = atoms_guard.get(aref.get_atom_uuid()) {
                return Ok(atom.clone());
            }
        }
        Err("Atom not found".into())
    });
    
    let store_clone = Arc::clone(&store);
    let create_atom_fn: CreateAtomFn = Arc::new(move |schema_name: &str, source_pub_key: String, _prev_atom_uuid: Option<String>, content: JsonValue, _status: Option<AtomStatus>| -> Result<Atom, Box<dyn std::error::Error>> {
        let atom = Atom::new(
            schema_name.to_string(),
            source_pub_key,
            content,
        );
        let mut atoms_guard = store_clone.atoms.lock().unwrap();
        atoms_guard.insert(atom.uuid().to_string(), atom.clone());
        Ok(atom)
    });
    
    let store_clone = Arc::clone(&store);
    let update_atom_ref_fn: UpdateAtomRefFn = Arc::new(move |aref_uuid: &str, atom_uuid: String, source_pub_key: String| -> Result<AtomRef, Box<dyn std::error::Error>> {
        let mut atom_refs_guard = store_clone.atom_refs.lock().unwrap();
        if let Some(aref) = atom_refs_guard.get_mut(aref_uuid) {
            aref.set_atom_uuid(atom_uuid);
            return Ok(aref.clone());
        }
        Err("AtomRef not found".into())
    });
    
    // Create registry
    let registry = Arc::new(TransformRegistry::new(
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
    ));
    
    (store, registry)
}

#[test]
fn test_register_transform() {
    let (_, registry) = setup_test_env();
    
    // Create a transform
    let transform = Transform::new(
        "input1 + input2".to_string(),
        false,
        None,
        false,
    );
    
    // Register the transform
    let result = registry.register_transform(
        "test_transform".to_string(),
        transform,
        vec!["input1".to_string(), "input2".to_string()],
        "output".to_string(),
    );
    
    assert!(result.is_ok(), "Failed to register transform");
    
    // Check that the transform was registered
    let dependent_transforms = registry.get_dependent_transforms("input1");
    assert!(dependent_transforms.contains("test_transform"), "Transform not registered correctly");
    
    let transform_inputs = registry.get_transform_inputs("test_transform");
    assert!(transform_inputs.contains("input1"), "Transform input not registered correctly");
    assert!(transform_inputs.contains("input2"), "Transform input not registered correctly");
    
    let transform_output = registry.get_transform_output("test_transform");
    assert_eq!(transform_output, Some("output".to_string()), "Transform output not registered correctly");
}

#[test]
fn test_execute_transform() {
    let (store, registry) = setup_test_env();
    
    // Create a transform with a pre-parsed expression
    use fold_node::schema::transform::ast::{Expression, Operator};
    
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Variable("input1".to_string())),
        operator: Operator::Add,
        right: Box::new(Expression::Variable("input2".to_string())),
    };
    
    let transform = Transform::new_with_expr(
        "input1 + input2".to_string(),
        expr,
        false,
        None,
        false,
    );
    
    // Register the transform
    let result = registry.register_transform(
        "test_transform".to_string(),
        transform,
        vec!["input1".to_string(), "input2".to_string()],
        "output".to_string(),
    );
    
    assert!(result.is_ok(), "Failed to register transform");
    
    // Execute the transform
    let results = registry.handle_atom_ref_update("input1");
    
    assert_eq!(results.len(), 1, "Expected one transform result");
    assert!(results[0].is_ok(), "Transform execution failed");
    
    // Check the result value
    let result_value = results[0].as_ref().unwrap();
    // Compare the numeric values, not the exact JSON representation
    match result_value {
        JsonValue::Number(n) => {
            let value = n.as_f64().unwrap();
            assert!((value - 15.0).abs() < 0.001, "Expected 15, got {}", value);
        },
        _ => panic!("Expected number, got {:?}", result_value),
    }
    
    // Check that the output atom reference was updated
    let atom_refs = store.atom_refs.lock().unwrap();
    let output_aref = atom_refs.get("output").unwrap();
    
    let atoms = store.atoms.lock().unwrap();
    let output_atom = atoms.get(output_aref.get_atom_uuid()).unwrap();
    
    // Compare the numeric values, not the exact JSON representation
    match output_atom.content() {
        JsonValue::Number(n) => {
            let value = n.as_f64().unwrap();
            assert!((value - 15.0).abs() < 0.001, "Expected 15, got {}", value);
        },
        _ => panic!("Expected number, got {:?}", output_atom.content()),
    }
}