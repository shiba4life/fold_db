//! Performance monitoring structures and implementations for signature authentication
//!
//! This module provides comprehensive performance monitoring, caching, and optimization
//! features for the signature authentication system.

use crate::datafold_node::signature_auth::{SecurityMetrics, PerformanceBreakdown, NonceStorePerformanceStats};
use crate::security_types::Severity;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
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

/// Performance alert with serializable timestamp
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
        if let Some((lru_key, _)) = self.keys.iter()
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
    current_rps: f64,
    /// Recent latency measurements
    recent_latencies: VecDeque<u64>,
    /// Performance alerts
    alerts: Vec<PerformanceAlert>,
    /// Monitoring start time
    start_time: Instant,
    /// Last monitoring update
    last_update: Instant,
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
    
    pub fn check_performance_thresholds(&mut self, _config: &crate::datafold_node::signature_auth::SignatureAuthConfig) {
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

    pub fn record_signature_verification_time(&self, duration_ms: u64) {
        if let Ok(mut times) = self.signature_verification_times.write() {
            times.record(duration_ms);
        }
    }

    pub fn record_database_lookup_time(&self, duration_ms: u64) {
        if let Ok(mut times) = self.database_lookup_times.write() {
            times.record(duration_ms);
        }
    }

    pub fn record_nonce_validation_time(&self, duration_ms: u64) {
        if let Ok(mut times) = self.nonce_validation_times.write() {
            times.record(duration_ms);
        }
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cleanup_operation(&self) {
        self.cleanup_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_performance_alert(&self) {
        self.performance_alerts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_nonce_store_utilization(&self, size: usize) {
        self.nonce_store_utilization.store(size, Ordering::Relaxed);
    }

    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn get_requests_per_second(&self) -> f64 {
        if let Ok(timestamps) = self.request_timestamps.read() {
            let count = timestamps.len();
            if count == 0 {
                return 0.0;
            }
            
            // Calculate RPS over the last 60 seconds
            let duration = if let (Some(&first), Some(&last)) = (timestamps.front(), timestamps.back()) {
                last.duration_since(first).as_secs_f64()
            } else {
                1.0
            };
            
            count as f64 / duration.max(1.0)
        } else {
            0.0
        }
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

    /// Get detailed performance breakdown
    pub fn get_performance_breakdown(&self) -> PerformanceBreakdown {
        let processing_times = self.processing_times.read().unwrap();
        let sig_times = self.signature_verification_times.read().unwrap();
        let db_times = self.database_lookup_times.read().unwrap();
        let nonce_times = self.nonce_validation_times.read().unwrap();

        PerformanceBreakdown {
            total_requests: self.total_attempts.load(Ordering::Relaxed),
            success_rate: {
                let total = self.total_attempts.load(Ordering::Relaxed);
                if total > 0 {
                    (self.total_successes.load(Ordering::Relaxed) as f64 / total as f64) * 100.0
                } else {
                    0.0
                }
            },
            avg_processing_time_ms: processing_times.average(),
            avg_signature_verification_ms: sig_times.average(),
            avg_database_lookup_ms: db_times.average(),
            avg_nonce_validation_ms: nonce_times.average(),
            p50_latency_ms: processing_times.percentile(50.0).unwrap_or(0) as f64,
            p95_latency_ms: processing_times.percentile(95.0).unwrap_or(0) as f64,
            p99_latency_ms: processing_times.percentile(99.0).unwrap_or(0) as f64,
            cache_hit_rate: self.get_cache_hit_rate(),
            requests_per_second: self.get_requests_per_second(),
        }
    }
}