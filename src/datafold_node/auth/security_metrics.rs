//! Security metrics and performance monitoring for DataFold authentication
//!
//! This module provides comprehensive security metrics collection and performance
//! monitoring capabilities. It tracks authentication attempts, latencies, cache
//! performance, and system health indicators for security analytics and reporting.

use std::time::{Duration, Instant};
use super::auth_config::SignatureAuthConfig;
use super::auth_types::{
    EnhancedSecurityMetricsCollector, LatencyHistogram, PerformanceAlert, 
    PerformanceAlertType, PerformanceMonitor, SecurityMetrics,
};
use crate::security_types::Severity;

impl EnhancedSecurityMetricsCollector {
    /// Record an authentication attempt with timing information
    /// 
    /// # Arguments
    /// * `success` - Whether the authentication was successful
    /// * `processing_time_ms` - Total processing time in milliseconds
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

    /// Record signature verification timing
    pub fn record_signature_verification_time(&self, time_ms: u64) {
        if let Ok(mut times) = self.signature_verification_times.write() {
            times.record(time_ms);
        }
    }

    /// Record database lookup timing
    pub fn record_database_lookup_time(&self, time_ms: u64) {
        if let Ok(mut times) = self.database_lookup_times.write() {
            times.record(time_ms);
        }
    }

    /// Record nonce validation timing
    pub fn record_nonce_validation_time(&self, time_ms: u64) {
        if let Ok(mut times) = self.nonce_validation_times.write() {
            times.record(time_ms);
        }
    }

    /// Update nonce store utilization metrics
    pub fn update_nonce_store_utilization(&self, size: usize) {
        self.nonce_store_utilization.store(size, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Update memory usage tracking
    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage_bytes.store(bytes, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increment cleanup operation counter
    pub fn record_cleanup_operation(&self) {
        self.cleanup_operations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increment performance alert counter
    pub fn record_performance_alert(&self) {
        self.performance_alerts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get comprehensive security metrics
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

    /// Calculate cache hit rate
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

    /// Calculate current requests per second
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

    /// Get basic statistics
    pub fn get_basic_stats(&self) -> (u64, u64, u64) {
        let attempts = self.total_attempts.load(std::sync::atomic::Ordering::Relaxed);
        let successes = self.total_successes.load(std::sync::atomic::Ordering::Relaxed);
        let failures = self.total_failures.load(std::sync::atomic::Ordering::Relaxed);
        (attempts, successes, failures)
    }

    /// Reset all metrics (useful for testing)
    pub fn reset_metrics(&self) {
        self.total_attempts.store(0, std::sync::atomic::Ordering::Relaxed);
        self.total_successes.store(0, std::sync::atomic::Ordering::Relaxed);
        self.total_failures.store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_hits.store(0, std::sync::atomic::Ordering::Relaxed);
        self.cache_misses.store(0, std::sync::atomic::Ordering::Relaxed);
        self.memory_usage_bytes.store(0, std::sync::atomic::Ordering::Relaxed);
        self.performance_alerts.store(0, std::sync::atomic::Ordering::Relaxed);
        self.cleanup_operations.store(0, std::sync::atomic::Ordering::Relaxed);

        // Reset histograms
        if let Ok(mut times) = self.processing_times.write() {
            *times = LatencyHistogram::new(times.max_measurements);
        }
        if let Ok(mut times) = self.signature_verification_times.write() {
            *times = LatencyHistogram::new(times.max_measurements);
        }
        if let Ok(mut times) = self.database_lookup_times.write() {
            *times = LatencyHistogram::new(times.max_measurements);
        }
        if let Ok(mut times) = self.nonce_validation_times.write() {
            *times = LatencyHistogram::new(times.max_measurements);
        }
        if let Ok(mut timestamps) = self.request_timestamps.write() {
            timestamps.clear();
        }
    }
}

impl LatencyHistogram {
    /// Record a latency measurement
    pub fn record(&mut self, latency_ms: u64) {
        // Add to measurements
        self.measurements.push_back(latency_ms);
        
        // Keep only the most recent measurements
        while self.measurements.len() > self.max_measurements {
            self.measurements.pop_front();
        }

        // Update histogram buckets
        for &bucket in self.buckets.keys() {
            if latency_ms <= bucket {
                *self.buckets.get_mut(&bucket).unwrap() += 1;
                break;
            }
        }
    }

    /// Calculate average latency
    pub fn average(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: u64 = self.measurements.iter().sum();
        sum as f64 / self.measurements.len() as f64
    }

    /// Calculate percentile latency
    pub fn percentile(&self, p: f64) -> Option<u64> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = self.measurements.iter().cloned().collect();
        sorted.sort_unstable();

        let index = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted.get(index).copied()
    }

    /// Get min, max latencies
    pub fn min_max(&self) -> Option<(u64, u64)> {
        if self.measurements.is_empty() {
            return None;
        }

        let min = *self.measurements.iter().min().unwrap();
        let max = *self.measurements.iter().max().unwrap();
        Some((min, max))
    }

    /// Get bucket counts for histogram visualization
    pub fn get_bucket_counts(&self) -> Vec<(u64, u64)> {
        let mut buckets: Vec<(u64, u64)> = self.buckets.iter()
            .map(|(&bucket, &count)| (bucket, count))
            .collect();
        buckets.sort_by_key(|&(bucket, _)| bucket);
        buckets
    }

    /// Get the count of measurements
    pub fn count(&self) -> usize {
        self.measurements.len()
    }
}

impl PerformanceMonitor {
    /// Record a request with its latency
    pub fn record_request(&mut self, latency_ms: u64) {
        self.recent_latencies.push_back(latency_ms);

        // Keep only recent measurements (last 60 seconds worth)
        while self.recent_latencies.len() > 1000 {
            self.recent_latencies.pop_front();
        }

        self.last_update = Instant::now();
    }

    /// Check performance thresholds and generate alerts
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

    /// Get average latency from recent measurements
    pub fn get_average_latency(&self) -> Option<f64> {
        if self.recent_latencies.is_empty() {
            return None;
        }

        let sum: u64 = self.recent_latencies.iter().sum();
        Some(sum as f64 / self.recent_latencies.len() as f64)
    }

    /// Get percentile latency
    pub fn get_percentile_latency(&self, p: f64) -> Option<f64> {
        if self.recent_latencies.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = self.recent_latencies.iter().cloned().collect();
        sorted.sort_unstable();

        let index = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted.get(index).map(|&v| v as f64)
    }

    /// Get recent performance alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        self.alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Get all alerts count
    pub fn get_total_alert_count(&self) -> usize {
        self.alerts.len()
    }

    /// Clear old alerts (keep only recent ones)
    pub fn cleanup_old_alerts(&mut self, max_alerts: usize) {
        if self.alerts.len() > max_alerts {
            let to_remove = self.alerts.len() - max_alerts;
            self.alerts.drain(0..to_remove);
        }
    }

    /// Create a performance alert
    fn create_alert(
        &mut self,
        alert_type: PerformanceAlertType,
        message: String,
        metric_value: f64,
        threshold: f64,
        severity: Severity,
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

    /// Get performance summary
    pub fn get_performance_summary(&self) -> Option<PerformanceSummary> {
        if self.recent_latencies.is_empty() {
            return None;
        }

        Some(PerformanceSummary {
            sample_count: self.recent_latencies.len(),
            avg_latency_ms: self.get_average_latency().unwrap_or(0.0),
            p50_latency_ms: self.get_percentile_latency(50.0).unwrap_or(0.0),
            p95_latency_ms: self.get_percentile_latency(95.0).unwrap_or(0.0),
            p99_latency_ms: self.get_percentile_latency(99.0).unwrap_or(0.0),
            alert_count: self.alerts.len(),
            last_update: self.last_update,
        })
    }
}

/// Performance summary for reporting
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub sample_count: usize,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub alert_count: usize,
    pub last_update: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_security_metrics_collector() {
        let collector = EnhancedSecurityMetricsCollector::new();

        // Test recording attempts
        collector.record_attempt(true, 50);
        collector.record_attempt(false, 100);
        collector.record_attempt(true, 75);

        let (attempts, successes, failures) = collector.get_basic_stats();
        assert_eq!(attempts, 3);
        assert_eq!(successes, 2);
        assert_eq!(failures, 1);

        // Test cache metrics
        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_miss();

        let hit_rate = collector.get_cache_hit_rate();
        assert!((hit_rate - 0.6666666666666666).abs() < 0.001);

        // Test metrics retrieval
        let metrics = collector.get_enhanced_security_metrics(1000);
        assert!(metrics.processing_time_ms > 0);
        assert!(metrics.cache_hit_rate > 0.0);
    }

    #[test]
    fn test_latency_histogram() {
        let mut histogram = LatencyHistogram::new(100);

        // Record some measurements
        histogram.record(10);
        histogram.record(50);
        histogram.record(100);
        histogram.record(200);

        // Test average
        let avg = histogram.average();
        assert_eq!(avg, 90.0); // (10 + 50 + 100 + 200) / 4

        // Test percentiles
        assert_eq!(histogram.percentile(50.0), Some(50));
        assert_eq!(histogram.percentile(100.0), Some(200));

        // Test min/max
        let (min, max) = histogram.min_max().unwrap();
        assert_eq!(min, 10);
        assert_eq!(max, 200);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        // Record some requests
        monitor.record_request(50);
        monitor.record_request(100);
        monitor.record_request(75);

        // Test average latency
        let avg = monitor.get_average_latency().unwrap();
        assert_eq!(avg, 75.0); // (50 + 100 + 75) / 3

        // Test percentiles
        assert_eq!(monitor.get_percentile_latency(50.0), Some(75.0));

        // Test performance summary
        let summary = monitor.get_performance_summary().unwrap();
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.avg_latency_ms, 75.0);
    }

    #[test]
    fn test_metrics_reset() {
        let collector = EnhancedSecurityMetricsCollector::new();

        // Add some data
        collector.record_attempt(true, 50);
        collector.record_cache_hit();

        let (attempts, _, _) = collector.get_basic_stats();
        assert_eq!(attempts, 1);

        // Reset and verify
        collector.reset_metrics();
        let (attempts, successes, failures) = collector.get_basic_stats();
        assert_eq!(attempts, 0);
        assert_eq!(successes, 0);
        assert_eq!(failures, 0);
        assert_eq!(collector.get_cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_performance_alerts() {
        let mut monitor = PerformanceMonitor::new();
        let config = SignatureAuthConfig::default();

        // Record high latency to trigger alert
        for _ in 0..10 {
            monitor.record_request(150); // Above 100ms threshold
        }

        monitor.check_performance_thresholds(&config);

        let alerts = monitor.get_recent_alerts(5);
        assert!(!alerts.is_empty());
        assert!(matches!(alerts[0].alert_type, PerformanceAlertType::HighLatency));
    }
}