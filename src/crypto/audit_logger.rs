//! Comprehensive audit logging for cryptographic operations
//!
//! This module provides structured audit logging for all encryption-related
//! operations including encryption/decryption, key derivation, backup/restore,
//! and security events.

use super::enhanced_error::{EnhancedCryptoError, ErrorSeverity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit event types for categorization and filtering
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Encryption operation events
    Encryption,
    /// Decryption operation events
    Decryption,
    /// Key derivation and generation events
    KeyOperation,
    /// Backup creation and restoration events
    BackupRestore,
    /// Security-related events (failures, violations)
    Security,
    /// Performance monitoring events
    Performance,
    /// Configuration changes
    Configuration,
    /// System startup and shutdown
    System,
}

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational events for normal operations
    Info,
    /// Warning events for potential issues
    Warning,
    /// Error events for failed operations
    Error,
    /// Critical events requiring immediate attention
    Critical,
}

/// Detailed audit event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Event timestamp in UTC
    pub timestamp: DateTime<Utc>,
    /// Type of audit event
    pub event_type: AuditEventType,
    /// Severity level
    pub severity: AuditSeverity,
    /// Component that generated the event
    pub component: String,
    /// Operation being performed
    pub operation: String,
    /// User or system identifier
    pub actor: Option<String>,
    /// Operation result (success/failure)
    pub result: OperationResult,
    /// Duration of the operation
    pub duration: Option<Duration>,
    /// Additional metadata specific to the event
    pub metadata: HashMap<String, serde_json::Value>,
    /// Related event IDs for correlation
    pub correlation_id: Option<Uuid>,
    /// Session or transaction identifier
    pub session_id: Option<String>,
}

/// Operation result for audit logging
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationResult {
    /// Operation completed successfully
    Success,
    /// Operation failed with error details
    Failure {
        error_type: String,
        error_message: String,
        error_code: Option<String>,
    },
    /// Operation was aborted or cancelled
    Aborted,
    /// Operation is still in progress
    InProgress,
}

/// Performance metrics for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU time used
    pub cpu_time: Option<Duration>,
    /// Memory usage peak
    pub memory_usage: Option<u64>,
    /// Data processed (bytes)
    pub data_size: Option<u64>,
    /// Throughput (bytes per second)
    pub throughput: Option<f64>,
    /// Custom performance counters
    pub custom_metrics: HashMap<String, f64>,
}

/// Security event details for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventDetails {
    /// Type of security event
    pub event_type: String,
    /// Threat level assessment
    pub threat_level: String,
    /// Source of the event (IP, user, etc.)
    pub source: Option<String>,
    /// Target of the event
    pub target: Option<String>,
    /// Additional security-specific metadata
    pub security_metadata: HashMap<String, String>,
}

/// Configuration for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log level for audit events
    pub log_level: String,
    /// Maximum number of events to keep in memory
    pub max_events_in_memory: usize,
    /// Enable structured JSON logging
    pub enable_json_output: bool,
    /// Enable file output for audit logs
    pub enable_file_output: bool,
    /// Audit log file path
    pub file_path: Option<String>,
    /// Log rotation settings
    pub rotation_size_mb: u64,
    /// Maximum number of archived log files
    pub max_archived_files: u32,
    /// Enable real-time alerting for critical events
    pub enable_alerting: bool,
    /// Batch size for bulk logging operations
    pub batch_size: usize,
    /// Event types to include in audit logs
    pub included_event_types: Vec<AuditEventType>,
    /// Event types to exclude from audit logs
    pub excluded_event_types: Vec<AuditEventType>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "INFO".to_string(),
            max_events_in_memory: 10000,
            enable_json_output: true,
            enable_file_output: true,
            file_path: Some("logs/crypto_audit.json".to_string()),
            rotation_size_mb: 100,
            max_archived_files: 10,
            enable_alerting: true,
            batch_size: 100,
            included_event_types: vec![
                AuditEventType::Encryption,
                AuditEventType::Decryption,
                AuditEventType::KeyOperation,
                AuditEventType::BackupRestore,
                AuditEventType::Security,
                AuditEventType::Performance,
            ],
            excluded_event_types: vec![],
        }
    }
}

/// Statistics about audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total number of events logged
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<AuditEventType, u64>,
    /// Events by severity
    pub events_by_severity: HashMap<AuditSeverity, u64>,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Number of security events
    pub security_events: u64,
    /// Average operation duration
    pub avg_operation_duration: Option<Duration>,
    /// Last event timestamp
    pub last_event_time: Option<DateTime<Utc>>,
}

/// Main audit logger for cryptographic operations
#[derive(Clone)]
pub struct CryptoAuditLogger {
    /// Configuration for audit logging
    config: AuditConfig,
    /// In-memory event storage
    events: Arc<RwLock<Vec<AuditEvent>>>,
    /// Audit statistics
    statistics: Arc<RwLock<AuditStatistics>>,
    /// Current session ID
    session_id: String,
}

impl CryptoAuditLogger {
    /// Create a new crypto audit logger
    pub fn new(config: AuditConfig) -> Self {
        let session_id = Uuid::new_v4().to_string();

        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(AuditStatistics {
                total_events: 0,
                events_by_type: HashMap::new(),
                events_by_severity: HashMap::new(),
                failed_operations: 0,
                security_events: 0,
                avg_operation_duration: None,
                last_event_time: None,
            })),
            session_id,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(AuditConfig::default())
    }

    /// Log an encryption operation
    pub async fn log_encryption_operation(
        &self,
        operation: &str,
        context: &str,
        data_size: usize,
        duration: Duration,
        result: OperationResult,
        correlation_id: Option<Uuid>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert(
            "context".to_string(),
            serde_json::Value::String(context.to_string()),
        );
        metadata.insert(
            "data_size".to_string(),
            serde_json::Value::Number(data_size.into()),
        );

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Encryption,
            severity: if matches!(result, OperationResult::Success) {
                AuditSeverity::Info
            } else {
                AuditSeverity::Error
            },
            component: "crypto_encryption".to_string(),
            operation: operation.to_string(),
            actor: None,
            result,
            duration: Some(duration),
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Log a decryption operation
    pub async fn log_decryption_operation(
        &self,
        operation: &str,
        context: &str,
        data_size: usize,
        duration: Duration,
        result: OperationResult,
        correlation_id: Option<Uuid>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert(
            "context".to_string(),
            serde_json::Value::String(context.to_string()),
        );
        metadata.insert(
            "data_size".to_string(),
            serde_json::Value::Number(data_size.into()),
        );

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Decryption,
            severity: if matches!(result, OperationResult::Success) {
                AuditSeverity::Info
            } else {
                AuditSeverity::Error
            },
            component: "crypto_decryption".to_string(),
            operation: operation.to_string(),
            actor: None,
            result,
            duration: Some(duration),
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Log a key operation (generation, derivation, etc.)
    pub async fn log_key_operation(
        &self,
        operation: &str,
        key_type: &str,
        duration: Duration,
        result: OperationResult,
        correlation_id: Option<Uuid>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert(
            "key_type".to_string(),
            serde_json::Value::String(key_type.to_string()),
        );

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::KeyOperation,
            severity: if matches!(result, OperationResult::Success) {
                AuditSeverity::Info
            } else {
                AuditSeverity::Error
            },
            component: "crypto_key_ops".to_string(),
            operation: operation.to_string(),
            actor: None,
            result,
            duration: Some(duration),
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Log a backup or restore operation
    #[allow(clippy::too_many_arguments)]
    pub async fn log_backup_restore_operation(
        &self,
        operation: &str,
        backup_path: &str,
        items_processed: u64,
        total_size: u64,
        duration: Duration,
        result: OperationResult,
        correlation_id: Option<Uuid>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert(
            "backup_path".to_string(),
            serde_json::Value::String(backup_path.to_string()),
        );
        metadata.insert(
            "items_processed".to_string(),
            serde_json::Value::Number(items_processed.into()),
        );
        metadata.insert(
            "total_size".to_string(),
            serde_json::Value::Number(total_size.into()),
        );

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::BackupRestore,
            severity: if matches!(result, OperationResult::Success) {
                AuditSeverity::Info
            } else {
                AuditSeverity::Error
            },
            component: "crypto_backup".to_string(),
            operation: operation.to_string(),
            actor: None,
            result,
            duration: Some(duration),
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Log a security event
    pub async fn log_security_event(
        &self,
        operation: &str,
        security_details: SecurityEventDetails,
        result: OperationResult,
        correlation_id: Option<Uuid>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert(
            "event_type".to_string(),
            serde_json::Value::String(security_details.event_type),
        );
        metadata.insert(
            "threat_level".to_string(),
            serde_json::Value::String(security_details.threat_level),
        );

        if let Some(source) = security_details.source {
            metadata.insert("source".to_string(), serde_json::Value::String(source));
        }

        if let Some(target) = security_details.target {
            metadata.insert("target".to_string(), serde_json::Value::String(target));
        }

        // Add security metadata
        for (key, value) in security_details.security_metadata {
            metadata.insert(key, serde_json::Value::String(value));
        }

        let severity = match result {
            OperationResult::Success => AuditSeverity::Warning,
            OperationResult::Failure { .. } => AuditSeverity::Critical,
            _ => AuditSeverity::Error,
        };

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Security,
            severity,
            component: "crypto_security".to_string(),
            operation: operation.to_string(),
            actor: None,
            result,
            duration: None,
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Log a performance metrics event
    pub async fn log_performance_metrics(
        &self,
        operation: &str,
        metrics: PerformanceMetrics,
        correlation_id: Option<Uuid>,
    ) {
        let mut metadata = HashMap::new();

        if let Some(cpu_time) = metrics.cpu_time {
            metadata.insert(
                "cpu_time_ms".to_string(),
                serde_json::Value::Number((cpu_time.as_millis() as u64).into()),
            );
        }

        if let Some(memory_usage) = metrics.memory_usage {
            metadata.insert(
                "memory_usage".to_string(),
                serde_json::Value::Number(memory_usage.into()),
            );
        }

        if let Some(data_size) = metrics.data_size {
            metadata.insert(
                "data_size".to_string(),
                serde_json::Value::Number(data_size.into()),
            );
        }

        if let Some(throughput) = metrics.throughput {
            metadata.insert("throughput".to_string(), serde_json::json!(throughput));
        }

        // Add custom metrics
        for (key, value) in metrics.custom_metrics {
            metadata.insert(key, serde_json::json!(value));
        }

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Performance,
            severity: AuditSeverity::Info,
            component: "crypto_performance".to_string(),
            operation: operation.to_string(),
            actor: None,
            result: OperationResult::Success,
            duration: None,
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Log an error event from an EnhancedCryptoError
    pub async fn log_error_event(&self, error: &EnhancedCryptoError, correlation_id: Option<Uuid>) {
        let mut metadata = HashMap::new();
        metadata.insert(
            "error_type".to_string(),
            serde_json::Value::String(error.error_type_name().to_string()),
        );
        metadata.insert("severity".to_string(), serde_json::json!(error.severity()));
        metadata.insert(
            "recovery_actions".to_string(),
            serde_json::json!(error.recovery_actions()),
        );

        // Add error context metadata
        for (key, value) in &error.context().metadata {
            metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        let audit_severity = match error.severity() {
            ErrorSeverity::Low => AuditSeverity::Info,
            ErrorSeverity::Medium => AuditSeverity::Warning,
            ErrorSeverity::High => AuditSeverity::Error,
            ErrorSeverity::Critical => AuditSeverity::Critical,
        };

        let result = OperationResult::Failure {
            error_type: error.error_type_name().to_string(),
            error_message: error.to_string(),
            error_code: None,
        };

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Security, // Errors are treated as security events
            severity: audit_severity,
            component: error.context().component.clone(),
            operation: error.context().operation.clone(),
            actor: None,
            result,
            duration: None,
            metadata,
            correlation_id,
            session_id: Some(self.session_id.clone()),
        };

        self.log_event(event).await;
    }

    /// Get current audit statistics
    pub async fn get_statistics(&self) -> AuditStatistics {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    /// Get recent audit events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Filter events by criteria
    pub async fn filter_events(
        &self,
        event_type: Option<AuditEventType>,
        severity: Option<AuditSeverity>,
        component: Option<String>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<AuditEvent> {
        let events = self.events.read().await;

        events
            .iter()
            .filter(|event| {
                if let Some(ref et) = event_type {
                    if &event.event_type != et {
                        return false;
                    }
                }

                if let Some(ref sev) = severity {
                    if &event.severity != sev {
                        return false;
                    }
                }

                if let Some(ref comp) = component {
                    if event.component != *comp {
                        return false;
                    }
                }

                if let Some(start) = start_time {
                    if event.timestamp < start {
                        return false;
                    }
                }

                if let Some(end) = end_time {
                    if event.timestamp > end {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Export audit events to JSON format
    pub async fn export_events_json(&self) -> serde_json::Result<String> {
        let events = self.events.read().await;
        serde_json::to_string_pretty(&*events)
    }

    /// Clear all stored events (use with caution)
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();

        let mut stats = self.statistics.write().await;
        *stats = AuditStatistics {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            failed_operations: 0,
            security_events: 0,
            avg_operation_duration: None,
            last_event_time: None,
        };
    }

    // Private methods

    /// Internal method to log an event
    async fn log_event(&self, event: AuditEvent) {
        if !self.config.enabled {
            return;
        }

        // Check if event type should be logged
        if !self.config.included_event_types.is_empty()
            && !self.config.included_event_types.contains(&event.event_type)
        {
            return;
        }

        if self.config.excluded_event_types.contains(&event.event_type) {
            return;
        }

        // Update statistics
        self.update_statistics(&event).await;

        // Store event in memory
        let mut events = self.events.write().await;
        events.push(event.clone());

        // Manage memory usage
        if events.len() > self.config.max_events_in_memory {
            events.remove(0);
        }

        // Log to structured output if enabled
        if self.config.enable_json_output {
            self.log_to_structured_output(&event).await;
        }

        // Check for alerting conditions
        if self.config.enable_alerting && self.should_alert(&event) {
            self.send_alert(&event).await;
        }
    }

    /// Update audit statistics
    async fn update_statistics(&self, event: &AuditEvent) {
        let mut stats = self.statistics.write().await;

        stats.total_events += 1;

        *stats
            .events_by_type
            .entry(event.event_type.clone())
            .or_insert(0) += 1;
        *stats.events_by_severity.entry(event.severity).or_insert(0) += 1;

        if matches!(event.result, OperationResult::Failure { .. }) {
            stats.failed_operations += 1;
        }

        if event.event_type == AuditEventType::Security {
            stats.security_events += 1;
        }

        stats.last_event_time = Some(event.timestamp);

        // Update average duration if available
        if let Some(duration) = event.duration {
            if let Some(current_avg) = stats.avg_operation_duration {
                stats.avg_operation_duration = Some(Duration::from_millis(
                    (current_avg.as_millis() + duration.as_millis()) as u64 / 2,
                ));
            } else {
                stats.avg_operation_duration = Some(duration);
            }
        }
    }

    /// Log event to structured output
    async fn log_to_structured_output(&self, event: &AuditEvent) {
        // Use the existing logging infrastructure
        let json_str = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
        log::info!(target: "crypto_audit", "{}", json_str);
    }

    /// Check if event should trigger an alert
    fn should_alert(&self, event: &AuditEvent) -> bool {
        matches!(
            event.severity,
            AuditSeverity::Critical | AuditSeverity::Error
        ) || event.event_type == AuditEventType::Security
    }

    /// Send alert for critical events
    async fn send_alert(&self, event: &AuditEvent) {
        // For now, just log the alert
        log::warn!(
            target: "crypto_audit_alert",
            "CRYPTO AUDIT ALERT: {:?} - {} - {}",
            event.severity,
            event.component,
            event.operation
        );
    }
}

/// Global audit logger instance
static mut GLOBAL_AUDIT_LOGGER: Option<Arc<CryptoAuditLogger>> = None;
static AUDIT_LOGGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global audit logger
pub fn init_global_audit_logger(config: AuditConfig) {
    AUDIT_LOGGER_INIT.call_once(|| {
        let logger = Arc::new(CryptoAuditLogger::new(config));
        unsafe {
            GLOBAL_AUDIT_LOGGER = Some(logger);
        }
    });
}

/// Get the global audit logger instance
pub fn get_global_audit_logger() -> Option<Arc<CryptoAuditLogger>> {
    #[allow(static_mut_refs)]
    unsafe {
        GLOBAL_AUDIT_LOGGER.as_ref().map(Arc::clone)
    }
}

/// Convenience function to log an encryption operation globally
pub async fn audit_encryption_operation(
    operation: &str,
    context: &str,
    data_size: usize,
    duration: Duration,
    result: OperationResult,
    correlation_id: Option<Uuid>,
) {
    if let Some(logger) = get_global_audit_logger() {
        logger
            .log_encryption_operation(
                operation,
                context,
                data_size,
                duration,
                result,
                correlation_id,
            )
            .await;
    }
}

/// Convenience function to log a decryption operation globally
pub async fn audit_decryption_operation(
    operation: &str,
    context: &str,
    data_size: usize,
    duration: Duration,
    result: OperationResult,
    correlation_id: Option<Uuid>,
) {
    if let Some(logger) = get_global_audit_logger() {
        logger
            .log_decryption_operation(
                operation,
                context,
                data_size,
                duration,
                result,
                correlation_id,
            )
            .await;
    }
}

/// Convenience function to log a security event globally
pub async fn audit_security_event(
    operation: &str,
    security_details: SecurityEventDetails,
    result: OperationResult,
    correlation_id: Option<Uuid>,
) {
    if let Some(logger) = get_global_audit_logger() {
        logger
            .log_security_event(operation, security_details, result, correlation_id)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = CryptoAuditLogger::new(config);

        let stats = logger.get_statistics().await;
        assert_eq!(stats.total_events, 0);
    }

    #[tokio::test]
    async fn test_encryption_operation_logging() {
        let config = AuditConfig::default();
        let logger = CryptoAuditLogger::new(config);

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
    }

    #[tokio::test]
    async fn test_security_event_logging() {
        let config = AuditConfig::default();
        let logger = CryptoAuditLogger::new(config);

        let security_details = SecurityEventDetails {
            event_type: "unauthorized_access".to_string(),
            threat_level: "high".to_string(),
            source: Some("192.168.1.100".to_string()),
            target: Some("encryption_key".to_string()),
            security_metadata: HashMap::new(),
        };

        logger
            .log_security_event(
                "access_attempt",
                security_details,
                OperationResult::Failure {
                    error_type: "unauthorized".to_string(),
                    error_message: "Access denied".to_string(),
                    error_code: Some("AUTH001".to_string()),
                },
                None,
            )
            .await;

        let stats = logger.get_statistics().await;
        assert_eq!(stats.security_events, 1);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let config = AuditConfig::default();
        let logger = CryptoAuditLogger::new(config);

        // Log multiple events
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

        // Filter by event type
        let encryption_events = logger
            .filter_events(Some(AuditEventType::Encryption), None, None, None, None)
            .await;

        assert_eq!(encryption_events.len(), 1);
        assert_eq!(encryption_events[0].operation, "encrypt1");
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let config = AuditConfig::default();
        let logger = CryptoAuditLogger::new(config);

        // Log successful and failed operations
        logger
            .log_encryption_operation(
                "encrypt_success",
                "context1",
                100,
                Duration::from_millis(10),
                OperationResult::Success,
                None,
            )
            .await;

        logger
            .log_encryption_operation(
                "encrypt_fail",
                "context1",
                100,
                Duration::from_millis(20),
                OperationResult::Failure {
                    error_type: "test".to_string(),
                    error_message: "test error".to_string(),
                    error_code: None,
                },
                None,
            )
            .await;

        let stats = logger.get_statistics().await;
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.failed_operations, 1);
        assert!(stats.avg_operation_duration.is_some());
    }
}
