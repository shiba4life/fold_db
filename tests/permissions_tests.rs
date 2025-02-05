use fold_db::permissions::permission_manager::PermissionManager;
use fold_db::permissions::types::policy::{PermissionsPolicy, ExplicitCounts};
use std::collections::HashMap;

fn setup_permission_manager() -> PermissionManager {
    PermissionManager {}
}

#[test]
fn test_trust_based_permissions() {
    let manager = setup_permission_manager();
    let pub_key = "test_key";
    
    // Test read permissions with different trust levels
    let policy = PermissionsPolicy::new(2, 3); // read_policy: 2, write_policy: 3
    
    // Read permissions
    assert!(manager.has_read_permission(pub_key, &policy, 1)); // Trust distance less than policy
    assert!(manager.has_read_permission(pub_key, &policy, 2)); // Trust distance equal to policy
    assert!(!manager.has_read_permission(pub_key, &policy, 3)); // Trust distance greater than policy

    // Write permissions
    assert!(manager.has_write_permission(pub_key, &policy, 2)); // Trust distance less than policy
    assert!(manager.has_write_permission(pub_key, &policy, 3)); // Trust distance equal to policy
    assert!(!manager.has_write_permission(pub_key, &policy, 4)); // Trust distance greater than policy
}

#[test]
fn test_explicit_permissions() {
    let manager = setup_permission_manager();
    let pub_key = "explicit_key";
    let other_key = "other_key";

    // Setup explicit permissions
    let mut read_counts = HashMap::new();
    read_counts.insert(pub_key.to_string(), 1);
    
    let mut write_counts = HashMap::new();
    write_counts.insert(pub_key.to_string(), 1);

    let policy = PermissionsPolicy {
        read_policy: 0,
        write_policy: 0,
        explicit_read_policy: Some(ExplicitCounts { counts_by_pub_key: read_counts }),
        explicit_write_policy: Some(ExplicitCounts { counts_by_pub_key: write_counts }),
    };

    // Test explicit permissions
    assert!(manager.has_read_permission(pub_key, &policy, 1)); // Explicitly allowed
    assert!(!manager.has_read_permission(other_key, &policy, 1)); // Not explicitly allowed

    assert!(manager.has_write_permission(pub_key, &policy, 1)); // Explicitly allowed
    assert!(!manager.has_write_permission(other_key, &policy, 1)); // Not explicitly allowed
}

#[test]
fn test_combined_permissions() {
    let manager = setup_permission_manager();
    let pub_key = "explicit_key";
    let trust_key = "trust_key";

    // Setup combined permissions
    let mut read_counts = HashMap::new();
    read_counts.insert(pub_key.to_string(), 1);
    
    let policy = PermissionsPolicy {
        read_policy: 2, // Allow trust-based access
        write_policy: 0, // Require explicit access
        explicit_read_policy: Some(ExplicitCounts { counts_by_pub_key: read_counts.clone() }),
        explicit_write_policy: Some(ExplicitCounts { counts_by_pub_key: read_counts }),
    };

    // Test trust-based read access
    assert!(manager.has_read_permission(trust_key, &policy, 1)); // Within trust distance
    assert!(manager.has_read_permission(trust_key, &policy, 2)); // At trust distance
    assert!(!manager.has_read_permission(trust_key, &policy, 3)); // Beyond trust distance

    // Test explicit access
    assert!(manager.has_read_permission(pub_key, &policy, 3)); // Explicit access overrides trust distance
    assert!(manager.has_write_permission(pub_key, &policy, 1)); // Explicit write access
    assert!(!manager.has_write_permission(trust_key, &policy, 1)); // No explicit write access
}

#[test]
fn test_default_permissions() {
    let manager = setup_permission_manager();
    let pub_key = "test_key";
    
    let policy = PermissionsPolicy::default();
    
    // Default policy should deny all access
    assert!(!manager.has_read_permission(pub_key, &policy, 1));
    assert!(!manager.has_write_permission(pub_key, &policy, 1));
}
