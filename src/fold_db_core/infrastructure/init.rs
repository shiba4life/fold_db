use crate::fold_db_core::{
    managers::FieldManager,
    infrastructure::message_bus::MessageBus,
    transform_manager::TransformManager,
    orchestration::TransformOrchestrator,
};
use crate::db_operations::DbOperations;
use sled::Tree;
use std::sync::Arc;


pub fn init_transform_manager(
    db_ops: Arc<DbOperations>,
    message_bus: Arc<MessageBus>,
) -> Result<Arc<TransformManager>, sled::Error> {
    let mgr = TransformManager::new(
        db_ops.clone(),
        message_bus,
    )
    .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
    Ok(Arc::new(mgr))
}

pub fn init_orchestrator(
    _field_manager: &FieldManager,
    transform_manager: Arc<TransformManager>,
    tree: Tree,
    message_bus: Arc<MessageBus>,
) -> Result<Arc<TransformOrchestrator>, sled::Error> {
    // In event-driven mode, transform manager and orchestrator integration happens through events
    let orchestrator = Arc::new(TransformOrchestrator::new(transform_manager, tree, message_bus));
    Ok(orchestrator)
}
