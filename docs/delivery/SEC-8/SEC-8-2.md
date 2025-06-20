# SEC-8-2: Update MessageVerifier for Persistence

[Back to task list](./tasks.md)

## Description

Modify the `MessageVerifier` to load persisted public keys from the database on startup and save new keys to the database when they are registered. This ensures that public keys persist across node restarts while maintaining all existing functionality.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 14:12:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

### Functional Requirements
- Load all persisted public keys from database on `MessageVerifier` initialization
- Save new public keys to database immediately when registered
- Maintain existing in-memory HashMap for performance
- Support graceful fallback if database operations fail
- Preserve all existing public key verification functionality

### Technical Requirements
- Modify `MessageVerifier::new()` to accept `DbOperations` parameter
- Add database operations while maintaining in-memory cache
- Handle database errors without breaking verification functionality
- Maintain thread safety for concurrent operations
- Follow existing error handling patterns

### Dependencies
- SEC-8-1: Public Key Database Operations (must be completed first)
- Existing `MessageVerifier` implementation
- `DbOperations` integration

## Implementation Plan

### 1. Update MessageVerifier constructor
**Location**: `src/security/signing.rs`

```rust
/// Message verifier for Ed25519 signatures with persistence
pub struct MessageVerifier {
    /// Registered public keys (in-memory cache)
    public_keys: Arc<RwLock<HashMap<String, PublicKeyInfo>>>,
    /// Database operations for persistence
    db_ops: Option<Arc<crate::db_operations::DbOperations>>,
    /// Maximum allowed timestamp drift in seconds
    max_timestamp_drift: i64,
}

impl MessageVerifier {
    /// Create a new message verifier with optional database persistence
    pub fn new(max_timestamp_drift: i64) -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
            db_ops: None,
            max_timestamp_drift,
        }
    }

    /// Create a new message verifier with database persistence
    pub fn new_with_persistence(
        max_timestamp_drift: i64,
        db_ops: Arc<crate::db_operations::DbOperations>
    ) -> SecurityResult<Self> {
        let mut verifier = Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
            db_ops: Some(db_ops.clone()),
            max_timestamp_drift,
        };

        // Load persisted keys from database
        verifier.load_persisted_keys()?;
        Ok(verifier)
    }

    /// Load all persisted public keys from database into memory
    fn load_persisted_keys(&self) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            match db_ops.get_all_public_keys() {
                Ok(persisted_keys) => {
                    let mut keys = self.public_keys.write()
                        .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
                    
                    for key_info in persisted_keys {
                        keys.insert(key_info.id.clone(), key_info);
                    }
                    
                    log::info!("Loaded {} public keys from database", keys.len());
                    Ok(())
                }
                Err(e) => {
                    log::warn!("Failed to load persisted public keys: {}", e);
                    // Don't fail initialization - continue without persisted keys
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Persist a public key to database
    fn persist_public_key(&self, key_info: &PublicKeyInfo) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            match db_ops.store_public_key(key_info) {
                Ok(()) => {
                    log::debug!("Persisted public key: {}", key_info.id);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to persist public key {}: {}", key_info.id, e);
                    // Don't fail the operation - key is still in memory
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }
}
```

### 2. Update register_public_key method
**Location**: `src/security/signing.rs`

```rust
impl MessageVerifier {
    /// Register a public key with automatic persistence
    pub fn register_public_key(&self, key_info: PublicKeyInfo) -> SecurityResult<()> {
        // Store in memory first
        {
            let mut keys = self.public_keys.write()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
            keys.insert(key_info.id.clone(), key_info.clone());
        }

        // Then persist to database
        self.persist_public_key(&key_info)?;

        log::info!("Registered public key: {}", key_info.id);
        Ok(())
    }
}
```

### 3. Update remove_public_key method
**Location**: `src/security/signing.rs`

```rust
impl MessageVerifier {
    /// Remove a public key from both memory and database
    pub fn remove_public_key(&self, key_id: &str) -> SecurityResult<()> {
        // Remove from memory
        {
            let mut keys = self.public_keys.write()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
            keys.remove(key_id);
        }

        // Remove from database
        if let Some(db_ops) = &self.db_ops {
            match db_ops.delete_public_key(key_id) {
                Ok(_) => log::debug!("Removed public key from database: {}", key_id),
                Err(e) => log::error!("Failed to remove public key from database {}: {}", key_id, e),
            }
        }

        log::info!("Removed public key: {}", key_id);
        Ok(())
    }
}
```

### 4. Update SecurityManager integration
**Location**: `src/security/utils.rs`

```rust
impl SecurityManager {
    /// Create a new security manager with database persistence
    pub fn new(config: crate::security::SecurityConfig) -> SecurityResult<Self> {
        let verifier = Arc::new(MessageVerifier::new(300)); // 5 minute timestamp drift
        
        let encryption = Arc::new(ConditionalEncryption::new(
            config.encrypt_at_rest,
            config.master_key,
        )?);
        
        Ok(Self {
            verifier,
            encryption,
            config,
        })
    }

    /// Create a new security manager with database persistence
    pub fn new_with_persistence(
        config: crate::security::SecurityConfig,
        db_ops: Arc<crate::db_operations::DbOperations>
    ) -> SecurityResult<Self> {
        let verifier = Arc::new(MessageVerifier::new_with_persistence(300, db_ops)?);
        
        let encryption = Arc::new(ConditionalEncryption::new(
            config.encrypt_at_rest,
            config.master_key,
        )?);
        
        Ok(Self {
            verifier,
            encryption,
            config,
        })
    }
}
```

### 5. Add comprehensive unit tests
**Location**: `src/security/signing.rs`

```rust
#[cfg(test)]
mod persistence_tests {
    use super::*;
    use crate::testing_utils::TestDatabaseFactory;

    #[test]
    fn test_message_verifier_persistence() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let db_ops = Arc::new(db_ops);

        // Create verifier with persistence
        let verifier = MessageVerifier::new_with_persistence(300, db_ops.clone()).unwrap();

        // Register a key
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "test_public_key".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_public_key(key_info.clone()).unwrap();

        // Verify key is in memory
        let retrieved = verifier.get_public_key("test_key").unwrap();
        assert!(retrieved.is_some());

        // Create new verifier instance to simulate restart
        let verifier2 = MessageVerifier::new_with_persistence(300, db_ops).unwrap();

        // Verify key was loaded from database
        let retrieved2 = verifier2.get_public_key("test_key").unwrap();
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap().id, "test_key");
    }

    #[test]
    fn test_remove_public_key_persistence() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let db_ops = Arc::new(db_ops);

        let verifier = MessageVerifier::new_with_persistence(300, db_ops.clone()).unwrap();

        // Register and then remove a key
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "test_public_key".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_public_key(key_info).unwrap();
        verifier.remove_public_key("test_key").unwrap();

        // Create new verifier to check persistence
        let verifier2 = MessageVerifier::new_with_persistence(300, db_ops).unwrap();
        let retrieved = verifier2.get_public_key("test_key").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_graceful_database_failure() {
        // Test that MessageVerifier continues to work even if database operations fail
        let verifier = MessageVerifier::new(300); // No database
        
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "test_public_key".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        
        // Should work fine without database
        verifier.register_public_key(key_info).unwrap();
        let retrieved = verifier.get_public_key("test_key").unwrap();
        assert!(retrieved.is_some());
    }
}
```

## Verification

### Acceptance Criteria
- [ ] `MessageVerifier` loads persisted keys on initialization
- [ ] New public keys are automatically saved to database
- [ ] Removed public keys are deleted from database
- [ ] All existing verification functionality continues to work
- [ ] Graceful handling of database failures
- [ ] Thread-safe operations maintained
- [ ] Unit tests pass with comprehensive coverage

### Test Plan
1. **Persistence Tests**: Verify keys persist across verifier restarts
2. **Performance Tests**: Ensure database operations don't impact verification speed
3. **Error Handling**: Test graceful degradation when database is unavailable
4. **Integration Tests**: Verify SecurityManager integration works correctly

## Files Modified

- `src/security/signing.rs` (modified - add persistence support)
- `src/security/utils.rs` (modified - update SecurityManager constructor)