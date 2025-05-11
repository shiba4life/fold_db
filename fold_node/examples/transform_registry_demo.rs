//! Transform Registry Demo
//!
//! This example demonstrates how to use the Transform Registry to track dependencies
//! between atom references and transforms, and execute transforms when atom references
//! are updated.

use fold_node::atom::{Atom, AtomRef, AtomRefBehavior, AtomStatus};
use fold_node::schema::types::Transform;
use fold_node::schema::transform::{TransformRegistry, GetAtomFn, CreateAtomFn, UpdateAtomRefFn};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("Transform Registry Demo");
    
    // Create some atoms
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
    let mut aref1 = AtomRef::new(atom1.uuid().to_string(), "test_key".to_string());
    let mut aref2 = AtomRef::new(atom2.uuid().to_string(), "test_key".to_string());
    
    // Create a dummy output atom reference
    let mut output_aref = AtomRef::new("dummy".to_string(), "test_key".to_string());
    
    // Store atoms and atom references in hashmaps for our simple in-memory database
    let atoms = Arc::new(std::sync::Mutex::new(HashMap::new()));
    atoms.lock().unwrap().insert(atom1.uuid().to_string(), atom1.clone());
    atoms.lock().unwrap().insert(atom2.uuid().to_string(), atom2.clone());
    
    let atom_refs = Arc::new(std::sync::Mutex::new(HashMap::new()));
    atom_refs.lock().unwrap().insert("input1".to_string(), aref1.clone());
    atom_refs.lock().unwrap().insert("input2".to_string(), aref2.clone());
    atom_refs.lock().unwrap().insert("output".to_string(), output_aref.clone());
    
    // Create callback functions for the transform registry
    // Create a struct to hold our state
    struct AtomStore {
        atoms: Arc<std::sync::Mutex<HashMap<String, Atom>>>,
        atom_refs: Arc<std::sync::Mutex<HashMap<String, AtomRef>>>,
    }
    
    let store = Arc::new(AtomStore {
        atoms,
        atom_refs,
    });
    
    // Create callback functions that capture the store by reference
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
    
    // Create a transform registry
    let registry = Arc::new(TransformRegistry::new(
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
    ));
    
    // Create a transform
    let transform = Transform::new(
        "input1 + input2".to_string(),
        false,
        None,
        false,
    );
    
    // Register the transform
    println!("Registering transform: input1 + input2");
    let result = registry.register_transform(
        "test_transform".to_string(),
        transform,
        vec!["input1".to_string(), "input2".to_string()],
        "output".to_string(),
    );
    
    if result.is_ok() {
        println!("Transform registered successfully");
    } else {
        println!("Failed to register transform: {:?}", result.err());
        return;
    }
    
    // Check the dependent transforms
    let dependent_transforms = registry.get_dependent_transforms("input1");
    println!("Transforms dependent on input1: {:?}", dependent_transforms);
    
    // Execute the transform by updating an input atom reference
    println!("\nUpdating input atom reference to trigger transform execution...");
    
    // Create a new atom with an updated value
    let new_atom = Atom::new(
        "test_schema".to_string(),
        "test_key".to_string(),
        json!(15),
    );
    
    // Add the new atom to our in-memory database
    store.atoms.lock().unwrap().insert(new_atom.uuid().to_string(), new_atom.clone());
    
    // Update the atom reference
    println!("Updating atom reference input1 to point to new atom with value 15");
    
    // Manually trigger the transform execution
    let results = registry.handle_atom_ref_update("input1");
    
    // Check the results
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("Transform {} executed successfully: {}", i, value),
            Err(e) => println!("Transform {} execution failed: {}", i, e),
        }
    }
    
    println!("\nDemo completed.");
}