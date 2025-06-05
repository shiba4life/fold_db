use fold_node::fold_db_core::infrastructure::message_bus::MessageBus;
use fold_node::fold_db_core::transform_manager::TransformManager;
use fold_node::db_operations::DbOperations;
use fold_node::schema::types::{Schema, Transform};
use fold_node::schema::types::field::single_field::SingleField;
use fold_node::schema::types::field::variant::FieldVariant;
use fold_node::schema::types::field::common::{Field, FieldCommon};
use fold_node::atom::{Atom, AtomRef};
use fold_node::permissions::types::policy::PermissionsPolicy;
use fold_node::fees::types::config::FieldPaymentConfig;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Testing Transform Execution Implementation");
    
    // Use centralized database initialization - eliminates duplicate setup pattern
    let test_env = fold_node::schema::DatabaseInitHelper::create_test_environment()?;
    let db_ops = &test_env.db_ops;
    let message_bus = &test_env.message_bus;
    
    // Create TransformBase schema with test values
    let mut transform_base_schema = Schema::new("TransformBase".to_string());
    
    // Create value1 field = 15
    let value1_atom = Atom::new("TransformBase".to_string(), "test_user".to_string(), json!(15));
    let value1_atom_uuid = uuid::Uuid::new_v4().to_string();
    db_ops.store_item(&format!("atom:{}", value1_atom_uuid), &value1_atom)?;
    
    let value1_ref_uuid = uuid::Uuid::new_v4().to_string();
    let value1_ref = AtomRef::new(value1_ref_uuid.clone(), value1_atom_uuid.clone());
    db_ops.store_item(&format!("ref:{}", value1_ref_uuid), &value1_ref)?;
    
    // Use centralized field factory - eliminates 18+ field creation duplicate patterns
    let mut value1_field = fold_node::schema::FieldFactory::create_single_field();
    value1_field.set_ref_atom_uuid(value1_ref_uuid.clone());
    transform_base_schema.fields.insert("value1".to_string(), FieldVariant::Single(value1_field));
    
    // Create value2 field = 25
    let value2_atom = Atom::new("TransformBase".to_string(), "test_user".to_string(), json!(25));
    let value2_atom_uuid = uuid::Uuid::new_v4().to_string();
    db_ops.store_item(&format!("atom:{}", value2_atom_uuid), &value2_atom)?;
    
    let value2_ref_uuid = uuid::Uuid::new_v4().to_string();
    let value2_ref = AtomRef::new(value2_ref_uuid.clone(), value2_atom_uuid.clone());
    db_ops.store_item(&format!("ref:{}", value2_ref_uuid), &value2_ref)?;
    
    // Use centralized field factory - eliminates duplicate field creation patterns
    let mut value2_field = fold_node::schema::FieldFactory::create_single_field();
    value2_field.set_ref_atom_uuid(value2_ref_uuid.clone());
    transform_base_schema.fields.insert("value2".to_string(), FieldVariant::Single(value2_field));
    
    // Store TransformBase schema
    db_ops.store_schema("TransformBase", &transform_base_schema)?;
    
    // Use centralized schema setup - eliminates duplicate schema creation patterns
    fold_node::schema::TransformSetupHelper::create_transform_test_schema(
        "TransformSchema",
        vec![("result", serde_json::json!(null))],
        &db_ops,
    )?;
    
    // Create and store a transform
    let mut transform = Transform::new(
        "TransformBase.value1 + TransformBase.value2".to_string(),
        "TransformSchema.result".to_string(),
    );
    transform.set_inputs(vec!["TransformBase.value1".to_string(), "TransformBase.value2".to_string()]);
    db_ops.store_transform("test_transform", &transform)?;
    
    println!("✅ Set up test data: value1=15, value2=25, expected result=40");
    
    // Test direct transform execution using new event-driven architecture
    println!("\n🔧 Testing direct transform execution");
    
    let (count, success, error) = TransformManager::execute_transform_with_db(
        "test_transform",
        &message_bus,
        Some(&db_ops),
    );
    
    println!("📊 Result: count={}, success={}, error={:?}", count, success, error);
    
    if success {
        // Check if TransformSchema.result was updated
        if let Ok(Some(updated_schema)) = db_ops.get_schema("TransformSchema") {
            if let Some(result_field) = updated_schema.fields.get("result") {
                if let Some(ref_uuid) = result_field.ref_atom_uuid() {
                    if let Ok(Some(atom_ref)) = db_ops.get_item::<AtomRef>(&format!("ref:{}", ref_uuid)) {
                        let atom_uuid = atom_ref.get_atom_uuid();
                        if let Ok(Some(atom)) = db_ops.get_item::<Atom>(&format!("atom:{}", atom_uuid)) {
                            println!("✅ Found computed result: {}", atom.content());
                            
                            if *atom.content() == json!(40) {
                                println!("🎯 PERFECT! Transform computed 15 + 25 = 40 correctly!");
                                println!("✅ Result storage and field linking working perfectly!");
                            } else {
                                println!("⚠️ Unexpected result: expected 40, got {}", atom.content());
                            }
                        } else {
                            println!("❌ Atom not found for uuid: {}", atom_uuid);
                        }
                    } else {
                        println!("❌ AtomRef not found for ref_uuid: {}", ref_uuid);
                    }
                } else {
                    println!("❌ No ref_atom_uuid found in result field");
                }
            } else {
                println!("❌ Result field not found in TransformSchema");
            }
        } else {
            println!("❌ TransformSchema not found or could not be loaded");
        }
    } else {
        println!("❌ Transform execution failed");
    }
    
    // Test result queryability
    println!("🔍 Testing result queryability...");
    if let Ok(Some(schema)) = db_ops.get_schema("TransformSchema") {
        if let Some(result_field) = schema.fields.get("result") {
            if let Some(ref_uuid) = result_field.ref_atom_uuid() {
                println!("✅ TransformSchema.result is queryable with ref_uuid: {}", ref_uuid);
            } else {
                println!("❌ TransformSchema.result has no ref_atom_uuid - not queryable");
            }
        }
    }
    
    println!("\n🎉 Transform execution implementation test completed!");
    Ok(())
}