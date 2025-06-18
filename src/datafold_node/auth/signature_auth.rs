//! Ed25519 signature verification middleware for DataFold HTTP server (Refactored)
//!
//! This module serves as the main coordinator for the signature authentication system,
//! delegating to specialized modules while maintaining backward compatibility with
//! the existing public API.

// Re-export all public types and functions for backward compatibility
pub use super::{
    auth_config::{
        AttackDetectionConfig, RateLimitingConfig, ResponseSecurityConfig, SecurityLoggingConfig,
        SignatureAuthConfig,
    },
    auth_errors::{AuthenticationError, CustomAuthError, ErrorDetails, ErrorResponse},
    auth_middleware::{
        should_skip_verification, SecurityLogger, SignatureVerificationMiddleware,
        SignatureVerificationService, SignatureVerificationState,
    },
    auth_types::{
        AttackDetector, AttackPatternType, AuthenticatedClient, CacheStats, CacheWarmupResult,
        CachedPublicKey, ClientInfo, EnhancedSecurityMetricsCollector, LatencyHistogram,
        NonceStore, NonceStorePerformanceStats, NonceStoreStats, PerformanceAlert,
        PerformanceAlertType, PerformanceBreakdown, PerformanceMetrics, PerformanceMonitor,
        PublicKeyCache, RateLimiter, RequestInfo, SecurityEvent, SecurityEventType,
        SecurityMetrics, SecurityProfile, SignatureComponents, SlidingWindowStats,
        SuspiciousPattern, SystemHealthStatus,
    },
    key_management::{DetailedCacheStats, KeyManager},
    signature_verification::{verify_request_signature, SignatureVerifier},
};

use log::{debug, warn};
use std::time::{Duration, Instant};


// Implement missing methods for auth_types
impl NonceStore {
    pub fn contains_nonce(&self, nonce: &str) -> bool {
        self.nonces.contains_key(nonce)
    }

    pub fn add_nonce(&mut self, nonce: String, created: u64) {
        self.nonces.insert(nonce, created);
    }

    pub fn size(&self) -> usize {
        self.nonces.len()
    }

    pub fn cleanup_expired(&mut self, ttl_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
    pub fn enforce_size_limit(&mut self, max_size: usize) {
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
    pub fn get_oldest_nonce_age(&self) -> Option<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.nonces
            .values()
            .min()
            .map(|&oldest| now.saturating_sub(oldest))
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

// Implement missing methods for RateLimiter
impl RateLimiter {
    pub fn check_rate_limit(
        &mut self,
        client_id: &str,
        config: &RateLimitingConfig,
        is_failure: bool,
    ) -> bool {
        if !config.enabled {
            return true; // Rate limiting disabled
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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

// Implement missing methods for AttackDetector
impl AttackDetector {
    pub fn detect_attack_patterns(
        &mut self,
        client_id: &str,
        event: &SecurityEvent,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
        if !config.enabled {
            return None;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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

// Implementation for EnhancedSecurityMetricsCollector methods
impl EnhancedSecurityMetricsCollector {
    pub fn record_attempt(&self, success: bool, processing_time_ms: u64) {
        self.total_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if success {
            self.total_successes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.total_failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        self.nonce_store_utilization.store(size, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_enhanced_security_metrics(&self, nonce_store_max_size: usize) -> SecurityMetrics {
        let processing_times = self.processing_times.read().unwrap();
        let sig_times = self.signature_verification_times.read().unwrap();
        let db_times = self.database_lookup_times.read().unwrap();
        let nonce_times = self.nonce_validation_times.read().unwrap();

        let nonce_utilization = self.nonce_store_utilization.load(std::sync::atomic::Ordering::Relaxed);
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
            recent_failures: self.total_failures.load(std::sync::atomic::Ordering::Relaxed) as usize,
            pattern_score: 0.0,
            cache_hit_rate: self.get_cache_hit_rate(),
            cache_miss_count: self.cache_misses.load(std::sync::atomic::Ordering::Relaxed),
            requests_per_second: self.get_requests_per_second(),
            avg_latency_ms: processing_times.average(),
            p95_latency_ms: processing_times.percentile(95.0).unwrap_or(0) as f64,
            p99_latency_ms: processing_times.percentile(99.0).unwrap_or(0) as f64,
            memory_usage_bytes: self.memory_usage_bytes.load(std::sync::atomic::Ordering::Relaxed),
            nonce_cleanup_operations: self.cleanup_operations.load(std::sync::atomic::Ordering::Relaxed),
            performance_alert_count: self.performance_alerts.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.cache_misses.load(std::sync::atomic::Ordering::Relaxed);
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

// Implementation for PerformanceMonitor methods
impl PerformanceMonitor {
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
                    crate::security_types::Severity::Warning,
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
        severity: crate::security_types::Severity,
    ) {
        let alert = crate::datafold_node::auth::auth_types::create_performance_alert(
            alert_type,
            message,
            metric_value,
            threshold,
            severity,
        );

        self.alerts.push(alert);

        // Limit stored alerts
        if self.alerts.len() > 1000 {
            self.alerts.drain(0..100);
        }
    }
}

// Legacy test module for backward compatibility
#[cfg(test)]
mod tests {
    use super::*;
    
    // Re-export common test utilities if needed
    // Note: The actual tests are now in the auth_tests module
}
