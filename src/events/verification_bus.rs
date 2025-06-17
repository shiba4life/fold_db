//! Core Verification Event Bus Implementation
//!
//! This module implements the centralized event bus for security operations monitoring.
//! It provides async event processing, pluggable handlers, and cross-platform support.

use super::correlation::CorrelationManager;
use super::event_types::SecurityEvent;
use super::handlers::EventHandler;
use super::verification_bus_config::VerificationBusConfig;
use super::verification_bus_types::{EventBusStatistics, EventProcessingResult};
use super::verification_processing;
use super::verification_statistics;
use crate::config::unified_config::UnifiedConfig;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::JoinHandle;
use uuid::Uuid;

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

        let statistics = Arc::new(RwLock::new(EventBusStatistics::new()));
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
        let processor_handle = verification_processing::start_event_processor(
            &self.event_sender,
            Arc::clone(&self.handlers),
            Arc::clone(&self.statistics),
            self.config.clone(),
        )
        .await?;
        self._processor_handle = Some(processor_handle);

        // Start statistics updater
        let stats_handle = verification_statistics::start_statistics_updater(
            Arc::clone(&self.statistics),
            self.start_time,
        )
        .await?;
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
        verification_statistics::get_current_statistics(&self.statistics, self.start_time).await
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
        verification_statistics::clear_statistics(&self.statistics).await;
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

