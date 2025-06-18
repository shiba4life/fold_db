//! Authentication types, structs, and enums for DataFold node
//!
//! This module consolidates all authentication-related data structures including
//! authentication tokens, credentials, result types, security events, and metrics.

use crate::security_types::Severity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicU64, AtomicUsize},
    Arc, RwLock,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::auth_errors::AuthenticationError;

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

/// Information about an authenticated client
#[derive(Debug, Clone)]
pub struct AuthenticatedClient {
    pub client_id: String,
}

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

/// Rate limiter for preventing abuse
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Track requests per client (IP or key_id)
    pub client_requests: HashMap<String, Vec<u64>>,
    /// Track failures per client
    pub client_failures: HashMap<String, Vec<u64>>,
}

/// Attack pattern detector
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AttackDetector {
    /// Brute force attempts per client
    pub brute_force_attempts: HashMap<String, Vec<u64>>,
    /// Replay attempts per nonce
    pub replay_attempts: HashMap<String, Vec<u64>>,
    /// Suspicious patterns detected
    pub _suspicious_patterns: HashMap<String, SuspiciousPattern>,
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
    pub total_attempts: AtomicU64,
    /// Total authentication successes
    pub total_successes: AtomicU64,
    /// Total authentication failures
    pub total_failures: AtomicU64,
    /// Processing time measurements (with histogram buckets)
    pub processing_times: Arc<RwLock<LatencyHistogram>>,
    /// Signature verification times
    pub signature_verification_times: Arc<RwLock<LatencyHistogram>>,
    /// Database lookup times
    pub database_lookup_times: Arc<RwLock<LatencyHistogram>>,
    /// Nonce validation times
    pub nonce_validation_times: Arc<RwLock<LatencyHistogram>>,
    /// Current nonce store utilization
    pub nonce_store_utilization: AtomicUsize,
    /// Cache performance metrics
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    /// Request rate tracking
    pub request_timestamps: Arc<RwLock<VecDeque<Instant>>>,
    /// Memory usage tracking
    pub memory_usage_bytes: AtomicU64,
    /// Performance alert counters
    pub performance_alerts: AtomicU64,
    /// Cleanup operation counters
    pub cleanup_operations: AtomicU64,
}

/// Latency histogram for performance monitoring
#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    pub measurements: VecDeque<u64>,
    pub buckets: HashMap<u64, u64>, // bucket_ms -> count
    pub max_measurements: usize,
}

/// Performance monitor for real-time tracking
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Current requests per second
    #[allow(dead_code)]
    pub current_rps: f64,
    /// Recent latency measurements
    pub recent_latencies: VecDeque<u64>,
    /// Performance alerts
    pub alerts: Vec<PerformanceAlert>,
    /// Monitoring start time
    #[allow(dead_code)]
    pub start_time: Instant,
    /// Last monitoring update
    pub last_update: Instant,
}

/// Public key cache for performance optimization
#[derive(Debug)]
pub struct PublicKeyCache {
    /// Cached public keys by key_id
    pub keys: HashMap<String, CachedPublicKey>,
    /// Cache hit statistics
    pub hit_count: u64,
    /// Cache miss statistics
    pub miss_count: u64,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache warmup status
    pub warmup_completed: bool,
    /// Last cleanup timestamp
    pub last_cleanup: Instant,
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

/// Statistics about the nonce store
#[derive(Debug, Clone, Default)]
pub struct NonceStoreStats {
    pub total_nonces: usize,
    pub max_capacity: usize,
    pub oldest_nonce_age: Option<u64>,
}

/// In-memory nonce store for replay prevention with advanced features
#[derive(Debug)]
pub struct NonceStore {
    /// Map of nonce to creation timestamp
    pub nonces: HashMap<String, u64>,
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
    pub signature_input: String,
    pub signature: String,
    pub created: u64,
    pub keyid: String,
    pub algorithm: String,
    pub nonce: String,
    pub covered_components: Vec<String>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AttackDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EnhancedSecurityMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NonceStore {
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
}

impl AttackDetector {
    pub fn new() -> Self {
        Self {
            brute_force_attempts: HashMap::new(),
            replay_attempts: HashMap::new(),
            _suspicious_patterns: HashMap::new(),
        }
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
}

impl NonceStore {
    pub fn new() -> Self {
        Self {
            nonces: HashMap::new(),
        }
    }
}

/// Create a security event for logging
pub fn create_security_event(
    event_type: SecurityEventType,
    severity: Severity,
    client_info: ClientInfo,
    request_info: RequestInfo,
    error_details: Option<AuthenticationError>,
    metrics: SecurityMetrics,
) -> SecurityEvent {
    SecurityEvent {
        event_id: Uuid::new_v4().to_string(),
        correlation_id: error_details
            .as_ref()
            .map(|e| e.correlation_id().to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        event_type,
        severity,
        client_info,
        request_info,
        error_details,
        metrics,
    }
}

/// Create a performance alert
pub fn create_performance_alert(
    alert_type: PerformanceAlertType,
    message: String,
    metric_value: f64,
    threshold: f64,
    severity: Severity,
) -> PerformanceAlert {
    PerformanceAlert {
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
    }
}