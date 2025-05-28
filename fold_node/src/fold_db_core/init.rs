use super::{
    atom_manager::AtomManager,
    field_manager::FieldManager,
    transform_manager::{TransformManager, types::{GetAtomFn, CreateAtomFn, UpdateAtomRefFn, GetFieldFn}},
    transform_orchestrator::TransformOrchestrator,
    SchemaCore, SchemaError
};
use crate::db_operations::DbOperations;
use serde_json::Value;
use std::sync::Arc;
use sled::Tree;

pub(super) fn build_closure_fns(
    atom_manager: &AtomManager,
    schema_manager: &Arc<SchemaCore>,
) -> (GetAtomFn, CreateAtomFn, UpdateAtomRefFn, GetFieldFn) {
    let am = atom_manager.clone();
    let get_atom_fn: GetAtomFn = Arc::new(move |aref_uuid: &str| am.get_latest_atom(aref_uuid));

    let am = atom_manager.clone();
    let create_atom_fn: CreateAtomFn = Arc::new(move |schema_name: &str,
        source_pub_key: String,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<crate::atom::AtomStatus>| {
            am.create_atom(schema_name, source_pub_key, prev_atom_uuid, content, status)
        });

    let am = atom_manager.clone();
    let update_atom_ref_fn: UpdateAtomRefFn = Arc::new(move |aref_uuid: &str, atom_uuid: String, source_pub_key: String| {
        am.update_atom_ref(aref_uuid, atom_uuid, source_pub_key)
    });

    let field_value_manager = FieldManager::new(atom_manager.clone());
    let schema_clone = Arc::clone(schema_manager);
    let get_field_fn: GetFieldFn = Arc::new(move |schema_name: &str, field_name: &str| {
        match schema_clone.get_schema(schema_name)? {
            Some(schema) => field_value_manager.get_field_value(&schema, field_name),
            None => Err(SchemaError::InvalidField(format!("Field not found: {}.{}", schema_name, field_name))),
        }
    });

    (get_atom_fn, create_atom_fn, update_atom_ref_fn, get_field_fn)
}

pub(super) fn init_transform_manager(
    db_ops: Arc<DbOperations>,
    closures: (GetAtomFn, CreateAtomFn, UpdateAtomRefFn, GetFieldFn),
) -> Result<Arc<TransformManager>, sled::Error> {
    let (get_atom_fn, create_atom_fn, update_atom_ref_fn, get_field_fn) = closures;
    let mgr = TransformManager::new(
        db_ops.clone(),
        get_atom_fn,
        create_atom_fn,
        update_atom_ref_fn,
        get_field_fn,
    ).map_err(|e| sled::Error::Unsupported(e.to_string()))?;
    Ok(Arc::new(mgr))
}

pub(super) fn init_orchestrator(
    field_manager: &FieldManager,
    transform_manager: Arc<TransformManager>,
    tree: Tree,
) -> Result<Arc<TransformOrchestrator>, sled::Error> {
    field_manager
        .set_transform_manager(transform_manager.clone())
        .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
    let orchestrator = Arc::new(TransformOrchestrator::new(transform_manager, tree));
    field_manager
        .set_orchestrator(orchestrator.clone())
        .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
    Ok(orchestrator)
}
