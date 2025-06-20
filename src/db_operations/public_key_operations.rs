use crate::db_operations::DbOperations;
use crate::security::types::PublicKeyInfo;
use crate::schema::types::SchemaError;
use crate::db_operations::error_utils::ErrorUtils;

impl DbOperations {
    /// Store a public key in the database
    pub fn store_public_key(&self, key_info: &PublicKeyInfo) -> Result<(), SchemaError> {
        self.store_in_tree(&self.public_keys_tree, &key_info.id, key_info)
    }

    /// Retrieve a public key by ID
    pub fn get_public_key(&self, key_id: &str) -> Result<Option<PublicKeyInfo>, SchemaError> {
        self.get_from_tree(&self.public_keys_tree, key_id)
    }

    /// List all public key IDs
    pub fn list_public_key_ids(&self) -> Result<Vec<String>, SchemaError> {
        self.list_keys_in_tree(&self.public_keys_tree)
    }

    /// Get all public keys
    pub fn get_all_public_keys(&self) -> Result<Vec<PublicKeyInfo>, SchemaError> {
        let key_ids = self.list_public_key_ids()?;
        let mut keys = Vec::new();
        
        for key_id in key_ids {
            if let Some(key_info) = self.get_public_key(&key_id)? {
                keys.push(key_info);
            }
        }
        
        Ok(keys)
    }

    /// Delete a public key from the database
    pub fn delete_public_key(&self, key_id: &str) -> Result<bool, SchemaError> {
        match self.public_keys_tree.remove(key_id.as_bytes()) {
            Ok(old_value) => Ok(old_value.is_some()),
            Err(e) => Err(ErrorUtils::database_error("delete public key", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_utils::TestDatabaseFactory;
    use crate::security::types::PublicKeyInfo;

    #[test]
    fn test_store_and_retrieve_public_key() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        
        let key_info = PublicKeyInfo::new(
            "test_key_id".to_string(),
            "test_public_key_base64".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );

        // Store the key
        db_ops.store_public_key(&key_info).unwrap();
        
        // Retrieve the key
        let retrieved = db_ops.get_public_key("test_key_id").unwrap();
        assert!(retrieved.is_some());
        let retrieved_key = retrieved.unwrap();
        assert_eq!(retrieved_key.id, "test_key_id");
        assert_eq!(retrieved_key.public_key, "test_public_key_base64");
        assert_eq!(retrieved_key.owner_id, "test_owner");
        assert_eq!(retrieved_key.permissions, vec!["read".to_string()]);
    }

    #[test]
    fn test_get_nonexistent_public_key() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        
        // Try to retrieve a key that doesn't exist
        let result = db_ops.get_public_key("nonexistent_key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_and_delete_public_keys() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        
        // Store multiple keys
        for i in 0..3 {
            let key_info = PublicKeyInfo::new(
                format!("key_{}", i),
                "test_key".to_string(),
                "test_owner".to_string(),
                vec!["read".to_string()],
            );
            db_ops.store_public_key(&key_info).unwrap();
        }

        // List keys
        let key_ids = db_ops.list_public_key_ids().unwrap();
        assert_eq!(key_ids.len(), 3);
        assert!(key_ids.contains(&"key_0".to_string()));
        assert!(key_ids.contains(&"key_1".to_string()));
        assert!(key_ids.contains(&"key_2".to_string()));

        // Delete a key
        let deleted = db_ops.delete_public_key("key_1").unwrap();
        assert!(deleted);

        // Verify deletion
        let remaining = db_ops.list_public_key_ids().unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(!remaining.contains(&"key_1".to_string()));
        
        // Try to delete the same key again
        let deleted_again = db_ops.delete_public_key("key_1").unwrap();
        assert!(!deleted_again);
    }

    #[test]
    fn test_get_all_public_keys() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        
        // Store multiple keys with different attributes
        let key1 = PublicKeyInfo::new(
            "key_1".to_string(),
            "public_key_1".to_string(),
            "owner_1".to_string(),
            vec!["read".to_string()],
        );
        let key2 = PublicKeyInfo::new(
            "key_2".to_string(),
            "public_key_2".to_string(),
            "owner_2".to_string(),
            vec!["write".to_string()],
        );
        
        db_ops.store_public_key(&key1).unwrap();
        db_ops.store_public_key(&key2).unwrap();

        // Get all keys
        let all_keys = db_ops.get_all_public_keys().unwrap();
        assert_eq!(all_keys.len(), 2);
        
        // Find keys by ID to verify content
        let retrieved_key1 = all_keys.iter().find(|k| k.id == "key_1").unwrap();
        let retrieved_key2 = all_keys.iter().find(|k| k.id == "key_2").unwrap();
        
        assert_eq!(retrieved_key1.public_key, "public_key_1");
        assert_eq!(retrieved_key1.owner_id, "owner_1");
        assert_eq!(retrieved_key2.public_key, "public_key_2");
        assert_eq!(retrieved_key2.owner_id, "owner_2");
    }

    #[test]
    fn test_store_update_public_key() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        
        // Store initial key
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "original_key".to_string(),
            "original_owner".to_string(),
            vec!["read".to_string()],
        );
        db_ops.store_public_key(&key_info).unwrap();
        
        // Update the key (same ID, different content)
        let updated_key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "updated_key".to_string(),
            "updated_owner".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );
        db_ops.store_public_key(&updated_key_info).unwrap();
        
        // Verify the key was updated
        let retrieved = db_ops.get_public_key("test_key").unwrap().unwrap();
        assert_eq!(retrieved.public_key, "updated_key");
        assert_eq!(retrieved.owner_id, "updated_owner");
        assert_eq!(retrieved.permissions.len(), 2);
    }
}