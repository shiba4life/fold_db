//! Event publishing utilities for consistent event handling across the transform system.
//!
//! This module provides unified event publishing patterns with consistent error handling
//! and logging to eliminate duplicate event publishing code.

use crate::fold_db_core::infrastructure::message_bus::{MessageBus, TransformExecuted};
use crate::schema::types::SchemaError;
use log::{info, error};
use std::sync::Arc;

/// Utility for publishing transform-related events with consistent error handling and logging
pub struct EventPublisher;

impl EventPublisher {
    /// Publish a TransformExecuted success event with consistent error handling
    pub fn publish_transform_executed_success(
        message_bus: &Arc<MessageBus>,
        transform_id: &str,
    ) -> Result<(), SchemaError> {
        info!("📢 Publishing TransformExecuted success event for: {}", transform_id);
        
        let event = TransformExecuted::new(transform_id, "success");
        
        match message_bus.publish(event) {
            Ok(_) => {
                info!("✅ Published TransformExecuted success event for transform: {}", transform_id);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to publish TransformExecuted success event for {}: {}", transform_id, e);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Publish a TransformExecuted failure event with consistent error handling
    pub fn publish_transform_executed_failure(
        message_bus: &Arc<MessageBus>,
        transform_id: &str,
    ) -> Result<(), SchemaError> {
        info!("📢 Publishing TransformExecuted failure event for: {}", transform_id);
        
        let event = TransformExecuted::new(transform_id, "failed");
        
        match message_bus.publish(event) {
            Ok(_) => {
                info!("✅ Published TransformExecuted failure event for transform: {}", transform_id);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to publish TransformExecuted failure event for {}: {}", transform_id, e);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Publish a TransformExecuted event with the specified status
    pub fn publish_transform_executed(
        message_bus: &Arc<MessageBus>,
        transform_id: &str,
        status: &str,
    ) -> Result<(), SchemaError> {
        info!("📢 Publishing TransformExecuted {} event for: {}", status, transform_id);
        
        let event = TransformExecuted::new(transform_id, status);
        
        match message_bus.publish(event) {
            Ok(_) => {
                info!("✅ Published TransformExecuted {} event for transform: {}", status, transform_id);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to publish TransformExecuted {} event for {}: {}", status, transform_id, e);
                error!("❌ {}", error_msg);
                Err(SchemaError::InvalidData(error_msg))
            }
        }
    }

    /// Execute and publish pattern - combines execution result handling with event publishing
    pub fn handle_execution_result_and_publish(
        message_bus: &Arc<MessageBus>,
        transform_id: &str,
        execution_result: &Result<serde_json::Value, crate::schema::types::SchemaError>,
    ) {
        match execution_result {
            Ok(value) => {
                info!("✅ Transform {} execution completed successfully", transform_id);
                info!("📊 Execution result details: {:?}", value);
                
                if let Err(e) = Self::publish_transform_executed_success(message_bus, transform_id) {
                    error!("❌ Event publishing failed after successful execution: {}", e);
                }
            }
            Err(e) => {
                error!("❌ Transform {} execution failed", transform_id);
                error!("❌ Failure details: {:?}", e);
                
                if let Err(publish_err) = Self::publish_transform_executed_failure(message_bus, transform_id) {
                    error!("❌ Event publishing failed after execution failure: {}", publish_err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold_db_core::infrastructure::message_bus::MessageBus;

    #[test]
    fn test_event_publisher_success() {
        let message_bus = Arc::new(MessageBus::new());
        let result = EventPublisher::publish_transform_executed_success(&message_bus, "test_transform");
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_publisher_failure() {
        let message_bus = Arc::new(MessageBus::new());
        let result = EventPublisher::publish_transform_executed_failure(&message_bus, "test_transform");
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_publisher_custom_status() {
        let message_bus = Arc::new(MessageBus::new());
        let result = EventPublisher::publish_transform_executed(&message_bus, "test_transform", "custom_status");
        assert!(result.is_ok());
    }
}