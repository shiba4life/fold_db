//! Configuration for the Verification Event Bus
//!
//! This module contains configuration types and implementations for the verification event bus.

use crate::config::unified_config::EnvironmentConfig;
use crate::security_types::Severity;
use serde::{Deserialize, Serialize};

/// Configuration for the verification event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationBusConfig {
    /// Enable the event bus
    pub enabled: bool,
    /// Maximum number of events to buffer in memory
    pub buffer_size: usize,
    /// Event processing timeout in milliseconds
    pub processing_timeout_ms: u64,
    /// Maximum number of concurrent event handlers
    pub max_concurrent_handlers: usize,
    /// Enable event persistence
    pub enable_persistence: bool,
    /// Event retention period in hours
    pub retention_hours: u64,
    /// Minimum severity level for processing
    pub min_severity: Severity,
    /// Enable cross-platform correlation
    pub enable_correlation: bool,
    /// Correlation window size in minutes
    pub correlation_window_minutes: u64,
    /// Enable graceful degradation on handler failures
    pub graceful_degradation: bool,
    /// Batch size for bulk event processing
    pub batch_size: usize,
    /// Event handler timeout in milliseconds
    pub handler_timeout_ms: u64,
}

impl Default for VerificationBusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            processing_timeout_ms: 5000,
            max_concurrent_handlers: 10,
            enable_persistence: true,
            retention_hours: 24,
            min_severity: Severity::Info,
            enable_correlation: true,
            correlation_window_minutes: 60,
            graceful_degradation: true,
            batch_size: 100,
            handler_timeout_ms: 3000,
        }
    }
}

impl VerificationBusConfig {
    /// Create configuration from unified environment config
    pub fn from_environment_config(env_config: &EnvironmentConfig) -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            processing_timeout_ms: env_config.performance.default_timeout_secs * 1000,
            max_concurrent_handlers: env_config.performance.max_concurrent_signs,
            enable_persistence: true,
            retention_hours: 24,
            min_severity: match env_config.logging.level.as_str() {
                "debug" => Severity::Info,
                "info" => Severity::Info,
                "warn" => Severity::Warning,
                "error" => Severity::Error,
                _ => Severity::Info,
            },
            enable_correlation: true,
            correlation_window_minutes: 60,
            graceful_degradation: true,
            batch_size: 100,
            handler_timeout_ms: 3000,
        }
    }
}