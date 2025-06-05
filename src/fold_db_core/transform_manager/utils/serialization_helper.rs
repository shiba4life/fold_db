//! Serialization utilities for consistent data serialization across the transform system.
//!
//! This module provides unified serialization patterns with consistent error handling
//! to eliminate duplicate serialization code, particularly for mapping persistence.

use crate::schema::types::SchemaError;
use crate::fold_db_core::transform_manager::utils::lock_helper::LockHelper;
use log::{info, error};
use std::sync::RwLock;

/// Utility for serializing mappings with consistent error handling and logging
pub struct SerializationHelper;

impl SerializationHelper {
    /// Serialize a mapping to bytes with consistent error handling
    pub fn serialize_mapping<T>(
        mapping: &RwLock<T>,
        mapping_name: &str,
    ) -> Result<Vec<u8>, SchemaError>
    where
        T: serde::Serialize,
    {
        info!("🔄 Serializing mapping: {}", mapping_name);
        
        let map = LockHelper::read_lock(mapping, mapping_name)?;
        let json = serde_json::to_vec(&*map).map_err(|e| {
            let error_msg = format!("Failed to serialize {}: {}", mapping_name, e);
            error!("❌ {}", error_msg);
            SchemaError::InvalidData(error_msg)
        })?;
        
        info!("✅ Successfully serialized mapping: {} ({} bytes)", mapping_name, json.len());
        Ok(json)
    }

    /// Serialize a mapping with additional logging for the contents
    pub fn serialize_mapping_with_debug<T>(
        mapping: &RwLock<T>,
        mapping_name: &str,
    ) -> Result<Vec<u8>, SchemaError>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        info!("🔄 Serializing mapping with debug: {}", mapping_name);
        
        let map = LockHelper::read_lock(mapping, mapping_name)?;
        
        // Debug logging for field_to_transforms specifically
        if mapping_name == "field_to_transforms" {
            info!("🔍 DEBUG: Storing {} mapping with data: {:?}", mapping_name, *map);
        }
        
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
                // Return default on deserialization error
                info!("🔄 Using default value for {} due to deserialization error", mapping_name);
                Ok(T::default())
            }
        }
    }

    /// Load mapping from database with fallback to default
    pub fn load_mapping_or_default<T>(
        db_ops: &std::sync::Arc<crate::db_operations::DbOperations>,
        key: &str,
        mapping_name: &str,
    ) -> Result<T, SchemaError>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        info!("📥 Loading mapping: {} from key: {}", mapping_name, key);
        
        match db_ops.get_transform_mapping(key)? {
            Some(data) => {
                info!("✅ Found stored data for mapping: {}", mapping_name);
                Self::deserialize_mapping(&data, mapping_name)
            }
            None => {
                info!("ℹ️ No stored data found for mapping: {}, using default", mapping_name);
                Ok(T::default())
            }
        }
    }

    /// Store serialized mapping to database
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

    /// Store mapping with debug logging (for field_to_transforms)
    pub fn store_mapping_with_debug<T>(
        db_ops: &std::sync::Arc<crate::db_operations::DbOperations>,
        mapping: &RwLock<T>,
        key: &str,
        mapping_name: &str,
    ) -> Result<(), SchemaError>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        info!("💾 Storing mapping with debug: {} to key: {}", mapping_name, key);
        
        // Special debug logging for field_to_transforms
        if mapping_name == "field_to_transforms" {
            let _map = mapping.read()
                .map_err(|_| SchemaError::InvalidData(format!("Failed to acquire read lock for {}", mapping_name)))?;
            info!("🔍 DEBUG: Storing field_to_transforms mapping to database");
        }
        
        let json = Self::serialize_mapping_with_debug(mapping, mapping_name)?;
        db_ops.store_transform_mapping(key, &json)?;
        
        info!("✅ Successfully stored mapping: {} to database", mapping_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    use std::collections::HashMap;

    #[test]
    fn test_serialize_mapping() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        
        let mapping = RwLock::new(map);
        let result = SerializationHelper::serialize_mapping(&mapping, "test_mapping");
        
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_deserialize_mapping() {
        let test_data = r#"{"key1":"value1","key2":"value2"}"#.as_bytes();
        let result: Result<HashMap<String, String>, _> = 
            SerializationHelper::deserialize_mapping(test_data, "test_mapping");
        
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_deserialize_mapping_with_error() {
        let invalid_data = b"invalid json";
        let result: Result<HashMap<String, String>, _> = 
            SerializationHelper::deserialize_mapping(invalid_data, "test_mapping");
        
        // Should return default value on error
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.is_empty()); // Default HashMap is empty
    }
}