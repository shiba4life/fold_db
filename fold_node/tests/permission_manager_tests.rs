use fold_node::permissions::permission_manager::PermissionManager;
use fold_node::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use std::collections::HashMap;

#[test]
fn read_permission_trust_distance() {
    let manager = PermissionManager::new();
    // allow reads within distance 2
    let policy = PermissionsPolicy {
        read_policy: TrustDistance::Distance(2),
        write_policy: TrustDistance::Distance(0),
        explicit_write_policy: None,
        explicit_read_policy: None,
    };

    assert!(manager.has_read_permission("any", &policy, 1));
    assert!(!manager.has_read_permission("any", &policy, 3));
}

#[test]
fn read_permission_explicit_fallback() {
    let manager = PermissionManager::new();
    let mut counts = HashMap::new();
    counts.insert("allowed".to_string(), 1);
    let policy = PermissionsPolicy {
        read_policy: TrustDistance::Distance(1),
        write_policy: TrustDistance::Distance(0),
        explicit_write_policy: None,
        explicit_read_policy: Some(ExplicitCounts { counts_by_pub_key: counts }),
    };

    // exceeds trust distance but in explicit list
    assert!(manager.has_read_permission("allowed", &policy, 5));
    // not in explicit list
    assert!(!manager.has_read_permission("denied", &policy, 5));
}

#[test]
fn write_permission_trust_distance() {
    let manager = PermissionManager::new();
    let policy = PermissionsPolicy {
        read_policy: TrustDistance::Distance(0),
        write_policy: TrustDistance::Distance(2),
        explicit_write_policy: None,
        explicit_read_policy: None,
    };

    assert!(manager.has_write_permission("any", &policy, 2));
    assert!(!manager.has_write_permission("any", &policy, 3));
}

#[test]
fn write_permission_explicit_fallback() {
    let manager = PermissionManager::new();
    let mut counts = HashMap::new();
    counts.insert("writer".to_string(), 1);
    let policy = PermissionsPolicy {
        read_policy: TrustDistance::Distance(0),
        write_policy: TrustDistance::Distance(1),
        explicit_write_policy: Some(ExplicitCounts { counts_by_pub_key: counts }),
        explicit_read_policy: None,
    };

    assert!(manager.has_write_permission("writer", &policy, 5));
    assert!(!manager.has_write_permission("other", &policy, 5));
}
