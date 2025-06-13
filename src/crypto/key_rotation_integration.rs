//! Integration module for key rotation error handling and recovery
//!
//! This module provides a unified interface for all key rotation error handling
//! and recovery capabilities, integrating all components into a cohesive system.

use crate::crypto::key_rotation::{KeyRotationError, RotationContext, RotationReason};
use crate::crypto::key_rotation_error_handling::{KeyRotationErrorHandler, ErrorContext, RecoveryStrategy};
use crate::crypto::key_rotation_recovery::{
    KeyRotationRecoveryManager, RecoveryOperationType, RecoveryScope, RecoveryPlan, RecoveryResult
};
use crate::crypto::audit_logger::CryptoAuditLogger;
use crate::db_operations::key_rotation_operations::KeyRotationRecord;
use crate::db_operations::key_rotation_rollback::{RollbackPlan, RollbackRecord, RollbackReason};
use crate::db_operations::core::DbOperations;
use crate::datafold_node::key_rotation_health::{
    KeyRotationHealthMonitor, HealthMetrics, HealthAlert, RotationHealthStatus
};
use chrono::{DateTime, Utc};
use log::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use uuid::Uuid;

/// Integrated key rotation management system
pub struct IntegratedKeyRotationManager {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// Error handler
    error_handler: Arc<Mutex<KeyRotationErrorHandler>>,
    /// Recovery manager
    recovery_manager: Arc<KeyRotationRecoveryManager>,
    /// Health monitor
    health_monitor: Arc<KeyRotationHealthMonitor>,
    /// Audit logger
    audit_logger: Option<Arc<CryptoAuditLogger>>,
    /// System configuration
    config: IntegratedSystemConfig,
    /// System state
    system_state: Arc<RwLock<IntegratedSystemState>>,
}

/// Integrated system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedSystemConfig {
    /// Enable automatic error recovery
    pub enable_auto_recovery: bool,
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
    /// Enable automatic rollback on critical failures
    pub enable_auto_rollback: bool,
    /// Maximum concurrent recovery operations
    pub max_concurrent_recoveries: usize,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Error escalation thresholds
    pub escalation_thresholds: EscalationThresholds,
}

/// Error escalation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationThresholds {
    /// Critical error count threshold
    pub critical_error_threshold: u32,
    /// Failed operation percentage threshold
    pub failed_operation_threshold: f64,
    /// Recovery failure threshold
    pub recovery_failure_threshold: u32,
    /// Time window for threshold evaluation (seconds)
    pub evaluation_window_secs: u64,
}

impl Default for IntegratedSystemConfig {
    fn default() -> Self {
        Self {
            enable_auto_recovery: true,
            enable_health_monitoring: true,
            enable_auto_rollback: true,
            max_concurrent_recoveries: 5,
            health_check_interval: Duration::from_secs(30),
            escalation_thresholds: EscalationThresholds {
                critical_error_threshold: 5,
                failed_operation_threshold: 10.0,
                recovery_failure_threshold: 3,
                evaluation_window_secs: 300, // 5 minutes
            },
        }
    }
}

/// Integrated system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedSystemState {
    /// Overall system status
    pub overall_status: SystemStatus,
    /// Last status update
    pub last_update: DateTime<Utc>,
    /// Active error handling operations
    pub active_error_operations: usize,
    /// Active recovery operations
    pub active_recovery_operations: usize,
    /// Active rollback operations
    pub active_rollback_operations: usize,
    /// System performance metrics
    pub performance_metrics: HashMap<String, f64>,
    /// Recent escalations
    pub recent_escalations: Vec<SystemEscalation>,
}

/// Overall system status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemStatus {
    /// System is operating normally
    Operational,
    /// System has degraded performance
    Degraded,
    /// System is experiencing critical issues
    Critical,
    /// System is under emergency maintenance
    Emergency,
    /// System is unavailable
    Unavailable,
}

/// System escalation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEscalation {
    /// Escalation ID
    pub escalation_id: Uuid,
    /// Escalation type
    pub escalation_type: EscalationType,
    /// Escalation reason
    pub reason: String,
    /// When escalation occurred
    pub escalated_at: DateTime<Utc>,
    /// Escalation status
    pub status: EscalationStatus,
    /// Actions taken
    pub actions_taken: Vec<String>,
}

/// Types of system escalations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationType {
    /// High error rate detected
    HighErrorRate,
    /// Critical failure threshold exceeded
    CriticalFailures,
    /// Recovery system failure
    RecoveryFailure,
    /// Health monitoring alert
    HealthAlert,
    /// Manual escalation
    Manual,
}

/// Escalation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationStatus {
    /// Escalation is active
    Active,
    /// Escalation is being handled
    InProgress,
    /// Escalation has been resolved
    Resolved,
    /// Escalation was a false alarm
    FalseAlarm,
}

/// Comprehensive error handling result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveErrorResult {
    /// Error handling was successful
    pub success: bool,
    /// Error context
    pub error_context: ErrorContext,
    /// Recovery plan if applicable
    pub recovery_plan: Option<RecoveryPlan>,
    /// Rollback plan if applicable
    pub rollback_plan: Option<RollbackPlan>,
    /// Actions taken
    pub actions_taken: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Duration of error handling process
    pub duration: Duration,
}

impl IntegratedKeyRotationManager {
    /// Create a new integrated key rotation manager
    pub fn new(
        db_ops: Arc<DbOperations>,
        audit_logger: Option<Arc<CryptoAuditLogger>>,
    ) -> Self {
        let error_handler = Arc::new(Mutex::new(KeyRotationErrorHandler::new()));
        let recovery_manager = Arc::new(KeyRotationRecoveryManager::new(
            Arc::clone(&db_ops),
            Arc::clone(&error_handler),
            audit_logger.clone(),
        ));
        let health_monitor = Arc::new(KeyRotationHealthMonitor::new(
            Arc::clone(&db_ops),
            Arc::clone(&error_handler),
            Some(Arc::clone(&recovery_manager)),
        ));
        
        let config = IntegratedSystemConfig::default();
        
        Self {
            db_ops,
            error_handler,
            recovery_manager,
            health_monitor,
            audit_logger,
            config,
            system_state: Arc::new(RwLock::new(IntegratedSystemState {
                overall_status: SystemStatus::Operational,
                last_update: Utc::now(),
                active_error_operations: 0,
                active_recovery_operations: 0,
                active_rollback_operations: 0,
                performance_metrics: HashMap::new(),
                recent_escalations: Vec::new(),
            })),
        }
    }

    /// Create integrated manager with custom configuration
    pub fn with_config(
        db_ops: Arc<DbOperations>,
        audit_logger: Option<Arc<CryptoAuditLogger>>,
        config: IntegratedSystemConfig,
    ) -> Self {
        let mut manager = Self::new(db_ops, audit_logger);
        manager.config = config;
        manager
    }

    /// Start the integrated system
    pub async fn start_system(&self) -> Result<(), KeyRotationError> {
        info!("Starting integrated key rotation error handling and recovery system");
        
        // Start health monitoring if enabled
        if self.config.enable_health_monitoring {
            let _health_monitor_task = self.health_monitor.start_monitoring().await;
            info!("Health monitoring started");
        }
        
        // Start background tasks
        let _error_monitoring_task = self.start_error_monitoring().await;
        let _recovery_monitoring_task = self.start_recovery_monitoring().await;
        let _escalation_monitoring_task = self.start_escalation_monitoring().await;
        
        info!("Integrated key rotation system started successfully");
        Ok(())
    }

    /// Handle a key rotation error comprehensively
    pub async fn handle_rotation_error(
        &self,
        error: KeyRotationError,
        context: Option<RotationContext>,
    ) -> Result<ComprehensiveErrorResult, KeyRotationError> {
        let start_time = std::time::Instant::now();
        let operation_start = Utc::now();
        
        info!("Handling rotation error: {} - {}", error.code, error.message);
        
        // Update system state
        {
            let mut state = self.system_state.write().await;
            state.active_error_operations += 1;
            state.last_update = operation_start;
        }
        
        let mut actions_taken = Vec::new();
        let mut recommendations = Vec::new();
        
        // Step 1: Classify and handle error
        let error_context = {
            let mut error_handler = self.error_handler.lock().await;
            error_handler.handle_error(error.clone(), context.clone()).await
        };
        
        actions_taken.push("Error classified and context created".to_string());
        
        // Step 2: Determine recovery strategy
        let mut recovery_plan = None;
        let mut rollback_plan = None;
        
        match &error_context.recovery_strategy {
            RecoveryStrategy::Retry { .. } => {
                if self.config.enable_auto_recovery {
                    // Create automatic recovery plan
                    if let Some(ref ctx) = context {
                        let scope = RecoveryScope::SingleOperation { 
                            operation_id: ctx.correlation_id 
                        };
                        
                        match self.recovery_manager.create_recovery_plan(
                            scope,
                            RecoveryOperationType::Automatic,
                        ).await {
                            Ok(plan) => {
                                recovery_plan = Some(plan.clone());
                                actions_taken.push("Automatic recovery plan created".to_string());
                                
                                // Execute recovery plan if auto-recovery is enabled
                                match self.recovery_manager.execute_recovery_plan(&plan.recovery_id).await {
                                    Ok(result) => {
                                        if result.success {
                                            actions_taken.push("Automatic recovery completed successfully".to_string());
                                        } else {
                                            actions_taken.push("Automatic recovery partially failed".to_string());
                                            recommendations.push("Manual intervention may be required".to_string());
                                        }
                                    }
                                    Err(e) => {
                                        actions_taken.push(format!("Automatic recovery failed: {}", e.message));
                                        recommendations.push("Consider manual recovery or rollback".to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                actions_taken.push(format!("Failed to create recovery plan: {}", e.message));
                                recommendations.push("Manual recovery assessment required".to_string());
                            }
                        }
                    }
                }
            }
            
            RecoveryStrategy::Rollback => {
                if self.config.enable_auto_rollback {
                    // Create rollback plan
                    if let Some(ref ctx) = context {
                        match self.db_ops.create_rollback_plan(
                            &ctx.correlation_id,
                            RollbackReason::OperationFailure,
                        ).await {
                            Ok(plan) => {
                                rollback_plan = Some(plan.clone());
                                actions_taken.push("Rollback plan created".to_string());
                                
                                // Execute rollback if auto-rollback is enabled
                                match self.db_ops.execute_rollback(
                                    &plan.plan_id,
                                    Some("automatic_system".to_string()),
                                    self.audit_logger.as_ref().map(|l| l.as_ref()),
                                ).await {
                                    Ok(result) => {
                                        if result.success {
                                            actions_taken.push("Automatic rollback completed successfully".to_string());
                                        } else {
                                            actions_taken.push("Automatic rollback partially failed".to_string());
                                            recommendations.push("Manual rollback verification required".to_string());
                                        }
                                    }
                                    Err(e) => {
                                        actions_taken.push(format!("Automatic rollback failed: {}", e.message));
                                        recommendations.push("Manual rollback required".to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                actions_taken.push(format!("Failed to create rollback plan: {}", e.message));
                                recommendations.push("Manual rollback assessment required".to_string());
                            }
                        }
                    }
                }
            }
            
            RecoveryStrategy::Manual => {
                actions_taken.push("Manual intervention required".to_string());
                recommendations.push("Contact system administrator for manual resolution".to_string());
                recommendations.push("Review error context and system logs".to_string());
            }
            
            RecoveryStrategy::FailFast => {
                actions_taken.push("Error marked as non-recoverable".to_string());
                recommendations.push("Review error cause and prevent similar failures".to_string());
            }
            
            RecoveryStrategy::CircuitBreaker { .. } => {
                actions_taken.push("Circuit breaker activated".to_string());
                recommendations.push("Wait for circuit breaker timeout or manual reset".to_string());
                recommendations.push("Investigate underlying cause of failures".to_string());
            }
        }
        
        // Step 3: Check if escalation is needed
        let should_escalate = self.should_escalate_error(&error_context).await?;
        
        if should_escalate {
            let escalation = self.create_escalation(&error_context, &actions_taken).await?;
            actions_taken.push(format!("System escalation created: {}", escalation.escalation_id));
            recommendations.push("Escalation triggered - review system status and take appropriate action".to_string());
        }
        
        // Step 4: Update system metrics
        self.update_error_metrics(&error_context).await?;
        
        let duration = start_time.elapsed();
        let success = error_context.is_recoverable && 
                     (recovery_plan.is_some() || rollback_plan.is_some() || 
                      matches!(error_context.recovery_strategy, RecoveryStrategy::FailFast));
        
        // Update system state
        {
            let mut state = self.system_state.write().await;
            state.active_error_operations = state.active_error_operations.saturating_sub(1);
            state.performance_metrics.insert(
                "last_error_handling_duration_ms".to_string(),
                duration.as_millis() as f64,
            );
            state.last_update = Utc::now();
        }
        
        let result = ComprehensiveErrorResult {
            success,
            error_context,
            recovery_plan,
            rollback_plan,
            actions_taken,
            recommendations,
            duration,
        };
        
        info!(
            "Error handling completed: {} ({:?}) - Success: {}",
            error.code, duration, success
        );
        
        Ok(result)
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> IntegratedSystemState {
        let state = self.system_state.read().await;
        state.clone()
    }

    /// Get comprehensive health report
    pub async fn get_comprehensive_health_report(&self) -> Result<ComprehensiveHealthReport, KeyRotationError> {
        let health_metrics = self.health_monitor.get_current_metrics().await;
        let active_alerts = self.health_monitor.get_active_alerts().await;
        let recovery_state = self.recovery_manager.get_system_state().await;
        let system_state = self.get_system_status().await;
        
        let error_statistics = {
            let error_handler = self.error_handler.lock().await;
            error_handler.get_error_statistics().clone()
        };
        
        Ok(ComprehensiveHealthReport {
            report_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            overall_status: self.determine_overall_system_status(&health_metrics, &recovery_state, &system_state).await,
            health_metrics,
            active_alerts,
            recovery_state,
            error_statistics,
            system_state,
            recommendations: self.generate_system_recommendations(&health_metrics, &recovery_state).await,
        })
    }

    /// Perform emergency shutdown
    pub async fn emergency_shutdown(&self, reason: &str) -> Result<(), KeyRotationError> {
        warn!("Initiating emergency shutdown: {}", reason);
        
        // Update system status
        {
            let mut state = self.system_state.write().await;
            state.overall_status = SystemStatus::Emergency;
            state.last_update = Utc::now();
        }
        
        // Create emergency escalation
        let escalation = SystemEscalation {
            escalation_id: Uuid::new_v4(),
            escalation_type: EscalationType::Manual,
            reason: format!("Emergency shutdown: {}", reason),
            escalated_at: Utc::now(),
            status: EscalationStatus::Active,
            actions_taken: vec!["Emergency shutdown initiated".to_string()],
        };
        
        {
            let mut state = self.system_state.write().await;
            state.recent_escalations.push(escalation);
        }
        
        // Log emergency shutdown
        if let Some(ref logger) = self.audit_logger {
            logger.log_key_operation(
                "emergency_shutdown",
                "system_management",
                Duration::from_millis(0),
                crate::crypto::audit_logger::OperationResult::Success,
                None,
            ).await;
        }
        
        error!("Emergency shutdown completed: {}", reason);
        Ok(())
    }

    // Helper methods for background monitoring and system management
    
    async fn start_error_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let error_handler = Arc::clone(&self.error_handler);
        let system_state = Arc::clone(&self.system_state);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Monitor error rates and circuit breaker states
                let error_handler = error_handler.lock().await;
                let circuit_states = error_handler.get_circuit_breaker_states();
                let error_stats = error_handler.get_error_statistics();
                
                // Check for high error rates or open circuits
                let open_circuits = circuit_states.values()
                    .filter(|s| s.state == crate::crypto::key_rotation_error_handling::CircuitState::Open)
                    .count();
                
                if open_circuits > 0 {
                    debug!("Monitoring detected {} open circuit breakers", open_circuits);
                }
                
                // Update system state
                {
                    let mut state = system_state.write().await;
                    state.performance_metrics.insert(
                        "open_circuit_breakers".to_string(),
                        open_circuits as f64,
                    );
                    state.performance_metrics.insert(
                        "total_error_categories".to_string(),
                        error_stats.len() as f64,
                    );
                }
            }
        })
    }

    async fn start_recovery_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let recovery_manager = Arc::clone(&self.recovery_manager);
        let system_state = Arc::clone(&self.system_state);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Monitor recovery operations
                let active_recoveries = recovery_manager.get_active_recoveries().await;
                let recovery_state = recovery_manager.get_system_state().await;
                
                // Update system state
                {
                    let mut state = system_state.write().await;
                    state.active_recovery_operations = active_recoveries.len();
                    state.performance_metrics.insert(
                        "recovery_success_rate".to_string(),
                        recovery_state.recovery_success_rate,
                    );
                    state.performance_metrics.insert(
                        "failed_operations_detected".to_string(),
                        recovery_state.failed_operations_detected as f64,
                    );
                }
            }
        })
    }

    async fn start_escalation_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let system_state = Arc::clone(&self.system_state);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.escalation_thresholds.evaluation_window_secs));
            
            loop {
                interval.tick().await;
                
                // Monitor escalation conditions
                // This would check thresholds and create escalations as needed
                // Implementation would depend on specific escalation logic
                
                debug!("Escalation monitoring cycle completed");
            }
        })
    }

    async fn should_escalate_error(&self, error_context: &ErrorContext) -> Result<bool, KeyRotationError> {
        // Determine if error should trigger escalation
        match error_context.severity {
            crate::crypto::key_rotation_error_handling::ErrorSeverity::Critical => Ok(true),
            crate::crypto::key_rotation_error_handling::ErrorSeverity::High => {
                // Check recent error frequency
                let error_handler = self.error_handler.lock().await;
                let error_stats = error_handler.get_error_statistics();
                
                let recent_high_errors = error_stats.get(&error_context.category)
                    .map(|stats| stats.total_count)
                    .unwrap_or(0);
                
                Ok(recent_high_errors >= self.config.escalation_thresholds.critical_error_threshold as u64)
            }
            _ => Ok(false),
        }
    }

    async fn create_escalation(
        &self,
        error_context: &ErrorContext,
        actions_taken: &[String],
    ) -> Result<SystemEscalation, KeyRotationError> {
        let escalation = SystemEscalation {
            escalation_id: Uuid::new_v4(),
            escalation_type: match error_context.severity {
                crate::crypto::key_rotation_error_handling::ErrorSeverity::Critical => EscalationType::CriticalFailures,
                _ => EscalationType::HighErrorRate,
            },
            reason: format!("Error escalation: {} - {}", error_context.original_error.code, error_context.original_error.message),
            escalated_at: Utc::now(),
            status: EscalationStatus::Active,
            actions_taken: actions_taken.to_vec(),
        };
        
        // Store escalation in system state
        {
            let mut state = self.system_state.write().await;
            state.recent_escalations.push(escalation.clone());
            
            // Keep only recent escalations
            if state.recent_escalations.len() > 50 {
                state.recent_escalations.drain(0..state.recent_escalations.len() - 50);
            }
        }
        
        Ok(escalation)
    }

    async fn update_error_metrics(&self, _error_context: &ErrorContext) -> Result<(), KeyRotationError> {
        // Update error-related metrics
        let mut state = self.system_state.write().await;
        state.last_update = Utc::now();
        
        // Update performance metrics
        let total_errors = state.performance_metrics.get("total_errors").unwrap_or(&0.0) + 1.0;
        state.performance_metrics.insert("total_errors".to_string(), total_errors);
        
        Ok(())
    }

    async fn determine_overall_system_status(
        &self,
        health_metrics: &HealthMetrics,
        recovery_state: &crate::crypto::key_rotation_recovery::RecoverySystemState,
        system_state: &IntegratedSystemState,
    ) -> SystemStatus {
        // Determine overall status based on multiple factors
        if system_state.overall_status == SystemStatus::Emergency {
            return SystemStatus::Emergency;
        }
        
        match health_metrics.overall_status {
            RotationHealthStatus::Critical => SystemStatus::Critical,
            RotationHealthStatus::Unavailable => SystemStatus::Unavailable,
            RotationHealthStatus::Degraded => SystemStatus::Degraded,
            RotationHealthStatus::Warning => {
                if matches!(recovery_state.health_status, crate::crypto::key_rotation_recovery::RecoveryHealthStatus::Critical) {
                    SystemStatus::Critical
                } else {
                    SystemStatus::Degraded
                }
            }
            RotationHealthStatus::Healthy => {
                if system_state.active_error_operations > 10 || system_state.active_recovery_operations > 5 {
                    SystemStatus::Degraded
                } else {
                    SystemStatus::Operational
                }
            }
        }
    }

    async fn generate_system_recommendations(
        &self,
        health_metrics: &HealthMetrics,
        recovery_state: &crate::crypto::key_rotation_recovery::RecoverySystemState,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if health_metrics.success_rate < 95.0 {
            recommendations.push("Success rate is below optimal threshold - investigate error patterns".to_string());
        }
        
        if health_metrics.queue_length > 100 {
            recommendations.push("Queue length is high - consider scaling resources".to_string());
        }
        
        if recovery_state.failed_operations_detected > 10 {
            recommendations.push("Multiple failed operations detected - review system health".to_string());
        }
        
        if recovery_state.recovery_success_rate < 90.0 {
            recommendations.push("Recovery success rate is low - review recovery procedures".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("System is operating within normal parameters".to_string());
        }
        
        recommendations
    }
}

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveHealthReport {
    /// Report identifier
    pub report_id: Uuid,
    /// When report was generated
    pub timestamp: DateTime<Utc>,
    /// Overall system status
    pub overall_status: SystemStatus,
    /// Health metrics
    pub health_metrics: HealthMetrics,
    /// Active alerts
    pub active_alerts: Vec<HealthAlert>,
    /// Recovery system state
    pub recovery_state: crate::crypto::key_rotation_recovery::RecoverySystemState,
    /// Error statistics
    pub error_statistics: HashMap<crate::crypto::key_rotation_error_handling::ErrorCategory, crate::crypto::key_rotation_error_handling::ErrorStatistics>,
    /// System state
    pub system_state: IntegratedSystemState,
    /// System recommendations
    pub recommendations: Vec<String>,
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
    async fn test_integrated_manager_creation() {
        let db_ops = create_test_db_ops();
        let manager = IntegratedKeyRotationManager::new(db_ops, None);
        
        let status = manager.get_system_status().await;
        assert_eq!(status.overall_status, SystemStatus::Operational);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let db_ops = create_test_db_ops();
        let manager = IntegratedKeyRotationManager::new(db_ops, None);
        
        let error = KeyRotationError::new("TEST_ERROR", "Test error for integration testing");
        let result = manager.handle_rotation_error(error, None).await.unwrap();
        
        assert!(!result.actions_taken.is_empty());
        assert!(!result.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_system_status_determination() {
        let db_ops = create_test_db_ops();
        let manager = IntegratedKeyRotationManager::new(db_ops, None);
        
        let health_report = manager.get_comprehensive_health_report().await.unwrap();
        assert!(matches!(health_report.overall_status, SystemStatus::Operational));
        assert!(!health_report.recommendations.is_empty());
    }
}