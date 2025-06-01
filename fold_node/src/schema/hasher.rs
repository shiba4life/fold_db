use crate::schema::SchemaError;
use serde_json::{Value, Map};
use sha2::{Sha256, Digest};
use log::info;

/// Schema hasher for adding integrity verification to schemas
pub struct SchemaHasher;

impl SchemaHasher {
    /// Calculate hash for a schema JSON, excluding hash, payment_config, and permission_policy fields
    pub fn calculate_hash(schema_json: &Value) -> Result<String, SchemaError> {
        // Clone the schema and recursively remove fields that should not affect the hash
        let mut schema_for_hash = schema_json.clone();
        Self::remove_excluded_fields(&mut schema_for_hash);
        
        // Serialize to canonical JSON (sorted keys)
        let canonical_json = Self::to_canonical_json(&schema_for_hash)?;
        
        // Calculate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(canonical_json.as_bytes());
        let hash_bytes = hasher.finalize();
        
        // Convert to hex string
        Ok(format!("{:x}", hash_bytes))
    }
    
    /// Recursively remove excluded fields from JSON value
    fn remove_excluded_fields(value: &mut Value) {
        match value {
            Value::Object(obj) => {
                // Remove excluded fields at this level
                obj.remove("hash");
                obj.remove("payment_config");
                obj.remove("permission_policy");
                obj.remove("name"); // Exclude name to detect content duplicates regardless of schema name
                
                // Recursively process all nested objects
                for (_, nested_value) in obj.iter_mut() {
                    Self::remove_excluded_fields(nested_value);
                }
            }
            Value::Array(arr) => {
                // Recursively process array elements
                for item in arr.iter_mut() {
                    Self::remove_excluded_fields(item);
                }
            }
            _ => {
                // Primitive values don't need processing
            }
        }
    }
    
    /// Add or update hash field in a schema JSON
    pub fn add_hash_to_schema(schema_json: &mut Value) -> Result<String, SchemaError> {
        let hash = Self::calculate_hash(schema_json)?;
        
        if let Value::Object(ref mut obj) = schema_json {
            obj.insert("hash".to_string(), Value::String(hash.clone()));
        } else {
            return Err(SchemaError::InvalidData("Schema must be a JSON object".to_string()));
        }
        
        Ok(hash)
    }
    
    /// Verify that a schema's hash matches its content
    pub fn verify_schema_hash(schema_json: &Value) -> Result<bool, SchemaError> {
        if let Value::Object(obj) = schema_json {
            if let Some(Value::String(stored_hash)) = obj.get("hash") {
                let calculated_hash = Self::calculate_hash(schema_json)?;
                Ok(stored_hash == &calculated_hash)
            } else {
                // No hash field present
                Ok(false)
            }
        } else {
            Err(SchemaError::InvalidData("Schema must be a JSON object".to_string()))
        }
    }
    
    /// Process a schema file: read, add/update hash, and write back
    pub fn hash_schema_file(file_path: &std::path::Path) -> Result<String, SchemaError> {
        info!("Processing schema file: {}", file_path.display());
        
        // Read the file
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read file: {}", e)))?;
        
        // Parse JSON
        let mut schema_json: Value = serde_json::from_str(&content)
            .map_err(|e| SchemaError::InvalidData(format!("Invalid JSON: {}", e)))?;
        
        // Add/update hash
        let hash = Self::add_hash_to_schema(&mut schema_json)?;
        
        // Write back to file with pretty formatting
        let formatted_json = serde_json::to_string_pretty(&schema_json)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to format JSON: {}", e)))?;
        
        std::fs::write(file_path, formatted_json)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to write file: {}", e)))?;
        
        info!("Updated schema file {} with hash: {}", file_path.display(), hash);
        Ok(hash)
    }
    
    /// Process all schema files in the specified directory
    pub fn hash_schemas_directory<P: AsRef<std::path::Path>>(directory_path: P) -> Result<Vec<(String, String)>, SchemaError> {
        let available_schemas_dir = directory_path.as_ref();
        let mut results = Vec::new();
        
        info!("Processing schemas in directory: {}", available_schemas_dir.display());
        
        if !available_schemas_dir.exists() {
            return Err(SchemaError::InvalidData(format!(
                "Directory does not exist: {}", 
                available_schemas_dir.display()
            )));
        }
        
        let entries = std::fs::read_dir(available_schemas_dir)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| SchemaError::InvalidData(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            
            // Only process .json files, skip README.md and other files
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match Self::hash_schema_file(&path) {
                    Ok(hash) => {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        info!("✅ Processed: {} -> {}", filename, hash);
                        results.push((filename.clone(), hash.clone()));
                    }
                    Err(e) => {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        info!("❌ Failed to process {}: {}", filename, e);
                        return Err(e);
                    }
                }
            }
        }
        
        info!("Successfully processed {} schema files", results.len());
        Ok(results)
    }
    
    /// Process all schema files in the available_schemas directory (convenience method)
    pub fn hash_available_schemas_directory() -> Result<Vec<(String, String)>, SchemaError> {
        SchemaHasher::hash_schemas_directory("available_schemas")
    }
    
    /// Verify all schemas in the specified directory
    pub fn verify_schemas_directory<P: AsRef<std::path::Path>>(directory_path: P) -> Result<Vec<(String, bool)>, SchemaError> {
        let available_schemas_dir = directory_path.as_ref();
        let mut results = Vec::new();
        
        info!("Verifying schemas in directory: {}", available_schemas_dir.display());
        
        let entries = std::fs::read_dir(available_schemas_dir)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| SchemaError::InvalidData(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to read file: {}", e)))?;
                
                let schema_json: Value = serde_json::from_str(&content)
                    .map_err(|e| SchemaError::InvalidData(format!("Invalid JSON: {}", e)))?;
                
                let is_valid = Self::verify_schema_hash(&schema_json)?;
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                results.push((filename.clone(), is_valid));
                
                if is_valid {
                    info!("✅ Valid hash: {}", filename);
                } else {
                    info!("❌ Invalid/missing hash: {}", filename);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Verify all schemas in the available_schemas directory (convenience method)
    pub fn verify_available_schemas_directory() -> Result<Vec<(String, bool)>, SchemaError> {
        SchemaHasher::verify_schemas_directory("available_schemas")
    }
    
    /// Convert JSON to canonical form (sorted keys) for consistent hashing
    fn to_canonical_json(value: &Value) -> Result<String, SchemaError> {
        match value {
            Value::Object(obj) => {
                let mut sorted_obj = Map::new();
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                
                for key in keys {
                    if let Some(val) = obj.get(key) {
                        let canonical_val = Self::to_canonical_json(val)?;
                        let parsed_val: Value = serde_json::from_str(&canonical_val)
                            .map_err(|e| SchemaError::InvalidData(format!("Failed to parse canonical JSON: {}", e)))?;
                        sorted_obj.insert(key.clone(), parsed_val);
                    }
                }
                
                serde_json::to_string(&Value::Object(sorted_obj))
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize canonical JSON: {}", e)))
            }
            Value::Array(arr) => {
                let mut canonical_items = Vec::new();
                for item in arr {
                    let canonical_item = Self::to_canonical_json(item)?;
                    let parsed_item: Value = serde_json::from_str(&canonical_item)
                        .map_err(|e| SchemaError::InvalidData(format!("Failed to parse canonical JSON: {}", e)))?;
                    canonical_items.push(parsed_item);
                }
                
                serde_json::to_string(&Value::Array(canonical_items))
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize canonical JSON: {}", e)))
            }
            _ => {
                serde_json::to_string(value)
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize JSON: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_calculate_hash() {
        let schema = json!({
            "name": "TestSchema",
            "fields": {
                "field1": {
                    "field_type": "Single"
                }
            }
        });
        
        let hash = SchemaHasher::calculate_hash(&schema).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 hex string length
    }
    
    #[test]
    fn test_hash_consistency() {
        let schema1 = json!({
            "name": "TestSchema",
            "fields": {
                "field1": {"field_type": "Single"},
                "field2": {"field_type": "Collection"}
            }
        });
        
        let schema2 = json!({
            "fields": {
                "field2": {"field_type": "Collection"},
                "field1": {"field_type": "Single"}
            },
            "name": "TestSchema"
        });
        
        let hash1 = SchemaHasher::calculate_hash(&schema1).unwrap();
        let hash2 = SchemaHasher::calculate_hash(&schema2).unwrap();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_hash_excludes_existing_hash() {
        let mut schema = json!({
            "name": "TestSchema",
            "fields": {
                "field1": {"field_type": "Single"}
            }
        });
        
        let hash1 = SchemaHasher::calculate_hash(&schema).unwrap();
        
        // Add hash to schema
        SchemaHasher::add_hash_to_schema(&mut schema).unwrap();
        
        // Hash should be the same even with hash field present
        let hash2 = SchemaHasher::calculate_hash(&schema).unwrap();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_add_and_verify_hash() {
        let mut schema = json!({
            "name": "TestSchema",
            "fields": {
                "field1": {"field_type": "Single"}
            }
        });
        
        let hash = SchemaHasher::add_hash_to_schema(&mut schema).unwrap();
        assert!(!hash.is_empty());
        
        let is_valid = SchemaHasher::verify_schema_hash(&schema).unwrap();
        assert!(is_valid);
    }
}