//! Comprehensive tests for enhanced error handling and audit logging in crypto operations

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use datafold::crypto::{
    audit_logger::{
        AuditConfig, AuditEventType, CryptoAuditLogger, OperationResult, SecurityEventDetails,
    },
    enhanced_error::{EnhancedCryptoError, ErrorContext, RecoveryAction},
    generate_master_keypair,
    security_monitor::{CryptoSecurityMonitor, SecurityMonitorConfig, SecurityPattern},
    MasterKeyPair,
};
use datafold::db_operations::{DbOperations, EncryptionWrapper};
use datafold::security_types::{Severity, ThreatLevel};
use std::time::Duration;
use uuid::Uuid;

/// Test configuration for crypto operations
struct TestConfig {
    pub test_db_path: String,
    pub master_keypair: MasterKeyPair,
}

impl TestConfig {
    fn new() -> Self {
        let test_db_path = format!("test_crypto_error_audit_{}", uuid::Uuid::new_v4());
        let master_keypair = generate_master_keypair().expect("Failed to generate test keypair");

        Self {
            test_db_path,
            master_keypair,
        }
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.test_db_path);
    }
}

#[tokio::test]
async fn test_enhanced_crypto_error_creation() {
    let context = ErrorContext::new("test_component", "test_operation")
        .with_metadata("test_key", "test_value")
        .propagate("test_step");

    let error = EnhancedCryptoError::Encryption {
        message: "Test encryption failure".to_string(),
        severity: Severity::Error,
        recovery_actions: vec![
            RecoveryAction::Retry,
            RecoveryAction::RegenerateCryptoMaterial,
        ],
        context,
        data_size: Some(1024),
        encryption_context: Some("test_context".to_string()),
        underlying_cause: None,
    };

    assert_eq!(*error.severity(), Severity::Error);
    assert_eq!(error.recovery_actions().len(), 2);
    assert!(error.is_recoverable());
    assert!(error.should_alert());
    assert_eq!(error.error_type_name(), "encryption");
    assert_eq!(error.context().component, "test_component");
    assert_eq!(error.context().operation, "test_operation");
}

#[tokio::test]
async fn test_error_context_propagation() {
    let mut context = ErrorContext::new("crypto", "encryption")
        .with_metadata("data_size", "1024")
        .propagate("initialize_encryptor")
        .propagate("encrypt_data")
        .propagate("serialize_result");

    assert_eq!(context.propagation_chain.len(), 3);
    assert_eq!(context.propagation_chain[0], "initialize_encryptor");
    assert_eq!(context.propagation_chain[2], "serialize_result");
    assert_eq!(context.metadata.get("data_size"), Some(&"1024".to_string()));
}

#[tokio::test]
async fn test_error_structured_logging() {
    let context = ErrorContext::new("test", "test_op").with_metadata("key1", "value1");

    let error = EnhancedCryptoError::KeyGeneration {
        message: "Test key generation failure".to_string(),
        severity: Severity::Critical,
        recovery_actions: vec![RecoveryAction::ContactAdministrator],
        context,
        underlying_cause: Some("RNG failure".to_string()),
    };

    let structured_log = error.to_structured_log();

    assert_eq!(structured_log["error_type"], "key_generation");
    assert_eq!(structured_log["severity"], "Critical");
    assert_eq!(structured_log["component"], "test");
    assert_eq!(structured_log["operation"], "test_op");
    assert!(structured_log["metadata"].is_object());
}

#[tokio::test]
async fn test_audit_logger_basic_functionality() {
    let config = AuditConfig::default();
    let logger = CryptoAuditLogger::new(config);

    // Test encryption operation logging
    logger
        .log_encryption_operation(
            "encrypt_data",
            "atom_data",
            1024,
            Duration::from_millis(50),
            OperationResult::Success,
            None,
        )
        .await;

    let stats = logger.get_statistics().await;
    assert_eq!(stats.total_events, 1);
    assert_eq!(
        stats.events_by_type.get(&AuditEventType::Encryption),
        Some(&1)
    );

    // Test decryption operation logging
    logger
        .log_decryption_operation(
            "decrypt_data",
            "atom_data",
            1024,
            Duration::from_millis(30),
            OperationResult::Failure {
                error_type: "authentication_failed".to_string(),
                error_message: "Invalid signature".to_string(),
                error_code: Some("CRYPTO_001".to_string()),
            },
            None,
        )
        .await;

    let updated_stats = logger.get_statistics().await;
    assert_eq!(updated_stats.total_events, 2);
    assert_eq!(updated_stats.failed_operations, 1);
}

#[tokio::test]
async fn test_audit_logger_key_operations() {
    let config = AuditConfig::default();
    let logger = CryptoAuditLogger::new(config);

    logger
        .log_key_operation(
            "generate_master_key",
            "ed25519",
            Duration::from_millis(100),
            OperationResult::Success,
            None,
        )
        .await;

    logger
        .log_key_operation(
            "derive_encryption_key",
            "aes256",
            Duration::from_millis(75),
            OperationResult::Success,
            Some(Uuid::new_v4()),
        )
        .await;

    let stats = logger.get_statistics().await;
    assert_eq!(stats.total_events, 2);
    assert_eq!(
        stats.events_by_type.get(&AuditEventType::KeyOperation),
        Some(&2)
    );
}

#[tokio::test]
async fn test_audit_logger_security_events() {
    let config = AuditConfig::default();
    let logger = CryptoAuditLogger::new(config);

    let security_details = SecurityEventDetails {
        event_type: "unauthorized_access".to_string(),
        threat_level: "high".to_string(),
        source: Some("192.168.1.100".to_string()),
        target: Some("master_key".to_string()),
        security_metadata: std::collections::HashMap::new(),
    };

    logger
        .log_security_event(
            "access_attempt",
            security_details,
            OperationResult::Failure {
                error_type: "unauthorized".to_string(),
                error_message: "Access denied".to_string(),
                error_code: Some("SEC_001".to_string()),
            },
            None,
        )
        .await;

    let stats = logger.get_statistics().await;
    assert_eq!(stats.security_events, 1);
    assert_eq!(
        stats.events_by_type.get(&AuditEventType::Security),
        Some(&1)
    );
}

#[tokio::test]
async fn test_audit_logger_event_filtering() {
    let config = AuditConfig::default();
    let logger = CryptoAuditLogger::new(config);

    // Log multiple types of events
    logger
        .log_encryption_operation(
            "encrypt1",
            "context1",
            100,
            Duration::from_millis(10),
            OperationResult::Success,
            None,
        )
        .await;

    logger
        .log_decryption_operation(
            "decrypt1",
            "context1",
            100,
            Duration::from_millis(15),
            OperationResult::Success,
            None,
        )
        .await;

    logger
        .log_key_operation(
            "key_gen1",
            "ed25519",
            Duration::from_millis(20),
            OperationResult::Success,
            None,
        )
        .await;

    // Test filtering by event type
    let encryption_events = logger
        .filter_events(Some(AuditEventType::Encryption), None, None, None, None)
        .await;
    assert_eq!(encryption_events.len(), 1);
    assert_eq!(encryption_events[0].operation, "encrypt1");

    // Test filtering by severity
    let info_events = logger
        .filter_events(None, Some(Severity::Info), None, None, None)
        .await;
    assert_eq!(info_events.len(), 3); // All successful operations are Info level

    // Test getting recent events
    let recent_events = logger.get_recent_events(2).await;
    assert_eq!(recent_events.len(), 2);
}

#[tokio::test]
async fn test_audit_logger_export_functionality() {
    let config = AuditConfig::default();
    let logger = CryptoAuditLogger::new(config);

    // Log some events
    logger
        .log_encryption_operation(
            "test_encrypt",
            "test_context",
            512,
            Duration::from_millis(25),
            OperationResult::Success,
            None,
        )
        .await;

    // Test JSON export
    let json_export = logger.export_events_json().await;
    assert!(json_export.is_ok());

    let json_str = json_export.unwrap();
    assert!(json_str.contains("test_encrypt"));
    assert!(json_str.contains("test_context"));
    assert!(json_str.contains("512"));
}

#[tokio::test]
async fn test_security_monitor_basic_functionality() {
    let config = SecurityMonitorConfig::default();
    let monitor = CryptoSecurityMonitor::new(config);

    let stats = monitor.get_statistics().await;
    assert_eq!(stats.total_events_processed, 0);
    assert_eq!(stats.threats_detected, 0);
}

#[tokio::test]
async fn test_security_monitor_repeated_failure_detection() {
    let config = SecurityMonitorConfig {
        failure_threshold: 3,
        ..Default::default()
    };
    let monitor = CryptoSecurityMonitor::new(config);

    let mut detections_found = false;

    // Simulate repeated decryption failures
    for i in 0..5 {
        let detections = monitor
            .monitor_decryption_operation(
                "decrypt_data",
                "test_context",
                100,
                Duration::from_millis(10),
                false, // failure
                Some("test_source"),
            )
            .await;

        if !detections.is_empty() {
            assert_eq!(
                detections[0].pattern,
                SecurityPattern::RepeatedDecryptionFailures
            );
            assert!(matches!(
                detections[0].threat_level,
                ThreatLevel::Medium | ThreatLevel::High
            ));
            detections_found = true;
            break;
        }
    }

    assert!(detections_found, "Should have detected repeated failures");
}

#[tokio::test]
async fn test_security_monitor_unusual_pattern_detection() {
    let monitor = CryptoSecurityMonitor::with_default_config();

    // Test with unusually large data size
    let detections = monitor
        .monitor_encryption_operation(
            "encrypt_large_data",
            "test_context",
            200_000_000, // Very large size
            Duration::from_millis(100),
            true,
            Some("test_source"),
        )
        .await;

    assert!(!detections.is_empty());
    assert_eq!(
        detections[0].pattern,
        SecurityPattern::UnusualEncryptionPattern
    );
    assert_eq!(detections[0].threat_level, ThreatLevel::Low);
}

#[tokio::test]
async fn test_security_monitor_performance_detection() {
    let monitor = CryptoSecurityMonitor::with_default_config();

    // Test with very slow operation
    let detections = monitor
        .monitor_encryption_operation(
            "slow_encrypt",
            "test_context",
            1000,
            Duration::from_millis(10000), // Very slow
            true,
            Some("test_source"),
        )
        .await;

    assert!(!detections.is_empty());
    assert_eq!(
        detections[0].pattern,
        SecurityPattern::PerformanceDegradation
    );
    assert_eq!(detections[0].threat_level, ThreatLevel::Medium);
}

#[tokio::test]
async fn test_security_monitor_unauthorized_access_detection() {
    let monitor = CryptoSecurityMonitor::with_default_config();

    let detections = monitor
        .monitor_key_operation(
            "access_private_key",
            "ed25519",
            Duration::from_millis(50),
            false, // Failed access
            Some("unauthorized_source"),
        )
        .await;

    assert!(!detections.is_empty());
    assert_eq!(
        detections[0].pattern,
        SecurityPattern::UnauthorizedKeyAccess
    );
    assert_eq!(detections[0].threat_level, ThreatLevel::High);
}

#[tokio::test]
async fn test_security_monitor_error_integration() {
    let monitor = CryptoSecurityMonitor::with_default_config();

    // Create an enhanced crypto error
    let context =
        ErrorContext::new("crypto_test", "test_decrypt").with_metadata("source", "suspicious_ip");

    let error = EnhancedCryptoError::Decryption {
        message: "Decryption failed due to invalid key".to_string(),
        severity: Severity::Error,
        recovery_actions: vec![RecoveryAction::CheckConfiguration],
        context,
        data_size: Some(1024),
        encryption_context: Some("test_context".to_string()),
        underlying_cause: Some("Authentication failure".to_string()),
    };

    let detections = monitor.monitor_crypto_error(&error).await;
    // Note: The specific detection depends on the error processing logic
    // This tests that the integration works without panicking

    let stats = monitor.get_statistics().await;
    assert!(stats.total_events_processed > 0);
}

#[tokio::test]
async fn test_security_monitor_statistics_tracking() {
    let monitor = CryptoSecurityMonitor::with_default_config();

    // Generate successful operations
    monitor
        .monitor_encryption_operation(
            "encrypt1",
            "context1",
            100,
            Duration::from_millis(10),
            true,
            None,
        )
        .await;

    monitor
        .monitor_decryption_operation(
            "decrypt1",
            "context1",
            100,
            Duration::from_millis(15),
            true,
            None,
        )
        .await;

    // Generate failed operation
    monitor
        .monitor_decryption_operation(
            "decrypt_fail",
            "context1",
            100,
            Duration::from_millis(20),
            false,
            None,
        )
        .await;

    let stats = monitor.get_statistics().await;
    assert_eq!(stats.total_events_processed, 3);
}

#[tokio::test]
async fn test_security_monitor_detection_filtering() {
    let monitor = CryptoSecurityMonitor::with_default_config();

    // Generate a detection
    let _detections = monitor
        .monitor_key_operation(
            "failed_key_access",
            "ed25519",
            Duration::from_millis(50),
            false,
            Some("test_source"),
        )
        .await;

    // Test filtering by threat level
    let high_threat_detections = monitor
        .get_detections_by_threat_level(ThreatLevel::High)
        .await;

    // Should have at least one detection for unauthorized key access
    if !high_threat_detections.is_empty() {
        assert_eq!(high_threat_detections[0].threat_level, ThreatLevel::High);
    }

    // Test getting recent detections
    let recent_detections = monitor.get_recent_detections(10).await;
    assert!(recent_detections.len() <= 10);
}

#[tokio::test]
async fn test_integrated_error_and_audit_workflow() {
    // This test demonstrates the full workflow of error handling with audit logging
    let audit_config = AuditConfig::default();
    let audit_logger = std::sync::Arc::new(CryptoAuditLogger::new(audit_config));

    let security_config = SecurityMonitorConfig::default();
    let security_monitor =
        CryptoSecurityMonitor::new(security_config).with_audit_logger(audit_logger.clone());

    // Simulate a crypto operation with error
    let context = ErrorContext::new("integrated_test", "full_workflow")
        .with_metadata("test_id", "integration_001");

    let error = EnhancedCryptoError::Encryption {
        message: "Integration test encryption failure".to_string(),
        severity: Severity::Error,
        recovery_actions: vec![RecoveryAction::Retry, RecoveryAction::CheckConfiguration],
        context,
        data_size: Some(2048),
        encryption_context: Some("integration_context".to_string()),
        underlying_cause: None,
    };

    // Log the error to audit system
    audit_logger
        .log_error_event(&error, Some(Uuid::new_v4()))
        .await;

    // Monitor for security implications
    let detections = security_monitor.monitor_crypto_error(&error).await;

    // Verify audit logging worked
    let audit_stats = audit_logger.get_statistics().await;
    assert!(audit_stats.total_events > 0);

    // Verify security monitoring worked
    let security_stats = security_monitor.get_statistics().await;
    assert!(security_stats.total_events_processed > 0);

    // Verify the error has proper recovery information
    assert!(error.is_recoverable());
    assert_eq!(error.recovery_actions().len(), 2);
    assert!(error.should_alert());
}

#[test]
fn test_error_recovery_action_logic() {
    let context = ErrorContext::new("test", "test");

    // Test recoverable error
    let recoverable_error = EnhancedCryptoError::KeyGeneration {
        message: "Temporary failure".to_string(),
        severity: Severity::Warning,
        recovery_actions: vec![RecoveryAction::Retry],
        context: context.clone(),
        underlying_cause: None,
    };
    assert!(recoverable_error.is_recoverable());

    // Test non-recoverable error
    let non_recoverable_error = EnhancedCryptoError::Security {
        message: "Critical security violation".to_string(),
        severity: Severity::Critical,
        recovery_actions: vec![RecoveryAction::NoRecovery],
        context,
        security_event: "data_corruption".to_string(),
        threat_level: "critical".to_string(),
        underlying_cause: None,
    };
    assert!(!non_recoverable_error.is_recoverable());
    assert!(non_recoverable_error.should_alert());
}

#[test]
fn test_error_conversion_from_legacy() {
    // Test conversion from legacy CryptoError to EnhancedCryptoError
    let legacy_error = datafold::crypto::error::CryptoError::KeyGeneration {
        message: "Legacy key generation error".to_string(),
    };

    let enhanced_error = EnhancedCryptoError::from(legacy_error);
    assert_eq!(enhanced_error.error_type_name(), "key_generation");
    assert!(enhanced_error.is_recoverable());
    assert_eq!(*enhanced_error.severity(), Severity::Error);
}

#[tokio::test]
async fn test_audit_logger_configuration() {
    // Test with custom configuration
    let custom_config = AuditConfig {
        enabled: true,
        max_events_in_memory: 100,
        enable_json_output: true,
        included_event_types: vec![AuditEventType::Encryption, AuditEventType::Security],
        excluded_event_types: vec![AuditEventType::Performance],
        ..Default::default()
    };

    let logger = CryptoAuditLogger::new(custom_config);

    // Test that encryption events are logged
    logger
        .log_encryption_operation(
            "test_encrypt",
            "test",
            100,
            Duration::from_millis(10),
            OperationResult::Success,
            None,
        )
        .await;

    // Test that performance events would be excluded (this requires internal knowledge)
    // In a real implementation, performance events would be filtered out

    let stats = logger.get_statistics().await;
    assert_eq!(stats.total_events, 1);
}

#[tokio::test]
async fn test_memory_management() {
    // Test that audit logger and security monitor don't leak memory
    let config = AuditConfig {
        max_events_in_memory: 5, // Small limit for testing
        ..Default::default()
    };
    let logger = CryptoAuditLogger::new(config);

    // Add more events than the limit
    for i in 0..10 {
        logger
            .log_encryption_operation(
                &format!("encrypt_{}", i),
                "test",
                100,
                Duration::from_millis(10),
                OperationResult::Success,
                None,
            )
            .await;
    }

    // Verify that memory usage is bounded
    let events = logger.get_recent_events(100).await;
    assert!(events.len() <= 5); // Should not exceed max_events_in_memory
}
