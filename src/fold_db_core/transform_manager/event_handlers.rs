use super::manager::TransformManager;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus, TransformTriggered, TransformExecuted,
    TransformTriggerRequest, TransformTriggerResponse,
    TransformExecutionRequest, TransformExecutionResponse
};
use log::{info, error};
use std::sync::Arc;
use std::thread;

impl TransformManager {
    /// Set up monitoring of TransformTriggered events for direct execution
    pub(super) fn setup_transform_triggered_monitoring(
        message_bus: Arc<MessageBus>,
    ) -> thread::JoinHandle<()> {
        let mut transform_triggered_consumer = message_bus.subscribe::<TransformTriggered>();
        let message_bus_for_publish = Arc::clone(&message_bus);
        
        thread::spawn(move || {
            info!("🔍 TransformManager: Starting monitoring and execution of TransformTriggered events");
            
            loop {
                match transform_triggered_consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(event) => {
                        info!(
                            "🎯 TransformManager: TransformTriggered event received for transform: {}",
                            event.transform_id
                        );
                        
                        // Execute the transform directly in response to the event
                        // This implements proper event-driven architecture where components
                        // handle events they subscribe to
                        Self::execute_transform_from_event(
                            &event.transform_id,
                            &message_bus_for_publish,
                        );
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }
    
    /// Execute a transform in response to a TransformTriggered event
    pub(super) fn execute_transform_from_event(
        transform_id: &str,
        message_bus: &Arc<MessageBus>,
    ) {
        info!("🚀 TransformManager: Processing TransformTriggered event for: {}", transform_id);
        
        // Instead of executing directly, publish a TransformExecutionRequest
        // This maintains event-driven architecture and proper separation of concerns
        let correlation_id = format!("transform_triggered_{}", transform_id);
        error!("🔍 DEBUG: Creating TransformExecutionRequest with correlation_id: '{}'", correlation_id);
        let execution_request = crate::fold_db_core::infrastructure::message_bus::TransformExecutionRequest {
            correlation_id,
        };
        
        match message_bus.publish(execution_request) {
            Ok(_) => {
                info!(
                    "📢 TransformManager: Published TransformExecutionRequest for {}",
                    transform_id
                );
            }
            Err(e) => {
                error!(
                    "❌ TransformManager: Failed to publish TransformExecutionRequest for {}: {}",
                    transform_id, e
                );
                
                // Publish failure event
                let executed_event = TransformExecuted {
                    transform_id: transform_id.to_string(),
                    result: format!("failed_to_trigger: {}", e),
                };
                let _ = message_bus.publish(executed_event);
            }
        }
    }

    /// Set up processing of TransformTriggerRequest events
    pub(super) fn setup_transform_trigger_request_processing(
        message_bus: Arc<MessageBus>,
    ) -> thread::JoinHandle<()> {
        let mut consumer = message_bus.subscribe::<TransformTriggerRequest>();
        let message_bus_for_publish = Arc::clone(&message_bus);
        
        thread::spawn(move || {
            info!("🔍 TransformManager: Starting processing of TransformTriggerRequest events");
            
            loop {
                match consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(request) => {
                        info!(
                            "🎯 TransformManager: TransformTriggerRequest received - correlation_id: {}, schema: {}, field: {}",
                            request.correlation_id, request.schema_name, request.field_name
                        );
                        
                        // This should check for transforms triggered by the field and publish TransformTriggered events
                        // For now, we'll publish a TransformExecutionRequest to maintain the event flow
                        let execution_request = crate::fold_db_core::infrastructure::message_bus::TransformExecutionRequest {
                            correlation_id: format!("trigger_{}_{}.{}", request.correlation_id, request.schema_name, request.field_name),
                        };
                        
                        let (success, error) = match message_bus_for_publish.publish(execution_request) {
                            Ok(_) => {
                                info!("📢 Published TransformExecutionRequest for {}.{}", request.schema_name, request.field_name);
                                (true, None)
                            }
                            Err(e) => {
                                error!("❌ Failed to publish TransformExecutionRequest: {}", e);
                                (false, Some(format!("Failed to trigger execution: {}", e)))
                            }
                        };
                        
                        let response = TransformTriggerResponse {
                            correlation_id: request.correlation_id.clone(),
                            success,
                            error,
                        };
                        
                        match message_bus_for_publish.publish(response) {
                            Ok(_) => {
                                info!(
                                    "📢 TransformManager: Published TransformTriggerResponse for correlation_id: {}",
                                    request.correlation_id
                                );
                            }
                            Err(e) => {
                                error!(
                                    "❌ TransformManager: Failed to publish TransformTriggerResponse for {}: {}",
                                    request.correlation_id, e
                                );
                            }
                        }
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

    /// Set up processing of TransformExecutionRequest events
    pub(super) fn setup_transform_execution_request_processing(
        message_bus: Arc<MessageBus>,
    ) -> thread::JoinHandle<()> {
        let mut consumer = message_bus.subscribe::<TransformExecutionRequest>();
        let message_bus_for_publish = Arc::clone(&message_bus);
        
        thread::spawn(move || {
            info!("🔍 TransformManager: Starting processing of TransformExecutionRequest events");
            
            loop {
                match consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(request) => {
                        Self::process_transform_execution_request(request, &message_bus_for_publish);
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

    /// Process a single TransformExecutionRequest
    pub(super) fn process_transform_execution_request(
        request: TransformExecutionRequest,
        message_bus: &Arc<MessageBus>,
    ) {
        Self::process_transform_execution_request_with_db(request, message_bus, None);
    }

    /// Process a single TransformExecutionRequest with database access
    pub(super) fn process_transform_execution_request_with_db(
        request: TransformExecutionRequest,
        message_bus: &Arc<MessageBus>,
        db_ops: Option<&Arc<crate::db_operations::DbOperations>>,
    ) {
        info!(
            "🎯 TransformManager: TransformExecutionRequest received - correlation_id: {}",
            request.correlation_id
        );
        
        let (transforms_executed, success, error) = Self::execute_transform_from_correlation_with_db(
            &request.correlation_id,
            message_bus,
            db_ops
        );
        
        let response = TransformExecutionResponse {
            correlation_id: request.correlation_id.clone(),
            success,
            transforms_executed,
            error,
        };
        
        Self::publish_execution_response(response, message_bus, &request.correlation_id);
    }

    /// Publish the execution response with error handling
    pub(super) fn publish_execution_response(
        response: TransformExecutionResponse,
        message_bus: &Arc<MessageBus>,
        correlation_id: &str,
    ) {
        match message_bus.publish(response) {
            Ok(_) => {
                info!(
                    "📢 TransformManager: Published TransformExecutionResponse for correlation_id: {}",
                    correlation_id
                );
            }
            Err(e) => {
                error!(
                    "❌ TransformManager: Failed to publish TransformExecutionResponse for {}: {}",
                    correlation_id, e
                );
            }
        }
    }
}