//! Auto Transform Demo
//!
//! This example demonstrates how transforms can automatically collect their inputs.

use fold_node::atom::{Atom, AtomRef, AtomRefBehavior, AtomStatus};
use fold_node::schema::transform::{TransformRegistry, GetAtomFn, CreateAtomFn, UpdateAtomRefFn};
use fold_node::schema::types::Transform;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() {
    println!("Auto Transform Demo");
    
    // Create a simple in-memory store for atoms and atom references
    let atoms = Arc::new(Mutex::new(HashMap::new()));
    let atom_refs = Arc::new(Mutex::new(HashMap::new()));
    
    // Create some atoms with test data
    let temperature_atom = Atom::new(
        "sensor_data".to_string(),
        "system".to_string(),
        json!(25.0),  // 25°C
    );
    
    let humidity_atom = Atom::new(
        "sensor_data".to_string(),
        "system".to_string(),
        json!(60.0),  // 60% humidity
    );
    
    // Store the atoms
    atoms.lock().unwrap().insert(temperature_atom.uuid().to_string(), temperature_atom.clone());
    atoms.lock().unwrap().insert(humidity_atom.uuid().to_string(), humidity_atom.clone());
    
    // Create atom references
    let temperature_ref = AtomRef::new(temperature_atom.uuid().to_string(), "system".to_string());
    let humidity_ref = AtomRef::new(humidity_atom.uuid().to_string(), "system".to_string());
    let output_ref = AtomRef::new("dummy".to_string(), "system".to_string());
    
    // Store the atom references
    atom_refs.lock().unwrap().insert("temperature".to_string(), temperature_ref);
    atom_refs.lock().unwrap().insert("humidity".to_string(), humidity_ref);
    atom_refs.lock().unwrap().insert("fahrenheit".to_string(), output_ref);
    
    // Create callback functions for the transform registry
    let atoms_clone = Arc::clone(&atoms);
    let atom_refs_clone = Arc::clone(&atom_refs);
    let get_atom_fn: GetAtomFn = Arc::new(move |aref_uuid: &str| -> Result<Atom, Box<dyn std::error::Error>> {
        let atom_refs_guard = atom_refs_clone.lock().unwrap();
        if let Some(aref) = atom_refs_guard.get(aref_uuid) {
            let atoms_guard = atoms_clone.lock().unwrap();
            if let Some(atom) = atoms_guard.get(aref.get_atom_uuid()) {
                return Ok(atom.clone());
            }
        }
        Err(format!("Atom not found for reference: {}", aref_uuid).into())
    });
    
    let atoms_clone = Arc::clone(&atoms);
    let create_atom_fn: CreateAtomFn = Arc::new(move |schema_name: &str, source_pub_key: String, _prev_atom_uuid: Option<String>, content: JsonValue, _status: Option<AtomStatus>| -> Result<Atom, Box<dyn std::error::Error>> {
        let atom = Atom::new(
            schema_name.to_string(),
            source_pub_key,
            content,
        );
        let mut atoms_guard = atoms_clone.lock().unwrap();
        atoms_guard.insert(atom.uuid().to_string(), atom.clone());
        Ok(atom)
    });
    
    let atom_refs_clone = Arc::clone(&atom_refs);
    let update_atom_ref_fn: UpdateAtomRefFn = Arc::new(move |aref_uuid: &str, atom_uuid: String, source_pub_key: String| -> Result<AtomRef, Box<dyn std::error::Error>> {
        let mut atom_refs_guard = atom_refs_clone.lock().unwrap();
        if let Some(aref) = atom_refs_guard.get_mut(aref_uuid) {
            aref.set_atom_uuid(atom_uuid);
            return Ok(aref.clone());
        }
        Err(format!("Atom reference not found: {}", aref_uuid).into())
    });
    
    // Create a transform registry
    let registry = Arc::new(TransformRegistry::new(
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
    ));
    
    // Create a transform to convert Celsius to Fahrenheit
    // Formula: F = C * 1.8 + 32
    let transform = Transform::new(
        "temperature * 1.8 + 32 + humidity / 100".to_string(),
        false,
        None,
        false,
    );
    
    // Method 1: Register with explicit dependencies
    println!("\nMethod 1: Register with explicit dependencies");
    let result = registry.register_transform(
        "celsius_to_fahrenheit".to_string(),
        transform.clone(),
        vec!["temperature".to_string(), "humidity".to_string()],
        "fahrenheit".to_string(),
    );
    
    if result.is_ok() {
        println!("Transform registered successfully with explicit dependencies");
    } else {
        println!("Failed to register transform: {:?}", result.err());
        return;
    }
    
    // Execute the transform
    let results = registry.handle_atom_ref_update("temperature");
    
    if let Some(Ok(result)) = results.first() {
        println!("Transform result: {}", result);
        println!("25°C with 60% humidity = {}°F", result);
    } else {
        println!("Transform execution failed");
    }
    
    // Method 2: Register with automatic dependency detection
    println!("\nMethod 2: Register with automatic dependency detection");
    
    // First, unregister the previous transform
    registry.unregister_transform("celsius_to_fahrenheit");
    
    // Create a new transform with the same logic
    let transform = Transform::new(
        "temperature * 1.8 + 32 + humidity / 100".to_string(),
        false,
        None,
        false,
    );
    
    // Register with automatic dependency detection
    let result = registry.register_transform_auto(
        "celsius_to_fahrenheit_auto".to_string(),
        transform,
        "fahrenheit".to_string(),
    );
    
    if result.is_ok() {
        println!("Transform registered successfully with automatic dependency detection");
    } else {
        println!("Failed to register transform: {:?}", result.err());
        return;
    }
    
    // Check what dependencies were detected
    let transform_inputs = registry.get_transform_inputs("celsius_to_fahrenheit_auto");
    println!("Detected dependencies: {:?}", transform_inputs);
    
    // Execute the transform
    let results = registry.handle_atom_ref_update("temperature");
    
    if let Some(Ok(result)) = results.first() {
        println!("Transform result: {}", result);
        println!("25°C with 60% humidity = {}°F", result);
    } else {
        println!("Transform execution failed");
    }
    
    // Update the temperature and see the transform execute automatically
    println!("\nUpdating temperature to 30°C");
    
    // Create a new atom with updated temperature
    let new_temp_atom = Atom::new(
        "sensor_data".to_string(),
        "system".to_string(),
        json!(30.0),  // 30°C
    );
    
    // Store the new atom
    atoms.lock().unwrap().insert(new_temp_atom.uuid().to_string(), new_temp_atom.clone());
    
    // Update the temperature reference
    let mut atom_refs_guard = atom_refs.lock().unwrap();
    if let Some(aref) = atom_refs_guard.get_mut("temperature") {
        aref.set_atom_uuid(new_temp_atom.uuid().to_string());
        println!("Temperature reference updated");
    }
    
    // Execute the transform manually (in a real system, this would happen automatically)
    drop(atom_refs_guard); // Release the lock before calling handle_atom_ref_update
    let results = registry.handle_atom_ref_update("temperature");
    
    if let Some(Ok(result)) = results.first() {
        println!("Transform result: {}", result);
        println!("30°C with 60% humidity = {}°F", result);
    } else {
        println!("Transform execution failed");
    }
    
    println!("\nDemo completed.");
}