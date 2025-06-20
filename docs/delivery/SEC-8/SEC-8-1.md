# SEC-8-1: Add Public Key Database Operations

[Back to task list](./tasks.md)

## Description

Create database operations for storing and retrieving public keys in the sled database. This task adds a new `public_key_operations.rs` module to `DbOperations` that provides persistent storage for `PublicKeyInfo` objects.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 14:11:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

### Functional Requirements
- Store `PublicKeyInfo` objects in sled database using key_id as the key
- Retrieve individual public keys by key_id
- List all stored public keys
- Delete public keys by key_id
- Handle serialization/deserialization errors gracefully

### Technical Requirements
- Follow existing `DbOperations` patterns and error handling
- Use sled tree named "public_keys"
- Integrate with existing `SchemaError` error types
- Maintain consistency with other database operations modules

### Dependencies
- Existing `DbOperations` infrastructure
- Sled database integration
- `PublicKeyInfo` serialization support

## Implementation Plan

### 1. Create public_key_operations.rs module
**Location**: `src/db_operations/public_key_operations.rs`

```rust
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
        self.list_tree_keys(&self.public_keys_tree)
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
```

### 2. Update DbOperations core to include public_keys tree
**Location**: `src/db_operations/core.rs`

Add to struct:
```rust
/// Tree for storing public keys
public_keys_tree: Tree,
```

Add to `new()` method:
```rust
let public_keys_tree = db.open_tree("public_keys")?;
```

### 3. Update module exports
**Location**: `src/db_operations/mod.rs`

Add:
```rust
mod public_key_operations;
```

### 4. Add unit tests
**Location**: `src/db_operations/public_key_operations.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(retrieved.unwrap().id, "test_key_id");
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

        // Delete a key
        let deleted = db_ops.delete_public_key("key_1").unwrap();
        assert!(deleted);

        // Verify deletion
        let remaining = db_ops.list_public_key_ids().unwrap();
        assert_eq!(remaining.len(), 2);
    }
}
```

## Verification

### Acceptance Criteria
- [ ] `DbOperations` can store `PublicKeyInfo` objects
- [ ] `DbOperations` can retrieve public keys by ID
- [ ] `DbOperations` can list all public key IDs
- [ ] `DbOperations` can delete public keys
- [ ] All operations handle errors gracefully
- [ ] Unit tests pass with 100% coverage
- [ ] Integration with existing error handling patterns

### Test Plan
1. **Unit Tests**: Test all CRUD operations for public keys
2. **Error Handling**: Test database failures, serialization errors
3. **Integration**: Verify compatibility with existing `DbOperations` patterns
4. **Performance**: Ensure operations are efficient for typical key counts

## Files Modified

- `src/db_operations/public_key_operations.rs` (new)
- `src/db_operations/core.rs` (modified - add public_keys_tree)
- `src/db_operations/mod.rs` (modified - add module export)