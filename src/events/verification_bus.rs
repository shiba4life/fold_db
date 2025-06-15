//! Core Verification Event Bus Implementation
//!
//! This module implements the centralized event bus for security operations monitoring.
//! It provides async event processing, pluggable handlers, and cross-platform support.

use super::correlation::CorrelationManager;
use super::event_types::SecurityEvent;
use crate::security_types::Severity;
use super::handlers::{EventHandler, EventHandlerResult};
use crate::config::unified_config::{EnvironmentConfig, UnifiedConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{interval, timeout};
use uuid::Uuid;

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

/// Statistics about event bus operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusStatistics {
    /// Total events processed
    pub total_events: u64,
    /// Events by severity level
    pub events_by_severity: HashMap<String, u64>,
    /// Events by category
    pub events_by_category: HashMap<String, u64>,
    /// Events by platform
    pub events_by_platform: HashMap<String, u64>,
    /// Number of active handlers
    pub active_handlers: usize,
    /// Handler success rate
    pub handler_success_rate: f64,
    /// Average processing time
    pub avg_processing_time_ms: f64,
    /// Events dropped due to buffer overflow
    pub dropped_events: u64,
    /// Last event timestamp
    pub last_event_time: Option<DateTime<Utc>>,
    /// Bus uptime in seconds
    pub uptime_seconds: u64,
    /// Correlation statistics
    pub correlations_created: u64,
}

/// Event processing result
#[derive(Debug, Clone)]
pub struct EventProcessingResult {
    /// Event ID that was processed
    pub event_id: Uuid,
    /// Whether processing was successful
    pub success: bool,
    /// Processing duration
    pub duration: Duration,
    /// Handler results
    pub handler_results: Vec<EventHandlerResult>,
    /// Error message if processing failed
    pub error: Option<String>,
}

/// Main verification event bus
pub struct VerificationEventBus {
    /// Bus configuration
    config: VerificationBusConfig,
    /// Event channel for publishing/subscribing
    event_sender: broadcast::Sender<SecurityEvent>,
    /// Registered event handlers
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler + Send + Sync>>>>,
    /// Correlation manager for cross-platform events
    correlation_manager: Arc<Mutex<CorrelationManager>>,
    /// Event bus statistics
    statistics: Arc<RwLock<EventBusStatistics>>,
    /// Background processing task handle
    _processor_handle: Option<JoinHandle<()>>,
    /// Statistics update task handle
    _stats_handle: Option<JoinHandle<()>>,
    /// Bus start time for uptime calculation
    start_time: DateTime<Utc>,
}

impl VerificationEventBus {
    /// Create a new verification event bus
    pub fn new(config: VerificationBusConfig) -> Self {
        let (event_sender, _) = broadcast::channel(config.buffer_size);
        let correlation_manager = Arc::new(Mutex::new(CorrelationManager::new(
            Duration::from_secs(config.correlation_window_minutes * 60),
        )));

        let statistics = Arc::new(RwLock::new(EventBusStatistics {
            total_events: 0,
            events_by_severity: HashMap::new(),
            events_by_category: HashMap::new(),
            events_by_platform: HashMap::new(),
            active_handlers: 0,
            handler_success_rate: 1.0,
            avg_processing_time_ms: 0.0,
            dropped_events: 0,
            last_event_time: None,
            uptime_seconds: 0,
            correlations_created: 0,
        }));

        let start_time = Utc::now();

        Self {
            config,
            event_sender,
            handlers: Arc::new(RwLock::new(Vec::new())),
            correlation_manager,
            statistics,
            _processor_handle: None,
            _stats_handle: None,
            start_time,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(VerificationBusConfig::default())
    }

    /// Create from unified configuration
    pub fn from_unified_config(
        unified_config: &UnifiedConfig,
        environment: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let env_config = unified_config
            .environments
            .get(environment)
            .ok_or_else(|| format!("Environment '{}' not found in configuration", environment))?;

        let config = VerificationBusConfig::from_environment_config(env_config);
        Ok(Self::new(config))
    }

    /// Start the event bus with background processing
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("Event bus is disabled in configuration".into());
        }

        // Start background event processor
        let processor_handle = self.start_event_processor().await?;
        self._processor_handle = Some(processor_handle);

        // Start statistics updater
        let stats_handle = self.start_statistics_updater().await?;
        self._stats_handle = Some(stats_handle);

        log::info!(
            "🚀 VerificationEventBus started with {} buffer size",
            self.config.buffer_size
        );
        Ok(())
    }

    /// Register an event handler
    pub async fn register_handler(
        &self,
        handler: Box<dyn EventHandler + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.active_handlers = handlers.len();

        log::info!(
            "📝 Registered new event handler. Total handlers: {}",
            handlers.len()
        );
        Ok(())
    }

    /// Publish a security event to the bus
    pub async fn publish_event(
        &self,
        event: SecurityEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(()); // Silently ignore if disabled
        }

        // Check severity filter
        if !event.should_alert(self.config.min_severity) {
            return Ok(()); // Filter out events below minimum severity
        }

        // Update correlation if enabled
        if self.config.enable_correlation {
            let mut correlation_manager = self.correlation_manager.lock().await;
            correlation_manager.add_event(&event).await;
        }

        // Send event to channel
        match self.event_sender.send(event.clone()) {
            Ok(_) => {
                // Update statistics
                let mut stats = self.statistics.write().await;
                stats.total_events += 1;
                stats.last_event_time = Some(Utc::now());

                let severity_key = format!("{:?}", event.severity());
                *stats.events_by_severity.entry(severity_key).or_insert(0) += 1;

                let category_key = format!("{:?}", event.category());
                *stats.events_by_category.entry(category_key).or_insert(0) += 1;

                let platform_key = format!("{:?}", event.platform());
                *stats.events_by_platform.entry(platform_key).or_insert(0) += 1;

                Ok(())
            }
            Err(_) => {
                // Channel is full or has no receivers
                let mut stats = self.statistics.write().await;
                stats.dropped_events += 1;

                if self.config.graceful_degradation {
                    log::warn!(
                        "⚠️ Event bus buffer full, dropping event: {}",
                        event.base_event().event_id
                    );
                    Ok(())
                } else {
                    Err("Event bus buffer full".into())
                }
            }
        }
    }

    /// Subscribe to events (returns a receiver for external processing)
    pub fn subscribe(&self) -> broadcast::Receiver<SecurityEvent> {
        self.event_sender.subscribe()
    }

    /// Get current event bus statistics
    pub async fn get_statistics(&self) -> EventBusStatistics {
        let mut stats = self.statistics.read().await.clone();
        stats.uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;
        stats
    }

    /// Get event bus configuration (read-only)
    pub fn get_config(&self) -> &VerificationBusConfig {
        &self.config
    }

    /// Get correlation information for an event
    pub async fn get_correlations(&self, event_id: Uuid) -> Vec<SecurityEvent> {
        if self.config.enable_correlation {
            let correlation_manager = self.correlation_manager.lock().await;
            correlation_manager.get_correlated_events(event_id).await
        } else {
            Vec::new()
        }
    }

    /// Clear all statistics (useful for testing)
    pub async fn clear_statistics(&self) {
        let mut stats = self.statistics.write().await;
        *stats = EventBusStatistics {
            total_events: 0,
            events_by_severity: HashMap::new(),
            events_by_category: HashMap::new(),
            events_by_platform: HashMap::new(),
            active_handlers: stats.active_handlers, // Keep handler count
            handler_success_rate: 1.0,
            avg_processing_time_ms: 0.0,
            dropped_events: 0,
            last_event_time: None,
            uptime_seconds: 0,
            correlations_created: 0,
        };
    }

    /// Stop the event bus gracefully
    pub async fn stop(&mut self) {
        log::info!("🛑 Stopping VerificationEventBus...");

        // Cancel background tasks
        if let Some(handle) = self._processor_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self._stats_handle.take() {
            handle.abort();
        }

        log::info!("✅ VerificationEventBus stopped");
    }

    // Private methods

    /// Start the background event processor
    async fn start_event_processor(
        &self,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let mut receiver = self.event_sender.subscribe();
        let handlers = Arc::clone(&self.handlers);
        let statistics = Arc::clone(&self.statistics);
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            log::info!("🔄 Event processor started");

            while let Ok(event) = receiver.recv().await {
                let start_time = std::time::Instant::now();
                let event_id = event.base_event().event_id;

                // Process event with timeout
                let processing_result = timeout(
                    Duration::from_millis(config.processing_timeout_ms),
                    Self::process_event_with_handlers(event, &handlers, &config),
                )
                .await;

                let duration = start_time.elapsed();

                match processing_result {
                    Ok(result) => {
                        // Update statistics with processing result
                        Self::update_processing_statistics(&statistics, &result, duration).await;

                        if !result.success {
                            log::error!(
                                "❌ Event processing failed for {}: {:?}",
                                event_id,
                                result.error
                            );
                        }
                    }
                    Err(_) => {
                        log::error!("⏰ Event processing timeout for {}", event_id);

                        let failed_result = EventProcessingResult {
                            event_id,
                            success: false,
                            duration,
                            handler_results: Vec::new(),
                            error: Some("Processing timeout".to_string()),
                        };

                        Self::update_processing_statistics(&statistics, &failed_result, duration)
                            .await;
                    }
                }
            }

            log::info!("🔄 Event processor stopped");
        });

        Ok(handle)
    }

    /// Start the statistics updater
    async fn start_statistics_updater(
        &self,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let statistics = Arc::clone(&self.statistics);
        let start_time = self.start_time;

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Update every minute

            loop {
                interval.tick().await;

                let mut stats = statistics.write().await;
                stats.uptime_seconds = (Utc::now() - start_time).num_seconds() as u64;
            }
        });

        Ok(handle)
    }

    /// Process an event with all registered handlers
    async fn process_event_with_handlers(
        event: SecurityEvent,
        handlers: &Arc<RwLock<Vec<Box<dyn EventHandler + Send + Sync>>>>,
        config: &VerificationBusConfig,
    ) -> EventProcessingResult {
        let event_id = event.base_event().event_id;
        let mut handler_results = Vec::new();
        let mut overall_success = true;

        let handlers_guard = handlers.read().await;

        for handler in handlers_guard.iter() {
            let handler_timeout = Duration::from_millis(config.handler_timeout_ms);

            let result = timeout(handler_timeout, handler.handle_event(&event)).await;

            match result {
                Ok(handler_result) => {
                    let success = handler_result.success;
                    handler_results.push(handler_result);

                    if !success {
                        overall_success = false;
                        if !config.graceful_degradation {
                            break; // Stop processing on first failure if not graceful
                        }
                    }
                }
                Err(_) => {
                    // Handler timeout
                    let timeout_result = EventHandlerResult {
                        handler_name: "unknown".to_string(),
                        success: false,
                        duration: handler_timeout,
                        error: Some("Handler timeout".to_string()),
                        metadata: HashMap::new(),
                    };
                    handler_results.push(timeout_result);
                    overall_success = false;

                    if !config.graceful_degradation {
                        break;
                    }
                }
            }
        }

        EventProcessingResult {
            event_id,
            success: overall_success,
            duration: Duration::from_millis(0), // Will be set by caller
            handler_results,
            error: if overall_success {
                None
            } else {
                Some("One or more handlers failed".to_string())
            },
        }
    }

    /// Update processing statistics
    async fn update_processing_statistics(
        statistics: &Arc<RwLock<EventBusStatistics>>,
        result: &EventProcessingResult,
        duration: Duration,
    ) {
        let mut stats = statistics.write().await;

        // Update handler success rate
        let total_handlers = result.handler_results.len() as f64;
        if total_handlers > 0.0 {
            let successful_handlers =
                result.handler_results.iter().filter(|r| r.success).count() as f64;

            let current_rate = successful_handlers / total_handlers;

            // Exponential moving average
            stats.handler_success_rate = 0.9 * stats.handler_success_rate + 0.1 * current_rate;
        }

        // Update average processing time
        let current_time_ms = duration.as_millis() as f64;
        stats.avg_processing_time_ms = 0.9 * stats.avg_processing_time_ms + 0.1 * current_time_ms;
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

impl Drop for VerificationEventBus {
    fn drop(&mut self) {
        // Attempt to stop gracefully
        if let Some(handle) = self._processor_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self._stats_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::{
        CreateVerificationEvent, PlatformSource, SecurityEventCategory, VerificationEvent,
    };

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = VerificationEventBus::with_default_config();
        assert!(bus.config.enabled);
        assert_eq!(bus.config.buffer_size, 10000);
    }

    #[tokio::test]
    async fn test_event_publishing() {
        let bus = VerificationEventBus::with_default_config();

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));

        let result = bus.publish_event(event).await;
        assert!(result.is_ok());

        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let config = VerificationBusConfig {
            min_severity: Severity::Error,
            ..Default::default()
        };

        let bus = VerificationEventBus::new(config);

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        // This should be filtered out
        let info_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Performance,
            Severity::Info,
            PlatformSource::JavaScriptSdk,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));

        // This should pass through
        let error_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            Severity::Error,
            PlatformSource::PythonSdk,
            "security_component".to_string(),
            "security_operation".to_string(),
        ));

        bus.publish_event(info_event).await.unwrap();
        bus.publish_event(error_event).await.unwrap();

        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 1); // Only error event should be counted
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let bus = VerificationEventBus::with_default_config();

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        let event1 = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "auth".to_string(),
            "login".to_string(),
        ));

        let event2 = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            Severity::Critical,
            PlatformSource::DataFoldNode,
            "security".to_string(),
            "threat_detected".to_string(),
        ));

        bus.publish_event(event1).await.unwrap();
        bus.publish_event(event2).await.unwrap();

        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.events_by_severity.get("Info"), Some(&1));
        assert_eq!(stats.events_by_severity.get("Critical"), Some(&1));
        assert_eq!(stats.events_by_category.get("Authentication"), Some(&1));
        assert_eq!(stats.events_by_category.get("Security"), Some(&1));
    }
}
