use super::FoldDB;
use crate::schema::types::Query;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

impl FoldDB {
    /// Query a Range schema and return grouped results by range_key
    pub fn query_range_schema(&self, query: Query) -> Result<Value, SchemaError> {
        info!(
            "🎯 FoldDB::query_range_schema - schema: {}, fields: {:?}",
            query.schema_name, query.fields
        );

        // Get schema and validate it's a Range schema
        let schema = match self.schema_manager.get_schema(&query.schema_name) {
            Ok(Some(schema)) => schema,
            Ok(None) => {
                return Err(SchemaError::NotFound(format!(
                    "Schema {} not found",
                    query.schema_name
                )));
            }
            Err(e) => return Err(e),
        };

        // Validate this is a Range schema
        if schema.range_key().is_none() {
            return Err(SchemaError::InvalidData(format!(
                "Schema '{}' is not a Range schema",
                query.schema_name
            )));
        }

        // Validate filter for Range schema
        if let Some(ref filter) = query.filter {
            schema.validate_range_filter(filter)?;
        } else {
            return Err(SchemaError::InvalidData(
                "Range schema queries require a filter with range_filter".to_string(),
            ));
        }

        // Check permissions for all fields
        for field_name in &query.fields {
            let perm = self.permission_wrapper.check_query_field_permission(
                &query,
                field_name,
                &self.schema_manager,
            );

            if !perm.allowed {
                let err = perm.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                ));
                return Err(err);
            }
        }

        // Extract range_filter from the main filter
        let range_filter = query
            .filter
            .as_ref()
            .and_then(|f| f.get("range_filter"))
            .ok_or_else(|| SchemaError::InvalidData("Missing range_filter in query".to_string()))?;

        // Use FieldRetrievalService to get grouped results
        let grouped_results = self.field_retrieval_service.query_range_schema(
            &self.atom_manager,
            &schema,
            &query.fields,
            range_filter,
        )?;

        // Convert HashMap to JSON Value
        let result = serde_json::to_value(grouped_results).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to serialize grouped results: {}", e))
        })?;

        info!(
            "✅ FoldDB::query_range_schema COMPLETE - schema: {}",
            query.schema_name
        );
        Ok(result)
    }

    pub fn query_schema(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        // Check if this is a Range schema query and route accordingly
        if let Ok(Some(schema)) = self.schema_manager.get_schema(&query.schema_name) {
            if schema.range_key().is_some() && query.filter.is_some() {
                // For Range schemas with filters, check if it's a range_filter
                if let Some(filter_obj) = query.filter.as_ref().and_then(|f| f.as_object()) {
                    if filter_obj.contains_key("range_filter") {
                        info!(
                            "🎯 Routing to Range schema query for schema: {}",
                            query.schema_name
                        );
                        // Route to Range schema query and return as single result
                        match self.query_range_schema(query) {
                            Ok(result) => return vec![Ok(result)],
                            Err(e) => return vec![Err(e)],
                        }
                    }
                }
            }
        }

        // Fall back to original field-by-field processing
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

                let result = if let Some(ref filter_value) = query.filter {
                    info!(
                        "Query processing - field: {}, has filter: true, filter: {:?}",
                        field_name, filter_value
                    );
                    self.field_manager.get_field_value_with_filter(
                        &schema,
                        field_name,
                        filter_value,
                    )
                } else {
                    info!(
                        "Query processing - field: {}, has filter: false",
                        field_name
                    );
                    self.field_manager.get_field_value(&schema, field_name)
                };

                match &result {
                    Ok(value) => {
                        info!(
                            "Query processing - field: {}, result: {:?}",
                            field_name, value
                        );
                    }
                    Err(e) => {
                        info!("Query processing - field: {}, error: {:?}", field_name, e);
                    }
                }

                result
            })
            .collect::<Vec<Result<Value, SchemaError>>>()
    }
}
