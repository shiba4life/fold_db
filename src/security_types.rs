//! # Security Types Module
//!
//! This module provides unified security-related enum definitions that are used across
//! the DataFold codebase. It serves as the single source of truth for all security
//! type definitions, eliminating duplication and ensuring consistency.
//!
//! ## Core Types
//!
//! - [`RotationStatus`] - Status tracking for key rotation operations
//! - [`Severity`] - Universal severity levels for events, alerts, and errors
//! - [`SecurityLevel`] - Security configuration levels for crypto operations
//! - [`HealthStatus`] - System and component health status tracking
//!
//! ## Usage Examples
//!
//! ```rust
//! use datafold::security_types::{RotationStatus, Severity, SecurityLevel, HealthStatus};
//!
//! // Key rotation status tracking
//! let status = RotationStatus::InProgress;
//! if status.is_active() {
//!     println!("Rotation is currently active");
//! }
//!
//! // Event severity handling
//! let severity = Severity::Critical;
//! if severity.requires_immediate_action() {
//!     // Handle critical event
//! }
//!
//! // Security level configuration
//! let level = SecurityLevel::High;
//! let params = level.argon2_params();
//!
//! // Health status monitoring
//! let health = HealthStatus::Warning;
//! if !health.is_healthy() {
//!     // Take corrective action
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Status of key rotation operations
///
/// This enum tracks the lifecycle of key rotation operations across all modules.
/// It consolidates the previously conflicting definitions from crypto and db_operations
/// modules into a single, comprehensive status tracking system.
///
/// # State Transitions
///
/// ```text
/// Requested → Validating → InProgress → Completed
///     ↓           ↓            ↓           ↑
/// Cancelled   Cancelled    Failed → RolledBack
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationStatus {
    /// Rotation requested but not yet started
    Requested,
    /// Pre-rotation validation in progress
    Validating,
    /// Rotation operation in progress
    InProgress,
    /// Rotation completed successfully
    Completed,
    /// Rotation failed with error
    Failed,
    /// Rotation cancelled by user or system
    Cancelled,
    /// Rotation was rolled back due to issues
    RolledBack,
}

impl RotationStatus {
    /// Returns true if the rotation is currently active (not in a terminal state)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Requested | Self::Validating | Self::InProgress)
    }

    /// Returns true if the rotation completed successfully
    pub fn is_successful(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Returns true if the rotation failed or was cancelled
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled | Self::RolledBack)
    }

    /// Returns true if the rotation is in a terminal state (complete, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        !self.is_active()
    }
}

/// Universal severity levels for events, alerts, and errors
///
/// This enum provides a consistent 4-level severity system that replaces multiple
/// overlapping severity enums throughout the codebase including EventSeverity,
/// AlertSeverity, AuditSeverity, SecurityEventSeverity, and ErrorSeverity variants.
///
/// # Severity Guidelines
///
/// - **Info**: Normal operations, routine events
/// - **Warning**: Potential issues that should be monitored
/// - **Error**: Failed operations requiring attention
/// - **Critical**: Immediate action required, system stability at risk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Informational events for normal operations
    Info,
    /// Warning events for potential issues that should be monitored
    Warning,
    /// Error events for failed operations requiring attention
    Error,
    /// Critical events requiring immediate intervention
    Critical,
}

impl Severity {
    /// Returns true if this severity level requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Returns true if this severity indicates a problem
    pub fn is_problematic(&self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }

    /// Returns true if this severity should trigger alerts
    pub fn should_alert(&self) -> bool {
        matches!(self, Self::Warning | Self::Error | Self::Critical)
    }

    /// Returns a numeric value for severity comparison (higher = more severe)
    pub fn level(&self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Warning => 1,
            Self::Error => 2,
            Self::Critical => 3,
        }
    }

    /// Returns true if this severity is at least as severe as the given threshold
    pub fn meets_threshold(&self, threshold: Self) -> bool {
        self.level() >= threshold.level()
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

/// Security configuration levels for cryptographic operations
///
/// This enum defines security levels that affect cryptographic parameter selection,
/// balancing performance and security requirements. It consolidates multiple
/// SecurityLevel definitions across config and CLI modules.
///
/// # Performance vs Security Trade-offs
///
/// - **Low**: Fast parameters suitable for interactive use, lower security
/// - **Standard**: Balanced parameters for general use, good security/performance balance  
/// - **High**: Strong parameters for sensitive operations, higher security
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Fast parameters suitable for interactive use (lower security)
    Low,
    /// Balanced parameters for general use (standard security)
    Standard,
    /// Strong parameters for sensitive operations (higher security)
    High,
}

impl SecurityLevel {
    /// Returns Argon2 parameters (memory, time, parallelism) for this security level
    pub fn argon2_params(&self) -> (u32, u32, u32) {
        match self {
            Self::Low => (32768, 2, 2),      // Fast/Interactive: ~50ms
            Self::Standard => (65536, 3, 4), // Balanced: ~200ms
            Self::High => (131_072, 4, 8),   // High security/Sensitive: ~500ms
        }
    }

    /// Get string representation of security level for compatibility
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Standard => "Standard",
            Self::High => "High",
        }
    }

    /// Returns the recommended key derivation iterations for this security level
    pub fn key_derivation_rounds(&self) -> u32 {
        match self {
            Self::Low => 10_000,
            Self::Standard => 50_000,
            Self::High => 100_000,
        }
    }

    /// Returns true if this level is suitable for interactive operations
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Low)
    }

    /// Returns true if this level provides high security guarantees
    pub fn is_high_security(&self) -> bool {
        matches!(self, Self::High)
    }
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Standard
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Standard => write!(f, "Standard"),
            Self::High => write!(f, "High"),
        }
    }
}

/// System and component health status
///
/// This enum provides a unified health status system that consolidates multiple
/// health-related enums including RotationHealthStatus, RecoveryHealthStatus,
/// and SystemStatus variants.
///
/// # Health States
///
/// - **Healthy**: System operating normally
/// - **Warning**: Minor issues detected, functionality maintained
/// - **Critical**: Significant issues affecting operation
/// - **Failed**: Component has failed and is non-functional
/// - **Offline**: Component is unreachable or disconnected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System/component is operating normally
    Healthy,
    /// System/component has minor issues but maintains functionality
    Warning,
    /// System/component has significant issues affecting operation
    Critical,
    /// System/component has failed and is non-functional
    Failed,
    /// System/component is offline or unreachable
    Offline,
}

impl HealthStatus {
    /// Returns true if the component is considered healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Returns true if the component is operational (healthy or warning)
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Warning)
    }

    /// Returns true if the component requires immediate attention
    pub fn requires_attention(&self) -> bool {
        matches!(self, Self::Critical | Self::Failed)
    }

    /// Returns true if the component is available for operations
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::Failed | Self::Offline)
    }

    /// Returns a severity level corresponding to this health status
    pub fn to_severity(&self) -> Severity {
        match self {
            Self::Healthy => Severity::Info,
            Self::Warning => Severity::Warning,
            Self::Critical => Severity::Error,
            Self::Failed | Self::Offline => Severity::Critical,
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Warning => write!(f, "Warning"),
            Self::Critical => write!(f, "Critical"),
            Self::Failed => write!(f, "Failed"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

/// Security threat assessment levels
///
/// This enum provides standardized threat level classification for security
/// monitoring, threat detection, and risk assessment across all modules.
/// It consolidates the ThreatLevel definitions from monitoring modules.
///
/// # Threat Level Guidelines
///
/// - **Low**: Informational or minor security events requiring monitoring
/// - **Medium**: Suspicious activity that should be investigated
/// - **High**: Probable security threats requiring immediate attention
/// - **Critical**: Active attacks or imminent security breaches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Low threat - informational events requiring monitoring
    Low,
    /// Medium threat - suspicious activity requiring investigation
    Medium,
    /// High threat - probable attack requiring immediate attention
    High,
    /// Critical threat - active attack requiring emergency response
    Critical,
}

impl ThreatLevel {
    /// Returns true if this threat level requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Returns true if this threat level indicates active threat
    pub fn is_active_threat(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Returns true if this threat level should trigger alerts
    pub fn should_alert(&self) -> bool {
        matches!(self, Self::Medium | Self::High | Self::Critical)
    }

    /// Returns a numeric value for threat comparison (higher = more severe)
    pub fn level(&self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Critical => 3,
        }
    }

    /// Returns true if this threat level is at least as severe as the given threshold
    pub fn meets_threshold(&self, threshold: Self) -> bool {
        self.level() >= threshold.level()
    }

    /// Convert to corresponding severity level
    pub fn to_severity(&self) -> Severity {
        match self {
            Self::Low => Severity::Info,
            Self::Medium => Severity::Warning,
            Self::High => Severity::Error,
            Self::Critical => Severity::Critical,
        }
    }
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl PartialOrd for ThreatLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ThreatLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_status_states() {
        assert!(RotationStatus::InProgress.is_active());
        assert!(!RotationStatus::Completed.is_active());
        assert!(RotationStatus::Completed.is_successful());
        assert!(RotationStatus::Failed.is_failed());
        assert!(RotationStatus::Completed.is_terminal());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);

        assert!(Severity::Critical.meets_threshold(Severity::Warning));
        assert!(!Severity::Info.meets_threshold(Severity::Error));
    }

    #[test]
    fn test_security_level_params() {
        let low_params = SecurityLevel::Low.argon2_params();
        let high_params = SecurityLevel::High.argon2_params();

        // High security should have higher memory usage
        assert!(high_params.0 > low_params.0);

        assert!(SecurityLevel::Low.is_interactive());
        assert!(!SecurityLevel::High.is_interactive());
    }

    #[test]
    fn test_health_status_operational() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Warning.is_operational());
        assert!(!HealthStatus::Failed.is_operational());

        assert_eq!(HealthStatus::Critical.to_severity(), Severity::Error);
        assert_eq!(HealthStatus::Failed.to_severity(), Severity::Critical);
    }

    #[test]
    fn test_threat_level_ordering() {
        assert!(ThreatLevel::Critical > ThreatLevel::High);
        assert!(ThreatLevel::High > ThreatLevel::Medium);
        assert!(ThreatLevel::Medium > ThreatLevel::Low);

        assert!(ThreatLevel::Critical.meets_threshold(ThreatLevel::Medium));
        assert!(!ThreatLevel::Low.meets_threshold(ThreatLevel::High));
    }

    #[test]
    fn test_threat_level_actions() {
        assert!(ThreatLevel::Critical.requires_immediate_action());
        assert!(ThreatLevel::High.requires_immediate_action());
        assert!(!ThreatLevel::Medium.requires_immediate_action());
        assert!(!ThreatLevel::Low.requires_immediate_action());

        assert!(ThreatLevel::Critical.is_active_threat());
        assert!(!ThreatLevel::High.is_active_threat());

        assert_eq!(ThreatLevel::Critical.to_severity(), Severity::Critical);
        assert_eq!(ThreatLevel::High.to_severity(), Severity::Error);
    }

    #[test]
    fn test_serialization() {
        // Test that all enums can be serialized and deserialized
        let status = RotationStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RotationStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);

        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(severity, deserialized);

        let threat_level = ThreatLevel::High;
        let json = serde_json::to_string(&threat_level).unwrap();
        let deserialized: ThreatLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(threat_level, deserialized);
    }
}
