use super::error_utils::ErrorUtils;
use crate::schema::SchemaError;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// Enhanced database operations struct that provides unified access to all database operations.
/// This replaces the previous mixed approach of direct sled access and DbOperations.
#[derive(Clone)]
pub struct DbOperations {
    /// The underlying sled database instance
    db: sled::Db,
    /// Cached trees for performance
    pub(crate) metadata_tree: sled::Tree,
    pub(crate) permissions_tree: sled::Tree,
    pub(crate) transforms_tree: sled::Tree,
    pub(crate) orchestrator_tree: sled::Tree,
    pub(crate) schema_states_tree: sled::Tree,
    pub(crate) schemas_tree: sled::Tree,
}

impl DbOperations {
    /// Creates a new enhanced DbOperations instance with all required trees
    pub fn new(db: sled::Db) -> Result<Self, sled::Error> {
        let metadata_tree = db.open_tree("metadata")?;
        let permissions_tree = db.open_tree("node_id_schema_permissions")?;
        let transforms_tree = db.open_tree("transforms")?;
        let orchestrator_tree = db.open_tree("orchestrator_state")?;
        let schema_states_tree = db.open_tree("schema_states")?;
        let schemas_tree = db.open_tree("schemas")?;

        Ok(Self {
            db,
            metadata_tree,
            permissions_tree,
            transforms_tree,
            orchestrator_tree,
            schema_states_tree,
            schemas_tree,
        })
    }

    /// Gets a reference to the underlying database
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Generic function to store a serializable item in the database
    pub fn store_item<T: Serialize>(&self, key: &str, item: &T) -> Result<(), SchemaError> {
        let bytes =
            serde_json::to_vec(item).map_err(ErrorUtils::from_serialization_error("item"))?;

        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(ErrorUtils::from_sled_error("insert"))?;

        // Ensure the data is durably written to disk
        self.db
            .flush()
            .map_err(ErrorUtils::from_sled_error("flush"))?;

        Ok(())
    }

    /// Generic function to retrieve a deserializable item from the database
    pub fn get_item<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SchemaError> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let item = serde_json::from_slice(&bytes)
                    .map_err(ErrorUtils::from_deserialization_error("item"))?;
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ErrorUtils::database_error("retrieve", e)),
        }
    }

    /// Lists all items with a given prefix
    pub fn list_items_with_prefix(&self, prefix: &str) -> Result<Vec<String>, SchemaError> {
        let mut items = Vec::new();
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, _) = result
                .map_err(|e| SchemaError::InvalidData(format!("Failed to scan prefix: {}", e)))?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            items.push(key_str);
        }
        Ok(items)
    }

    /// Gets database statistics
    pub fn get_stats(&self) -> Result<HashMap<String, u64>, SchemaError> {
        let mut stats = HashMap::new();

        // Count items in each tree
        stats.insert("atoms".to_string(), self.count_items_with_prefix("atom:")?);
        stats.insert("refs".to_string(), self.count_items_with_prefix("ref:")?);
        stats.insert("metadata".to_string(), self.metadata_tree.len() as u64);
        stats.insert(
            "permissions".to_string(),
            self.permissions_tree.len() as u64,
        );
        stats.insert("transforms".to_string(), self.transforms_tree.len() as u64);
        stats.insert(
            "orchestrator".to_string(),
            self.orchestrator_tree.len() as u64,
        );
        stats.insert(
            "schema_states".to_string(),
            self.schema_states_tree.len() as u64,
        );
        stats.insert("schemas".to_string(), self.schemas_tree.len() as u64);

        Ok(stats)
    }

    /// Counts items with a given prefix
    fn count_items_with_prefix(&self, prefix: &str) -> Result<u64, SchemaError> {
        let mut count = 0;
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            result
                .map_err(|e| SchemaError::InvalidData(format!("Failed to scan prefix: {}", e)))?;
            count += 1;
        }
        Ok(count)
    }

    // ========== GENERIC TREE OPERATIONS ==========

    /// Generic function to store any serializable item in a specific tree
    pub fn store_in_tree<T: Serialize>(
        &self,
        tree: &sled::Tree,
        key: &str,
        item: &T,
    ) -> Result<(), SchemaError> {
        use crate::transform_execution::error::error_conversion::ErrorConversion;
        
        let bytes = serde_json::to_vec(item)
            .map_serialization_error()?;

        tree.insert(key.as_bytes(), bytes)
            .map_database_error("insert")?;

        tree.flush()
            .map_flush_error()?;

        Ok(())
    }

    /// Generic function to retrieve any deserializable item from a specific tree
    pub fn get_from_tree<T: DeserializeOwned>(
        &self,
        tree: &sled::Tree,
        key: &str,
    ) -> Result<Option<T>, SchemaError> {
        match tree.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let item = serde_json::from_slice(&bytes).map_err(|e| {
                    SchemaError::InvalidData(format!("Deserialization failed: {}", e))
                })?;
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SchemaError::InvalidData(format!("Retrieval failed: {}", e))),
        }
    }

    /// List all keys in a tree
    pub fn list_keys_in_tree(&self, tree: &sled::Tree) -> Result<Vec<String>, SchemaError> {
        let mut keys = Vec::new();
        for result in tree.iter() {
            let (key, _) = result
                .map_err(|e| SchemaError::InvalidData(format!("Tree iteration failed: {}", e)))?;
            keys.push(String::from_utf8_lossy(&key).to_string());
        }
        Ok(keys)
    }

    /// List all key-value pairs in a tree
    pub fn list_items_in_tree<T: DeserializeOwned>(
        &self,
        tree: &sled::Tree,
    ) -> Result<Vec<(String, T)>, SchemaError> {
        let mut items = Vec::new();
        for result in tree.iter() {
            let (key, value) = result
                .map_err(|e| SchemaError::InvalidData(format!("Tree iteration failed: {}", e)))?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            let item = serde_json::from_slice(&value).map_err(|e| {
                SchemaError::InvalidData(format!(
                    "Deserialization failed for key '{}': {}",
                    key_str, e
                ))
            })?;
            items.push((key_str, item));
        }
        Ok(items)
    }

    /// Delete an item from a specific tree
    pub fn delete_from_tree(&self, tree: &sled::Tree, key: &str) -> Result<bool, SchemaError> {
        let existed = tree
            .remove(key.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Delete failed: {}", e)))?
            .is_some();

        tree.flush()
            .map_err(|e| SchemaError::InvalidData(format!("Flush failed: {}", e)))?;

        Ok(existed)
    }

    /// Check if a key exists in a specific tree
    pub fn exists_in_tree(&self, tree: &sled::Tree, key: &str) -> Result<bool, SchemaError> {
        tree.contains_key(key.as_bytes())
            .map_err(|e| SchemaError::InvalidData(format!("Existence check failed: {}", e)))
    }

    // ========== PUBLIC TREE ACCESSORS FOR TESTING ==========

    /// Get a reference to the metadata tree
    pub fn metadata_tree(&self) -> &sled::Tree {
        &self.metadata_tree
    }

    /// Get a reference to the schemas tree
    #[cfg(test)]
    pub fn schemas_tree(&self) -> &sled::Tree {
        &self.schemas_tree
    }

    /// Get a reference to the transforms tree
    #[cfg(test)]
    pub fn transforms_tree(&self) -> &sled::Tree {
        &self.transforms_tree
    }

    /// Get a reference to the permissions tree
    #[cfg(test)]
    pub fn permissions_tree(&self) -> &sled::Tree {
        &self.permissions_tree
    }

    /// Get a reference to the orchestrator tree
    #[cfg(test)]
    pub fn orchestrator_tree(&self) -> &sled::Tree {
        &self.orchestrator_tree
    }

    /// Get a reference to the schema states tree
    #[cfg(test)]
    pub fn schema_states_tree(&self) -> &sled::Tree {
        &self.schema_states_tree
    }

    /// Check if crypto metadata exists in the database
    pub fn has_crypto_metadata(&self) -> Result<bool, SchemaError> {
        self.exists_in_tree(&self.metadata_tree, "crypto_metadata")
    }

    /// Get crypto metadata from the database
    pub fn get_crypto_metadata(&self) -> Result<Option<CryptoMetadata>, SchemaError> {
        self.get_from_tree(&self.metadata_tree, "crypto_metadata")
    }

    /// Store crypto metadata in the database
    pub fn store_crypto_metadata(&self, metadata: &CryptoMetadata) -> Result<(), SchemaError> {
        self.store_in_tree(&self.metadata_tree, "crypto_metadata", metadata)
    }

    // ========== KEY ROTATION OPERATIONS ==========

    /// Store a key association for rotation tracking
    pub fn store_key_association(&self, association: &KeyAssociation) -> Result<(), SchemaError> {
        let key_rotation_tree = self.db.open_tree("key_associations")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_associations tree: {}", e)))?;
        self.store_in_tree(&key_rotation_tree, &association.association_id, association)
    }

    /// Get a key association by ID
    pub fn get_key_association(&self, association_id: &str) -> Result<Option<KeyAssociation>, SchemaError> {
        let key_rotation_tree = self.db.open_tree("key_associations")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_associations tree: {}", e)))?;
        self.get_from_tree(&key_rotation_tree, association_id)
    }

    /// Get all key associations for a public key
    pub fn get_key_associations(&self, public_key_hex: &str) -> Result<Vec<KeyAssociation>, SchemaError> {
        let key_rotation_tree = self.db.open_tree("key_associations")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_associations tree: {}", e)))?;
        
        let mut associations = Vec::new();
        for result in key_rotation_tree.iter() {
            let (_, value) = result.map_err(|e| SchemaError::InvalidData(format!("Iterator failed: {}", e)))?;
            let association: KeyAssociation = serde_json::from_slice(&value)
                .map_err(|e| SchemaError::InvalidData(format!("Deserialization failed: {}", e)))?;
            
            if association.public_key_hex == public_key_hex {
                associations.push(association);
            }
        }
        Ok(associations)
    }

    /// Delete a key association
    pub fn delete_key_association(&self, association_id: &str) -> Result<bool, SchemaError> {
        let key_rotation_tree = self.db.open_tree("key_associations")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_associations tree: {}", e)))?;
        self.delete_from_tree(&key_rotation_tree, association_id)
    }

    /// Store a key rotation record
    pub fn store_rotation_record(&self, record: &KeyRotationRecord) -> Result<(), SchemaError> {
        let rotation_tree = self.db.open_tree("key_rotation_records")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_rotation_records tree: {}", e)))?;
        self.store_in_tree(&rotation_tree, &record.operation_id, record)
    }

    /// Get a rotation record by operation ID
    pub fn get_rotation_record(&self, operation_id: &str) -> Result<Option<KeyRotationRecord>, SchemaError> {
        let rotation_tree = self.db.open_tree("key_rotation_records")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_rotation_records tree: {}", e)))?;
        self.get_from_tree(&rotation_tree, operation_id)
    }

    /// Get rotation history for a public key
    pub fn get_rotation_history(&self, public_key_hex: &str) -> Result<Vec<KeyRotationRecord>, SchemaError> {
        let rotation_tree = self.db.open_tree("key_rotation_records")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_rotation_records tree: {}", e)))?;
        
        let mut records = Vec::new();
        for result in rotation_tree.iter() {
            let (_, value) = result.map_err(|e| SchemaError::InvalidData(format!("Iterator failed: {}", e)))?;
            let record: KeyRotationRecord = serde_json::from_slice(&value)
                .map_err(|e| SchemaError::InvalidData(format!("Deserialization failed: {}", e)))?;
            
            if record.old_public_key_hex == public_key_hex || record.new_public_key_hex == public_key_hex {
                records.push(record);
            }
        }
        Ok(records)
    }

    /// Get rotation statistics
    pub fn get_rotation_statistics(&self) -> Result<RotationStatistics, SchemaError> {
        let rotation_tree = self.db.open_tree("key_rotation_records")
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open key_rotation_records tree: {}", e)))?;
        
        let mut stats = RotationStatistics::default();
        for result in rotation_tree.iter() {
            let (_, value) = result.map_err(|e| SchemaError::InvalidData(format!("Iterator failed: {}", e)))?;
            let _record: KeyRotationRecord = serde_json::from_slice(&value)
                .map_err(|e| SchemaError::InvalidData(format!("Deserialization failed: {}", e)))?;
            stats.total_rotations += 1;
        }
        Ok(stats)
    }
}

/// Crypto metadata structure for database storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CryptoMetadata {
    /// Whether crypto is initialized
    pub initialized: bool,
    /// Crypto version
    pub version: String,
    /// Algorithm in use
    pub algorithm: String,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Default for CryptoMetadata {
    fn default() -> Self {
        Self {
            initialized: false,
            version: "1.0".to_string(),
            algorithm: "Ed25519".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Key association for rotation tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyAssociation {
    /// Association identifier
    pub association_id: String,
    /// Public key hex
    pub public_key_hex: String,
    /// Associated metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
}

/// Key rotation record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyRotationRecord {
    /// Operation identifier
    pub operation_id: String,
    /// Old public key hex
    pub old_public_key_hex: String,
    /// New public key hex
    pub new_public_key_hex: String,
    /// Rotation timestamp
    pub rotated_at: std::time::SystemTime,
    /// Rotation reason
    pub reason: String,
    /// Success status
    pub success: bool,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Rotation statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RotationStatistics {
    /// Total number of rotations
    pub total_rotations: u64,
    /// Number of successful rotations
    pub successful_rotations: u64,
    /// Number of failed rotations
    pub failed_rotations: u64,
}
