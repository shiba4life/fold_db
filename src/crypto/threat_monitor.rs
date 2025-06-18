//! Main monitor coordination for rotation threat detection
//!
//! This module contains the primary `RotationThreatMonitor` struct that coordinates
//! threat detection activities and manages the overall threat monitoring system.

use super::threat_types::{RemediationAction, RotationThreatDetection, RotationThreatPattern};
use super::threat_config::{RotationThreatMonitorConfig, UserBehaviorBaseline};
use super::threat_detection::{RotationActivity, ThreatDetectionEngine};
use super::key_rotation::KeyRotationRequest;
use super::key_rotation_audit::{
    KeyRotationAuditLogger, KeyRotationSecurityMetadata, RotationAuditCorrelation,
};
use super::key_rotation_security::KeyRotationSecurityManager;
use super::security_monitor::CryptoSecurityMonitor;
use crate::security_types::ThreatLevel;
use chrono::{DateTime, Duration as ChronoDuration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Key rotation threat monitor
pub struct RotationThreatMonitor {
    /// Configuration
    config: RotationThreatMonitorConfig,
    /// Base security monitor
    #[allow(dead_code)]
    base_monitor: Arc<CryptoSecurityMonitor>,
    /// Audit logger
    audit_logger: Arc<KeyRotationAuditLogger>,
    /// Security manager for policy enforcement
    #[allow(dead_code)]
    security_manager: Arc<KeyRotationSecurityManager>,
    /// Recent rotation activities
    recent_activities: Arc<RwLock<Vec<RotationActivity>>>,
    /// Active threat detections
    active_threats: Arc<RwLock<HashMap<Uuid, RotationThreatDetection>>>,
    /// User behavior baselines
    user_baselines: Arc<RwLock<HashMap<String, UserBehaviorBaseline>>>,
    /// Threat detection engine
    detection_engine: ThreatDetectionEngine,
}

impl RotationThreatMonitor {
    /// Create a new rotation threat monitor
    pub fn new(
        config: RotationThreatMonitorConfig,
        base_monitor: Arc<CryptoSecurityMonitor>,
        audit_logger: Arc<KeyRotationAuditLogger>,
        security_manager: Arc<KeyRotationSecurityManager>,
    ) -> Self {
        let detection_engine = ThreatDetectionEngine::new(config.min_confidence_threshold);
        
        Self {
            config,
            base_monitor,
            audit_logger,
            security_manager,
            recent_activities: Arc::new(RwLock::new(Vec::new())),
            active_threats: Arc::new(RwLock::new(HashMap::new())),
            user_baselines: Arc::new(RwLock::new(HashMap::new())),
            detection_engine,
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
        let activity = RotationActivity::new(
            operation_id,
            request,
            security_metadata,
            user_id,
            success,
            risk_score,
        );

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
        if let Some(detection) = self.detection_engine.detect_attack_progression(correlation).await {
            detections.push(detection);
        }

        // Check for privilege escalation patterns
        if let Some(detection) = self.detection_engine.detect_privilege_escalation(correlation).await {
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
        let recent_activities = self.recent_activities.read().await;
        let activities_slice = recent_activities.as_slice();

        match pattern {
            RotationThreatPattern::FrequentRotationRequests => {
                self.detection_engine.detect_frequent_requests(current_activity, activities_slice).await
            }
            RotationThreatPattern::RepeatedRotationFailures => {
                self.detection_engine.detect_repeated_failures(current_activity, activities_slice).await
            }
            RotationThreatPattern::UnusualLocationRotation => {
                let user_baselines = self.user_baselines.read().await;
                let baseline = current_activity.user_id.as_ref()
                    .and_then(|uid| user_baselines.get(uid));
                self.detection_engine.detect_unusual_location(current_activity, baseline).await
            }
            RotationThreatPattern::OffHoursRotation => {
                self.detection_engine.detect_off_hours_rotation(current_activity).await
            }
            RotationThreatPattern::AutomatedRotationPattern => {
                self.detection_engine.detect_automated_pattern(current_activity, activities_slice).await
            }
            RotationThreatPattern::CompromiseIndicators => {
                self.detection_engine.detect_compromise_indicators(current_activity, activities_slice).await
            }
            _ => None, // Other patterns not implemented yet
        }
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

            // Store the threat
            {
                let mut threats = self.active_threats.write().await;
                threats.insert(detection.base_detection.detection_id, detection.clone());
            }

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
        let baseline = baselines
            .entry(user_id.to_string())
            .or_insert_with(|| UserBehaviorBaseline::new(user_id.to_string()));

        // Extract data from activity
        let current_hour = activity.timestamp.time().hour() as u8;
        let country = activity
            .security_metadata
            .geolocation
            .as_ref()
            .and_then(|g| g.country.as_deref());
        let device = activity
            .security_metadata
            .device_fingerprint
            .as_deref();

        // Update baseline
        baseline.update_with_activity(
            current_hour,
            country,
            device,
            &activity.rotation_reason,
        );
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