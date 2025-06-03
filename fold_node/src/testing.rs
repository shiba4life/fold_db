// Re-export types needed for testing
pub use crate::schema::types::field::FieldType;
pub use crate::schema::types::schema::default_schema_type;
pub use crate::schema::types::{CollectionField, Field, FieldVariant, RangeField, SingleField};
pub use crate::schema::types::{Mutation, MutationType, Operation, Query, Transform};
pub use crate::schema::Schema;
pub use crate::schema::SchemaCore;
pub use crate::schema::SchemaError;
pub use crate::schema::SchemaValidator;

pub use crate::transform::parser::TransformParser;

pub use crate::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
pub use crate::permissions::PermissionWrapper;

pub use crate::fees::{FieldPaymentConfig, SchemaPaymentConfig, TrustDistanceScaling};

pub use crate::atom::{
    Atom, AtomRef, AtomRefBehavior, AtomRefCollection, AtomRefRange, AtomStatus,
};

use serde_json::Value;
use std::collections::HashMap;

pub fn create_test_schema(name: &str) -> Schema {
    Schema::new(name.to_string())
}

pub fn create_test_value(value: &str) -> Value {
    serde_json::from_str(value).unwrap()
}

pub fn create_test_fields() -> HashMap<String, Value> {
    let mut fields = HashMap::new();
    fields.insert(
        "test_field".to_string(),
        Value::String("test_value".to_string()),
    );
    fields
}
