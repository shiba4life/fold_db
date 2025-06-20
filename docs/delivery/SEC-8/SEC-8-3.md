# SEC-8-3: Add Migration for Existing Keys

[Back to task list](./tasks.md)

## Description

Create a migration utility to handle existing in-memory public keys during the upgrade to persistent storage. This ensures that deployments with existing registered keys can smoothly transition to the new persistence system without losing authentication state.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 14:13:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

### Functional Requirements
- Detect when the system is upgrading from non-persistent to persistent storage
- Migrate existing in-memory keys to database during first startup with persistence
- Provide CLI command for manual migration if needed
- Handle edge cases like partial migrations or corrupted data
- Ensure idempotent migration (safe to run multiple times)

### Technical Requirements
- Add migration detection logic to `MessageVerifier`
- Create standalone migration utility for manual operations
- Add proper logging and error handling for migration process
- Maintain backward compatibility during transition period
- Document migration process for operators

### Dependencies
- SEC-8-1: Public Key Database Operations
- SEC-8-2: MessageVerifier Persistence Support
- Existing node configuration system

## Implementation Plan

### 1. Add migration detection to MessageVerifier
**Location**: `src/security/signing.rs`

```rust
impl MessageVerifier {
    /// Check if migration is needed and perform it
    fn check_and_migrate_keys(&self) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            // Check if any keys exist in database
            match db_ops.list_public_key_ids() {
                Ok(existing_keys) => {
                    if existing_keys.is_empty() {
                        log::info!("No existing keys in database - migration check complete");
                    } else {
                        log::info!("Found {} existing keys in database", existing_keys.len());
                    }
                    Ok(())
                }
                Err(e) => {
                    log::warn!("Could not check for existing keys during migration: {}", e);
                    Ok(()) // Don't fail startup for migration issues
                }
            }
        } else {
            Ok(())
        }
    }

    /// Migrate in-memory keys to database (for development/testing scenarios)
    pub fn migrate_memory_keys_to_database(&self) -> SecurityResult<usize> {
        if let Some(db_ops) = &self.db_ops {
            let keys = self.public_keys.read()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire read lock".to_string()))?;
            
            let mut migrated_count = 0;
            for (key_id, key_info) in keys.iter() {
                // Check if key already exists in database
                match db_ops.get_public_key(key_id) {
                    Ok(Some(_)) => {
                        log::debug!("Key {} already exists in database, skipping", key_id);
                    }
                    Ok(None) => {
                        // Key doesn't exist, migrate it
                        match db_ops.store_public_key(key_info) {
                            Ok(()) => {
                                migrated_count += 1;
                                log::info!("Migrated key to database: {}", key_id);
                            }
                            Err(e) => {
                                log::error!("Failed to migrate key {}: {}", key_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error checking existing key {}: {}", key_id, e);
                    }
                }
            }
            
            log::info!("Migration complete - migrated {} keys to database", migrated_count);
            Ok(migrated_count)
        } else {
            Err(SecurityError::KeyNotFound("No database available for migration".to_string()))
        }
    }
}
```

### 2. Create standalone migration CLI command
**Location**: `src/bin/key_migration.rs`

```rust
//! Standalone utility for migrating public keys to persistent storage

use clap::{Arg, Command};
use datafold::security::{SecurityManager, SecurityConfig};
use datafold::db_operations::DbOperations;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("key-migration")
        .version("1.0")
        .about("Migrate public keys to persistent storage")
        .arg(
            Arg::new("database-path")
                .short('d')
                .long("database")
                .value_name("PATH")
                .help("Path to the database directory")
                .required(true),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be migrated without making changes")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .help("Force migration even if keys already exist in database")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let database_path = matches.get_one::<String>("database-path").unwrap();
    let dry_run = matches.get_flag("dry-run");
    let force = matches.get_flag("force");

    println!("Key Migration Utility");
    println!("Database path: {}", database_path);
    
    if dry_run {
        println!("DRY RUN MODE - no changes will be made");
    }

    // Open database
    let db = sled::open(database_path)?;
    let db_ops = Arc::new(DbOperations::new(db)?);

    // Check current state
    let existing_keys = db_ops.list_public_key_ids()?;
    println!("Found {} existing keys in database", existing_keys.len());

    if !existing_keys.is_empty() && !force {
        println!("Database already contains keys. Use --force to proceed anyway.");
        return Ok(());
    }

    if dry_run {
        println!("Would migrate keys from memory to database");
        println!("Note: This utility cannot access in-memory keys from a stopped node");
        println!("Migration happens automatically when starting node with persistence enabled");
    } else {
        println!("Migration utility complete");
        println!("Start your node with persistence enabled to perform automatic migration");
    }

    Ok(())
}
```

### 3. Add migration support to node startup
**Location**: `src/datafold_node/node.rs`

```rust
impl DatafoldNode {
    /// Initialize security with automatic migration support
    pub fn initialize_security_with_migration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = SecurityConfig::from_env();
        
        if let Some(db_ops) = &self.db_ops {
            // Create SecurityManager with persistence
            let security_manager = SecurityManager::new_with_persistence(config, db_ops.clone())?;
            
            // Check if migration is needed
            log::info!("Checking for public key migration requirements...");
            
            self.security_manager = Some(Arc::new(security_manager));
            log::info!("Security system initialized with persistence support");
        } else {
            // Fallback to non-persistent mode
            let security_manager = SecurityManager::new(config)?;
            self.security_manager = Some(Arc::new(security_manager));
            log::warn!("Security system initialized without persistence");
        }

        Ok(())
    }
}
```

### 4. Add migration configuration
**Location**: `src/security/utils.rs`

```rust
/// Migration configuration for public key persistence
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to enable automatic migration
    pub auto_migrate: bool,
    /// Maximum number of keys to migrate in one batch
    pub batch_size: usize,
    /// Whether to backup keys before migration
    pub backup_keys: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_migrate: true,
            batch_size: 100,
            backup_keys: true,
        }
    }
}

impl MigrationConfig {
    /// Load migration configuration from environment
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(value) = std::env::var("DATAFOLD_AUTO_MIGRATE_KEYS") {
            config.auto_migrate = value.parse().unwrap_or(true);
        }
        
        if let Ok(value) = std::env::var("DATAFOLD_MIGRATION_BATCH_SIZE") {
            config.batch_size = value.parse().unwrap_or(100);
        }
        
        config
    }
}
```

### 5. Add comprehensive migration tests
**Location**: `src/security/migration_tests.rs`

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    use crate::testing_utils::TestDatabaseFactory;

    #[test]
    fn test_empty_database_migration() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let db_ops = Arc::new(db_ops);

        // Create verifier with some in-memory keys
        let verifier = MessageVerifier::new(300);
        
        // Add some keys to memory only
        let key1 = PublicKeyInfo::new("key1".to_string(), "pubkey1".to_string(), "owner1".to_string(), vec!["read".to_string()]);
        let key2 = PublicKeyInfo::new("key2".to_string(), "pubkey2".to_string(), "owner2".to_string(), vec!["write".to_string()]);
        
        verifier.register_public_key(key1).unwrap();
        verifier.register_public_key(key2).unwrap();

        // Now add database persistence
        verifier.db_ops = Some(db_ops.clone());

        // Perform migration
        let migrated = verifier.migrate_memory_keys_to_database().unwrap();
        assert_eq!(migrated, 2);

        // Verify keys are in database
        let db_key1 = db_ops.get_public_key("key1").unwrap();
        let db_key2 = db_ops.get_public_key("key2").unwrap();
        
        assert!(db_key1.is_some());
        assert!(db_key2.is_some());
    }

    #[test]
    fn test_idempotent_migration() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let db_ops = Arc::new(db_ops);

        let verifier = MessageVerifier::new(300);
        
        // Add key to memory
        let key = PublicKeyInfo::new("key1".to_string(), "pubkey1".to_string(), "owner1".to_string(), vec!["read".to_string()]);
        verifier.register_public_key(key).unwrap();

        verifier.db_ops = Some(db_ops.clone());

        // First migration
        let migrated1 = verifier.migrate_memory_keys_to_database().unwrap();
        assert_eq!(migrated1, 1);

        // Second migration should not migrate anything
        let migrated2 = verifier.migrate_memory_keys_to_database().unwrap();
        assert_eq!(migrated2, 0);
    }

    #[test]
    fn test_partial_migration_recovery() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let db_ops = Arc::new(db_ops);

        let verifier = MessageVerifier::new(300);
        
        // Add keys to memory
        let key1 = PublicKeyInfo::new("key1".to_string(), "pubkey1".to_string(), "owner1".to_string(), vec!["read".to_string()]);
        let key2 = PublicKeyInfo::new("key2".to_string(), "pubkey2".to_string(), "owner2".to_string(), vec!["write".to_string()]);
        
        verifier.register_public_key(key1).unwrap();
        verifier.register_public_key(key2).unwrap();

        // Manually store one key to simulate partial migration
        let key1_info = verifier.get_public_key("key1").unwrap().unwrap();
        db_ops.store_public_key(&key1_info).unwrap();

        verifier.db_ops = Some(db_ops.clone());

        // Migration should only migrate the missing key
        let migrated = verifier.migrate_memory_keys_to_database().unwrap();
        assert_eq!(migrated, 1);

        // Verify both keys are now in database
        assert!(db_ops.get_public_key("key1").unwrap().is_some());
        assert!(db_ops.get_public_key("key2").unwrap().is_some());
    }
}
```

## Verification

### Acceptance Criteria
- [ ] Migration detects upgrade scenarios correctly
- [ ] In-memory keys are migrated to database automatically
- [ ] Migration is idempotent (safe to run multiple times)
- [ ] CLI utility provides manual migration capability
- [ ] Partial migration scenarios are handled gracefully
- [ ] Migration process is properly logged
- [ ] Node startup includes migration support

### Test Plan
1. **Automatic Migration**: Test migration during node startup
2. **Idempotency**: Verify migration can be run multiple times safely
3. **Partial Migration**: Test recovery from incomplete migrations
4. **CLI Utility**: Test standalone migration command
5. **Error Handling**: Test behavior with database failures during migration

## Files Modified

- `src/security/signing.rs` (modified - add migration support)
- `src/bin/key_migration.rs` (new - standalone migration utility)
- `src/datafold_node/node.rs` (modified - add migration to startup)
- `src/security/utils.rs` (modified - add migration configuration)
- `src/security/migration_tests.rs` (new - migration test suite)