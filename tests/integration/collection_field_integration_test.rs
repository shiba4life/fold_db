//! Integration tests for collection field operations
//! Tests the complete functionality of AtomRefCollection through the FoldDB API

use fold_db::atom::{Atom, AtomRefCollection, CollectionOperation};
use fold_db::db_operations::DbOperations;
use fold_db::schema::field_factory::FieldFactory;
use fold_db::schema::types::field::FieldVariant;
use fold_db::schema::types::Schema;
use serde_json::json;
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_collection_field_operations() {
    println!("🧪 TEST: Collection Field Operations");
    
    // Setup
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let db = sled::Config::new()
        .path(temp_dir.path())
        .temporary(true)
        .open()
        .expect("Failed to open database");
    
    let db_ops = DbOperations::new(db).expect("Failed to create DbOperations");
    
    // Test 1: Create collection with add operation
    {
        println!("📝 Test 1: Create and add to collection");
        
        let atom1 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"value": 1}));
        let atom1_uuid = atom1.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom1_uuid), &atom1).expect("Failed to store atom1");
        
        let aref_uuid = "test_collection_1";
        let collection = db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom1_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to create collection");
        
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&atom1_uuid));
        println!("✅ Collection created with 1 item");
    }
    
    // Test 2: Add multiple items
    {
        println!("📝 Test 2: Add multiple items to collection");
        
        let aref_uuid = "test_collection_2";
        
        // Add three atoms
        for i in 1..=3 {
            let atom = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"value": i}));
            let atom_uuid = atom.uuid().to_string();
            db_ops.store_item(&format!("atom:{}", atom_uuid), &atom).expect("Failed to store atom");
            
            db_ops.update_atom_ref_collection(
                aref_uuid,
                CollectionOperation::Add { atom_uuid },
                "user1".to_string(),
            ).expect("Failed to add to collection");
        }
        
        // Verify collection has 3 items
        let collection = db_ops.get_item::<AtomRefCollection>(&format!("ref:{}", aref_uuid))
            .expect("Failed to load collection")
            .expect("Collection should exist");
        
        assert_eq!(collection.len(), 3);
        println!("✅ Collection has 3 items");
    }
    
    // Test 3: Update by index
    {
        println!("📝 Test 3: Update collection item by index");
        
        let aref_uuid = "test_collection_3";
        
        // Create collection with 2 items
        let atom1 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"original": true}));
        let atom1_uuid = atom1.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom1_uuid), &atom1).expect("Failed to store atom1");
        
        let atom2 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"original": true}));
        let atom2_uuid = atom2.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom2_uuid), &atom2).expect("Failed to store atom2");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom1_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to add first item");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom2_uuid },
            "user1".to_string(),
        ).expect("Failed to add second item");
        
        // Create replacement atom
        let atom_new = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"updated": true}));
        let atom_new_uuid = atom_new.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom_new_uuid), &atom_new).expect("Failed to store new atom");
        
        // Update index 0
        let collection = db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::UpdateByIndex { index: 0, atom_uuid: atom_new_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to update by index");
        
        assert_eq!(collection.get_atom_uuid_at(0), Some(&atom_new_uuid));
        assert_ne!(collection.get_atom_uuid_at(0), Some(&atom1_uuid));
        println!("✅ Successfully updated item at index 0");
    }
    
    // Test 4: Insert at index
    {
        println!("📝 Test 4: Insert at specific index");
        
        let aref_uuid = "test_collection_4";
        
        // Create collection with 2 items
        let atom1 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"position": 1}));
        let atom1_uuid = atom1.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom1_uuid), &atom1).expect("Failed to store atom");
        
        let atom3 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"position": 3}));
        let atom3_uuid = atom3.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom3_uuid), &atom3).expect("Failed to store atom");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom1_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to add first");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom3_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to add second");
        
        // Insert at index 1
        let atom2 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"position": 2}));
        let atom2_uuid = atom2.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom2_uuid), &atom2).expect("Failed to store atom");
        
        let collection = db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Insert { index: 1, atom_uuid: atom2_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to insert at index");
        
        assert_eq!(collection.len(), 3);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&atom1_uuid));
        assert_eq!(collection.get_atom_uuid_at(1), Some(&atom2_uuid));
        assert_eq!(collection.get_atom_uuid_at(2), Some(&atom3_uuid));
        println!("✅ Successfully inserted item at index 1");
    }
    
    // Test 5: Remove item
    {
        println!("📝 Test 5: Remove item from collection");
        
        let aref_uuid = "test_collection_5";
        
        // Add two items
        let atom1 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"keep": true}));
        let atom1_uuid = atom1.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom1_uuid), &atom1).expect("Failed to store atom");
        
        let atom2 = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"remove": true}));
        let atom2_uuid = atom2.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom2_uuid), &atom2).expect("Failed to store atom");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom1_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to add first");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom2_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to add second");
        
        // Remove the second item
        let collection = db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Remove { atom_uuid: atom2_uuid },
            "user1".to_string(),
        ).expect("Failed to remove item");
        
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&atom1_uuid));
        println!("✅ Successfully removed item");
    }
    
    // Test 6: Clear collection
    {
        println!("📝 Test 6: Clear collection");
        
        let aref_uuid = "test_collection_6";
        
        // Add items
        for i in 1..=5 {
            let atom = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"value": i}));
            let atom_uuid = atom.uuid().to_string();
            db_ops.store_item(&format!("atom:{}", atom_uuid), &atom).expect("Failed to store atom");
            
            db_ops.update_atom_ref_collection(
                aref_uuid,
                CollectionOperation::Add { atom_uuid },
                "user1".to_string(),
            ).expect("Failed to add to collection");
        }
        
        // Clear the collection
        let collection = db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Clear,
            "user1".to_string(),
        ).expect("Failed to clear collection");
        
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
        println!("✅ Successfully cleared collection");
    }
    
    // Test 7: Load from disk
    {
        println!("📝 Test 7: Load collection from disk");
        
        let aref_uuid = "test_collection_persist";
        
        // Create and populate collection
        let atom = Atom::new("TestSchema".to_string(), "user1".to_string(), json!({"persistent": true}));
        let atom_uuid = atom.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", atom_uuid), &atom).expect("Failed to store atom");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom_uuid.clone() },
            "user1".to_string(),
        ).expect("Failed to create collection");
        
        // Load from disk
        let loaded_collection = db_ops.get_item::<AtomRefCollection>(&format!("ref:{}", aref_uuid))
            .expect("Failed to load from disk")
            .expect("Collection should exist");
        
        assert_eq!(loaded_collection.len(), 1);
        assert_eq!(loaded_collection.get_atom_uuid_at(0), Some(&atom_uuid));
        println!("✅ Successfully loaded collection from disk");
    }
    
    println!("✅ All Collection Field Operations Tests PASSED");
}

#[test]
fn test_collection_field_array_atom_creation() {
    println!("🧪 TEST: Collection Field Array Atom Creation");
    println!("   Verifies that individual atoms are created for each array element");
    
    // Setup
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let db = sled::Config::new()
        .path(temp_dir.path())
        .temporary(true)
        .open()
        .expect("Failed to open database");
    
    let db_ops = DbOperations::new(db).expect("Failed to create DbOperations");
    
    // Test array value processing
    {
        println!("📝 Creating collection from array value");
        
        // Create a FieldValueSetRequest simulator - in real usage this would come through the message bus
        let array_values = json!([
            {"id": 1, "name": "First"},
            {"id": 2, "name": "Second"},
            {"id": 3, "name": "Third"}
        ]);
        
        // Manually simulate what would happen in field processing
        let aref_uuid = "test_array_collection";
        let mut collection = AtomRefCollection::new("test_user".to_string());
        
        // Create individual atoms for each array element
        let array = array_values.as_array().unwrap();
        let mut created_atoms = Vec::new();
        
        for (index, element) in array.iter().enumerate() {
            let atom = Atom::new("CollectionItem".to_string(), "test_user".to_string(), element.clone());
            let atom_uuid = atom.uuid().to_string();
            
            // Store the atom
            db_ops.store_item(&format!("atom:{}", atom_uuid), &atom)
                .expect(&format!("Failed to store atom {}", index));
            
            // Add to collection
            collection.add_atom_uuid(atom_uuid.clone(), "test_user".to_string());
            created_atoms.push((atom_uuid, element.clone()));
            
            println!("  ✓ Created atom {} for element {}: {}", 
                atom_uuid, index, element);
        }
        
        // Store the collection
        db_ops.store_item(&format!("ref:{}", aref_uuid), &collection)
            .expect("Failed to store collection");
        
        println!("  ✓ Stored collection with {} atoms", collection.len());
        
        // Verify each atom has the correct content
        for (atom_uuid, expected_content) in &created_atoms {
            let stored_atom = db_ops.get_item::<Atom>(&format!("atom:{}", atom_uuid))
                .expect("Failed to load atom")
                .expect("Atom should exist");
            
            assert_eq!(stored_atom.content(), expected_content);
            println!("  ✓ Verified atom {} has correct content", atom_uuid);
        }
        
        // Verify collection has all atom UUIDs
        assert_eq!(collection.len(), 3);
        for (i, (atom_uuid, _)) in created_atoms.iter().enumerate() {
            assert_eq!(collection.get_atom_uuid_at(i), Some(atom_uuid));
        }
        
        println!("✅ Array elements correctly stored as individual atoms");
    }
    
    // Test updating individual elements
    {
        println!("📝 Testing individual element updates");
        
        let aref_uuid = "test_update_collection";
        
        // Create initial collection with 3 items
        let initial_values = vec![
            json!({"value": "original_1"}),
            json!({"value": "original_2"}),
            json!({"value": "original_3"}),
        ];
        
        let mut atom_uuids = Vec::new();
        for value in &initial_values {
            let atom = Atom::new("CollectionItem".to_string(), "test_user".to_string(), value.clone());
            let atom_uuid = atom.uuid().to_string();
            db_ops.store_item(&format!("atom:{}", atom_uuid), &atom).expect("Failed to store atom");
            atom_uuids.push(atom_uuid);
        }
        
        // Create collection with initial atoms
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom_uuids[0].clone() },
            "test_user".to_string(),
        ).expect("Failed to add first");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom_uuids[1].clone() },
            "test_user".to_string(),
        ).expect("Failed to add second");
        
        db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::Add { atom_uuid: atom_uuids[2].clone() },
            "test_user".to_string(),
        ).expect("Failed to add third");
        
        // Update element at index 1 with new value
        let new_value = json!({"value": "updated_2"});
        let new_atom = Atom::new("CollectionItem".to_string(), "test_user".to_string(), new_value.clone());
        let new_atom_uuid = new_atom.uuid().to_string();
        db_ops.store_item(&format!("atom:{}", new_atom_uuid), &new_atom).expect("Failed to store new atom");
        
        let collection = db_ops.update_atom_ref_collection(
            aref_uuid,
            CollectionOperation::UpdateByIndex { index: 1, atom_uuid: new_atom_uuid.clone() },
            "test_user".to_string(),
        ).expect("Failed to update by index");
        
        // Verify the update
        assert_eq!(collection.get_atom_uuid_at(1), Some(&new_atom_uuid));
        
        // Verify the new atom has the updated content
        let updated_atom = db_ops.get_item::<Atom>(&format!("atom:{}", new_atom_uuid))
            .expect("Failed to load updated atom")
            .expect("Updated atom should exist");
        
        assert_eq!(updated_atom.content(), &new_value);
        
        println!("✅ Individual element update successful");
    }
    
    println!("✅ Collection Field Array Atom Creation Test PASSED");
}

#[test]
fn test_collection_field_in_schema() {
    println!("🧪 TEST: Collection Field in Schema");
    
    // Setup
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let db = sled::Config::new()
        .path(temp_dir.path())
        .temporary(true)
        .open()
        .expect("Failed to open database");
    
    let db_ops = DbOperations::new(db).expect("Failed to create DbOperations");
    
    // Create schema with collection field
    let mut schema = Schema::new("BlogPost".to_string());
    schema.fields.insert(
        "tags".to_string(),
        FieldFactory::create_collection_variant(),
    );
    schema.fields.insert(
        "comments".to_string(),
        FieldFactory::create_collection_variant(),
    );
    
    // Verify collection fields exist
    assert!(matches!(schema.fields.get("tags"), Some(FieldVariant::Collection(_))));
    assert!(matches!(schema.fields.get("comments"), Some(FieldVariant::Collection(_))));
    
    println!("✅ Schema with collection fields created successfully");
}