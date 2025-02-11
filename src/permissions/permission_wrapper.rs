use crate::schema::types::{Query, Mutation, SchemaError};
use crate::permissions::permission_manager::PermissionManager;
use crate::schema::schema_manager::SchemaManager;

#[derive(Default)]
pub struct PermissionWrapper {
    permission_manager: PermissionManager,
}

#[derive(Debug, Clone)]
pub struct FieldPermissionResult {
    pub field_name: String,
    pub allowed: bool,
    pub error: Option<SchemaError>,
}

impl PermissionWrapper {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_query_field_permission(
        &self,
        query: &Query,
        field_name: &str,
        schema_manager: &SchemaManager,
    ) -> FieldPermissionResult {
        let schema = match schema_manager.get_schema(&query.schema_name) {
            Ok(Some(s)) => s,
            Ok(None) => return FieldPermissionResult {
                field_name: field_name.to_string(),
                allowed: false,
                error: Some(SchemaError::NotFound(format!("Schema {} not found", query.schema_name))),
            },
            Err(e) => return FieldPermissionResult {
                field_name: field_name.to_string(),
                allowed: false,
                error: Some(e),
            },
        };

        schema.fields.get(field_name).map_or_else(
            || FieldPermissionResult {
                field_name: field_name.to_string(),
                allowed: false,
                error: Some(SchemaError::InvalidField(
                    format!("Field {field_name} not found")
                ))
            },
            |field| {
                let allowed = self.permission_manager.has_read_permission(
                    &query.pub_key,
                    &field.permission_policy,
                    query.trust_distance
                );

                FieldPermissionResult {
                    field_name: field_name.to_string(),
                    allowed,
                    error: if allowed {
                        None
                    } else {
                        Some(SchemaError::InvalidPermission(
                            format!("Read access denied for field {field_name}")
                        ))
                    }
                }
            }
        )
    }

    pub fn check_mutation_field_permission(
        &self,
        mutation: &Mutation,
        field_name: &str,
        schema_manager: &SchemaManager,
    ) -> FieldPermissionResult {
        let schema = match schema_manager.get_schema(&mutation.schema_name) {
            Ok(Some(s)) => s,
            Ok(None) => return FieldPermissionResult {
                field_name: field_name.to_string(),
                allowed: false,
                error: Some(SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name))),
            },
            Err(e) => return FieldPermissionResult {
                field_name: field_name.to_string(),
                allowed: false,
                error: Some(e),
            },
        };

        schema.fields.get(field_name).map_or_else(
            || FieldPermissionResult {
                field_name: field_name.to_string(),
                allowed: false,
                error: Some(SchemaError::InvalidField(
                    format!("Field {field_name} not found")
                ))
            },
            |field| {
                let allowed = self.permission_manager.has_write_permission(
                    &mutation.pub_key,
                    &field.permission_policy,
                    mutation.trust_distance
                );

                FieldPermissionResult {
                    field_name: field_name.to_string(),
                    allowed,
                    error: if allowed {
                        None
                    } else {
                        Some(SchemaError::InvalidPermission(
                            format!("Write access denied for field {field_name}")
                        ))
                    }
                }
            }
        )
    }
}
