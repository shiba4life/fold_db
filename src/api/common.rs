use std::sync::Arc;
use crate::schema::{SchemaManager, Operation};

pub struct OperationContext<'a> {
    pub schema_manager: &'a Arc<SchemaManager>,
    pub schema: &'a str,
    pub target: &'a str,
    pub operation: Operation,
    pub public_key: &'a str,
    pub distance: u32,
}

pub fn check_permissions(ctx: &OperationContext) -> Result<(), String> {
    let mut internal_schema = ctx.schema_manager
        .get_schema(ctx.schema)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "schema not loaded".to_string())?
        .clone();

    let has_permission = crate::schema::SecurityManager::check_permission(
        &mut internal_schema,
        ctx.target,
        ctx.operation.clone(),
        ctx.distance,
        true,
        ctx.public_key,
    );

    // Update the schema to persist any permission changes
    if has_permission {
        ctx.schema_manager.load_schema(ctx.schema, internal_schema)?;
        Ok(())
    } else {
        Err("permission denied".to_string())
    }
}
