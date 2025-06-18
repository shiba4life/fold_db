//! Configuration management for authentication systems in DataFold node
//!
//! This module provides comprehensive configuration management for all aspects
//! of the authentication system including security profiles, timing windows,
//! rate limiting, attack detection, and response security settings.

use crate::datafold_node::error::NodeResult;
use crate::error::FoldDbError;
use crate::security_types::Severity;
use serde::{Deserialize, Serialize};

use super::auth_types::SecurityProfile;

/// Configuration for signature verification middleware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureAuthConfig {
    /// Security profile determining validation strictness
    pub security_profile: SecurityProfile,
    /// Allowed time window for signature timestamps (seconds)
    pub allowed_time_window_secs: u64,
    /// Clock skew tolerance for client-server time differences (seconds)
    pub clock_skew_tolerance_secs: u64,
    /// TTL for nonces in replay prevention store (seconds)
    pub nonce_ttl_secs: u64,
    /// Maximum number of nonces to store in memory
    pub max_nonce_store_size: usize,
    /// Whether to enforce RFC 3339 timestamp format validation
    pub enforce_rfc3339_timestamps: bool,
    /// Whether to require UUID4 format for nonces
    pub require_uuid4_nonces: bool,
    /// Maximum allowed future timestamp drift (seconds)
    pub max_future_timestamp_secs: u64,
    /// Required signature components that must be signed
    pub required_signature_components: Vec<String>,
    /// Enable detailed logging for replay attempts
    pub log_replay_attempts: bool,
    /// Security logging configuration
    pub security_logging: SecurityLoggingConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Attack pattern detection settings
    pub attack_detection: AttackDetectionConfig,
    /// Response security settings
    pub response_security: ResponseSecurityConfig,
}

/// Security logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLoggingConfig {
    /// Enable structured security logging
    pub enabled: bool,
    /// Include request correlation IDs
    pub include_correlation_ids: bool,
    /// Include client IP addresses and user agents
    pub include_client_info: bool,
    /// Include performance metrics in logs
    pub include_performance_metrics: bool,
    /// Log successful authentications (not just failures)
    pub log_successful_auth: bool,
    /// Minimum severity level for security events
    pub min_severity: Severity,
    /// Maximum log entry size in bytes
    pub max_log_entry_size: usize,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Maximum requests per client per window
    pub max_requests_per_window: usize,
    /// Time window for rate limiting in seconds
    pub window_size_secs: u64,
    /// Track failures separately from successes
    pub track_failures_separately: bool,
    /// Failure rate limit (per window)
    pub max_failures_per_window: usize,
}

/// Attack pattern detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDetectionConfig {
    /// Enable attack pattern detection
    pub enabled: bool,
    /// Threshold for brute force detection (failures in time window)
    pub brute_force_threshold: usize,
    /// Time window for brute force detection in seconds
    pub brute_force_window_secs: u64,
    /// Threshold for replay attack detection
    pub replay_threshold: usize,
    /// Enable timing attack protection
    pub enable_timing_protection: bool,
    /// Base response delay in milliseconds
    pub base_response_delay_ms: u64,
}

/// Response security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSecurityConfig {
    /// Include security headers in error responses
    pub include_security_headers: bool,
    /// Consistent timing for all responses
    pub consistent_timing: bool,
    /// Detailed error messages (for debugging, disable in production)
    pub detailed_error_messages: bool,
    /// Include correlation ID in error responses
    pub include_correlation_id: bool,
}

impl Default for SignatureAuthConfig {
    fn default() -> Self {
        Self {
            security_profile: SecurityProfile::default(),
            allowed_time_window_secs: 300, // 5 minutes
            clock_skew_tolerance_secs: 30, // 30 seconds tolerance for clock differences
            nonce_ttl_secs: 300,
            max_nonce_store_size: 10000, // Maximum 10k nonces in memory
            enforce_rfc3339_timestamps: true,
            require_uuid4_nonces: true,
            max_future_timestamp_secs: 60, // Allow up to 1 minute in the future
            required_signature_components: vec![
                "@method".to_string(),
                "@target-uri".to_string(),
                "content-type".to_string(),
                "content-digest".to_string(),
            ],
            log_replay_attempts: true,
            security_logging: SecurityLoggingConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            attack_detection: AttackDetectionConfig::default(),
            response_security: ResponseSecurityConfig::default(),
        }
    }
}

impl Default for SecurityLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_correlation_ids: true,
            include_client_info: true,
            include_performance_metrics: true,
            log_successful_auth: false, // Only log failures by default for performance
            min_severity: Severity::Info,
            max_log_entry_size: 8192, // 8KB max log entry
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_window: 100,
            window_size_secs: 60, // 1 minute window
            track_failures_separately: true,
            max_failures_per_window: 10, // More restrictive for failures
        }
    }
}

impl Default for AttackDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            brute_force_threshold: 5,     // 5 failures in window
            brute_force_window_secs: 300, // 5 minute window
            replay_threshold: 3,          // 3 replay attempts
            enable_timing_protection: true,
            base_response_delay_ms: 100, // 100ms base delay
        }
    }
}

impl Default for ResponseSecurityConfig {
    fn default() -> Self {
        Self {
            include_security_headers: true,
            consistent_timing: true,
            detailed_error_messages: false, // Disable in production
            include_correlation_id: true,
        }
    }
}

impl SignatureAuthConfig {
    /// Create configuration with strict security profile
    pub fn strict() -> Self {
        Self {
            security_profile: SecurityProfile::Strict,
            allowed_time_window_secs: 60,  // 1 minute
            clock_skew_tolerance_secs: 5,  // 5 seconds tolerance
            max_future_timestamp_secs: 10, // 10 seconds future tolerance
            enforce_rfc3339_timestamps: true,
            require_uuid4_nonces: true,
            log_replay_attempts: true,
            rate_limiting: RateLimitingConfig {
                max_requests_per_window: 50, // More restrictive
                max_failures_per_window: 3,  // Very restrictive
                ..RateLimitingConfig::default()
            },
            attack_detection: AttackDetectionConfig {
                brute_force_threshold: 3, // More sensitive
                enable_timing_protection: true,
                ..AttackDetectionConfig::default()
            },
            response_security: ResponseSecurityConfig {
                detailed_error_messages: false, // Never in strict mode
                ..ResponseSecurityConfig::default()
            },
            ..Self::default()
        }
    }

    /// Create configuration with lenient security profile for development
    pub fn lenient() -> Self {
        Self {
            security_profile: SecurityProfile::Lenient,
            allowed_time_window_secs: 600,  // 10 minutes
            clock_skew_tolerance_secs: 120, // 2 minutes tolerance
            max_future_timestamp_secs: 300, // 5 minutes future tolerance
            enforce_rfc3339_timestamps: false,
            require_uuid4_nonces: false,
            log_replay_attempts: false,
            rate_limiting: RateLimitingConfig {
                enabled: false, // Disable rate limiting in lenient mode
                ..RateLimitingConfig::default()
            },
            attack_detection: AttackDetectionConfig {
                enabled: false, // Disable attack detection in lenient mode
                ..AttackDetectionConfig::default()
            },
            response_security: ResponseSecurityConfig {
                detailed_error_messages: true, // Enable for debugging
                ..ResponseSecurityConfig::default()
            },
            ..Self::default()
        }
    }

    /// Create configuration with production-ready settings
    pub fn production() -> Self {
        Self {
            security_profile: SecurityProfile::Standard,
            allowed_time_window_secs: 300,  // 5 minutes
            clock_skew_tolerance_secs: 30,  // 30 seconds tolerance
            max_future_timestamp_secs: 60,  // 1 minute future tolerance
            enforce_rfc3339_timestamps: true,
            require_uuid4_nonces: true,
            log_replay_attempts: true,
            security_logging: SecurityLoggingConfig {
                enabled: true,
                include_correlation_ids: true,
                include_client_info: false, // Disable for privacy in production
                include_performance_metrics: true,
                log_successful_auth: false, // Performance optimization
                min_severity: Severity::Warning, // Only log warnings and above
                max_log_entry_size: 4096, // Smaller logs in production
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                max_requests_per_window: 200, // Higher limit for production
                window_size_secs: 60,
                track_failures_separately: true,
                max_failures_per_window: 10,
            },
            attack_detection: AttackDetectionConfig {
                enabled: true,
                brute_force_threshold: 5,
                brute_force_window_secs: 300,
                replay_threshold: 3,
                enable_timing_protection: true,
                base_response_delay_ms: 50, // Faster responses in production
            },
            response_security: ResponseSecurityConfig {
                include_security_headers: true,
                consistent_timing: true,
                detailed_error_messages: false, // Security: no detailed errors
                include_correlation_id: true,
            },
            ..Self::default()
        }
    }

    /// Create configuration optimized for development environments
    pub fn development() -> Self {
        Self {
            security_profile: SecurityProfile::Standard,
            allowed_time_window_secs: 600,  // Longer window for dev
            clock_skew_tolerance_secs: 60,  // More tolerance for dev
            max_future_timestamp_secs: 120, // More future tolerance
            enforce_rfc3339_timestamps: false,
            require_uuid4_nonces: false,
            log_replay_attempts: true,
            security_logging: SecurityLoggingConfig {
                enabled: true,
                include_correlation_ids: true,
                include_client_info: true,
                include_performance_metrics: true,
                log_successful_auth: true, // Log everything in dev
                min_severity: Severity::Info, // Log all levels
                max_log_entry_size: 16384, // Larger logs for debugging
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                max_requests_per_window: 1000, // High limit for dev
                window_size_secs: 60,
                track_failures_separately: true,
                max_failures_per_window: 50, // Lenient failure limit
            },
            attack_detection: AttackDetectionConfig {
                enabled: true,
                brute_force_threshold: 10, // Less sensitive
                brute_force_window_secs: 300,
                replay_threshold: 5,
                enable_timing_protection: false, // Faster dev experience
                base_response_delay_ms: 0, // No artificial delays
            },
            response_security: ResponseSecurityConfig {
                include_security_headers: true,
                consistent_timing: false, // Faster dev responses
                detailed_error_messages: true, // Enable for debugging
                include_correlation_id: true,
            },
            ..Self::default()
        }
    }

    /// Create configuration for testing environments
    pub fn testing() -> Self {
        Self {
            security_profile: SecurityProfile::Lenient,
            allowed_time_window_secs: 3600, // Very long window for tests
            clock_skew_tolerance_secs: 300, // High tolerance
            max_future_timestamp_secs: 600, // Allow far future timestamps
            enforce_rfc3339_timestamps: false,
            require_uuid4_nonces: false,
            log_replay_attempts: false, // Reduce test noise
            security_logging: SecurityLoggingConfig {
                enabled: false, // Disable logging in tests
                include_correlation_ids: true,
                include_client_info: false,
                include_performance_metrics: false,
                log_successful_auth: false,
                min_severity: Severity::Critical, // Only critical events
                max_log_entry_size: 1024,
            },
            rate_limiting: RateLimitingConfig {
                enabled: false, // Disable for testing
                max_requests_per_window: 10000,
                window_size_secs: 60,
                track_failures_separately: false,
                max_failures_per_window: 1000,
            },
            attack_detection: AttackDetectionConfig {
                enabled: false, // Disable for testing
                brute_force_threshold: 1000,
                brute_force_window_secs: 3600,
                replay_threshold: 1000,
                enable_timing_protection: false,
                base_response_delay_ms: 0,
            },
            response_security: ResponseSecurityConfig {
                include_security_headers: false,
                consistent_timing: false,
                detailed_error_messages: true, // Helpful for test debugging
                include_correlation_id: true,
            },
            ..Self::default()
        }
    }

    /// Validate the configuration parameters
    pub fn validate(&self) -> NodeResult<()> {
        if self.allowed_time_window_secs == 0 {
            return Err(FoldDbError::Permission(
                "Time window must be greater than 0".to_string(),
            ));
        }

        if self.nonce_ttl_secs == 0 {
            return Err(FoldDbError::Permission(
                "Nonce TTL must be greater than 0".to_string(),
            ));
        }

        if self.max_nonce_store_size == 0 {
            return Err(FoldDbError::Permission(
                "Nonce store size must be greater than 0".to_string(),
            ));
        }

        if self.clock_skew_tolerance_secs > self.allowed_time_window_secs {
            return Err(FoldDbError::Permission(
                "Clock skew tolerance cannot exceed time window".to_string(),
            ));
        }

        if self.required_signature_components.is_empty() {
            return Err(FoldDbError::Permission(
                "At least one signature component must be required".to_string(),
            ));
        }

        // Validate security logging configuration
        if self.security_logging.max_log_entry_size == 0 {
            return Err(FoldDbError::Permission(
                "Log entry size must be greater than 0".to_string(),
            ));
        }

        if self.security_logging.max_log_entry_size > 1048576 {
            // 1MB max
            return Err(FoldDbError::Permission(
                "Log entry size cannot exceed 1MB".to_string(),
            ));
        }

        // Validate rate limiting configuration
        if self.rate_limiting.enabled {
            if self.rate_limiting.max_requests_per_window == 0 {
                return Err(FoldDbError::Permission(
                    "Max requests per window must be greater than 0".to_string(),
                ));
            }

            if self.rate_limiting.window_size_secs == 0 {
                return Err(FoldDbError::Permission(
                    "Rate limiting window size must be greater than 0".to_string(),
                ));
            }

            if self.rate_limiting.track_failures_separately
                && self.rate_limiting.max_failures_per_window == 0
            {
                return Err(FoldDbError::Permission(
                    "Max failures per window must be greater than 0 when tracking failures separately".to_string(),
                ));
            }
        }

        // Validate attack detection configuration
        if self.attack_detection.enabled {
            if self.attack_detection.brute_force_threshold == 0 {
                return Err(FoldDbError::Permission(
                    "Brute force threshold must be greater than 0".to_string(),
                ));
            }

            if self.attack_detection.brute_force_window_secs == 0 {
                return Err(FoldDbError::Permission(
                    "Brute force window size must be greater than 0".to_string(),
                ));
            }

            if self.attack_detection.replay_threshold == 0 {
                return Err(FoldDbError::Permission(
                    "Replay threshold must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get configuration summary for logging
    pub fn summary(&self) -> String {
        format!(
            "AuthConfig[profile={:?}, time_window={}s, nonce_ttl={}s, rate_limit={}, attack_detection={}]",
            self.security_profile,
            self.allowed_time_window_secs,
            self.nonce_ttl_secs,
            self.rate_limiting.enabled,
            self.attack_detection.enabled
        )
    }

    /// Check if this is a production configuration
    pub fn is_production(&self) -> bool {
        !self.response_security.detailed_error_messages
            && self.security_logging.min_severity >= Severity::Warning
            && self.rate_limiting.enabled
            && self.attack_detection.enabled
    }

    /// Check if this is a development configuration
    pub fn is_development(&self) -> bool {
        self.response_security.detailed_error_messages
            && self.security_logging.log_successful_auth
            && !self.attack_detection.enable_timing_protection
    }

    /// Update configuration for specific environment
    pub fn for_environment(mut self, env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "production" | "prod" => Self::production(),
            "development" | "dev" => Self::development(),
            "testing" | "test" => Self::testing(),
            "strict" => Self::strict(),
            "lenient" => Self::lenient(),
            _ => self,
        }
    }

    /// Enable debug mode (detailed errors and logging)
    pub fn with_debug(mut self) -> Self {
        self.response_security.detailed_error_messages = true;
        self.security_logging.log_successful_auth = true;
        self.security_logging.min_severity = Severity::Info;
        self.attack_detection.enable_timing_protection = false;
        self.attack_detection.base_response_delay_ms = 0;
        self
    }

    /// Disable debug mode (production-safe)
    pub fn without_debug(mut self) -> Self {
        self.response_security.detailed_error_messages = false;
        self.security_logging.log_successful_auth = false;
        self.security_logging.min_severity = Severity::Warning;
        self.attack_detection.enable_timing_protection = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = SignatureAuthConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_strict_config_validation() {
        let config = SignatureAuthConfig::strict();
        assert!(config.validate().is_ok());
        assert!(!config.is_development());
    }

    #[test]
    fn test_lenient_config_validation() {
        let config = SignatureAuthConfig::lenient();
        assert!(config.validate().is_ok());
        assert!(!config.rate_limiting.enabled);
        assert!(!config.attack_detection.enabled);
    }

    #[test]
    fn test_production_config() {
        let config = SignatureAuthConfig::production();
        assert!(config.validate().is_ok());
        assert!(config.is_production());
        assert!(!config.is_development());
        assert!(!config.response_security.detailed_error_messages);
    }

    #[test]
    fn test_development_config() {
        let config = SignatureAuthConfig::development();
        assert!(config.validate().is_ok());
        assert!(config.is_development());
        assert!(!config.is_production());
        assert!(config.response_security.detailed_error_messages);
    }

    #[test]
    fn test_testing_config() {
        let config = SignatureAuthConfig::testing();
        assert!(config.validate().is_ok());
        assert!(!config.rate_limiting.enabled);
        assert!(!config.attack_detection.enabled);
        assert!(!config.security_logging.enabled);
    }

    #[test]
    fn test_invalid_config_zero_time_window() {
        let mut config = SignatureAuthConfig::default();
        config.allowed_time_window_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_config_clock_skew_too_large() {
        let mut config = SignatureAuthConfig::default();
        config.clock_skew_tolerance_secs = config.allowed_time_window_secs + 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_for_environment() {
        let base_config = SignatureAuthConfig::default();
        
        let prod_config = base_config.clone().for_environment("production");
        assert!(prod_config.is_production());
        
        let dev_config = base_config.clone().for_environment("development");
        assert!(dev_config.is_development());
        
        let test_config = base_config.clone().for_environment("testing");
        assert!(!test_config.rate_limiting.enabled);
    }

    #[test]
    fn test_debug_mode_toggle() {
        let config = SignatureAuthConfig::production().with_debug();
        assert!(config.response_security.detailed_error_messages);
        assert!(config.security_logging.log_successful_auth);
        
        let config = config.without_debug();
        assert!(!config.response_security.detailed_error_messages);
        assert!(!config.security_logging.log_successful_auth);
    }
}