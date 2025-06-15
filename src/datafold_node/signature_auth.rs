//! Ed25519 signature verification middleware for DataFold HTTP server
//!
//! This module implements RFC 9421-compliant HTTP message signatures using Ed25519
//! cryptography for authentication and replay prevention with comprehensive security logging.

use crate::datafold_node::error::NodeResult;
use crate::error::FoldDbError;
use crate::security_types::Severity;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc, RwLock,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Cache warmup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmupResult {
    pub keys_loaded: u64,
    pub errors: u64,
    pub duration_ms: u64,
    pub cache_size_after: u64,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub hit_rate: f64,
    pub warmup_completed: bool,
}

/// Performance metrics for monitoring dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub security_metrics: SecurityMetrics,
    pub nonce_store_stats: NonceStorePerformanceStats,
    pub cache_stats: CacheStats,
    pub performance_alerts: Vec<PerformanceAlert>,
    pub system_health: SystemHealthStatus,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub status: String,
    pub health_score: f64,
    pub issues: Vec<String>,
    pub last_updated: u64,
}

/// Nonce store performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NonceStorePerformanceStats {
    pub total_nonces: usize,
    pub max_capacity: usize,
    pub utilization_percent: f64,
    pub oldest_nonce_age_secs: Option<u64>,
    pub cleanup_operations: u64,
    pub memory_usage_bytes: u64,
}

/// Performance breakdown for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBreakdown {
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_processing_time_ms: f64,
    pub avg_signature_verification_ms: f64,
    pub avg_database_lookup_ms: f64,
    pub avg_nonce_validation_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub requests_per_second: f64,
}

/// Enhanced error types for authentication failures with detailed categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthenticationError {
    /// Missing required headers
    MissingHeaders {
        missing: Vec<String>,
        correlation_id: String,
    },
    /// Invalid signature format or parsing errors
    InvalidSignatureFormat {
        reason: String,
        correlation_id: String,
    },
    /// Signature verification failures
    SignatureVerificationFailed {
        key_id: String,
        correlation_id: String,
    },
    /// Timestamp validation failures
    TimestampValidationFailed {
        timestamp: u64,
        current_time: u64,
        reason: String,
        correlation_id: String,
    },
    /// Nonce validation failures (replay attempts)
    NonceValidationFailed {
        nonce: String,
        reason: String,
        correlation_id: String,
    },
    /// Public key lookup failures
    PublicKeyLookupFailed {
        key_id: String,
        correlation_id: String,
    },
    /// Configuration errors
    ConfigurationError {
        reason: String,
        correlation_id: String,
    },
    /// Algorithm not supported
    UnsupportedAlgorithm {
        algorithm: String,
        correlation_id: String,
    },
    /// Rate limiting triggered
    RateLimitExceeded {
        client_id: String,
        correlation_id: String,
    },
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

    /// Get error code for programmatic error handling
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::MissingHeaders { .. } => "MISSING_HEADERS",
            Self::InvalidSignatureFormat { .. } => "INVALID_SIGNATURE_FORMAT",
            Self::SignatureVerificationFailed { .. } => "SIGNATURE_VERIFICATION_FAILED",
            Self::TimestampValidationFailed { .. } => "TIMESTAMP_VALIDATION_FAILED",
            Self::NonceValidationFailed { .. } => "NONCE_VALIDATION_FAILED",
            Self::PublicKeyLookupFailed { .. } => "PUBLIC_KEY_LOOKUP_FAILED",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::UnsupportedAlgorithm { .. } => "UNSUPPORTED_ALGORITHM",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
        }
    }

    pub fn public_message(&self) -> String {
        match self {
            Self::MissingHeaders { .. } => "Missing required authentication headers. Please include Signature-Input and Signature headers.".to_string(),
            Self::InvalidSignatureFormat { .. } => "Invalid signature format. Please verify your signature encoding and header format.".to_string(),
            Self::SignatureVerificationFailed { .. } => "Signature verification failed. Please check your signature calculation and key.".to_string(),
            Self::TimestampValidationFailed { .. } => "Request timestamp invalid. Please ensure timestamp is within allowed time window.".to_string(),
            Self::NonceValidationFailed { .. } => "Request validation failed. Please use a unique nonce for each request.".to_string(),
            Self::PublicKeyLookupFailed { .. } => "Authentication failed. Please verify your key ID is registered.".to_string(),
            Self::ConfigurationError { .. } => "Internal server error. Please contact system administrator.".to_string(),
            Self::UnsupportedAlgorithm { .. } => "Unsupported signature algorithm. Please use ed25519.".to_string(),
            Self::RateLimitExceeded { .. } => "Rate limit exceeded. Please reduce request frequency and try again later.".to_string(),
        }
    }

    /// Get user-friendly error message with troubleshooting guidance
    pub fn user_friendly_message(&self, environment: &str) -> String {
        let base_message = self.public_message();
        let troubleshooting = self.get_troubleshooting_guidance();
        let docs_link = self.get_documentation_link();

        if environment == "development" {
            format!(
                "{}\n\nTroubleshooting:\n{}\n\nFor more help: {}",
                base_message, troubleshooting, docs_link
            )
        } else {
            format!(
                "{}. For assistance, reference error ID: {} or visit: {}",
                base_message,
                self.correlation_id(),
                docs_link
            )
        }
    }

    /// Get specific troubleshooting guidance for each error type
    pub fn get_troubleshooting_guidance(&self) -> String {
        match self {
            Self::MissingHeaders { missing, .. } => {
                format!(
                    "Missing headers: {}. \
                • Ensure your HTTP client includes both 'Signature-Input' and 'Signature' headers\n\
                • Verify header names are lowercase\n\
                • Check your signature library configuration",
                    missing.join(", ")
                )
            }
            Self::InvalidSignatureFormat { reason, .. } => {
                format!("Signature format error: {}\n\
                • Verify signature is hex-encoded\n\
                • Check signature-input format: sig1=(\"@method\" \"@target-uri\");created=timestamp;keyid=\"your-key\";alg=\"ed25519\";nonce=\"unique-nonce\"\n\
                • Ensure all required parameters are present: created, keyid, alg, nonce", reason)
            }
            Self::SignatureVerificationFailed { key_id, .. } => {
                format!(
                    "Signature verification failed for key: {}\n\
                • Verify the canonical message construction matches the server\n\
                • Check that covered components match exactly\n\
                • Ensure private key corresponds to registered public key\n\
                • Verify Ed25519 signature calculation",
                    key_id
                )
            }
            Self::TimestampValidationFailed {
                timestamp,
                current_time,
                reason,
                ..
            } => {
                format!(
                    "Timestamp error: {} (request: {}, server: {})\n\
                • Check system clock synchronization (NTP)\n\
                • Ensure timestamp is Unix epoch seconds\n\
                • Verify timestamp is not too old or too far in future\n\
                • Allow for reasonable clock skew",
                    reason, timestamp, current_time
                )
            }
            Self::NonceValidationFailed { nonce, reason, .. } => {
                format!(
                    "Nonce validation failed for '{}': {}\n\
                • Use a unique nonce for every request\n\
                • Ensure nonce follows required format (UUID4 if configured)\n\
                • Check nonce length restrictions\n\
                • Verify nonce contains only allowed characters",
                    nonce, reason
                )
            }
            Self::PublicKeyLookupFailed { key_id, .. } => {
                format!(
                    "Key lookup failed for: {}\n\
                • Verify the key ID is correctly registered\n\
                • Check key registration status (must be 'active')\n\
                • Ensure key ID matches exactly (case-sensitive)\n\
                • Contact administrator if key should be registered",
                    key_id
                )
            }
            Self::ConfigurationError { reason, .. } => {
                format!(
                    "Server configuration error: {}\n\
                • This is a server-side issue\n\
                • Contact system administrator\n\
                • Provide correlation ID for debugging",
                    reason
                )
            }
            Self::UnsupportedAlgorithm { algorithm, .. } => {
                format!(
                    "Unsupported algorithm: {}\n\
                • Use 'ed25519' as the signature algorithm\n\
                • Update signature-input header: alg=\"ed25519\"\n\
                • Verify signature library supports Ed25519",
                    algorithm
                )
            }
            Self::RateLimitExceeded { client_id, .. } => {
                format!(
                    "Rate limit exceeded for client: {}\n\
                • Reduce request frequency\n\
                • Implement exponential backoff retry logic\n\
                • Contact administrator if limits seem too restrictive\n\
                • Check for duplicate or unnecessary requests",
                    client_id
                )
            }
        }
    }

    /// Get documentation link for specific error type
    pub fn get_documentation_link(&self) -> String {
        match self {
            Self::MissingHeaders { .. } | Self::InvalidSignatureFormat { .. } => {
                "https://docs.datafold.dev/signature-auth/setup"
            }
            Self::SignatureVerificationFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/troubleshooting#signature-verification"
            }
            Self::TimestampValidationFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/troubleshooting#timestamp-issues"
            }
            Self::NonceValidationFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/troubleshooting#nonce-validation"
            }
            Self::PublicKeyLookupFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/key-management"
            }
            Self::ConfigurationError { .. } => {
                "https://docs.datafold.dev/signature-auth/server-config"
            }
            Self::UnsupportedAlgorithm { .. } => {
                "https://docs.datafold.dev/signature-auth/algorithms"
            }
            Self::RateLimitExceeded { .. } => {
                "https://docs.datafold.dev/signature-auth/rate-limits"
            }
        }
        .to_string()
    }

    /// Get suggested next steps for resolving the error
    pub fn get_suggested_actions(&self) -> Vec<String> {
        match self {
            Self::MissingHeaders { .. } => vec![
                "Include both Signature-Input and Signature headers in your request".to_string(),
                "Verify your HTTP client configuration".to_string(),
                "Check signature library documentation".to_string(),
            ],
            Self::InvalidSignatureFormat { .. } => vec![
                "Validate signature-input header format".to_string(),
                "Verify signature is properly hex-encoded".to_string(),
                "Test with signature validation endpoint".to_string(),
            ],
            Self::SignatureVerificationFailed { .. } => vec![
                "Test signature generation with validation endpoint".to_string(),
                "Verify private/public key pair consistency".to_string(),
                "Check canonical message construction".to_string(),
            ],
            Self::TimestampValidationFailed { .. } => vec![
                "Synchronize system clock with NTP".to_string(),
                "Check server time window configuration".to_string(),
                "Use current Unix timestamp".to_string(),
            ],
            Self::NonceValidationFailed { .. } => vec![
                "Generate a new unique nonce for each request".to_string(),
                "Verify nonce format requirements".to_string(),
                "Check nonce length and character restrictions".to_string(),
            ],
            Self::PublicKeyLookupFailed { .. } => vec![
                "Register your public key with the server".to_string(),
                "Verify key ID matches registration".to_string(),
                "Check key status is 'active'".to_string(),
            ],
            Self::ConfigurationError { .. } => vec![
                "Contact system administrator".to_string(),
                "Provide correlation ID for support".to_string(),
                "Check server logs if accessible".to_string(),
            ],
            Self::UnsupportedAlgorithm { .. } => vec![
                "Change algorithm to 'ed25519'".to_string(),
                "Update signature library configuration".to_string(),
                "Regenerate signatures with Ed25519".to_string(),
            ],
            Self::RateLimitExceeded { .. } => vec![
                "Implement request throttling".to_string(),
                "Use exponential backoff retry logic".to_string(),
                "Review and optimize request patterns".to_string(),
            ],
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            Self::MissingHeaders { .. } => Severity::Info,
            Self::InvalidSignatureFormat { .. } => Severity::Warning,
            Self::SignatureVerificationFailed { .. } => Severity::Warning,
            Self::TimestampValidationFailed { .. } => Severity::Warning,
            Self::NonceValidationFailed { .. } => Severity::Critical,
            Self::PublicKeyLookupFailed { .. } => Severity::Warning,
            Self::ConfigurationError { .. } => Severity::Critical,
            Self::UnsupportedAlgorithm { .. } => Severity::Info,
            Self::RateLimitExceeded { .. } => Severity::Critical,
        }
    }
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeaders {
                missing,
                correlation_id,
            } => {
                write!(
                    f,
                    "Missing required headers: {} (correlation_id: {})",
                    missing.join(", "),
                    correlation_id
                )
            }
            Self::InvalidSignatureFormat {
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Invalid signature format: {} (correlation_id: {})",
                    reason, correlation_id
                )
            }
            Self::SignatureVerificationFailed {
                key_id,
                correlation_id,
            } => {
                write!(
                    f,
                    "Signature verification failed for key_id: {} (correlation_id: {})",
                    key_id, correlation_id
                )
            }
            Self::TimestampValidationFailed {
                timestamp,
                current_time,
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Timestamp validation failed: {} at {} (current: {}) (correlation_id: {})",
                    reason, timestamp, current_time, correlation_id
                )
            }
            Self::NonceValidationFailed {
                nonce,
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Nonce validation failed for {}: {} (correlation_id: {})",
                    nonce, reason, correlation_id
                )
            }
            Self::PublicKeyLookupFailed {
                key_id,
                correlation_id,
            } => {
                write!(
                    f,
                    "Public key lookup failed for key_id: {} (correlation_id: {})",
                    key_id, correlation_id
                )
            }
            Self::ConfigurationError {
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Configuration error: {} (correlation_id: {})",
                    reason, correlation_id
                )
            }
            Self::UnsupportedAlgorithm {
                algorithm,
                correlation_id,
            } => {
                write!(
                    f,
                    "Unsupported algorithm: {} (correlation_id: {})",
                    algorithm, correlation_id
                )
            }
            Self::RateLimitExceeded {
                client_id,
                correlation_id,
            } => {
                write!(
                    f,
                    "Rate limit exceeded for client: {} (correlation_id: {})",
                    client_id, correlation_id
                )
            }
        }
    }
}

impl std::error::Error for AuthenticationError {}

/// Security event types for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: String,
    pub correlation_id: String,
    pub timestamp: u64,
    pub event_type: SecurityEventType,
    pub severity: Severity,
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

/// Enhanced security metrics for comprehensive performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityMetrics {
    pub processing_time_ms: u64,
    pub signature_verification_time_ms: u64,
    pub database_lookup_time_ms: u64,
    pub nonce_validation_time_ms: u64,
    pub nonce_store_size: usize,
    pub nonce_store_utilization_percent: f64,
    pub recent_failures: usize,
    pub pattern_score: f64,
    pub cache_hit_rate: f64,
    pub cache_miss_count: u64,
    pub requests_per_second: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub memory_usage_bytes: u64,
    pub nonce_cleanup_operations: u64,
    pub performance_alert_count: u64,
}

/// Security profile for timestamp and nonce validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
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

        Ok(())
    }
}

/// Enhanced signature verification middleware state with performance optimizations
#[derive(Clone)]
pub struct SignatureVerificationState {
    /// Configuration for signature verification
    config: SignatureAuthConfig,
    /// High-performance nonce store for replay prevention
    nonce_store: Arc<RwLock<NonceStore>>,
    /// Public key cache for performance optimization
    key_cache: Arc<RwLock<PublicKeyCache>>,
    /// Security event logger
    security_logger: Arc<SecurityLogger>,
    /// Rate limiting tracker
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Attack pattern detector
    attack_detector: Arc<RwLock<AttackDetector>>,
    /// Enhanced security metrics collector
    metrics_collector: Arc<EnhancedSecurityMetricsCollector>,
    /// Performance monitor for real-time tracking
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
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

/// Enhanced security metrics collector with comprehensive performance tracking
pub struct EnhancedSecurityMetricsCollector {
    /// Total authentication attempts
    total_attempts: AtomicU64,
    /// Total authentication successes
    total_successes: AtomicU64,
    /// Total authentication failures
    total_failures: AtomicU64,
    /// Processing time measurements (with histogram buckets)
    processing_times: Arc<RwLock<LatencyHistogram>>,
    /// Signature verification times
    signature_verification_times: Arc<RwLock<LatencyHistogram>>,
    /// Database lookup times
    database_lookup_times: Arc<RwLock<LatencyHistogram>>,
    /// Nonce validation times
    nonce_validation_times: Arc<RwLock<LatencyHistogram>>,
    /// Current nonce store utilization
    nonce_store_utilization: AtomicUsize,
    /// Cache performance metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    /// Request rate tracking
    request_timestamps: Arc<RwLock<VecDeque<Instant>>>,
    /// Memory usage tracking
    memory_usage_bytes: AtomicU64,
    /// Performance alert counters
    performance_alerts: AtomicU64,
    /// Cleanup operation counters
    cleanup_operations: AtomicU64,
}

/// Latency histogram for performance monitoring
#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    measurements: VecDeque<u64>,
    buckets: HashMap<u64, u64>, // bucket_ms -> count
    max_measurements: usize,
}

impl LatencyHistogram {
    pub fn new(max_measurements: usize) -> Self {
        let mut buckets = HashMap::new();
        // Initialize buckets for latency ranges: <1ms, <5ms, <10ms, <50ms, <100ms, <500ms, <1s, >=1s
        for &bucket in &[1, 5, 10, 50, 100, 500, 1000, u64::MAX] {
            buckets.insert(bucket, 0);
        }

        Self {
            measurements: VecDeque::new(),
            buckets,
            max_measurements,
        }
    }

    pub fn record(&mut self, latency_ms: u64) {
        // Add to measurements
        self.measurements.push_back(latency_ms);
        if self.measurements.len() > self.max_measurements {
            self.measurements.pop_front();
        }

        // Update histogram buckets
        for &bucket in &[1, 5, 10, 50, 100, 500, 1000, u64::MAX] {
            if latency_ms < bucket {
                *self.buckets.entry(bucket).or_insert(0) += 1;
                break;
            }
        }
    }

    pub fn percentile(&self, p: f64) -> Option<u64> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = self.measurements.iter().copied().collect();
        sorted.sort_unstable();

        let index = ((sorted.len() as f64 * p / 100.0) as usize).saturating_sub(1);
        sorted.get(index).copied()
    }

    pub fn average(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: u64 = self.measurements.iter().sum();
        sum as f64 / self.measurements.len() as f64
    }

    pub fn count(&self) -> usize {
        self.measurements.len()
    }
}

/// Public key cache for performance optimization
#[derive(Debug)]
pub struct PublicKeyCache {
    /// Cached public keys by key_id
    keys: HashMap<String, CachedPublicKey>,
    /// Cache hit statistics
    hit_count: u64,
    /// Cache miss statistics
    miss_count: u64,
    /// Maximum cache size
    max_size: usize,
    /// Cache warmup status
    warmup_completed: bool,
    /// Last cleanup timestamp
    last_cleanup: Instant,
}

/// Cached public key entry
#[derive(Debug, Clone)]
pub struct CachedPublicKey {
    /// The public key bytes
    pub key_bytes: [u8; 32],
    /// Timestamp when cached
    pub cached_at: Instant,
    /// Number of times this key was accessed
    pub access_count: u64,
    /// Last access timestamp
    pub last_accessed: Instant,
    /// Key status (active, inactive, etc.)
    pub status: String,
}

impl PublicKeyCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            keys: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
            max_size,
            warmup_completed: false,
            last_cleanup: Instant::now(),
        }
    }

    pub fn get(&mut self, key_id: &str) -> Option<CachedPublicKey> {
        if let Some(cached_key) = self.keys.get_mut(key_id) {
            self.hit_count += 1;
            cached_key.access_count += 1;
            cached_key.last_accessed = Instant::now();
            Some(cached_key.clone())
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn put(&mut self, key_id: String, key_bytes: [u8; 32], status: String) {
        // Enforce cache size limit
        if self.keys.len() >= self.max_size {
            self.evict_least_recently_used();
        }

        let cached_key = CachedPublicKey {
            key_bytes,
            cached_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
            status,
        };

        self.keys.insert(key_id, cached_key);
    }

    pub fn invalidate(&mut self, key_id: &str) {
        self.keys.remove(key_id);
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.hit_count = 0;
        self.miss_count = 0;
        self.warmup_completed = false;
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }

    pub fn size(&self) -> usize {
        self.keys.len()
    }

    pub fn is_warmup_completed(&self) -> bool {
        self.warmup_completed
    }

    pub fn mark_warmup_completed(&mut self) {
        self.warmup_completed = true;
    }

    fn evict_least_recently_used(&mut self) {
        if let Some((lru_key, _)) = self
            .keys
            .iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.keys.remove(&lru_key);
        }
    }

    pub fn cleanup_expired(&mut self, ttl: Duration) {
        let now = Instant::now();
        let cutoff = now - ttl;

        self.keys.retain(|_, cached| cached.cached_at > cutoff);
        self.last_cleanup = now;
    }
}

/// Performance monitor for real-time tracking
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Current requests per second
    #[allow(dead_code)]
    current_rps: f64,
    /// Recent latency measurements
    recent_latencies: VecDeque<u64>,
    /// Performance alerts
    alerts: Vec<PerformanceAlert>,
    /// Monitoring start time
    #[allow(dead_code)]
    start_time: Instant,
    /// Last monitoring update
    last_update: Instant,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_id: String,
    pub timestamp_ms: u64, // Unix timestamp in milliseconds
    pub alert_type: PerformanceAlertType,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub severity: Severity,
}

/// Types of performance alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlertType {
    HighLatency,
    LowCacheHitRate,
    HighMemoryUsage,
    NonceStoreOverflow,
    DatabaseSlowdown,
    RequestRateSpike,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            current_rps: 0.0,
            recent_latencies: VecDeque::new(),
            alerts: Vec::new(),
            start_time: Instant::now(),
            last_update: Instant::now(),
        }
    }

    pub fn record_request(&mut self, latency_ms: u64) {
        self.recent_latencies.push_back(latency_ms);

        // Keep only recent measurements (last 60 seconds)
        while self.recent_latencies.len() > 1000 {
            self.recent_latencies.pop_front();
        }

        self.last_update = Instant::now();
    }

    pub fn check_performance_thresholds(&mut self, _config: &SignatureAuthConfig) {
        let _now = Instant::now();

        // Check latency threshold (>100ms)
        if let Some(avg_latency) = self.get_average_latency() {
            if avg_latency > 100.0 {
                self.create_alert(
                    PerformanceAlertType::HighLatency,
                    format!("Average latency {}ms exceeds threshold", avg_latency),
                    avg_latency,
                    100.0,
                    Severity::Warning,
                );
            }
        }

        // Additional threshold checks can be added here
    }

    pub fn get_average_latency(&self) -> Option<f64> {
        if self.recent_latencies.is_empty() {
            return None;
        }

        let sum: u64 = self.recent_latencies.iter().sum();
        Some(sum as f64 / self.recent_latencies.len() as f64)
    }

    pub fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        self.alerts.iter().rev().take(limit).cloned().collect()
    }

    fn create_alert(
        &mut self,
        alert_type: PerformanceAlertType,
        message: String,
        metric_value: f64,
        threshold: f64,
        severity: Severity,
    ) {
        let alert = PerformanceAlert {
            alert_id: Uuid::new_v4().to_string(),
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            alert_type,
            message,
            metric_value,
            threshold,
            severity,
        };

        self.alerts.push(alert);

        // Limit stored alerts
        if self.alerts.len() > 1000 {
            self.alerts.drain(0..100);
        }
    }
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
                    Severity::Info => info!("SECURITY_EVENT: {}", json_str),
                    Severity::Warning => warn!("SECURITY_EVENT: {}", json_str),
                    Severity::Error | Severity::Critical => error!("SECURITY_EVENT: {}", json_str),
                }
            }
            Err(e) => {
                error!("Failed to serialize security event: {}", e);
            }
        }
    }

    fn should_log_severity(&self, severity: &Severity) -> bool {
        matches!(
            (&self.config.min_severity, severity),
            (Severity::Critical, Severity::Critical)
                | (
                    Severity::Warning,
                    Severity::Warning | Severity::Error | Severity::Critical
                )
                | (Severity::Info, _)
        )
    }

    fn serialize_event(&self, event: &SecurityEvent) -> Result<String, serde_json::Error> {
        let mut json_str = serde_json::to_string(event)?;

        // Enforce maximum log entry size
        if json_str.len() > self.config.max_log_entry_size {
            let truncated = format!(
                "{{\"truncated\": true, \"original_size\": {}, \"message\": \"Event too large\"}}",
                json_str.len()
            );
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

    pub fn check_rate_limit(
        &mut self,
        client_id: &str,
        config: &RateLimitingConfig,
        is_failure: bool,
    ) -> bool {
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
        let requests = self
            .client_requests
            .entry(client_id.to_string())
            .or_default();
        requests.push(now);

        if requests.len() > config.max_requests_per_window {
            return false;
        }

        // Check failure rate limit separately if enabled
        if config.track_failures_separately && is_failure {
            let failures = self
                .client_failures
                .entry(client_id.to_string())
                .or_default();
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
        self.client_requests
            .retain(|_, requests| !requests.is_empty());

        // Clean up failure tracking
        for failures in self.client_failures.values_mut() {
            failures.retain(|&timestamp| timestamp > cutoff);
        }
        self.client_failures
            .retain(|_, failures| !failures.is_empty());
    }

    pub fn get_client_stats(&self, client_id: &str) -> (usize, usize) {
        let requests = self
            .client_requests
            .get(client_id)
            .map(|v| v.len())
            .unwrap_or(0);
        let failures = self
            .client_failures
            .get(client_id)
            .map(|v| v.len())
            .unwrap_or(0);
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

    pub fn detect_attack_patterns(
        &mut self,
        client_id: &str,
        event: &SecurityEvent,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
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
            Some(auth_error) => match auth_error {
                AuthenticationError::SignatureVerificationFailed { .. }
                | AuthenticationError::TimestampValidationFailed { .. }
                | AuthenticationError::PublicKeyLookupFailed { .. } => {
                    self.track_brute_force_attempt(client_id, now, config)
                }
                AuthenticationError::NonceValidationFailed { .. } => {
                    self.track_replay_attempt(client_id, now, config)
                }
                _ => None,
            },
            None => None,
        }
    }

    fn track_brute_force_attempt(
        &mut self,
        client_id: &str,
        now: u64,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
        let attempts = self
            .brute_force_attempts
            .entry(client_id.to_string())
            .or_default();
        attempts.push(now);

        if attempts.len() >= config.brute_force_threshold {
            Some(SuspiciousPattern {
                pattern_type: AttackPatternType::BruteForce,
                detection_time: now,
                severity_score: (attempts.len() as f64 / config.brute_force_threshold as f64)
                    * 10.0,
                client_id: client_id.to_string(),
            })
        } else {
            None
        }
    }

    fn track_replay_attempt(
        &mut self,
        client_id: &str,
        now: u64,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
        let attempts = self
            .replay_attempts
            .entry(client_id.to_string())
            .or_default();
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
        self.brute_force_attempts
            .retain(|_, attempts| !attempts.is_empty());

        for attempts in self.replay_attempts.values_mut() {
            attempts.retain(|&timestamp| timestamp > cutoff);
        }
        self.replay_attempts
            .retain(|_, attempts| !attempts.is_empty());
    }
}

impl Default for EnhancedSecurityMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedSecurityMetricsCollector {
    pub fn new() -> Self {
        Self {
            total_attempts: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            processing_times: Arc::new(RwLock::new(LatencyHistogram::new(10000))),
            signature_verification_times: Arc::new(RwLock::new(LatencyHistogram::new(10000))),
            database_lookup_times: Arc::new(RwLock::new(LatencyHistogram::new(10000))),
            nonce_validation_times: Arc::new(RwLock::new(LatencyHistogram::new(10000))),
            nonce_store_utilization: AtomicUsize::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            request_timestamps: Arc::new(RwLock::new(VecDeque::new())),
            memory_usage_bytes: AtomicU64::new(0),
            performance_alerts: AtomicU64::new(0),
            cleanup_operations: AtomicU64::new(0),
        }
    }

    pub fn record_attempt(&self, success: bool, processing_time_ms: u64) {
        self.total_attempts.fetch_add(1, Ordering::Relaxed);

        if success {
            self.total_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_failures.fetch_add(1, Ordering::Relaxed);
        }

        // Record processing time in histogram
        if let Ok(mut times) = self.processing_times.write() {
            times.record(processing_time_ms);
        }

        // Track request timestamps for RPS calculation
        if let Ok(mut timestamps) = self.request_timestamps.write() {
            timestamps.push_back(Instant::now());
            // Keep only last 60 seconds
            let cutoff = Instant::now() - Duration::from_secs(60);
            while let Some(&front) = timestamps.front() {
                if front < cutoff {
                    timestamps.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    pub fn update_nonce_store_utilization(&self, size: usize) {
        self.nonce_store_utilization.store(size, Ordering::Relaxed);
    }

    pub fn get_enhanced_security_metrics(&self, nonce_store_max_size: usize) -> SecurityMetrics {
        let processing_times = self.processing_times.read().unwrap();
        let sig_times = self.signature_verification_times.read().unwrap();
        let db_times = self.database_lookup_times.read().unwrap();
        let nonce_times = self.nonce_validation_times.read().unwrap();

        let nonce_utilization = self.nonce_store_utilization.load(Ordering::Relaxed);
        let utilization_percent = if nonce_store_max_size > 0 {
            (nonce_utilization as f64 / nonce_store_max_size as f64) * 100.0
        } else {
            0.0
        };

        SecurityMetrics {
            processing_time_ms: processing_times.average() as u64,
            signature_verification_time_ms: sig_times.average() as u64,
            database_lookup_time_ms: db_times.average() as u64,
            nonce_validation_time_ms: nonce_times.average() as u64,
            nonce_store_size: nonce_utilization,
            nonce_store_utilization_percent: utilization_percent,
            recent_failures: self.total_failures.load(Ordering::Relaxed) as usize,
            pattern_score: 0.0,
            cache_hit_rate: self.get_cache_hit_rate(),
            cache_miss_count: self.cache_misses.load(Ordering::Relaxed),
            requests_per_second: self.get_requests_per_second(),
            avg_latency_ms: processing_times.average(),
            p95_latency_ms: processing_times.percentile(95.0).unwrap_or(0) as f64,
            p99_latency_ms: processing_times.percentile(99.0).unwrap_or(0) as f64,
            memory_usage_bytes: self.memory_usage_bytes.load(Ordering::Relaxed),
            nonce_cleanup_operations: self.cleanup_operations.load(Ordering::Relaxed),
            performance_alert_count: self.performance_alerts.load(Ordering::Relaxed),
        }
    }

    fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    fn get_requests_per_second(&self) -> f64 {
        if let Ok(timestamps) = self.request_timestamps.read() {
            let count = timestamps.len();
            if count == 0 {
                return 0.0;
            }

            // Calculate RPS over the last 60 seconds
            let duration =
                if let (Some(&first), Some(&last)) = (timestamps.front(), timestamps.back()) {
                    last.duration_since(first).as_secs_f64()
                } else {
                    1.0
                };

            count as f64 / duration.max(1.0)
        } else {
            0.0
        }
    }
}

impl SignatureVerificationState {
    pub fn new(config: SignatureAuthConfig) -> NodeResult<Self> {
        // Validate configuration before creating state
        config.validate()?;

        let metrics_collector = Arc::new(EnhancedSecurityMetricsCollector::new());

        Ok(Self {
            config: config.clone(),
            nonce_store: Arc::new(RwLock::new(NonceStore::new())),
            key_cache: Arc::new(RwLock::new(PublicKeyCache::new(1000))),
            security_logger: Arc::new(SecurityLogger::new(config.security_logging.clone())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            attack_detector: Arc::new(RwLock::new(AttackDetector::new())),
            metrics_collector,
            performance_monitor: Arc::new(RwLock::new(PerformanceMonitor::new())),
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
        let public_key_bytes = match self
            .lookup_public_key(&components.keyid, db_ops, &correlation_id)
            .await
        {
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
                        reason: format!(
                            "Invalid signature length: expected {}, got {}",
                            crate::crypto::ed25519::SIGNATURE_LENGTH,
                            bytes.len()
                        ),
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
                debug!(
                    "Signature verification successful for key_id: {}",
                    components.keyid
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Signature verification failed for key_id: {}: {}",
                    components.keyid, e
                );
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
        use crate::datafold_node::crypto_routes::{
            CLIENT_KEY_INDEX_TREE, PUBLIC_KEY_REGISTRATIONS_TREE,
        };

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
        let registration_key =
            format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id_str);
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
            debug!(
                "Public key for {} is not active: {}",
                key_id, registration.status
            );
            return Err(AuthenticationError::PublicKeyLookupFailed {
                key_id: key_id.to_string(),
                correlation_id: correlation_id.to_string(),
            });
        }

        debug!(
            "Successfully found active public key for client_id: {}",
            key_id
        );
        Ok(registration.public_key_bytes)
    }

    /// Enhanced authentication with comprehensive security logging and error handling
    pub fn authenticate_request(
        &self,
        req: &ServiceRequest,
    ) -> Result<String, AuthenticationError> {
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
                self.log_authentication_failure(
                    &auth_error,
                    &client_info,
                    &request_info,
                    start_time,
                );
                return Err(auth_error);
            }
        };

        // TODO: Integrate unified validation once security module is stable
        // For now, use existing legacy validation flow
        // Validate timestamp with enhanced error details
        if let Err(e) = self.validate_timestamp_enhanced(components.created, &correlation_id) {
            self.log_authentication_failure(&e, &client_info, &request_info, start_time);
            return Err(e);
        }

        // Check and store nonce with enhanced error handling
        if let Err(e) = self.check_and_store_nonce_enhanced(
            &components.nonce,
            components.created,
            &correlation_id,
        ) {
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
            self.log_authentication_success(
                &components.keyid,
                &client_info,
                &request_info,
                start_time,
            );
        }

        // Record success metrics
        self.metrics_collector
            .record_attempt(true, start_time.elapsed().as_millis() as u64);

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
            user_agent: req
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            key_id: None, // Will be filled later when available
            forwarded_for: req
                .headers()
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
            query_params: req
                .query_string()
                .is_empty()
                .then(|| req.query_string().to_string()),
            content_type: req
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            content_length: req
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            signature_components: None, // Will be filled when available
        }
    }

    /// Check rate limits with enhanced error handling
    fn check_rate_limits(
        &self,
        client_info: &ClientInfo,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
        if !self.config.rate_limiting.enabled {
            return Ok(());
        }

        let client_id = client_info
            .ip_address
            .as_deref()
            .or(client_info.key_id.as_deref())
            .unwrap_or("unknown");

        let mut rate_limiter =
            self.rate_limiter
                .write()
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
    fn create_signature_parsing_error(
        &self,
        error: &FoldDbError,
        correlation_id: &str,
    ) -> AuthenticationError {
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
            },
        }
    }

    /// Enhanced timestamp validation with detailed error information
    fn validate_timestamp_enhanced(
        &self,
        created: u64,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
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
                debug!(
                    "Accepting future timestamp within clock skew tolerance: {}s",
                    future_diff
                );
                return Ok(());
            }
        }

        // Check for past timestamps
        let time_diff = if now >= created {
            now - created
        } else {
            created - now
        };

        let effective_window =
            self.config.allowed_time_window_secs + self.config.clock_skew_tolerance_secs;

        if time_diff > effective_window {
            return Err(AuthenticationError::TimestampValidationFailed {
                timestamp: created,
                current_time: now,
                reason: format!(
                    "Timestamp outside allowed window: {} seconds (max: {})",
                    time_diff, effective_window
                ),
                correlation_id: correlation_id.to_string(),
            });
        }

        Ok(())
    }

    /// Enhanced nonce validation with detailed error information
    fn check_and_store_nonce_enhanced(
        &self,
        nonce: &str,
        created: u64,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
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

        let mut store =
            self.nonce_store
                .write()
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
        self.metrics_collector
            .update_nonce_store_utilization(store.size());

        Ok(())
    }

    /// Validate required signature components
    fn validate_signature_components(
        &self,
        components: &SignatureComponents,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
        for required_component in &self.config.required_signature_components {
            if !components.covered_components.contains(required_component) {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!(
                        "Required component '{}' not covered by signature",
                        required_component
                    ),
                    correlation_id: correlation_id.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Log authentication failure with security event
    fn log_authentication_failure(
        &self,
        error: &AuthenticationError,
        client_info: &ClientInfo,
        request_info: &RequestInfo,
        start_time: Instant,
    ) {
        let processing_time = start_time.elapsed().as_millis() as u64;

        // Record failure metrics
        self.metrics_collector
            .record_attempt(false, processing_time);

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
                signature_verification_time_ms: 0,
                database_lookup_time_ms: 0,
                nonce_validation_time_ms: 0,
                nonce_store_size: self.nonce_store.read().unwrap().size(),
                nonce_store_utilization_percent: 0.0,
                recent_failures: 1,
                pattern_score: 0.0,
                cache_hit_rate: 0.0,
                cache_miss_count: 0,
                requests_per_second: 0.0,
                avg_latency_ms: processing_time as f64,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                memory_usage_bytes: 0,
                nonce_cleanup_operations: 0,
                performance_alert_count: 0,
            },
        };

        // Log the security event
        self.security_logger.log_security_event(event.clone());

        // Check for attack patterns
        if let Ok(mut detector) = self.attack_detector.write() {
            if let Some(pattern) = detector.detect_attack_patterns(
                client_info.ip_address.as_deref().unwrap_or("unknown"),
                &event,
                &self.config.attack_detection,
            ) {
                // Log suspicious pattern detection
                let pattern_event = SecurityEvent {
                    event_id: Uuid::new_v4().to_string(),
                    correlation_id: error.correlation_id().to_string(),
                    timestamp: pattern.detection_time,
                    event_type: SecurityEventType::SuspiciousActivity,
                    severity: Severity::Critical,
                    client_info: client_info.clone(),
                    request_info: request_info.clone(),
                    error_details: None,
                    metrics: SecurityMetrics {
                        processing_time_ms: processing_time,
                        signature_verification_time_ms: 0,
                        database_lookup_time_ms: 0,
                        nonce_validation_time_ms: 0,
                        nonce_store_size: self.nonce_store.read().unwrap().size(),
                        nonce_store_utilization_percent: 0.0,
                        recent_failures: 1,
                        pattern_score: pattern.severity_score,
                        cache_hit_rate: 0.0,
                        cache_miss_count: 0,
                        requests_per_second: 0.0,
                        avg_latency_ms: processing_time as f64,
                        p95_latency_ms: 0.0,
                        p99_latency_ms: 0.0,
                        memory_usage_bytes: 0,
                        nonce_cleanup_operations: 0,
                        performance_alert_count: 0,
                    },
                };

                self.security_logger.log_security_event(pattern_event);
            }
        }
    }

    /// Log successful authentication
    fn log_authentication_success(
        &self,
        key_id: &str,
        client_info: &ClientInfo,
        request_info: &RequestInfo,
        start_time: Instant,
    ) {
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
            severity: Severity::Info,
            client_info: client_info_with_key,
            request_info: request_info.clone(),
            error_details: None,
            metrics: SecurityMetrics {
                processing_time_ms: processing_time,
                signature_verification_time_ms: 0,
                database_lookup_time_ms: 0,
                nonce_validation_time_ms: 0,
                nonce_store_size: self.nonce_store.read().unwrap().size(),
                nonce_store_utilization_percent: 0.0,
                recent_failures: 0,
                pattern_score: 0.0,
                cache_hit_rate: 0.0,
                cache_miss_count: 0,
                requests_per_second: 0.0,
                avg_latency_ms: processing_time as f64,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                memory_usage_bytes: 0,
                nonce_cleanup_operations: 0,
                performance_alert_count: 0,
            },
        };

        self.security_logger.log_security_event(event);
    }

    /// Get formatted error message for response with environment-aware detail levels
    pub fn get_error_message(&self, error: &AuthenticationError) -> String {
        let environment = if self.config.response_security.detailed_error_messages {
            "development"
        } else {
            "production"
        };

        error.user_friendly_message(environment)
    }

    /// Create standardized error response with consistent format
    pub fn create_error_response(&self, error: &AuthenticationError) -> ErrorResponse {
        let environment = if self.config.response_security.detailed_error_messages {
            "development"
        } else {
            "production"
        };

        ErrorResponse {
            error: true,
            error_code: error.error_code().to_string(),
            message: error.user_friendly_message(environment),
            correlation_id: if self.config.response_security.include_correlation_id {
                Some(error.correlation_id().to_string())
            } else {
                None
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            details: if environment == "development" {
                Some(ErrorDetails {
                    error_type: format!("{:?}", error)
                        .split('(')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string(),
                    troubleshooting: error.get_troubleshooting_guidance(),
                    suggested_actions: error.get_suggested_actions(),
                    documentation_link: error.get_documentation_link(),
                })
            } else {
                None
            },
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
    pub fn check_and_store_nonce_enhanced_for_test(
        &self,
        nonce: &str,
        created: u64,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
        self.check_and_store_nonce_enhanced(nonce, created, correlation_id)
    }

    #[cfg(test)]
    pub fn validate_timestamp_enhanced_for_test(
        &self,
        created: u64,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
        self.validate_timestamp_enhanced(created, correlation_id)
    }

    pub fn get_config(&self) -> &SignatureAuthConfig {
        &self.config
    }

    pub fn get_metrics_collector(&self) -> &EnhancedSecurityMetricsCollector {
        &self.metrics_collector
    }

    pub fn get_security_logger(&self) -> &SecurityLogger {
        &self.security_logger
    }

    /// Generate correlation ID for troubleshooting endpoints
    pub fn generate_correlation_id_public(&self) -> String {
        self.generate_correlation_id()
    }

    /// Validate timestamp for troubleshooting (public version)
    pub fn validate_timestamp_enhanced_public(
        &self,
        timestamp: u64,
        correlation_id: &str,
    ) -> Result<(), AuthenticationError> {
        self.validate_timestamp_enhanced(timestamp, correlation_id)
    }

    #[cfg(test)]
    pub fn get_config_test(&self) -> &SignatureAuthConfig {
        &self.config
    }

    #[cfg(test)]
    pub fn get_metrics_collector_test(&self) -> &EnhancedSecurityMetricsCollector {
        &self.metrics_collector
    }

    #[cfg(test)]
    pub fn get_security_logger_test(&self) -> &SecurityLogger {
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
                    warn!(
                        "Future timestamp detected: created={}, now={}, diff={}s",
                        created, now, future_diff
                    );
                }
                return Err(FoldDbError::Permission(format!(
                    "Timestamp too far in future: {} seconds",
                    future_diff
                )));
            }

            // Allow small future timestamps within clock skew tolerance
            if future_diff <= self.config.clock_skew_tolerance_secs {
                debug!(
                    "Accepting future timestamp within clock skew tolerance: {}s",
                    future_diff
                );
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
        let effective_window =
            self.config.allowed_time_window_secs + self.config.clock_skew_tolerance_secs;

        if time_diff > effective_window {
            if self.config.log_replay_attempts {
                warn!(
                    "Timestamp outside allowed window: created={}, now={}, diff={}s, window={}s",
                    created, now, time_diff, effective_window
                );
            }
            return Err(FoldDbError::Permission(format!(
                "Timestamp outside allowed window: {} seconds (max: {})",
                time_diff, effective_window
            )));
        }

        debug!(
            "Timestamp validation successful: diff={}s, window={}s",
            time_diff, effective_window
        );
        Ok(())
    }

    /// Validate RFC 3339 timestamp format
    pub fn validate_rfc3339_timestamp(&self, timestamp_str: &str) -> NodeResult<u64> {
        if !self.config.enforce_rfc3339_timestamps {
            // Fallback to simple unix timestamp parsing
            return timestamp_str
                .parse::<u64>()
                .map_err(|_| FoldDbError::Permission("Invalid timestamp format".to_string()));
        }

        // Simple RFC 3339 validation without external dependencies
        if !self.is_valid_rfc3339_format(timestamp_str) {
            return Err(FoldDbError::Permission(
                "Invalid RFC 3339 timestamp format".to_string(),
            ));
        }

        // For now, we'll require clients to send unix timestamps
        // Full RFC 3339 parsing would require chrono dependency
        Err(FoldDbError::Permission(
            "RFC 3339 parsing not implemented - please send unix timestamp".to_string(),
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
        chars.get(4) == Some(&'-')
            && chars.get(7) == Some(&'-')
            && chars.get(10) == Some(&'T')
            && chars.get(13) == Some(&':')
            && chars.get(16) == Some(&':')
            && chars[0..4].iter().all(|c| c.is_ascii_digit())
            && chars[5..7].iter().all(|c| c.is_ascii_digit())
            && chars[8..10].iter().all(|c| c.is_ascii_digit())
            && chars[11..13].iter().all(|c| c.is_ascii_digit())
            && chars[14..16].iter().all(|c| c.is_ascii_digit())
            && chars[17..19].iter().all(|c| c.is_ascii_digit())
    }

    /// Validate nonce format (UUID4 if required)
    pub fn validate_nonce_format(&self, nonce: &str) -> NodeResult<()> {
        if self.config.require_uuid4_nonces {
            self.validate_uuid4_format(nonce)?;
            return Ok(()); // If UUID4 validation passes, we're done
        }

        // Basic validation for non-UUID nonces
        if nonce.is_empty() {
            return Err(FoldDbError::Permission("Nonce cannot be empty".to_string()));
        }

        if nonce.len() > 128 {
            return Err(FoldDbError::Permission(
                "Nonce too long (max 128 characters)".to_string(),
            ));
        }

        // Ensure nonce contains only safe characters (alphanumeric, hyphens, underscores)
        if !nonce
            .chars()
            .all(|c| c.is_alphanumeric() || "-_".contains(c))
        {
            return Err(FoldDbError::Permission(
                "Nonce contains invalid characters (only alphanumeric, -, _ allowed)".to_string(),
            ));
        }

        Ok(())
    }

    /// Simple UUID4 format validation without external dependencies
    fn validate_uuid4_format(&self, nonce: &str) -> NodeResult<()> {
        // UUID4 format: 8-4-4-4-12 hexadecimal digits with hyphens
        // Example: 550e8400-e29b-41d4-a716-446655440000
        if nonce.len() != 36 {
            return Err(FoldDbError::Permission(
                "UUID must be 36 characters long".to_string(),
            ));
        }

        let chars: Vec<char> = nonce.chars().collect();

        // Check hyphen positions
        if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
            return Err(FoldDbError::Permission("Invalid UUID format".to_string()));
        }

        // Check that version is 4 (position 14)
        if chars[14] != '4' {
            return Err(FoldDbError::Permission(
                "Nonce must be UUID version 4".to_string(),
            ));
        }

        // Check that all other characters are hexadecimal
        for (i, &c) in chars.iter().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                continue; // Skip hyphens
            }
            if !c.is_ascii_hexdigit() {
                return Err(FoldDbError::Permission(
                    "UUID contains non-hexadecimal characters".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get current enhanced nonce store statistics
    pub fn get_nonce_store_stats(&self) -> NodeResult<NonceStorePerformanceStats> {
        let store = self
            .nonce_store
            .read()
            .map_err(|_| FoldDbError::Permission("Failed to acquire read lock".to_string()))?;

        Ok(store.get_performance_stats(self.config.max_nonce_store_size))
    }

    /// Warm up the public key cache by preloading frequently used keys
    pub async fn warm_cache(
        &self,
        _db_ops: std::sync::Arc<crate::db_operations::core::DbOperations>,
    ) -> NodeResult<CacheWarmupResult> {
        let warmup_start = Instant::now();
        let keys_loaded = 0;
        let errors = 0;

        info!("Starting public key cache warmup...");

        // Note: This is a simplified warmup - in production you might want to
        // load keys based on usage patterns or have a configurable warmup list

        let mut cache = self
            .key_cache
            .write()
            .map_err(|_| FoldDbError::Permission("Failed to acquire cache lock".to_string()))?;

        cache.mark_warmup_completed();
        drop(cache);

        let warmup_duration = warmup_start.elapsed();

        info!(
            "Cache warmup completed: {} keys loaded, {} errors, took {:?}",
            keys_loaded, errors, warmup_duration
        );

        Ok(CacheWarmupResult {
            keys_loaded,
            errors,
            duration_ms: warmup_duration.as_millis() as u64,
            cache_size_after: keys_loaded,
        })
    }

    /// Get comprehensive performance metrics for monitoring dashboard
    pub fn get_performance_metrics(&self) -> NodeResult<PerformanceMetrics> {
        let metrics = self
            .metrics_collector
            .get_enhanced_security_metrics(self.config.max_nonce_store_size);
        let nonce_stats = self.get_nonce_store_stats()?;

        let cache_stats = {
            let cache = self
                .key_cache
                .read()
                .map_err(|_| FoldDbError::Permission("Failed to acquire cache lock".to_string()))?;
            CacheStats {
                size: cache.size(),
                hit_rate: cache.hit_rate(),
                warmup_completed: cache.is_warmup_completed(),
            }
        };

        let performance_alerts = {
            let monitor = self.performance_monitor.read().map_err(|_| {
                FoldDbError::Permission("Failed to acquire monitor lock".to_string())
            })?;
            monitor.get_recent_alerts(10)
        };

        Ok(PerformanceMetrics {
            security_metrics: metrics,
            nonce_store_stats: nonce_stats,
            cache_stats,
            performance_alerts,
            system_health: self.assess_system_health()?,
        })
    }

    /// Assess overall system health based on performance metrics
    fn assess_system_health(&self) -> NodeResult<SystemHealthStatus> {
        let metrics = self
            .metrics_collector
            .get_enhanced_security_metrics(self.config.max_nonce_store_size);

        let mut health_score = 100.0;
        let mut issues = Vec::new();

        // Check latency (penalize if >10ms average)
        if metrics.avg_latency_ms > 10.0 {
            health_score -= 20.0;
            issues.push(format!(
                "High average latency: {:.1}ms",
                metrics.avg_latency_ms
            ));
        }

        // Check cache hit rate (penalize if <80%)
        if metrics.cache_hit_rate < 0.8 {
            health_score -= 15.0;
            issues.push(format!(
                "Low cache hit rate: {:.1}%",
                metrics.cache_hit_rate * 100.0
            ));
        }

        // Check nonce store utilization (penalize if >90%)
        if metrics.nonce_store_utilization_percent > 90.0 {
            health_score -= 10.0;
            issues.push(format!(
                "High nonce store utilization: {:.1}%",
                metrics.nonce_store_utilization_percent
            ));
        }

        // Check requests per second (warn if >1000 RPS)
        if metrics.requests_per_second > 1000.0 {
            health_score -= 5.0;
            issues.push(format!(
                "High request rate: {:.1} RPS",
                metrics.requests_per_second
            ));
        }

        let status = if health_score >= 90.0 {
            "healthy"
        } else if health_score >= 70.0 {
            "degraded"
        } else if health_score >= 50.0 {
            "unhealthy"
        } else {
            "critical"
        };

        Ok(SystemHealthStatus {
            status: status.to_string(),
            health_score,
            issues,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Clear cache for testing or maintenance
    pub fn clear_cache(&self) -> NodeResult<()> {
        let mut cache = self
            .key_cache
            .write()
            .map_err(|_| FoldDbError::Permission("Failed to acquire cache lock".to_string()))?;
        cache.clear();
        info!("Public key cache cleared");
        Ok(())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> NodeResult<CacheStats> {
        let cache = self
            .key_cache
            .read()
            .map_err(|_| FoldDbError::Permission("Failed to acquire cache lock".to_string()))?;

        Ok(CacheStats {
            size: cache.size(),
            hit_rate: cache.hit_rate(),
            warmup_completed: cache.is_warmup_completed(),
        })
    }

    /// Perform cache maintenance (cleanup expired entries)
    pub fn maintain_cache(&self) -> NodeResult<u64> {
        let mut cache = self
            .key_cache
            .write()
            .map_err(|_| FoldDbError::Permission("Failed to acquire cache lock".to_string()))?;

        let initial_size = cache.size();
        cache.cleanup_expired(Duration::from_secs(3600)); // 1 hour TTL
        let final_size = cache.size();
        let cleaned = initial_size.saturating_sub(final_size) as u64;

        if cleaned > 0 {
            debug!("Cache maintenance: removed {} expired entries", cleaned);
        }

        Ok(cleaned)
    }

    /// Trigger performance monitoring update
    pub fn update_performance_monitoring(&self) -> NodeResult<()> {
        let mut monitor = self
            .performance_monitor
            .write()
            .map_err(|_| FoldDbError::Permission("Failed to acquire monitor lock".to_string()))?;

        monitor.check_performance_thresholds(&self.config);
        Ok(())
    }

    // Legacy compatibility method
    pub fn get_nonce_store_stats_legacy(&self) -> NodeResult<NonceStoreStats> {
        let store = self
            .nonce_store
            .read()
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
        self.nonces
            .retain(|_, &mut created| now.saturating_sub(created) < ttl_secs);

        let removed = initial_size - self.nonces.len();
        if removed > 0 {
            debug!(
                "Cleaned up {} expired nonces, {} remaining",
                removed,
                self.nonces.len()
            );
        }
    }

    /// Enforce size limits by removing oldest nonces
    fn enforce_size_limit(&mut self, max_size: usize) {
        if self.nonces.len() <= max_size {
            return;
        }

        let to_remove = self.nonces.len() - max_size;

        // Collect all nonces with timestamps first
        let mut nonce_timestamps: Vec<(String, u64)> = self
            .nonces
            .iter()
            .map(|(nonce, &timestamp)| (nonce.clone(), timestamp))
            .collect();

        // Sort by timestamp (oldest first)
        nonce_timestamps.sort_by_key(|(_, timestamp)| *timestamp);

        // Remove the oldest nonces
        for (nonce, _) in nonce_timestamps.into_iter().take(to_remove) {
            self.nonces.remove(&nonce);
        }

        warn!(
            "Enforced nonce store size limit: removed {} oldest nonces, {} remaining",
            to_remove,
            self.nonces.len()
        );
    }

    /// Get the age of the oldest nonce in seconds
    fn get_oldest_nonce_age(&self) -> Option<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.nonces
            .values()
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
        let recent_count = self
            .nonces
            .values()
            .filter(|&&timestamp| timestamp >= window_start)
            .count();

        SlidingWindowStats {
            window_secs,
            recent_nonces: recent_count,
            total_nonces: self.nonces.len(),
            requests_per_second: recent_count as f64 / window_secs as f64,
        }
    }

    pub fn get_performance_stats(&self, max_capacity: usize) -> NonceStorePerformanceStats {
        let oldest_age = self.get_oldest_nonce_age();
        let total_nonces = self.nonces.len();
        let utilization_percent = if max_capacity > 0 {
            (total_nonces as f64 / max_capacity as f64) * 100.0
        } else {
            0.0
        };

        NonceStorePerformanceStats {
            total_nonces,
            max_capacity,
            utilization_percent,
            oldest_nonce_age_secs: oldest_age,
            cleanup_operations: 0, // This would be tracked separately
            memory_usage_bytes: (total_nonces * 64) as u64, // Rough estimate
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

/// Standardized error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: bool,
    pub error_code: String,
    pub message: String,
    pub correlation_id: Option<String>,
    pub timestamp: u64,
    pub details: Option<ErrorDetails>,
}

/// Detailed error information for development environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub troubleshooting: String,
    pub suggested_actions: Vec<String>,
    pub documentation_link: String,
}

/// Parsed signature components from RFC 9421 headers
#[derive(Debug)]
#[allow(dead_code)]
pub struct SignatureComponents {
    pub signature_input: String,
    pub signature: String,
    pub created: u64,
    pub keyid: String,
    pub algorithm: String,
    pub nonce: String,
    pub covered_components: Vec<String>,
}

impl SignatureComponents {
    /// Parse signature components from HTTP headers
    pub fn parse_from_headers(req: &ServiceRequest) -> NodeResult<Self> {
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
        let created = params
            .get("created")
            .ok_or_else(|| FoldDbError::Permission("Missing 'created' parameter".to_string()))?
            .trim_matches('"')
            .parse::<u64>()
            .map_err(|_| FoldDbError::Permission("Invalid 'created' timestamp".to_string()))?;

        let keyid = params
            .get("keyid")
            .ok_or_else(|| FoldDbError::Permission("Missing 'keyid' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        let algorithm = params
            .get("alg")
            .ok_or_else(|| FoldDbError::Permission("Missing 'alg' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        let nonce = params
            .get("nonce")
            .ok_or_else(|| FoldDbError::Permission("Missing 'nonce' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        // Validate algorithm
        if algorithm != "ed25519" {
            return Err(FoldDbError::Permission(format!(
                "Unsupported algorithm: {}",
                algorithm
            )));
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
            return Err(FoldDbError::Permission(
                "Invalid signature-input format".to_string(),
            ));
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
                return Err(FoldDbError::Permission(
                    "Invalid components format".to_string(),
                ));
            }

            let inner = &components_str[1..components_str.len() - 1];
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
                    let target_uri = format!(
                        "{}{}",
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
        lines.push(format!(
            "\"@signature-params\": {}",
            self.build_signature_params()
        ));

        Ok(lines.join("\n"))
    }

    /// Build the signature parameters line
    fn build_signature_params(&self) -> String {
        let components_str = self
            .covered_components
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(" ");

        format!("({})", components_str)
    }
}

/// Custom error type for authentication failures that implements ResponseError
#[derive(Debug)]
pub struct CustomAuthError {
    auth_error: AuthenticationError,
    error_message: String,
}

impl std::fmt::Display for CustomAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl actix_web::ResponseError for CustomAuthError {
    fn status_code(&self) -> StatusCode {
        self.auth_error.http_status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        match status {
            StatusCode::BAD_REQUEST => HttpResponse::BadRequest(),
            StatusCode::UNAUTHORIZED => HttpResponse::Unauthorized(),
            StatusCode::TOO_MANY_REQUESTS => HttpResponse::TooManyRequests(),
            StatusCode::INTERNAL_SERVER_ERROR => HttpResponse::InternalServerError(),
            _ => HttpResponse::Unauthorized(),
        }
        .json(serde_json::json!({
            "error": true,
            "error_code": self.auth_error.error_code(),
            "message": self.error_message,
            "correlation_id": self.auth_error.correlation_id()
        }))
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
                        client_id: client_id.clone(),
                    });

                    info!(
                        "Successfully verified signature for client {} on path {}",
                        client_id,
                        req.path()
                    );

                    // Apply timing protection if enabled
                    if state.config.attack_detection.enable_timing_protection {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        if elapsed < state.config.attack_detection.base_response_delay_ms {
                            let delay =
                                state.config.attack_detection.base_response_delay_ms - elapsed;
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                        }
                    }

                    service.call(req).await
                }
                Err(auth_error) => {
                    error!(
                        "Authentication failed for {}: {}",
                        req.path(),
                        auth_error.public_message()
                    );

                    // Apply consistent timing for error responses
                    if state.config.response_security.consistent_timing {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        if elapsed < state.config.attack_detection.base_response_delay_ms {
                            let delay =
                                state.config.attack_detection.base_response_delay_ms - elapsed;
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                        }
                    }

                    // Create error message with appropriate detail level
                    let error_message = state.get_error_message(&auth_error);

                    // Create custom error that implements ResponseError for proper HTTP response
                    let custom_error = CustomAuthError {
                        auth_error,
                        error_message,
                    };

                    // Use actix-web's error handling - CustomAuthError implements ResponseError
                    Err(actix_web::Error::from(custom_error))
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
            return Err(FoldDbError::Permission(format!(
                "Required component '{}' not covered by signature",
                required_component
            )));
        }
    }

    // Verify the signature against the stored public key
    match state
        .verify_signature_against_database(&components, req, app_state)
        .await
    {
        Ok(()) => {
            info!(
                "Signature verification successful for client: {}",
                components.keyid
            );
            Ok(components.keyid)
        }
        Err(auth_error) => {
            warn!(
                "Signature verification failed for client {}: {}",
                components.keyid, auth_error
            );
            Err(FoldDbError::Permission(format!(
                "Signature verification failed: {}",
                auth_error
            )))
        }
    }
}

/// Check if signature verification should be skipped for this path
fn should_skip_verification(path: &str) -> bool {
    // Only allow these specific paths to skip verification (minimal set for system operation)
    const SKIP_PATHS: &[&str] = &[
        "/api/system/status",        // Health checks
        "/api/crypto/keys/register", // Initial key registration
        "/",                         // Static file serving
        "/index.html",               // Static file serving
    ];

    SKIP_PATHS
        .iter()
        .any(|&skip_path| path == skip_path || (skip_path == "/" && path.starts_with("/static")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
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
        let _app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

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
