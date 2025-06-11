//! Pluggable Event Handler Architecture
//!
//! This module provides the foundation for pluggable event handlers that can process
//! security events from the verification bus. Includes built-in handlers for common
//! use cases and a trait for custom handler implementation.

use super::event_types::SecurityEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Result of event handler processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandlerResult {
    /// Name of the handler that processed the event
    pub handler_name: String,
    /// Whether the handler processed the event successfully
    pub success: bool,
    /// Duration taken to process the event
    pub duration: Duration,
    /// Error message if processing failed
    pub error: Option<String>,
    /// Additional metadata from the handler
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trait for implementing custom event handlers
#[async_trait]
pub trait EventHandler {
    /// Handle a security event and return the result
    async fn handle_event(&self, event: &SecurityEvent) -> EventHandlerResult;
    
    /// Get the name of this handler
    fn handler_name(&self) -> String;
    
    /// Check if this handler can process the given event type
    fn can_handle(&self, event: &SecurityEvent) -> bool {
        // Default implementation accepts all events
        let _ = event;
        true
    }
    
    /// Initialize the handler (called once when registered)
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Cleanup handler resources (called when shutting down)
    async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Built-in audit logging handler
pub struct AuditLogHandler {
    /// Name of this handler
    name: String,
    /// File path for audit logs
    log_path: String,
    /// Whether to use structured JSON logging
    structured: bool,
    /// Whether to include sensitive data in logs
    #[allow(dead_code)]
    include_sensitive: bool,
}

impl AuditLogHandler {
    /// Create a new audit log handler
    pub fn new(log_path: String) -> Self {
        Self {
            name: "audit_log_handler".to_string(),
            log_path,
            structured: true,
            include_sensitive: false,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(log_path: String, structured: bool, include_sensitive: bool) -> Self {
        Self {
            name: "audit_log_handler".to_string(),
            log_path,
            structured,
            include_sensitive,
        }
    }
    
    /// Set handler name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

#[async_trait]
impl EventHandler for AuditLogHandler {
    async fn handle_event(&self, event: &SecurityEvent) -> EventHandlerResult {
        let start_time = std::time::Instant::now();
        let base_event = event.base_event();
        
        let log_entry = if self.structured {
            // JSON structured logging
            match serde_json::to_string(&event) {
                Ok(json) => format!("{}\n", json),
                Err(e) => {
                    return EventHandlerResult {
                        handler_name: self.name.clone(),
                        success: false,
                        duration: start_time.elapsed(),
                        error: Some(format!("JSON serialization failed: {}", e)),
                        metadata: HashMap::new(),
                    };
                }
            }
        } else {
            // Human-readable logging
            format!(
                "[{}] {} {} {} {}: {} - {}\n",
                base_event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                base_event.severity,
                base_event.category,
                base_event.platform,
                base_event.component,
                base_event.operation,
                match &base_event.result {
                    crate::events::event_types::OperationResult::Success => "SUCCESS".to_string(),
                    crate::events::event_types::OperationResult::Failure { error_type, .. } => format!("FAILED: {}", error_type),
                    crate::events::event_types::OperationResult::Cancelled => "CANCELLED".to_string(),
                    crate::events::event_types::OperationResult::InProgress => "IN_PROGRESS".to_string(),
                    crate::events::event_types::OperationResult::Timeout => "TIMEOUT".to_string(),
                }
            )
        };
        
        // Write to log file
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await
        {
            Ok(mut file) => {
                match file.write_all(log_entry.as_bytes()).await {
                    Ok(_) => {
                        let mut metadata = HashMap::new();
                        metadata.insert("log_path".to_string(), serde_json::Value::String(self.log_path.clone()));
                        metadata.insert("structured".to_string(), serde_json::Value::Bool(self.structured));
                        
                        EventHandlerResult {
                            handler_name: self.name.clone(),
                            success: true,
                            duration: start_time.elapsed(),
                            error: None,
                            metadata,
                        }
                    }
                    Err(e) => EventHandlerResult {
                        handler_name: self.name.clone(),
                        success: false,
                        duration: start_time.elapsed(),
                        error: Some(format!("Failed to write to log file: {}", e)),
                        metadata: HashMap::new(),
                    }
                }
            }
            Err(e) => EventHandlerResult {
                handler_name: self.name.clone(),
                success: false,
                duration: start_time.elapsed(),
                error: Some(format!("Failed to open log file: {}", e)),
                metadata: HashMap::new(),
            }
        }
    }
    
    fn handler_name(&self) -> String {
        self.name.clone()
    }
}

/// Built-in metrics collection handler
pub struct MetricsHandler {
    /// Name of this handler
    name: String,
    /// Collected metrics
    metrics: std::sync::Arc<tokio::sync::RwLock<HashMap<String, f64>>>,
    /// Whether to track detailed metrics
    detailed: bool,
}

impl MetricsHandler {
    /// Create a new metrics handler
    pub fn new() -> Self {
        Self {
            name: "metrics_handler".to_string(),
            metrics: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            detailed: true,
        }
    }
    
    /// Create with custom name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    
    /// Set whether to collect detailed metrics
    pub fn with_detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }
    
    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> HashMap<String, f64> {
        self.metrics.read().await.clone()
    }
    
    /// Reset all metrics
    pub async fn reset_metrics(&self) {
        self.metrics.write().await.clear();
    }
}

impl Default for MetricsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler for MetricsHandler {
    async fn handle_event(&self, event: &SecurityEvent) -> EventHandlerResult {
        let start_time = std::time::Instant::now();
        let base_event = event.base_event();
        
        let mut metrics = self.metrics.write().await;
        
        // Update basic counters
        let total_key = "total_events".to_string();
        *metrics.entry(total_key).or_insert(0.0) += 1.0;
        
        let severity_key = format!("events_by_severity_{:?}", base_event.severity);
        *metrics.entry(severity_key).or_insert(0.0) += 1.0;
        
        let category_key = format!("events_by_category_{:?}", base_event.category);
        *metrics.entry(category_key).or_insert(0.0) += 1.0;
        
        let platform_key = format!("events_by_platform_{:?}", base_event.platform);
        *metrics.entry(platform_key).or_insert(0.0) += 1.0;
        
        // Update result counters
        let result_key = match &base_event.result {
            crate::events::event_types::OperationResult::Success => "results_success",
            crate::events::event_types::OperationResult::Failure { .. } => "results_failure",
            crate::events::event_types::OperationResult::Cancelled => "results_cancelled",
            crate::events::event_types::OperationResult::InProgress => "results_in_progress",
            crate::events::event_types::OperationResult::Timeout => "results_timeout",
        };
        *metrics.entry(result_key.to_string()).or_insert(0.0) += 1.0;
        
        // Track duration metrics if available
        if let Some(duration) = &base_event.duration {
            let duration_ms = duration.as_millis() as f64;
            
            let operation_key = format!("duration_{}", base_event.operation);
            let current_avg = metrics.get(&operation_key).unwrap_or(&0.0);
            let count_key = format!("count_{}", base_event.operation);
            let count = metrics.get(&count_key).unwrap_or(&0.0) + 1.0;
            
            // Exponential moving average
            let new_avg = if *current_avg == 0.0 {
                duration_ms
            } else {
                0.9 * current_avg + 0.1 * duration_ms
            };
            
            metrics.insert(operation_key, new_avg);
            metrics.insert(count_key, count);
        }
        
        // Detailed metrics if enabled
        if self.detailed {
            let component_key = format!("events_by_component_{}", base_event.component);
            *metrics.entry(component_key).or_insert(0.0) += 1.0;
            
            let operation_key = format!("events_by_operation_{}", base_event.operation);
            *metrics.entry(operation_key).or_insert(0.0) += 1.0;
        }
        
        let mut result_metadata = HashMap::new();
        result_metadata.insert("metrics_collected".to_string(), serde_json::Value::Number(metrics.len().into()));
        
        EventHandlerResult {
            handler_name: self.name.clone(),
            success: true,
            duration: start_time.elapsed(),
            error: None,
            metadata: result_metadata,
        }
    }
    
    fn handler_name(&self) -> String {
        self.name.clone()
    }
}

/// Built-in security alerting handler
pub struct SecurityAlertHandler {
    /// Name of this handler
    name: String,
    /// Minimum severity for alerts
    min_severity: crate::events::event_types::EventSeverity,
    /// Alert destinations
    alert_destinations: Vec<AlertDestination>,
    /// Rate limiting configuration
    rate_limit: Option<RateLimit>,
    /// Last alert times for rate limiting
    last_alert_times: std::sync::Arc<tokio::sync::RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

/// Alert destination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertDestination {
    /// Log alert to console
    Console,
    /// Send alert to webhook URL
    Webhook { url: String, headers: HashMap<String, String> },
    /// Write alert to file
    File { path: String },
    /// Send email alert (placeholder for future implementation)
    Email { to: String, subject: String },
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum alerts per time window
    max_alerts: u32,
    /// Time window in seconds
    window_seconds: u64,
}

impl SecurityAlertHandler {
    /// Create a new security alert handler
    pub fn new(min_severity: crate::events::event_types::EventSeverity) -> Self {
        Self {
            name: "security_alert_handler".to_string(),
            min_severity,
            alert_destinations: vec![AlertDestination::Console],
            rate_limit: Some(RateLimit {
                max_alerts: 10,
                window_seconds: 300, // 5 minutes
            }),
            last_alert_times: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    /// Add an alert destination
    pub fn add_destination(mut self, destination: AlertDestination) -> Self {
        self.alert_destinations.push(destination);
        self
    }
    
    /// Set rate limiting
    pub fn with_rate_limit(mut self, rate_limit: Option<RateLimit>) -> Self {
        self.rate_limit = rate_limit;
        self
    }
    
    /// Set handler name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    
    /// Check if an alert should be rate limited
    async fn should_rate_limit(&self, alert_key: &str) -> bool {
        if let Some(rate_limit) = &self.rate_limit {
            let mut last_times = self.last_alert_times.write().await;
            let now = chrono::Utc::now();
            let window_start = now - chrono::Duration::seconds(rate_limit.window_seconds as i64);
            
            // Clean up old entries
            last_times.retain(|_, time| *time > window_start);
            
            // Count recent alerts for this key
            let recent_count = last_times.values()
                .filter(|time| **time > window_start)
                .count() as u32;
            
            if recent_count >= rate_limit.max_alerts {
                return true; // Rate limited
            }
            
            // Record this alert
            last_times.insert(alert_key.to_string(), now);
        }
        
        false
    }
}

#[async_trait]
impl EventHandler for SecurityAlertHandler {
    async fn handle_event(&self, event: &SecurityEvent) -> EventHandlerResult {
        let start_time = std::time::Instant::now();
        let base_event = event.base_event();
        
        // Check if this event meets the severity threshold
        if !event.should_alert(self.min_severity) {
            return EventHandlerResult {
                handler_name: self.name.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("skipped".to_string(), serde_json::Value::Bool(true));
                    metadata.insert("reason".to_string(), serde_json::Value::String("Below severity threshold".to_string()));
                    metadata
                },
            };
        }
        
        // Check rate limiting
        let alert_key = format!("{}_{}", base_event.category, base_event.severity);
        if self.should_rate_limit(&alert_key).await {
            return EventHandlerResult {
                handler_name: self.name.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("rate_limited".to_string(), serde_json::Value::Bool(true));
                    metadata
                },
            };
        }
        
        // Generate alert message
        let alert_message = format!(
            "🚨 SECURITY ALERT: {} {} event from {} - {} in {}: {}",
            base_event.severity,
            base_event.category,
            base_event.platform,
            base_event.operation,
            base_event.component,
            match &base_event.result {
                crate::events::event_types::OperationResult::Success => "completed successfully".to_string(),
                crate::events::event_types::OperationResult::Failure { error_message, .. } => format!("failed: {}", error_message),
                crate::events::event_types::OperationResult::Cancelled => "was cancelled".to_string(),
                crate::events::event_types::OperationResult::InProgress => "is in progress".to_string(),
                crate::events::event_types::OperationResult::Timeout => "timed out".to_string(),
            }
        );
        
        // Send alerts to all destinations
        let mut success = true;
        let mut errors = Vec::new();
        
        for destination in &self.alert_destinations {
            match destination {
                AlertDestination::Console => {
                    log::warn!("{}", alert_message);
                }
                AlertDestination::File { path } => {
                    if let Err(e) = self.write_alert_to_file(path, &alert_message).await {
                        success = false;
                        errors.push(format!("File alert failed: {}", e));
                    }
                }
                AlertDestination::Webhook { url, headers } => {
                    if let Err(e) = self.send_webhook_alert(url, headers, &alert_message, event).await {
                        success = false;
                        errors.push(format!("Webhook alert failed: {}", e));
                    }
                }
                AlertDestination::Email { .. } => {
                    // Placeholder for email implementation
                    log::info!("Email alerting not yet implemented");
                }
            }
        }
        
        let mut metadata = HashMap::new();
        metadata.insert("destinations_count".to_string(), serde_json::Value::Number(self.alert_destinations.len().into()));
        metadata.insert("alert_message".to_string(), serde_json::Value::String(alert_message));
        
        if !errors.is_empty() {
            metadata.insert("errors".to_string(), serde_json::Value::Array(
                errors.iter().map(|e| serde_json::Value::String(e.clone())).collect()
            ));
        }
        
        EventHandlerResult {
            handler_name: self.name.clone(),
            success,
            duration: start_time.elapsed(),
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            metadata,
        }
    }
    
    fn handler_name(&self) -> String {
        self.name.clone()
    }
    
    fn can_handle(&self, event: &SecurityEvent) -> bool {
        event.should_alert(self.min_severity)
    }
}

impl SecurityAlertHandler {
    /// Write alert to file
    async fn write_alert_to_file(&self, path: &str, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        
        let timestamped_message = format!("[{}] {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"), message);
        file.write_all(timestamped_message.as_bytes()).await?;
        
        Ok(())
    }
    
    /// Send webhook alert
    async fn send_webhook_alert(
        &self,
        url: &str,
        _headers: &HashMap<String, String>,
        message: &str,
        event: &SecurityEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create webhook payload
        let payload = serde_json::json!({
            "message": message,
            "event": event,
            "timestamp": chrono::Utc::now(),
            "source": "datafold_verification_bus"
        });
        
        // This is a placeholder - in a real implementation, you would use an HTTP client
        // like reqwest to send the webhook
        log::info!("Would send webhook to {}: {}", url, payload);
        
        // For now, just log that we would send it
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::{SecurityEvent, SecurityEventCategory, EventSeverity, PlatformSource, CreateVerificationEvent, VerificationEvent};

    #[tokio::test]
    async fn test_metrics_handler() {
        let handler = MetricsHandler::new();
        
        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            EventSeverity::Info,
            PlatformSource::RustCli,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));
        
        let result = handler.handle_event(&event).await;
        assert!(result.success);
        assert_eq!(result.handler_name, "metrics_handler");
        
        let metrics = handler.get_metrics().await;
        assert_eq!(metrics.get("total_events"), Some(&1.0));
        assert_eq!(metrics.get("events_by_severity_Info"), Some(&1.0));
    }

    #[tokio::test]
    async fn test_security_alert_handler() {
        let handler = SecurityAlertHandler::new(EventSeverity::Warning);
        
        // This should trigger an alert
        let critical_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            EventSeverity::Critical,
            PlatformSource::DataFoldNode,
            "security_monitor".to_string(),
            "threat_detected".to_string(),
        ));
        
        let result = handler.handle_event(&critical_event).await;
        assert!(result.success);
        
        // This should be skipped
        let info_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Performance,
            EventSeverity::Info,
            PlatformSource::JavaScriptSdk,
            "perf_monitor".to_string(),
            "metric_update".to_string(),
        ));
        
        let result = handler.handle_event(&info_event).await;
        assert!(result.success);
        assert_eq!(result.metadata.get("skipped"), Some(&serde_json::Value::Bool(true)));
    }
}