use serde_json::Value as JsonValue;
use super::response::ResponseBuilder;

pub fn handle_operation_result<T, B: ResponseBuilder>(
    result: Result<T, String>,
    context: JsonValue,
    response_builder: &mut B,
    success_handler: impl FnOnce(T, JsonValue, &mut B),
) {
    match result {
        Ok(value) => success_handler(value, context, response_builder),
        Err(err) => {
            if err == "schema not loaded" {
                response_builder.schema_not_loaded(context);
            } else if err == "permission denied" {
                response_builder.permission_denied(context);
            } else {
                response_builder.operation_error(context, err);
            }
        }
    }
}
