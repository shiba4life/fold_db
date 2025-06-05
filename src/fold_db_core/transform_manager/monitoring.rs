use super::manager::TransformManager;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use log::info;
use std::sync::Arc;
use std::thread;

impl TransformManager {
    /// Set up monitoring of SchemaChanged events to reload transforms
    pub(super) fn setup_schema_changed_monitoring(
        message_bus: Arc<MessageBus>,
        db_ops: Arc<DbOperations>,
    ) -> thread::JoinHandle<()> {
        let mut consumer = message_bus.subscribe::<crate::fold_db_core::infrastructure::message_bus::SchemaChanged>();
        
        thread::spawn(move || {
            info!("🔍 TransformManager: Starting monitoring of SchemaChanged events for transform reloading");
            
            loop {
                match consumer.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(event) => {
                        info!(
                            "🎯 TransformManager: SchemaChanged event received for schema: {}",
                            event.schema
                        );
                        
                        // Note: Transform reloading is now handled via event-driven architecture
                        // TransformOrchestrator will automatically handle transform registration
                        // when new schemas are loaded
                        info!(
                            "✅ TransformManager: Schema change handled via event-driven architecture for '{}'",
                            event.schema
                        );
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

}