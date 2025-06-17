//! Types for the Verification Event Bus
//!
//! This module contains statistics and result types for the verification event bus.

use super::handlers::EventHandlerResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

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

impl EventBusStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
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
        }
    }

    /// Clear all statistics while preserving handler count
    pub fn clear(&mut self) {
        let active_handlers = self.active_handlers;
        *self = Self::new();
        self.active_handlers = active_handlers;
    }
}

impl Default for EventBusStatistics {
    fn default() -> Self {
        Self::new()
    }
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

impl EventProcessingResult {
    /// Create a new successful processing result
    pub fn success(event_id: Uuid, duration: Duration, handler_results: Vec<EventHandlerResult>) -> Self {
        Self {
            event_id,
            success: true,
            duration,
            handler_results,
            error: None,
        }
    }

    /// Create a new failed processing result
    pub fn failure(event_id: Uuid, duration: Duration, error: String) -> Self {
        Self {
            event_id,
            success: false,
            duration,
            handler_results: Vec::new(),
            error: Some(error),
        }
    }

    /// Create a failed result with handler results
    pub fn failure_with_handlers(
        event_id: Uuid,
        duration: Duration,
        handler_results: Vec<EventHandlerResult>,
        error: String,
    ) -> Self {
        Self {
            event_id,
            success: false,
            duration,
            handler_results,
            error: Some(error),
        }
    }
}