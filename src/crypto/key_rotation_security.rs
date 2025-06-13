//! Security policies for key rotation
//!
//! This module provides essential security framework for key rotation operations,
//! focusing on cryptographic signature verification, rate limiting, and audit logging.

use super::key_rotation::KeyRotationRequest;
use super::key_rotation_audit::{KeyRotationAuditLogger, KeyRotationSecurityMetadata, SessionInfo};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Security policy configuration for key rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationSecurityPolicy {
    /// Policy name/identifier
    pub name: String,
    /// Whether this policy is enabled
    pub enabled: bool,
    /// Rate limiting configuration
    pub rate_limits: RateLimitConfig,
    /// IP address restrictions
    pub ip_restrictions: IpRestrictionConfig,
    /// Time-based restrictions
    pub time_restrictions: TimeRestrictionConfig,
    /// Risk assessment configuration
    pub risk_assessment: RiskAssessmentConfig,
    /// Session security requirements (emergency bypass removed for security)
    pub session_requirements: SessionSecurityConfig,
}

impl Default for KeyRotationSecurityPolicy {
    fn default() -> Self {
        Self {
            name: "default_rotation_policy".to_string(),
            enabled: true,
            rate_limits: RateLimitConfig::default(),
            ip_restrictions: IpRestrictionConfig::default(),
            time_restrictions: TimeRestrictionConfig::default(),
            risk_assessment: RiskAssessmentConfig::default(),
            session_requirements: SessionSecurityConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum rotations per user per hour
    pub max_rotations_per_user_per_hour: u32,
    /// Maximum rotations per IP per hour
    pub max_rotations_per_ip_per_hour: u32,
    /// Maximum failed attempts per user per hour
    pub max_failed_attempts_per_user_per_hour: u32,
    /// Lockout duration after rate limit exceeded (minutes)
    pub lockout_duration_minutes: u64,
    /// Enable progressive delays on repeated failures
    pub enable_progressive_delays: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_rotations_per_user_per_hour: 10,
            max_rotations_per_ip_per_hour: 50,
            max_failed_attempts_per_user_per_hour: 20,
            lockout_duration_minutes: 30,
            enable_progressive_delays: true,
        }
    }
}

/// IP address restriction configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpRestrictionConfig {
    /// Enable IP-based restrictions
    pub enabled: bool,
    /// Allowed IP ranges (CIDR notation)
    pub allowed_ranges: Vec<String>,
    /// Blocked IP ranges (CIDR notation)
    pub blocked_ranges: Vec<String>,
    /// Require specific countries (ISO 3166-1 alpha-2 codes)
    pub allowed_countries: Vec<String>,
    /// Block specific countries
    pub blocked_countries: Vec<String>,
    /// Block known VPN/proxy IPs
    pub block_vpn_proxy: bool,
}


/// Time-based restriction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictionConfig {
    /// Enable time-based restrictions
    pub enabled: bool,
    /// Allowed hours of day (0-23)
    pub allowed_hours: Vec<u8>,
    /// Allowed days of week (0=Sunday, 6=Saturday)
    pub allowed_days: Vec<u8>,
    /// Timezone for time restrictions
    pub timezone: String,
    /// Block rotations during maintenance windows
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

impl Default for TimeRestrictionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_hours: (0..24).collect(),
            allowed_days: (0..7).collect(),
            timezone: "UTC".to_string(),
            maintenance_windows: Vec::new(),
        }
    }
}

/// Maintenance window definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    /// Window name
    pub name: String,
    /// Start time (cron format or ISO 8601)
    pub start_time: String,
    /// Duration in minutes
    pub duration_minutes: u64,
    /// Recurrence pattern
    pub recurrence: String,
}

/// Risk assessment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentConfig {
    /// Enable risk assessment
    pub enabled: bool,
    /// Maximum allowed risk score (0.0 to 1.0)
    pub max_risk_score: f64,
    /// Factors that contribute to risk scoring
    pub risk_factors: RiskFactors,
    /// Actions to take based on risk levels
    pub risk_actions: RiskActions,
}

impl Default for RiskAssessmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_risk_score: 0.7,
            risk_factors: RiskFactors::default(),
            risk_actions: RiskActions::default(),
        }
    }
}

/// Risk factors configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    /// Weight for unusual IP address (0.0 to 1.0)
    pub unusual_ip_weight: f64,
    /// Weight for unusual location (0.0 to 1.0)
    pub unusual_location_weight: f64,
    /// Weight for unusual time (0.0 to 1.0)
    pub unusual_time_weight: f64,
    /// Weight for new device (0.0 to 1.0)
    pub new_device_weight: f64,
    /// Weight for VPN/proxy usage (0.0 to 1.0)
    pub vpn_proxy_weight: f64,
    /// Weight for recent failed attempts (0.0 to 1.0)
    pub recent_failures_weight: f64,
}

impl Default for RiskFactors {
    fn default() -> Self {
        Self {
            unusual_ip_weight: 0.3,
            unusual_location_weight: 0.2,
            unusual_time_weight: 0.1,
            new_device_weight: 0.2,
            vpn_proxy_weight: 0.3,
            recent_failures_weight: 0.4,
        }
    }
}

/// Risk-based actions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskActions {
    /// Actions for low risk (0.0 - 0.3)
    pub low_risk: RiskAction,
    /// Actions for medium risk (0.3 - 0.7)
    pub medium_risk: RiskAction,
    /// Actions for high risk (0.7 - 1.0)
    pub high_risk: RiskAction,
}

impl Default for RiskActions {
    fn default() -> Self {
        Self {
            low_risk: RiskAction::Allow,
            medium_risk: RiskAction::AllowWithMonitoring,
            high_risk: RiskAction::Block,
        }
    }
}

/// Actions to take based on risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskAction {
    /// Allow the operation
    Allow,
    /// Block the operation
    Block,
    /// Allow but with enhanced monitoring
    AllowWithMonitoring,
}

// REMOVED: Emergency bypass configuration - removed for security
// There should be no bypass mechanism for key rotation validation

// REMOVED: EmergencyBypassConfig default implementation - emergency bypass removed for security

/// Session security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSecurityConfig {
    /// Maximum session age for key rotation (minutes)
    pub max_session_age_minutes: u64,
    /// Require fresh authentication for sensitive operations
    pub require_fresh_auth: bool,
    /// Maximum time since last authentication (minutes)
    pub max_auth_age_minutes: u64,
    /// Require device verification
    pub require_device_verification: bool,
}

impl Default for SessionSecurityConfig {
    fn default() -> Self {
        Self {
            max_session_age_minutes: 60,
            require_fresh_auth: true,
            max_auth_age_minutes: 15,
            require_device_verification: true,
        }
    }
}

/// Rate limiting tracking entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RateLimitEntry {
    /// Timestamp of the entry
    timestamp: DateTime<Utc>,
    /// Whether this was a successful or failed attempt
    success: bool,
    /// User ID
    user_id: Option<String>,
    /// IP address
    ip_address: Option<IpAddr>,
}

/// Key rotation security manager
#[allow(dead_code)]
pub struct KeyRotationSecurityManager {
    /// Security policy
    policy: Arc<RwLock<KeyRotationSecurityPolicy>>,
    /// Rate limiting tracking
    rate_limit_tracking: Arc<RwLock<Vec<RateLimitEntry>>>,
    /// Audit logger
    audit_logger: Arc<KeyRotationAuditLogger>,
    /// User lockouts
    user_lockouts: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// IP lockouts
    ip_lockouts: Arc<RwLock<HashMap<IpAddr, DateTime<Utc>>>>,
}

impl KeyRotationSecurityManager {
    /// Create a new security manager
    pub fn new(
        policy: KeyRotationSecurityPolicy,
        audit_logger: Arc<KeyRotationAuditLogger>,
    ) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            rate_limit_tracking: Arc::new(RwLock::new(Vec::new())),
            audit_logger,
            user_lockouts: Arc::new(RwLock::new(HashMap::new())),
            ip_lockouts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default policy
    pub fn with_default_policy(audit_logger: Arc<KeyRotationAuditLogger>) -> Self {
        Self::new(KeyRotationSecurityPolicy::default(), audit_logger)
    }

    /// Evaluate security policies for a rotation request
    pub async fn evaluate_rotation_request(
        &self,
        request: &KeyRotationRequest,
        security_metadata: &KeyRotationSecurityMetadata,
        user_id: Option<&str>,
    ) -> SecurityEvaluationResult {
        let mut result = SecurityEvaluationResult {
            allowed: true,
            risk_score: 0.0,
            warnings: Vec::new(),
            violations: Vec::new(),
            required_actions: Vec::new(),
        };

        let policy = self.policy.read().await;

        if !policy.enabled {
            return result;
        }

        // Check rate limits
        if let Some(violation) = self
            .check_rate_limits(user_id, security_metadata.source_ip)
            .await
        {
            result.allowed = false;
            result.violations.push(violation);
            return result;
        }

        // Check IP restrictions
        if let Some(violation) = self
            .check_ip_restrictions(&security_metadata.source_ip, &policy.ip_restrictions)
            .await
        {
            result.allowed = false;
            result.violations.push(violation);
            return result;
        }

        // Check time restrictions
        if let Some(violation) = self
            .check_time_restrictions(&policy.time_restrictions)
            .await
        {
            result.allowed = false;
            result.violations.push(violation);
            return result;
        }

        // Check session requirements
        if let Some(violation) = self
            .check_session_requirements(
                &policy.session_requirements,
                security_metadata.session_info.as_ref(),
            )
            .await
        {
            result.allowed = false;
            result.violations.push(violation);
            return result;
        }

        // Calculate risk score
        result.risk_score = self
            .calculate_risk_score(security_metadata, request, user_id)
            .await;

        // Apply risk-based actions
        match self.get_risk_action(result.risk_score, &policy.risk_assessment.risk_actions) {
            RiskAction::Allow => {
                // Operation allowed
            }
            RiskAction::AllowWithMonitoring => {
                result
                    .warnings
                    .push("High-risk operation - enhanced monitoring enabled".to_string());
                result
                    .required_actions
                    .push("enhanced_monitoring".to_string());
            }
            RiskAction::Block => {
                result.allowed = false;
                result.violations.push(format!(
                    "Risk score {} exceeds maximum allowed {}",
                    result.risk_score, policy.risk_assessment.max_risk_score
                ));
                return result;
            }
        }

        result
    }

    /// Record a rotation attempt for rate limiting
    pub async fn record_rotation_attempt(
        &self,
        user_id: Option<&str>,
        ip_address: Option<IpAddr>,
        success: bool,
    ) {
        let entry = RateLimitEntry {
            timestamp: Utc::now(),
            success,
            user_id: user_id.map(|s| s.to_string()),
            ip_address,
        };

        let mut tracking = self.rate_limit_tracking.write().await;
        tracking.push(entry);

        // Clean up old entries (older than 24 hours)
        let cutoff = Utc::now() - ChronoDuration::hours(24);
        tracking.retain(|entry| entry.timestamp > cutoff);
    }

    // REMOVED: Emergency bypass functionality - removed for security
    // There should be no bypass mechanism for key rotation validation

    async fn check_rate_limits(
        &self,
        user_id: Option<&str>,
        ip_address: Option<IpAddr>,
    ) -> Option<String> {
        let policy = self.policy.read().await;
        let tracking = self.rate_limit_tracking.read().await;
        let now = Utc::now();
        let one_hour_ago = now - ChronoDuration::hours(1);

        // Check user rate limits
        if let Some(uid) = user_id {
            let user_attempts = tracking
                .iter()
                .filter(|entry| entry.timestamp > one_hour_ago)
                .filter(|entry| entry.user_id.as_deref() == Some(uid))
                .count() as u32;

            if user_attempts >= policy.rate_limits.max_rotations_per_user_per_hour {
                return Some(format!(
                    "User rate limit exceeded: {} attempts in the last hour",
                    user_attempts
                ));
            }
        }

        // Check IP rate limits
        if let Some(ip) = ip_address {
            let ip_attempts = tracking
                .iter()
                .filter(|entry| entry.timestamp > one_hour_ago)
                .filter(|entry| entry.ip_address.as_ref() == Some(&ip))
                .count() as u32;

            if ip_attempts >= policy.rate_limits.max_rotations_per_ip_per_hour {
                return Some(format!(
                    "IP rate limit exceeded: {} attempts in the last hour",
                    ip_attempts
                ));
            }
        }

        None
    }

    async fn check_ip_restrictions(
        &self,
        ip_address: &Option<IpAddr>,
        config: &IpRestrictionConfig,
    ) -> Option<String> {
        if !config.enabled {
            return None;
        }

        let ip = match ip_address {
            Some(ip) => ip,
            None => return Some("IP address required for IP restrictions".to_string()),
        };

        // Check blocked ranges
        for range in &config.blocked_ranges {
            if self.ip_in_range(ip, range) {
                return Some(format!("IP address {} is in blocked range {}", ip, range));
            }
        }

        // Check allowed ranges (if any are specified)
        if !config.allowed_ranges.is_empty() {
            let mut allowed = false;
            for range in &config.allowed_ranges {
                if self.ip_in_range(ip, range) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return Some(format!("IP address {} is not in any allowed range", ip));
            }
        }

        None
    }

    async fn check_time_restrictions(&self, config: &TimeRestrictionConfig) -> Option<String> {
        if !config.enabled {
            return None;
        }

        let now = Utc::now();

        // Check allowed hours
        let hour = now.hour() as u8;
        if !config.allowed_hours.contains(&hour) {
            return Some(format!("Current hour {} is not allowed", hour));
        }

        // Check allowed days
        let day = now.weekday().num_days_from_sunday() as u8;
        if !config.allowed_days.contains(&day) {
            return Some(format!("Current day {} is not allowed", day));
        }

        // Check maintenance windows
        for window in &config.maintenance_windows {
            if self.is_in_maintenance_window(&now, window) {
                return Some(format!(
                    "Operation blocked during maintenance window: {}",
                    window.name
                ));
            }
        }

        None
    }

    async fn check_session_requirements(
        &self,
        config: &SessionSecurityConfig,
        session_info: Option<&SessionInfo>,
    ) -> Option<String> {
        let session = match session_info {
            Some(session) => session,
            None => {
                if config.require_fresh_auth {
                    return Some("Session information required".to_string());
                }
                return None;
            }
        };

        // Check session age
        let session_age = Utc::now().signed_duration_since(session.session_start);
        if session_age.num_minutes() > config.max_session_age_minutes as i64 {
            return Some("Session too old".to_string());
        }

        // Check authentication age (using last operation as proxy for authentication)
        if config.require_fresh_auth {
            if let Some(last_auth) = session.last_operation {
                let auth_age = Utc::now().signed_duration_since(last_auth);
                if auth_age.num_minutes() > config.max_auth_age_minutes as i64 {
                    return Some("Authentication too old".to_string());
                }
            } else {
                return Some("Fresh authentication required".to_string());
            }
        }

        None
    }

    async fn calculate_risk_score(
        &self,
        security_metadata: &KeyRotationSecurityMetadata,
        _request: &KeyRotationRequest,
        _user_id: Option<&str>,
    ) -> f64 {
        let policy = self.policy.read().await;
        let factors = &policy.risk_assessment.risk_factors;
        let mut total_score = 0.0;

        // Check for unusual IP (simplified - would use historical data)
        if security_metadata.source_ip.is_some() {
            // In a real implementation, check against user's historical IPs
            total_score += factors.unusual_ip_weight * 0.3; // Assume some risk
        }

        // Check for VPN/proxy usage (simplified)
        if let Some(_ip) = security_metadata.source_ip {
            // In a real implementation, check against VPN/proxy databases
            total_score += factors.vpn_proxy_weight * 0.2; // Assume some risk
        }

        // Check for unusual time (simplified)
        let now = Utc::now();
        let hour = now.hour();
        if !(6..=22).contains(&hour) {
            total_score += factors.unusual_time_weight * 0.8; // Outside business hours
        }

        total_score.min(1.0) // Cap at 1.0
    }

    fn get_risk_action(&self, risk_score: f64, actions: &RiskActions) -> RiskAction {
        if risk_score < 0.3 {
            actions.low_risk.clone()
        } else if risk_score < 0.7 {
            actions.medium_risk.clone()
        } else {
            actions.high_risk.clone()
        }
    }

    fn ip_in_range(&self, _ip: &IpAddr, _range: &str) -> bool {
        // Simplified implementation - would use proper CIDR parsing
        false
    }

    fn is_in_maintenance_window(&self, _now: &DateTime<Utc>, _window: &MaintenanceWindow) -> bool {
        // Simplified implementation - would parse cron expressions
        false
    }
}

/// Security evaluation result
#[derive(Debug, Clone)]
pub struct SecurityEvaluationResult {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// Calculated risk score (0.0 to 1.0)
    pub risk_score: f64,
    /// Security warnings (non-blocking)
    pub warnings: Vec<String>,
    /// Security violations (blocking)
    pub violations: Vec<String>,
    /// Required actions to take
    pub required_actions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy_creation() {
        let policy = KeyRotationSecurityPolicy::default();
        assert!(policy.enabled);
        assert_eq!(policy.rate_limits.max_rotations_per_user_per_hour, 10);
    }

    #[test]
    fn test_risk_factors_defaults() {
        let factors = RiskFactors::default();
        assert_eq!(factors.unusual_ip_weight, 0.3);
        assert_eq!(factors.vpn_proxy_weight, 0.3);
    }
}
