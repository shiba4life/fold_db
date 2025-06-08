//! Security monitoring and threat detection for cryptographic operations
//!
//! This module provides real-time security monitoring, anomaly detection,
//! and alerting for encryption-related security events.

use super::audit_logger::{SecurityEventDetails, OperationResult, CryptoAuditLogger, AuditEventType};
use super::enhanced_error::{EnhancedCryptoError, ErrorSeverity};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Security threat levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Low threat - informational
    Low,
    /// Medium threat - suspicious activity
    Medium,
    /// High threat - probable attack
    High,
    /// Critical threat - active attack detected
    Critical,
}

/// Types of security patterns to monitor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum SecurityPattern {
    /// Repeated failed decryption attempts
    RepeatedDecryptionFailures,
    /// Unusual encryption patterns
    UnusualEncryptionPattern,
    /// Key usage anomalies
    KeyUsageAnomaly,
    /// Performance degradation attacks
    PerformanceDegradation,
    /// Timing attack attempts
    TimingAttack,
    /// Memory exhaustion attempts
    MemoryExhaustion,
    /// Unauthorized key access
    UnauthorizedKeyAccess,
    /// Data corruption detection
    DataCorruption,
    /// Backup tampering
    BackupTampering,
    /// Configuration manipulation
    ConfigurationTampering,
}

/// Security event detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDetection {
    /// Unique detection ID
    pub detection_id: Uuid,
    /// Timestamp of detection
    pub timestamp: DateTime<Utc>,
    /// Pattern that was detected
    pub pattern: SecurityPattern,
    /// Threat level assessment
    pub threat_level: ThreatLevel,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Source of the detection
    pub source: String,
    /// Description of what was detected
    pub description: String,
    /// Evidence supporting the detection
    pub evidence: HashMap<String, serde_json::Value>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Configuration for security monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitorConfig {
    /// Enable security monitoring
    pub enabled: bool,
    /// Window size for pattern detection (minutes)
    pub detection_window_minutes: u64,
    /// Threshold for repeated failures
    pub failure_threshold: u32,
    /// Threshold for unusual patterns
    pub anomaly_threshold: f64,
    /// Enable real-time alerting
    pub enable_alerting: bool,
    /// Enable automatic response to threats
    pub enable_auto_response: bool,
    /// Patterns to monitor
    pub monitored_patterns: Vec<SecurityPattern>,
    /// Minimum threat level for alerts
    pub alert_threshold: ThreatLevel,
}

impl Default for SecurityMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detection_window_minutes: 10,
            failure_threshold: 5,
            anomaly_threshold: 0.7,
            enable_alerting: true,
            enable_auto_response: false,
            monitored_patterns: vec![
                SecurityPattern::RepeatedDecryptionFailures,
                SecurityPattern::UnusualEncryptionPattern,
                SecurityPattern::KeyUsageAnomaly,
                SecurityPattern::PerformanceDegradation,
                SecurityPattern::UnauthorizedKeyAccess,
                SecurityPattern::DataCorruption,
            ],
            alert_threshold: ThreatLevel::Medium,
        }
    }
}

/// Statistics for security monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatistics {
    /// Total number of security events processed
    pub total_events_processed: u64,
    /// Number of threats detected
    pub threats_detected: u64,
    /// Threats by level
    pub threats_by_level: HashMap<ThreatLevel, u64>,
    /// Patterns detected
    pub patterns_detected: HashMap<SecurityPattern, u64>,
    /// False positive rate
    pub false_positive_rate: f64,
    /// Last detection timestamp
    pub last_detection: Option<DateTime<Utc>>,
    /// Average detection time
    pub avg_detection_time: Option<Duration>,
}

/// Pattern tracking for anomaly detection
#[derive(Debug, Clone)]
struct PatternTracker {
    /// Events in the current window
    events: Vec<SecurityEvent>,
    /// Last cleanup time
    last_cleanup: Instant,
    /// Pattern-specific state
    state: HashMap<String, serde_json::Value>,
}

impl PatternTracker {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            last_cleanup: Instant::now(),
            state: HashMap::new(),
        }
    }

    fn add_event(&mut self, event: SecurityEvent, window_duration: Duration) {
        // Clean up old events
        let cutoff = Instant::now() - window_duration;
        self.events.retain(|e| e.timestamp > cutoff);
        
        // Add new event
        self.events.push(event);
        self.last_cleanup = Instant::now();
    }

    fn get_events_in_window(&self, window_duration: Duration) -> Vec<&SecurityEvent> {
        let cutoff = Instant::now() - window_duration;
        self.events.iter().filter(|e| e.timestamp > cutoff).collect()
    }
}

/// Internal security event representation
#[derive(Debug, Clone)]
struct SecurityEvent {
    timestamp: Instant,
    event_type: String,
    component: String,
    operation: String,
    success: bool,
    metadata: HashMap<String, String>,
}

/// Main security monitor for cryptographic operations
pub struct CryptoSecurityMonitor {
    /// Configuration
    config: SecurityMonitorConfig,
    /// Pattern trackers for different event types
    pattern_trackers: Arc<RwLock<HashMap<String, PatternTracker>>>,
    /// Security statistics
    statistics: Arc<RwLock<SecurityStatistics>>,
    /// Detected threats
    detections: Arc<RwLock<Vec<SecurityDetection>>>,
    /// Audit logger for security events
    audit_logger: Option<Arc<CryptoAuditLogger>>,
}

impl CryptoSecurityMonitor {
    /// Create a new security monitor
    pub fn new(config: SecurityMonitorConfig) -> Self {
        Self {
            config,
            pattern_trackers: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(SecurityStatistics {
                total_events_processed: 0,
                threats_detected: 0,
                threats_by_level: HashMap::new(),
                patterns_detected: HashMap::new(),
                false_positive_rate: 0.0,
                last_detection: None,
                avg_detection_time: None,
            })),
            detections: Arc::new(RwLock::new(Vec::new())),
            audit_logger: None,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(SecurityMonitorConfig::default())
    }

    /// Set audit logger for security event logging
    pub fn with_audit_logger(mut self, audit_logger: Arc<CryptoAuditLogger>) -> Self {
        self.audit_logger = Some(audit_logger);
        self
    }

    /// Process an encryption operation for security monitoring
    pub async fn monitor_encryption_operation(
        &self,
        operation: &str,
        context: &str,
        data_size: usize,
        duration: Duration,
        success: bool,
        source: Option<&str>,
    ) -> Vec<SecurityDetection> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut metadata = HashMap::new();
        metadata.insert("context".to_string(), context.to_string());
        metadata.insert("data_size".to_string(), data_size.to_string());
        metadata.insert("duration_ms".to_string(), duration.as_millis().to_string());
        if let Some(src) = source {
            metadata.insert("source".to_string(), src.to_string());
        }

        let event = SecurityEvent {
            timestamp: Instant::now(),
            event_type: "encryption".to_string(),
            component: "crypto_encryption".to_string(),
            operation: operation.to_string(),
            success,
            metadata,
        };

        self.process_security_event(event).await
    }

    /// Process a decryption operation for security monitoring
    pub async fn monitor_decryption_operation(
        &self,
        operation: &str,
        context: &str,
        data_size: usize,
        duration: Duration,
        success: bool,
        source: Option<&str>,
    ) -> Vec<SecurityDetection> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut metadata = HashMap::new();
        metadata.insert("context".to_string(), context.to_string());
        metadata.insert("data_size".to_string(), data_size.to_string());
        metadata.insert("duration_ms".to_string(), duration.as_millis().to_string());
        if let Some(src) = source {
            metadata.insert("source".to_string(), src.to_string());
        }

        let event = SecurityEvent {
            timestamp: Instant::now(),
            event_type: "decryption".to_string(),
            component: "crypto_decryption".to_string(),
            operation: operation.to_string(),
            success,
            metadata,
        };

        self.process_security_event(event).await
    }

    /// Process a key operation for security monitoring
    pub async fn monitor_key_operation(
        &self,
        operation: &str,
        key_type: &str,
        duration: Duration,
        success: bool,
        source: Option<&str>,
    ) -> Vec<SecurityDetection> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut metadata = HashMap::new();
        metadata.insert("key_type".to_string(), key_type.to_string());
        metadata.insert("duration_ms".to_string(), duration.as_millis().to_string());
        if let Some(src) = source {
            metadata.insert("source".to_string(), src.to_string());
        }

        let event = SecurityEvent {
            timestamp: Instant::now(),
            event_type: "key_operation".to_string(),
            component: "crypto_key_ops".to_string(),
            operation: operation.to_string(),
            success,
            metadata,
        };

        self.process_security_event(event).await
    }

    /// Process an enhanced crypto error for security monitoring
    pub async fn monitor_crypto_error(&self, error: &EnhancedCryptoError) -> Vec<SecurityDetection> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut metadata = HashMap::new();
        metadata.insert("error_type".to_string(), error.error_type_name().to_string());
        metadata.insert("severity".to_string(), format!("{:?}", error.severity()));
        
        // Add error context metadata
        for (key, value) in &error.context().metadata {
            metadata.insert(format!("ctx_{}", key), value.clone());
        }

        let event = SecurityEvent {
            timestamp: Instant::now(),
            event_type: "crypto_error".to_string(),
            component: error.context().component.clone(),
            operation: error.context().operation.clone(),
            success: false,
            metadata,
        };

        let detections = self.process_security_event(event).await;

        // Log security events if we have an audit logger
        if let Some(audit_logger) = &self.audit_logger {
            for detection in &detections {
                let security_details = SecurityEventDetails {
                    event_type: format!("{:?}", detection.pattern),
                    threat_level: format!("{:?}", detection.threat_level),
                    source: Some(detection.source.clone()),
                    target: None,
                    security_metadata: detection.evidence.iter()
                        .map(|(k, v)| (k.clone(), v.to_string()))
                        .collect(),
                };

                audit_logger.log_security_event(
                    &format!("threat_detected_{:?}", detection.pattern),
                    security_details,
                    OperationResult::Success,
                    Some(detection.detection_id),
                ).await;
            }
        }

        detections
    }

    /// Get current security statistics
    pub async fn get_statistics(&self) -> SecurityStatistics {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    /// Get recent security detections
    pub async fn get_recent_detections(&self, limit: usize) -> Vec<SecurityDetection> {
        let detections = self.detections.read().await;
        detections.iter().rev().take(limit).cloned().collect()
    }

    /// Get detections by threat level
    pub async fn get_detections_by_threat_level(&self, threat_level: ThreatLevel) -> Vec<SecurityDetection> {
        let detections = self.detections.read().await;
        detections.iter()
            .filter(|d| d.threat_level == threat_level)
            .cloned()
            .collect()
    }

    /// Clear all detections (use with caution)
    pub async fn clear_detections(&self) {
        let mut detections = self.detections.write().await;
        detections.clear();
        
        let mut stats = self.statistics.write().await;
        stats.threats_detected = 0;
        stats.threats_by_level.clear();
        stats.patterns_detected.clear();
        stats.last_detection = None;
    }

    // Private methods

    /// Process a security event and detect threats
    async fn process_security_event(&self, event: SecurityEvent) -> Vec<SecurityDetection> {
        let mut detections = Vec::new();

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_events_processed += 1;
        }

        // Add event to pattern trackers
        {
            let mut trackers = self.pattern_trackers.write().await;
            let tracker = trackers.entry(event.event_type.clone()).or_insert_with(PatternTracker::new);
            tracker.add_event(event.clone(), Duration::from_secs(self.config.detection_window_minutes * 60));
        }

        // Run pattern detection algorithms
        for pattern in &self.config.monitored_patterns {
            if let Some(detection) = self.detect_pattern(pattern, &event).await {
                detections.push(detection);
            }
        }

        // Update statistics with detections
        if !detections.is_empty() {
            let mut stats = self.statistics.write().await;
            stats.threats_detected += detections.len() as u64;
            stats.last_detection = Some(Utc::now());

            for detection in &detections {
                *stats.threats_by_level.entry(detection.threat_level.clone()).or_insert(0) += 1;
                *stats.patterns_detected.entry(detection.pattern.clone()).or_insert(0) += 1;
            }

            // Store detections
            let mut stored_detections = self.detections.write().await;
            stored_detections.extend(detections.clone());

            // Limit stored detections to prevent memory issues
            if stored_detections.len() > 10000 {
                stored_detections.drain(0..1000);
            }
        }

        // Send alerts if configured
        if self.config.enable_alerting {
            for detection in &detections {
                if self.should_alert(&detection.threat_level) {
                    self.send_alert(detection).await;
                }
            }
        }

        detections
    }

    /// Detect specific security patterns
    async fn detect_pattern(&self, pattern: &SecurityPattern, event: &SecurityEvent) -> Option<SecurityDetection> {
        match pattern {
            SecurityPattern::RepeatedDecryptionFailures => {
                self.detect_repeated_failures("decryption", event).await
            }
            SecurityPattern::UnusualEncryptionPattern => {
                self.detect_unusual_encryption_pattern(event).await
            }
            SecurityPattern::KeyUsageAnomaly => {
                self.detect_key_usage_anomaly(event).await
            }
            SecurityPattern::PerformanceDegradation => {
                self.detect_performance_degradation(event).await
            }
            SecurityPattern::UnauthorizedKeyAccess => {
                self.detect_unauthorized_key_access(event).await
            }
            SecurityPattern::DataCorruption => {
                self.detect_data_corruption(event).await
            }
            _ => None, // Other patterns not implemented yet
        }
    }

    /// Detect repeated failure patterns
    async fn detect_repeated_failures(&self, operation_type: &str, event: &SecurityEvent) -> Option<SecurityDetection> {
        if event.event_type != operation_type || event.success {
            return None;
        }

        let trackers = self.pattern_trackers.read().await;
        if let Some(tracker) = trackers.get(operation_type) {
            let window_duration = Duration::from_secs(self.config.detection_window_minutes * 60);
            let recent_events = tracker.get_events_in_window(window_duration);
            
            let failure_count = recent_events.iter()
                .filter(|e| !e.success && e.operation == event.operation)
                .count();

            if failure_count >= self.config.failure_threshold as usize {
                let mut evidence = HashMap::new();
                evidence.insert("failure_count".to_string(), serde_json::Value::Number(failure_count.into()));
                evidence.insert("threshold".to_string(), serde_json::Value::Number(self.config.failure_threshold.into()));
                evidence.insert("window_minutes".to_string(), serde_json::Value::Number(self.config.detection_window_minutes.into()));

                return Some(SecurityDetection {
                    detection_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    pattern: SecurityPattern::RepeatedDecryptionFailures,
                    threat_level: if failure_count > self.config.failure_threshold as usize * 2 {
                        ThreatLevel::High
                    } else {
                        ThreatLevel::Medium
                    },
                    confidence: 0.8,
                    source: event.metadata.get("source").cloned().unwrap_or("unknown".to_string()),
                    description: format!(
                        "Detected {} repeated {} failures in {} minutes",
                        failure_count, operation_type, self.config.detection_window_minutes
                    ),
                    evidence,
                    recommended_actions: vec![
                        "Investigate source of failures".to_string(),
                        "Check for brute force attacks".to_string(),
                        "Consider rate limiting".to_string(),
                    ],
                });
            }
        }

        None
    }

    /// Detect unusual encryption patterns
    async fn detect_unusual_encryption_pattern(&self, event: &SecurityEvent) -> Option<SecurityDetection> {
        if event.event_type != "encryption" {
            return None;
        }

        // Check for unusual data sizes
        if let Some(data_size_str) = event.metadata.get("data_size") {
            if let Ok(data_size) = data_size_str.parse::<usize>() {
                // Flag very large or very small data sizes as potentially suspicious
                if data_size > 100_000_000 || (data_size < 10 && data_size > 0) {
                    let mut evidence = HashMap::new();
                    evidence.insert("data_size".to_string(), serde_json::Value::Number(data_size.into()));
                    evidence.insert("threshold_exceeded".to_string(), serde_json::Value::Bool(true));

                    return Some(SecurityDetection {
                        detection_id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        pattern: SecurityPattern::UnusualEncryptionPattern,
                        threat_level: ThreatLevel::Low,
                        confidence: 0.6,
                        source: event.metadata.get("source").cloned().unwrap_or("unknown".to_string()),
                        description: format!("Unusual encryption data size: {} bytes", data_size),
                        evidence,
                        recommended_actions: vec![
                            "Verify data source".to_string(),
                            "Check for data exfiltration attempts".to_string(),
                        ],
                    });
                }
            }
        }

        None
    }

    /// Detect key usage anomalies
    async fn detect_key_usage_anomaly(&self, event: &SecurityEvent) -> Option<SecurityDetection> {
        if event.event_type != "key_operation" {
            return None;
        }

        // Check for excessive key generation
        let trackers = self.pattern_trackers.read().await;
        if let Some(tracker) = trackers.get("key_operation") {
            let window_duration = Duration::from_secs(300); // 5 minute window
            let recent_events = tracker.get_events_in_window(window_duration);
            
            let key_gen_count = recent_events.iter()
                .filter(|e| e.operation.contains("generate") || e.operation.contains("derive"))
                .count();

            if key_gen_count > 10 {
                let mut evidence = HashMap::new();
                evidence.insert("key_generation_count".to_string(), serde_json::Value::Number(key_gen_count.into()));
                evidence.insert("time_window_minutes".to_string(), serde_json::Value::Number(5.into()));

                return Some(SecurityDetection {
                    detection_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    pattern: SecurityPattern::KeyUsageAnomaly,
                    threat_level: ThreatLevel::Medium,
                    confidence: 0.7,
                    source: event.metadata.get("source").cloned().unwrap_or("unknown".to_string()),
                    description: format!("Excessive key generation: {} operations in 5 minutes", key_gen_count),
                    evidence,
                    recommended_actions: vec![
                        "Investigate key usage patterns".to_string(),
                        "Check for key exhaustion attacks".to_string(),
                        "Review key management policies".to_string(),
                    ],
                });
            }
        }

        None
    }

    /// Detect performance degradation attacks
    async fn detect_performance_degradation(&self, event: &SecurityEvent) -> Option<SecurityDetection> {
        if let Some(duration_str) = event.metadata.get("duration_ms") {
            if let Ok(duration_ms) = duration_str.parse::<u64>() {
                // Flag operations that take longer than 5 seconds
                if duration_ms > 5000 {
                    let mut evidence = HashMap::new();
                    evidence.insert("duration_ms".to_string(), serde_json::Value::Number(duration_ms.into()));
                    evidence.insert("threshold_ms".to_string(), serde_json::Value::Number(5000.into()));

                    return Some(SecurityDetection {
                        detection_id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        pattern: SecurityPattern::PerformanceDegradation,
                        threat_level: ThreatLevel::Medium,
                        confidence: 0.5,
                        source: event.metadata.get("source").cloned().unwrap_or("unknown".to_string()),
                        description: format!("Slow cryptographic operation: {} ms", duration_ms),
                        evidence,
                        recommended_actions: vec![
                            "Monitor system resources".to_string(),
                            "Check for denial of service attacks".to_string(),
                            "Optimize crypto operations".to_string(),
                        ],
                    });
                }
            }
        }

        None
    }

    /// Detect unauthorized key access attempts
    async fn detect_unauthorized_key_access(&self, event: &SecurityEvent) -> Option<SecurityDetection> {
        if event.event_type == "key_operation" && !event.success {
            let mut evidence = HashMap::new();
            evidence.insert("operation".to_string(), serde_json::Value::String(event.operation.clone()));
            evidence.insert("component".to_string(), serde_json::Value::String(event.component.clone()));

            return Some(SecurityDetection {
                detection_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                pattern: SecurityPattern::UnauthorizedKeyAccess,
                threat_level: ThreatLevel::High,
                confidence: 0.9,
                source: event.metadata.get("source").cloned().unwrap_or("unknown".to_string()),
                description: format!("Failed key operation: {}", event.operation),
                evidence,
                recommended_actions: vec![
                    "Investigate access attempt".to_string(),
                    "Review access controls".to_string(),
                    "Check for privilege escalation".to_string(),
                ],
            });
        }

        None
    }

    /// Detect data corruption
    async fn detect_data_corruption(&self, event: &SecurityEvent) -> Option<SecurityDetection> {
        if event.event_type == "decryption" && !event.success {
            // Additional checks for corruption vs. authentication failure would go here
            let operation_contains_corruption = event.operation.to_lowercase().contains("corrupt") ||
                event.operation.to_lowercase().contains("invalid") ||
                event.operation.to_lowercase().contains("tamper");

            if operation_contains_corruption {
                let mut evidence = HashMap::new();
                evidence.insert("operation".to_string(), serde_json::Value::String(event.operation.clone()));

                return Some(SecurityDetection {
                    detection_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    pattern: SecurityPattern::DataCorruption,
                    threat_level: ThreatLevel::High,
                    confidence: 0.8,
                    source: event.metadata.get("source").cloned().unwrap_or("unknown".to_string()),
                    description: "Potential data corruption detected during decryption".to_string(),
                    evidence,
                    recommended_actions: vec![
                        "Verify data integrity".to_string(),
                        "Check for tampering".to_string(),
                        "Restore from backup if necessary".to_string(),
                    ],
                });
            }
        }

        None
    }

    /// Check if a threat level should trigger an alert
    fn should_alert(&self, threat_level: &ThreatLevel) -> bool {
        match (&self.config.alert_threshold, threat_level) {
            (ThreatLevel::Low, _) => true,
            (ThreatLevel::Medium, ThreatLevel::Medium | ThreatLevel::High | ThreatLevel::Critical) => true,
            (ThreatLevel::High, ThreatLevel::High | ThreatLevel::Critical) => true,
            (ThreatLevel::Critical, ThreatLevel::Critical) => true,
            _ => false,
        }
    }

    /// Send an alert for a security detection
    async fn send_alert(&self, detection: &SecurityDetection) {
        // For now, just log the alert
        log::warn!(
            target: "crypto_security_alert",
            "SECURITY THREAT DETECTED: {:?} - {} - {} (Confidence: {:.2})",
            detection.threat_level,
            detection.pattern as u32,
            detection.description,
            detection.confidence
        );
    }
}

/// Global security monitor instance
static mut GLOBAL_SECURITY_MONITOR: Option<Arc<CryptoSecurityMonitor>> = None;
static SECURITY_MONITOR_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global security monitor
pub fn init_global_security_monitor(config: SecurityMonitorConfig) {
    SECURITY_MONITOR_INIT.call_once(|| {
        let monitor = Arc::new(CryptoSecurityMonitor::new(config));
        unsafe {
            GLOBAL_SECURITY_MONITOR = Some(monitor);
        }
    });
}

/// Get the global security monitor instance
pub fn get_global_security_monitor() -> Option<Arc<CryptoSecurityMonitor>> {
    unsafe { GLOBAL_SECURITY_MONITOR.as_ref().map(Arc::clone) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_security_monitor_creation() {
        let config = SecurityMonitorConfig::default();
        let monitor = CryptoSecurityMonitor::new(config);
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total_events_processed, 0);
    }

    #[tokio::test]
    async fn test_repeated_failure_detection() {
        let config = SecurityMonitorConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let monitor = CryptoSecurityMonitor::new(config);
        
        // Simulate repeated failures
        for _ in 0..4 {
            let detections = monitor.monitor_decryption_operation(
                "decrypt_data",
                "test_context",
                100,
                Duration::from_millis(10),
                false, // failure
                Some("test_source"),
            ).await;
            
            // Should detect threat on the 4th failure (exceeding threshold of 3)
            if !detections.is_empty() {
                assert_eq!(detections[0].pattern, SecurityPattern::RepeatedDecryptionFailures);
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_unusual_encryption_pattern_detection() {
        let monitor = CryptoSecurityMonitor::with_default_config();
        
        // Test with unusually large data size
        let detections = monitor.monitor_encryption_operation(
            "encrypt_data",
            "test_context",
            200_000_000, // Very large size
            Duration::from_millis(100),
            true,
            Some("test_source"),
        ).await;
        
        assert!(!detections.is_empty());
        assert_eq!(detections[0].pattern, SecurityPattern::UnusualEncryptionPattern);
    }

    #[tokio::test]
    async fn test_performance_degradation_detection() {
        let monitor = CryptoSecurityMonitor::with_default_config();
        
        let detections = monitor.monitor_encryption_operation(
            "encrypt_data",
            "test_context",
            1000,
            Duration::from_millis(10000), // Very slow operation
            true,
            Some("test_source"),
        ).await;
        
        assert!(!detections.is_empty());
        assert_eq!(detections[0].pattern, SecurityPattern::PerformanceDegradation);
    }

    #[tokio::test]
    async fn test_unauthorized_key_access_detection() {
        let monitor = CryptoSecurityMonitor::with_default_config();
        
        let detections = monitor.monitor_key_operation(
            "access_private_key",
            "ed25519",
            Duration::from_millis(50),
            false, // Failed access
            Some("unauthorized_source"),
        ).await;
        
        assert!(!detections.is_empty());
        assert_eq!(detections[0].pattern, SecurityPattern::UnauthorizedKeyAccess);
        assert_eq!(detections[0].threat_level, ThreatLevel::High);
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let monitor = CryptoSecurityMonitor::with_default_config();
        
        // Generate some events
        monitor.monitor_encryption_operation(
            "encrypt1", "context1", 100, Duration::from_millis(10), true, None
        ).await;
        
        monitor.monitor_decryption_operation(
            "decrypt1", "context1", 100, Duration::from_millis(15), false, None
        ).await;
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total_events_processed, 2);
    }
}