use super::FoldDB;
use crate::schema::types::Query;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

impl FoldDB {
    pub fn query_schema(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        query
            .fields
            .iter()
            .map(|field_name| {
                info!("Processing field: {}", field_name);
                let perm_allowed = if query.trust_distance == 0 {
                    true
                } else {
                    let perm = self.permission_wrapper.check_query_field_permission(
                        &query,
                        field_name,
                        &self.schema_manager,
                    );
                    perm.allowed
                };

                if !perm_allowed {
                    let err = self
                        .permission_wrapper
                        .check_query_field_permission(&query, field_name, &self.schema_manager)
                        .error
                        .unwrap_or(SchemaError::InvalidPermission(
                            "Unknown permission error".to_string(),
                        ));
                    return Err(err);
                }

                let schema = match self.schema_manager.get_schema(&query.schema_name) {
                    Ok(Some(schema)) => schema,
                    Ok(None) => {
                        return Err(SchemaError::NotFound(format!(
                            "Schema {} not found",
                            query.schema_name
                        )))
                    }
                    Err(e) => return Err(e),
                };

                if let Some(ref filter_value) = query.filter {
                    info!("Query processing - field: {}, has filter: true, filter: {:?}", field_name, filter_value);
                    self.field_manager.get_filtered_field_value(&schema, field_name, filter_value)
                } else {
                    info!("Query processing - field: {}, has filter: false", field_name);
                    self.field_manager.get_field_value(&schema, field_name)
                }
            })
            .collect::<Vec<Result<Value, SchemaError>>>()
    }
}
