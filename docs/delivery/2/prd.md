# PBI-2: HashAtomRef Type with Complete Database Operations

## Overview

This PBI implements a new Hash-based AtomRef type with complete database operations, providing efficient O(1) key-value storage and retrieval capabilities as a foundation for the composable AtomRef system.

[View in Backlog](../backlog.md#user-content-2)

## Problem Statement

The current AtomRef system only supports three basic types (Single, Range, Collection) and lacks efficient hash-based lookups. We need a HashAtomRef type that provides O(1) key access with complete database operations (insert, get, remove, update, delete) to support advanced data modeling patterns and serve as a foundation for composable types.

## User Stories

### Primary User Story
As a developer, I want a HashAtomRef type with complete database operations, so that I can efficiently store and retrieve key-value data with O(1) access.

### Supporting User Stories
- As a developer, I want HashAtomRef with O(1) key lookups, so that I can efficiently access data by key
- As a developer, I want complete CRUD operations for hash ARefs, so that I can manage hash data lifecycle
- As a developer, I want database integration for hash ARefs, so that hash data persists correctly
- As a developer, I want hash ARef error handling, so that invalid operations are handled gracefully

## Technical Approach

### HashAtomRef Implementation

#### Core AtomRefHash Structure
```rust
/// Hash-based AtomRef for key-value lookups with O(1) access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefHash {
    uuid: String,
    /// HashMap for O(1) key lookups
    pub atom_uuids: HashMap<String, String>,
    updated_at: DateTime<Utc>,
    status: AtomRefStatus,
    update_history: Vec<AtomRefUpdate>,
}

impl AtomRefHash {
    pub fn new(source_pub_key: String) -> Self;
    pub fn set_atom_uuid(&mut self, key: String, atom_uuid: String);
    pub fn get_atom_uuid(&self, key: &str) -> Option<&String>;
    pub fn remove_atom_uuid(&mut self, key: &str) -> Option<String>;
    pub fn contains_key(&self, key: &str) -> bool;
    pub fn keys(&self) -> impl Iterator<Item = &String>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn clear(&mut self);
}

impl AtomRefBehavior for AtomRefHash {
    fn uuid(&self) -> &str;
    fn updated_at(&self) -> &DateTime<Utc>;
    fn status(&self) -> &AtomRefStatus;
    fn update_history(&self) -> &[AtomRefUpdate];
}
```

### Database Operations for HashAtomRef

#### Enhanced DbOperations
```rust
impl DbOperations {
    /// Create a new hash AtomRef
    pub fn create_hash_atom_ref(
        &self,
        aref_uuid: &str,
        source_pub_key: String,
    ) -> Result<AtomRefHash, DbError> {
        let hash_aref = AtomRefHash::new(source_pub_key);
        let key = format!("ref:{}", aref_uuid);
        self.store_item(&key, &hash_aref)?;
        Ok(hash_aref)
    }
    
    /// Insert/update a key-value pair in hash AtomRef
    pub fn hash_insert(
        &self,
        aref_uuid: &str,
        hash_key: String,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<(), DbError> {
        let key = format!("ref:{}", aref_uuid);
        let mut hash_aref: AtomRefHash = self.get_item(&key)?
            .ok_or_else(|| DbError::ARefNotFound(aref_uuid.to_string()))?;
        
        hash_aref.set_atom_uuid(hash_key, atom_uuid);
        hash_aref.add_update_history(AtomRefUpdate {
            timestamp: Utc::now(),
            status: AtomRefStatus::Active,
            source_pub_key,
        });
        
        self.store_item(&key, &hash_aref)?;
        Ok(())
    }
    
    /// Get value from hash AtomRef by key
    pub fn hash_get(
        &self,
        aref_uuid: &str,
        hash_key: &str,
    ) -> Result<Option<String>, DbError> {
        let key = format!("ref:{}", aref_uuid);
        let hash_aref: AtomRefHash = self.get_item(&key)?
            .ok_or_else(|| DbError::ARefNotFound(aref_uuid.to_string()))?;
        
        Ok(hash_aref.get_atom_uuid(hash_key).cloned())
    }
    
    /// Remove a key from hash AtomRef
    pub fn hash_remove(
        &self,
        aref_uuid: &str,
        hash_key: &str,
        source_pub_key: String,
    ) -> Result<Option<String>, DbError> {
        let key = format!("ref:{}", aref_uuid);
        let mut hash_aref: AtomRefHash = self.get_item(&key)?
            .ok_or_else(|| DbError::ARefNotFound(aref_uuid.to_string()))?;
        
        let removed = hash_aref.remove_atom_uuid(hash_key);
        if removed.is_some() {
            hash_aref.add_update_history(AtomRefUpdate {
                timestamp: Utc::now(),
                status: AtomRefStatus::Active,
                source_pub_key,
            });
            
            self.store_item(&key, &hash_aref)?;
        }
        
        Ok(removed)
    }
    
    /// Update hash AtomRef (replace entire hash)
    pub fn hash_update(
        &self,
        aref_uuid: &str,
        new_hash_data: HashMap<String, String>,
        source_pub_key: String,
    ) -> Result<(), DbError> {
        let key = format!("ref:{}", aref_uuid);
        let mut hash_aref: AtomRefHash = self.get_item(&key)?
            .ok_or_else(|| DbError::ARefNotFound(aref_uuid.to_string()))?;
        
        hash_aref.atom_uuids = new_hash_data;
        hash_aref.updated_at = Utc::now();
        hash_aref.add_update_history(AtomRefUpdate {
            timestamp: Utc::now(),
            status: AtomRefStatus::Active,
            source_pub_key,
        });
        
        self.store_item(&key, &hash_aref)?;
        Ok(())
    }
    
    /// Delete hash AtomRef entirely
    pub fn hash_delete(&self, aref_uuid: &str) -> Result<(), DbError> {
        let key = format!("ref:{}", aref_uuid);
        self.delete_item(&key)
    }
    
    /// Get all keys from hash AtomRef
    pub fn hash_keys(&self, aref_uuid: &str) -> Result<Vec<String>, DbError> {
        let key = format!("ref:{}", aref_uuid);
        let hash_aref: AtomRefHash = self.get_item(&key)?
            .ok_or_else(|| DbError::ARefNotFound(aref_uuid.to_string()))?;
        
        Ok(hash_aref.keys().cloned().collect())
    }
    
    /// Check if hash AtomRef contains key
    pub fn hash_contains_key(&self, aref_uuid: &str, hash_key: &str) -> Result<bool, DbError> {
        let key = format!("ref:{}", aref_uuid);
        let hash_aref: AtomRefHash = self.get_item(&key)?
            .ok_or_else(|| DbError::ARefNotFound(aref_uuid.to_string()))?;
        
        Ok(hash_aref.contains_key(hash_key))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HashARefError {
    #[error("Hash key not found: {key}")]
    KeyNotFound { key: String },
    #[error("Hash operation failed: {operation}")]
    OperationFailed { operation: String },
    #[error("Invalid hash key: {key}")]
    InvalidKey { key: String },
}
```

### Implementation Plan

#### Phase 1: Core HashAtomRef Implementation (Days 1-2)
1. Implement `AtomRefHash` struct with HashMap storage
2. Add all core hash operations (set, get, remove, contains, keys, etc.)
3. Implement `AtomRefBehavior` trait for consistency
4. Add comprehensive unit tests for hash operations

#### Phase 2: Database Operations (Days 3-4)
1. Implement all database operations in `DbOperations`
2. Add CRUD operations: create, insert, get, remove, update, delete
3. Add utility operations: keys, contains_key, size checking
4. Implement proper error handling with `HashARefError`

#### Phase 3: Integration and Testing (Days 5)
1. Integration tests with existing AtomRef infrastructure
2. Performance testing for O(1) operations
3. Database persistence validation
4. Error scenario testing

## UX/UI Considerations

### Developer Experience
- Clear error messages for invalid compositions
- Comprehensive documentation with examples
- Type-safe APIs that prevent runtime errors

### API Consistency
- Maintain existing AtomRef behavior for backward compatibility
- Consistent method signatures across all AtomRef types
- Clear naming conventions for new types

## Acceptance Criteria

1. **AtomRefHash Implementation**
   - [ ] AtomRefHash struct with HashMap-based storage
   - [ ] O(1) key lookup performance verified
   - [ ] Complete hash operations (set, get, remove, contains, keys, len, clear)
   - [ ] Proper serialization/deserialization support
   - [ ] AtomRefBehavior trait implementation

2. **Database Operations**
   - [ ] All CRUD operations implemented (create, insert, get, remove, update, delete)
   - [ ] Utility operations working (hash_keys, hash_contains_key)
   - [ ] Proper error handling with HashARefError
   - [ ] Database persistence validated
   - [ ] Update history tracking for all modifications

3. **Performance Requirements**
   - [ ] O(1) key lookup performance verified through benchmarks
   - [ ] Hash operations faster than equivalent Range operations
   - [ ] Memory usage reasonable for large hash datasets
   - [ ] Database operations complete within acceptable time limits

4. **Testing**
   - [ ] Unit tests for all hash operations
   - [ ] Unit tests for all database operations
   - [ ] Error case testing for invalid operations
   - [ ] Performance tests for O(1) guarantees
   - [ ] Integration tests with existing AtomRef infrastructure

5. **Error Handling**
   - [ ] HashARefError covers all failure scenarios
   - [ ] Clear error messages for invalid operations
   - [ ] Graceful handling of missing keys and ARefs
   - [ ] Database error propagation working correctly

6. **Integration**
   - [ ] Compatible with existing AtomRef infrastructure
   - [ ] All traits implemented (Clone, Debug, Serialize, Deserialize)
   - [ ] No breaking changes to current AtomRef interfaces
   - [ ] Ready for use in composable AtomRef framework

## Dependencies

- **Internal**: Current AtomRef types (AtomRef, AtomRefRange, AtomRefCollection), AtomRefBehavior trait, database operations
- **External**: HashMap, serde, chrono, uuid crates
- **Testing**: Existing test infrastructure, benchmarking framework

## Open Questions

1. **Key Validation**: Should we enforce specific key format restrictions for hash operations?
2. **Memory Management**: What's the optimal strategy for large hash datasets?
3. **Performance Targets**: What are acceptable benchmarks for O(1) operations under load?
4. **Concurrency**: How should we handle concurrent access to hash ARefs?
5. **Migration**: How do we migrate existing data to use hash ARefs when needed?

## Related Tasks

This PBI will generate detailed tasks covering:
- AtomRefHash struct implementation with all core operations
- Database operations implementation for all hash CRUD operations
- Performance testing and O(1) verification
- Integration testing with existing AtomRef infrastructure
- Error handling and validation framework 