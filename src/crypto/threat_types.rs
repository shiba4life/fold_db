//! Core types and patterns for rotation threat detection
//!
//! This module defines the fundamental types used throughout the threat monitoring system,
//! including threat patterns, detection results, and remediation actions.

use crate::security_types::ThreatLevel;
use super::security_monitor::SecurityPattern;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Key rotation specific threat patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum RotationThreatPattern {
    /// Frequent rotation requests from same source
    FrequentRotationRequests,
    /// Multiple failed rotation attempts
    RepeatedRotationFailures,
    /// Rotation requests from unusual locations
    UnusualLocationRotation,
    /// Rotation requests outside normal hours
    OffHoursRotation,
    /// Multiple simultaneous rotation attempts
    ConcurrentRotationAttempts,
    /// Rotation pattern indicating potential compromise
    CompromiseIndicators,
    /// Unusual rotation reasons pattern
    UnusualRotationReasons,
    /// Geographic anomalies in rotation requests
    GeographicAnomalies,
    /// Device fingerprint anomalies
    DeviceAnomalies,
    /// Session hijacking indicators
    SessionHijackingIndicators,
    /// Automated rotation patterns (potential bot)
    AutomatedRotationPattern,
    /// Privilege escalation attempts via rotation
    PrivilegeEscalationAttempt,
}

impl RotationThreatPattern {
    /// Get the base security pattern for this rotation threat
    pub fn to_security_pattern(&self) -> SecurityPattern {
        match self {
            RotationThreatPattern::FrequentRotationRequests
            | RotationThreatPattern::RepeatedRotationFailures => {
                SecurityPattern::RepeatedDecryptionFailures
            }
            RotationThreatPattern::UnusualLocationRotation
            | RotationThreatPattern::OffHoursRotation
            | RotationThreatPattern::GeographicAnomalies => {
                SecurityPattern::UnusualEncryptionPattern
            }
            RotationThreatPattern::ConcurrentRotationAttempts
            | RotationThreatPattern::AutomatedRotationPattern => {
                SecurityPattern::PerformanceDegradation
            }
            RotationThreatPattern::CompromiseIndicators
            | RotationThreatPattern::PrivilegeEscalationAttempt => {
                SecurityPattern::UnauthorizedKeyAccess
            }
            RotationThreatPattern::UnusualRotationReasons
            | RotationThreatPattern::DeviceAnomalies
            | RotationThreatPattern::SessionHijackingIndicators => SecurityPattern::KeyUsageAnomaly,
        }
    }

    /// Get the default threat level for this pattern
    pub fn default_threat_level(&self) -> ThreatLevel {
        match self {
            RotationThreatPattern::CompromiseIndicators
            | RotationThreatPattern::PrivilegeEscalationAttempt => ThreatLevel::Critical,
            RotationThreatPattern::SessionHijackingIndicators
            | RotationThreatPattern::AutomatedRotationPattern => ThreatLevel::High,
            RotationThreatPattern::FrequentRotationRequests
            | RotationThreatPattern::UnusualLocationRotation
            | RotationThreatPattern::GeographicAnomalies
            | RotationThreatPattern::DeviceAnomalies => ThreatLevel::Medium,
            RotationThreatPattern::RepeatedRotationFailures
            | RotationThreatPattern::OffHoursRotation
            | RotationThreatPattern::ConcurrentRotationAttempts
            | RotationThreatPattern::UnusualRotationReasons => ThreatLevel::Low,
        }
    }
}

/// Rotation threat detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationThreatDetection {
    /// Base security detection
    pub base_detection: super::security_monitor::SecurityDetection,
    /// Rotation-specific pattern
    pub rotation_pattern: RotationThreatPattern,
    /// Operation IDs involved in this threat
    pub involved_operations: Vec<Uuid>,
    /// Time window of the threat activity
    pub activity_window: ActivityWindow,
    /// Attack progression indicators
    pub attack_progression: AttackProgression,
    /// Remediation recommendations
    pub remediation_recommendations: Vec<RemediationAction>,
}

/// Time window for threat activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityWindow {
    /// Start of suspicious activity
    pub start_time: DateTime<Utc>,
    /// End of suspicious activity (if completed)
    pub end_time: Option<DateTime<Utc>>,
    /// Duration of activity
    pub duration: Duration,
    /// Peak activity timestamp
    pub peak_activity: Option<DateTime<Utc>>,
}

/// Attack progression indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackProgression {
    /// Current stage of attack (1-5, 5 being most severe)
    pub stage: u8,
    /// Confidence in attack progression (0.0 to 1.0)
    pub confidence: f64,
    /// Indicators that led to this assessment
    pub indicators: Vec<String>,
    /// Predicted next steps in attack
    pub predicted_next_steps: Vec<String>,
}

/// Remediation action recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemediationAction {
    /// Block the source IP immediately
    BlockSourceIp,
    /// Suspend user account
    SuspendUser,
    /// Force session termination
    ForceSessionTermination,
    /// Enable enhanced monitoring
    EnableEnhancedMonitoring,
    /// Alert security team
    AlertSecurityTeam,
    /// Lock all user keys temporarily
    LockUserKeys,
    /// Initiate incident response
    InitiateIncidentResponse,
    /// Review recent rotations
    ReviewRecentRotations,
    /// Update security policies
    UpdateSecurityPolicies,
}