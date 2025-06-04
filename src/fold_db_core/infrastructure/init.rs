use crate::fold_db_core::{
    managers::{AtomManager, FieldManager},
    infrastructure::message_bus::MessageBus,
    transform_manager::{
        types::{CreateAtomFn, GetAtomFn, GetFieldFn, UpdateAtomRefFn},
        TransformManager,
    },
    orchestration::TransformOrchestrator,
};
use crate::schema::{SchemaCore, SchemaError};
use crate::db_operations::DbOperations;
use serde_json::Value;
use sled::Tree;
use std::sync::Arc;

pub fn build_closure_fns(
    _atom_manager: &AtomManager,
    schema_manager: &Arc<SchemaCore>,
) -> (GetAtomFn, CreateAtomFn, UpdateAtomRefFn, GetFieldFn) {
    // CONVERTED TO EVENT-DRIVEN: Replace direct AtomManager calls with event-driven error responses
    let get_atom_fn: GetAtomFn = Arc::new(move |_aref_uuid: &str| {
        Err("Method deprecated: Use event-driven AtomGetRequest via message bus instead of direct method calls".into())
    });

    let create_atom_fn: CreateAtomFn = Arc::new(
        move |_schema_name: &str,
              _source_pub_key: String,
              _prev_atom_uuid: Option<String>,
              _content: Value,
              _status: Option<crate::atom::AtomStatus>| {
            Err("Method deprecated: Use event-driven AtomCreateRequest via message bus instead of direct method calls".into())
        },
    );

    let update_atom_ref_fn: UpdateAtomRefFn = Arc::new(
        move |_aref_uuid: &str, _atom_uuid: String, _source_pub_key: String| {
            Err("Method deprecated: Use event-driven AtomRefUpdateRequest via message bus instead of direct method calls".into())
        },
    );

    let message_bus = Arc::new(MessageBus::new());
    let field_value_manager = FieldManager::new(message_bus);
    let schema_clone = Arc::clone(schema_manager);
    let get_field_fn: GetFieldFn = Arc::new(move |schema_name: &str, field_name: &str| {
        match schema_clone.get_schema(schema_name)? {
            Some(schema) => field_value_manager.get_field_value(&schema, field_name),
            None => Err(SchemaError::InvalidField(format!(
                "Field not found: {}.{}",
                schema_name, field_name
            ))),
        }
    });

    (
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
        get_field_fn,
    )
}

pub fn init_transform_manager(
    db_ops: Arc<DbOperations>,
    closures: (GetAtomFn, CreateAtomFn, UpdateAtomRefFn, GetFieldFn),
    message_bus: Arc<MessageBus>,
) -> Result<Arc<TransformManager>, sled::Error> {
    let (get_atom_fn, create_atom_fn, update_atom_ref_fn, get_field_fn) = closures;
    let mgr = TransformManager::new(
        db_ops.clone(),
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
        get_field_fn,
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
