//! Event type definitions for the Security Operations Event Bus
//!
//! This module defines all event types that can be published through the centralized
//! verification event bus, supporting cross-platform security monitoring and correlation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Security event categories for verification monitoring
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityEventCategory {
    /// Authentication-related events (login, key validation, etc.)
    Authentication,
    /// Authorization events (permission checks, access control)
    Authorization,
    /// Configuration change events (policy updates, environment switches)
    Configuration,
    /// Performance monitoring events (timing, throughput, errors)
    Performance,
    /// Security threat detection events (anomalies, attacks)
    Security,
    /// Verification operation events (signature validation, crypto operations)
    Verification,
    /// System lifecycle events (startup, shutdown, initialization)
    System,
}

impl std::fmt::Display for SecurityEventCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventCategory::Authentication => write!(f, "Authentication"),
            SecurityEventCategory::Authorization => write!(f, "Authorization"),
            SecurityEventCategory::Configuration => write!(f, "Configuration"),
            SecurityEventCategory::Performance => write!(f, "Performance"),
            SecurityEventCategory::Security => write!(f, "Security"),
            SecurityEventCategory::Verification => write!(f, "Verification"),
            SecurityEventCategory::System => write!(f, "System"),
        }
    }
}

/// Event severity levels aligned with security operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Informational events for normal operations
    Info,
    /// Warning events for potential issues
    Warning,
    /// Error events for failed operations
    Error,
    /// Critical events requiring immediate attention
    Critical,
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Info => write!(f, "INFO"),
            EventSeverity::Warning => write!(f, "WARNING"),
            EventSeverity::Error => write!(f, "ERROR"),
            EventSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Platform source for cross-platform event correlation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlatformSource {
    /// Rust CLI application
    RustCli,
    /// JavaScript SDK
    JavaScriptSdk,
    /// Python SDK
    PythonSdk,
    /// DataFold Node server
    DataFoldNode,
    /// Unknown or custom platform
    Other(String),
}

impl std::fmt::Display for PlatformSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformSource::RustCli => write!(f, "RustCli"),
            PlatformSource::JavaScriptSdk => write!(f, "JavaScriptSdk"),
            PlatformSource::PythonSdk => write!(f, "PythonSdk"),
            PlatformSource::DataFoldNode => write!(f, "DataFoldNode"),
            PlatformSource::Other(name) => write!(f, "Other({})", name),
        }
    }
}

/// Core verification event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Event timestamp in UTC
    pub timestamp: DateTime<Utc>,
    /// Event category
    pub category: SecurityEventCategory,
    /// Event severity level
    pub severity: EventSeverity,
    /// Platform that generated the event
    pub platform: PlatformSource,
    /// Component within the platform
    pub component: String,
    /// Operation being performed
    pub operation: String,
    /// Actor performing the operation (user, service, etc.)
    pub actor: Option<String>,
    /// Operation result
    pub result: OperationResult,
    /// Operation duration if applicable
    pub duration: Option<Duration>,
    /// Event-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Correlation ID for cross-platform tracing
    pub correlation_id: Option<Uuid>,
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
    /// Session or transaction identifier
    pub session_id: Option<String>,
    /// Environment where event occurred
    pub environment: Option<String>,
}

/// Result of an operation for event tracking
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
    /// Operation was cancelled or aborted
    Cancelled,
    /// Operation is still in progress
    InProgress,
    /// Operation timed out
    Timeout,
}

/// Authentication event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// Type of authentication operation
    pub auth_type: String,
    /// Authentication method used
    pub method: String,
    /// Key identifier if applicable
    pub key_id: Option<String>,
    /// Source IP address if available
    pub source_ip: Option<String>,
    /// User agent if available
    pub user_agent: Option<String>,
    /// Whether MFA was used
    pub mfa_used: bool,
}

/// Authorization event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// Resource being accessed
    pub resource: String,
    /// Action being performed
    pub action: String,
    /// Policy that was evaluated
    pub policy: Option<String>,
    /// Access decision (allow/deny)
    pub decision: String,
    /// Reason for the decision
    pub reason: Option<String>,
}

/// Configuration change event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// Type of configuration change
    pub change_type: String,
    /// Configuration section affected
    pub section: String,
    /// Previous value (if available)
    pub previous_value: Option<serde_json::Value>,
    /// New value
    pub new_value: serde_json::Value,
    /// Configuration source
    pub source: String,
}

/// Performance monitoring event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// Metric name
    pub metric: String,
    /// Metric value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Performance threshold if applicable
    pub threshold: Option<f64>,
    /// Whether threshold was exceeded
    pub threshold_exceeded: bool,
    /// Additional performance metrics
    pub metrics: HashMap<String, f64>,
}

/// Security threat event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityThreatEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// Type of threat detected
    pub threat_type: String,
    /// Threat level assessment
    pub threat_level: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Source of the threat
    pub threat_source: Option<String>,
    /// Target of the threat
    pub target: Option<String>,
    /// Evidence supporting the detection
    pub evidence: HashMap<String, serde_json::Value>,
    /// Recommended response actions
    pub recommended_actions: Vec<String>,
    /// Whether automated response was triggered
    pub auto_response_triggered: bool,
}

/// Verification operation event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationOperationEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// Type of verification operation
    pub verification_type: String,
    /// Algorithm used
    pub algorithm: Option<String>,
    /// Key identifier
    pub key_id: Option<String>,
    /// Data size processed
    pub data_size: Option<usize>,
    /// Verification result details
    pub verification_result: VerificationResult,
    /// Performance metrics
    pub performance_metrics: Option<VerificationMetrics>,
}

/// Detailed verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification succeeded
    pub valid: bool,
    /// Signature components verified
    pub verified_components: Vec<String>,
    /// Failed components if any
    pub failed_components: Vec<String>,
    /// Verification error if any
    pub error: Option<String>,
    /// Additional result details
    pub details: HashMap<String, serde_json::Value>,
}

/// Performance metrics for verification operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMetrics {
    /// CPU time used
    pub cpu_time: Option<Duration>,
    /// Memory usage peak
    pub memory_usage: Option<u64>,
    /// Throughput (operations per second)
    pub throughput: Option<f64>,
    /// Custom performance counters
    pub custom_metrics: HashMap<String, f64>,
}

/// System lifecycle event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Base verification event
    pub base: VerificationEvent,
    /// System event type (startup, shutdown, etc.)
    pub system_event_type: String,
    /// System version
    pub version: Option<String>,
    /// Configuration applied
    pub configuration: Option<String>,
    /// System state information
    pub state: HashMap<String, serde_json::Value>,
}

/// Unified event wrapper for all event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// Authentication event
    Authentication(AuthenticationEvent),
    /// Authorization event
    Authorization(AuthorizationEvent),
    /// Configuration change event
    Configuration(ConfigurationEvent),
    /// Performance monitoring event
    Performance(PerformanceEvent),
    /// Security threat event
    Security(SecurityThreatEvent),
    /// Verification operation event
    Verification(VerificationOperationEvent),
    /// System lifecycle event
    System(SystemEvent),
    /// Generic verification event
    Generic(VerificationEvent),
}

impl SecurityEvent {
    /// Get the base verification event from any security event
    pub fn base_event(&self) -> &VerificationEvent {
        match self {
            SecurityEvent::Authentication(e) => &e.base,
            SecurityEvent::Authorization(e) => &e.base,
            SecurityEvent::Configuration(e) => &e.base,
            SecurityEvent::Performance(e) => &e.base,
            SecurityEvent::Security(e) => &e.base,
            SecurityEvent::Verification(e) => &e.base,
            SecurityEvent::System(e) => &e.base,
            SecurityEvent::Generic(e) => e,
        }
    }

    /// Get the event category
    pub fn category(&self) -> SecurityEventCategory {
        self.base_event().category.clone()
    }

    /// Get the event severity
    pub fn severity(&self) -> EventSeverity {
        self.base_event().severity
    }

    /// Get the platform source
    pub fn platform(&self) -> &PlatformSource {
        &self.base_event().platform
    }

    /// Get the correlation ID if present
    pub fn correlation_id(&self) -> Option<Uuid> {
        self.base_event().correlation_id
    }

    /// Get the trace ID if present
    pub fn trace_id(&self) -> Option<&String> {
        self.base_event().trace_id.as_ref()
    }

    /// Check if this event should trigger an alert based on severity
    pub fn should_alert(&self, minimum_severity: EventSeverity) -> bool {
        match (minimum_severity, self.severity()) {
            (EventSeverity::Info, _) => true,
            (EventSeverity::Warning, EventSeverity::Info) => false,
            (EventSeverity::Warning, _) => true,
            (EventSeverity::Error, EventSeverity::Info | EventSeverity::Warning) => false,
            (EventSeverity::Error, _) => true,
            (EventSeverity::Critical, EventSeverity::Critical) => true,
            (EventSeverity::Critical, _) => false,
        }
    }
}

/// Helper trait for creating verification events
pub trait CreateVerificationEvent {
    /// Create a new verification event with common fields
    fn create_base_event(
        category: SecurityEventCategory,
        severity: EventSeverity,
        platform: PlatformSource,
        component: String,
        operation: String,
    ) -> VerificationEvent {
        VerificationEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            severity,
            platform,
            component,
            operation,
            actor: None,
            result: OperationResult::InProgress,
            duration: None,
            metadata: HashMap::new(),
            correlation_id: None,
            trace_id: None,
            session_id: None,
            environment: None,
        }
    }
}

impl CreateVerificationEvent for VerificationEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_event_creation() {
        let event = VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            EventSeverity::Info,
            PlatformSource::RustCli,
            "auth_handler".to_string(),
            "login".to_string(),
        );

        assert_eq!(event.category, SecurityEventCategory::Authentication);
        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.platform, PlatformSource::RustCli);
        assert_eq!(event.component, "auth_handler");
        assert_eq!(event.operation, "login");
    }

    #[test]
    fn test_security_event_methods() {
        let base = VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            EventSeverity::Critical,
            PlatformSource::DataFoldNode,
            "security_monitor".to_string(),
            "threat_detected".to_string(),
        );

        let security_event = SecurityEvent::Generic(base);

        assert_eq!(security_event.category(), SecurityEventCategory::Security);
        assert_eq!(security_event.severity(), EventSeverity::Critical);
        assert!(security_event.should_alert(EventSeverity::Warning));
        assert!(security_event.should_alert(EventSeverity::Critical));
    }

    #[test]
    fn test_alert_threshold_logic() {
        let info_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Performance,
            EventSeverity::Info,
            PlatformSource::JavaScriptSdk,
            "perf_monitor".to_string(),
            "metric_update".to_string(),
        ));

        let critical_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            EventSeverity::Critical,
            PlatformSource::PythonSdk,
            "security_monitor".to_string(),
            "attack_detected".to_string(),
        ));

        // Info event should not trigger critical alerts
        assert!(!info_event.should_alert(EventSeverity::Critical));
        assert!(info_event.should_alert(EventSeverity::Info));

        // Critical event should trigger all alert levels
        assert!(critical_event.should_alert(EventSeverity::Info));
        assert!(critical_event.should_alert(EventSeverity::Warning));
        assert!(critical_event.should_alert(EventSeverity::Error));
        assert!(critical_event.should_alert(EventSeverity::Critical));
    }
}