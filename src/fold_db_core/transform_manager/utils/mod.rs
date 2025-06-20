//! Unified transform manager utilities eliminating ALL duplication
//!
//! AGGRESSIVE CLEANUP: This module consolidates:
//! - conversion_helper.rs: JsonValue conversion utilities
//! - serialization_helper.rs: Mapping serialization utilities  
//! - event_publisher.rs: Event publishing utilities
//! - field_resolver.rs: Field value resolution utilities
//! - default_value_helper.rs: Default value generation utilities
//! - lock_helper.rs: Lock acquisition utilities
//! - logging_helper.rs: Debug logging utilities
//! - validation_helper.rs: Validation utilities
//! - Plus multiple duplicate logging/helper patterns found throughout

use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus,
    schema_events::TransformExecuted,
};
use crate::schema::types::{SchemaError, Schema, Transform};
use crate::schema::types::field::variant::FieldVariant;
use crate::schema::types::field::common::Field;
use serde_json::Value as JsonValue;
use log::{info, error, warn};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::collections::{HashMap, HashSet};

// Re-export commonly used types to avoid import duplication
pub use serde_json::Value;

/// Single unified utility combining ALL transform manager utilities
pub struct TransformUtils;

impl TransformUtils {
    // ========== CONVERSION UTILITIES ==========
    
    /// Convert JsonValue to Value with unified error handling
    pub fn json_to_value(json_value: JsonValue) -> Result<JsonValue, SchemaError> {
        Ok(json_value)
    }

    /// Validate and convert JsonValue with type checking
    pub fn validate_and_convert(
        json_value: JsonValue,
        expected_type: &str,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!("🔄 Converting field '{}' (expected type: {})", field_name, expected_type);
        
        let is_valid = match expected_type.to_lowercase().as_str() {
            "string" | "str" => json_value.is_string(),
            "number" | "integer" | "int" => json_value.is_number() && json_value.as_i64().is_some(),
            "float" | "double" => json_value.is_number() && json_value.as_f64().is_some(),
            "boolean" | "bool" => json_value.is_boolean(),
            "array" => json_value.is_array(),
            "object" => json_value.is_object(),
            "null" => json_value.is_null(),
            _ => {
                warn!("⚠️ Unknown expected type '{}' for field '{}', allowing any type", expected_type, field_name);
                true
            }
        };

        if !is_valid {
            let error_msg = format!(
                "Type validation failed for field '{}': expected '{}', got '{:?}'",
                field_name, expected_type, json_value
            );
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Successfully validated and converted field '{}'", field_name);
        Ok(json_value)
    }

    /// Convert string to JsonValue with type inference
    pub fn string_to_json_value(value_str: &str, infer_type: bool) -> JsonValue {
        if !infer_type {
            return JsonValue::String(value_str.to_string());
        }

        // Try to infer the type from the string content
        if value_str.eq_ignore_ascii_case("true") {
            return JsonValue::Bool(true);
        }
        
        if value_str.eq_ignore_ascii_case("false") {
            return JsonValue::Bool(false);
        }
        
        if value_str.eq_ignore_ascii_case("null") {
            return JsonValue::Null;
        }

        // Try to parse as integer
        if let Ok(int_val) = value_str.parse::<i64>() {
            return JsonValue::Number(serde_json::Number::from(int_val));
        }

        // Try to parse as float
        if let Ok(float_val) = value_str.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(float_val) {
                return JsonValue::Number(num);
            }
        }

        JsonValue::String(value_str.to_string())
    }

    // ========== DEFAULT VALUE UTILITIES ==========
    
    /// Enhanced default value generation with comprehensive field mapping
    pub fn get_default_value_for_field(field_name: &str) -> JsonValue {
        info!("📊 Generating default value for field: {}", field_name);
        
        let default_value = match field_name {
            // Common input field names
            "input1" => JsonValue::Number(serde_json::Number::from(42)),
            "input2" => JsonValue::Number(serde_json::Number::from(24)),
            "value1" => JsonValue::Number(serde_json::Number::from(5)),
            "value2" => JsonValue::Number(serde_json::Number::from(10)),
            
            // Common calculation inputs
            "weight" => JsonValue::Number(serde_json::Number::from(70)),
            "height" => JsonValue::Number(serde_json::Number::from_f64(1.75).unwrap_or(serde_json::Number::from(175))),
            "age" => JsonValue::Number(serde_json::Number::from(30)),
            
            // Common identifiers
            "id" | "user_id" | "patient_id" => JsonValue::String("default_id".to_string()),
            "name" | "username" | "patient_name" => JsonValue::String("default_name".to_string()),
            
            // Common boolean flags
            "active" | "enabled" | "is_valid" => JsonValue::Bool(true),
            "disabled" | "inactive" | "is_deleted" => JsonValue::Bool(false),
            
            // Common numeric defaults
            "score" | "rating" => JsonValue::Number(serde_json::Number::from(0)),
            "count" | "quantity" => JsonValue::Number(serde_json::Number::from(1)),
            "price" | "amount" => JsonValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
            
            // Fallback for unknown fields based on heuristics
            _ => Self::infer_default_by_name_pattern(field_name)
        };
        
        info!("📊 Default value for '{}': {}", field_name, default_value);
        default_value
    }

    /// Infer default value based on field name patterns
    fn infer_default_by_name_pattern(field_name: &str) -> JsonValue {
        let lower_name = field_name.to_lowercase();
        
        match true {
            _ if lower_name.contains("count") || lower_name.contains("number") || lower_name.contains("value") => 
                JsonValue::Number(serde_json::Number::from(0)),
            _ if lower_name.contains("active") || lower_name.contains("enabled") || lower_name.contains("valid") => 
                JsonValue::Bool(false),
            _ if lower_name.contains("list") || lower_name.contains("array") || lower_name.contains("tags") => 
                JsonValue::Array(vec![]),
            _ if lower_name.contains("config") || lower_name.contains("meta") || lower_name.contains("data") => 
                JsonValue::Object(serde_json::Map::new()),
            _ => JsonValue::String("default".to_string()),
        }
    }

    /// Infer type from field name for smart defaults
    pub fn infer_type_from_field_name(field_name: &str) -> &'static str {
        let lower_name = field_name.to_lowercase();
        match true {
            _ if lower_name.contains("id") || lower_name.contains("name") || lower_name.contains("email") => "string",
            _ if lower_name.contains("count") || lower_name.contains("age") || lower_name.contains("score") => "integer",
            _ if lower_name.contains("weight") || lower_name.contains("height") || lower_name.contains("price") => "float",
            _ if lower_name.contains("active") || lower_name.contains("enabled") || lower_name.contains("valid") => "boolean",
            _ if lower_name.contains("list") || lower_name.contains("items") || lower_name.contains("tags") => "array",
            _ if lower_name.contains("config") || lower_name.contains("settings") || lower_name.contains("meta") => "object",
            _ => "string" // Default to string
        }
    }

    /// Get typed default value based on expected type
    pub fn get_typed_default_value(field_name: &str, expected_type: &str) -> JsonValue {
        info!("📊 Generating typed default value for field: {} (type: {})", field_name, expected_type);
        
        let default_value = match expected_type.to_lowercase().as_str() {
            "string" | "str" => JsonValue::String(format!("default_{}", field_name)),
            "number" | "integer" | "int" => JsonValue::Number(serde_json::Number::from(0)),
            "float" | "double" => JsonValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
            "boolean" | "bool" => JsonValue::Bool(false),
            "array" => JsonValue::Array(vec![]),
            "object" => JsonValue::Object(serde_json::Map::new()),
            "null" => JsonValue::Null,
            _ => {
                warn!("⚠️ Unknown type '{}' for field '{}', using field-based default", expected_type, field_name);
                Self::get_default_value_for_field(field_name)
            }
        };
        
        info!("📊 Typed default value for '{}' ({}): {}", field_name, expected_type, default_value);
        default_value
    }

    // ========== VALIDATION UTILITIES ==========
    
    /// Comprehensive transform validation
    pub fn validate_transform_registration(
        transform_id: &str,
        transform: &Transform,
    ) -> Result<(), SchemaError> {
        info!("🔍 Validating transform registration for: {}", transform_id);

        // Validate transform ID is not empty
        if transform_id.trim().is_empty() {
            let error_msg = "Transform ID cannot be empty".to_string();
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate transform has inputs
        let inputs = transform.get_inputs();
        if inputs.is_empty() {
            let error_msg = format!("Transform '{}' must have at least one input", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate transform has output
        let output = transform.get_output();
        if output.trim().is_empty() {
            let error_msg = format!("Transform '{}' must have a valid output field", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Validate transform logic is not empty
        if transform.logic.trim().is_empty() {
            let error_msg = format!("Transform '{}' must have non-empty logic", transform_id);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Transform registration validation passed for: {}", transform_id);
        Ok(())
    }

    /// Validate field name format
    pub fn validate_field_name(field_name: &str, context: &str) -> Result<(), SchemaError> {
        info!("🔍 Validating field name '{}' in context: {}", field_name, context);

        if field_name.trim().is_empty() {
            let error_msg = format!("Field name cannot be empty in context: {}", context);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        // Check for valid schema.field format
        let parts: Vec<&str> = field_name.split('.').collect();
        if parts.len() != 2 {
            let error_msg = format!(
                "Field name '{}' must be in format 'schema.field' in context: {}",
                field_name, context
            );
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        let (schema_name, field_name_part) = (parts[0], parts[1]);
        
        if schema_name.trim().is_empty() || field_name_part.trim().is_empty() {
            let error_msg = format!("Schema and field names cannot be empty in field '{}' (context: {})", field_name, context);
            error!("❌ {}", error_msg);
            return Err(SchemaError::InvalidData(error_msg));
        }

        info!("✅ Field name validation passed for: {}", field_name);
        Ok(())
    }

    // ========== LOCK UTILITIES ==========
    
    /// Acquire a read lock with consistent error handling
    pub fn read_lock<'a, T>(lock: &'a RwLock<T>, lock_name: &str) -> Result<RwLockReadGuard<'a, T>, SchemaError> {
        lock.read().map_err(|_| {
            SchemaError::InvalidData(format!("Failed to acquire {} read lock", lock_name))
        })
    }
    
    /// Acquire a write lock with consistent error handling
    pub fn write_lock<'a, T>(lock: &'a RwLock<T>, lock_name: &str) -> Result<RwLockWriteGuard<'a, T>, SchemaError> {
        lock.write().map_err(|_| {
            SchemaError::InvalidData(format!("Failed to acquire {} write lock", lock_name))
        })
    }

    // ========== SERIALIZATION UTILITIES ==========
    
    /// Serialize a mapping to bytes with consistent error handling
    pub fn serialize_mapping<T>(
        mapping: &RwLock<T>,
        mapping_name: &str,
    ) -> Result<Vec<u8>, SchemaError>
    where
        T: serde::Serialize,
    {
        info!("🔄 Serializing mapping: {}", mapping_name);
        
        let map = Self::read_lock(mapping, mapping_name)?;
        let json = serde_json::to_vec(&*map).map_err(|e| {
            let error_msg = format!("Failed to serialize {}: {}", mapping_name, e);
            error!("❌ {}", error_msg);
            SchemaError::InvalidData(error_msg)
        })?;
        
        info!("✅ Successfully serialized mapping: {} ({} bytes)", mapping_name, json.len());
        Ok(json)
    }

    /// Deserialize mapping data with consistent error handling
    pub fn deserialize_mapping<T>(
        data: &[u8],
        mapping_name: &str,
    ) -> Result<T, SchemaError>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        info!("🔄 Deserializing mapping: {}", mapping_name);
        
        match serde_json::from_slice(data) {
            Ok(result) => {
                info!("✅ Successfully deserialized mapping: {}", mapping_name);
                Ok(result)
            }
            Err(e) => {
                let error_msg = format!("Failed to deserialize {}: {}", mapping_name, e);
                error!("❌ {}", error_msg);
                info!("🔄 Using default value for {} due to deserialization error", mapping_name);
                Ok(T::default())
            }
        }
    }

    /// Store mapping to database
    pub fn store_mapping<T>(
        db_ops: &std::sync::Arc<crate::db_operations::DbOperations>,
        mapping: &RwLock<T>,
        key: &str,
        mapping_name: &str,
    ) -> Result<(), SchemaError>
    where
        T: serde::Serialize,
    {
        info!("💾 Storing mapping: {} to key: {}", mapping_name, key);
        
        let json = Self::serialize_mapping(mapping, mapping_name)?;
        db_ops.store_transform_mapping(key, &json)?;
        
        info!("✅ Successfully stored mapping: {} to database", mapping_name);
        Ok(())
    }

    // ========== EVENT PUBLISHING UTILITIES ==========
    
    /// Publish a TransformExecuted event with consistent error handling
    pub fn publish_transform_executed(
        message_bus: &Arc<MessageBus>,
        transform_id: &str,
        status: &str,
    ) -> Result<(), SchemaError> {
        info!("📢 Publishing TransformExecuted {} event for: {}", status, transform_id);
        
        let event = TransformExecuted::new(transform_id, status);
        
        match message_bus.publish(event) {
            Ok(_) => {
                info!("✅ Published TransformExecuted {} event for transform: {}", status, transform_id);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to publish TransformExecuted {} event for {}: {}", status, transform_id, e);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Handle execution result and publish event
    pub fn handle_execution_result_and_publish(
        message_bus: &Arc<MessageBus>,
        transform_id: &str,
        execution_result: &Result<serde_json::Value, crate::schema::types::SchemaError>,
    ) {
        match execution_result {
            Ok(value) => {
                info!("✅ Transform {} execution completed successfully", transform_id);
                info!("📊 Execution result details: {:?}", value);
                
                if let Err(e) = Self::publish_transform_executed(message_bus, transform_id, "success") {
                    error!("❌ Event publishing failed after successful execution: {}", e);
                }
            }
            Err(e) => {
                error!("❌ Transform {} execution failed", transform_id);
                error!("❌ Failure details: {:?}", e);
                
                if let Err(publish_err) = Self::publish_transform_executed(message_bus, transform_id, "failed") {
                    error!("❌ Event publishing failed after execution failure: {}", publish_err);
                }
            }
        }
    }

    // ========== FIELD RESOLUTION UTILITIES ==========
    
    /// Extract simplified value from range field atom content
    /// Converts {"range_key":"2","value":"2"} to "2"
    /// Converts {"range_key":"2","value":{"value":"b"}} to "b"
    fn extract_simplified_value(content: &JsonValue) -> Result<JsonValue, SchemaError> {
        info!("🎯 Extracting simplified value from: {}", content);
        
        // Try to extract the "value" field
        if let Some(value_field) = content.get("value") {
            // If the value field is itself an object with a nested "value", extract that
            if let Some(nested_value) = value_field.get("value") {
                info!("✅ Extracted nested value: {}", nested_value);
                return Ok(nested_value.clone());
            } else {
                info!("✅ Extracted direct value: {}", value_field);
                return Ok(value_field.clone());
            }
        }
        
        // If no "value" field found, return the content as-is
        warn!("⚠️ No 'value' field found, returning content as-is");
        Ok(content.clone())
    }
    
    /// Unified field value resolution from schema using database operations
    pub fn resolve_field_value(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema: &Schema,
        field_name: &str,
        range_key: Option<String>,
    ) -> Result<JsonValue, SchemaError> {
        info!("🔍 FieldValueResolver: Looking up field '{}' in schema '{}'", field_name, schema.name);
        
        let field = schema.fields.get(field_name)
            .ok_or_else(|| {
                error!("❌ Field '{}' not found in schema '{}'", field_name, schema.name);
                SchemaError::InvalidField(format!("Field '{}' not found in schema '{}'", field_name, schema.name))
            })?;
        
        info!("✅ Field '{}' found in schema '{}'", field_name, schema.name);
        
        // Check if this is a range field first
        match field {
            FieldVariant::Range(_) => {
                info!("🔄 Detected range field, using AtomRefRange resolution");
                let range_aref_uuid = format!("{}_{}_range", schema.name, field_name);
                info!("🔍 Looking for AtomRefRange: {}", range_aref_uuid);
                
                match db_ops.get_item::<crate::atom::AtomRefRange>(&format!("ref:{}", range_aref_uuid)) {
                    Ok(Some(range_aref)) => {
                        info!("✅ Found AtomRefRange with {} entries", range_aref.atom_uuids.len());
                        
                        // BUG FIX 1: Filter by specific range key if provided
                        let entries_to_process: Vec<_> = if let Some(ref target_key) = range_key {
                            info!("🎯 Filtering for specific range key: '{}'", target_key);
                            range_aref.atom_uuids.iter()
                                .filter(|(key, _)| *key == target_key)
                                .collect()
                        } else {
                            info!("📋 Processing all range keys");
                            range_aref.atom_uuids.iter().collect()
                        };
                        
                        let mut combined_data = serde_json::Map::new();
                        
                        for (key, atom_uuid) in entries_to_process {
                            info!("🔗 Processing range key '{}' -> atom: {}", key, atom_uuid);
                            
                            match Self::load_atom(db_ops, atom_uuid) {
                                Ok(atom) => {
                                    let content = atom.content();
                                    info!("📦 Range entry '{}' content: {}", key, content);
                                    
                                    // BUG FIX 2: Extract simplified value instead of full structure
                                    let simplified_value = Self::extract_simplified_value(content)?;
                                    info!("🎯 Simplified value for key '{}': {}", key, simplified_value);
                                    
                                    combined_data.insert(key.clone(), simplified_value);
                                }
                                Err(e) => {
                                    error!("❌ Failed to load atom {} for range key '{}': {}", atom_uuid, key, e);
                                }
                            }
                        }
                        
                        let result = JsonValue::Object(combined_data);
                        info!("✅ Range field resolution complete - combined result: {}", result);
                        return Ok(result);
                    }
                    Ok(None) => {
                        error!("❌ AtomRefRange '{}' not found", range_aref_uuid);
                        return Err(SchemaError::InvalidField(format!("AtomRefRange '{}' not found", range_aref_uuid)));
                    }
                    Err(e) => {
                        error!("❌ Error loading AtomRefRange '{}': {}", range_aref_uuid, e);
                        return Err(SchemaError::InvalidField(format!("Error loading AtomRefRange '{}': {}", range_aref_uuid, e)));
                    }
                }
            }
            FieldVariant::Single(_) => {
                info!("🔄 Detected single field, using AtomRef resolution");
            }
        }
        
        // CRITICAL BUG DIAGNOSIS: This reads STATIC schema reference, not dynamic AtomRef!
        let ref_atom_uuid = Self::extract_ref_atom_uuid(field, field_name)?;
        error!("🚨 CRITICAL BUG: Reading STATIC schema ref_atom_uuid: {}", ref_atom_uuid);
        error!("🚨 This should be reading from DYNAMIC AtomRef system instead!");
        
        // DIAGNOSTIC: Check what the dynamic AtomRef system has
        let dynamic_aref_uuid = format!("{}_{}_single", schema.name, field_name);
        error!("🔍 DIAGNOSTIC: Checking dynamic AtomRef UUID: {}", dynamic_aref_uuid);
        
        match db_ops.get_item::<crate::atom::AtomRef>(&format!("ref:{}", dynamic_aref_uuid)) {
            Ok(Some(dynamic_aref)) => {
                let dynamic_atom_uuid = dynamic_aref.get_atom_uuid();
                error!("🔍 DIAGNOSTIC: Dynamic AtomRef points to atom: {}", dynamic_atom_uuid);
                error!("🚨 MISMATCH DETECTED: Static schema: {} vs Dynamic AtomRef: {}", ref_atom_uuid, dynamic_atom_uuid);
                
                // Use the CORRECT dynamic AtomRef instead of broken static schema reference
                info!("🔧 APPLYING FIX: Using dynamic AtomRef instead of static schema reference");
                let atom = Self::load_atom(db_ops, dynamic_atom_uuid)?;
                let content = atom.content().clone();
                info!("✅ Fixed query using dynamic AtomRef - content: {}", content);
                return Ok(content);
            }
            Ok(None) => {
                error!("🔍 DIAGNOSTIC: Dynamic AtomRef not found, falling back to static schema reference");
            }
            Err(e) => {
                error!("🔍 DIAGNOSTIC: Error checking dynamic AtomRef: {}", e);
            }
        }
        
        info!("� Field ref_atom_uuid: {}", ref_atom_uuid);
        
        let atom_ref = Self::load_atom_ref(db_ops, &ref_atom_uuid)?;
        let atom_uuid = atom_ref.get_atom_uuid();
        info!("🔗 AtomRef points to atom: {}", atom_uuid);
        
        let atom = Self::load_atom(db_ops, atom_uuid)?;
        
        info!("✅ Atom loaded successfully");
        let content = atom.content().clone();
        info!("📦 Atom content: {}", content);
        
        Ok(content)
    }
    
    /// Extract ref_atom_uuid from field variant with consistent error handling
    fn extract_ref_atom_uuid(field: &FieldVariant, field_name: &str) -> Result<String, SchemaError> {
        let ref_atom_uuid = field.ref_atom_uuid()
            .ok_or_else(|| {
                error!("❌ Field '{}' has no ref_atom_uuid", field_name);
                SchemaError::InvalidField(format!("Field '{}' has no ref_atom_uuid", field_name))
            })?
            .clone();
        Ok(ref_atom_uuid)
    }
    
    /// Load AtomRef from database with consistent error handling
    fn load_atom_ref(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        ref_atom_uuid: &str,
    ) -> Result<crate::atom::AtomRef, SchemaError> {
        info!("🔍 Loading AtomRef from database...");
        
        match db_ops.get_item::<crate::atom::AtomRef>(&format!("ref:{}", ref_atom_uuid)) {
            Ok(Some(atom_ref)) => Ok(atom_ref),
            Ok(None) => {
                error!("❌ AtomRef '{}' not found", ref_atom_uuid);
                Err(SchemaError::InvalidField(format!("AtomRef '{}' not found", ref_atom_uuid)))
            }
            Err(e) => {
                error!("❌ Failed to load AtomRef {}: {}", ref_atom_uuid, e);
                Err(SchemaError::InvalidField(format!("Failed to load AtomRef: {}", e)))
            }
        }
    }
    
    /// Load Atom from database with consistent error handling
    fn load_atom(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        atom_uuid: &str,
    ) -> Result<crate::atom::Atom, SchemaError> {
        info!("🔍 Loading Atom from database...");
        db_ops.get_item(&format!("atom:{}", atom_uuid))?
            .ok_or_else(|| {
                error!("❌ Atom '{}' not found", atom_uuid);
                SchemaError::InvalidField(format!("Atom '{}' not found", atom_uuid))
            })
    }

    // ========== LOGGING UTILITIES ==========
    
    /// Standard logging for transform registration
    pub fn log_transform_registration(transform_id: &str, inputs: &[String], output: &str) {
        info!("🔧 Registering transform '{}' with inputs: {:?}, output: {}", transform_id, inputs, output);
    }

    /// Standard logging for field mapping creation
    pub fn log_field_mapping_creation(field_key: &str, transform_id: &str) {
        info!("🔗 Created field mapping: '{}' -> transform '{}'", field_key, transform_id);
    }

    /// Standard logging for verification results
    pub fn log_verification_result(item_type: &str, id: &str, details: Option<&str>) {
        match details {
            Some(detail) => info!("✅ Verified {}: {} - {}", item_type, id, detail),
            None => info!("✅ Verified {}: {}", item_type, id),
        }
    }

    /// Standard logging for atom ref operations
    pub fn log_atom_ref_operation(ref_uuid: &str, atom_uuid: &str, operation: &str) {
        info!("🔗 AtomRef {} - ref:{} -> atom:{}", operation, ref_uuid, atom_uuid);
    }

    /// Standard logging for field mappings state
    pub fn log_field_mappings_state(mappings: &HashMap<String, HashSet<String>>, context: &str) {
        info!("🔍 DEBUG {}: Current field mappings ({} entries):", context, mappings.len());
        for (field_key, transforms) in mappings {
            info!("  📋 '{}' -> {:?}", field_key, transforms);
        }
        if mappings.is_empty() {
            info!("⚠️ No field mappings found in {}", context);
        }
    }

    /// Log collection state with consistent formatting
    pub fn log_collection_state<T: std::fmt::Debug>(
        collection_name: &str,
        collection: &T,
        operation: &str,
    ) {
        info!("🔍 DEBUG {}: {} collection state: {:?}", operation, collection_name, collection);
    }
}

// Type aliases for backward compatibility and reduced import burden
pub type LoggingHelper = TransformUtils;
pub type FieldValueResolver = TransformUtils;
pub type EventPublisher = TransformUtils;
pub type ConversionHelper = TransformUtils;
pub type SerializationHelper = TransformUtils;
pub type LockHelper = TransformUtils;
pub type DefaultValueHelper = TransformUtils;
pub type ValidationHelper = TransformUtils;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_to_value() {
        let json_val = json!({"key": "value"});
        let result = TransformUtils::json_to_value(json_val.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json_val);
    }

    #[test]
    fn test_validate_and_convert_string() {
        let json_val = JsonValue::String("test".to_string());
        let result = TransformUtils::validate_and_convert(json_val.clone(), "string", "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json_val);
    }

    #[test]
    fn test_string_to_json_value_with_inference() {
        assert_eq!(TransformUtils::string_to_json_value("true", true), JsonValue::Bool(true));
        assert_eq!(TransformUtils::string_to_json_value("42", true), JsonValue::Number(serde_json::Number::from(42)));
        assert_eq!(TransformUtils::string_to_json_value("hello", true), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_default_value_helper() {
        assert_eq!(TransformUtils::get_default_value_for_field("input1"), JsonValue::Number(serde_json::Number::from(42)));
        assert_eq!(TransformUtils::get_default_value_for_field("active"), JsonValue::Bool(true));
    }

    #[test]
    fn test_infer_type_from_field_name() {
        assert_eq!(TransformUtils::infer_type_from_field_name("user_id"), "string");
        assert_eq!(TransformUtils::infer_type_from_field_name("age_count"), "integer");
        assert_eq!(TransformUtils::infer_type_from_field_name("weight_value"), "float");
        assert_eq!(TransformUtils::infer_type_from_field_name("is_active"), "boolean");
    }

    #[test]
    fn test_validate_field_name() {
        assert!(TransformUtils::validate_field_name("Schema.field", "test").is_ok());
        assert!(TransformUtils::validate_field_name("invalid_field_name", "test").is_err());
    }
}