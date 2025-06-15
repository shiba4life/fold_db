//! Specialized audit logging for key rotation operations
//!
//! This module extends the existing audit system with key rotation-specific
//! audit events, tamper-proof trails, and enhanced security metadata.

use super::audit_logger::{
    AuditEvent, AuditEventType, CryptoAuditLogger, OperationResult,
};
use super::key_rotation::{KeyRotationRequest, RotationReason};
use crate::security_types::{RotationStatus, Severity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Key rotation specific audit event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyRotationAuditEventType {
    /// Rotation request initiated
    RotationRequested,
    /// Rotation request validated
    RotationValidated,
    /// Rotation in progress
    RotationInProgress,
    /// Rotation completed successfully
    RotationCompleted,
    /// Rotation failed
    RotationFailed,
    /// Old key invalidated
    OldKeyInvalidated,
    /// New key activated
    NewKeyActivated,
    // REMOVED: Emergency bypass functionality for security
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Suspicious rotation pattern
    SuspiciousPattern,
    /// Policy violation detected
    PolicyViolation,
    /// Compliance check performed
    ComplianceCheck,
}

impl KeyRotationAuditEventType {
    /// Get the base audit event type for this rotation event
    pub fn to_audit_event_type(&self) -> AuditEventType {
        match self {
            KeyRotationAuditEventType::RateLimitExceeded
            | KeyRotationAuditEventType::SuspiciousPattern
            | KeyRotationAuditEventType::PolicyViolation => AuditEventType::Security,
            KeyRotationAuditEventType::ComplianceCheck => AuditEventType::Configuration,
            _ => AuditEventType::KeyOperation,
        }
    }

    /// Get the severity level for this event type
    pub fn default_severity(&self) -> Severity {
        match self {
            KeyRotationAuditEventType::RotationFailed
            | KeyRotationAuditEventType::PolicyViolation => Severity::Error,
            KeyRotationAuditEventType::RateLimitExceeded
            | KeyRotationAuditEventType::SuspiciousPattern => Severity::Warning,
            KeyRotationAuditEventType::RotationCompleted
            | KeyRotationAuditEventType::NewKeyActivated
            | KeyRotationAuditEventType::OldKeyInvalidated => Severity::Info,
            _ => Severity::Info,
        }
    }
}

/// Enhanced security metadata for key rotation auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationSecurityMetadata {
    /// Source IP address of the request
    pub source_ip: Option<IpAddr>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Geolocation information
    pub geolocation: Option<GeolocationInfo>,
    /// Session information
    pub session_info: Option<SessionInfo>,
    /// Device fingerprint
    pub device_fingerprint: Option<String>,
    /// Authentication method used
    pub auth_method: Option<String>,
    /// Risk score (0.0 to 1.0)
    pub risk_score: Option<f64>,
    /// Request source (API, CLI, Web, etc.)
    pub request_source: Option<String>,
}

/// Geolocation information for audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeolocationInfo {
    /// Country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// ISP information
    pub isp: Option<String>,
    /// Is this a known VPN/proxy?
    pub is_vpn: Option<bool>,
}

/// Session information for audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub session_id: String,
    /// Session start time
    pub session_start: DateTime<Utc>,
    /// Session duration at time of request
    pub session_duration: Duration,
    /// Number of operations in this session
    pub operations_count: u64,
    /// Previous operation timestamp
    pub last_operation: Option<DateTime<Utc>>,
}

/// Tamper-proof audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperProofAuditEntry {
    /// Event ID
    pub event_id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: KeyRotationAuditEventType,
    /// Audit data hash (SHA-256)
    pub data_hash: String,
    /// Previous entry hash (chain)
    pub previous_hash: Option<String>,
    /// Digital signature of this entry
    pub signature: Option<String>,
    /// Sequence number in the chain
    pub sequence_number: u64,
}

impl TamperProofAuditEntry {
    /// Create a new tamper-proof audit entry
    pub fn new(
        event_id: Uuid,
        event_type: KeyRotationAuditEventType,
        data: &[u8],
        previous_hash: Option<String>,
        sequence_number: u64,
    ) -> Self {
        let timestamp = Utc::now();
        let data_hash = Self::compute_hash(data);

        Self {
            event_id,
            timestamp,
            event_type,
            data_hash,
            previous_hash,
            signature: None,
            sequence_number,
        }
    }

    /// Compute SHA-256 hash of data
    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Get the hash of this entry for chaining
    pub fn entry_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.event_id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(format!("{:?}", self.event_type).as_bytes());
        hasher.update(&self.data_hash);
        hasher.update(self.sequence_number.to_le_bytes());
        if let Some(ref prev_hash) = self.previous_hash {
            hasher.update(prev_hash.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

/// Key rotation audit correlation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationAuditCorrelation {
    /// Primary correlation ID for the rotation operation
    pub operation_id: Uuid,
    /// User ID who initiated the rotation
    pub user_id: Option<String>,
    /// Old key fingerprint
    pub old_key_id: String,
    /// New key fingerprint
    pub new_key_id: String,
    /// Rotation reason
    pub rotation_reason: RotationReason,
    /// Start timestamp
    pub started_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Related event IDs in chronological order
    pub related_events: Vec<Uuid>,
    /// Security metadata
    pub security_metadata: KeyRotationSecurityMetadata,
    /// Current status
    pub status: RotationStatus,
    /// Failed attempts count
    pub failed_attempts: u32,
    /// Warnings collected during rotation
    pub warnings: Vec<String>,
}

/// Specialized audit logger for key rotation operations
pub struct KeyRotationAuditLogger {
    /// Base audit logger
    base_logger: Arc<CryptoAuditLogger>,
    /// Active rotation correlations
    correlations: Arc<RwLock<HashMap<Uuid, RotationAuditCorrelation>>>,
    /// Tamper-proof audit chain
    audit_chain: Arc<RwLock<Vec<TamperProofAuditEntry>>>,
    /// Sequence counter for audit chain
    sequence_counter: Arc<RwLock<u64>>,
}

impl KeyRotationAuditLogger {
    /// Create a new key rotation audit logger
    pub fn new(base_logger: Arc<CryptoAuditLogger>) -> Self {
        Self {
            base_logger,
            correlations: Arc::new(RwLock::new(HashMap::new())),
            audit_chain: Arc::new(RwLock::new(Vec::new())),
            sequence_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Start tracking a new key rotation operation
    pub async fn start_rotation_tracking(
        &self,
        request: &KeyRotationRequest,
        user_id: Option<String>,
        security_metadata: KeyRotationSecurityMetadata,
    ) -> Uuid {
        let operation_id = Uuid::new_v4();
        let old_key_id = hex::encode(&request.old_public_key.to_bytes()[..8]);
        let new_key_id = hex::encode(&request.new_public_key.to_bytes()[..8]);

        let correlation = RotationAuditCorrelation {
            operation_id,
            user_id,
            old_key_id,
            new_key_id,
            rotation_reason: request.reason.clone(),
            started_at: Utc::now(),
            completed_at: None,
            related_events: Vec::new(),
            security_metadata,
            status: RotationStatus::Requested,
            failed_attempts: 0,
            warnings: Vec::new(),
        };

        {
            let mut correlations = self.correlations.write().await;
            correlations.insert(operation_id, correlation);
        }

        // Log the rotation requested event
        self.log_rotation_event(
            operation_id,
            KeyRotationAuditEventType::RotationRequested,
            "Key rotation operation initiated",
            None,
            Some(Duration::from_millis(0)),
            OperationResult::InProgress,
        )
        .await;

        operation_id
    }

    /// Log a key rotation event with correlation
    pub async fn log_rotation_event(
        &self,
        operation_id: Uuid,
        event_type: KeyRotationAuditEventType,
        description: &str,
        actor: Option<String>,
        duration: Option<Duration>,
        result: OperationResult,
    ) {
        let event_id = Uuid::new_v4();

        // Update correlation tracking
        {
            let mut correlations = self.correlations.write().await;
            if let Some(correlation) = correlations.get_mut(&operation_id) {
                correlation.related_events.push(event_id);

                // Update status based on event type
                match event_type {
                    KeyRotationAuditEventType::RotationValidated => {
                        correlation.status = RotationStatus::Validating;
                    }
                    KeyRotationAuditEventType::RotationInProgress => {
                        correlation.status = RotationStatus::InProgress;
                    }
                    KeyRotationAuditEventType::RotationCompleted => {
                        correlation.status = RotationStatus::Completed;
                        correlation.completed_at = Some(Utc::now());
                    }
                    KeyRotationAuditEventType::RotationFailed => {
                        correlation.status = RotationStatus::Failed;
                        correlation.failed_attempts += 1;
                        correlation.completed_at = Some(Utc::now());
                    }
                    // REMOVED: Emergency bypass case - removed for security
                    _ => {}
                }
            }
        }

        // Create enhanced metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "operation_id".to_string(),
            serde_json::Value::String(operation_id.to_string()),
        );
        metadata.insert("event_type".to_string(), serde_json::json!(event_type));
        metadata.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );

        // Add correlation metadata
        if let Some(correlation) = self.get_correlation(&operation_id).await {
            metadata.insert(
                "old_key_id".to_string(),
                serde_json::Value::String(correlation.old_key_id),
            );
            metadata.insert(
                "new_key_id".to_string(),
                serde_json::Value::String(correlation.new_key_id),
            );
            metadata.insert(
                "rotation_reason".to_string(),
                serde_json::json!(correlation.rotation_reason),
            );
            metadata.insert(
                "operation_status".to_string(),
                serde_json::json!(correlation.status),
            );
            metadata.insert(
                "failed_attempts".to_string(),
                serde_json::Value::Number(correlation.failed_attempts.into()),
            );

            // Add security metadata
            let sec_meta = &correlation.security_metadata;
            if let Some(ref ip) = sec_meta.source_ip {
                metadata.insert(
                    "source_ip".to_string(),
                    serde_json::Value::String(ip.to_string()),
                );
            }
            if let Some(ref user_agent) = sec_meta.user_agent {
                metadata.insert(
                    "user_agent".to_string(),
                    serde_json::Value::String(user_agent.clone()),
                );
            }
            if let Some(ref geo) = sec_meta.geolocation {
                metadata.insert("geolocation".to_string(), serde_json::json!(geo));
            }
            if let Some(ref session) = sec_meta.session_info {
                metadata.insert("session_info".to_string(), serde_json::json!(session));
            }
            if let Some(ref device) = sec_meta.device_fingerprint {
                metadata.insert(
                    "device_fingerprint".to_string(),
                    serde_json::Value::String(device.clone()),
                );
            }
            if let Some(ref auth) = sec_meta.auth_method {
                metadata.insert(
                    "auth_method".to_string(),
                    serde_json::Value::String(auth.clone()),
                );
            }
            if let Some(risk_score) = sec_meta.risk_score {
                metadata.insert("risk_score".to_string(), serde_json::json!(risk_score));
            }
            if let Some(ref source) = sec_meta.request_source {
                metadata.insert(
                    "request_source".to_string(),
                    serde_json::Value::String(source.clone()),
                );
            }
        }

        // Create tamper-proof audit entry
        let event_data = serde_json::to_vec(&metadata).unwrap_or_default();
        let sequence_number = {
            let mut counter = self.sequence_counter.write().await;
            *counter += 1;
            *counter
        };

        let previous_hash = {
            let chain = self.audit_chain.read().await;
            chain.last().map(|entry| entry.entry_hash())
        };

        let tamper_proof_entry = TamperProofAuditEntry::new(
            event_id,
            event_type.clone(),
            &event_data,
            previous_hash,
            sequence_number,
        );

        // Store in tamper-proof chain
        {
            let mut chain = self.audit_chain.write().await;
            chain.push(tamper_proof_entry);

            // Limit chain size to prevent memory issues
            if chain.len() > 100_000 {
                chain.drain(0..10000);
            }
        }

        // Log to base audit logger
        let _audit_event = AuditEvent {
            event_id,
            timestamp: Utc::now(),
            event_type: event_type.to_audit_event_type(),
            severity: event_type.default_severity(),
            component: "key_rotation_audit".to_string(),
            operation: description.to_string(),
            actor,
            result: result.clone(),
            duration,
            metadata,
            correlation_id: Some(operation_id),
            session_id: None,
        };

        // Use base logger's internal log_event method
        self.base_logger
            .log_key_operation(
                description,
                "key_rotation",
                duration.unwrap_or_default(),
                result.clone(),
                Some(operation_id),
            )
            .await;
    }

    /// Log security policy violation
    pub async fn log_policy_violation(
        &self,
        operation_id: Uuid,
        policy_name: &str,
        violation_details: &str,
        actor: Option<String>,
    ) {
        let description = format!(
            "Policy violation detected: {} - {}",
            policy_name, violation_details
        );

        // Add warning to correlation
        {
            let mut correlations = self.correlations.write().await;
            if let Some(correlation) = correlations.get_mut(&operation_id) {
                correlation.warnings.push(description.clone());
            }
        }

        self.log_rotation_event(
            operation_id,
            KeyRotationAuditEventType::PolicyViolation,
            &description,
            actor,
            None,
            OperationResult::Failure {
                error_type: "PolicyViolation".to_string(),
                error_message: violation_details.to_string(),
                error_code: Some("POLICY_VIOLATION".to_string()),
            },
        )
        .await;
    }

    /// Log suspicious pattern detection
    pub async fn log_suspicious_pattern(
        &self,
        operation_id: Uuid,
        pattern_type: &str,
        pattern_details: &str,
        risk_score: f64,
        actor: Option<String>,
    ) {
        let description = format!(
            "Suspicious pattern detected: {} - {} (risk: {:.2})",
            pattern_type, pattern_details, risk_score
        );

        // Add warning to correlation
        {
            let mut correlations = self.correlations.write().await;
            if let Some(correlation) = correlations.get_mut(&operation_id) {
                correlation.warnings.push(description.clone());
                // Update risk score if higher
                if let Some(current_risk) = correlation.security_metadata.risk_score {
                    if risk_score > current_risk {
                        correlation.security_metadata.risk_score = Some(risk_score);
                    }
                } else {
                    correlation.security_metadata.risk_score = Some(risk_score);
                }
            }
        }

        let result = if risk_score > 0.8 {
            OperationResult::Failure {
                error_type: "HighRiskPattern".to_string(),
                error_message: pattern_details.to_string(),
                error_code: Some("SUSPICIOUS_PATTERN".to_string()),
            }
        } else {
            OperationResult::Success
        };

        self.log_rotation_event(
            operation_id,
            KeyRotationAuditEventType::SuspiciousPattern,
            &description,
            actor,
            None,
            result,
        )
        .await;
    }

    /// Get correlation information for an operation
    pub async fn get_correlation(&self, operation_id: &Uuid) -> Option<RotationAuditCorrelation> {
        let correlations = self.correlations.read().await;
        correlations.get(operation_id).cloned()
    }

    /// Get all events for a rotation operation
    pub async fn get_operation_events(&self, operation_id: &Uuid) -> Vec<Uuid> {
        let correlations = self.correlations.read().await;
        correlations
            .get(operation_id)
            .map(|c| c.related_events.clone())
            .unwrap_or_default()
    }

    /// Get tamper-proof audit chain
    pub async fn get_audit_chain(&self) -> Vec<TamperProofAuditEntry> {
        let chain = self.audit_chain.read().await;
        chain.clone()
    }

    /// Verify integrity of the audit chain
    pub async fn verify_audit_chain_integrity(&self) -> bool {
        let chain = self.audit_chain.read().await;

        if chain.is_empty() {
            return true;
        }

        // Verify chain consistency
        for i in 1..chain.len() {
            let current = &chain[i];
            let previous = &chain[i - 1];

            // Check sequence numbers
            if current.sequence_number != previous.sequence_number + 1 {
                return false;
            }

            // Check hash chaining
            if let Some(ref prev_hash) = current.previous_hash {
                if *prev_hash != previous.entry_hash() {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Export audit data for compliance reporting
    pub async fn export_audit_data(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> serde_json::Result<String> {
        let correlations = self.correlations.read().await;
        let chain = self.audit_chain.read().await;

        let filtered_correlations: Vec<&RotationAuditCorrelation> = correlations
            .values()
            .filter(|c| {
                if let Some(start) = start_time {
                    if c.started_at < start {
                        return false;
                    }
                }
                if let Some(end) = end_time {
                    if c.started_at > end {
                        return false;
                    }
                }
                true
            })
            .collect();

        let filtered_chain: Vec<&TamperProofAuditEntry> = chain
            .iter()
            .filter(|e| {
                if let Some(start) = start_time {
                    if e.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = end_time {
                    if e.timestamp > end {
                        return false;
                    }
                }
                true
            })
            .collect();

        let export_data = serde_json::json!({
            "export_timestamp": Utc::now(),
            "start_time": start_time,
            "end_time": end_time,
            "correlations": filtered_correlations,
            "audit_chain": filtered_chain,
            "chain_integrity_verified": self.verify_audit_chain_integrity().await
        });

        serde_json::to_string_pretty(&export_data)
    }

    /// Clean up old correlations and audit entries
    pub async fn cleanup_old_data(&self, retention_days: u64) {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);

        // Clean up correlations
        {
            let mut correlations = self.correlations.write().await;
            correlations.retain(|_, c| c.started_at > cutoff);
        }

        // Clean up audit chain (but keep some for integrity)
        {
            let mut chain = self.audit_chain.write().await;
            let retain_count = chain.len().saturating_sub(50000); // Keep last 50k entries minimum
            if retain_count > 0 {
                let cutoff_index = chain
                    .iter()
                    .position(|e| e.timestamp > cutoff)
                    .unwrap_or(retain_count)
                    .min(retain_count);

                if cutoff_index > 0 {
                    chain.drain(0..cutoff_index);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_keypair;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_key_rotation_audit_tracking() {
        let base_logger = Arc::new(CryptoAuditLogger::with_default_config());
        let audit_logger = KeyRotationAuditLogger::new(base_logger);

        // Create test rotation request
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::Scheduled,
            Some("test-client".to_string()),
            HashMap::new(),
        )
        .unwrap();

        let security_metadata = KeyRotationSecurityMetadata {
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            user_agent: Some("DataFold-CLI/1.0".to_string()),
            geolocation: None,
            session_info: None,
            device_fingerprint: Some("test-device-123".to_string()),
            auth_method: Some("signature".to_string()),
            risk_score: Some(0.1),
            request_source: Some("CLI".to_string()),
        };

        // Start tracking
        let operation_id = audit_logger
            .start_rotation_tracking(&request, Some("test-user".to_string()), security_metadata)
            .await;

        // Log some events
        audit_logger
            .log_rotation_event(
                operation_id,
                KeyRotationAuditEventType::RotationValidated,
                "Request validation successful",
                Some("test-user".to_string()),
                Some(Duration::from_millis(50)),
                OperationResult::Success,
            )
            .await;

        audit_logger
            .log_rotation_event(
                operation_id,
                KeyRotationAuditEventType::RotationCompleted,
                "Key rotation completed successfully",
                Some("test-user".to_string()),
                Some(Duration::from_secs(1)),
                OperationResult::Success,
            )
            .await;

        // Verify correlation
        let correlation = audit_logger.get_correlation(&operation_id).await.unwrap();
        assert_eq!(correlation.status, RotationStatus::Completed);
        assert_eq!(correlation.related_events.len(), 3); // start + validate + complete
        assert!(correlation.completed_at.is_some());

        // Verify audit chain integrity
        assert!(audit_logger.verify_audit_chain_integrity().await);
    }

    #[tokio::test]
    async fn test_security_logging() {
        let base_logger = Arc::new(CryptoAuditLogger::with_default_config());
        let _base_logger_for_test = base_logger.clone();
        let audit_logger = KeyRotationAuditLogger::new(base_logger);

        let operation_id = Uuid::new_v4();

        // Log a basic rotation event
        audit_logger
            .log_rotation_event(
                operation_id,
                KeyRotationAuditEventType::RotationRequested,
                "Test security logging",
                None,
                Some(Duration::from_millis(10)),
                OperationResult::Success,
            )
            .await;

        // Verify audit chain has entries
        let audit_chain = audit_logger.get_audit_chain().await;
        assert!(!audit_chain.is_empty());
        
        // Verify the logged event was recorded
        let last_entry = audit_chain.last().unwrap();
        assert!(matches!(last_entry.event_type, KeyRotationAuditEventType::RotationRequested));
    }

    #[tokio::test]
    async fn test_audit_chain_integrity() {
        let base_logger = Arc::new(CryptoAuditLogger::with_default_config());
        let audit_logger = KeyRotationAuditLogger::new(base_logger);

        // Log several events
        for i in 0..5 {
            audit_logger
                .log_rotation_event(
                    Uuid::new_v4(),
                    KeyRotationAuditEventType::RotationRequested,
                    &format!("Test event {}", i),
                    None,
                    None,
                    OperationResult::Success,
                )
                .await;
        }

        // Verify chain integrity
        assert!(audit_logger.verify_audit_chain_integrity().await);

        let chain = audit_logger.get_audit_chain().await;
        assert_eq!(chain.len(), 5);

        // Verify sequence numbers
        for (i, entry) in chain.iter().enumerate() {
            assert_eq!(entry.sequence_number, (i + 1) as u64);
        }
    }
}
