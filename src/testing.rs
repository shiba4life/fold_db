// Re-export types needed for testing
pub use crate::schema::Schema;
pub use crate::schema::types::{Mutation, Query, MutationType, Operation};
pub use crate::schema::types::fields::{FieldType, SchemaField};
pub use crate::schema::SchemaCore;
pub use crate::schema::SchemaError;

pub use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance, ExplicitCounts};
pub use crate::permissions::PermissionWrapper;

pub use crate::fees::{FieldPaymentConfig, SchemaPaymentConfig, TrustDistanceScaling};

pub use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomRefBehavior, AtomStatus};

use serde_json::Value;
use std::collections::HashMap;

pub fn create_test_schema(name: &str) -> Schema {
    Schema {
        name: name.to_string(),
        fields: HashMap::new(),
        payment_config: Default::default(),
    }
}

pub fn create_test_value(value: &str) -> Value {
    serde_json::from_str(value).unwrap()
}

pub fn create_test_fields() -> HashMap<String, Value> {
    let mut fields = HashMap::new();
    fields.insert("test_field".to_string(), Value::String("test_value".to_string()));
    fields
}
