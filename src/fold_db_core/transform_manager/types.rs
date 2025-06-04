use crate::atom::{Atom, AtomRef};
use crate::schema::SchemaError;
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::sync::Arc;

/// Callback function type for getting an atom by its reference UUID
pub type GetAtomFn = Arc<dyn Fn(&str) -> Result<Atom, Box<dyn std::error::Error>> + Send + Sync>;

/// Callback function type for creating a new atom
pub type CreateAtomFn = Arc<
    dyn Fn(
            &str,
            String,
            Option<String>,
            JsonValue,
            Option<crate::atom::AtomStatus>,
        ) -> Result<Atom, Box<dyn std::error::Error>>
        + Send
        + Sync,
>;

/// Callback function type for updating an atom reference
pub type UpdateAtomRefFn =
    Arc<dyn Fn(&str, String, String) -> Result<AtomRef, Box<dyn std::error::Error>> + Send + Sync>;

/// Callback function type for getting a field value by schema and field name
pub type GetFieldFn = Arc<dyn Fn(&str, &str) -> Result<JsonValue, SchemaError> + Send + Sync>;

/// Trait abstraction over transform execution for easier testing.
pub trait TransformRunner: Send + Sync {
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError>;
    fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError>;
    fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError>;
}
