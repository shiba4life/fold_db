//! Health monitoring and diagnostics for key rotation operations
//!
//! This module provides comprehensive health monitoring including:
//! - Real-time monitoring of rotation health
//! - Early warning systems for potential failures
//! - Automatic escalation for critical failures
//! - Integration with existing alerting infrastructure
//! - Health checks and diagnostics

use crate::crypto::key_rotation_error_handling::{KeyRotationErrorHandler, ErrorCategory, ErrorSeverity};
use crate::crypto::key_rotation_recovery::{KeyRotationRecoveryManager, RecoverySystemState};
use crate::db_operations::key_rotation_operations::{KeyRotationRecord, RotationStatus};
use crate::db_operations::core::DbOperations;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use log::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tokio::time::Instant;
use uuid::Uuid;

/// Overall health status of key rotation system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationHealthStatus {
    /// System is operating normally
    Healthy,
    /// System has minor issues but is functional
    Warning,
    /// System has significant issues affecting performance
    Degraded,
    /// System has critical issues requiring immediate attention
    Critical,
    /// System is unavailable
    Unavailable,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check identifier
    pub check_id: String,
    /// Check name
    pub check_name: String,
    /// Check status
    pub status: HealthCheckStatus,
    /// Check result message
    pub message: String,
    /// When check was performed
    pub timestamp: DateTime<Utc>,
    /// Check execution duration
    pub duration: Duration,
    /// Check metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCheckStatus {
    /// Check passed
    Pass,
    /// Check failed but not critical
    Warn,
    /// Check failed critically
    Fail,
    /// Check could not be performed
    Unknown,
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Overall system health status
    pub overall_status: RotationHealthStatus,
    /// Last health check timestamp
    pub last_check_timestamp: DateTime<Utc>,
    /// Total rotations in last 24 hours
    pub rotations_24h: u64,
    /// Failed rotations in last 24 hours
    pub failed_rotations_24h: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average rotation duration
    pub avg_rotation_duration: Option<Duration>,
    /// Current active rotations
    pub active_rotations: u64,
    /// Queue length
    pub queue_length: u64,
    /// Error rate by category
    pub error_rates: HashMap<ErrorCategory, f64>,
    /// Recovery system status
    pub recovery_status: Option<RecoverySystemState>,
    /// Recent alerts
    pub recent_alerts: Vec<HealthAlert>,
    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert identifier
    pub alert_id: Uuid,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert category
    pub category: AlertCategory,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// When alert was triggered
    pub triggered_at: DateTime<Utc>,
    /// Alert status
    pub status: AlertStatus,
    /// Alert metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert requiring attention
    Error,
    /// Critical alert requiring immediate action
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCategory {
    /// Performance related alerts
    Performance,
    /// Error rate alerts
    ErrorRate,
    /// System resource alerts
    Resources,
    /// Security related alerts
    Security,
    /// Recovery system alerts
    Recovery,
    /// Configuration alerts
    Configuration,
}

/// Alert status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active
    Active,
    /// Alert has been acknowledged
    Acknowledged,
    /// Alert has been resolved
    Resolved,
    /// Alert was a false positive
    FalsePositive,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    /// Health check interval
    pub check_interval: Duration,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Alert evaluation interval
    pub alert_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
    /// Number of recent metrics to keep
    pub metrics_history_size: usize,
    /// Number of recent alerts to keep
    pub alerts_history_size: usize,
    /// Enable automatic recovery
    pub enable_auto_recovery: bool,
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
    /// Error rate thresholds
    pub error_rate_thresholds: ErrorRateThresholds,
}

/// Performance threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum acceptable rotation duration (seconds)
    pub max_rotation_duration_secs: f64,
    /// Maximum acceptable queue length
    pub max_queue_length: u64,
    /// Minimum acceptable success rate percentage
    pub min_success_rate: f64,
    /// Maximum acceptable active rotations
    pub max_active_rotations: u64,
}

/// Error rate threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRateThresholds {
    /// Maximum acceptable overall error rate percentage
    pub max_overall_error_rate: f64,
    /// Maximum acceptable network error rate percentage
    pub max_network_error_rate: f64,
    /// Maximum acceptable database error rate percentage
    pub max_database_error_rate: f64,
    /// Maximum acceptable security error rate percentage
    pub max_security_error_rate: f64,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            metrics_interval: Duration::from_secs(60),
            alert_interval: Duration::from_secs(10),
            check_timeout: Duration::from_secs(30),
            metrics_history_size: 1000,
            alerts_history_size: 500,
            enable_auto_recovery: true,
            performance_thresholds: PerformanceThresholds {
                max_rotation_duration_secs: 300.0, // 5 minutes
                max_queue_length: 1000,
                min_success_rate: 95.0,
                max_active_rotations: 100,
            },
            error_rate_thresholds: ErrorRateThresholds {
                max_overall_error_rate: 5.0,
                max_network_error_rate: 10.0,
                max_database_error_rate: 2.0,
                max_security_error_rate: 0.1,
            },
        }
    }
}

/// Key rotation health monitor
pub struct KeyRotationHealthMonitor {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// Error handler
    error_handler: Arc<Mutex<KeyRotationErrorHandler>>,
    /// Recovery manager
    recovery_manager: Option<Arc<KeyRotationRecoveryManager>>,
    /// Configuration
    config: HealthMonitorConfig,
    /// Current health metrics
    current_metrics: Arc<RwLock<HealthMetrics>>,
    /// Health check history
    health_checks: Arc<RwLock<VecDeque<HealthCheck>>>,
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<Uuid, HealthAlert>>>,
    /// Alert history
    alert_history: Arc<RwLock<VecDeque<HealthAlert>>>,
    /// Metrics history
    metrics_history: Arc<RwLock<VecDeque<HealthMetrics>>>,
    /// Last health check time
    last_health_check: Arc<RwLock<Option<Instant>>>,
}

impl KeyRotationHealthMonitor {
    /// Create a new health monitor
    pub fn new(
        db_ops: Arc<DbOperations>,
        error_handler: Arc<Mutex<KeyRotationErrorHandler>>,
        recovery_manager: Option<Arc<KeyRotationRecoveryManager>>,
    ) -> Self {
        let config = HealthMonitorConfig::default();
        
        Self {
            db_ops,
            error_handler,
            recovery_manager,
            config: config.clone(),
            current_metrics: Arc::new(RwLock::new(HealthMetrics {
                overall_status: RotationHealthStatus::Healthy,
                last_check_timestamp: Utc::now(),
                rotations_24h: 0,
                failed_rotations_24h: 0,
                success_rate: 100.0,
                avg_rotation_duration: None,
                active_rotations: 0,
                queue_length: 0,
                error_rates: HashMap::new(),
                recovery_status: None,
                recent_alerts: Vec::new(),
                performance_metrics: HashMap::new(),
            })),
            health_checks: Arc::new(RwLock::new(VecDeque::with_capacity(config.metrics_history_size))),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.alerts_history_size))),
            metrics_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.metrics_history_size))),
            last_health_check: Arc::new(RwLock::new(None)),
        }
    }

    /// Create health monitor with custom configuration
    pub fn with_config(
        db_ops: Arc<DbOperations>,
        error_handler: Arc<Mutex<KeyRotationErrorHandler>>,
        recovery_manager: Option<Arc<KeyRotationRecoveryManager>>,
        config: HealthMonitorConfig,
    ) -> Self {
        Self {
            config: config.clone(),
            health_checks: Arc::new(RwLock::new(VecDeque::with_capacity(config.metrics_history_size))),
            alert_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.alerts_history_size))),
            metrics_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.metrics_history_size))),
            ..Self::new(db_ops, error_handler, recovery_manager)
        }
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let monitor = self.clone_for_monitoring();
        
        tokio::spawn(async move {
            let mut health_check_interval = tokio::time::interval(monitor.config.check_interval);
            let mut metrics_interval = tokio::time::interval(monitor.config.metrics_interval);
            let mut alert_interval = tokio::time::interval(monitor.config.alert_interval);
            
            loop {
                tokio::select! {
                    _ = health_check_interval.tick() => {
                        if let Err(e) = monitor.perform_health_checks().await {
                            error!("Health check failed: {}", e);
                        }
                    }
                    _ = metrics_interval.tick() => {
                        if let Err(e) = monitor.collect_metrics().await {
                            error!("Metrics collection failed: {}", e);
                        }
                    }
                    _ = alert_interval.tick() => {
                        if let Err(e) = monitor.evaluate_alerts().await {
                            error!("Alert evaluation failed: {}", e);
                        }
                    }
                }
            }
        })
    }

    /// Perform comprehensive health checks
    pub async fn perform_health_checks(&self) -> Result<Vec<HealthCheck>, String> {
        debug!("Performing health checks");
        
        let mut health_checks = Vec::new();
        
        // Check 1: Database connectivity
        let db_check = self.check_database_health().await;
        health_checks.push(db_check);
        
        // Check 2: Rotation queue health
        let queue_check = self.check_queue_health().await;
        health_checks.push(queue_check);
        
        // Check 3: Error handler health
        let error_handler_check = self.check_error_handler_health().await;
        health_checks.push(error_handler_check);
        
        // Check 4: Recovery system health
        if let Some(ref recovery_manager) = self.recovery_manager {
            let recovery_check = self.check_recovery_system_health(recovery_manager).await;
            health_checks.push(recovery_check);
        }
        
        // Check 5: Performance health
        let performance_check = self.check_performance_health().await;
        health_checks.push(performance_check);
        
        // Check 6: Resource health
        let resource_check = self.check_resource_health().await;
        health_checks.push(resource_check);
        
        // Store health checks
        {
            let mut checks = self.health_checks.write().await;
            for check in &health_checks {
                checks.push_back(check.clone());
                if checks.len() > self.config.metrics_history_size {
                    checks.pop_front();
                }
            }
        }
        
        // Update last health check time
        {
            let mut last_check = self.last_health_check.write().await;
            *last_check = Some(Instant::now());
        }
        
        info!("Completed {} health checks", health_checks.len());
        Ok(health_checks)
    }

    /// Collect system metrics
    pub async fn collect_metrics(&self) -> Result<HealthMetrics, String> {
        debug!("Collecting health metrics");
        
        let now = Utc::now();
        let metrics_start = Instant::now();
        
        // Get rotation statistics
        let rotation_stats = self.get_rotation_statistics(now).await?;
        
        // Get error statistics
        let error_stats = self.get_error_statistics().await?;
        
        // Get recovery system status
        let recovery_status = if let Some(ref recovery_manager) = self.recovery_manager {
            Some(recovery_manager.get_system_state().await)
        } else {
            None
        };
        
        // Get recent alerts
        let recent_alerts = {
            let alerts = self.active_alerts.read().await;
            alerts.values().cloned().collect()
        };
        
        // Calculate overall health status
        let overall_status = self.calculate_overall_health_status(&rotation_stats, &error_stats, &recovery_status).await;
        
        let metrics = HealthMetrics {
            overall_status,
            last_check_timestamp: now,
            rotations_24h: rotation_stats.get("total_rotations_24h")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            failed_rotations_24h: rotation_stats.get("failed_rotations_24h")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            success_rate: rotation_stats.get("success_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0),
            avg_rotation_duration: rotation_stats.get("avg_duration_ms")
                .and_then(|v| v.as_u64())
                .map(Duration::from_millis),
            active_rotations: rotation_stats.get("active_rotations")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            queue_length: rotation_stats.get("queue_length")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            error_rates: error_stats,
            recovery_status,
            recent_alerts,
            performance_metrics: HashMap::from([
                ("metrics_collection_duration_ms".to_string(), metrics_start.elapsed().as_millis() as f64),
            ]),
        };
        
        // Update current metrics
        {
            let mut current = self.current_metrics.write().await;
            *current = metrics.clone();
        }
        
        // Store metrics history
        {
            let mut history = self.metrics_history.write().await;
            history.push_back(metrics.clone());
            if history.len() > self.config.metrics_history_size {
                history.pop_front();
            }
        }
        
        debug!("Collected metrics with overall status: {:?}", metrics.overall_status);
        Ok(metrics)
    }

    /// Evaluate and trigger alerts
    pub async fn evaluate_alerts(&self) -> Result<Vec<HealthAlert>, String> {
        debug!("Evaluating alerts");
        
        let current_metrics = {
            let metrics = self.current_metrics.read().await;
            metrics.clone()
        };
        
        let mut new_alerts = Vec::new();
        
        // Performance alerts
        if let Some(performance_alerts) = self.evaluate_performance_alerts(&current_metrics).await {
            new_alerts.extend(performance_alerts);
        }
        
        // Error rate alerts
        if let Some(error_rate_alerts) = self.evaluate_error_rate_alerts(&current_metrics).await {
            new_alerts.extend(error_rate_alerts);
        }
        
        // Recovery system alerts
        if let Some(recovery_alerts) = self.evaluate_recovery_alerts(&current_metrics).await {
            new_alerts.extend(recovery_alerts);
        }
        
        // Resource alerts
        if let Some(resource_alerts) = self.evaluate_resource_alerts(&current_metrics).await {
            new_alerts.extend(resource_alerts);
        }
        
        // Process new alerts
        for alert in &new_alerts {
            self.process_alert(alert).await?;
        }
        
        debug!("Evaluated {} new alerts", new_alerts.len());
        Ok(new_alerts)
    }

    /// Get current health metrics
    pub async fn get_current_metrics(&self) -> HealthMetrics {
        let metrics = self.current_metrics.read().await;
        metrics.clone()
    }

    /// Get health metrics history
    pub async fn get_metrics_history(&self, limit: Option<usize>) -> Vec<HealthMetrics> {
        let history = self.metrics_history.read().await;
        
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.iter().cloned().collect()
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<HealthAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<HealthAlert> {
        let history = self.alert_history.read().await;
        
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.iter().cloned().collect()
        }
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &Uuid, acknowledged_by: &str) -> Result<(), String> {
        let mut alerts = self.active_alerts.write().await;
        
        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.metadata.insert("acknowledged_by".to_string(), serde_json::Value::String(acknowledged_by.to_string()));
            alert.metadata.insert("acknowledged_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
            
            info!("Alert {} acknowledged by {}", alert_id, acknowledged_by);
            Ok(())
        } else {
            Err(format!("Alert {} not found", alert_id))
        }
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &Uuid, resolved_by: &str, resolution_note: Option<&str>) -> Result<(), String> {
        let mut alerts = self.active_alerts.write().await;
        
        if let Some(mut alert) = alerts.remove(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.metadata.insert("resolved_by".to_string(), serde_json::Value::String(resolved_by.to_string()));
            alert.metadata.insert("resolved_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
            
            if let Some(note) = resolution_note {
                alert.metadata.insert("resolution_note".to_string(), serde_json::Value::String(note.to_string()));
            }
            
            // Move to history
            {
                let mut history = self.alert_history.write().await;
                history.push_back(alert);
                if history.len() > self.config.alerts_history_size {
                    history.pop_front();
                }
            }
            
            info!("Alert {} resolved by {}", alert_id, resolved_by);
            Ok(())
        } else {
            Err(format!("Alert {} not found", alert_id))
        }
    }

    // Helper methods for health checks
    
    async fn check_database_health(&self) -> HealthCheck {
        let start_time = Instant::now();
        let check_id = "database_health".to_string();
        
        // Test database connectivity and basic operations
        let (status, message) = match self.test_database_operations().await {
            Ok(_) => (HealthCheckStatus::Pass, "Database operations are healthy".to_string()),
            Err(e) => (HealthCheckStatus::Fail, format!("Database health check failed: {}", e)),
        };
        
        HealthCheck {
            check_id: check_id.clone(),
            check_name: "Database Health".to_string(),
            status,
            message,
            timestamp: Utc::now(),
            duration: start_time.elapsed(),
            metadata: HashMap::new(),
        }
    }

    async fn check_queue_health(&self) -> HealthCheck {
        let start_time = Instant::now();
        let check_id = "queue_health".to_string();
        
        // Check queue status and performance
        let (status, message) = match self.get_queue_status().await {
            Ok(queue_stats) => {
                let queue_length = queue_stats.get("length").and_then(|v| v.as_u64()).unwrap_or(0);
                let processing_rate = queue_stats.get("processing_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                if queue_length > self.config.performance_thresholds.max_queue_length {
                    (HealthCheckStatus::Warn, format!("Queue length {} exceeds threshold {}", queue_length, self.config.performance_thresholds.max_queue_length))
                } else if processing_rate < 0.1 {
                    (HealthCheckStatus::Warn, "Low queue processing rate detected".to_string())
                } else {
                    (HealthCheckStatus::Pass, "Queue is healthy".to_string())
                }
            }
            Err(e) => (HealthCheckStatus::Fail, format!("Queue health check failed: {}", e)),
        };
        
        HealthCheck {
            check_id: check_id.clone(),
            check_name: "Queue Health".to_string(),
            status,
            message,
            timestamp: Utc::now(),
            duration: start_time.elapsed(),
            metadata: HashMap::new(),
        }
    }

    async fn check_error_handler_health(&self) -> HealthCheck {
        let start_time = Instant::now();
        let check_id = "error_handler_health".to_string();
        
        let (status, message) = {
            let error_handler = self.error_handler.lock().await;
            let error_stats = error_handler.get_error_statistics();
            let circuit_states = error_handler.get_circuit_breaker_states();
            
            let total_errors: u64 = error_stats.values().map(|s| s.total_count).sum();
            let open_circuits = circuit_states.values().filter(|s| s.state == crate::crypto::key_rotation_error_handling::CircuitState::Open).count();
            
            if open_circuits > 0 {
                (HealthCheckStatus::Warn, format!("{} circuit breakers are open", open_circuits))
            } else if total_errors > 100 {
                (HealthCheckStatus::Warn, format("High error count: {}", total_errors))
            } else {
                (HealthCheckStatus::Pass, "Error handler is healthy".to_string())
            }
        };
        
        HealthCheck {
            check_id: check_id.clone(),
            check_name: "Error Handler Health".to_string(),
            status,
            message,
            timestamp: Utc::now(),
            duration: start_time.elapsed(),
            metadata: HashMap::new(),
        }
    }

    async fn check_recovery_system_health(&self, recovery_manager: &KeyRotationRecoveryManager) -> HealthCheck {
        let start_time = Instant::now();
        let check_id = "recovery_system_health".to_string();
        
        let recovery_state = recovery_manager.get_system_state().await;
        
        let (status, message) = match recovery_state.health_status {
            crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Healthy => {
                (HealthCheckStatus::Pass, "Recovery system is healthy".to_string())
            }
            crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Warning => {
                (HealthCheckStatus::Warn, "Recovery system has warnings".to_string())
            }
            crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Degraded => {
                (HealthCheckStatus::Warn, "Recovery system is degraded".to_string())
            }
            crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Critical => {
                (HealthCheckStatus::Fail, "Recovery system is in critical state".to_string())
            }
        };
        
        HealthCheck {
            check_id: check_id.clone(),
            check_name: "Recovery System Health".to_string(),
            status,
            message,
            timestamp: Utc::now(),
            duration: start_time.elapsed(),
            metadata: HashMap::from([
                ("active_recoveries".to_string(), serde_json::json!(recovery_state.active_recoveries)),
                ("failed_operations".to_string(), serde_json::json!(recovery_state.failed_operations_detected)),
                ("success_rate".to_string(), serde_json::json!(recovery_state.recovery_success_rate)),
            ]),
        }
    }

    async fn check_performance_health(&self) -> HealthCheck {
        let start_time = Instant::now();
        let check_id = "performance_health".to_string();
        
        let current_metrics = {
            let metrics = self.current_metrics.read().await;
            metrics.clone()
        };
        
        let (status, message) = {
            let mut issues = Vec::new();
            
            if current_metrics.success_rate < self.config.performance_thresholds.min_success_rate {
                issues.push(format!("Success rate {:.2}% below threshold {:.2}%", 
                    current_metrics.success_rate, self.config.performance_thresholds.min_success_rate));
            }
            
            if current_metrics.queue_length > self.config.performance_thresholds.max_queue_length {
                issues.push(format!("Queue length {} exceeds threshold {}", 
                    current_metrics.queue_length, self.config.performance_thresholds.max_queue_length));
            }
            
            if current_metrics.active_rotations > self.config.performance_thresholds.max_active_rotations {
                issues.push(format!("Active rotations {} exceeds threshold {}", 
                    current_metrics.active_rotations, self.config.performance_thresholds.max_active_rotations));
            }
            
            if let Some(avg_duration) = current_metrics.avg_rotation_duration {
                if avg_duration.as_secs_f64() > self.config.performance_thresholds.max_rotation_duration_secs {
                    issues.push(format!("Average rotation duration {:.2}s exceeds threshold {:.2}s", 
                        avg_duration.as_secs_f64(), self.config.performance_thresholds.max_rotation_duration_secs));
                }
            }
            
            if issues.is_empty() {
                (HealthCheckStatus::Pass, "Performance is within acceptable thresholds".to_string())
            } else {
                (HealthCheckStatus::Warn, issues.join("; "))
            }
        };
        
        HealthCheck {
            check_id: check_id.clone(),
            check_name: "Performance Health".to_string(),
            status,
            message,
            timestamp: Utc::now(),
            duration: start_time.elapsed(),
            metadata: HashMap::from([
                ("success_rate".to_string(), serde_json::json!(current_metrics.success_rate)),
                ("queue_length".to_string(), serde_json::json!(current_metrics.queue_length)),
                ("active_rotations".to_string(), serde_json::json!(current_metrics.active_rotations)),
            ]),
        }
    }

    async fn check_resource_health(&self) -> HealthCheck {
        let start_time = Instant::now();
        let check_id = "resource_health".to_string();
        
        // Check system resources (memory, disk, etc.)
        let (status, message) = match self.check_system_resources().await {
            Ok(resources) => {
                let memory_usage = resources.get("memory_usage_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let disk_usage = resources.get("disk_usage_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                if memory_usage > 90.0 || disk_usage > 90.0 {
                    (HealthCheckStatus::Fail, format!("Resource usage critical: memory {:.1}%, disk {:.1}%", memory_usage, disk_usage))
                } else if memory_usage > 80.0 || disk_usage > 80.0 {
                    (HealthCheckStatus::Warn, format!("Resource usage high: memory {:.1}%, disk {:.1}%", memory_usage, disk_usage))
                } else {
                    (HealthCheckStatus::Pass, "Resource usage is normal".to_string())
                }
            }
            Err(e) => (HealthCheckStatus::Unknown, format!("Could not check resources: {}", e)),
        };
        
        HealthCheck {
            check_id: check_id.clone(),
            check_name: "Resource Health".to_string(),
            status,
            message,
            timestamp: Utc::now(),
            duration: start_time.elapsed(),
            metadata: HashMap::new(),
        }
    }

    // Helper methods (implementation stubs)
    
    async fn test_database_operations(&self) -> Result<(), String> {
        // Test basic database operations
        Ok(()) // Placeholder
    }

    async fn get_queue_status(&self) -> Result<HashMap<String, serde_json::Value>, String> {
        // Get queue status from system
        Ok(HashMap::from([
            ("length".to_string(), serde_json::json!(0)),
            ("processing_rate".to_string(), serde_json::json!(1.0)),
        ])) // Placeholder
    }

    async fn get_rotation_statistics(&self, _now: DateTime<Utc>) -> Result<HashMap<String, serde_json::Value>, String> {
        // Get rotation statistics from database
        match self.db_ops.get_rotation_statistics() {
            Ok(stats) => Ok(stats),
            Err(e) => Err(format!("Failed to get rotation statistics: {}", e)),
        }
    }

    async fn get_error_statistics(&self) -> Result<HashMap<ErrorCategory, f64>, String> {
        // Get error statistics from error handler
        let error_handler = self.error_handler.lock().await;
        let error_stats = error_handler.get_error_statistics();
        
        let mut error_rates = HashMap::new();
        
        for (category, stats) in error_stats {
            // Calculate error rate (errors per hour)
            let rate = if let Some(last_occurrence) = stats.last_occurrence {
                let hours_since = (Utc::now() - last_occurrence).num_hours() as f64;
                if hours_since > 0.0 {
                    stats.total_count as f64 / hours_since
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            error_rates.insert(category, rate);
        }
        
        Ok(error_rates)
    }

    async fn check_system_resources(&self) -> Result<HashMap<String, serde_json::Value>, String> {
        // Check system resources
        Ok(HashMap::from([
            ("memory_usage_percent".to_string(), serde_json::json!(50.0)),
            ("disk_usage_percent".to_string(), serde_json::json!(30.0)),
        ])) // Placeholder
    }

    async fn calculate_overall_health_status(
        &self,
        rotation_stats: &HashMap<String, serde_json::Value>,
        error_stats: &HashMap<ErrorCategory, f64>,
        recovery_status: &Option<RecoverySystemState>,
    ) -> RotationHealthStatus {
        let success_rate = rotation_stats.get("success_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);
        
        let total_error_rate: f64 = error_stats.values().sum();
        
        let recovery_health = recovery_status.as_ref()
            .map(|s| &s.health_status)
            .unwrap_or(&crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Healthy);
        
        // Determine overall status based on multiple factors
        if success_rate < 80.0 || total_error_rate > 20.0 || 
           matches!(recovery_health, crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Critical) {
            RotationHealthStatus::Critical
        } else if success_rate < 90.0 || total_error_rate > 10.0 || 
                  matches!(recovery_health, crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Degraded) {
            RotationHealthStatus::Degraded
        } else if success_rate < 95.0 || total_error_rate > 5.0 || 
                  matches!(recovery_health, crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Warning) {
            RotationHealthStatus::Warning
        } else {
            RotationHealthStatus::Healthy
        }
    }

    // Alert evaluation methods (implementation stubs)
    
    async fn evaluate_performance_alerts(&self, _metrics: &HealthMetrics) -> Option<Vec<HealthAlert>> {
        // Evaluate performance-related alerts
        None // Placeholder
    }

    async fn evaluate_error_rate_alerts(&self, _metrics: &HealthMetrics) -> Option<Vec<HealthAlert>> {
        // Evaluate error rate alerts
        None // Placeholder
    }

    async fn evaluate_recovery_alerts(&self, _metrics: &HealthMetrics) -> Option<Vec<HealthAlert>> {
        // Evaluate recovery system alerts
        None // Placeholder
    }

    async fn evaluate_resource_alerts(&self, _metrics: &HealthMetrics) -> Option<Vec<HealthAlert>> {
        // Evaluate resource alerts
        None // Placeholder
    }

    async fn process_alert(&self, alert: &HealthAlert) -> Result<(), String> {
        // Process new alert (store, notify, etc.)
        let mut alerts = self.active_alerts.write().await;
        alerts.insert(alert.alert_id, alert.clone());
        
        info!("New alert triggered: {} - {}", alert.title, alert.description);
        Ok(())
    }

    fn clone_for_monitoring(&self) -> KeyRotationHealthMonitor {
        KeyRotationHealthMonitor {
            db_ops: Arc::clone(&self.db_ops),
            error_handler: Arc::clone(&self.error_handler),
            recovery_manager: self.recovery_manager.as_ref().map(Arc::clone),
            config: self.config.clone(),
            current_metrics: Arc::clone(&self.current_metrics),
            health_checks: Arc::clone(&self.health_checks),
            active_alerts: Arc::clone(&self.active_alerts),
            alert_history: Arc::clone(&self.alert_history),
            metrics_history: Arc::clone(&self.metrics_history),
            last_health_check: Arc::clone(&self.last_health_check),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_db_ops() -> Arc<DbOperations> {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        Arc::new(DbOperations::new(db).unwrap())
    }

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let db_ops = create_test_db_ops();
        let error_handler = Arc::new(Mutex::new(KeyRotationErrorHandler::new()));
        
        let health_monitor = KeyRotationHealthMonitor::new(
            db_ops,
            error_handler,
            None,
        );
        
        let metrics = health_monitor.get_current_metrics().await;
        assert_eq!(metrics.overall_status, RotationHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_checks() {
        let db_ops = create_test_db_ops();
        let error_handler = Arc::new(Mutex::new(KeyRotationErrorHandler::new()));
        
        let health_monitor = KeyRotationHealthMonitor::new(
            db_ops,
            error_handler,
            None,
        );
        
        let health_checks = health_monitor.perform_health_checks().await.unwrap();
        assert!(!health_checks.is_empty());
        
        // All checks should pass for a clean system
        for check in &health_checks {
            assert!(matches!(check.status, HealthCheckStatus::Pass | HealthCheckStatus::Unknown));
        }
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let db_ops = create_test_db_ops();
        let error_handler = Arc::new(Mutex::new(KeyRotationErrorHandler::new()));
        
        let health_monitor = KeyRotationHealthMonitor::new(
            db_ops,
            error_handler,
            None,
        );
        
        let metrics = health_monitor.collect_metrics().await.unwrap();
        assert_eq!(metrics.overall_status, RotationHealthStatus::Healthy);
        assert!(metrics.performance_metrics.contains_key("metrics_collection_duration_ms"));
    }
}