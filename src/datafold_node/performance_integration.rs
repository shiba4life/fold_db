//! Performance integration utilities for gradual performance monitoring adoption
//!
//! This module provides utilities to integrate performance monitoring features
//! with the existing signature authentication system without breaking changes.

use crate::datafold_node::signature_auth::{SignatureVerificationState, SecurityMetrics};
use crate::datafold_node::performance_routes::*;
use crate::error::FoldDbError;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Performance monitoring wrapper that can be added to existing systems
pub struct PerformanceMonitoringWrapper {
    /// Start time for uptime tracking
    start_time: Instant,
    /// System start timestamp
    system_start: u64,
}

impl PerformanceMonitoringWrapper {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            system_start: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create performance dashboard data from existing signature auth state
    pub fn create_dashboard_from_state(
        &self,
        state: &SignatureVerificationState,
    ) -> Result<PerformanceDashboard, FoldDbError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let uptime_seconds = self.start_time.elapsed().as_secs();

        // Get existing metrics from the signature auth system
        let security_metrics = state.get_metrics_collector().get_security_metrics();
        
        // Build signature auth performance metrics from existing data
        let signature_auth_metrics = self.build_signature_auth_metrics_from_existing(&security_metrics)?;
        
        // Assess system health
        let system_health = self.assess_health_from_metrics(&signature_auth_metrics);
        
        // Generate alerts based on current metrics
        let alerts = self.generate_alerts_from_metrics(&signature_auth_metrics);
        
        // Generate basic recommendations
        let recommendations = self.generate_recommendations_from_metrics(&signature_auth_metrics);

        Ok(PerformanceDashboard {
            timestamp,
            uptime_seconds,
            signature_auth_metrics,
            system_health,
            alerts,
            recommendations,
        })
    }

    /// Build enhanced metrics from existing SecurityMetrics
    fn build_signature_auth_metrics_from_existing(
        &self,
        security_metrics: &SecurityMetrics,
    ) -> Result<SignatureAuthPerformanceMetrics, FoldDbError> {
        // Extract available data and estimate missing data
        let processing_time = security_metrics.processing_time_ms as f64;
        let nonce_store_size = security_metrics.nonce_store_size;
        let recent_failures = security_metrics.recent_failures;
        
        // Estimate metrics based on available data
        let estimated_total_requests = (nonce_store_size + recent_failures).max(1) as u64;
        let estimated_success_rate = if estimated_total_requests > 0 {
            ((estimated_total_requests - recent_failures as u64) as f64 / estimated_total_requests as f64) * 100.0
        } else {
            100.0
        };

        // Performance target compliance
        let target_latency = 10.0; // 10ms target from T11.6
        let target_met = processing_time <= target_latency;
        let compliance_percent = if processing_time > 0.0 {
            (target_latency / processing_time * 100.0).min(100.0)
        } else {
            100.0
        };

        let performance_target_compliance = PerformanceTargetCompliance {
            latency_target_ms: target_latency,
            current_avg_latency_ms: processing_time,
            target_met,
            target_compliance_percent: compliance_percent,
            recommendations: if !target_met {
                vec![
                    "Enable performance monitoring for detailed analysis".to_string(),
                    "Consider implementing caching strategies".to_string(),
                    "Review and optimize database operations".to_string(),
                ]
            } else {
                vec!["Performance is meeting targets".to_string()]
            },
        };

        // Estimated breakdown (based on typical performance patterns)
        let breakdown = PerformanceBreakdownMetrics {
            signature_verification_ms: processing_time * 0.4,
            database_lookup_ms: processing_time * 0.3,
            nonce_validation_ms: processing_time * 0.2,
            cache_operations_ms: processing_time * 0.05,
            overhead_ms: processing_time * 0.05,
        };

        // Calculate utilization percentage
        let max_capacity = 10000; // Default from config
        let utilization_percent = (nonce_store_size as f64 / max_capacity as f64) * 100.0;

        Ok(SignatureAuthPerformanceMetrics {
            total_requests: estimated_total_requests,
            successful_authentications: (estimated_total_requests as f64 * estimated_success_rate / 100.0) as u64,
            failed_authentications: recent_failures as u64,
            success_rate_percent: estimated_success_rate,
            avg_processing_time_ms: processing_time,
            p50_latency_ms: processing_time * 0.8, // Estimates
            p95_latency_ms: processing_time * 1.5,
            p99_latency_ms: processing_time * 2.0,
            requests_per_second: self.estimate_requests_per_second(estimated_total_requests),
            nonce_store_utilization_percent: utilization_percent,
            nonce_store_size,
            nonce_store_max_capacity: max_capacity,
            performance_target_compliance,
            breakdown,
        })
    }

    /// Estimate requests per second based on uptime and total requests
    fn estimate_requests_per_second(&self, total_requests: u64) -> f64 {
        let uptime_secs = self.start_time.elapsed().as_secs().max(1);
        total_requests as f64 / uptime_secs as f64
    }

    /// Assess system health from performance metrics
    fn assess_health_from_metrics(&self, metrics: &SignatureAuthPerformanceMetrics) -> PerformanceHealthStatus {
        let mut health_score = 100.0;
        let mut critical_issues = Vec::new();
        let mut warnings = Vec::new();
        let mut status_reasons = Vec::new();

        // Check latency performance
        if metrics.avg_processing_time_ms > 50.0 {
            health_score -= 30.0;
            critical_issues.push("Average latency exceeds 50ms".to_string());
        } else if metrics.avg_processing_time_ms > 20.0 {
            health_score -= 15.0;
            warnings.push("Latency elevated above 20ms".to_string());
        } else if metrics.avg_processing_time_ms > 10.0 {
            health_score -= 5.0;
            warnings.push("Latency above 10ms target".to_string());
        }

        // Check success rate
        if metrics.success_rate_percent < 90.0 {
            health_score -= 25.0;
            critical_issues.push(format!("Low success rate: {:.1}%", metrics.success_rate_percent));
        } else if metrics.success_rate_percent < 95.0 {
            health_score -= 10.0;
            warnings.push(format!("Success rate below optimal: {:.1}%", metrics.success_rate_percent));
        }

        // Check nonce store utilization
        if metrics.nonce_store_utilization_percent > 90.0 {
            health_score -= 20.0;
            critical_issues.push(format!("Nonce store near capacity: {:.1}%", metrics.nonce_store_utilization_percent));
        } else if metrics.nonce_store_utilization_percent > 75.0 {
            health_score -= 5.0;
            warnings.push(format!("Nonce store utilization high: {:.1}%", metrics.nonce_store_utilization_percent));
        }

        // Determine overall status
        let overall_status = if health_score >= 95.0 {
            "excellent"
        } else if health_score >= 85.0 {
            "healthy"
        } else if health_score >= 70.0 {
            "degraded"
        } else if health_score >= 50.0 {
            "unhealthy"
        } else {
            "critical"
        };

        // Add positive status reasons if no issues
        if critical_issues.is_empty() && warnings.is_empty() {
            status_reasons.push("All performance metrics within acceptable ranges".to_string());
            if metrics.performance_target_compliance.target_met {
                status_reasons.push("Meeting performance targets".to_string());
            }
        }

        PerformanceHealthStatus {
            overall_status: overall_status.to_string(),
            health_score,
            status_reasons,
            critical_issues,
            warnings,
            last_health_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Generate performance alerts from current metrics
    fn generate_alerts_from_metrics(&self, metrics: &SignatureAuthPerformanceMetrics) -> Vec<PerformanceAlertInfo> {
        let mut alerts = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // High latency alert
        if metrics.avg_processing_time_ms > 50.0 {
            alerts.push(PerformanceAlertInfo {
                alert_id: format!("high-latency-{}", timestamp),
                timestamp,
                severity: "critical".to_string(),
                alert_type: "high_latency".to_string(),
                message: format!("Average processing time {:.1}ms exceeds critical threshold", metrics.avg_processing_time_ms),
                metric_name: "avg_processing_time_ms".to_string(),
                current_value: metrics.avg_processing_time_ms,
                threshold_value: 50.0,
                suggested_action: "Investigate performance bottlenecks and consider optimization".to_string(),
            });
        } else if metrics.avg_processing_time_ms > 10.0 {
            alerts.push(PerformanceAlertInfo {
                alert_id: format!("elevated-latency-{}", timestamp),
                timestamp,
                severity: "warning".to_string(),
                alert_type: "elevated_latency".to_string(),
                message: format!("Average processing time {:.1}ms above target", metrics.avg_processing_time_ms),
                metric_name: "avg_processing_time_ms".to_string(),
                current_value: metrics.avg_processing_time_ms,
                threshold_value: 10.0,
                suggested_action: "Monitor trends and consider performance optimization".to_string(),
            });
        }

        // Success rate alert
        if metrics.success_rate_percent < 95.0 {
            let severity = if metrics.success_rate_percent < 90.0 { "critical" } else { "warning" };
            alerts.push(PerformanceAlertInfo {
                alert_id: format!("low-success-rate-{}", timestamp),
                timestamp,
                severity: severity.to_string(),
                alert_type: "low_success_rate".to_string(),
                message: format!("Authentication success rate {:.1}% below optimal", metrics.success_rate_percent),
                metric_name: "success_rate_percent".to_string(),
                current_value: metrics.success_rate_percent,
                threshold_value: 95.0,
                suggested_action: "Review authentication failures and improve error handling".to_string(),
            });
        }

        // Capacity alert
        if metrics.nonce_store_utilization_percent > 85.0 {
            let severity = if metrics.nonce_store_utilization_percent > 95.0 { "critical" } else { "warning" };
            alerts.push(PerformanceAlertInfo {
                alert_id: format!("high-utilization-{}", timestamp),
                timestamp,
                severity: severity.to_string(),
                alert_type: "high_utilization".to_string(),
                message: format!("Nonce store utilization {:.1}% approaching capacity", metrics.nonce_store_utilization_percent),
                metric_name: "nonce_store_utilization_percent".to_string(),
                current_value: metrics.nonce_store_utilization_percent,
                threshold_value: 85.0,
                suggested_action: "Consider increasing capacity or improving cleanup efficiency".to_string(),
            });
        }

        alerts
    }

    /// Generate recommendations based on current performance
    fn generate_recommendations_from_metrics(&self, metrics: &SignatureAuthPerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Latency recommendations
        if metrics.avg_processing_time_ms > 10.0 {
            recommendations.push("Consider enabling caching for frequently accessed public keys".to_string());
            recommendations.push("Optimize database queries for key lookups".to_string());
            recommendations.push("Review signature verification algorithms for efficiency".to_string());
        }

        // Capacity recommendations
        if metrics.nonce_store_utilization_percent > 75.0 {
            recommendations.push("Monitor nonce store growth and adjust cleanup policies".to_string());
            recommendations.push("Consider increasing nonce store capacity if usage continues to grow".to_string());
        }

        // Success rate recommendations
        if metrics.success_rate_percent < 95.0 {
            recommendations.push("Analyze authentication failure patterns for improvement opportunities".to_string());
            recommendations.push("Enhance error handling and user guidance for failed authentications".to_string());
        }

        // Performance monitoring recommendations
        if !recommendations.is_empty() {
            recommendations.push("Enable detailed performance monitoring for deeper insights".to_string());
            recommendations.push("Set up automated alerting for performance degradation".to_string());
        }

        // If no issues, provide optimization suggestions
        if recommendations.is_empty() {
            recommendations.push("Performance is optimal - continue monitoring for sustained excellence".to_string());
            recommendations.push("Consider implementing proactive monitoring and alerting".to_string());
        }

        recommendations
    }
}

/// Helper function to integrate performance monitoring with existing HTTP server
pub fn add_performance_routes_to_server(
    cfg: &mut actix_web::web::ServiceConfig,
) {
    use crate::datafold_node::performance_routes::configure_performance_routes;
    configure_performance_routes(cfg);
}

/// Helper to create a performance monitoring wrapper for an existing signature auth state
pub fn create_performance_wrapper_for_state(
    state: &SignatureVerificationState,
) -> Result<PerformanceDashboard, FoldDbError> {
    let wrapper = PerformanceMonitoringWrapper::new();
    wrapper.create_dashboard_from_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::signature_auth::SignatureAuthConfig;

    #[test]
    fn test_performance_wrapper_creation() {
        let wrapper = PerformanceMonitoringWrapper::new();
        assert!(wrapper.start_time.elapsed().as_millis() < 100); // Should be recent
    }

    #[test]
    fn test_dashboard_creation_from_existing_state() {
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).unwrap();
        
        let wrapper = PerformanceMonitoringWrapper::new();
        let dashboard = wrapper.create_dashboard_from_state(&state);
        
        assert!(dashboard.is_ok());
        let dashboard = dashboard.unwrap();
        
        assert!(!dashboard.signature_auth_metrics.performance_target_compliance.recommendations.is_empty());
        assert!(dashboard.system_health.health_score > 0.0);
    }
}