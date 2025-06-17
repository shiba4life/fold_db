//! Event Processing Logic for the Verification Event Bus
//!
//! This module contains the core event processing and handler execution logic.

use super::event_types::SecurityEvent;
use super::handlers::{EventHandler, EventHandlerResult};
use super::verification_bus_config::VerificationBusConfig;
use super::verification_bus_types::{EventBusStatistics, EventProcessingResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

/// Start the background event processor
pub async fn start_event_processor(
    event_sender: &broadcast::Sender<SecurityEvent>,
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler + Send + Sync>>>>,
    statistics: Arc<RwLock<EventBusStatistics>>,
    config: VerificationBusConfig,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
    let mut receiver = event_sender.subscribe();

    let handle = tokio::spawn(async move {
        log::info!("🔄 Event processor started");

        while let Ok(event) = receiver.recv().await {
            let start_time = std::time::Instant::now();
            let event_id = event.base_event().event_id;

            // Process event with timeout
            let processing_result = timeout(
                Duration::from_millis(config.processing_timeout_ms),
                process_event_with_handlers(event, &handlers, &config),
            )
            .await;

            let duration = start_time.elapsed();

            match processing_result {
                Ok(result) => {
                    // Update statistics with processing result
                    update_processing_statistics(&statistics, &result, duration).await;

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

                    let failed_result = EventProcessingResult::failure(
                        event_id,
                        duration,
                        "Processing timeout".to_string(),
                    );

                    update_processing_statistics(&statistics, &failed_result, duration).await;
                }
            }
        }

        log::info!("🔄 Event processor stopped");
    });

    Ok(handle)
}

/// Process an event with all registered handlers
pub async fn process_event_with_handlers(
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

    if overall_success {
        EventProcessingResult::success(event_id, Duration::from_millis(0), handler_results)
    } else {
        EventProcessingResult::failure_with_handlers(
            event_id,
            Duration::from_millis(0),
            handler_results,
            "One or more handlers failed".to_string(),
        )
    }
}

/// Update processing statistics
pub async fn update_processing_statistics(
    statistics: &Arc<RwLock<EventBusStatistics>>,
    result: &EventProcessingResult,
    duration: Duration,
) {
    let mut stats = statistics.write().await;

    // Update handler success rate
    let total_handlers = result.handler_results.len() as f64;
    if total_handlers > 0.0 {
        let successful_handlers = result
            .handler_results
            .iter()
            .filter(|r| r.success)
            .count() as f64;

        let current_rate = successful_handlers / total_handlers;

        // Exponential moving average
        stats.handler_success_rate = 0.9 * stats.handler_success_rate + 0.1 * current_rate;
    }

    // Update average processing time
    let current_time_ms = duration.as_millis() as f64;
    stats.avg_processing_time_ms = 0.9 * stats.avg_processing_time_ms + 0.1 * current_time_ms;
}