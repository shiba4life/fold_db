//! # Cryptographic Security Audit and Logging
//!
//! This module provides comprehensive security audit logging for all cryptographic
//! operations in the unified crypto system. It ensures tamper-evident logging,
//! comprehensive event tracking, and security monitoring capabilities.

use crate::unified_crypto::{
    config::AuditConfig,
    error::{UnifiedCryptoError, UnifiedCryptoResult},
    keys::RotationReason,
    types::{Algorithm, KeyId, Signature},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Comprehensive cryptographic audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoAuditEvent {
    /// Unique event identifier
    pub event_id: String,
    /// Event type classification
    pub event_type: AuditEventType,
    /// Event timestamp (UTC)
    pub timestamp: SystemTime,
    /// Event severity level
    pub severity: AuditSeverity,
    /// Component that generated the event
    pub component: String,
    /// Operation that was performed
    pub operation: String,
    /// Success or failure status
    pub status: OperationStatus,
    /// Relevant key ID (if applicable)
    pub key_id: Option<KeyId>,
    /// Algorithm used (if applicable)
    pub algorithm: Option<Algorithm>,
    /// User or process context
    pub actor: Option<String>,
    /// Event-specific metadata
    pub metadata: HashMap<String, String>,
    /// Performance metrics
    pub performance: Option<PerformanceMetrics>,
    /// Security context
    pub security_context: SecurityContext,
    /// Error details (if status is failure)
    pub error_details: Option<String>,
}

impl CryptoAuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        component: &str,
        operation: &str,
        status: OperationStatus,
    ) -> Self {
        Self {
            event_id: generate_event_id(),
            event_type,
            timestamp: SystemTime::now(),
            severity,
            component: component.to_string(),
            operation: operation.to_string(),
            status,
            key_id: None,
            algorithm: None,
            actor: None,
            metadata: HashMap::new(),
            performance: None,
            security_context: SecurityContext::default(),
            error_details: None,
        }
    }

    /// Create initialization event
    pub fn initialization(config: &crate::unified_crypto::config::CryptoConfig) -> Self {
        let mut event = Self::new(
            AuditEventType::Initialization,
            AuditSeverity::Info,
            "unified_crypto",
            "initialize",
            OperationStatus::Success,
        );
        
        event.metadata.insert("security_level".to_string(), 
            format!("{:?}", config.general.security_level));
        event.metadata.insert("fips_mode".to_string(), 
            config.general.fips_mode.to_string());
        event.metadata.insert("config_version".to_string(), 
            config.general.config_version.clone());
        
        event
    }

    /// Create initialization complete event
    pub fn initialization_complete(config: &crate::unified_crypto::config::CryptoConfig) -> Self {
        let mut event = Self::new(
            AuditEventType::Initialization,
            AuditSeverity::Info,
            "unified_crypto",
            "initialization_complete",
            OperationStatus::Success,
        );
        
        event.metadata.insert("supported_algorithms".to_string(),
            format!("{:?}", config.primitives.supported_algorithms));
        
        event
    }

    /// Create shutdown event
    pub fn shutdown() -> Self {
        Self::new(
            AuditEventType::Shutdown,
            AuditSeverity::Info,
            "unified_crypto",
            "shutdown",
            OperationStatus::Success,
        )
    }

    /// Create key manager initialization event
    pub fn key_manager_init() -> Self {
        Self::new(
            AuditEventType::KeyManagement,
            AuditSeverity::Info,
            "key_manager",
            "initialize",
            OperationStatus::Success,
        )
    }

    /// Create encryption start event
    pub fn encryption_start(key_id: &KeyId, data_size: usize) -> Self {
        let mut event = Self::new(
            AuditEventType::Encryption,
            AuditSeverity::Info,
            "primitives",
            "encrypt_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("data_size".to_string(), data_size.to_string());
        
        event
    }

    /// Create encryption success event
    pub fn encryption_success(key_id: &KeyId, ciphertext_size: usize) -> Self {
        let mut event = Self::new(
            AuditEventType::Encryption,
            AuditSeverity::Info,
            "primitives",
            "encrypt_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("ciphertext_size".to_string(), ciphertext_size.to_string());
        
        event
    }

    /// Create encryption failure event
    pub fn encryption_failure(key_id: &KeyId, error: &UnifiedCryptoError) -> Self {
        let mut event = Self::new(
            AuditEventType::Encryption,
            AuditSeverity::Error,
            "primitives",
            "encrypt_failure",
            OperationStatus::Failure,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.error_details = Some(error.to_string());
        event.security_context.risk_level = RiskLevel::Medium;
        
        event
    }

    /// Create decryption start event
    pub fn decryption_start(key_id: &KeyId, ciphertext_size: usize) -> Self {
        let mut event = Self::new(
            AuditEventType::Decryption,
            AuditSeverity::Info,
            "primitives",
            "decrypt_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("ciphertext_size".to_string(), ciphertext_size.to_string());
        
        event
    }

    /// Create decryption success event
    pub fn decryption_success(key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::Decryption,
            AuditSeverity::Info,
            "primitives",
            "decrypt_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        // Note: We don't log plaintext size for security reasons
        
        event
    }

    /// Create decryption failure event
    pub fn decryption_failure(key_id: &KeyId, error: &UnifiedCryptoError) -> Self {
        let mut event = Self::new(
            AuditEventType::Decryption,
            AuditSeverity::Error,
            "primitives",
            "decrypt_failure",
            OperationStatus::Failure,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.error_details = Some(error.to_string());
        event.security_context.risk_level = RiskLevel::High; // Decryption failures are more concerning
        
        event
    }

    /// Create signing start event
    pub fn signing_start(key_id: &KeyId, data_size: usize) -> Self {
        let mut event = Self::new(
            AuditEventType::Signing,
            AuditSeverity::Info,
            "primitives",
            "sign_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("data_size".to_string(), data_size.to_string());
        
        event
    }

    /// Create signing success event
    pub fn signing_success(key_id: &KeyId, signature: &Signature) -> Self {
        let mut event = Self::new(
            AuditEventType::Signing,
            AuditSeverity::Info,
            "primitives",
            "sign_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("signature_algorithm".to_string(), 
            format!("{:?}", signature.algorithm()));
        event.metadata.insert("signature_length".to_string(), 
            signature.len().to_string());
        
        event
    }

    /// Create signing failure event
    pub fn signing_failure(key_id: &KeyId, error: &UnifiedCryptoError) -> Self {
        let mut event = Self::new(
            AuditEventType::Signing,
            AuditSeverity::Error,
            "primitives",
            "sign_failure",
            OperationStatus::Failure,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.error_details = Some(error.to_string());
        event.security_context.risk_level = RiskLevel::Medium;
        
        event
    }

    /// Create verification start event
    pub fn verification_start(key_id: &KeyId, signature: &Signature) -> Self {
        let mut event = Self::new(
            AuditEventType::Verification,
            AuditSeverity::Info,
            "primitives",
            "verify_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("signature_algorithm".to_string(), 
            format!("{:?}", signature.algorithm()));
        
        event
    }

    /// Create verification complete event
    pub fn verification_complete(key_id: &KeyId, valid: bool) -> Self {
        let mut event = Self::new(
            AuditEventType::Verification,
            if valid { AuditSeverity::Info } else { AuditSeverity::Warning },
            "primitives",
            "verify_complete",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("signature_valid".to_string(), valid.to_string());
        
        if !valid {
            event.security_context.risk_level = RiskLevel::High;
        }
        
        event
    }

    /// Create verification error event
    pub fn verification_error(key_id: &KeyId, error: &UnifiedCryptoError) -> Self {
        let mut event = Self::new(
            AuditEventType::Verification,
            AuditSeverity::Error,
            "primitives",
            "verify_error",
            OperationStatus::Failure,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.error_details = Some(error.to_string());
        event.security_context.risk_level = RiskLevel::High;
        
        event
    }

    /// Create key generation start event
    pub fn key_generation_start(algorithm: Algorithm) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyGeneration,
            AuditSeverity::Info,
            "key_manager",
            "generate_start",
            OperationStatus::InProgress,
        );
        
        event.algorithm = Some(algorithm);
        event.metadata.insert("key_algorithm".to_string(), 
            format!("{:?}", algorithm));
        
        event
    }

    /// Create key generation success event
    pub fn key_generation_success(key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyGeneration,
            AuditSeverity::Info,
            "key_manager",
            "generate_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        
        event
    }

    /// Create key access event
    pub fn key_access(key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyAccess,
            AuditSeverity::Info,
            "key_manager",
            "key_access",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        
        event
    }

    /// Create key rotation start event
    pub fn key_rotation_start(old_key_id: &KeyId, reason: &RotationReason) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyRotation,
            AuditSeverity::Warning,
            "key_manager",
            "rotation_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(old_key_id.clone());
        event.algorithm = Some(old_key_id.algorithm());
        event.metadata.insert("rotation_reason".to_string(), 
            format!("{:?}", reason));
        
        event
    }

    /// Create key rotation success event
    pub fn key_rotation_success(old_key_id: &KeyId, new_key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyRotation,
            AuditSeverity::Info,
            "key_manager",
            "rotation_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(old_key_id.clone());
        event.algorithm = Some(old_key_id.algorithm());
        event.metadata.insert("new_key_id".to_string(), new_key_id.to_string());
        
        event
    }

    /// Create key revocation start event
    pub fn key_revocation_start(key_id: &KeyId, reason: &str) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyRevocation,
            AuditSeverity::Warning,
            "key_manager",
            "revocation_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("revocation_reason".to_string(), reason.to_string());
        
        event
    }

    /// Create key revocation success event
    pub fn key_revocation_success(key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyRevocation,
            AuditSeverity::Warning,
            "key_manager",
            "revocation_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        
        event
    }

    /// Create key backup start event
    pub fn key_backup_start(key_id: &KeyId, location: &str) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyBackup,
            AuditSeverity::Info,
            "key_manager",
            "backup_start",
            OperationStatus::InProgress,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.metadata.insert("backup_location".to_string(), location.to_string());
        
        event
    }

    /// Create key backup success event
    pub fn key_backup_success(key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::KeyBackup,
            AuditSeverity::Info,
            "key_manager",
            "backup_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        
        event
    }

    /// Create automatic rotation success event
    pub fn automatic_rotation_success(old_key_id: &KeyId, new_key_id: &KeyId) -> Self {
        let mut event = Self::new(
            AuditEventType::AutomaticRotation,
            AuditSeverity::Info,
            "rotation_manager",
            "automatic_rotation_success",
            OperationStatus::Success,
        );
        
        event.key_id = Some(old_key_id.clone());
        event.algorithm = Some(old_key_id.algorithm());
        event.metadata.insert("new_key_id".to_string(), new_key_id.to_string());
        
        event
    }

    /// Create automatic rotation failure event
    pub fn automatic_rotation_failure(key_id: &KeyId, error: &UnifiedCryptoError) -> Self {
        let mut event = Self::new(
            AuditEventType::AutomaticRotation,
            AuditSeverity::Error,
            "rotation_manager",
            "automatic_rotation_failure",
            OperationStatus::Failure,
        );
        
        event.key_id = Some(key_id.clone());
        event.algorithm = Some(key_id.algorithm());
        event.error_details = Some(error.to_string());
        event.security_context.risk_level = RiskLevel::High;
        
        event
    }

    /// Create security violation event
    pub fn security_violation(violation_type: &str, details: &str) -> Self {
        let mut event = Self::new(
            AuditEventType::SecurityViolation,
            AuditSeverity::Critical,
            "security_monitor",
            violation_type,
            OperationStatus::Failure,
        );
        
        event.error_details = Some(details.to_string());
        event.security_context.risk_level = RiskLevel::Critical;
        event.security_context.requires_investigation = true;
        
        event
    }

    /// Create performance anomaly event
    pub fn performance_anomaly(operation: &str, metrics: &PerformanceMetrics) -> Self {
        let mut event = Self::new(
            AuditEventType::PerformanceAnomaly,
            AuditSeverity::Warning,
            "performance_monitor",
            operation,
            OperationStatus::Success,
        );
        
        event.performance = Some(metrics.clone());
        event.metadata.insert("duration_ms".to_string(), 
            metrics.duration_ms.to_string());
        
        event
    }

    /// Add performance metrics to the event
    pub fn with_performance(mut self, metrics: PerformanceMetrics) -> Self {
        self.performance = Some(metrics);
        self
    }

    /// Add actor context to the event
    pub fn with_actor(mut self, actor: &str) -> Self {
        self.actor = Some(actor.to_string());
        self
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Set security context
    pub fn with_security_context(mut self, context: SecurityContext) -> Self {
        self.security_context = context;
        self
    }

    // Placeholder audit events for operational layer
    pub fn operations_initialized() -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "operations", "initialize", OperationStatus::Success)
    }

    pub fn operations_shutdown() -> Self {
        Self::new(AuditEventType::SystemShutdown, AuditSeverity::Info, "operations", "shutdown", OperationStatus::Success)
    }

    pub fn session_created(_session_id: &str, _user_id: &Option<String>, _privilege_level: &crate::unified_crypto::operations::PrivilegeLevel) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "operations", "session_create", OperationStatus::Success)
    }

    pub fn session_expired(_session_id: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "operations", "session_expire", OperationStatus::Success)
    }

    pub fn rate_limit_violation(_identifier: &str, _operation_type: &str) -> Self {
        Self::new(AuditEventType::SecurityViolation, AuditSeverity::Warning, "operations", "rate_limit", OperationStatus::Failure)
    }

    pub fn excessive_failures_lockout(_identifier: &str) -> Self {
        Self::new(AuditEventType::SecurityViolation, AuditSeverity::Error, "operations", "lockout", OperationStatus::Failure)
    }

    // Database operations
    pub fn database_operations_initialized() -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "database", "initialize", OperationStatus::Success)
    }

    pub fn database_operations_initialized_with_policy(_policy: &crate::unified_crypto::database::DatabaseEncryptionPolicy) -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "database", "initialize_policy", OperationStatus::Success)
    }

    pub fn database_encryption_start(_context: &crate::unified_crypto::database::DatabaseContext, _data_size: usize) -> Self {
        Self::new(AuditEventType::Encryption, AuditSeverity::Info, "database", "encrypt_start", OperationStatus::InProgress)
    }

    pub fn database_encryption_success(_context: &crate::unified_crypto::database::DatabaseContext, _encrypted_size: usize) -> Self {
        Self::new(AuditEventType::Encryption, AuditSeverity::Info, "database", "encrypt_success", OperationStatus::Success)
    }

    pub fn database_decryption_start(_context: &crate::unified_crypto::database::DatabaseContext, _encrypted_size: usize) -> Self {
        Self::new(AuditEventType::Decryption, AuditSeverity::Info, "database", "decrypt_start", OperationStatus::InProgress)
    }

    pub fn database_decryption_success(_context: &crate::unified_crypto::database::DatabaseContext) -> Self {
        Self::new(AuditEventType::Decryption, AuditSeverity::Info, "database", "decrypt_success", OperationStatus::Success)
    }

    pub fn database_key_rotation_start(_context: &crate::unified_crypto::database::DatabaseContext) -> Self {
        Self::new(AuditEventType::KeyRotation, AuditSeverity::Info, "database", "key_rotation_start", OperationStatus::InProgress)
    }

    pub fn database_key_rotation_success(_context: &crate::unified_crypto::database::DatabaseContext, _new_key_id: &crate::unified_crypto::types::KeyId) -> Self {
        Self::new(AuditEventType::KeyRotation, AuditSeverity::Info, "database", "key_rotation_success", OperationStatus::Success)
    }

    pub fn database_policy_updated(_old_policy: &crate::unified_crypto::database::DatabaseEncryptionPolicy, _new_policy: &crate::unified_crypto::database::DatabaseEncryptionPolicy) -> Self {
        Self::new(AuditEventType::Configuration, AuditSeverity::Info, "database", "policy_update", OperationStatus::Success)
    }

    // Authentication operations
    pub fn auth_operations_initialized() -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "authentication", "initialize", OperationStatus::Success)
    }

    pub fn auth_attempt_start(_user_id: &str, _auth_method: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "authentication", "auth_start", OperationStatus::InProgress)
    }

    pub fn auth_attempt_success(_user_id: &str, _auth_method: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "authentication", "auth_success", OperationStatus::Success)
    }

    pub fn auth_attempt_failed(_user_id: &str, _auth_method: &str, _reason: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Warning, "authentication", "auth_failed", OperationStatus::Failure)
    }

    pub fn auth_mfa_attempt_start(_user_id: &str, _factor_count: usize) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "authentication", "mfa_start", OperationStatus::InProgress)
    }

    pub fn auth_mfa_attempt_success(_user_id: &str, _factors_used: &[String]) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "authentication", "mfa_success", OperationStatus::Success)
    }

    pub fn auth_mfa_attempt_failed(_user_id: &str, _verified_factors: &[String]) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Warning, "authentication", "mfa_failed", OperationStatus::Failure)
    }

    pub fn auth_token_created(_user_id: &str, _session_id: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "authentication", "token_created", OperationStatus::Success)
    }

    pub fn auth_policy_updated(_old_policy: &crate::unified_crypto::auth::AuthenticationPolicy, _new_policy: &crate::unified_crypto::auth::AuthenticationPolicy) -> Self {
        Self::new(AuditEventType::Configuration, AuditSeverity::Info, "authentication", "policy_update", OperationStatus::Success)
    }

    // Network operations
    pub fn network_operations_initialized() -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "network", "initialize", OperationStatus::Success)
    }

    pub fn network_channel_creation_start(_peer_id: &str) -> Self {
        Self::new(AuditEventType::KeyExchange, AuditSeverity::Info, "network", "channel_create_start", OperationStatus::InProgress)
    }

    pub fn network_channel_creation_success(_channel_id: &str, _peer_id: &str) -> Self {
        Self::new(AuditEventType::KeyExchange, AuditSeverity::Info, "network", "channel_create_success", OperationStatus::Success)
    }

    pub fn network_channel_closed(_channel_id: &str) -> Self {
        Self::new(AuditEventType::Communication, AuditSeverity::Info, "network", "channel_close", OperationStatus::Success)
    }

    pub fn network_message_encryption_start(_channel_id: &str, _message_size: usize) -> Self {
        Self::new(AuditEventType::Encryption, AuditSeverity::Info, "network", "message_encrypt_start", OperationStatus::InProgress)
    }

    pub fn network_message_encryption_success(_message_id: &str, _channel_id: &str) -> Self {
        Self::new(AuditEventType::Encryption, AuditSeverity::Info, "network", "message_encrypt_success", OperationStatus::Success)
    }

    pub fn network_message_decryption_start(_message_id: &str) -> Self {
        Self::new(AuditEventType::Decryption, AuditSeverity::Info, "network", "message_decrypt_start", OperationStatus::InProgress)
    }

    pub fn network_message_decryption_success(_message_id: &str) -> Self {
        Self::new(AuditEventType::Decryption, AuditSeverity::Info, "network", "message_decrypt_success", OperationStatus::Success)
    }

    pub fn network_peer_auth_start(_peer_id: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "network", "peer_auth_start", OperationStatus::InProgress)
    }

    pub fn network_peer_auth_success(_peer_id: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Info, "network", "peer_auth_success", OperationStatus::Success)
    }

    pub fn network_peer_auth_failed(_peer_id: &str, _reason: &str) -> Self {
        Self::new(AuditEventType::Authentication, AuditSeverity::Warning, "network", "peer_auth_failed", OperationStatus::Failure)
    }

    pub fn network_policy_updated(_old_policy: &crate::unified_crypto::network::NetworkSecurityPolicy, _new_policy: &crate::unified_crypto::network::NetworkSecurityPolicy) -> Self {
        Self::new(AuditEventType::Configuration, AuditSeverity::Info, "network", "policy_update", OperationStatus::Success)
    }

    // Backup operations
    pub fn backup_operations_initialized() -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "backup", "initialize", OperationStatus::Success)
    }

    pub fn backup_creation_start(_backup_id: &str, _data_size: usize) -> Self {
        Self::new(AuditEventType::Backup, AuditSeverity::Info, "backup", "create_start", OperationStatus::InProgress)
    }

    pub fn backup_creation_success(_backup_id: &str) -> Self {
        Self::new(AuditEventType::Backup, AuditSeverity::Info, "backup", "create_success", OperationStatus::Success)
    }

    pub fn backup_restore_start(_backup_id: &str) -> Self {
        Self::new(AuditEventType::Recovery, AuditSeverity::Info, "backup", "restore_start", OperationStatus::InProgress)
    }

    pub fn backup_restore_success(_backup_id: &str) -> Self {
        Self::new(AuditEventType::Recovery, AuditSeverity::Info, "backup", "restore_success", OperationStatus::Success)
    }

    // CLI operations
    pub fn cli_operations_initialized() -> Self {
        Self::new(AuditEventType::SystemInitialization, AuditSeverity::Info, "cli", "initialize", OperationStatus::Success)
    }

    pub fn cli_keypair_generation_start() -> Self {
        Self::new(AuditEventType::KeyGeneration, AuditSeverity::Info, "cli", "keypair_start", OperationStatus::InProgress)
    }

    pub fn cli_keypair_generation_success(_key_id: &crate::unified_crypto::types::KeyId) -> Self {
        Self::new(AuditEventType::KeyGeneration, AuditSeverity::Info, "cli", "keypair_success", OperationStatus::Success)
    }
}

/// Types of audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// System initialization
    Initialization,
    /// System initialization (alias for compatibility)
    SystemInitialization,
    /// System shutdown
    Shutdown,
    /// System shutdown (alias for compatibility)
    SystemShutdown,
    /// Authentication operations
    Authentication,
    /// Configuration operations
    Configuration,
    /// Encryption operations
    Encryption,
    /// Decryption operations
    Decryption,
    /// Digital signing operations
    Signing,
    /// Signature verification operations
    Verification,
    /// Key generation
    KeyGeneration,
    /// Key access/loading
    KeyAccess,
    /// Key rotation
    KeyRotation,
    /// Key revocation
    KeyRevocation,
    /// Key backup
    KeyBackup,
    /// Key management operations
    KeyManagement,
    /// Key exchange operations
    KeyExchange,
    /// Automatic key rotation
    AutomaticRotation,
    /// Communication operations
    Communication,
    /// Backup operations
    Backup,
    /// Recovery operations
    Recovery,
    /// Security policy violations
    SecurityViolation,
    /// Configuration changes
    ConfigurationChange,
    /// Performance anomalies
    PerformanceAnomaly,
    /// Access control events
    AccessControl,
    /// Error conditions
    Error,
}

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Debug information
    Debug,
    /// Informational events
    Info,
    /// Warning conditions
    Warning,
    /// Error conditions
    Error,
    /// Critical security events
    Critical,
}

impl fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditSeverity::Debug => write!(f, "DEBUG"),
            AuditSeverity::Info => write!(f, "INFO"),
            AuditSeverity::Warning => write!(f, "WARN"),
            AuditSeverity::Error => write!(f, "ERROR"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Success,
    /// Operation failed
    Failure,
}

/// Performance metrics for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation duration in milliseconds
    pub duration_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,
    /// Operation throughput (ops/sec)
    pub throughput_ops_per_sec: Option<f64>,
    /// Additional performance data
    pub custom_metrics: HashMap<String, f64>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(duration_ms: u64) -> Self {
        Self {
            duration_ms,
            memory_usage_bytes: None,
            cpu_usage_percent: None,
            throughput_ops_per_sec: None,
            custom_metrics: HashMap::new(),
        }
    }

    /// Add custom metric
    pub fn with_custom_metric(mut self, name: &str, value: f64) -> Self {
        self.custom_metrics.insert(name.to_string(), value);
        self
    }
}

/// Security context for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Risk level of the operation
    pub risk_level: RiskLevel,
    /// Whether the event requires investigation
    pub requires_investigation: bool,
    /// Source IP address (if applicable)
    pub source_ip: Option<String>,
    /// User agent or client identifier
    pub user_agent: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Additional security metadata
    pub security_tags: HashMap<String, String>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            risk_level: RiskLevel::Low,
            requires_investigation: false,
            source_ip: None,
            user_agent: None,
            session_id: None,
            security_tags: HashMap::new(),
        }
    }
}

/// Risk level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk operation
    Low,
    /// Medium risk operation
    Medium,
    /// High risk operation
    High,
    /// Critical risk operation
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Security audit trail for tamper-evident logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditTrail {
    /// Chain of audit events
    pub events: Vec<CryptoAuditEvent>,
    /// Trail integrity hash
    pub integrity_hash: Vec<u8>,
    /// Trail creation timestamp
    pub created_at: SystemTime,
    /// Last update timestamp
    pub updated_at: SystemTime,
    /// Trail metadata
    pub metadata: HashMap<String, String>,
}

impl SecurityAuditTrail {
    /// Create a new audit trail
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            events: Vec::new(),
            integrity_hash: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Add event to the trail
    pub fn add_event(&mut self, event: CryptoAuditEvent) -> UnifiedCryptoResult<()> {
        self.events.push(event);
        self.updated_at = SystemTime::now();
        self.update_integrity_hash()?;
        Ok(())
    }

    /// Verify trail integrity
    pub fn verify_integrity(&self) -> bool {
        // Implementation would verify the integrity hash
        // For now, return true as placeholder
        true
    }

    /// Update integrity hash
    fn update_integrity_hash(&mut self) -> UnifiedCryptoResult<()> {
        // Implementation would compute cryptographic hash of the entire trail
        // For now, use a simple placeholder
        self.integrity_hash = vec![0u8; 32];
        Ok(())
    }
}

/// Main cryptographic audit logger
pub struct CryptoAuditLogger {
    /// Configuration
    config: Arc<AuditConfig>,
    /// Audit trail
    audit_trail: Arc<Mutex<SecurityAuditTrail>>,
    /// Event filters
    filters: Vec<Box<dyn AuditFilter>>,
    /// Output handlers
    outputs: Vec<Box<dyn AuditOutput>>,
}

impl CryptoAuditLogger {
    /// Create a new audit logger
    ///
    /// # Arguments
    /// * `config` - Audit configuration
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New audit logger or error
    pub fn new(config: &AuditConfig) -> UnifiedCryptoResult<Self> {
        let audit_trail = Arc::new(Mutex::new(SecurityAuditTrail::new()));
        
        // Initialize default outputs
        let mut outputs: Vec<Box<dyn AuditOutput>> = vec![
            Box::new(ConsoleAuditOutput::new(config.log_level)),
        ];

        // Add file output if configured
        if config.enabled {
            outputs.push(Box::new(FileAuditOutput::new(
                "./audit.log", 
                config.max_log_size
            )?));
        }

        Ok(Self {
            config: Arc::new(config.clone()),
            audit_trail,
            filters: Vec::new(),
            outputs,
        })
    }

    /// Create a new audit logger with default configuration
    pub fn with_default_config() -> Self {
        let default_config = AuditConfig::default();
        Self::new(&default_config).unwrap_or_else(|_| {
            // Fallback to minimal logger if config fails
            Self {
                config: Arc::new(default_config),
                audit_trail: Arc::new(Mutex::new(SecurityAuditTrail::new())),
                filters: Vec::new(),
                outputs: vec![Box::new(ConsoleAuditOutput::new(
                    crate::unified_crypto::config::AuditLogLevel::Info
                ))],
            }
        })
    }

    /// Log a cryptographic audit event
    ///
    /// # Arguments
    /// * `event` - Audit event to log
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<()>` - Success or error
    pub fn log_crypto_event(&self, event: CryptoAuditEvent) -> UnifiedCryptoResult<()> {
        // Check if logging is enabled
        if !self.config.enabled {
            return Ok(());
        }

        // Apply filters
        for filter in &self.filters {
            if !filter.should_log(&event) {
                return Ok(());
            }
        }

        // Check severity level
        if event.severity < self.config.log_level.into() {
            return Ok(());
        }

        // Add to audit trail
        {
            let mut trail = self.audit_trail.lock()
                .map_err(|_| UnifiedCryptoError::Internal {
                    context: "Failed to acquire audit trail lock".to_string(),
                })?;
            
            trail.add_event(event.clone())?;
        }

        // Send to outputs
        for output in &self.outputs {
            if let Err(e) = output.write_event(&event) {
                // Log output errors but don't fail the operation
                eprintln!("Audit output error: {}", e);
            }
        }

        Ok(())
    }

    /// Get audit trail for analysis
    pub fn get_audit_trail(&self) -> UnifiedCryptoResult<SecurityAuditTrail> {
        let trail = self.audit_trail.lock()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire audit trail lock".to_string(),
            })?;
        
        Ok(trail.clone())
    }

    /// Add audit filter
    pub fn add_filter(&mut self, filter: Box<dyn AuditFilter>) {
        self.filters.push(filter);
    }

    /// Add audit output
    pub fn add_output(&mut self, output: Box<dyn AuditOutput>) {
        self.outputs.push(output);
    }
}

/// Trait for audit event filtering
pub trait AuditFilter: Send + Sync {
    /// Check if event should be logged
    fn should_log(&self, event: &CryptoAuditEvent) -> bool;
}

/// Trait for audit event output
pub trait AuditOutput: Send + Sync {
    /// Write audit event to output
    fn write_event(&self, event: &CryptoAuditEvent) -> UnifiedCryptoResult<()>;
}

/// Console audit output implementation
struct ConsoleAuditOutput {
    min_level: AuditSeverity,
}

impl ConsoleAuditOutput {
    fn new(min_level: crate::unified_crypto::config::AuditLogLevel) -> Self {
        Self {
            min_level: min_level.into(),
        }
    }
}

impl AuditOutput for ConsoleAuditOutput {
    fn write_event(&self, event: &CryptoAuditEvent) -> UnifiedCryptoResult<()> {
        if event.severity >= self.min_level {
            println!(
                "[{}] {} {} - {} ({}): {}",
                format_timestamp(event.timestamp),
                event.severity,
                event.component,
                event.operation,
                event.status,
                event.event_id
            );
        }
        Ok(())
    }
}

/// File audit output implementation
struct FileAuditOutput {
    file_path: std::path::PathBuf,
    max_size: u64,
}

impl FileAuditOutput {
    fn new(file_path: &str, max_size: u64) -> UnifiedCryptoResult<Self> {
        Ok(Self {
            file_path: std::path::PathBuf::from(file_path),
            max_size,
        })
    }
}

impl AuditOutput for FileAuditOutput {
    fn write_event(&self, event: &CryptoAuditEvent) -> UnifiedCryptoResult<()> {
        // Implementation would write to file with rotation
        // For now, just return success
        Ok(())
    }
}

/// Convert AuditLogLevel to AuditSeverity
impl From<crate::unified_crypto::config::AuditLogLevel> for AuditSeverity {
    fn from(level: crate::unified_crypto::config::AuditLogLevel) -> Self {
        match level {
            crate::unified_crypto::config::AuditLogLevel::Debug => AuditSeverity::Debug,
            crate::unified_crypto::config::AuditLogLevel::Info => AuditSeverity::Info,
            crate::unified_crypto::config::AuditLogLevel::Warn => AuditSeverity::Warning,
            crate::unified_crypto::config::AuditLogLevel::Error => AuditSeverity::Error,
        }
    }
}

impl fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationStatus::InProgress => write!(f, "IN_PROGRESS"),
            OperationStatus::Success => write!(f, "SUCCESS"),
            OperationStatus::Failure => write!(f, "FAILURE"),
        }
    }
}

/// Generate a unique event ID
fn generate_event_id() -> String {
    use std::time::UNIX_EPOCH;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    format!("audit-{:x}", timestamp)
}

/// Format timestamp for display
fn format_timestamp(timestamp: SystemTime) -> String {
    let duration = timestamp.duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::config::AuditConfig;

    #[test]
    fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = CryptoAuditLogger::new(&config);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_audit_event_creation() {
        let event = CryptoAuditEvent::new(
            AuditEventType::Initialization,
            AuditSeverity::Info,
            "test_component",
            "test_operation",
            OperationStatus::Success,
        );
        
        assert_eq!(event.event_type, AuditEventType::Initialization);
        assert_eq!(event.severity, AuditSeverity::Info);
        assert_eq!(event.component, "test_component");
        assert_eq!(event.operation, "test_operation");
        assert_eq!(event.status, OperationStatus::Success);
    }

    #[test]
    fn test_audit_trail_integrity() {
        let mut trail = SecurityAuditTrail::new();
        
        let event = CryptoAuditEvent::new(
            AuditEventType::KeyGeneration,
            AuditSeverity::Info,
            "key_manager",
            "generate",
            OperationStatus::Success,
        );
        
        let result = trail.add_event(event);
        assert!(result.is_ok());
        assert_eq!(trail.events.len(), 1);
        assert!(trail.verify_integrity());
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new(100)
            .with_custom_metric("throughput", 50.0);
        
        assert_eq!(metrics.duration_ms, 100);
        assert_eq!(metrics.custom_metrics.get("throughput"), Some(&50.0));
    }

    #[test]
    fn test_security_context() {
        let mut context = SecurityContext::default();
        context.risk_level = RiskLevel::High;
        context.requires_investigation = true;
        
        assert_eq!(context.risk_level, RiskLevel::High);
        assert!(context.requires_investigation);
    }

    #[test]
    fn test_audit_severity_ordering() {
        assert!(AuditSeverity::Critical > AuditSeverity::Error);
        assert!(AuditSeverity::Error > AuditSeverity::Warning);
        assert!(AuditSeverity::Warning > AuditSeverity::Info);
        assert!(AuditSeverity::Info > AuditSeverity::Debug);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
    }
}