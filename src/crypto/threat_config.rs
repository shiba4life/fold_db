//! Configuration and baselines for rotation threat monitoring
//!
//! This module contains configuration structures and user behavior baselines
//! used to customize threat detection parameters and track normal user patterns.

use super::threat_types::RotationThreatPattern;
use super::key_rotation::RotationReason;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// User behavior baseline for anomaly detection
#[derive(Debug, Clone)]
pub struct UserBehaviorBaseline {
    /// User ID
    pub user_id: String,
    /// Typical rotation frequency (per day)
    pub typical_frequency: f64,
    /// Typical rotation times (hours of day)
    pub typical_hours: Vec<u8>,
    /// Typical locations (countries)
    pub typical_countries: Vec<String>,
    /// Typical devices
    pub typical_devices: Vec<String>,
    /// Most common rotation reasons
    pub common_reasons: HashMap<RotationReason, u32>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl UserBehaviorBaseline {
    /// Create a new user behavior baseline
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            typical_frequency: 0.0,
            typical_hours: Vec::new(),
            typical_countries: Vec::new(),
            typical_devices: Vec::new(),
            common_reasons: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    /// Check if an hour is typical for this user
    pub fn is_typical_hour(&self, hour: u8) -> bool {
        self.typical_hours.contains(&hour)
    }

    /// Check if a country is typical for this user
    pub fn is_typical_country(&self, country: &str) -> bool {
        self.typical_countries.contains(&country.to_string())
    }

    /// Check if a device is typical for this user
    pub fn is_typical_device(&self, device: &str) -> bool {
        self.typical_devices.contains(&device.to_string())
    }

    /// Get the most common rotation reason for this user
    pub fn most_common_reason(&self) -> Option<&RotationReason> {
        self.common_reasons
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(reason, _)| reason)
    }

    /// Update the baseline with new activity data
    pub fn update_with_activity(
        &mut self,
        hour: u8,
        country: Option<&str>,
        device: Option<&str>,
        reason: &RotationReason,
    ) {
        // Update typical hours
        if !self.typical_hours.contains(&hour) {
            self.typical_hours.push(hour);
        }

        // Update typical countries
        if let Some(country) = country {
            let country = country.to_string();
            if !self.typical_countries.contains(&country) {
                self.typical_countries.push(country);
            }
        }

        // Update typical devices
        if let Some(device) = device {
            let device = device.to_string();
            if !self.typical_devices.contains(&device) {
                self.typical_devices.push(device);
            }
        }

        // Update common reasons
        *self.common_reasons.entry(reason.clone()).or_insert(0) += 1;

        self.last_updated = Utc::now();
    }
}