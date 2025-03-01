use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::schema::types::MutationType;

/// Represents an operation that can be performed on the database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Operation {
    #[serde(rename = "query")]
    Query {
        schema: String,
        fields: Vec<String>,
        filter: Option<Value>,
    },
    #[serde(rename = "mutation")]
    Mutation {
        schema: String,
        data: Value,
        mutation_type: MutationType,
    }
}
