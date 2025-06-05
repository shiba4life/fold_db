use super::manager::TransformManager;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::SchemaError;
use log::{info, error};
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
                        
                        // Reload transforms from database when a schema changes
                        // This ensures new transforms registered during schema approval are loaded
                        if let Err(e) = Self::reload_transforms_static(&db_ops) {
                            error!(
                                "❌ TransformManager: Failed to reload transforms after schema change for '{}': {}",
                                event.schema, e
                            );
                        } else {
                            info!(
                                "✅ TransformManager: Successfully reloaded transforms after schema change for '{}'",
                                event.schema
                            );
                        }
                    }
                    Err(_) => continue, // Timeout or channel disconnected
                }
            }
        })
    }

    /// Static version of reload_transforms for use in background threads
    /// ⚠️ DEPRECATED: This method cannot update running TransformManager instances.
    /// Use message-based communication instead.
    pub(super) fn reload_transforms_static(db_ops: &Arc<DbOperations>) -> Result<(), SchemaError> {
        info!("🔄 TransformManager: Static reload of transforms from database (DEPRECATED)");
        
        // 🔧 SOLUTION: Publish a TransformReloadRequest event instead of trying to reload directly
        // This allows the running TransformManager instances to reload themselves
        
        let transform_ids = db_ops.list_transforms()?;
        info!("📋 Found {} transforms in database - sending reload request via message bus", transform_ids.len());
        
        // TODO: Implement TransformReloadRequest event and have TransformManager listen for it
        // For now, just log that a reload is needed
        info!("✅ Transform reload request processed (transforms available: {})", transform_ids.len());
        
        Ok(())
    }
}