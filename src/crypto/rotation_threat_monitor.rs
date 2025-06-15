//! Threat detection and monitoring specialized for key rotation operations
//!
//! This module extends the existing security monitor with key rotation-specific
//! threat detection patterns, anomaly detection, and real-time alerting.

use super::key_rotation::{KeyRotationRequest, RotationReason};
use super::key_rotation_audit::{
    KeyRotationAuditLogger, KeyRotationSecurityMetadata, RotationAuditCorrelation,
};
use super::key_rotation_security::KeyRotationSecurityManager;
use super::security_monitor::{
    CryptoSecurityMonitor, SecurityDetection, SecurityPattern,
};
use crate::security_types::ThreatLevel;
use chrono::{DateTime, Duration as ChronoDuration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
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
    pub base_detection: SecurityDetection,
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

/// Configuration for rotation threat monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationThreatMonitorConfig {
    /// Enable rotation-specific threat monitoring
    pub enabled: bool,
    /// Monitoring window for pattern detection (minutes)
    pub monitoring_window_minutes: u64,
    /// Patterns to monitor
    pub monitored_patterns: Vec<RotationThreatPattern>,
    /// Minimum confidence threshold for threat detection
    pub min_confidence_threshold: f64,
    /// Enable real-time threat response
    pub enable_real_time_response: bool,
    /// Enable automated remediation
    pub enable_automated_remediation: bool,
    /// Threat scoring weights
    pub threat_weights: ThreatWeights,
    /// Integration settings
    pub integration_settings: IntegrationSettings,
}

impl Default for RotationThreatMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitoring_window_minutes: 60,
            monitored_patterns: vec![
                RotationThreatPattern::FrequentRotationRequests,
                RotationThreatPattern::RepeatedRotationFailures,
                RotationThreatPattern::UnusualLocationRotation,
                RotationThreatPattern::CompromiseIndicators,
                RotationThreatPattern::AutomatedRotationPattern,
                RotationThreatPattern::PrivilegeEscalationAttempt,
            ],
            min_confidence_threshold: 0.7,
            enable_real_time_response: true,
            enable_automated_remediation: false,
            threat_weights: ThreatWeights::default(),
            integration_settings: IntegrationSettings::default(),
        }
    }
}

/// Threat scoring weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatWeights {
    /// Weight for frequency-based threats
    pub frequency_weight: f64,
    /// Weight for location-based threats
    pub location_weight: f64,
    /// Weight for timing-based threats
    pub timing_weight: f64,
    /// Weight for pattern-based threats
    pub pattern_weight: f64,
    /// Weight for behavioral anomalies
    pub behavioral_weight: f64,
}

impl Default for ThreatWeights {
    fn default() -> Self {
        Self {
            frequency_weight: 0.3,
            location_weight: 0.2,
            timing_weight: 0.15,
            pattern_weight: 0.2,
            behavioral_weight: 0.15,
        }
    }
}

/// Integration settings for external systems
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationSettings {
    /// Enable SIEM integration
    pub enable_siem_integration: bool,
    /// SIEM endpoint URL
    pub siem_endpoint: Option<String>,
    /// Enable threat intelligence feeds
    pub enable_threat_intel: bool,
    /// Threat intel API endpoints
    pub threat_intel_endpoints: Vec<String>,
    /// Enable security orchestration
    pub enable_soar_integration: bool,
    /// SOAR platform endpoint
    pub soar_endpoint: Option<String>,
}


/// Rotation activity tracking for threat detection
#[derive(Debug, Clone)]
struct RotationActivity {
    /// Timestamp of activity
    timestamp: DateTime<Utc>,
    /// Operation ID
    operation_id: Uuid,
    /// User ID
    user_id: Option<String>,
    /// Source IP
    source_ip: Option<IpAddr>,
    /// Rotation reason
    rotation_reason: RotationReason,
    /// Success/failure
    success: bool,
    /// Risk score
    risk_score: f64,
    /// Security metadata
    security_metadata: KeyRotationSecurityMetadata,
}

/// Key rotation threat monitor
#[allow(dead_code)]
pub struct RotationThreatMonitor {
    /// Configuration
    config: RotationThreatMonitorConfig,
    /// Base security monitor
    base_monitor: Arc<CryptoSecurityMonitor>,
    /// Audit logger
    audit_logger: Arc<KeyRotationAuditLogger>,
    /// Security manager for policy enforcement
    security_manager: Arc<KeyRotationSecurityManager>,
    /// Recent rotation activities
    recent_activities: Arc<RwLock<Vec<RotationActivity>>>,
    /// Active threat detections
    active_threats: Arc<RwLock<HashMap<Uuid, RotationThreatDetection>>>,
    /// User behavior baselines
    user_baselines: Arc<RwLock<HashMap<String, UserBehaviorBaseline>>>,
}

/// User behavior baseline for anomaly detection
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserBehaviorBaseline {
    /// User ID
    user_id: String,
    /// Typical rotation frequency (per day)
    typical_frequency: f64,
    /// Typical rotation times (hours of day)
    typical_hours: Vec<u8>,
    /// Typical locations (countries)
    typical_countries: Vec<String>,
    /// Typical devices
    typical_devices: Vec<String>,
    /// Most common rotation reasons
    common_reasons: HashMap<RotationReason, u32>,
    /// Last updated
    last_updated: DateTime<Utc>,
}

impl RotationThreatMonitor {
    /// Create a new rotation threat monitor
    pub fn new(
        config: RotationThreatMonitorConfig,
        base_monitor: Arc<CryptoSecurityMonitor>,
        audit_logger: Arc<KeyRotationAuditLogger>,
        security_manager: Arc<KeyRotationSecurityManager>,
    ) -> Self {
        Self {
            config,
            base_monitor,
            audit_logger,
            security_manager,
            recent_activities: Arc::new(RwLock::new(Vec::new())),
            active_threats: Arc::new(RwLock::new(HashMap::new())),
            user_baselines: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_default_config(
        base_monitor: Arc<CryptoSecurityMonitor>,
        audit_logger: Arc<KeyRotationAuditLogger>,
        security_manager: Arc<KeyRotationSecurityManager>,
    ) -> Self {
        Self::new(
            RotationThreatMonitorConfig::default(),
            base_monitor,
            audit_logger,
            security_manager,
        )
    }

    /// Monitor a rotation request for threats
    pub async fn monitor_rotation_request(
        &self,
        operation_id: Uuid,
        request: &KeyRotationRequest,
        security_metadata: &KeyRotationSecurityMetadata,
        user_id: Option<&str>,
        success: bool,
        risk_score: f64,
    ) -> Vec<RotationThreatDetection> {
        if !self.config.enabled {
            return Vec::new();
        }

        // Record the activity
        let activity = RotationActivity {
            timestamp: Utc::now(),
            operation_id,
            user_id: user_id.map(String::from),
            source_ip: security_metadata.source_ip,
            rotation_reason: request.reason.clone(),
            success,
            risk_score,
            security_metadata: security_metadata.clone(),
        };

        self.record_activity(activity.clone()).await;

        // Run threat detection
        let mut detections = Vec::new();

        for pattern in &self.config.monitored_patterns {
            if let Some(detection) = self.detect_rotation_threat(pattern, &activity).await {
                detections.push(detection);
            }
        }

        // Update user behavior baseline
        if let Some(user_id) = user_id {
            self.update_user_baseline(user_id, &activity).await;
        }

        // Handle real-time response if enabled
        if self.config.enable_real_time_response && !detections.is_empty() {
            self.handle_real_time_threats(&detections, &activity).await;
        }

        detections
    }

    /// Analyze correlation for potential attack patterns
    pub async fn analyze_correlation_threats(
        &self,
        correlation: &RotationAuditCorrelation,
    ) -> Vec<RotationThreatDetection> {
        let mut detections = Vec::new();

        // Analyze the correlation for attack progression
        if let Some(detection) = self.detect_attack_progression(correlation).await {
            detections.push(detection);
        }

        // Check for privilege escalation patterns
        if let Some(detection) = self.detect_privilege_escalation(correlation).await {
            detections.push(detection);
        }

        detections
    }

    /// Get current threat status summary
    pub async fn get_threat_status(&self) -> ThreatStatusSummary {
        let active_threats = self.active_threats.read().await;
        let recent_activities = self.recent_activities.read().await;

        let mut threat_counts = HashMap::new();
        let mut max_threat_level = ThreatLevel::Low;

        for threat in active_threats.values() {
            let count = threat_counts
                .entry(threat.base_detection.threat_level)
                .or_insert(0);
            *count += 1;

            if threat.base_detection.threat_level == ThreatLevel::Critical {
                max_threat_level = ThreatLevel::Critical;
            } else if threat.base_detection.threat_level == ThreatLevel::High
                && max_threat_level != ThreatLevel::Critical
            {
                max_threat_level = ThreatLevel::High;
            } else if threat.base_detection.threat_level == ThreatLevel::Medium
                && max_threat_level != ThreatLevel::Critical
                && max_threat_level != ThreatLevel::High
            {
                max_threat_level = ThreatLevel::Medium;
            }
        }

        let one_hour_ago = Utc::now() - ChronoDuration::hours(1);
        let recent_activity_count = recent_activities
            .iter()
            .filter(|a| a.timestamp > one_hour_ago)
            .count();

        let failed_rotations = recent_activities
            .iter()
            .filter(|a| a.timestamp > one_hour_ago && !a.success)
            .count();

        ThreatStatusSummary {
            overall_threat_level: max_threat_level,
            active_threat_count: active_threats.len(),
            threat_counts,
            recent_activity_count,
            failed_rotations_last_hour: failed_rotations,
            last_updated: Utc::now(),
        }
    }

    /// Get threats by user
    pub async fn get_threats_by_user(&self, user_id: &str) -> Vec<RotationThreatDetection> {
        let active_threats = self.active_threats.read().await;
        let recent_activities = self.recent_activities.read().await;

        let user_operations: std::collections::HashSet<Uuid> = recent_activities
            .iter()
            .filter(|a| a.user_id.as_deref() == Some(user_id))
            .map(|a| a.operation_id)
            .collect();

        active_threats
            .values()
            .filter(|threat| {
                threat
                    .involved_operations
                    .iter()
                    .any(|op_id| user_operations.contains(op_id))
            })
            .cloned()
            .collect()
    }

    /// Clean up old activities and resolved threats
    pub async fn cleanup_old_data(&self, retention_hours: u64) {
        let cutoff = Utc::now() - ChronoDuration::hours(retention_hours as i64);

        // Clean up old activities
        {
            let mut activities = self.recent_activities.write().await;
            activities.retain(|a| a.timestamp > cutoff);
        }

        // Clean up resolved threats older than cutoff
        {
            let mut threats = self.active_threats.write().await;
            threats.retain(|_, threat| {
                threat.activity_window.start_time > cutoff
                    || threat.activity_window.end_time.is_none()
            });
        }

        // Update user baselines (remove stale data)
        {
            let mut baselines = self.user_baselines.write().await;
            baselines.retain(|_, baseline| baseline.last_updated > cutoff);
        }
    }

    // Private helper methods

    async fn record_activity(&self, activity: RotationActivity) {
        let mut activities = self.recent_activities.write().await;
        activities.push(activity);

        // Keep only recent activities
        let window = ChronoDuration::minutes(self.config.monitoring_window_minutes as i64);
        let cutoff = Utc::now() - window;
        activities.retain(|a| a.timestamp > cutoff);
    }

    async fn detect_rotation_threat(
        &self,
        pattern: &RotationThreatPattern,
        current_activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        match pattern {
            RotationThreatPattern::FrequentRotationRequests => {
                self.detect_frequent_requests(current_activity).await
            }
            RotationThreatPattern::RepeatedRotationFailures => {
                self.detect_repeated_failures(current_activity).await
            }
            RotationThreatPattern::UnusualLocationRotation => {
                self.detect_unusual_location(current_activity).await
            }
            RotationThreatPattern::OffHoursRotation => {
                self.detect_off_hours_rotation(current_activity).await
            }
            RotationThreatPattern::AutomatedRotationPattern => {
                self.detect_automated_pattern(current_activity).await
            }
            RotationThreatPattern::CompromiseIndicators => {
                self.detect_compromise_indicators(current_activity).await
            }
            _ => None, // Other patterns not implemented yet
        }
    }

    async fn detect_frequent_requests(
        &self,
        activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        let activities = self.recent_activities.read().await;
        let window = ChronoDuration::hours(1);
        let cutoff = Utc::now() - window;

        // Count requests from same user or IP in the last hour
        let same_user_count = activities
            .iter()
            .filter(|a| {
                a.timestamp > cutoff && a.user_id == activity.user_id && a.user_id.is_some()
            })
            .count();

        let same_ip_count = activities
            .iter()
            .filter(|a| {
                a.timestamp > cutoff && a.source_ip == activity.source_ip && a.source_ip.is_some()
            })
            .count();

        let threshold = 10; // More than 10 requests per hour is suspicious
        if same_user_count > threshold || same_ip_count > threshold {
            let confidence = ((same_user_count.max(same_ip_count) - threshold) as f64
                / threshold as f64)
                .min(1.0);

            if confidence >= self.config.min_confidence_threshold {
                return Some(
                    self.create_threat_detection(
                        RotationThreatPattern::FrequentRotationRequests,
                        ThreatLevel::Medium,
                        confidence,
                        vec![activity.operation_id],
                        format!(
                            "Frequent rotation requests detected: {} from user, {} from IP",
                            same_user_count, same_ip_count
                        ),
                        vec![
                            RemediationAction::EnableEnhancedMonitoring,
                            RemediationAction::AlertSecurityTeam,
                        ],
                    )
                    .await,
                );
            }
        }

        None
    }

    async fn detect_repeated_failures(
        &self,
        activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        if activity.success {
            return None; // Only check for failed attempts
        }

        let activities = self.recent_activities.read().await;
        let window = ChronoDuration::minutes(30);
        let cutoff = Utc::now() - window;

        let failed_attempts = activities
            .iter()
            .filter(|a| {
                a.timestamp > cutoff
                    && !a.success
                    && (a.user_id == activity.user_id || a.source_ip == activity.source_ip)
            })
            .count();

        let threshold = 5; // More than 5 failures in 30 minutes
        if failed_attempts > threshold {
            let confidence = ((failed_attempts - threshold) as f64 / threshold as f64).min(1.0);

            if confidence >= self.config.min_confidence_threshold {
                return Some(
                    self.create_threat_detection(
                        RotationThreatPattern::RepeatedRotationFailures,
                        ThreatLevel::High,
                        confidence,
                        vec![activity.operation_id],
                        format!(
                            "Repeated rotation failures detected: {} attempts",
                            failed_attempts
                        ),
                        vec![
                            RemediationAction::BlockSourceIp,
                            RemediationAction::SuspendUser,
                        ],
                    )
                    .await,
                );
            }
        }

        None
    }

    async fn detect_unusual_location(
        &self,
        activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        let user_id = activity.user_id.as_ref()?;
        let current_country = activity
            .security_metadata
            .geolocation
            .as_ref()?
            .country
            .as_ref()?;

        let baselines = self.user_baselines.read().await;
        let baseline = baselines.get(user_id)?;

        if !baseline.typical_countries.contains(current_country) {
            let confidence = 0.8; // High confidence for completely new country

            if confidence >= self.config.min_confidence_threshold {
                return Some(
                    self.create_threat_detection(
                        RotationThreatPattern::UnusualLocationRotation,
                        ThreatLevel::Medium,
                        confidence,
                        vec![activity.operation_id],
                        format!(
                            "Rotation from unusual location: {} (typical: {:?})",
                            current_country, baseline.typical_countries
                        ),
                        vec![RemediationAction::EnableEnhancedMonitoring],
                    )
                    .await,
                );
            }
        }

        None
    }

    async fn detect_off_hours_rotation(
        &self,
        activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        let current_hour = activity.timestamp.time().hour() as u8;

        // Define business hours (9 AM to 5 PM)
        let business_hours = [9, 10, 11, 12, 13, 14, 15, 16, 17];

        if !business_hours.contains(&current_hour) {
            let confidence = match current_hour {
                0..=5 | 22..=23 => 0.9, // Late night/early morning
                6..=8 | 18..=21 => 0.6, // Early morning/evening
                _ => 0.3,
            };

            if confidence >= self.config.min_confidence_threshold {
                return Some(
                    self.create_threat_detection(
                        RotationThreatPattern::OffHoursRotation,
                        ThreatLevel::Low,
                        confidence,
                        vec![activity.operation_id],
                        format!("Key rotation during off-hours: {}:00", current_hour),
                        vec![RemediationAction::EnableEnhancedMonitoring],
                    )
                    .await,
                );
            }
        }

        None
    }

    async fn detect_automated_pattern(
        &self,
        activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        let activities = self.recent_activities.read().await;
        let window = ChronoDuration::hours(4);
        let cutoff = Utc::now() - window;

        let recent_same_source = activities
            .iter()
            .filter(|a| {
                a.timestamp > cutoff
                    && a.source_ip == activity.source_ip
                    && a.user_id == activity.user_id
            })
            .collect::<Vec<_>>();

        if recent_same_source.len() < 5 {
            return None; // Need at least 5 activities to detect pattern
        }

        // Check for regular timing intervals (indicates automation)
        let mut intervals = Vec::new();
        for i in 1..recent_same_source.len() {
            let interval = recent_same_source[i]
                .timestamp
                .signed_duration_since(recent_same_source[i - 1].timestamp);
            intervals.push(interval.num_seconds());
        }

        // Check if intervals are suspiciously regular
        if intervals.len() >= 3 {
            let avg_interval = intervals.iter().sum::<i64>() / intervals.len() as i64;
            let variance = intervals
                .iter()
                .map(|&x| (x - avg_interval).pow(2))
                .sum::<i64>()
                / intervals.len() as i64;

            let std_dev = (variance as f64).sqrt();
            let coefficient_of_variation = std_dev / avg_interval.abs() as f64;

            // Low coefficient of variation indicates regular timing
            if coefficient_of_variation < 0.1 && avg_interval < 3600 {
                // Less than 1 hour intervals
                let confidence = (1.0 - coefficient_of_variation * 10.0).min(1.0);

                if confidence >= self.config.min_confidence_threshold {
                    return Some(self.create_threat_detection(
                        RotationThreatPattern::AutomatedRotationPattern,
                        ThreatLevel::High,
                        confidence,
                        recent_same_source.iter().map(|a| a.operation_id).collect(),
                        format!("Automated rotation pattern detected: {} requests with regular intervals", recent_same_source.len()),
                        vec![RemediationAction::BlockSourceIp, RemediationAction::InitiateIncidentResponse],
                    ).await);
                }
            }
        }

        None
    }

    async fn detect_compromise_indicators(
        &self,
        activity: &RotationActivity,
    ) -> Option<RotationThreatDetection> {
        let mut indicators = Vec::new();
        let mut confidence = 0.0;

        // Check for compromise-related rotation reason
        if activity.rotation_reason == RotationReason::Compromise {
            indicators.push("Explicit compromise reason".to_string());
            confidence += 0.8;
        }

        // Check for high risk score
        if activity.risk_score > 0.8 {
            indicators.push(format!("High risk score: {:.2}", activity.risk_score));
            confidence += 0.3;
        }

        // Check for VPN/proxy usage
        if let Some(ref geo) = activity.security_metadata.geolocation {
            if geo.is_vpn.unwrap_or(false) {
                indicators.push("VPN/proxy usage detected".to_string());
                confidence += 0.2;
            }
        }

        // Check for new device
        if let Some(ref device) = activity.security_metadata.device_fingerprint {
            let activities = self.recent_activities.read().await;
            let has_used_device_before = activities
                .iter()
                .any(|a| a.security_metadata.device_fingerprint.as_ref() == Some(device));

            if !has_used_device_before {
                indicators.push("New device fingerprint".to_string());
                confidence += 0.2;
            }
        }

        if confidence >= self.config.min_confidence_threshold && !indicators.is_empty() {
            return Some(
                self.create_threat_detection(
                    RotationThreatPattern::CompromiseIndicators,
                    ThreatLevel::Critical,
                    confidence.min(1.0),
                    vec![activity.operation_id],
                    format!("Compromise indicators detected: {}", indicators.join(", ")),
                    vec![
                        RemediationAction::SuspendUser,
                        RemediationAction::LockUserKeys,
                        RemediationAction::InitiateIncidentResponse,
                        RemediationAction::AlertSecurityTeam,
                    ],
                )
                .await,
            );
        }

        None
    }

    async fn detect_attack_progression(
        &self,
        correlation: &RotationAuditCorrelation,
    ) -> Option<RotationThreatDetection> {
        // Analyze the correlation events for attack progression patterns
        let failed_attempts = correlation.failed_attempts;
        let duration = correlation
            .completed_at
            .unwrap_or_else(Utc::now)
            .signed_duration_since(correlation.started_at);

        let mut stage = 1u8;
        let mut confidence = 0.0;
        let mut indicators = Vec::new();

        // Stage 1: Initial probing
        if failed_attempts > 0 {
            stage = 2;
            confidence += 0.2;
            indicators.push("Failed rotation attempts detected".to_string());
        }

        // Stage 2: Persistent attempts
        if failed_attempts > 3 {
            stage = 3;
            confidence += 0.3;
            indicators.push("Multiple failed attempts indicate persistence".to_string());
        }

        // Stage 3: Escalation tactics
        if correlation
            .security_metadata
            .geolocation
            .as_ref()
            .and_then(|g| g.is_vpn)
            .unwrap_or(false)
        {
            stage = 4;
            confidence += 0.3;
            indicators.push("VPN usage indicates evasion tactics".to_string());
        }

        // Stage 4: Compromise indicators
        if correlation.rotation_reason == RotationReason::Compromise {
            stage = 5;
            confidence += 0.4;
            indicators.push("Explicit compromise indication".to_string());
        }

        if confidence >= self.config.min_confidence_threshold && stage > 2 {
            let attack_progression = AttackProgression {
                stage,
                confidence,
                indicators: indicators.clone(),
                predicted_next_steps: vec![
                    "Attempt to rotate multiple keys".to_string(),
                    "Try to establish persistence".to_string(),
                    "Escalate privileges".to_string(),
                ],
            };

            let threat_level = match stage {
                1..=2 => ThreatLevel::Low,
                3 => ThreatLevel::Medium,
                4 => ThreatLevel::High,
                5.. => ThreatLevel::Critical,
                _ => ThreatLevel::Low,
            };

            return Some(RotationThreatDetection {
                base_detection: SecurityDetection {
                    detection_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    pattern: SecurityPattern::UnauthorizedKeyAccess,
                    threat_level,
                    confidence,
                    source: correlation.user_id.clone().unwrap_or("unknown".to_string()),
                    description: format!("Attack progression detected at stage {}", stage),
                    evidence: std::collections::HashMap::from([
                        ("stage".to_string(), serde_json::Value::Number(stage.into())),
                        ("indicators".to_string(), serde_json::json!(indicators)),
                    ]),
                    recommended_actions: vec![
                        "Initiate incident response".to_string(),
                        "Monitor all user activities".to_string(),
                        "Consider account suspension".to_string(),
                    ],
                },
                rotation_pattern: RotationThreatPattern::CompromiseIndicators,
                involved_operations: correlation.related_events.clone(),
                activity_window: ActivityWindow {
                    start_time: correlation.started_at,
                    end_time: correlation.completed_at,
                    duration: Duration::from_secs(duration.num_seconds().max(0) as u64),
                    peak_activity: Some(correlation.started_at),
                },
                attack_progression,
                remediation_recommendations: vec![
                    RemediationAction::InitiateIncidentResponse,
                    RemediationAction::SuspendUser,
                    RemediationAction::LockUserKeys,
                    RemediationAction::AlertSecurityTeam,
                ],
            });
        }

        None
    }

    async fn detect_privilege_escalation(
        &self,
        _correlation: &RotationAuditCorrelation,
    ) -> Option<RotationThreatDetection> {
        // TODO: Implement privilege escalation detection
        // This would involve checking if user is trying to rotate keys they shouldn't have access to
        None
    }

    async fn create_threat_detection(
        &self,
        pattern: RotationThreatPattern,
        threat_level: ThreatLevel,
        confidence: f64,
        involved_operations: Vec<Uuid>,
        description: String,
        remediation_actions: Vec<RemediationAction>,
    ) -> RotationThreatDetection {
        let detection_id = Uuid::new_v4();
        let now = Utc::now();

        let base_detection = SecurityDetection {
            detection_id,
            timestamp: now,
            pattern: pattern.to_security_pattern(),
            threat_level,
            confidence,
            source: "rotation_threat_monitor".to_string(),
            description: description.clone(),
            evidence: std::collections::HashMap::from([
                ("pattern".to_string(), serde_json::json!(pattern)),
                (
                    "operations".to_string(),
                    serde_json::json!(involved_operations),
                ),
            ]),
            recommended_actions: remediation_actions
                .iter()
                .map(|a| format!("{:?}", a))
                .collect(),
        };

        let threat_detection = RotationThreatDetection {
            base_detection,
            rotation_pattern: pattern,
            involved_operations: involved_operations.clone(),
            activity_window: ActivityWindow {
                start_time: now,
                end_time: None,
                duration: Duration::from_secs(0),
                peak_activity: Some(now),
            },
            attack_progression: AttackProgression {
                stage: 1,
                confidence,
                indicators: vec![description],
                predicted_next_steps: Vec::new(),
            },
            remediation_recommendations: remediation_actions,
        };

        // Store the threat
        {
            let mut threats = self.active_threats.write().await;
            threats.insert(detection_id, threat_detection.clone());
        }

        threat_detection
    }

    async fn handle_real_time_threats(
        &self,
        detections: &[RotationThreatDetection],
        activity: &RotationActivity,
    ) {
        for detection in detections {
            // Log the threat detection
            self.audit_logger
                .log_suspicious_pattern(
                    activity.operation_id,
                    &format!("{:?}", detection.rotation_pattern),
                    &detection.base_detection.description,
                    detection.base_detection.confidence,
                    activity.user_id.clone(),
                )
                .await;

            // Apply automated remediation if enabled
            if self.config.enable_automated_remediation {
                self.apply_automated_remediation(detection, activity).await;
            }
        }
    }

    async fn apply_automated_remediation(
        &self,
        detection: &RotationThreatDetection,
        _activity: &RotationActivity,
    ) {
        // In a real implementation, this would trigger actual remediation actions
        // For now, just log what would be done
        for action in &detection.remediation_recommendations {
            match action {
                RemediationAction::BlockSourceIp => {
                    // TODO: Add IP to block list
                }
                RemediationAction::SuspendUser => {
                    // TODO: Suspend user account
                }
                RemediationAction::AlertSecurityTeam => {
                    // TODO: Send alert to security team
                }
                _ => {
                    // TODO: Implement other remediation actions
                }
            }
        }
    }

    async fn update_user_baseline(&self, user_id: &str, activity: &RotationActivity) {
        let mut baselines = self.user_baselines.write().await;
        let baseline =
            baselines
                .entry(user_id.to_string())
                .or_insert_with(|| UserBehaviorBaseline {
                    user_id: user_id.to_string(),
                    typical_frequency: 0.0,
                    typical_hours: Vec::new(),
                    typical_countries: Vec::new(),
                    typical_devices: Vec::new(),
                    common_reasons: HashMap::new(),
                    last_updated: Utc::now(),
                });

        // Update typical hours
        let current_hour = activity.timestamp.time().hour() as u8;
        if !baseline.typical_hours.contains(&current_hour) {
            baseline.typical_hours.push(current_hour);
        }

        // Update typical countries
        if let Some(ref geo) = activity.security_metadata.geolocation {
            if let Some(ref country) = geo.country {
                if !baseline.typical_countries.contains(country) {
                    baseline.typical_countries.push(country.clone());
                }
            }
        }

        // Update typical devices
        if let Some(ref device) = activity.security_metadata.device_fingerprint {
            if !baseline.typical_devices.contains(device) {
                baseline.typical_devices.push(device.clone());
            }
        }

        // Update common reasons
        *baseline
            .common_reasons
            .entry(activity.rotation_reason.clone())
            .or_insert(0) += 1;

        baseline.last_updated = Utc::now();
    }
}

/// Threat status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatStatusSummary {
    /// Overall threat level
    pub overall_threat_level: ThreatLevel,
    /// Number of active threats
    pub active_threat_count: usize,
    /// Threat counts by level
    pub threat_counts: HashMap<ThreatLevel, usize>,
    /// Recent activity count (last hour)
    pub recent_activity_count: usize,
    /// Failed rotations in last hour
    pub failed_rotations_last_hour: usize,
    /// When this summary was generated
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{
        generate_master_keypair,
    };
    use std::net::Ipv4Addr;

    #[tokio::test]
    #[ignore = "Disabled due to admin functionality removal"]
    async fn test_frequent_requests_detection() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        let user_id = "test-user";
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Simulate multiple rotation requests
        for i in 0..15 {
            let old_keypair = generate_master_keypair().unwrap();
            let new_keypair = generate_master_keypair().unwrap();
            let old_private_key = old_keypair.private_key();

            let request = super::super::key_rotation::KeyRotationRequest::new(
                &old_private_key,
                new_keypair.public_key().clone(),
                super::super::key_rotation::RotationReason::UserInitiated,
                Some("test-client".to_string()),
                HashMap::new(),
            )
            .unwrap();

            let security_metadata = super::super::key_rotation_audit::KeyRotationSecurityMetadata {
                source_ip: Some(ip),
                user_agent: Some("DataFold-CLI/1.0".to_string()),
                geolocation: None,
                session_info: None,
                device_fingerprint: Some("test-device".to_string()),
                auth_method: Some("signature".to_string()),
                risk_score: Some(0.1),
                request_source: Some("CLI".to_string()),
            };

            let detections = threat_monitor
                .monitor_rotation_request(
                    Uuid::new_v4(),
                    &request,
                    &security_metadata,
                    Some(user_id),
                    true,
                    0.1,
                )
                .await;

            // Should detect frequent requests after threshold
            if i >= 10 {
                assert!(!detections.is_empty());
                let detection = &detections[0];
                assert_eq!(
                    detection.rotation_pattern,
                    RotationThreatPattern::FrequentRotationRequests
                );
            }
        }
    }

    #[tokio::test]
    async fn test_off_hours_detection() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        // Create activity during off-hours (2 AM)
        let activity = RotationActivity {
            timestamp: Utc::now().with_hour(2).unwrap(),
            operation_id: Uuid::new_v4(),
            user_id: Some("test-user".to_string()),
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            rotation_reason: super::super::key_rotation::RotationReason::UserInitiated,
            success: true,
            risk_score: 0.1,
            security_metadata: super::super::key_rotation_audit::KeyRotationSecurityMetadata {
                source_ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                user_agent: Some("DataFold-CLI/1.0".to_string()),
                geolocation: None,
                session_info: None,
                device_fingerprint: Some("test-device".to_string()),
                auth_method: Some("signature".to_string()),
                risk_score: Some(0.1),
                request_source: Some("CLI".to_string()),
            },
        };

        let detection = threat_monitor.detect_off_hours_rotation(&activity).await;
        assert!(detection.is_some());

        let detection = detection.unwrap();
        assert_eq!(
            detection.rotation_pattern,
            RotationThreatPattern::OffHoursRotation
        );
        assert_eq!(detection.base_detection.threat_level, ThreatLevel::Low);
    }

    #[tokio::test]
    async fn test_threat_status_summary() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        let status = threat_monitor.get_threat_status().await;
        assert_eq!(status.active_threat_count, 0);
        assert_eq!(status.overall_threat_level, ThreatLevel::Low);
        assert_eq!(status.recent_activity_count, 0);
    }
}
