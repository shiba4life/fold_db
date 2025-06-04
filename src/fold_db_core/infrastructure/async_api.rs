//! # Async Event-Driven API Layer
//!
//! This module provides a pure event-driven API that operates entirely through events
//! without direct manager calls. It demonstrates how external interfaces can be
//! completely decoupled from the underlying implementation through event patterns.

use crate::fold_db_core::infrastructure::message_bus::{
    AsyncMessageBus, Event, FieldValueSet, QueryExecuted, MutationExecuted, AsyncRecvError, EnhancedMessageBus
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use thiserror::Error;

/// Errors for async API operations
#[derive(Error, Debug)]
pub enum AsyncApiError {
    #[error("Timeout waiting for operation result")]
    Timeout,
    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },
    #[error("Message bus error: {0}")]
    MessageBusError(#[from] crate::fold_db_core::infrastructure::message_bus::MessageBusError),
    #[error("Async receive error: {0}")]
    AsyncRecvError(#[from] AsyncRecvError),
    #[error("Channel receive error: {0}")]
    ChannelRecvError(#[from] tokio::sync::oneshot::error::RecvError),
}

/// Result type for async API operations
pub type AsyncApiResult<T> = Result<T, AsyncApiError>;

/// Request/Response correlation for async operations
#[derive(Debug, Clone)]
pub struct OperationRequest {
    pub request_id: String,
    pub operation_type: String,
    pub payload: Value,
    pub timeout_ms: u64,
}

impl OperationRequest {
    pub fn new(operation_type: impl Into<String>, payload: Value, timeout_ms: u64) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation_type: operation_type.into(),
            payload,
            timeout_ms,
        }
    }
}

/// Response for async operations
#[derive(Debug, Clone)]
pub struct OperationResponse {
    pub request_id: String,
    pub success: bool,
    pub result: Value,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Pure event-driven async API
#[allow(dead_code)]
pub struct AsyncEventApi {
    message_bus: Arc<AsyncMessageBus>,
    enhanced_bus: Arc<EnhancedMessageBus>,
    pending_requests: Arc<tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<OperationResponse>>>>,
}

impl AsyncEventApi {
    /// Create a new async event API
    pub fn new(message_bus: Arc<AsyncMessageBus>, enhanced_bus: Arc<EnhancedMessageBus>) -> Self {
        Self {
            message_bus,
            enhanced_bus,
            pending_requests: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Set field value asynchronously through events only
    pub async fn set_field_value_async(
        &self,
        field_path: impl Into<String>,
        value: Value,
        timeout_ms: u64,
    ) -> AsyncApiResult<OperationResponse> {
        let request = OperationRequest::new("set_field_value", value.clone(), timeout_ms);
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request.request_id.clone(), response_tx);
        }

        // Publish field value set event
        let event = FieldValueSet::new(field_path.into(), value, format!("async_api:{}", request.request_id));
        self.message_bus.publish_field_value_set(event).await?;

        // Wait for response with timeout
        let response = timeout(
            Duration::from_millis(timeout_ms),
            response_rx
        ).await.map_err(|_| AsyncApiError::Timeout)?
        .map_err(AsyncApiError::ChannelRecvError)?;

        Ok(response)
    }

    /// Execute query asynchronously through events only
    pub async fn query_async(
        &self,
        schema: impl Into<String>,
        query_type: impl Into<String>,
        filter: Option<Value>,
        timeout_ms: u64,
    ) -> AsyncApiResult<OperationResponse> {
        let schema_str = schema.into();
        let query_type_str = query_type.into();
        
        let request = OperationRequest::new(
            "query",
            serde_json::json!({
                "schema": schema_str.clone(),
                "query_type": query_type_str.clone(),
                "filter": filter
            }),
            timeout_ms
        );
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request.request_id.clone(), response_tx);
        }

        // Simulate query execution by publishing query executed event
        // In a real implementation, this would trigger query processing
        let query_event = QueryExecuted::new(
            query_type_str,
            schema_str.clone(),
            50, // simulated execution time
            10, // simulated result count
        );
        self.message_bus.publish_query_executed(query_event).await?;

        // Simulate immediate response for demo
        let response = OperationResponse {
            request_id: request.request_id.clone(),
            success: true,
            result: serde_json::json!({
                "results": [],
                "count": 10,
                "execution_time_ms": 50
            }),
            execution_time_ms: 50,
            error: None,
        };

        // Complete the request
        self.complete_request(&request.request_id, response.clone()).await;

        Ok(response)
    }

    /// Execute mutation asynchronously through events only
    pub async fn mutate_async(
        &self,
        schema: impl Into<String>,
        operation: impl Into<String>,
        data: Value,
        timeout_ms: u64,
    ) -> AsyncApiResult<OperationResponse> {
        let schema_str = schema.into();
        let operation_str = operation.into();
        
        let request = OperationRequest::new(
            "mutation",
            serde_json::json!({
                "schema": schema_str.clone(),
                "operation": operation_str.clone(),
                "data": data
            }),
            timeout_ms
        );
        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request.request_id.clone(), response_tx);
        }

        // Publish mutation executed event
        let mutation_event = MutationExecuted::new(
            operation_str,
            schema_str,
            75, // simulated execution time
            3,  // simulated fields affected
        );
        self.message_bus.publish_mutation_executed(mutation_event).await?;

        // Simulate immediate response for demo
        let response = OperationResponse {
            request_id: request.request_id.clone(),
            success: true,
            result: serde_json::json!({
                "affected_fields": 3,
                "execution_time_ms": 75
            }),
            execution_time_ms: 75,
            error: None,
        };

        // Complete the request
        self.complete_request(&request.request_id, response.clone()).await;

        Ok(response)
    }

    /// Start event processing loop
    pub async fn start_event_processor(&self) -> AsyncApiResult<()> {
        let mut consumer = self.message_bus.subscribe_all().await;
        let pending_requests = Arc::clone(&self.pending_requests);

        tokio::spawn(async move {
            while let Some(event) = consumer.recv().await {
                // Process events and complete pending requests
                match event {
                    Event::FieldValueSet(field_event) => {
                        // Extract request ID from source if it's from async API
                        if let Some(request_id) = Self::extract_request_id(&field_event.source) {
                            let response = OperationResponse {
                                request_id: request_id.clone(),
                                success: true,
                                result: serde_json::json!({
                                    "field": field_event.field,
                                    "value": field_event.value
                                }),
                                execution_time_ms: 25,
                                error: None,
                            };
                            Self::complete_request_static(&pending_requests, &request_id, response).await;
                        }
                    }
                    Event::QueryExecuted(query_event) => {
                        // Handle query completion if needed
                        log::info!("Query executed: {} on {} in {}ms", 
                                  query_event.query_type, query_event.schema, query_event.execution_time_ms);
                    }
                    Event::MutationExecuted(mutation_event) => {
                        // Handle mutation completion if needed
                        log::info!("Mutation executed: {} on {} in {}ms", 
                                  mutation_event.operation, mutation_event.schema, mutation_event.execution_time_ms);
                    }
                    _ => {
                        // Handle other events as needed
                    }
                }
            }
        });

        Ok(())
    }

    /// Complete a pending request
    async fn complete_request(&self, request_id: &str, response: OperationResponse) {
        let mut pending = self.pending_requests.lock().await;
        if let Some(sender) = pending.remove(request_id) {
            let _ = sender.send(response);
        }
    }

    /// Static version for use in spawned tasks
    async fn complete_request_static(
        pending_requests: &Arc<tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<OperationResponse>>>>,
        request_id: &str,
        response: OperationResponse
    ) {
        let mut pending = pending_requests.lock().await;
        if let Some(sender) = pending.remove(request_id) {
            let _ = sender.send(response);
        }
    }

    /// Extract request ID from source string
    fn extract_request_id(source: &str) -> Option<String> {
        if source.starts_with("async_api:") {
            Some(source.strip_prefix("async_api:").unwrap().to_string())
        } else {
            None
        }
    }

    /// Get statistics about pending requests
    pub async fn get_pending_requests_count(&self) -> usize {
        let pending = self.pending_requests.lock().await;
        pending.len()
    }
}

/// Concurrent event processor for high-performance async processing
pub struct ConcurrentEventProcessor {
    message_bus: Arc<AsyncMessageBus>,
    worker_count: usize,
    processing_stats: Arc<tokio::sync::Mutex<ProcessingStats>>,
}

/// Statistics for concurrent processing
#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub events_processed: u64,
    pub average_processing_time_ms: f64,
    pub concurrent_workers: usize,
    pub queue_depth: usize,
}

impl ConcurrentEventProcessor {
    /// Create a new concurrent event processor
    pub fn new(message_bus: Arc<AsyncMessageBus>, worker_count: usize) -> Self {
        Self {
            message_bus,
            worker_count,
            processing_stats: Arc::new(tokio::sync::Mutex::new(ProcessingStats {
                concurrent_workers: worker_count,
                ..Default::default()
            })),
        }
    }

    /// Start concurrent event processing
    pub async fn start_concurrent_processing(&self) -> AsyncApiResult<()> {
        let mut workers = Vec::new();

        for worker_id in 0..self.worker_count {
            let message_bus = Arc::clone(&self.message_bus);
            let stats = Arc::clone(&self.processing_stats);

            let worker = tokio::spawn(async move {
                let mut consumer = message_bus.subscribe_all().await;
                
                while let Some(event) = consumer.recv().await {
                    let start_time = std::time::Instant::now();
                    
                    // Simulate async processing work
                    match event {
                        Event::FieldValueSet(field_event) => {
                            // Simulate I/O-bound work
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            log::debug!("Worker {} processed field event: {}", worker_id, field_event.field);
                        }
                        Event::QueryExecuted(query_event) => {
                            // Simulate query result processing
                            tokio::time::sleep(Duration::from_millis(20)).await;
                            log::debug!("Worker {} processed query: {}", worker_id, query_event.query_type);
                        }
                        Event::MutationExecuted(mutation_event) => {
                            // Simulate mutation result processing
                            tokio::time::sleep(Duration::from_millis(15)).await;
                            log::debug!("Worker {} processed mutation: {}", worker_id, mutation_event.operation);
                        }
                        _ => {
                            // Handle other events
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    }

                    // Update stats
                    let processing_time = start_time.elapsed().as_millis() as f64;
                    let mut stats_guard = stats.lock().await;
                    stats_guard.events_processed += 1;
                    
                    // Update running average
                    let total_events = stats_guard.events_processed as f64;
                    stats_guard.average_processing_time_ms = 
                        (stats_guard.average_processing_time_ms * (total_events - 1.0) + processing_time) / total_events;
                }
            });

            workers.push(worker);
        }

        // Wait for all workers (they run indefinitely)
        for worker in workers {
            let _ = worker.await;
        }

        Ok(())
    }

    /// Get processing statistics
    pub async fn get_stats(&self) -> ProcessingStats {
        let stats = self.processing_stats.lock().await;
        ProcessingStats {
            events_processed: stats.events_processed,
            average_processing_time_ms: stats.average_processing_time_ms,
            concurrent_workers: stats.concurrent_workers,
            queue_depth: stats.queue_depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_async_field_value_api() {
        let async_bus = Arc::new(AsyncMessageBus::new());
        let enhanced_bus = Arc::new(EnhancedMessageBus::new());
        let api = AsyncEventApi::new(async_bus, enhanced_bus);

        // Start event processor
        let _ = api.start_event_processor().await;

        // Test async field value setting
        let result = api.set_field_value_async(
            "test.field",
            serde_json::json!("test_value"),
            1000
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.result["field"], "test.field");
    }

    #[tokio::test]
    async fn test_concurrent_event_processing() {
        let async_bus = Arc::new(AsyncMessageBus::new());
        let processor = ConcurrentEventProcessor::new(async_bus.clone(), 4);

        // Start processing in background
        tokio::spawn(async move {
            let _ = processor.start_concurrent_processing().await;
        });

        // Publish some events to process
        for i in 0..10 {
            let event = FieldValueSet::new(
                format!("test.field.{}", i),
                serde_json::json!(format!("value_{}", i)),
                "test_source"
            );
            let _ = async_bus.publish_field_value_set(event).await;
        }

        // Allow some processing time
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}