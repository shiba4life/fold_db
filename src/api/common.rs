use std::sync::Arc;
use crate::schema::{SchemaManager, Operation};
use crate::store::Store;

pub struct OperationContext<'a> {
    pub schema_manager: &'a Arc<SchemaManager>,
    pub store: &'a Arc<Store>,
    pub schema: &'a str,
    pub target: &'a str,
    pub operation: Operation,
}

pub fn check_permissions(ctx: &OperationContext) -> Result<(), String> {
    let internal_schema = ctx.schema_manager
        .get_schema(ctx.schema)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "schema not loaded".to_string())?;

    let public_key = "pubkey_example"; // TODO: Get from request context
    let distance = 0; // TODO: Calculate based on context

    if crate::schema::SecurityManager::check_permission(
        &mut internal_schema.clone(), // Clone to avoid mutability issues
        ctx.target,
        ctx.operation.clone(),
        distance,
        true,
        public_key,
    ) {
        Ok(())
    } else {
        Err("permission denied".to_string())
    }
}
