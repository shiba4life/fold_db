//! PBI-12 Key Rotation Backend Integration Test
//!
//! This test validates the complete key rotation backend implementation including:
//! - Key rotation protocol validation
//! - Database operations functionality  
//! - Audit logging functionality

use datafold::{
    crypto::{
        audit_logger::CryptoAuditLogger, generate_master_keypair, KeyRotationRequest,
        KeyRotationValidator, RotationReason,
    },
    db_operations::{
        core::DbOperations,
        key_rotation_operations::{KeyAssociation, KeyRotationRecord},
    },
    security_types::RotationStatus,
};
use std::collections::HashMap;
use tempfile::tempdir;
use uuid::Uuid;

/// Create test database operations
fn create_test_db_ops() -> DbOperations {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    DbOperations::new(db).unwrap()
}

#[tokio::test]
async fn test_key_rotation_protocol_validation() {
    println!("Testing key rotation protocol validation...");

    // Test different rotation reasons and their audit severity
    let reasons = vec![
        (RotationReason::Compromise, "Critical"),
        (RotationReason::Policy, "Warning"),
        (RotationReason::Scheduled, "Info"),
        (RotationReason::UserInitiated, "Info"),
        (RotationReason::Migration, "Warning"),
        (RotationReason::Maintenance, "Info"),
    ];

    for (reason, expected_severity) in reasons {
        let severity = reason.audit_severity();
        println!("Reason: {:?}, Severity: {:?}", reason, severity);
        assert_eq!(format!("{:?}", severity), expected_severity);
    }

    // Test timestamp validation
    let old_keypair = generate_master_keypair().unwrap();
    let new_keypair = generate_master_keypair().unwrap();

    let rotation_request = KeyRotationRequest::new(
        &old_keypair.private_key(),
        new_keypair.public_key().clone(),
        RotationReason::Scheduled,
        None,
        HashMap::new(),
    )
    .unwrap();

    assert!(rotation_request.is_timestamp_valid());
    assert!(rotation_request.is_within_lifetime());
    assert!(rotation_request.verify_signature().is_ok());
    assert!(rotation_request.validate_new_key().is_ok());

    println!("Key rotation protocol validation tests passed!");
}

#[tokio::test]
async fn test_key_rotation_validation_comprehensive() {
    println!("Testing comprehensive key rotation validation...");

    let old_keypair = generate_master_keypair().unwrap();
    let new_keypair = generate_master_keypair().unwrap();

    // Create validator
    let audit_logger = CryptoAuditLogger::with_default_config();
    let validator = KeyRotationValidator::new(Some(audit_logger));

    // Test valid request
    let valid_request = KeyRotationRequest::new(
        &old_keypair.private_key(),
        new_keypair.public_key().clone(),
        RotationReason::Policy,
        None,
        HashMap::new(),
    )
    .unwrap();

    let result = validator.validate_request(&valid_request).await;
    assert!(result.is_valid);
    assert!(result.errors.is_empty());

    // Test invalid request (try to create with same key - should fail at construction)
    let invalid_result = KeyRotationRequest::new(
        &old_keypair.private_key(),
        old_keypair.public_key().clone(), // Same key!
        RotationReason::Scheduled,
        None,
        HashMap::new(),
    );
    assert!(
        invalid_result.is_err(),
        "Should fail to create request with same old/new key"
    );

    println!("Comprehensive validation tests passed!");
}

#[tokio::test]
async fn test_database_operations() {
    println!("Testing database operations...");

    let db_ops = create_test_db_ops();

    // Test key association operations
    let old_keypair = generate_master_keypair().unwrap();
    let _new_keypair = generate_master_keypair().unwrap();

    // Create test association
    let association = KeyAssociation {
        association_id: "test_association".to_string(),
        public_key: hex::encode(old_keypair.public_key_bytes()),
        association_type: "test".to_string(),
        data_reference: "test_data".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: "active".to_string(),
    };

    // Store association
    db_ops.store_key_association(&association).unwrap();

    // Retrieve association
    let retrieved = db_ops.get_key_association("test_association").unwrap();
    assert_eq!(retrieved.association_id, association.association_id);

    // Get associations by public key
    let associations = db_ops
        .get_key_associations(&hex::encode(old_keypair.public_key_bytes()))
        .unwrap();
    assert!(!associations.is_empty());

    // Delete association
    let deleted = db_ops.delete_key_association("test_association").unwrap();
    assert!(deleted);

    // Verify deletion
    let result = db_ops.get_key_association("test_association");
    assert!(result.is_err());

    println!("Database operations tests passed!");
}

#[tokio::test]
async fn test_rotation_record_operations() {
    println!("Testing rotation record operations...");

    let db_ops = create_test_db_ops();
    let old_keypair = generate_master_keypair().unwrap();
    let new_keypair = generate_master_keypair().unwrap();

    let rotation_request = KeyRotationRequest::new(
        &old_keypair.private_key(),
        new_keypair.public_key().clone(),
        RotationReason::Scheduled,
        Some("test-client".to_string()),
        HashMap::new(),
    )
    .unwrap();

    let test_record = KeyRotationRecord {
        operation_id: Uuid::new_v4(),
        request: rotation_request.clone(),
        old_public_key: hex::encode(old_keypair.public_key_bytes()),
        new_public_key: hex::encode(new_keypair.public_key_bytes()),
        reason: RotationReason::Scheduled,
        status: RotationStatus::Completed,
        started_at: chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
        actor: Some("test-actor".to_string()),
        client_id: Some("test-client".to_string()),
        error_details: None,
        associations_updated: 0,
        metadata: HashMap::new(),
    };

    // Store rotation record
    db_ops.store_rotation_record(&test_record).unwrap();

    // Retrieve rotation record
    let retrieved = db_ops
        .get_rotation_record(&test_record.operation_id)
        .unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().operation_id, test_record.operation_id);

    // Test rotation history
    let history = db_ops
        .get_rotation_history(&hex::encode(old_keypair.public_key_bytes()))
        .unwrap();
    assert!(!history.is_empty());

    // Test rotation statistics
    let stats = db_ops.get_rotation_statistics().unwrap();
    assert!(stats.contains_key("total_rotations"));
    assert!(stats.contains_key("successful_rotations"));
    assert!(stats.contains_key("failed_rotations"));
    assert!(stats.contains_key("success_rate"));

    println!("Rotation record operations tests passed!");
}

#[tokio::test]
async fn test_audit_logging() {
    println!("Testing audit logging functionality...");

    let audit_logger = CryptoAuditLogger::with_default_config();

    // Log a test operation
    audit_logger
        .log_key_operation(
            "test_key_rotation",
            "key_rotation",
            std::time::Duration::from_millis(100),
            datafold::crypto::audit_logger::OperationResult::Success,
            Some(Uuid::new_v4()),
        )
        .await;

    // Log a security event
    let security_details = datafold::crypto::audit_logger::SecurityEventDetails {
        event_type: "test_security_event".to_string(),
        threat_level: "low".to_string(),
        source: Some("127.0.0.1".to_string()),
        target: Some("test_target".to_string()),
        security_metadata: HashMap::new(),
    };

    audit_logger
        .log_security_event(
            "test_event",
            security_details,
            datafold::crypto::audit_logger::OperationResult::Success,
            Some(Uuid::new_v4()),
        )
        .await;

    println!("Audit logging tests passed!");
}

#[test]
fn test_key_rotation_serialization() {
    println!("Testing key rotation data structure serialization...");

    let old_keypair = generate_master_keypair().unwrap();
    let new_keypair = generate_master_keypair().unwrap();

    // Test KeyRotationRequest serialization
    let rotation_request = KeyRotationRequest::new(
        &old_keypair.private_key(),
        new_keypair.public_key().clone(),
        RotationReason::UserInitiated,
        Some("test-client".to_string()),
        HashMap::from([("test_key".to_string(), "test_value".to_string())]),
    )
    .unwrap();

    // Test serialization to JSON
    let serialized = serde_json::to_string(&rotation_request).unwrap();
    assert!(!serialized.is_empty());

    // Test deserialization from JSON
    let deserialized: KeyRotationRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
        deserialized.old_public_key.to_bytes(),
        rotation_request.old_public_key.to_bytes()
    );
    assert_eq!(
        deserialized.new_public_key.to_bytes(),
        rotation_request.new_public_key.to_bytes()
    );
    assert_eq!(deserialized.reason, rotation_request.reason);

    // Test KeyRotationRecord serialization
    let test_record = KeyRotationRecord {
        operation_id: Uuid::new_v4(),
        request: rotation_request.clone(),
        old_public_key: hex::encode(old_keypair.public_key_bytes()),
        new_public_key: hex::encode(new_keypair.public_key_bytes()),
        reason: RotationReason::UserInitiated,
        status: RotationStatus::InProgress,
        started_at: chrono::Utc::now(),
        completed_at: None,
        actor: Some("test-actor".to_string()),
        client_id: Some("test-client".to_string()),
        error_details: None,
        associations_updated: 0,
        metadata: HashMap::new(),
    };

    let record_serialized = serde_json::to_string(&test_record).unwrap();
    assert!(!record_serialized.is_empty());

    let record_deserialized: KeyRotationRecord = serde_json::from_str(&record_serialized).unwrap();
    assert_eq!(record_deserialized.operation_id, test_record.operation_id);
    assert_eq!(record_deserialized.status, test_record.status);

    println!("Key rotation serialization tests passed!");
}

#[test]
fn test_compilation_integration() {
    println!("Testing that all components compile and integrate correctly...");

    // Test that we can create all the necessary components
    let _audit_logger = CryptoAuditLogger::with_default_config();
    let _keypair = generate_master_keypair().unwrap();

    // Test that all enum variants are accessible
    let _reasons = [RotationReason::Scheduled,
        RotationReason::Compromise,
        RotationReason::Policy,
        RotationReason::UserInitiated,
        RotationReason::Migration,
        RotationReason::Maintenance];

    let _statuses = [RotationStatus::InProgress,
        RotationStatus::Completed,
        RotationStatus::Failed,
        RotationStatus::RolledBack];

    println!("Compilation and integration tests passed!");
}
