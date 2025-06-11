//! Ed25519 signature verification middleware for DataFold HTTP server
//!
//! This module implements RFC 9421-compliant HTTP message signatures using Ed25519
//! cryptography for authentication and replay prevention with comprehensive security logging.

use crate::datafold_node::error::NodeResult;
use crate::error::FoldDbError;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
    http::StatusCode,
};
use futures_util::future::LocalBoxFuture;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use std::rc::Rc;
use uuid::Uuid;

/// Enhanced error types for authentication failures with detailed categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthenticationError {
    /// Missing required headers
    MissingHeaders { missing: Vec<String>, correlation_id: String },
    /// Invalid signature format or parsing errors
    InvalidSignatureFormat { reason: String, correlation_id: String },
    /// Signature verification failures
    SignatureVerificationFailed { key_id: String, correlation_id: String },
    /// Timestamp validation failures
    TimestampValidationFailed {
        timestamp: u64,
        current_time: u64,
        reason: String,
        correlation_id: String
    },
    /// Nonce validation failures (replay attempts)
    NonceValidationFailed {
        nonce: String,
        reason: String,
        correlation_id: String
    },
    /// Public key lookup failures
    PublicKeyLookupFailed { key_id: String, correlation_id: String },
    /// Configuration errors
    ConfigurationError { reason: String, correlation_id: String },
    /// Algorithm not supported
    UnsupportedAlgorithm { algorithm: String, correlation_id: String },
    /// Rate limiting triggered
    RateLimitExceeded { client_id: String, correlation_id: String },
}

impl AuthenticationError {
    pub fn correlation_id(&self) -> &str {
        match self {
            Self::MissingHeaders { correlation_id, .. } => correlation_id,
            Self::InvalidSignatureFormat { correlation_id, .. } => correlation_id,
            Self::SignatureVerificationFailed { correlation_id, .. } => correlation_id,
            Self::TimestampValidationFailed { correlation_id, .. } => correlation_id,
            Self::NonceValidationFailed { correlation_id, .. } => correlation_id,
            Self::PublicKeyLookupFailed { correlation_id, .. } => correlation_id,
            Self::ConfigurationError { correlation_id, .. } => correlation_id,
            Self::UnsupportedAlgorithm { correlation_id, .. } => correlation_id,
            Self::RateLimitExceeded { correlation_id, .. } => correlation_id,
        }
    }

    pub fn http_status_code(&self) -> StatusCode {
        match self {
            Self::MissingHeaders { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidSignatureFormat { .. } => StatusCode::BAD_REQUEST,
            Self::SignatureVerificationFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::TimestampValidationFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::NonceValidationFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::PublicKeyLookupFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::ConfigurationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnsupportedAlgorithm { .. } => StatusCode::BAD_REQUEST,
            Self::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    pub fn public_message(&self) -> String {
        match self {
            Self::MissingHeaders { .. } => "Missing required authentication headers".to_string(),
            Self::InvalidSignatureFormat { .. } => "Invalid signature format".to_string(),
            Self::SignatureVerificationFailed { .. } => "Signature verification failed".to_string(),
            Self::TimestampValidationFailed { .. } => "Request timestamp invalid".to_string(),
            Self::NonceValidationFailed { .. } => "Request validation failed".to_string(),
            Self::PublicKeyLookupFailed { .. } => "Authentication failed".to_string(),
            Self::ConfigurationError { .. } => "Internal server error".to_string(),
            Self::UnsupportedAlgorithm { .. } => "Unsupported signature algorithm".to_string(),
            Self::RateLimitExceeded { .. } => "Rate limit exceeded".to_string(),
        }
    }

    pub fn severity(&self) -> SecurityEventSeverity {
        match self {
            Self::MissingHeaders { .. } => SecurityEventSeverity::Info,
            Self::InvalidSignatureFormat { .. } => SecurityEventSeverity::Warn,
            Self::SignatureVerificationFailed { .. } => SecurityEventSeverity::Warn,
            Self::TimestampValidationFailed { .. } => SecurityEventSeverity::Warn,
            Self::NonceValidationFailed { .. } => SecurityEventSeverity::Critical,
            Self::PublicKeyLookupFailed { .. } => SecurityEventSeverity::Warn,
            Self::ConfigurationError { .. } => SecurityEventSeverity::Critical,
            Self::UnsupportedAlgorithm { .. } => SecurityEventSeverity::Info,
            Self::RateLimitExceeded { .. } => SecurityEventSeverity::Critical,
        }
    }
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeaders { missing, correlation_id } => {
                write!(f, "Missing required headers: {} (correlation_id: {})", missing.join(", "), correlation_id)
            }
            Self::InvalidSignatureFormat { reason, correlation_id } => {
                write!(f, "Invalid signature format: {} (correlation_id: {})", reason, correlation_id)
            }
            Self::SignatureVerificationFailed { key_id, correlation_id } => {
                write!(f, "Signature verification failed for key_id: {} (correlation_id: {})", key_id, correlation_id)
            }
            Self::TimestampValidationFailed { timestamp, current_time, reason, correlation_id } => {
                write!(f, "Timestamp validation failed: {} at {} (current: {}) (correlation_id: {})",
                       reason, timestamp, current_time, correlation_id)
            }
            Self::NonceValidationFailed { nonce, reason, correlation_id } => {
                write!(f, "Nonce validation failed for {}: {} (correlation_id: {})", nonce, reason, correlation_id)
            }
            Self::PublicKeyLookupFailed { key_id, correlation_id } => {
                write!(f, "Public key lookup failed for key_id: {} (correlation_id: {})", key_id, correlation_id)
            }
            Self::ConfigurationError { reason, correlation_id } => {
                write!(f, "Configuration error: {} (correlation_id: {})", reason, correlation_id)
            }
            Self::UnsupportedAlgorithm { algorithm, correlation_id } => {
                write!(f, "Unsupported algorithm: {} (correlation_id: {})", algorithm, correlation_id)
            }
            Self::RateLimitExceeded { client_id, correlation_id } => {
                write!(f, "Rate limit exceeded for client: {} (correlation_id: {})", client_id, correlation_id)
            }
        }
    }
}

impl std::error::Error for AuthenticationError {}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityEventSeverity {
    Info,
    Warn,
    Critical,
}

/// Security event types for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: String,
    pub correlation_id: String,
    pub timestamp: u64,
    pub event_type: SecurityEventType,
    pub severity: SecurityEventSeverity,
    pub client_info: ClientInfo,
    pub request_info: RequestInfo,
    pub error_details: Option<AuthenticationError>,
    pub metrics: SecurityMetrics,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    ReplayAttempt,
    RateLimitExceeded,
    SuspiciousActivity,
    ConfigurationError,
}

/// Client information for security logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub key_id: Option<String>,
    pub forwarded_for: Option<String>,
}

/// Request information for security logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub query_params: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub signature_components: Option<Vec<String>>,
}

/// Security metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityMetrics {
    pub processing_time_ms: u64,
    pub nonce_store_size: usize,
    pub recent_failures: usize,
    pub pattern_score: f64,
}

/// Security profile for timestamp and nonce validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum SecurityProfile {
    /// Strict security with tight time windows and validation
    Strict,
    /// Standard security with balanced settings
    #[default]
    Standard,
    /// Lenient security with relaxed validation for development
    Lenient,
}


/// Configuration for signature verification middleware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureAuthConfig {
    /// Whether signature verification is enabled
    pub enabled: bool,
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
    pub min_severity: SecurityEventSeverity,
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
            enabled: true,
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
            min_severity: SecurityEventSeverity::Info,
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
            brute_force_threshold: 5, // 5 failures in window
            brute_force_window_secs: 300, // 5 minute window
            replay_threshold: 3, // 3 replay attempts
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
            allowed_time_window_secs: 60, // 1 minute
            clock_skew_tolerance_secs: 5, // 5 seconds tolerance
            max_future_timestamp_secs: 10, // 10 seconds future tolerance
            enforce_rfc3339_timestamps: true,
            require_uuid4_nonces: true,
            log_replay_attempts: true,
            rate_limiting: RateLimitingConfig {
                max_requests_per_window: 50, // More restrictive
                max_failures_per_window: 3, // Very restrictive
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
            allowed_time_window_secs: 600, // 10 minutes
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

    /// Validate the configuration parameters
    pub fn validate(&self) -> NodeResult<()> {
        if self.allowed_time_window_secs == 0 {
            return Err(FoldDbError::Permission(
                "Time window must be greater than 0".to_string()
            ));
        }

        if self.nonce_ttl_secs == 0 {
            return Err(FoldDbError::Permission(
                "Nonce TTL must be greater than 0".to_string()
            ));
        }

        if self.max_nonce_store_size == 0 {
            return Err(FoldDbError::Permission(
                "Nonce store size must be greater than 0".to_string()
            ));
        }

        if self.clock_skew_tolerance_secs > self.allowed_time_window_secs {
            return Err(FoldDbError::Permission(
                "Clock skew tolerance cannot exceed time window".to_string()
            ));
        }

        Ok(())
    }
}

/// Signature verification middleware state
#[derive(Clone)]
pub struct SignatureVerificationState {
    /// Configuration for signature verification
    config: SignatureAuthConfig,
    /// In-memory store for replay prevention
    nonce_store: Arc<RwLock<NonceStore>>,
    /// Security event logger
    security_logger: Arc<SecurityLogger>,
    /// Rate limiting tracker
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Attack pattern detector
    attack_detector: Arc<RwLock<AttackDetector>>,
    /// Security metrics collector
    metrics_collector: Arc<SecurityMetricsCollector>,
}

/// Security logger for structured security events
#[derive(Clone)]
pub struct SecurityLogger {
    config: SecurityLoggingConfig,
}

/// Rate limiter for preventing abuse
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Track requests per client (IP or key_id)
    client_requests: HashMap<String, Vec<u64>>,
    /// Track failures per client
    client_failures: HashMap<String, Vec<u64>>,
}

/// Attack pattern detector
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AttackDetector {
    /// Brute force attempts per client
    brute_force_attempts: HashMap<String, Vec<u64>>,
    /// Replay attempts per nonce
    replay_attempts: HashMap<String, Vec<u64>>,
    /// Suspicious patterns detected
    _suspicious_patterns: HashMap<String, SuspiciousPattern>,
}

/// Pattern indicating suspicious activity
#[derive(Debug, Clone)]
pub struct SuspiciousPattern {
    pub pattern_type: AttackPatternType,
    pub detection_time: u64,
    pub severity_score: f64,
    pub client_id: String,
}

/// Types of attack patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackPatternType {
    BruteForce,
    ReplayAttack,
    TimingAttack,
    VolumeAnomaly,
    SignatureFormatProbing,
}

/// Security metrics collector
pub struct SecurityMetricsCollector {
    /// Total authentication attempts
    total_attempts: AtomicU64,
    /// Total authentication successes
    total_successes: AtomicU64,
    /// Total authentication failures
    total_failures: AtomicU64,
    /// Processing time measurements
    processing_times: Arc<RwLock<Vec<u64>>>,
    /// Current nonce store utilization
    nonce_store_utilization: AtomicUsize,
}

impl SecurityLogger {
    pub fn new(config: SecurityLoggingConfig) -> Self {
        Self { config }
    }

    pub fn log_security_event(&self, event: SecurityEvent) {
        if !self.config.enabled {
            return;
        }

        // Filter by minimum severity
        if !self.should_log_severity(&event.severity) {
            return;
        }

        // Serialize event to JSON for structured logging
        match self.serialize_event(&event) {
            Ok(json_str) => {
                // Log based on severity level
                match event.severity {
                    SecurityEventSeverity::Info => info!("SECURITY_EVENT: {}", json_str),
                    SecurityEventSeverity::Warn => warn!("SECURITY_EVENT: {}", json_str),
                    SecurityEventSeverity::Critical => error!("SECURITY_EVENT: {}", json_str),
                }
            }
            Err(e) => {
                error!("Failed to serialize security event: {}", e);
            }
        }
    }

    fn should_log_severity(&self, severity: &SecurityEventSeverity) -> bool {
        matches!(
            (&self.config.min_severity, severity),
            (SecurityEventSeverity::Critical, SecurityEventSeverity::Critical)
                | (SecurityEventSeverity::Warn, SecurityEventSeverity::Warn | SecurityEventSeverity::Critical)
                | (SecurityEventSeverity::Info, _)
        )
    }

    fn serialize_event(&self, event: &SecurityEvent) -> Result<String, serde_json::Error> {
        let mut json_str = serde_json::to_string(event)?;
        
        // Enforce maximum log entry size
        if json_str.len() > self.config.max_log_entry_size {
            let truncated = format!("{{\"truncated\": true, \"original_size\": {}, \"message\": \"Event too large\"}}", json_str.len());
            json_str = truncated;
        }

        Ok(json_str)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            client_requests: HashMap::new(),
            client_failures: HashMap::new(),
        }
    }

    pub fn check_rate_limit(&mut self, client_id: &str, config: &RateLimitingConfig, is_failure: bool) -> bool {
        if !config.enabled {
            return true; // Rate limiting disabled
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Clean up old entries first
        self.cleanup_expired_entries(now, config.window_size_secs);

        // Check general request rate limit
        let requests = self.client_requests.entry(client_id.to_string()).or_default();
        requests.push(now);

        if requests.len() > config.max_requests_per_window {
            return false;
        }

        // Check failure rate limit separately if enabled
        if config.track_failures_separately && is_failure {
            let failures = self.client_failures.entry(client_id.to_string()).or_default();
            failures.push(now);

            if failures.len() > config.max_failures_per_window {
                return false;
            }
        }

        true
    }

    fn cleanup_expired_entries(&mut self, now: u64, window_size: u64) {
        let cutoff = now.saturating_sub(window_size);

        // Clean up request tracking
        for requests in self.client_requests.values_mut() {
            requests.retain(|&timestamp| timestamp > cutoff);
        }
        self.client_requests.retain(|_, requests| !requests.is_empty());

        // Clean up failure tracking
        for failures in self.client_failures.values_mut() {
            failures.retain(|&timestamp| timestamp > cutoff);
        }
        self.client_failures.retain(|_, failures| !failures.is_empty());
    }

    pub fn get_client_stats(&self, client_id: &str) -> (usize, usize) {
        let requests = self.client_requests.get(client_id).map(|v| v.len()).unwrap_or(0);
        let failures = self.client_failures.get(client_id).map(|v| v.len()).unwrap_or(0);
        (requests, failures)
    }
}

impl Default for AttackDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AttackDetector {
    pub fn new() -> Self {
        Self {
            brute_force_attempts: HashMap::new(),
            replay_attempts: HashMap::new(),
            _suspicious_patterns: HashMap::new(),
        }
    }

    pub fn detect_attack_patterns(&mut self, client_id: &str, event: &SecurityEvent, config: &AttackDetectionConfig) -> Option<SuspiciousPattern> {
        if !config.enabled {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Clean up old entries
        self.cleanup_expired_patterns(now, config.brute_force_window_secs);

        match &event.error_details {
            Some(auth_error) => {
                match auth_error {
                    AuthenticationError::SignatureVerificationFailed { .. } |
                    AuthenticationError::TimestampValidationFailed { .. } |
                    AuthenticationError::PublicKeyLookupFailed { .. } => {
                        self.track_brute_force_attempt(client_id, now, config)
                    }
                    AuthenticationError::NonceValidationFailed { .. } => {
                        self.track_replay_attempt(client_id, now, config)
                    }
                    _ => None,
                }
            }
            None => None,
        }
    }

    fn track_brute_force_attempt(&mut self, client_id: &str, now: u64, config: &AttackDetectionConfig) -> Option<SuspiciousPattern> {
        let attempts = self.brute_force_attempts.entry(client_id.to_string()).or_default();
        attempts.push(now);

        if attempts.len() >= config.brute_force_threshold {
            Some(SuspiciousPattern {
                pattern_type: AttackPatternType::BruteForce,
                detection_time: now,
                severity_score: (attempts.len() as f64 / config.brute_force_threshold as f64) * 10.0,
                client_id: client_id.to_string(),
            })
        } else {
            None
        }
    }

    fn track_replay_attempt(&mut self, client_id: &str, now: u64, config: &AttackDetectionConfig) -> Option<SuspiciousPattern> {
        let attempts = self.replay_attempts.entry(client_id.to_string()).or_default();
        attempts.push(now);

        if attempts.len() >= config.replay_threshold {
            Some(SuspiciousPattern {
                pattern_type: AttackPatternType::ReplayAttack,
                detection_time: now,
                severity_score: (attempts.len() as f64 / config.replay_threshold as f64) * 15.0, // Higher severity
                client_id: client_id.to_string(),
            })
        } else {
            None
        }
    }

    fn cleanup_expired_patterns(&mut self, now: u64, window_size: u64) {
        let cutoff = now.saturating_sub(window_size);

        for attempts in self.brute_force_attempts.values_mut() {
            attempts.retain(|&timestamp| timestamp > cutoff);
        }
        self.brute_force_attempts.retain(|_, attempts| !attempts.is_empty());

        for attempts in self.replay_attempts.values_mut() {
            attempts.retain(|&timestamp| timestamp > cutoff);
        }
        self.replay_attempts.retain(|_, attempts| !attempts.is_empty());
    }
}

impl Default for SecurityMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityMetricsCollector {
    pub fn new() -> Self {
        Self {
            total_attempts: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            processing_times: Arc::new(RwLock::new(Vec::new())),
            nonce_store_utilization: AtomicUsize::new(0),
        }
    }

    pub fn record_attempt(&self, success: bool, processing_time_ms: u64) {
        self.total_attempts.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.total_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_failures.fetch_add(1, Ordering::Relaxed);
        }

        // Record processing time (keep only recent measurements)
        if let Ok(mut times) = self.processing_times.write() {
            times.push(processing_time_ms);
            if times.len() > 1000 {
                times.drain(0..500); // Keep only the most recent 500 measurements
            }
        }
    }

    pub fn update_nonce_store_utilization(&self, size: usize) {
        self.nonce_store_utilization.store(size, Ordering::Relaxed);
    }

    pub fn get_security_metrics(&self) -> SecurityMetrics {
        let processing_times = self.processing_times.read().unwrap();
        let avg_processing_time = if processing_times.is_empty() {
            0
        } else {
            processing_times.iter().sum::<u64>() / processing_times.len() as u64
        };

        SecurityMetrics {
            processing_time_ms: avg_processing_time,
            nonce_store_size: self.nonce_store_utilization.load(Ordering::Relaxed),
            recent_failures: self.total_failures.load(Ordering::Relaxed) as usize,
            pattern_score: 0.0, // This would be calculated based on detected patterns
        }
    }
}

impl SignatureVerificationState {
    pub fn new(config: SignatureAuthConfig) -> NodeResult<Self> {
        // Validate configuration before creating state
        config.validate()?;
        
        let metrics_collector = Arc::new(SecurityMetricsCollector::new());
        
        Ok(Self {
            config: config.clone(),
            nonce_store: Arc::new(RwLock::new(NonceStore::new())),
            security_logger: Arc::new(SecurityLogger::new(config.security_logging.clone())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            attack_detector: Arc::new(RwLock::new(AttackDetector::new())),
            metrics_collector,
        })
    }

    /// Verify a request signature against the stored public key
    pub async fn verify_signature_against_database(
        &self,
        components: &SignatureComponents,
        req: &actix_web::dev::ServiceRequest,
        app_state: &crate::datafold_node::http_server::AppState,
    ) -> Result<(), AuthenticationError> {
        let correlation_id = self.generate_correlation_id();

        // Get database access and extract db_ops in a scoped block
        let db_ops = {
            let node = app_state.node.lock().await;
            let db_guard = match node.db.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(AuthenticationError::ConfigurationError {
                        reason: "Cannot access database".to_string(),
                        correlation_id,
                    });
                }
            };
            db_guard.db_ops()
        }; // db_guard and node are dropped here

        // Look up the public key
        let public_key_bytes = match self.lookup_public_key(&components.keyid, db_ops, &correlation_id).await {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        // Construct the canonical message
        let canonical_message = match components.construct_canonical_message(req) {
            Ok(message) => message,
            Err(e) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!("Failed to construct canonical message: {}", e),
                    correlation_id,
                });
            }
        };

        // Decode the signature
        let signature_bytes = match hex::decode(&components.signature) {
            Ok(bytes) => {
                if bytes.len() != crate::crypto::ed25519::SIGNATURE_LENGTH {
                    return Err(AuthenticationError::InvalidSignatureFormat {
                        reason: format!("Invalid signature length: expected {}, got {}",
                                       crate::crypto::ed25519::SIGNATURE_LENGTH, bytes.len()),
                        correlation_id,
                    });
                }
                // Convert to fixed-size array
                let mut signature_array = [0u8; crate::crypto::ed25519::SIGNATURE_LENGTH];
                signature_array.copy_from_slice(&bytes);
                signature_array
            }
            Err(_) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: "Invalid hex encoding in signature".to_string(),
                    correlation_id,
                });
            }
        };

        // Verify the signature using Ed25519
        let public_key = match crate::crypto::ed25519::PublicKey::from_bytes(&public_key_bytes) {
            Ok(key) => key,
            Err(e) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!("Invalid public key format: {}", e),
                    correlation_id,
                });
            }
        };

        match public_key.verify(canonical_message.as_bytes(), &signature_bytes) {
            Ok(()) => {
                debug!("Signature verification successful for key_id: {}", components.keyid);
                Ok(())
            }
            Err(e) => {
                warn!("Signature verification failed for key_id: {}: {}", components.keyid, e);
                Err(AuthenticationError::SignatureVerificationFailed {
                    key_id: components.keyid.clone(),
                    correlation_id,
                })
            }
        }
    }

    /// Look up a public key from the database
    async fn lookup_public_key(
        &self,
        key_id: &str,
        db_ops: std::sync::Arc<crate::db_operations::core::DbOperations>,
        correlation_id: &str,
    ) -> Result<[u8; 32], AuthenticationError> {
        use crate::datafold_node::crypto_routes::{PUBLIC_KEY_REGISTRATIONS_TREE, CLIENT_KEY_INDEX_TREE};

        // Look up registration ID by client ID using the same pattern as crypto_routes
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, key_id);
        let registration_id_str = match db_ops.get_item::<String>(&client_index_key) {
            Ok(Some(reg_id)) => reg_id,
            Ok(None) => {
                debug!("No registration found for client_id: {}", key_id);
                return Err(AuthenticationError::PublicKeyLookupFailed {
                    key_id: key_id.to_string(),
                    correlation_id: correlation_id.to_string(),
                });
            }
            Err(e) => {
                error!("Failed to lookup client key index: {}", e);
                return Err(AuthenticationError::PublicKeyLookupFailed {
                    key_id: key_id.to_string(),
                    correlation_id: correlation_id.to_string(),
                });
            }
        };

        // Get registration record using the same pattern as crypto_routes
        let registration_key = format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id_str);
        let registration: crate::datafold_node::crypto_routes::PublicKeyRegistration =
            match db_ops.get_item(&registration_key) {
                Ok(Some(reg)) => reg,
                Ok(None) => {
                    debug!("Registration record not found: {}", registration_id_str);
                    return Err(AuthenticationError::PublicKeyLookupFailed {
                        key_id: key_id.to_string(),
                        correlation_id: correlation_id.to_string(),
                    });
                }
                Err(e) => {
                    error!("Failed to get registration record: {}", e);
                    return Err(AuthenticationError::PublicKeyLookupFailed {
                        key_id: key_id.to_string(),
                        correlation_id: correlation_id.to_string(),
                    });
                }
            };

        // Check if the key is active
        if registration.status != "active" {
            debug!("Public key for {} is not active: {}", key_id, registration.status);
            return Err(AuthenticationError::PublicKeyLookupFailed {
                key_id: key_id.to_string(),
                correlation_id: correlation_id.to_string(),
            });
        }

        debug!("Successfully found active public key for client_id: {}", key_id);
        Ok(registration.public_key_bytes)
    }

    /// Enhanced authentication with comprehensive security logging and error handling
    pub fn authenticate_request(&self, req: &ServiceRequest) -> Result<String, AuthenticationError> {
        let start_time = Instant::now();
        let correlation_id = self.generate_correlation_id();
        
        // Extract client information for logging
        let client_info = self.extract_client_info(req);
        let request_info = self.extract_request_info(req);
        
        // Check rate limiting first
        if let Err(auth_error) = self.check_rate_limits(&client_info, &correlation_id) {
            self.log_authentication_failure(&auth_error, &client_info, &request_info, start_time);
            return Err(auth_error);
        }

        // Parse signature components with enhanced error handling
        let components = match SignatureComponents::parse_from_headers(req) {
            Ok(components) => components,
            Err(e) => {
                let auth_error = self.create_signature_parsing_error(&e, &correlation_id);
                self.log_authentication_failure(&auth_error, &client_info, &request_info, start_time);
                return Err(auth_error);
            }
        };

        // Validate timestamp with enhanced error details
        if let Err(e) = self.validate_timestamp_enhanced(components.created, &correlation_id) {
            self.log_authentication_failure(&e, &client_info, &request_info, start_time);
            return Err(e);
        }

        // Check and store nonce with enhanced error handling
        if let Err(e) = self.check_and_store_nonce_enhanced(&components.nonce, components.created, &correlation_id) {
            self.log_authentication_failure(&e, &client_info, &request_info, start_time);
            return Err(e);
        }

        // Validate required signature components
        if let Err(e) = self.validate_signature_components(&components, &correlation_id) {
            self.log_authentication_failure(&e, &client_info, &request_info, start_time);
            return Err(e);
        }

        // Log successful authentication if configured
        if self.config.security_logging.log_successful_auth {
            self.log_authentication_success(&components.keyid, &client_info, &request_info, start_time);
        }

        // Record success metrics
        self.metrics_collector.record_attempt(true, start_time.elapsed().as_millis() as u64);

        Ok(components.keyid)
    }

    /// Generate correlation ID for request tracking
    fn generate_correlation_id(&self) -> String {
        if self.config.security_logging.include_correlation_ids {
            Uuid::new_v4().to_string()
        } else {
            "disabled".to_string()
        }
    }

    /// Extract client information from request
    fn extract_client_info(&self, req: &ServiceRequest) -> ClientInfo {
        if !self.config.security_logging.include_client_info {
            return ClientInfo {
                ip_address: None,
                user_agent: None,
                key_id: None,
                forwarded_for: None,
            };
        }

        ClientInfo {
            ip_address: req.peer_addr().map(|addr| addr.ip().to_string()),
            user_agent: req.headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            key_id: None, // Will be filled later when available
            forwarded_for: req.headers()
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
        }
    }

    /// Extract request information for logging
    fn extract_request_info(&self, req: &ServiceRequest) -> RequestInfo {
        RequestInfo {
            method: req.method().as_str().to_string(),
            path: req.path().to_string(),
            query_params: req.query_string().is_empty().then(|| req.query_string().to_string()),
            content_type: req.headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            content_length: req.headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            signature_components: None, // Will be filled when available
        }
    }

    /// Check rate limits with enhanced error handling
    fn check_rate_limits(&self, client_info: &ClientInfo, correlation_id: &str) -> Result<(), AuthenticationError> {
        if !self.config.rate_limiting.enabled {
            return Ok(());
        }

        let client_id = client_info.ip_address.as_deref()
            .or(client_info.key_id.as_deref())
            .unwrap_or("unknown");

        let mut rate_limiter = self.rate_limiter.write()
            .map_err(|_| AuthenticationError::ConfigurationError {
                reason: "Rate limiter lock failure".to_string(),
                correlation_id: correlation_id.to_string(),
            })?;

        if !rate_limiter.check_rate_limit(client_id, &self.config.rate_limiting, false) {
            return Err(AuthenticationError::RateLimitExceeded {
                client_id: client_id.to_string(),
                correlation_id: correlation_id.to_string(),
            });
        }

        Ok(())
    }

    /// Create authentication error from signature parsing failure
    fn create_signature_parsing_error(&self, error: &FoldDbError, correlation_id: &str) -> AuthenticationError {
        match error {
            FoldDbError::Permission(msg) if msg.contains("Missing") => {
                let missing_headers = if msg.contains("Signature-Input") {
                    vec!["Signature-Input".to_string()]
                } else if msg.contains("Signature") {
                    vec!["Signature".to_string()]
                } else {
                    vec!["Unknown".to_string()]
                };
                
                AuthenticationError::MissingHeaders {
                    missing: missing_headers,
                    correlation_id: correlation_id.to_string(),
                }
            }
            _ => AuthenticationError::InvalidSignatureFormat {
                reason: error.to_string(),
                correlation_id: correlation_id.to_string(),
            }
        }
    }

    /// Enhanced timestamp validation with detailed error information
    fn validate_timestamp_enhanced(&self, created: u64, correlation_id: &str) -> Result<(), AuthenticationError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthenticationError::ConfigurationError {
                reason: "System time error".to_string(),
                correlation_id: correlation_id.to_string(),
            })?
            .as_secs();

        // Check for future timestamps
        if created > now {
            let future_diff = created - now;
            if future_diff > self.config.max_future_timestamp_secs {
                return Err(AuthenticationError::TimestampValidationFailed {
                    timestamp: created,
                    current_time: now,
                    reason: format!("Timestamp too far in future: {} seconds", future_diff),
                    correlation_id: correlation_id.to_string(),
                });
            }
            
            // Allow small future timestamps within clock skew tolerance
            if future_diff <= self.config.clock_skew_tolerance_secs {
                debug!("Accepting future timestamp within clock skew tolerance: {}s", future_diff);
                return Ok(());
            }
        }

        // Check for past timestamps
        let time_diff = if now >= created {
            now - created
        } else {
            created - now
        };

        let effective_window = self.config.allowed_time_window_secs + self.config.clock_skew_tolerance_secs;

        if time_diff > effective_window {
            return Err(AuthenticationError::TimestampValidationFailed {
                timestamp: created,
                current_time: now,
                reason: format!("Timestamp outside allowed window: {} seconds (max: {})", time_diff, effective_window),
                correlation_id: correlation_id.to_string(),
            });
        }

        Ok(())
    }

    /// Enhanced nonce validation with detailed error information
    fn check_and_store_nonce_enhanced(&self, nonce: &str, created: u64, correlation_id: &str) -> Result<(), AuthenticationError> {
        // Validate timestamp first
        self.validate_timestamp_enhanced(created, correlation_id)?;

        // Validate nonce format
        if let Err(e) = self.validate_nonce_format(nonce) {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: e.to_string(),
                correlation_id: correlation_id.to_string(),
            });
        }

        let mut store = self.nonce_store.write()
            .map_err(|_| AuthenticationError::ConfigurationError {
                reason: "Failed to acquire nonce store lock".to_string(),
                correlation_id: correlation_id.to_string(),
            })?;

        // Clean up expired nonces first
        store.cleanup_expired(self.config.nonce_ttl_secs);

        // Check if nonce already exists (replay attack)
        if store.contains_nonce(nonce) {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "Nonce replay detected".to_string(),
                correlation_id: correlation_id.to_string(),
            });
        }

        // Enforce store size limits
        if store.size() >= self.config.max_nonce_store_size {
            store.enforce_size_limit(self.config.max_nonce_store_size - 1);
        }

        // Store the nonce
        store.add_nonce(nonce.to_string(), created);

        // Update metrics
        self.metrics_collector.update_nonce_store_utilization(store.size());

        Ok(())
    }

    /// Validate required signature components
    fn validate_signature_components(&self, components: &SignatureComponents, correlation_id: &str) -> Result<(), AuthenticationError> {
        for required_component in &self.config.required_signature_components {
            if !components.covered_components.contains(required_component) {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!("Required component '{}' not covered by signature", required_component),
                    correlation_id: correlation_id.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Log authentication failure with security event
    fn log_authentication_failure(&self, error: &AuthenticationError, client_info: &ClientInfo, request_info: &RequestInfo, start_time: Instant) {
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // Record failure metrics
        self.metrics_collector.record_attempt(false, processing_time);

        // Create security event
        let event = SecurityEvent {
            event_id: Uuid::new_v4().to_string(),
            correlation_id: error.correlation_id().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: SecurityEventType::AuthenticationFailure,
            severity: error.severity(),
            client_info: client_info.clone(),
            request_info: request_info.clone(),
            error_details: Some(error.clone()),
            metrics: SecurityMetrics {
                processing_time_ms: processing_time,
                nonce_store_size: self.nonce_store.read().unwrap().size(),
                recent_failures: 1,
                pattern_score: 0.0,
            },
        };

        // Log the security event
        self.security_logger.log_security_event(event.clone());

        // Check for attack patterns
        if let Ok(mut detector) = self.attack_detector.write() {
            if let Some(pattern) = detector.detect_attack_patterns(
                client_info.ip_address.as_deref().unwrap_or("unknown"),
                &event,
                &self.config.attack_detection
            ) {
                // Log suspicious pattern detection
                let pattern_event = SecurityEvent {
                    event_id: Uuid::new_v4().to_string(),
                    correlation_id: error.correlation_id().to_string(),
                    timestamp: pattern.detection_time,
                    event_type: SecurityEventType::SuspiciousActivity,
                    severity: SecurityEventSeverity::Critical,
                    client_info: client_info.clone(),
                    request_info: request_info.clone(),
                    error_details: None,
                    metrics: SecurityMetrics {
                        processing_time_ms: processing_time,
                        nonce_store_size: self.nonce_store.read().unwrap().size(),
                        recent_failures: 1,
                        pattern_score: pattern.severity_score,
                    },
                };
                
                self.security_logger.log_security_event(pattern_event);
            }
        }
    }

    /// Log successful authentication
    fn log_authentication_success(&self, key_id: &str, client_info: &ClientInfo, request_info: &RequestInfo, start_time: Instant) {
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        let mut client_info_with_key = client_info.clone();
        client_info_with_key.key_id = Some(key_id.to_string());

        let event = SecurityEvent {
            event_id: Uuid::new_v4().to_string(),
            correlation_id: self.generate_correlation_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: SecurityEventType::AuthenticationSuccess,
            severity: SecurityEventSeverity::Info,
            client_info: client_info_with_key,
            request_info: request_info.clone(),
            error_details: None,
            metrics: SecurityMetrics {
                processing_time_ms: processing_time,
                nonce_store_size: self.nonce_store.read().unwrap().size(),
                recent_failures: 0,
                pattern_score: 0.0,
            },
        };

        self.security_logger.log_security_event(event);
    }

    /// Get formatted error message for response
    pub fn get_error_message(&self, error: &AuthenticationError) -> String {
        if self.config.response_security.detailed_error_messages {
            format!("{}", error)
        } else {
            error.public_message()
        }
    }

    /// Check if a nonce has been used (and store it if new) - Legacy method for compatibility
    pub fn check_and_store_nonce(&self, nonce: &str, created: u64) -> NodeResult<()> {
        let correlation_id = self.generate_correlation_id();
        match self.check_and_store_nonce_enhanced(nonce, created, &correlation_id) {
            Ok(()) => Ok(()),
            Err(auth_error) => Err(FoldDbError::Permission(auth_error.public_message())),
        }
    }

    // Test helper methods - these should only be used in tests
    #[cfg(test)]
    pub fn generate_correlation_id_for_test(&self) -> String {
        self.generate_correlation_id()
    }

    #[cfg(test)]
    pub fn check_and_store_nonce_enhanced_for_test(&self, nonce: &str, created: u64, correlation_id: &str) -> Result<(), AuthenticationError> {
        self.check_and_store_nonce_enhanced(nonce, created, correlation_id)
    }

    #[cfg(test)]
    pub fn validate_timestamp_enhanced_for_test(&self, created: u64, correlation_id: &str) -> Result<(), AuthenticationError> {
        self.validate_timestamp_enhanced(created, correlation_id)
    }

    #[cfg(test)]
    pub fn get_config(&self) -> &SignatureAuthConfig {
        &self.config
    }

    #[cfg(test)]
    pub fn get_metrics_collector(&self) -> &SecurityMetricsCollector {
        &self.metrics_collector
    }

    #[cfg(test)]
    pub fn get_security_logger(&self) -> &SecurityLogger {
        &self.security_logger
    }

    /// Enhanced timestamp validation with clock skew tolerance and future timestamp checks
    pub fn validate_timestamp(&self, created: u64) -> NodeResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| FoldDbError::Permission("System time error".to_string()))?
            .as_secs();
        
        // Check for future timestamps
        if created > now {
            let future_diff = created - now;
            if future_diff > self.config.max_future_timestamp_secs {
                if self.config.log_replay_attempts {
                    warn!("Future timestamp detected: created={}, now={}, diff={}s",
                          created, now, future_diff);
                }
                return Err(FoldDbError::Permission(
                    format!("Timestamp too far in future: {} seconds", future_diff)
                ));
            }
            
            // Allow small future timestamps within clock skew tolerance
            if future_diff <= self.config.clock_skew_tolerance_secs {
                debug!("Accepting future timestamp within clock skew tolerance: {}s", future_diff);
                return Ok(());
            }
        }
        
        // Check for past timestamps
        let time_diff = if now >= created {
            now - created
        } else {
            created - now
        };
        
        // Apply clock skew tolerance to time window
        let effective_window = self.config.allowed_time_window_secs + self.config.clock_skew_tolerance_secs;
        
        if time_diff > effective_window {
            if self.config.log_replay_attempts {
                warn!("Timestamp outside allowed window: created={}, now={}, diff={}s, window={}s",
                      created, now, time_diff, effective_window);
            }
            return Err(FoldDbError::Permission(
                format!("Timestamp outside allowed window: {} seconds (max: {})",
                        time_diff, effective_window)
            ));
        }
        
        debug!("Timestamp validation successful: diff={}s, window={}s", time_diff, effective_window);
        Ok(())
    }

    /// Validate RFC 3339 timestamp format
    pub fn validate_rfc3339_timestamp(&self, timestamp_str: &str) -> NodeResult<u64> {
        if !self.config.enforce_rfc3339_timestamps {
            // Fallback to simple unix timestamp parsing
            return timestamp_str.parse::<u64>()
                .map_err(|_| FoldDbError::Permission("Invalid timestamp format".to_string()));
        }

        // Simple RFC 3339 validation without external dependencies
        if !self.is_valid_rfc3339_format(timestamp_str) {
            return Err(FoldDbError::Permission("Invalid RFC 3339 timestamp format".to_string()));
        }
        
        // For now, we'll require clients to send unix timestamps
        // Full RFC 3339 parsing would require chrono dependency
        Err(FoldDbError::Permission(
            "RFC 3339 parsing not implemented - please send unix timestamp".to_string()
        ))
    }

    /// Simple RFC 3339 format validation
    fn is_valid_rfc3339_format(&self, timestamp_str: &str) -> bool {
        // Basic pattern check for RFC 3339: YYYY-MM-DDTHH:MM:SSZ
        if timestamp_str.len() < 19 {
            return false;
        }
        
        let chars: Vec<char> = timestamp_str.chars().collect();
        
        // Check basic structure: YYYY-MM-DDTHH:MM:SS
        chars.get(4) == Some(&'-') &&
        chars.get(7) == Some(&'-') &&
        chars.get(10) == Some(&'T') &&
        chars.get(13) == Some(&':') &&
        chars.get(16) == Some(&':') &&
        chars[0..4].iter().all(|c| c.is_ascii_digit()) &&
        chars[5..7].iter().all(|c| c.is_ascii_digit()) &&
        chars[8..10].iter().all(|c| c.is_ascii_digit()) &&
        chars[11..13].iter().all(|c| c.is_ascii_digit()) &&
        chars[14..16].iter().all(|c| c.is_ascii_digit()) &&
        chars[17..19].iter().all(|c| c.is_ascii_digit())
    }

    /// Validate nonce format (UUID4 if required)
    fn validate_nonce_format(&self, nonce: &str) -> NodeResult<()> {
        if self.config.require_uuid4_nonces {
            self.validate_uuid4_format(nonce)?;
            return Ok(()); // If UUID4 validation passes, we're done
        }
        
        // Basic validation for non-UUID nonces
        if nonce.is_empty() {
            return Err(FoldDbError::Permission("Nonce cannot be empty".to_string()));
        }
        
        if nonce.len() > 128 {
            return Err(FoldDbError::Permission("Nonce too long (max 128 characters)".to_string()));
        }
        
        // Ensure nonce contains only safe characters (alphanumeric, hyphens, underscores)
        if !nonce.chars().all(|c| c.is_alphanumeric() || "-_".contains(c)) {
            return Err(FoldDbError::Permission(
                "Nonce contains invalid characters (only alphanumeric, -, _ allowed)".to_string()
            ));
        }
        
        Ok(())
    }

    /// Simple UUID4 format validation without external dependencies
    fn validate_uuid4_format(&self, nonce: &str) -> NodeResult<()> {
        // UUID4 format: 8-4-4-4-12 hexadecimal digits with hyphens
        // Example: 550e8400-e29b-41d4-a716-446655440000
        if nonce.len() != 36 {
            return Err(FoldDbError::Permission("UUID must be 36 characters long".to_string()));
        }
        
        let chars: Vec<char> = nonce.chars().collect();
        
        // Check hyphen positions
        if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
            return Err(FoldDbError::Permission("Invalid UUID format".to_string()));
        }
        
        // Check that version is 4 (position 14)
        if chars[14] != '4' {
            return Err(FoldDbError::Permission("Nonce must be UUID version 4".to_string()));
        }
        
        // Check that all other characters are hexadecimal
        for (i, &c) in chars.iter().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                continue; // Skip hyphens
            }
            if !c.is_ascii_hexdigit() {
                return Err(FoldDbError::Permission("UUID contains non-hexadecimal characters".to_string()));
            }
        }
        
        Ok(())
    }

    /// Get current nonce store statistics
    pub fn get_nonce_store_stats(&self) -> NodeResult<NonceStoreStats> {
        let store = self.nonce_store.read()
            .map_err(|_| FoldDbError::Permission("Failed to acquire read lock".to_string()))?;
        
        Ok(NonceStoreStats {
            total_nonces: store.size(),
            max_capacity: self.config.max_nonce_store_size,
            oldest_nonce_age: store.get_oldest_nonce_age(),
        })
    }
}

/// Statistics about the nonce store
#[derive(Debug, Clone, Default)]
pub struct NonceStoreStats {
    pub total_nonces: usize,
    pub max_capacity: usize,
    pub oldest_nonce_age: Option<u64>,
}

/// In-memory nonce store for replay prevention with advanced features
#[derive(Debug)]
struct NonceStore {
    /// Map of nonce to creation timestamp
    nonces: HashMap<String, u64>,
}

impl NonceStore {
    fn new() -> Self {
        Self {
            nonces: HashMap::new(),
        }
    }

    fn contains_nonce(&self, nonce: &str) -> bool {
        self.nonces.contains_key(nonce)
    }

    fn add_nonce(&mut self, nonce: String, created: u64) {
        self.nonces.insert(nonce, created);
    }

    fn size(&self) -> usize {
        self.nonces.len()
    }

    fn cleanup_expired(&mut self, ttl_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let initial_size = self.nonces.len();
        self.nonces.retain(|_, &mut created| {
            now.saturating_sub(created) < ttl_secs
        });
        
        let removed = initial_size - self.nonces.len();
        if removed > 0 {
            debug!("Cleaned up {} expired nonces, {} remaining", removed, self.nonces.len());
        }
    }

    /// Enforce size limits by removing oldest nonces
    fn enforce_size_limit(&mut self, max_size: usize) {
        if self.nonces.len() <= max_size {
            return;
        }

        let to_remove = self.nonces.len() - max_size;
        
        // Collect all nonces with timestamps first
        let mut nonce_timestamps: Vec<(String, u64)> = self.nonces.iter()
            .map(|(nonce, &timestamp)| (nonce.clone(), timestamp))
            .collect();
        
        // Sort by timestamp (oldest first)
        nonce_timestamps.sort_by_key(|(_, timestamp)| *timestamp);
        
        // Remove the oldest nonces
        for (nonce, _) in nonce_timestamps.into_iter().take(to_remove) {
            self.nonces.remove(&nonce);
        }
        
        warn!("Enforced nonce store size limit: removed {} oldest nonces, {} remaining",
              to_remove, self.nonces.len());
    }

    /// Get the age of the oldest nonce in seconds
    fn get_oldest_nonce_age(&self) -> Option<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.nonces.values()
            .min()
            .map(|&oldest| now.saturating_sub(oldest))
    }

    /// Get sliding window statistics for monitoring
    fn _get_sliding_window_stats(&self, window_secs: u64) -> SlidingWindowStats {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let window_start = now.saturating_sub(window_secs);
        let recent_count = self.nonces.values()
            .filter(|&&timestamp| timestamp >= window_start)
            .count();
        
        SlidingWindowStats {
            window_secs,
            recent_nonces: recent_count,
            total_nonces: self.nonces.len(),
            requests_per_second: recent_count as f64 / window_secs as f64,
        }
    }
}

/// Statistics for sliding window analysis
#[derive(Debug, Clone)]
pub struct SlidingWindowStats {
    pub window_secs: u64,
    pub recent_nonces: usize,
    pub total_nonces: usize,
    pub requests_per_second: f64,
}

/// Parsed signature components from RFC 9421 headers
#[derive(Debug)]
#[allow(dead_code)]
pub struct SignatureComponents {
    signature_input: String,
    signature: String,
    created: u64,
    keyid: String,
    algorithm: String,
    nonce: String,
    covered_components: Vec<String>,
}

impl SignatureComponents {
    /// Parse signature components from HTTP headers
    fn parse_from_headers(req: &ServiceRequest) -> NodeResult<Self> {
        // Extract Signature-Input header
        let signature_input = req
            .headers()
            .get("signature-input")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| FoldDbError::Permission("Missing Signature-Input header".to_string()))?;

        // Extract Signature header  
        let signature = req
            .headers()
            .get("signature")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| FoldDbError::Permission("Missing Signature header".to_string()))?;

        // Parse signature input components
        let (covered_components, params) = Self::parse_signature_input(signature_input)?;
        
        // Extract required parameters
        let created = params.get("created")
            .ok_or_else(|| FoldDbError::Permission("Missing 'created' parameter".to_string()))?
            .parse::<u64>()
            .map_err(|_| FoldDbError::Permission("Invalid 'created' timestamp".to_string()))?;

        let keyid = params.get("keyid")
            .ok_or_else(|| FoldDbError::Permission("Missing 'keyid' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        let algorithm = params.get("alg")
            .ok_or_else(|| FoldDbError::Permission("Missing 'alg' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        let nonce = params.get("nonce")
            .ok_or_else(|| FoldDbError::Permission("Missing 'nonce' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        // Validate algorithm
        if algorithm != "ed25519" {
            return Err(FoldDbError::Permission(
                format!("Unsupported algorithm: {}", algorithm)
            ));
        }

        Ok(Self {
            signature_input: signature_input.to_string(),
            signature: signature.to_string(),
            created,
            keyid,
            algorithm,
            nonce,
            covered_components,
        })
    }

    /// Parse the signature-input header to extract covered components and parameters
    fn parse_signature_input(input: &str) -> NodeResult<(Vec<String>, HashMap<String, String>)> {
        // Find the signature name and its definition
        // Format: sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key-ed25519";alg="ed25519";nonce="abc123"
        
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(FoldDbError::Permission("Invalid signature-input format".to_string()));
        }

        let definition = parts[1];
        
        // Split on semicolon to get components and parameters
        let mut components = Vec::new();
        let mut params = HashMap::new();
        
        let sections: Vec<&str> = definition.split(';').collect();
        
        // First section should be the covered components in parentheses
        if let Some(components_str) = sections.first() {
            let components_str = components_str.trim();
            if !components_str.starts_with('(') || !components_str.ends_with(')') {
                return Err(FoldDbError::Permission("Invalid components format".to_string()));
            }
            
            let inner = &components_str[1..components_str.len()-1];
            for component in inner.split_whitespace() {
                components.push(component.trim_matches('"').to_string());
            }
        }
        
        // Parse parameters
        for section in sections.iter().skip(1) {
            let param_parts: Vec<&str> = section.splitn(2, '=').collect();
            if param_parts.len() == 2 {
                let key = param_parts[0].trim();
                let value = param_parts[1].trim();
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        Ok((components, params))
    }

    /// Construct the canonical message for signature verification
    fn construct_canonical_message(&self, req: &ServiceRequest) -> NodeResult<String> {
        let mut lines = Vec::new();
        
        for component in &self.covered_components {
            let line = match component.as_str() {
                "@method" => {
                    format!("\"@method\": {}", req.method().as_str())
                }
                "@target-uri" => {
                    let uri = req.uri();
                    let target_uri = format!("{}{}", 
                        uri.path(),
                        uri.query().map(|q| format!("?{}", q)).unwrap_or_default()
                    );
                    format!("\"@target-uri\": {}", target_uri)
                }
                header_name => {
                    // Regular header
                    let header_value = req
                        .headers()
                        .get(header_name)
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("");
                    format!("\"{}\": {}", header_name, header_value)
                }
            };
            lines.push(line);
        }
        
        // Add signature parameters
        lines.push(format!("\"@signature-params\": {}", self.build_signature_params()));
        
        Ok(lines.join("\n"))
    }

    /// Build the signature parameters line
    fn build_signature_params(&self) -> String {
        let components_str = self.covered_components
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(" ");
        
        format!("({})", components_str)
    }
}

/// Signature verification middleware
pub struct SignatureVerificationMiddleware {
    state: Rc<SignatureVerificationState>,
}

impl SignatureVerificationMiddleware {
    pub fn new(state: SignatureVerificationState) -> Self {
        Self {
            state: Rc::new(state),
        }
    }

    pub fn try_new(config: SignatureAuthConfig) -> NodeResult<Self> {
        let state = SignatureVerificationState::new(config)?;
        Ok(Self::new(state))
    }
}

impl<S, B> Transform<S, ServiceRequest> for SignatureVerificationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SignatureVerificationService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(SignatureVerificationService {
            service: Rc::new(service),
            state: self.state.clone(),
        }))
    }
}

pub struct SignatureVerificationService<S> {
    service: Rc<S>,
    state: Rc<SignatureVerificationState>,
}

impl<S, B> Service<ServiceRequest> for SignatureVerificationService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let state = self.state.clone();

        Box::pin(async move {
            let start_time = Instant::now();
            
            // Skip signature verification if disabled
            if !state.config.enabled {
                debug!("Signature verification disabled, skipping");
                return service.call(req).await;
            }

            // Skip signature verification for certain paths (health checks, etc.)
            if should_skip_verification(req.path()) {
                debug!("Skipping signature verification for path: {}", req.path());
                return service.call(req).await;
            }

            // Perform enhanced signature verification with comprehensive error handling
            match state.authenticate_request(&req) {
                Ok(client_id) => {
                    // Add client ID to request extensions for downstream use
                    req.extensions_mut().insert(AuthenticatedClient {
                        client_id: client_id.clone()
                    });
                    
                    info!("Successfully verified signature for client {} on path {}", client_id, req.path());
                    
                    // Apply timing protection if enabled
                    if state.config.attack_detection.enable_timing_protection {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        if elapsed < state.config.attack_detection.base_response_delay_ms {
                            let delay = state.config.attack_detection.base_response_delay_ms - elapsed;
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                        }
                    }
                    
                    service.call(req).await
                }
                Err(auth_error) => {
                    error!("Authentication failed for {}: {}", req.path(), auth_error.public_message());
                    
                    // Apply consistent timing for error responses
                    if state.config.response_security.consistent_timing {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        if elapsed < state.config.attack_detection.base_response_delay_ms {
                            let delay = state.config.attack_detection.base_response_delay_ms - elapsed;
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                        }
                    }
                    
                    // Create error message with appropriate detail level
                    let error_message = state.get_error_message(&auth_error);
                    
                    // Return appropriate actix_web error based on authentication error type
                    match auth_error.http_status_code() {
                        StatusCode::BAD_REQUEST => Err(actix_web::error::ErrorBadRequest(error_message)),
                        StatusCode::UNAUTHORIZED => Err(actix_web::error::ErrorUnauthorized(error_message)),
                        StatusCode::TOO_MANY_REQUESTS => Err(actix_web::error::ErrorTooManyRequests(error_message)),
                        StatusCode::INTERNAL_SERVER_ERROR => Err(actix_web::error::ErrorInternalServerError(error_message)),
                        _ => Err(actix_web::error::ErrorUnauthorized(error_message)),
                    }
                }
            }
        })
    }
}

/// Information about an authenticated client
#[derive(Debug, Clone)]
pub struct AuthenticatedClient {
    pub client_id: String,
}

/// Verify the signature of an HTTP request
#[allow(dead_code)]
async fn verify_request_signature(
    req: &ServiceRequest,
    state: &SignatureVerificationState,
    app_state: &crate::datafold_node::http_server::AppState,
) -> NodeResult<String> {
    // Parse signature components from headers
    let components = SignatureComponents::parse_from_headers(req)?;
    
    // Validate timestamp
    state.validate_timestamp(components.created)?;
    
    // Check and store nonce for replay prevention
    state.check_and_store_nonce(&components.nonce, components.created)?;
    
    // Validate required signature components are covered
    for required_component in &state.config.required_signature_components {
        if !components.covered_components.contains(required_component) {
            return Err(FoldDbError::Permission(
                format!("Required component '{}' not covered by signature", required_component)
            ));
        }
    }
    
    // Verify the signature against the stored public key
    match state.verify_signature_against_database(&components, req, app_state).await {
        Ok(()) => {
            info!("Signature verification successful for client: {}", components.keyid);
            Ok(components.keyid)
        }
        Err(auth_error) => {
            warn!("Signature verification failed for client {}: {}", components.keyid, auth_error);
            Err(FoldDbError::Permission(format!("Signature verification failed: {}", auth_error)))
        }
    }
}

/// Check if signature verification should be skipped for this path
fn should_skip_verification(path: &str) -> bool {
    // Skip verification for these paths
    const SKIP_PATHS: &[&str] = &[
        "/api/system/status",
        "/api/crypto/status", 
        "/api/crypto/keys/register",
        "/",
        "/index.html",
    ];
    
    SKIP_PATHS.iter().any(|&skip_path| path.starts_with(skip_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
    use actix_web::{test, web, App, HttpResponse};
    
    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().json("success")
    }
    
    #[actix_web::test]
    async fn test_signature_verification_success() {
        // Create test configuration
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        // Create test app with middleware
        let app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/test", web::get().to(test_handler))
        ).await;
        
        // TODO: Create a properly signed request and test verification
        // This would require implementing the full signature creation process
        // and integration with the existing public key database storage
    }
    
    #[tokio::test]
    async fn test_signature_input_parsing() {
        let input = r#"sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key";alg="ed25519";nonce="abc123""#;
        
        let (components, params) = SignatureComponents::parse_signature_input(input).unwrap();
        
        assert_eq!(components, vec!["@method", "@target-uri", "content-type"]);
        assert_eq!(params.get("created"), Some(&"1618884473".to_string()));
        assert_eq!(params.get("keyid"), Some(&"\"test-key\"".to_string()));
        assert_eq!(params.get("alg"), Some(&"\"ed25519\"".to_string()));
        assert_eq!(params.get("nonce"), Some(&"\"abc123\"".to_string()));
    }
    
    #[tokio::test]
    async fn test_nonce_store() {
        let mut store = NonceStore::new();
        
        // Test adding and checking nonces
        assert!(!store.contains_nonce("test-nonce"));
        store.add_nonce("test-nonce".to_string(), 1234567890);
        assert!(store.contains_nonce("test-nonce"));
        
        // Test cleanup (this would need to be tested with time manipulation)
        store.cleanup_expired(300);
    }
}