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
    
    // Create temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_db");
    let db_ops = Arc::new(DbOperations::new(db_path.to_str().unwrap())?);
    let message_bus = Arc::new(MessageBus::new());
    
    // Create TransformBase schema with test values
    let mut transform_base_schema = Schema::new("TransformBase".to_string());
    
    // Create value1 field = 15
    let value1_atom = Atom::new("TransformBase".to_string(), "test_user".to_string(), json!(15));
    let value1_atom_uuid = uuid::Uuid::new_v4().to_string();
    db_ops.store_item(&format!("atom:{}", value1_atom_uuid), &value1_atom)?;
    
    let value1_ref_uuid = uuid::Uuid::new_v4().to_string();
    let value1_ref = AtomRef::new(value1_ref_uuid.clone(), value1_atom_uuid.clone());
    db_ops.store_item(&format!("ref:{}", value1_ref_uuid), &value1_ref)?;
    
    let mut value1_field = SingleField { inner: FieldCommon::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    )};
    value1_field.set_ref_atom_uuid(value1_ref_uuid.clone());
    transform_base_schema.fields.insert("value1".to_string(), FieldVariant::Single(value1_field));
    
    // Create value2 field = 25
    let value2_atom = Atom::new("TransformBase".to_string(), "test_user".to_string(), json!(25));
    let value2_atom_uuid = uuid::Uuid::new_v4().to_string();
    db_ops.store_item(&format!("atom:{}", value2_atom_uuid), &value2_atom)?;
    
    let value2_ref_uuid = uuid::Uuid::new_v4().to_string();
    let value2_ref = AtomRef::new(value2_ref_uuid.clone(), value2_atom_uuid.clone());
    db_ops.store_item(&format!("ref:{}", value2_ref_uuid), &value2_ref)?;
    
    let mut value2_field = SingleField { inner: FieldCommon::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    )};
    value2_field.set_ref_atom_uuid(value2_ref_uuid.clone());
    transform_base_schema.fields.insert("value2".to_string(), FieldVariant::Single(value2_field));
    
    // Store TransformBase schema
    db_ops.store_schema("TransformBase", &transform_base_schema)?;
    
    // Create TransformSchema with result field
    let mut transform_schema = Schema::new("TransformSchema".to_string());
    let result_field = SingleField { inner: FieldCommon::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::default(),
        HashMap::new(),
    )};
    transform_schema.fields.insert("result".to_string(), FieldVariant::Single(result_field));
    db_ops.store_schema("TransformSchema", &transform_schema)?;
    
    // Create and store a transform
    let mut transform = Transform::new(
        "TransformBase.value1 + TransformBase.value2".to_string(),
        "TransformSchema.result".to_string(),
    );
    transform.set_inputs(vec!["TransformBase.value1".to_string(), "TransformBase.value2".to_string()]);
    db_ops.store_transform("test_transform", &transform)?;
    
    println!("✅ Set up test data: value1=15, value2=25, expected result=40");
    
    // Test the transform execution with different correlation ID patterns
    let test_patterns = vec![
        "trigger_cb764c7b-e24f-4a57-a4be-fdd9efb02508_TransformBase.value1",
        "transform_triggered_TransformSchema.result",
        "api_request_TransformSchema.result",
    ];
    
    for correlation_id in test_patterns {
        println!("\n🔧 Testing correlation_id: '{}'", correlation_id);
        
        let (count, success, error) = TransformManager::execute_transform_from_correlation_with_db(
            correlation_id,
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
                                } else {
                                    println!("⚠️  Unexpected result: expected 40, got {}", atom.content());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("\n🎉 Transform execution implementation test completed!");
    Ok(())
}