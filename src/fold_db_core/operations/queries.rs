//! Query processing logic
//!
//! This module handles query processing with range schema support,
//! query optimization and execution, and result formatting.

use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use crate::schema::core::SchemaCore;
use crate::schema::types::Query;
use crate::schema::{Schema, SchemaError};
use log::info;
use serde_json::Value;
use std::sync::Arc;

/// Query operations coordinator
pub struct QueryOperations {
    schema_manager: Arc<SchemaCore>,
    permission_wrapper: PermissionWrapper,
    db_ops: Arc<DbOperations>,
}

impl QueryOperations {
    /// Create a new query operations coordinator
    pub fn new(
        schema_manager: Arc<SchemaCore>,
        permission_wrapper: PermissionWrapper,
        db_ops: Arc<DbOperations>,
    ) -> Self {
        Self {
            schema_manager,
            permission_wrapper,
            db_ops,
        }
    }

    /// Query multiple fields from a schema
    pub fn query(&self, query: Query) -> Result<Value, SchemaError> {
        use log::info;

        info!("🔍 EVENT-DRIVEN query for schema: {}", query.schema_name);

        // Get schema first
        let schema = match self.schema_manager.get_schema(&query.schema_name)? {
            Some(schema) => schema,
            None => {
                return Err(SchemaError::NotFound(format!(
                    "Schema '{}' not found",
                    query.schema_name
                )));
            }
        };

        // Check field-level permissions for each field in the query
        for field_name in &query.fields {
            let permission_result = self.permission_wrapper.check_query_field_permission(
                &query,
                field_name,
                &self.schema_manager,
            );

            if !permission_result.allowed {
                return Err(permission_result.error.unwrap_or_else(|| {
                    SchemaError::InvalidData(format!(
                        "Permission denied for field '{}' in schema '{}' with trust distance {}",
                        field_name, query.schema_name, query.trust_distance
                    ))
                }));
            }
        }

        // Extract range key filter if this is a range schema with a filter
        let range_key_filter = self.extract_range_key_filter(&schema, &query);

        // Retrieve actual field values by accessing database directly
        let mut field_values = serde_json::Map::new();

        for field_name in &query.fields {
            match self.get_field_value_from_db(&schema, field_name, range_key_filter.clone()) {
                Ok(value) => {
                    field_values.insert(field_name.clone(), value);
                }
                Err(e) => {
                    info!("Failed to retrieve field '{}': {}", field_name, e);
                    field_values.insert(field_name.clone(), serde_json::Value::Null);
                }
            }
        }

        // Return actual field values
        Ok(serde_json::Value::Object(field_values))
    }

    /// Query a Range schema and return grouped results by range_key
    pub fn query_range_schema(&self, _query: Query) -> Result<Value, SchemaError> {
        // CONVERTED TO EVENT-DRIVEN: Use SchemaLoadRequest instead of direct schema_manager access
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven SchemaLoadRequest via message bus instead of direct schema_manager access".to_string()
        ))
    }

    /// Query a schema (compatibility method)
    pub fn query_schema(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        // Delegate to the main query method and wrap in Vec
        vec![self.query(query)]
    }

    /// Get field value directly from database using unified resolver
    fn get_field_value_from_db(
        &self,
        schema: &Schema,
        field_name: &str,
        range_key_filter: Option<String>,
    ) -> Result<Value, SchemaError> {
        // Use the unified FieldValueResolver to eliminate duplicate code
        crate::fold_db_core::transform_manager::utils::TransformUtils::resolve_field_value(
            &self.db_ops,
            schema,
            field_name,
            range_key_filter,
        )
    }

    /// Extract range key filter from query for range schemas
    fn extract_range_key_filter(&self, schema: &Schema, query: &Query) -> Option<String> {
        if let (Some(range_key), Some(filter)) = (schema.range_key(), &query.filter) {
            if let Some(range_filter_obj) = filter.get("range_filter") {
                if let Some(range_filter_map) = range_filter_obj.as_object() {
                    if let Some(range_key_value) = range_filter_map.get(range_key) {
                        // Extract the actual filter value - handle different filter types
                        let extracted_value = if let Some(obj) = range_key_value.as_object() {
                            // Handle complex filters like {"Key": "1"}, {"KeyPrefix": "abc"}, etc.
                            if let Some(key_value) = obj.get("Key") {
                                Some(key_value.as_str().unwrap_or("").to_string())
                            } else if let Some(prefix_value) = obj.get("KeyPrefix") {
                                Some(prefix_value.as_str().unwrap_or("").to_string())
                            } else if let Some(pattern_value) = obj.get("KeyPattern") {
                                Some(pattern_value.as_str().unwrap_or("").to_string())
                            } else {
                                // For other filter types, try to extract any string value
                                obj.values().find_map(|v| v.as_str()).map(|s| s.to_string())
                            }
                        } else {
                            // Simple string filter like "1"
                            Some(range_key_value.to_string().trim_matches('"').to_string())
                        };

                        info!(
                            "🎯 RANGE FILTER EXTRACTED: range_key='{}', filter_value={:?}",
                            range_key, extracted_value
                        );
                        return extracted_value;
                    }
                }
            }
        }
        None
    }

    /// Check permissions for query operation
    pub fn check_query_permissions(&self, query: &Query) -> Result<(), SchemaError> {
        for field_name in &query.fields {
            let permission_result = self.permission_wrapper.check_query_field_permission(
                query,
                field_name,
                &self.schema_manager,
            );

            if !permission_result.allowed {
                return Err(permission_result.error.unwrap_or_else(|| {
                    SchemaError::InvalidData(format!(
                        "Permission denied for field '{}' in schema '{}' with trust distance {}",
                        field_name, query.schema_name, query.trust_distance
                    ))
                }));
            }
        }
        Ok(())
    }

    /// Validate query against schema constraints
    pub fn validate_query(&self, query: &Query) -> Result<(), SchemaError> {
        // Get schema first
        let schema = match self.schema_manager.get_schema(&query.schema_name)? {
            Some(schema) => schema,
            None => {
                return Err(SchemaError::NotFound(format!(
                    "Schema '{}' not found",
                    query.schema_name
                )));
            }
        };

        // Validate that all requested fields exist in the schema
        for field_name in &query.fields {
            if !schema.fields.contains_key(field_name) {
                return Err(SchemaError::InvalidData(format!(
                    "Field '{}' not found in schema '{}'",
                    field_name, schema.name
                )));
            }
        }

        // Validate range key filter if this is a range schema
        if let Some(range_key) = schema.range_key() {
            if let Some(filter) = &query.filter {
                if let Some(range_filter) = filter.get("range_filter") {
                    if let Some(range_filter_map) = range_filter.as_object() {
                        if !range_filter_map.contains_key(range_key) {
                            return Err(SchemaError::InvalidData(format!(
                                "Range schema query missing filter for range_key '{}'",
                                range_key
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Optimize query execution plan
    pub fn optimize_query(&self, query: &Query) -> Result<Query, SchemaError> {
        // For now, return the query as-is
        // Future optimizations could include:
        // - Field access order optimization
        // - Index utilization
        // - Query result caching
        Ok(query.clone())
    }

    /// Format query results according to requested format
    pub fn format_results(
        &self,
        results: Value,
        format: Option<&str>,
    ) -> Result<Value, SchemaError> {
        match format {
            Some("compact") => {
                // Remove null values for compact format
                if let Value::Object(mut map) = results {
                    map.retain(|_, v| !v.is_null());
                    Ok(Value::Object(map))
                } else {
                    Ok(results)
                }
            }
            Some("full") | None => Ok(results),
            Some(unknown) => Err(SchemaError::InvalidData(format!(
                "Unknown result format: {}",
                unknown
            ))),
        }
    }
}