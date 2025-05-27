use crate::schema::SchemaError;
use crate::schema::types::transform::{Transform, TransformRegistration};
use super::core::DbOperations;

impl DbOperations {
    /// Stores a transform
    pub fn store_transform(&self, transform_id: &str, transform: &Transform) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(transform)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize transform: {}", e)))?;
        self.transforms_tree.insert(transform_id.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store transform: {}", e)))?;
        self.transforms_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush transforms: {}", e)))?;
        Ok(())
    }

    /// Gets a transform
    pub fn get_transform(&self, transform_id: &str) -> Result<Option<Transform>, SchemaError> {
        if let Some(bytes) = self.transforms_tree.get(transform_id.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get transform: {}", e)))? {
            let transform = serde_json::from_slice(&bytes)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to deserialize transform: {}", e)))?;
            Ok(Some(transform))
        } else {
            Ok(None)
        }
    }

    /// Lists all transform IDs
    pub fn list_transforms(&self) -> Result<Vec<String>, SchemaError> {
        let mut transforms = Vec::new();
        for result in self.transforms_tree.iter() {
            let (key, _) = result
                .map_err(|e| SchemaError::InvalidData(format!("Failed to iterate transforms: {}", e)))?;
            let transform_id = String::from_utf8_lossy(&key).to_string();
            transforms.push(transform_id);
        }
        Ok(transforms)
    }

    /// Deletes a transform
    pub fn delete_transform(&self, transform_id: &str) -> Result<(), SchemaError> {
        self.transforms_tree.remove(transform_id.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to delete transform: {}", e)))?;
        self.transforms_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush transforms: {}", e)))?;
        Ok(())
    }

    /// Stores a transform registration
    pub fn store_transform_registration(&self, registration: &TransformRegistration) -> Result<(), SchemaError> {
        let key = format!("registration:{}", registration.transform_id);
        self.store_item(&key, registration)
    }

    /// Gets a transform registration
    pub fn get_transform_registration(&self, transform_id: &str) -> Result<Option<TransformRegistration>, SchemaError> {
        let key = format!("registration:{}", transform_id);
        self.get_item(&key)
    }

    /// Stores a transform mapping (for internal mappings like aref_to_transforms)
    pub fn store_transform_mapping(&self, key: &str, data: &[u8]) -> Result<(), SchemaError> {
        self.transforms_tree.insert(key.as_bytes(), data)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store transform mapping: {}", e)))?;
        self.transforms_tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush transform mappings: {}", e)))?;
        Ok(())
    }

    /// Gets a transform mapping
    pub fn get_transform_mapping(&self, key: &str) -> Result<Option<Vec<u8>>, SchemaError> {
        if let Some(bytes) = self.transforms_tree.get(key.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get transform mapping: {}", e)))? {
            Ok(Some(bytes.to_vec()))
        } else {
            Ok(None)
        }
    }
}