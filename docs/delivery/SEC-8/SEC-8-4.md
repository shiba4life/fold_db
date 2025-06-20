# SEC-8-4: Integration Tests for Key Persistence

[Back to task list](./tasks.md)

## Description

Create comprehensive integration tests to verify that public key persistence works correctly across node restarts, handles failure scenarios gracefully, and maintains all security functionality. These tests will validate the entire persistence system end-to-end.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 14:13:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

### Functional Requirements
- Test public key persistence across simulated node restarts
- Verify that all security operations work with persisted keys
- Test failure scenarios (database corruption, disk full, etc.)
- Validate migration scenarios from non-persistent to persistent
- Test concurrent operations during persistence operations
- Verify performance impact of persistence operations

### Technical Requirements
- Use temporary databases for isolated testing
- Simulate node restart scenarios in tests
- Test both successful and failure paths
- Include performance benchmarks
- Test thread safety and concurrent access
- Validate data integrity across restarts

### Dependencies
- SEC-8-1: Public Key Database Operations
- SEC-8-2: MessageVerifier Persistence Support  
- SEC-8-3: Migration Support
- Existing test infrastructure

## Implementation Plan

### 1. Create integration test suite
**Location**: `tests/integration/public_key_persistence.rs`

```rust
//! Integration tests for public key persistence across node restarts

use datafold::security::{SecurityManager, SecurityConfig, PublicKeyInfo, KeyRegistrationRequest};
use datafold::db_operations::DbOperations;
use std::sync::Arc;
use tempfile::TempDir;

/// Test helper to create a temporary database
fn create_temp_database() -> (TempDir, Arc<DbOperations>) {
    let temp_dir = TempDir::new().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = Arc::new(DbOperations::new(db).unwrap());
    (temp_dir, db_ops)
}

/// Test helper to create a test public key
fn create_test_key(id: &str, owner: &str) -> PublicKeyInfo {
    PublicKeyInfo::new(
        id.to_string(),
        format!("pubkey_{}", id),
        owner.to_string(),
        vec!["read".to_string(), "write".to_string()],
    )
}

#[tokio::test]
async fn test_key_persistence_across_restart() {
    let (_temp_dir, db_ops) = create_temp_database();
    
    // Phase 1: Create SecurityManager and register keys
    {
        let config = SecurityConfig::new().with_encryption_disabled();
        let security_manager = SecurityManager::new_with_persistence(config, db_ops.clone()).unwrap();
        
        // Register some keys
        let key1 = KeyRegistrationRequest {
            public_key: "pubkey1_base64".to_string(),
            owner_id: "user1".to_string(),
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
            expires_at: None,
        };
        
        let key2 = KeyRegistrationRequest {
            public_key: "pubkey2_base64".to_string(),
            owner_id: "user2".to_string(),
            permissions: vec!["write".to_string()],
            metadata: std::collections::HashMap::new(),
            expires_at: None,
        };
        
        let response1 = security_manager.register_public_key(key1).unwrap();
        let response2 = security_manager.register_public_key(key2).unwrap();
        
        assert!(response1.success);
        assert!(response2.success);
        
        // Verify keys are accessible
        let keys = security_manager.list_public_keys().unwrap();
        assert_eq!(keys.len(), 2);
    }
    
    // Phase 2: Simulate restart by creating new SecurityManager with same database
    {
        let config = SecurityConfig::new().with_encryption_disabled();
        let security_manager = SecurityManager::new_with_persistence(config, db_ops.clone()).unwrap();
        
        // Verify keys were loaded from database
        let keys = security_manager.list_public_keys().unwrap();
        assert_eq!(keys.len(), 2);
        
        // Verify we can find specific keys
        let key1_id = response1.public_key_id.unwrap();
        let key2_id = response2.public_key_id.unwrap();
        
        let retrieved_key1 = security_manager.get_public_key(&key1_id).unwrap();
        let retrieved_key2 = security_manager.get_public_key(&key2_id).unwrap();
        
        assert!(retrieved_key1.is_some());
        assert!(retrieved_key2.is_some());
        
        assert_eq!(retrieved_key1.unwrap().owner_id, "user1");
        assert_eq!(retrieved_key2.unwrap().owner_id, "user2");
    }
}

#[tokio::test]
async fn test_message_verification_with_persisted_keys() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new().with_signatures(true);
    
    // Register a key and sign a message
    let signed_message = {
        let security_manager = SecurityManager::new_with_persistence(config.clone(), db_ops.clone()).unwrap();
        
        // Generate a key pair for testing
        use datafold::security::{Ed25519KeyPair, ClientSecurity};
        let keypair = ClientSecurity::generate_client_keypair().unwrap();
        
        let registration_request = ClientSecurity::create_registration_request(
            &keypair,
            "test_user".to_string(),
            vec!["read".to_string()],
        );
        
        let response = security_manager.register_public_key(registration_request).unwrap();
        let public_key_id = response.public_key_id.unwrap();
        
        // Create a signer and sign a message
        let signer = ClientSecurity::create_signer(keypair, public_key_id);
        let payload = serde_json::json!({"action": "test_action", "data": "test_data"});
        
        ClientSecurity::sign_message(&signer, payload).unwrap()
    };
    
    // Simulate restart and verify the message
    {
        let security_manager = SecurityManager::new_with_persistence(config, db_ops).unwrap();
        
        // Verify the signed message using persisted key
        let result = security_manager.verify_message(&signed_message).unwrap();
        
        assert!(result.is_valid);
        assert!(result.timestamp_valid);
        assert!(result.public_key_info.is_some());
        assert_eq!(result.public_key_info.unwrap().owner_id, "test_user");
    }
}

#[tokio::test]
async fn test_concurrent_key_operations() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new();
    let security_manager = Arc::new(SecurityManager::new_with_persistence(config, db_ops).unwrap());
    
    // Spawn multiple tasks that register keys concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let manager = security_manager.clone();
        let handle = tokio::spawn(async move {
            let key_request = KeyRegistrationRequest {
                public_key: format!("pubkey_{}_base64", i),
                owner_id: format!("user_{}", i),
                permissions: vec!["read".to_string()],
                metadata: std::collections::HashMap::new(),
                expires_at: None,
            };
            
            manager.register_public_key(key_request).unwrap()
        });
        handles.push(handle);
    }
    
    // Wait for all registrations to complete
    let mut responses = vec![];
    for handle in handles {
        let response = handle.await.unwrap();
        assert!(response.success);
        responses.push(response);
    }
    
    // Verify all keys were registered
    let keys = security_manager.list_public_keys().unwrap();
    assert_eq!(keys.len(), 10);
    
    // Verify each key can be retrieved
    for response in responses {
        let key_id = response.public_key_id.unwrap();
        let retrieved = security_manager.get_public_key(&key_id).unwrap();
        assert!(retrieved.is_some());
    }
}

#[tokio::test]
async fn test_database_failure_scenarios() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new();
    
    // Test graceful handling when database is unavailable during startup
    {
        // Close the database to simulate failure
        drop(db_ops);
        
        // Create SecurityManager without persistence (fallback mode)
        let security_manager = SecurityManager::new(config.clone()).unwrap();
        
        // Should still work in memory-only mode
        let key_request = KeyRegistrationRequest {
            public_key: "test_pubkey_base64".to_string(),
            owner_id: "test_user".to_string(),
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
            expires_at: None,
        };
        
        let response = security_manager.register_public_key(key_request).unwrap();
        assert!(response.success);
        
        // Key should be in memory
        let keys = security_manager.list_public_keys().unwrap();
        assert_eq!(keys.len(), 1);
    }
}

#[tokio::test]
async fn test_migration_integration() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new();
    
    // Phase 1: Start with non-persistent security manager (simulating old deployment)
    let keys_to_migrate = {
        let security_manager = SecurityManager::new(config.clone()).unwrap();
        
        // Register keys in memory only
        let mut registered_keys = vec![];
        for i in 0..5 {
            let key_request = KeyRegistrationRequest {
                public_key: format!("pubkey_{}_base64", i),
                owner_id: format!("user_{}", i),
                permissions: vec!["read".to_string()],
                metadata: std::collections::HashMap::new(),
                expires_at: None,
            };
            
            let response = security_manager.register_public_key(key_request).unwrap();
            registered_keys.push(response.public_key_id.unwrap());
        }
        
        // Verify keys are in memory
        let keys = security_manager.list_public_keys().unwrap();
        assert_eq!(keys.len(), 5);
        
        registered_keys
    };
    
    // Phase 2: Upgrade to persistent storage (simulating upgrade)
    {
        let security_manager = SecurityManager::new_with_persistence(config, db_ops).unwrap();
        
        // In real scenario, migration would happen during startup
        // For testing, we verify that new keys can be registered and persisted
        let new_key_request = KeyRegistrationRequest {
            public_key: "new_pubkey_base64".to_string(),
            owner_id: "new_user".to_string(),
            permissions: vec!["write".to_string()],
            metadata: std::collections::HashMap::new(),
            expires_at: None,
        };
        
        let response = security_manager.register_public_key(new_key_request).unwrap();
        assert!(response.success);
        
        // Verify the new key is accessible
        let keys = security_manager.list_public_keys().unwrap();
        assert_eq!(keys.len(), 1); // Only the new key, as migration is separate
    }
}

#[tokio::test]
async fn test_performance_impact() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new();
    
    // Test performance with persistence vs without
    let start = std::time::Instant::now();
    
    // Register many keys with persistence
    {
        let security_manager = SecurityManager::new_with_persistence(config.clone(), db_ops).unwrap();
        
        for i in 0..100 {
            let key_request = KeyRegistrationRequest {
                public_key: format!("pubkey_{}_base64", i),
                owner_id: format!("user_{}", i),
                permissions: vec!["read".to_string()],
                metadata: std::collections::HashMap::new(),
                expires_at: None,
            };
            
            security_manager.register_public_key(key_request).unwrap();
        }
    }
    
    let with_persistence_duration = start.elapsed();
    
    // Test memory-only performance
    let start = std::time::Instant::now();
    
    {
        let security_manager = SecurityManager::new(config).unwrap();
        
        for i in 0..100 {
            let key_request = KeyRegistrationRequest {
                public_key: format!("pubkey_{}_base64", i),
                owner_id: format!("user_{}", i),
                permissions: vec!["read".to_string()],
                metadata: std::collections::HashMap::new(),
                expires_at: None,
            };
            
            security_manager.register_public_key(key_request).unwrap();
        }
    }
    
    let memory_only_duration = start.elapsed();
    
    // Persistence should not add more than 5x overhead
    assert!(with_persistence_duration < memory_only_duration * 5);
    
    println!("Performance test results:");
    println!("  With persistence: {:?}", with_persistence_duration);
    println!("  Memory only: {:?}", memory_only_duration);
    println!("  Overhead ratio: {:.2}x", 
             with_persistence_duration.as_nanos() as f64 / memory_only_duration.as_nanos() as f64);
}
```

### 2. Add HTTP API integration tests
**Location**: `tests/integration/security_api_persistence.rs`

```rust
//! Integration tests for security API endpoints with persistence

use actix_web::{test, web, App};
use datafold::datafold_node::security_routes;
use datafold::security::{SecurityManager, SecurityConfig, KeyRegistrationRequest};
use std::sync::Arc;
use tempfile::TempDir;

#[actix_web::test]
async fn test_key_registration_api_with_persistence() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new();
    let security_manager = Arc::new(SecurityManager::new_with_persistence(config, db_ops).unwrap());
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState { security_manager }))
            .route("/security/keys/register", web::post().to(security_routes::register_public_key))
            .route("/security/keys", web::get().to(security_routes::list_public_keys))
            .route("/security/keys/{id}", web::get().to(security_routes::get_public_key))
    ).await;
    
    // Register a key via API
    let registration_request = KeyRegistrationRequest {
        public_key: "test_pubkey_base64".to_string(),
        owner_id: "api_test_user".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        metadata: std::collections::HashMap::new(),
        expires_at: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/security/keys/register")
        .set_json(&registration_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // List keys via API
    let req = test::TestRequest::get()
        .uri("/security/keys")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["keys"].as_array().unwrap().len(), 1);
}
```

### 3. Add stress tests
**Location**: `tests/stress/key_persistence_stress.rs`

```rust
//! Stress tests for public key persistence under high load

use datafold::security::{SecurityManager, SecurityConfig, KeyRegistrationRequest};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_high_volume_key_registration() {
    let (_temp_dir, db_ops) = create_temp_database();
    let config = SecurityConfig::new();
    let security_manager = Arc::new(SecurityManager::new_with_persistence(config, db_ops).unwrap());
    
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    
    // Spawn 100 concurrent tasks registering 10 keys each
    let mut handles = vec![];
    
    for task_id in 0..100 {
        let manager = security_manager.clone();
        let success_counter = success_count.clone();
        let error_counter = error_count.clone();
        
        let handle = tokio::spawn(async move {
            for key_id in 0..10 {
                let key_request = KeyRegistrationRequest {
                    public_key: format!("pubkey_{}_{}_base64", task_id, key_id),
                    owner_id: format!("user_{}_{}", task_id, key_id),
                    permissions: vec!["read".to_string()],
                    metadata: std::collections::HashMap::new(),
                    expires_at: None,
                };
                
                match manager.register_public_key(key_request) {
                    Ok(_) => success_counter.fetch_add(1, Ordering::Relaxed),
                    Err(_) => error_counter.fetch_add(1, Ordering::Relaxed),
                };
                
                // Small delay to avoid overwhelming the system
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    let final_success = success_count.load(Ordering::Relaxed);
    let final_errors = error_count.load(Ordering::Relaxed);
    
    println!("Stress test results:");
    println!("  Successful registrations: {}", final_success);
    println!("  Failed registrations: {}", final_errors);
    println!("  Success rate: {:.2}%", 
             final_success as f64 / (final_success + final_errors) as f64 * 100.0);
    
    // Expect at least 95% success rate
    assert!(final_success >= 950);
    
    // Verify keys are actually in the database
    let keys = security_manager.list_public_keys().unwrap();
    assert_eq!(keys.len(), final_success);
}
```

## Verification

### Acceptance Criteria
- [ ] Keys persist correctly across simulated node restarts
- [ ] Message verification works with persisted keys
- [ ] Concurrent operations are thread-safe
- [ ] Database failures are handled gracefully
- [ ] Migration scenarios work correctly
- [ ] Performance impact is acceptable (< 5x overhead)
- [ ] API endpoints work with persistence
- [ ] Stress tests pass under high load

### Test Plan
1. **Persistence Tests**: Verify restart scenarios
2. **Security Tests**: Validate message verification with persisted keys
3. **Concurrency Tests**: Test thread safety
4. **Failure Tests**: Verify graceful degradation
5. **Migration Tests**: Test upgrade scenarios
6. **Performance Tests**: Measure impact of persistence
7. **API Tests**: Verify HTTP endpoints work correctly
8. **Stress Tests**: High-load scenarios

## Files Modified

- `tests/integration/public_key_persistence.rs` (new)
- `tests/integration/security_api_persistence.rs` (new)
- `tests/stress/key_persistence_stress.rs` (new)
- `Cargo.toml` (modified - add test dependencies if needed)