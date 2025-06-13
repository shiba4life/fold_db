//! Comprehensive error handling and classification for key rotation operations
//!
//! This module provides advanced error handling capabilities including:
//! - Error classification and recovery strategies
//! - Retry policies with exponential backoff
//! - Circuit breaker patterns
//! - Error aggregation and reporting

use crate::crypto::key_rotation::{KeyRotationError, RotationReason, RotationContext};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use log::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

/// Error severity levels for prioritization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low impact errors that can be retried automatically
    Low,
    /// Medium impact errors requiring monitoring
    Medium,
    /// High impact errors requiring immediate attention
    High,
    /// Critical errors requiring emergency response
    Critical,
}

/// Error categories for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network-related errors (timeouts, connectivity)
    Network,
    /// Database transaction failures
    Database,
    /// Validation and business logic errors
    Validation,
    /// System resource errors (memory, disk)
    Resource,
    /// Authentication and authorization errors
    Security,
    /// Configuration and policy errors
    Configuration,
    /// External service dependencies
    External,
    /// Unknown or unclassified errors
    Unknown,
}

/// Recovery strategy for different error types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry with exponential backoff
    Retry {
        max_attempts: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
        backoff_multiplier: f64,
    },
    /// Rollback to previous state
    Rollback,
    /// Manual intervention required
    Manual,
    /// Fail immediately, no recovery
    FailFast,
    /// Circuit breaker - temporarily stop operations
    CircuitBreaker {
        failure_threshold: u32,
        timeout_duration_ms: u64,
    },
}

/// Error context with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique error identifier
    pub error_id: Uuid,
    /// Original error
    pub original_error: KeyRotationError,
    /// Error category
    pub category: ErrorCategory,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Recovery strategy
    pub recovery_strategy: RecoveryStrategy,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Operation context
    pub operation_context: Option<RotationContext>,
    /// Additional error metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Related error IDs for error chains
    pub related_errors: Vec<Uuid>,
    /// Retry attempt number (if applicable)
    pub retry_attempt: Option<u32>,
    /// Whether error is recoverable
    pub is_recoverable: bool,
}

/// Circuit breaker state for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    /// Current state
    pub state: CircuitState,
    /// Failure count in current window
    pub failure_count: u32,
    /// Success count in current window
    pub success_count: u32,
    /// Last state change timestamp
    pub last_state_change: DateTime<Utc>,
    /// Next retry timestamp (for Half-Open state)
    pub next_retry: Option<DateTime<Utc>>,
    /// Configuration
    pub config: CircuitBreakerConfig,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Allowing limited requests to test recovery
    HalfOpen,
    /// Blocking all requests
    Open,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Timeout before moving to half-open
    pub timeout_duration: ChronoDuration,
    /// Window size for counting failures
    pub window_duration: ChronoDuration,
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor to prevent thundering herd
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Error handling engine
pub struct KeyRotationErrorHandler {
    /// Circuit breaker states by operation type
    circuit_breakers: HashMap<String, CircuitBreakerState>,
    /// Error classification rules
    classification_rules: HashMap<String, (ErrorCategory, ErrorSeverity, RecoveryStrategy)>,
    /// Active error contexts
    active_errors: HashMap<Uuid, ErrorContext>,
    /// Error statistics
    error_stats: HashMap<ErrorCategory, ErrorStatistics>,
}

/// Error statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Total error count
    pub total_count: u64,
    /// Count by severity
    pub severity_counts: HashMap<ErrorSeverity, u64>,
    /// Count by recovery strategy
    pub recovery_counts: HashMap<String, u64>,
    /// Last occurrence
    pub last_occurrence: Option<DateTime<Utc>>,
    /// Average resolution time
    pub avg_resolution_time: Option<Duration>,
}

impl KeyRotationErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        let mut handler = Self {
            circuit_breakers: HashMap::new(),
            classification_rules: HashMap::new(),
            active_errors: HashMap::new(),
            error_stats: HashMap::new(),
        };
        
        handler.initialize_default_rules();
        handler
    }

    /// Initialize default error classification rules
    fn initialize_default_rules(&mut self) {
        // Network errors
        self.add_classification_rule(
            "TIMEOUT",
            ErrorCategory::Network,
            ErrorSeverity::Medium,
            RecoveryStrategy::Retry {
                max_attempts: 3,
                base_delay_ms: 1000,
                max_delay_ms: 30000,
                backoff_multiplier: 2.0,
            },
        );

        self.add_classification_rule(
            "CONNECTION_FAILED",
            ErrorCategory::Network,
            ErrorSeverity::High,
            RecoveryStrategy::CircuitBreaker {
                failure_threshold: 5,
                timeout_duration_ms: 60000,
            },
        );

        // Database errors
        self.add_classification_rule(
            "TRANSACTION_FAILED",
            ErrorCategory::Database,
            ErrorSeverity::High,
            RecoveryStrategy::Rollback,
        );

        self.add_classification_rule(
            "DEADLOCK_DETECTED",
            ErrorCategory::Database,
            ErrorSeverity::Medium,
            RecoveryStrategy::Retry {
                max_attempts: 5,
                base_delay_ms: 500,
                max_delay_ms: 10000,
                backoff_multiplier: 1.5,
            },
        );

        // Validation errors
        self.add_classification_rule(
            "INVALID_ROTATION_REQUEST",
            ErrorCategory::Validation,
            ErrorSeverity::Low,
            RecoveryStrategy::FailFast,
        );

        // Security errors
        self.add_classification_rule(
            "AUTHORIZATION_FAILED",
            ErrorCategory::Security,
            ErrorSeverity::High,
            RecoveryStrategy::FailFast,
        );

        // Resource errors
        self.add_classification_rule(
            "STORAGE_FULL",
            ErrorCategory::Resource,
            ErrorSeverity::Critical,
            RecoveryStrategy::Manual,
        );
    }

    /// Add error classification rule
    pub fn add_classification_rule(
        &mut self,
        error_code: &str,
        category: ErrorCategory,
        severity: ErrorSeverity,
        strategy: RecoveryStrategy,
    ) {
        self.classification_rules.insert(
            error_code.to_string(),
            (category, severity, strategy),
        );
    }

    /// Handle an error and return recovery instructions
    pub async fn handle_error(
        &mut self,
        error: KeyRotationError,
        context: Option<RotationContext>,
    ) -> ErrorContext {
        let error_id = Uuid::new_v4();
        
        // Classify error
        let (category, severity, recovery_strategy) = self.classify_error(&error);
        
        // Check circuit breaker
        let operation_type = context.as_ref()
            .map(|c| format!("{:?}", c.request.reason))
            .unwrap_or_else(|| "unknown".to_string());
            
        let circuit_breaker_state = self.check_circuit_breaker(&operation_type);
        
        // Create error context
        let mut error_context = ErrorContext {
            error_id,
            original_error: error.clone(),
            category: category.clone(),
            severity: severity.clone(),
            recovery_strategy: recovery_strategy.clone(),
            timestamp: Utc::now(),
            operation_context: context,
            metadata: HashMap::new(),
            related_errors: Vec::new(),
            retry_attempt: None,
            is_recoverable: self.is_recoverable(&recovery_strategy, &circuit_breaker_state),
        };

        // Add circuit breaker metadata
        if let Some(state) = circuit_breaker_state {
            error_context.metadata.insert(
                "circuit_breaker_state".to_string(),
                serde_json::json!(state),
            );
        }

        // Update statistics
        self.update_error_statistics(&category, &severity);
        
        // Store active error
        self.active_errors.insert(error_id, error_context.clone());
        
        info!(
            "Handled error {}: {} ({:?}, {:?})",
            error_id, error.message, category, severity
        );
        
        error_context
    }

    /// Classify an error based on its code and message
    fn classify_error(&self, error: &KeyRotationError) -> (ErrorCategory, ErrorSeverity, RecoveryStrategy) {
        // Check exact code match first
        if let Some((category, severity, strategy)) = self.classification_rules.get(&error.code) {
            return (category.clone(), severity.clone(), strategy.clone());
        }
        
        // Pattern matching for common error types
        let code_lower = error.code.to_lowercase();
        let message_lower = error.message.to_lowercase();
        
        if code_lower.contains("timeout") || message_lower.contains("timeout") {
            return (
                ErrorCategory::Network,
                ErrorSeverity::Medium,
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    base_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                },
            );
        }
        
        if code_lower.contains("deadlock") || message_lower.contains("deadlock") {
            return (
                ErrorCategory::Database,
                ErrorSeverity::Medium,
                RecoveryStrategy::Retry {
                    max_attempts: 5,
                    base_delay_ms: 500,
                    max_delay_ms: 10000,
                    backoff_multiplier: 1.5,
                },
            );
        }
        
        if code_lower.contains("validation") || code_lower.contains("invalid") {
            return (
                ErrorCategory::Validation,
                ErrorSeverity::Low,
                RecoveryStrategy::FailFast,
            );
        }
        
        if code_lower.contains("auth") || code_lower.contains("permission") {
            return (
                ErrorCategory::Security,
                ErrorSeverity::High,
                RecoveryStrategy::FailFast,
            );
        }
        
        if code_lower.contains("storage") || code_lower.contains("disk") {
            return (
                ErrorCategory::Resource,
                ErrorSeverity::High,
                RecoveryStrategy::Manual,
            );
        }
        
        // Default classification
        (
            ErrorCategory::Unknown,
            ErrorSeverity::Medium,
            RecoveryStrategy::Manual,
        )
    }

    /// Check circuit breaker state for operation type
    fn check_circuit_breaker(&mut self, operation_type: &str) -> Option<CircuitBreakerState> {
        if let Some(state) = self.circuit_breakers.get_mut(operation_type) {
            self.update_circuit_breaker_state(state);
            Some(state.clone())
        } else {
            None
        }
    }

    /// Update circuit breaker state based on time
    fn update_circuit_breaker_state(&mut self, state: &mut CircuitBreakerState) {
        let now = Utc::now();
        
        match state.state {
            CircuitState::Open => {
                if let Some(next_retry) = state.next_retry {
                    if now >= next_retry {
                        state.state = CircuitState::HalfOpen;
                        state.last_state_change = now;
                        state.next_retry = None;
                        info!("Circuit breaker moved to Half-Open state");
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
            }
            CircuitState::Closed => {
                // Reset counters if window has expired
                if now - state.last_state_change > state.config.window_duration {
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_state_change = now;
                }
            }
        }
    }

    /// Record operation result for circuit breaker
    pub fn record_operation_result(&mut self, operation_type: &str, success: bool) {
        let state = self.circuit_breakers.entry(operation_type.to_string())
            .or_insert_with(|| CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_state_change: Utc::now(),
                next_retry: None,
                config: CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    timeout_duration: ChronoDuration::minutes(1),
                    window_duration: ChronoDuration::minutes(5),
                },
            });

        if success {
            state.success_count += 1;
            
            match state.state {
                CircuitState::HalfOpen => {
                    if state.success_count >= state.config.success_threshold {
                        state.state = CircuitState::Closed;
                        state.failure_count = 0;
                        state.success_count = 0;
                        state.last_state_change = Utc::now();
                        info!("Circuit breaker closed for operation: {}", operation_type);
                    }
                }
                _ => {}
            }
        } else {
            state.failure_count += 1;
            
            match state.state {
                CircuitState::Closed | CircuitState::HalfOpen => {
                    if state.failure_count >= state.config.failure_threshold {
                        state.state = CircuitState::Open;
                        state.last_state_change = Utc::now();
                        state.next_retry = Some(Utc::now() + state.config.timeout_duration);
                        warn!("Circuit breaker opened for operation: {}", operation_type);
                    }
                }
                _ => {}
            }
        }
    }

    /// Check if error is recoverable
    fn is_recoverable(&self, strategy: &RecoveryStrategy, circuit_state: &Option<CircuitBreakerState>) -> bool {
        if let Some(state) = circuit_state {
            if state.state == CircuitState::Open {
                return false;
            }
        }
        
        matches!(strategy, 
            RecoveryStrategy::Retry { .. } | 
            RecoveryStrategy::Rollback |
            RecoveryStrategy::CircuitBreaker { .. }
        )
    }

    /// Calculate retry delay with exponential backoff and jitter
    pub fn calculate_retry_delay(&self, config: &RetryConfig, attempt: u32) -> Duration {
        if attempt == 0 {
            return config.base_delay;
        }
        
        let exponential_delay = config.base_delay.as_millis() as f64 * 
            config.backoff_multiplier.powi(attempt as i32 - 1);
        
        let capped_delay = exponential_delay.min(config.max_delay.as_millis() as f64);
        
        // Add jitter to prevent thundering herd
        let jitter = capped_delay * config.jitter_factor * (rand::random::<f64>() - 0.5);
        let final_delay = (capped_delay + jitter).max(0.0);
        
        Duration::from_millis(final_delay as u64)
    }

    /// Update error statistics
    fn update_error_statistics(&mut self, category: &ErrorCategory, severity: &ErrorSeverity) {
        let stats = self.error_stats.entry(category.clone())
            .or_insert_with(|| ErrorStatistics {
                total_count: 0,
                severity_counts: HashMap::new(),
                recovery_counts: HashMap::new(),
                last_occurrence: None,
                avg_resolution_time: None,
            });
        
        stats.total_count += 1;
        *stats.severity_counts.entry(severity.clone()).or_insert(0) += 1;
        stats.last_occurrence = Some(Utc::now());
    }

    /// Mark error as resolved
    pub fn mark_error_resolved(&mut self, error_id: &Uuid, resolution_time: Duration) {
        if let Some(error_context) = self.active_errors.remove(error_id) {
            // Update statistics
            if let Some(stats) = self.error_stats.get_mut(&error_context.category) {
                // Update average resolution time
                stats.avg_resolution_time = Some(
                    stats.avg_resolution_time
                        .map(|avg| Duration::from_millis(
                            (avg.as_millis() as u64 + resolution_time.as_millis() as u64) / 2
                        ))
                        .unwrap_or(resolution_time)
                );
            }
            
            info!("Error {} resolved in {:?}", error_id, resolution_time);
        }
    }

    /// Get error statistics
    pub fn get_error_statistics(&self) -> &HashMap<ErrorCategory, ErrorStatistics> {
        &self.error_stats
    }

    /// Get active errors
    pub fn get_active_errors(&self) -> &HashMap<Uuid, ErrorContext> {
        &self.active_errors
    }

    /// Get circuit breaker states
    pub fn get_circuit_breaker_states(&self) -> &HashMap<String, CircuitBreakerState> {
        &self.circuit_breakers
    }

    /// Check if operation should be allowed based on circuit breaker
    pub fn should_allow_operation(&mut self, operation_type: &str) -> bool {
        if let Some(state) = self.check_circuit_breaker(operation_type) {
            match state.state {
                CircuitState::Open => false,
                CircuitState::HalfOpen => {
                    // Allow limited requests in half-open state
                    state.success_count < state.config.success_threshold
                }
                CircuitState::Closed => true,
            }
        } else {
            true
        }
    }
}

impl Default for KeyRotationErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "Low"),
            ErrorSeverity::Medium => write!(f, "Medium"),
            ErrorSeverity::High => write!(f, "High"),
            ErrorSeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Database => write!(f, "Database"),
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::Security => write!(f, "Security"),
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::External => write!(f, "External"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_rotation::KeyRotationRequest;

    #[test]
    fn test_error_classification() {
        let mut handler = KeyRotationErrorHandler::new();
        
        // Test timeout error
        let timeout_error = KeyRotationError::new("TIMEOUT", "Operation timed out");
        let (category, severity, _) = handler.classify_error(&timeout_error);
        assert_eq!(category, ErrorCategory::Network);
        assert_eq!(severity, ErrorSeverity::Medium);
        
        // Test validation error
        let validation_error = KeyRotationError::new("INVALID_REQUEST", "Invalid rotation request");
        let (category, severity, _) = handler.classify_error(&validation_error);
        assert_eq!(category, ErrorCategory::Validation);
        assert_eq!(severity, ErrorSeverity::Low);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let handler = KeyRotationErrorHandler::new();
        let config = RetryConfig::default();
        
        let delay1 = handler.calculate_retry_delay(&config, 1);
        let delay2 = handler.calculate_retry_delay(&config, 2);
        let delay3 = handler.calculate_retry_delay(&config, 3);
        
        assert!(delay2 > delay1);
        assert!(delay3 > delay2);
        assert!(delay3 <= config.max_delay);
    }

    #[test]
    fn test_circuit_breaker_transitions() {
        let mut handler = KeyRotationErrorHandler::new();
        let operation_type = "test_operation";
        
        // Record failures to trip circuit breaker
        for _ in 0..5 {
            handler.record_operation_result(operation_type, false);
        }
        
        // Circuit should now be open
        assert!(!handler.should_allow_operation(operation_type));
        
        // Record successes in half-open state
        if let Some(state) = handler.circuit_breakers.get_mut(operation_type) {
            state.state = CircuitState::HalfOpen;
        }
        
        for _ in 0..3 {
            handler.record_operation_result(operation_type, true);
        }
        
        // Circuit should now be closed
        assert!(handler.should_allow_operation(operation_type));
    }
}