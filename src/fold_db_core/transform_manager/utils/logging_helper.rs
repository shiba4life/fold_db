//! Logging utilities for consistent debug logging patterns across the transform system.
//!
//! This module provides unified logging patterns with consistent formatting
//! to eliminate duplicate logging code throughout the transform manager.

use log::info;
use std::collections::{HashMap, HashSet};

/// Utility for consistent debug logging patterns across the transform system
pub struct LoggingHelper;

impl LoggingHelper {
    /// Log collection state with consistent formatting - consolidates repeated collection logging patterns
    pub fn log_collection_state<T: std::fmt::Debug>(
        collection_name: &str,
        collection: &T,
        operation: &str,
    ) {
        info!("🔍 DEBUG {}: {} collection state: {:?}", operation, collection_name, collection);
    }

    /// Log field mapping state with consistent formatting
    pub fn log_field_mappings_state(
        field_to_transforms: &HashMap<String, HashSet<String>>,
        operation: &str,
    ) {
        info!("🔍 DEBUG {}: field_to_transforms state with {} entries:", operation, field_to_transforms.len());
        for (field_key, transforms) in field_to_transforms {
            info!("🔗 DEBUG {}: Field '{}' -> transforms: {:?}", operation, field_key, transforms);
        }
        
        if field_to_transforms.is_empty() {
            info!("⚠️ DEBUG {}: No field mappings available!", operation);
        }
    }

    /// Log transform progress with consistent formatting - consolidates transform state logging
    pub fn log_transform_progress(
        transform_id: &str,
        operation: &str,
        details: Option<&str>,
    ) {
        match details {
            Some(detail_msg) => {
                info!("🔍 DEBUG {}: Transform '{}' - {}", operation, transform_id, detail_msg);
            }
            None => {
                info!("🔍 DEBUG {}: Processing transform '{}'", operation, transform_id);
            }
        }
    }

    /// Log transform registration with inputs
    pub fn log_transform_registration(
        transform_id: &str,
        inputs: &[String],
        output: &str,
    ) {
        info!("🔍 DEBUG: Creating field mappings for transform '{}' with inputs: {:?}", transform_id, inputs);
        info!("📋 Loaded new transform '{}' with inputs: {:?}, output: {}", transform_id, inputs, output);
    }

    /// Log field mapping creation with consistent formatting
    pub fn log_field_mapping_creation(
        field_key: &str,
        transform_id: &str,
    ) {
        info!("🔗 DEBUG: Mapped field '{}' -> transform '{}'", field_key, transform_id);
    }

    /// Log field mapping registration with consistent formatting  
    pub fn log_field_mapping_registration(
        field_key: &str,
        transform_id: &str,
    ) {
        info!("🔗 DEBUG: Registered field mapping '{}' -> transform '{}'", field_key, transform_id);
    }

    /// Log transform manager initialization state
    pub fn log_manager_initialization(
        field_to_transforms: &HashMap<String, HashSet<String>>,
    ) {
        info!("🔍 DEBUG TransformManager::new(): Loaded field_to_transforms with {} entries:", field_to_transforms.len());
        for (field_key, transforms) in field_to_transforms {
            info!("🔗 DEBUG TransformManager::new(): Field '{}' -> transforms: {:?}", field_key, transforms);
        }
        
        if field_to_transforms.is_empty() {
            info!("⚠️ DEBUG TransformManager::new(): No field mappings loaded from database!");
        }
    }

    /// Log database persistence operations
    pub fn log_persistence_operation(
        mapping_name: &str,
        operation: &str,
        success: bool,
    ) {
        if success {
            match operation {
                "load" => info!("🔍 DEBUG: Loaded {} mapping from database", mapping_name),
                "store" => info!("🔍 DEBUG: Storing {} mapping to database", mapping_name),
                _ => info!("🔍 DEBUG: {} operation on {} mapping completed", operation, mapping_name),
            }
        } else {
            info!("🔍 DEBUG: No {} mapping found in database - starting with empty map", mapping_name);
        }
    }

    /// Log atom reference operations with debug details
    pub fn log_atom_ref_operation(
        ref_uuid: &str,
        atom_uuid: &str,
        operation: &str,
    ) {
        info!("🔧 DEBUG {}: AtomRef UUID: {} (this is the reference ID)", operation, ref_uuid);
        info!("🔧 DEBUG {}: Target Atom UUID: {} (this is what the reference points to)", operation, atom_uuid);
        
        if operation == "verification" {
            info!("🔍 DEBUG: Reference chain: Schema.field → AtomRef {} → Data Atom {}", ref_uuid, atom_uuid);
        }
    }

    /// Log verification operations with debug details
    pub fn log_verification_result(
        item_type: &str,
        item_id: &str,
        content: Option<&str>,
    ) {
        match content {
            Some(content_str) => {
                info!("🔍 DEBUG: Verified {} {} exists with content: {}", item_type, item_id, content_str);
            }
            None => {
                info!("🔍 DEBUG: Verified {} {} exists", item_type, item_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_log_collection_state() {
        let mut test_map = HashMap::new();
        test_map.insert("key1".to_string(), "value1".to_string());
        
        // This should not panic - just logs
        LoggingHelper::log_collection_state("test_map", &test_map, "TEST");
    }

    #[test]
    fn test_log_field_mappings_state() {
        let mut field_mappings = HashMap::new();
        let mut transform_set = HashSet::new();
        transform_set.insert("transform1".to_string());
        field_mappings.insert("field1".to_string(), transform_set);
        
        // This should not panic - just logs
        LoggingHelper::log_field_mappings_state(&field_mappings, "TEST");
    }

    #[test]
    fn test_log_transform_progress() {
        // This should not panic - just logs
        LoggingHelper::log_transform_progress("test_transform", "TEST", Some("test details"));
        LoggingHelper::log_transform_progress("test_transform", "TEST", None);
    }
}