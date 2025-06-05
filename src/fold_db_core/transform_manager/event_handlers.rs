use super::manager::TransformManager;
use crate::fold_db_core::infrastructure::message_bus::{
    MessageBus, TransformTriggered, TransformExecuted
};
use log::{info, error};
use std::sync::Arc;
use std::thread;

impl TransformManager {


    /// Set up processing of TransformTriggered events
    pub(super) fn setup_transform_triggered_processing(
        message_bus: Arc<MessageBus>,
        db_ops: Option<Arc<crate::db_operations::DbOperations>>,
    ) -> thread::JoinHandle<()> {
        let mut consumer = message_bus.subscribe::<TransformTriggered>();
        let message_bus_for_publish = Arc::clone(&message_bus);
        
        thread::spawn(move || {
            info!("🔍 TransformManager: Starting processing of TransformTriggered events");
            
            loop {
                match consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(event) => {
                        info!(
                            "🎯 TransformManager: TransformTriggered received - transform_id: {}",
                            event.transform_id
                        );
                        
                        // Execute the transform using the existing execution pipeline
                        let (transforms_executed, success, error) = Self::execute_transform_with_db(
                            &event.transform_id,
                            &message_bus_for_publish,
                            db_ops.as_ref()
                        );
                        
                        // Publish TransformExecuted event
                        let executed_event = TransformExecuted {
                            transform_id: event.transform_id.clone(),
                            result: if success {
                                format!("success: {} transforms executed", transforms_executed)
                            } else {
                                error.unwrap_or("execution failed".to_string())
                            },
                        };
                        
                        match message_bus_for_publish.publish(executed_event) {
                            Ok(_) => {
                                info!(
                                    "📢 TransformManager: Published TransformExecuted for transform_id: {}",
                                    event.transform_id
                                );
                            }
                            Err(e) => {
                                error!(
                                    "❌ TransformManager: Failed to publish TransformExecuted for {}: {}",
                                    event.transform_id, e
                                );
                            }
                        }
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

}