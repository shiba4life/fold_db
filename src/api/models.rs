use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum QueryItem {
    Field {
        schema: String,
        field: String,
    },
    Collection {
        schema: String,
        collection: String,
        sort: Option<String>,      // "asc" or "desc"
        sort_field: Option<String>,// e.g., "created_at"
        limit: Option<usize>,
    },
}

#[derive(Deserialize, Serialize)]
pub struct QueryPayload {
    pub queries: Vec<QueryItem>,
    pub public_key: String,
    pub distance: Option<u32>,
}

#[derive(Serialize)]
pub struct QueryResult {
    pub query: JsonValue,
    pub result: JsonValue,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum WriteItem {
    WriteField {
        schema: String,
        field: String,
        value: JsonValue,
    },
    WriteCollection {
        schema: String,
        collection: String,
        item: JsonValue,
    },
}

#[derive(Deserialize, Serialize)]
pub struct WritePayload {
    pub writes: Vec<WriteItem>,
    pub public_key: String,
    pub distance: Option<u32>,
}

#[derive(Serialize)]
pub struct WriteResult {
    pub write: JsonValue,
    pub status: String,
}

#[derive(Serialize)]
pub struct WriteResponse {
    pub results: Vec<WriteResult>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
