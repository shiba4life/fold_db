//! Detection algorithms for rotation threat monitoring
//!
//! This module contains the core detection algorithms that analyze rotation
//! activities and patterns to identify potential security threats.

use super::threat_types::{
    ActivityWindow, AttackProgression, RemediationAction, RotationThreatDetection,
    RotationThreatPattern,
};
use super::threat_config::UserBehaviorBaseline;
use super::key_rotation::{KeyRotationRequest, RotationReason};
use super::key_rotation_audit::{KeyRotationSecurityMetadata, RotationAuditCorrelation};
use super::security_monitor::SecurityDetection;
use crate::security_types::ThreatLevel;
use chrono::{DateTime, Duration as ChronoDuration, Timelike, Utc};
use std::net::IpAddr;
use std::time::Duration;
use uuid::Uuid;

/// Rotation activity tracking for threat detection
#[derive(Debug, Clone)]
pub struct RotationActivity {
    /// Timestamp of activity
    pub timestamp: DateTime<Utc>,
    /// Operation ID
    pub operation_id: Uuid,
    /// User ID
    pub user_id: Option<String>,
    /// Source IP
    pub source_ip: Option<IpAddr>,
    /// Rotation reason
    pub rotation_reason: RotationReason,
    /// Success/failure
    pub success: bool,
    /// Risk score
    pub risk_score: f64,
    /// Security metadata
    pub security_metadata: KeyRotationSecurityMetadata,
}

impl RotationActivity {
    /// Create a new rotation activity record
    pub fn new(
        operation_id: Uuid,
        request: &KeyRotationRequest,
        security_metadata: &KeyRotationSecurityMetadata,
        user_id: Option<&str>,
        success: bool,
        risk_score: f64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            operation_id,
            user_id: user_id.map(String::from),
            source_ip: security_metadata.source_ip,
            rotation_reason: request.reason.clone(),
            success,
            risk_score,
            security_metadata: security_metadata.clone(),
        }
    }
}

/// Threat detection algorithms
pub struct ThreatDetectionEngine {
    /// Minimum confidence threshold
    min_confidence_threshold: f64,
}

impl ThreatDetectionEngine {
    /// Create a new threat detection engine
    pub fn new(min_confidence_threshold: f64) -> Self {
        Self {
            min_confidence_threshold,
        }
    }

    /// Detect frequent rotation requests from the same source
    pub async fn detect_frequent_requests(
        &self,
        activity: &RotationActivity,
        recent_activities: &[RotationActivity],
    ) -> Option<RotationThreatDetection> {
        let window = ChronoDuration::hours(1);
        let cutoff = Utc::now() - window;

        // Count requests from same user or IP in the last hour
        let same_user_count = recent_activities
            .iter()
            .filter(|a| {
                a.timestamp > cutoff && a.user_id == activity.user_id && a.user_id.is_some()
            })
            .count();

        let same_ip_count = recent_activities
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

            if confidence >= self.min_confidence_threshold {
                return Some(self.create_threat_detection(
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
                ));
            }
        }

        None
    }

    /// Detect repeated rotation failures indicating potential brute force
    pub async fn detect_repeated_failures(
        &self,
        activity: &RotationActivity,
        recent_activities: &[RotationActivity],
    ) -> Option<RotationThreatDetection> {
        if activity.success {
            return None; // Only check for failed attempts
        }

        let window = ChronoDuration::minutes(30);
        let cutoff = Utc::now() - window;

        let failed_attempts = recent_activities
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

            if confidence >= self.min_confidence_threshold {
                return Some(self.create_threat_detection(
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
                ));
            }
        }

        None
    }

    /// Detect rotation requests from unusual geographic locations
    pub async fn detect_unusual_location(
        &self,
        activity: &RotationActivity,
        user_baseline: Option<&UserBehaviorBaseline>,
    ) -> Option<RotationThreatDetection> {
        let _user_id = activity.user_id.as_ref()?;
        let current_country = activity
            .security_metadata
            .geolocation
            .as_ref()?
            .country
            .as_ref()?;

        let baseline = user_baseline?;

        if !baseline.is_typical_country(current_country) {
            let confidence = 0.8; // High confidence for completely new country

            if confidence >= self.min_confidence_threshold {
                return Some(self.create_threat_detection(
                    RotationThreatPattern::UnusualLocationRotation,
                    ThreatLevel::Medium,
                    confidence,
                    vec![activity.operation_id],
                    format!(
                        "Rotation from unusual location: {} (typical: {:?})",
                        current_country, baseline.typical_countries
                    ),
                    vec![RemediationAction::EnableEnhancedMonitoring],
                ));
            }
        }

        None
    }

    /// Detect rotation requests during off-hours
    pub async fn detect_off_hours_rotation(
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

            if confidence >= self.min_confidence_threshold {
                return Some(self.create_threat_detection(
                    RotationThreatPattern::OffHoursRotation,
                    ThreatLevel::Low,
                    confidence,
                    vec![activity.operation_id],
                    format!("Key rotation during off-hours: {}:00", current_hour),
                    vec![RemediationAction::EnableEnhancedMonitoring],
                ));
            }
        }

        None
    }

    /// Detect automated rotation patterns indicating bot activity
    pub async fn detect_automated_pattern(
        &self,
        activity: &RotationActivity,
        recent_activities: &[RotationActivity],
    ) -> Option<RotationThreatDetection> {
        let window = ChronoDuration::hours(4);
        let cutoff = Utc::now() - window;

        let recent_same_source = recent_activities
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

                if confidence >= self.min_confidence_threshold {
                    return Some(self.create_threat_detection(
                        RotationThreatPattern::AutomatedRotationPattern,
                        ThreatLevel::High,
                        confidence,
                        recent_same_source.iter().map(|a| a.operation_id).collect(),
                        format!("Automated rotation pattern detected: {} requests with regular intervals", recent_same_source.len()),
                        vec![RemediationAction::BlockSourceIp, RemediationAction::InitiateIncidentResponse],
                    ));
                }
            }
        }

        None
    }

    /// Detect indicators of potential compromise
    pub async fn detect_compromise_indicators(
        &self,
        activity: &RotationActivity,
        recent_activities: &[RotationActivity],
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
            let has_used_device_before = recent_activities
                .iter()
                .any(|a| a.security_metadata.device_fingerprint.as_ref() == Some(device));

            if !has_used_device_before {
                indicators.push("New device fingerprint".to_string());
                confidence += 0.2;
            }
        }

        if confidence >= self.min_confidence_threshold && !indicators.is_empty() {
            return Some(self.create_threat_detection(
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
            ));
        }

        None
    }

    /// Detect attack progression patterns from correlation data
    pub async fn detect_attack_progression(
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

        if confidence >= self.min_confidence_threshold && stage > 2 {
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
                    pattern: super::security_monitor::SecurityPattern::UnauthorizedKeyAccess,
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

    /// Detect privilege escalation attempts (placeholder implementation)
    pub async fn detect_privilege_escalation(
        &self,
        _correlation: &RotationAuditCorrelation,
    ) -> Option<RotationThreatDetection> {
        // TODO: Implement privilege escalation detection
        // This would involve checking if user is trying to rotate keys they shouldn't have access to
        None
    }

    /// Create a threat detection result
    fn create_threat_detection(
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

        RotationThreatDetection {
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
        }
    }
}