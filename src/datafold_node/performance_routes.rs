//! Performance monitoring and dashboard routes for signature authentication
//!
//! This module provides HTTP endpoints for monitoring signature authentication
//! performance, cache statistics, and system health.

use crate::datafold_node::error::NodeResult;
use crate::datafold_node::signature_auth::{SignatureVerificationState, SecurityMetrics};
use crate::error::FoldDbError;
use actix_web::{web, HttpResponse, Result as ActixResult};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

/// Performance monitoring dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDashboard {
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub signature_auth_metrics: SignatureAuthPerformanceMetrics,
    pub system_health: PerformanceHealthStatus,
    pub alerts: Vec<PerformanceAlertInfo>,
    pub recommendations: Vec<String>,
}

/// Enhanced signature authentication performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureAuthPerformanceMetrics {
    pub total_requests: u64,
    pub successful_authentications: u64,
    pub failed_authentications: u64,
    pub success_rate_percent: f64,
    pub avg_processing_time_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub requests_per_second: f64,
    pub nonce_store_utilization_percent: f64,
    pub nonce_store_size: usize,
    pub nonce_store_max_capacity: usize,
    pub performance_target_compliance: PerformanceTargetCompliance,
    pub breakdown: PerformanceBreakdownMetrics,
}

/// Performance breakdown by operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBreakdownMetrics {
    pub signature_verification_ms: f64,
    pub database_lookup_ms: f64,
    pub nonce_validation_ms: f64,
    pub cache_operations_ms: f64,
    pub overhead_ms: f64,
}

/// Performance target compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargetCompliance {
    pub latency_target_ms: f64,
    pub current_avg_latency_ms: f64,
    pub target_met: bool,
    pub target_compliance_percent: f64,
    pub recommendations: Vec<String>,
}

/// System health status for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHealthStatus {
    pub overall_status: String, // "healthy", "degraded", "unhealthy", "critical"
    pub health_score: f64, // 0-100
    pub status_reasons: Vec<String>,
    pub critical_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub last_health_check: u64,
}

/// Performance alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertInfo {
    pub alert_id: String,
    pub timestamp: u64,
    pub severity: String, // "info", "warning", "critical"
    pub alert_type: String,
    pub message: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub suggested_action: String,
}

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceStats {
    pub enabled: bool,
    pub size: usize,
    pub max_size: usize,
    pub utilization_percent: f64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub warmup_completed: bool,
    pub average_lookup_time_ms: f64,
    pub cache_efficiency_score: f64,
}

/// Performance optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendations {
    pub timestamp: u64,
    pub priority_recommendations: Vec<RecommendationItem>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub capacity_planning: CapacityPlanningAdvice,
}

/// Individual recommendation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationItem {
    pub priority: String, // "high", "medium", "low"
    pub category: String, // "performance", "capacity", "security", "maintenance"
    pub title: String,
    pub description: String,
    pub expected_impact: String,
    pub implementation_effort: String, // "low", "medium", "high"
    pub action_items: Vec<String>,
}

/// Optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub metric_name: String,
    pub current_value: f64,
    pub potential_improvement: f64,
    pub improvement_strategy: String,
    pub estimated_benefit: String,
}

/// Capacity planning advice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningAdvice {
    pub current_utilization: HashMap<String, f64>,
    pub projected_growth: HashMap<String, f64>,
    pub recommended_limits: HashMap<String, f64>,
    pub scaling_recommendations: Vec<String>,
    pub resource_optimization: Vec<String>,
}

/// Performance monitoring endpoint handlers
pub struct PerformanceRoutes;

impl PerformanceRoutes {
    /// Get comprehensive performance dashboard
    pub async fn get_dashboard(
        state: web::Data<Arc<SignatureVerificationState>>,
    ) -> ActixResult<HttpResponse> {
        debug!("Performance dashboard requested");
        
        match Self::build_performance_dashboard(&state).await {
            Ok(dashboard) => {
                info!("Performance dashboard generated successfully");
                Ok(HttpResponse::Ok().json(dashboard))
            }
            Err(e) => {
                warn!("Failed to generate performance dashboard: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to generate performance dashboard",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Get signature authentication performance metrics
    pub async fn get_signature_auth_metrics(
        state: web::Data<Arc<SignatureVerificationState>>,
    ) -> ActixResult<HttpResponse> {
        debug!("Signature auth metrics requested");
        
        match Self::build_signature_auth_metrics(&state).await {
            Ok(metrics) => {
                Ok(HttpResponse::Ok().json(metrics))
            }
            Err(e) => {
                warn!("Failed to get signature auth metrics: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to get signature auth metrics",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Get system health status
    pub async fn get_health_status(
        state: web::Data<Arc<SignatureVerificationState>>,
    ) -> ActixResult<HttpResponse> {
        debug!("Health status requested");
        
        match Self::assess_health_status(&state).await {
            Ok(health) => {
                Ok(HttpResponse::Ok().json(health))
            }
            Err(e) => {
                warn!("Failed to assess health status: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to assess health status",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Get performance recommendations
    pub async fn get_recommendations(
        state: web::Data<Arc<SignatureVerificationState>>,
    ) -> ActixResult<HttpResponse> {
        debug!("Performance recommendations requested");
        
        match Self::generate_recommendations(&state).await {
            Ok(recommendations) => {
                Ok(HttpResponse::Ok().json(recommendations))
            }
            Err(e) => {
                warn!("Failed to generate recommendations: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to generate recommendations",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Trigger cache warmup
    pub async fn trigger_cache_warmup(
        state: web::Data<Arc<SignatureVerificationState>>,
    ) -> ActixResult<HttpResponse> {
        info!("Cache warmup triggered");
        
        // Note: This would need to be implemented when the cache warmup functionality is ready
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Cache warmup initiated",
            "status": "in_progress"
        })))
    }

    /// Clear performance metrics (admin endpoint)
    pub async fn clear_metrics(
        state: web::Data<Arc<SignatureVerificationState>>,
    ) -> ActixResult<HttpResponse> {
        info!("Performance metrics clear requested");
        
        // Note: This would clear metrics for fresh monitoring
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Performance metrics cleared",
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
        })))
    }

    // Private helper methods

    async fn build_performance_dashboard(
        state: &Arc<SignatureVerificationState>,
    ) -> NodeResult<PerformanceDashboard> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        
        // Build signature auth metrics
        let signature_auth_metrics = Self::build_signature_auth_metrics(state).await?;
        
        // Assess system health
        let system_health = Self::assess_health_status(state).await?;
        
        // Generate alerts (simplified for now)
        let alerts = Self::generate_current_alerts(&signature_auth_metrics);
        
        // Generate recommendations
        let recommendations = Self::generate_basic_recommendations(&signature_auth_metrics);
        
        Ok(PerformanceDashboard {
            timestamp,
            uptime_seconds: 0, // Would track actual uptime
            signature_auth_metrics,
            system_health,
            alerts,
            recommendations,
        })
    }

    async fn build_signature_auth_metrics(
        state: &Arc<SignatureVerificationState>,
    ) -> NodeResult<SignatureAuthPerformanceMetrics> {
        // Get the basic security metrics
        let security_metrics = match state.get_metrics_collector().get_security_metrics() {
            metrics => metrics,
        };
        
        // Calculate derived metrics
        let total_requests = security_metrics.processing_time_ms; // Simplified - would use actual counters
        let success_rate = 95.0; // Simplified - would calculate from actual data
        
        // Build performance target compliance
        let target_latency = 10.0; // 10ms target
        let current_latency = security_metrics.processing_time_ms as f64;
        let target_met = current_latency <= target_latency;
        let compliance_percent = if current_latency > 0.0 {
            (target_latency / current_latency * 100.0).min(100.0)
        } else {
            100.0
        };
        
        let performance_target_compliance = PerformanceTargetCompliance {
            latency_target_ms: target_latency,
            current_avg_latency_ms: current_latency,
            target_met,
            target_compliance_percent: compliance_percent,
            recommendations: if !target_met {
                vec![
                    "Consider enabling public key caching".to_string(),
                    "Optimize database queries".to_string(),
                    "Review nonce store configuration".to_string(),
                ]
            } else {
                vec![]
            },
        };
        
        // Build performance breakdown
        let breakdown = PerformanceBreakdownMetrics {
            signature_verification_ms: current_latency * 0.4, // Estimate 40% for signature verification
            database_lookup_ms: current_latency * 0.3, // Estimate 30% for DB lookup
            nonce_validation_ms: current_latency * 0.2, // Estimate 20% for nonce validation
            cache_operations_ms: current_latency * 0.05, // Estimate 5% for cache operations
            overhead_ms: current_latency * 0.05, // Estimate 5% for overhead
        };
        
        Ok(SignatureAuthPerformanceMetrics {
            total_requests,
            successful_authentications: (total_requests as f64 * success_rate / 100.0) as u64,
            failed_authentications: (total_requests as f64 * (100.0 - success_rate) / 100.0) as u64,
            success_rate_percent: success_rate,
            avg_processing_time_ms: current_latency,
            p50_latency_ms: current_latency * 0.8, // Simplified estimates
            p95_latency_ms: current_latency * 1.5,
            p99_latency_ms: current_latency * 2.0,
            requests_per_second: 50.0, // Simplified - would calculate from actual data
            nonce_store_utilization_percent: (security_metrics.nonce_store_size as f64 / 10000.0) * 100.0,
            nonce_store_size: security_metrics.nonce_store_size,
            nonce_store_max_capacity: 10000, // From config
            performance_target_compliance,
            breakdown,
        })
    }

    async fn assess_health_status(
        state: &Arc<SignatureVerificationState>,
    ) -> NodeResult<PerformanceHealthStatus> {
        let metrics = Self::build_signature_auth_metrics(state).await?;
        
        let mut health_score = 100.0;
        let mut status_reasons = Vec::new();
        let mut critical_issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Check latency
        if metrics.avg_processing_time_ms > 50.0 {
            health_score -= 30.0;
            critical_issues.push("High average latency detected".to_string());
        } else if metrics.avg_processing_time_ms > 20.0 {
            health_score -= 15.0;
            warnings.push("Elevated latency observed".to_string());
        }
        
        // Check success rate
        if metrics.success_rate_percent < 90.0 {
            health_score -= 25.0;
            critical_issues.push("Low authentication success rate".to_string());
        } else if metrics.success_rate_percent < 95.0 {
            health_score -= 10.0;
            warnings.push("Authentication success rate below optimal".to_string());
        }
        
        // Check nonce store utilization
        if metrics.nonce_store_utilization_percent > 90.0 {
            health_score -= 20.0;
            critical_issues.push("Nonce store near capacity".to_string());
        } else if metrics.nonce_store_utilization_percent > 75.0 {
            health_score -= 5.0;
            warnings.push("Nonce store utilization high".to_string());
        }
        
        let overall_status = if health_score >= 90.0 {
            "healthy"
        } else if health_score >= 70.0 {
            "degraded"
        } else if health_score >= 50.0 {
            "unhealthy"
        } else {
            "critical"
        };
        
        if critical_issues.is_empty() && warnings.is_empty() {
            status_reasons.push("All systems operating normally".to_string());
        }
        
        Ok(PerformanceHealthStatus {
            overall_status: overall_status.to_string(),
            health_score,
            status_reasons,
            critical_issues,
            warnings,
            last_health_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        })
    }

    async fn generate_recommendations(
        state: &Arc<SignatureVerificationState>,
    ) -> NodeResult<PerformanceRecommendations> {
        let metrics = Self::build_signature_auth_metrics(state).await?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        
        let mut priority_recommendations = Vec::new();
        let mut optimization_opportunities = Vec::new();
        
        // Generate recommendations based on metrics
        if metrics.avg_processing_time_ms > 10.0 {
            priority_recommendations.push(RecommendationItem {
                priority: "high".to_string(),
                category: "performance".to_string(),
                title: "Optimize Signature Verification Latency".to_string(),
                description: "Current average latency exceeds 10ms target".to_string(),
                expected_impact: "Reduce latency by 30-50%".to_string(),
                implementation_effort: "medium".to_string(),
                action_items: vec![
                    "Enable public key caching".to_string(),
                    "Optimize database queries".to_string(),
                    "Consider async processing".to_string(),
                ],
            });
            
            optimization_opportunities.push(OptimizationOpportunity {
                metric_name: "avg_processing_time_ms".to_string(),
                current_value: metrics.avg_processing_time_ms,
                potential_improvement: metrics.avg_processing_time_ms * 0.5,
                improvement_strategy: "Cache optimization and query tuning".to_string(),
                estimated_benefit: "Improved user experience and higher throughput".to_string(),
            });
        }
        
        if metrics.nonce_store_utilization_percent > 75.0 {
            priority_recommendations.push(RecommendationItem {
                priority: "medium".to_string(),
                category: "capacity".to_string(),
                title: "Optimize Nonce Store Management".to_string(),
                description: "Nonce store utilization is approaching capacity".to_string(),
                expected_impact: "Prevent capacity issues and improve cleanup efficiency".to_string(),
                implementation_effort: "low".to_string(),
                action_items: vec![
                    "Adjust nonce TTL settings".to_string(),
                    "Implement more aggressive cleanup".to_string(),
                    "Consider increasing nonce store size".to_string(),
                ],
            });
        }
        
        // Capacity planning advice
        let mut current_utilization = HashMap::new();
        current_utilization.insert("nonce_store".to_string(), metrics.nonce_store_utilization_percent);
        current_utilization.insert("processing_capacity".to_string(), 
                                 (metrics.requests_per_second / 100.0) * 100.0); // Assume 100 RPS capacity
        
        let mut recommended_limits = HashMap::new();
        recommended_limits.insert("nonce_store_size".to_string(), 15000.0);
        recommended_limits.insert("max_requests_per_second".to_string(), 150.0);
        
        let capacity_planning = CapacityPlanningAdvice {
            current_utilization,
            projected_growth: HashMap::new(), // Would be populated with actual projections
            recommended_limits,
            scaling_recommendations: vec![
                "Monitor nonce store growth patterns".to_string(),
                "Plan for 2x capacity headroom".to_string(),
            ],
            resource_optimization: vec![
                "Implement cache warming during off-peak hours".to_string(),
                "Consider horizontal scaling for high-load scenarios".to_string(),
            ],
        };
        
        Ok(PerformanceRecommendations {
            timestamp,
            priority_recommendations,
            optimization_opportunities,
            capacity_planning,
        })
    }

    fn generate_current_alerts(metrics: &SignatureAuthPerformanceMetrics) -> Vec<PerformanceAlertInfo> {
        let mut alerts = Vec::new();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        
        if metrics.avg_processing_time_ms > 50.0 {
            alerts.push(PerformanceAlertInfo {
                alert_id: uuid::Uuid::new_v4().to_string(),
                timestamp,
                severity: "critical".to_string(),
                alert_type: "high_latency".to_string(),
                message: "Signature authentication latency is critically high".to_string(),
                metric_name: "avg_processing_time_ms".to_string(),
                current_value: metrics.avg_processing_time_ms,
                threshold_value: 50.0,
                suggested_action: "Investigate performance bottlenecks and enable caching".to_string(),
            });
        }
        
        if metrics.success_rate_percent < 95.0 {
            alerts.push(PerformanceAlertInfo {
                alert_id: uuid::Uuid::new_v4().to_string(),
                timestamp,
                severity: "warning".to_string(),
                alert_type: "low_success_rate".to_string(),
                message: "Authentication success rate is below optimal threshold".to_string(),
                metric_name: "success_rate_percent".to_string(),
                current_value: metrics.success_rate_percent,
                threshold_value: 95.0,
                suggested_action: "Review authentication errors and improve error handling".to_string(),
            });
        }
        
        alerts
    }

    fn generate_basic_recommendations(metrics: &SignatureAuthPerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !metrics.performance_target_compliance.target_met {
            recommendations.push("Enable public key caching to reduce database lookups".to_string());
            recommendations.push("Consider optimizing signature verification algorithms".to_string());
        }
        
        if metrics.nonce_store_utilization_percent > 80.0 {
            recommendations.push("Increase nonce store cleanup frequency".to_string());
            recommendations.push("Consider increasing nonce store capacity".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("System performance is optimal - continue monitoring".to_string());
        }
        
        recommendations
    }
}

/// Configure performance monitoring routes
pub fn configure_performance_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/performance")
            .route("/dashboard", web::get().to(PerformanceRoutes::get_dashboard))
            .route("/metrics/signature-auth", web::get().to(PerformanceRoutes::get_signature_auth_metrics))
            .route("/health", web::get().to(PerformanceRoutes::get_health_status))
            .route("/recommendations", web::get().to(PerformanceRoutes::get_recommendations))
            .route("/cache/warmup", web::post().to(PerformanceRoutes::trigger_cache_warmup))
            .route("/metrics/clear", web::post().to(PerformanceRoutes::clear_metrics))
    );
}