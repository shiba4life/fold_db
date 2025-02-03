mod models;
mod response;
mod collection;
mod common;
mod error;

use std::sync::Arc;
use crate::schema::SchemaManager;
use crate::store::Store;
use response::{ResponseBuilder, QueryResponseBuilder, WriteResponseBuilder};
use common::{OperationContext, check_permissions};
use collection::{CollectionOptions, process_collection};
use error::handle_operation_result;

pub use models::{QueryItem, QueryPayload, QueryResult, QueryResponse, WriteItem, WritePayload, WriteResult, WriteResponse};

/// Query data from the database
pub fn query(
    schema_manager: Arc<SchemaManager>,
    store: Arc<Store>,
    payload: QueryPayload,
) -> QueryResponse {
    let mut response_builder = QueryResponseBuilder::new();

    for query in &payload.queries {
        match query {
            QueryItem::Field { schema, field } => {
                let context = serde_json::json!({
                    "type": "field",
                    "schema": schema,
                    "field": field
                });

                let _op_ctx = OperationContext {
                    schema_manager: &schema_manager,
                    store: &store,
                    schema: schema.as_str(),
                    target: field.as_str(),
                    operation: crate::schema::Operation::Read,
                };

                // Check if schema exists first
                if !schema_manager.is_loaded(schema).unwrap_or(false) {
                    response_builder.schema_not_loaded(context);
                    continue;
                }

                handle_operation_result(
                    store.get_field_value(schema.as_str(), field.as_str()),
                    context,
                    &mut response_builder,
                    |value, ctx, builder| {
                        builder.operation_success(ctx, value)
                    }
                );
            },
            QueryItem::Collection { schema, collection, sort, sort_field, limit } => {
                let context = serde_json::json!({
                    "type": "collection",
                    "schema": schema,
                    "collection": collection,
                    "sort": sort,
                    "sort_field": sort_field,
                    "limit": limit
                });

                let _op_ctx = OperationContext {
                    schema_manager: &schema_manager,
                    store: &store,
                    schema: schema.as_str(),
                    target: collection.as_str(),
                    operation: crate::schema::Operation::Read,
                };

                handle_operation_result(
                    store.get_collection(schema.as_str(), collection.as_str())
                        .map(|items| process_collection(items, &CollectionOptions {
                            sort_field: sort_field.as_ref(),
                            sort_order: sort.as_ref(),
                            limit: *limit,
                        })),
                    context,
                    &mut response_builder,
                    |items, ctx, builder| builder.operation_success(ctx, serde_json::json!(items))
                );
            },
        }
    }

    QueryResponse {
        results: response_builder.build()
    }
}

/// Write data to the database
pub fn write(
    schema_manager: Arc<SchemaManager>,
    store: Arc<Store>,
    payload: WritePayload,
) -> WriteResponse {
    let mut response_builder = WriteResponseBuilder::new();

    for write in &payload.writes {
        match write {
            WriteItem::WriteField { schema, field, value } => {
                let context = serde_json::json!({
                    "type": "write_field",
                    "schema": schema,
                    "field": field
                });

                let _op_ctx = OperationContext {
                    schema_manager: &schema_manager,
                    store: &store,
                    schema: schema.as_str(),
                    target: field.as_str(),
                    operation: crate::schema::Operation::Write,
                };

                // Check if schema exists first
                if !schema_manager.is_loaded(schema).unwrap_or(false) {
                    response_builder.schema_not_loaded(context);
                    continue;
                }

                // Then check permissions and write
                handle_operation_result(
                    check_permissions(&_op_ctx)
                        .and_then(|_| store.write_field(schema.as_str(), field.as_str(), value)),
                    context,
                    &mut response_builder,
                    |_, ctx, builder| builder.operation_success(ctx, serde_json::Value::Null)
                );
            },
            WriteItem::WriteCollection { schema, collection, item } => {
                let context = serde_json::json!({
                    "type": "write_collection",
                    "schema": schema,
                    "collection": collection
                });

                let _op_ctx = OperationContext {
                    schema_manager: &schema_manager,
                    store: &store,
                    schema: schema.as_str(),
                    target: collection.as_str(),
                    operation: crate::schema::Operation::Write,
                };

                handle_operation_result(
                    check_permissions(&_op_ctx)
                        .and_then(|_| store.write_collection(schema.as_str(), collection.as_str(), item)),
                    context,
                    &mut response_builder,
                    |_, ctx, builder| {
                        // After writing, get the new item to return
                        match store.get_collection(schema.as_str(), collection.as_str()) {
                            Ok(items) => builder.operation_success(ctx, serde_json::json!(items)),
                            Err(e) => builder.operation_error(ctx, format!("Failed to read written collection: {}", e))
                        }
                    }
                );
            },
        }
    }

    WriteResponse {
        results: response_builder.build()
    }
}
