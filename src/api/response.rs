use serde_json::{json, Value as JsonValue};
use crate::api::models::{QueryResult, WriteResult};

/// Common trait for building API responses
pub trait ResponseBuilder {
    type Result;
    
    fn schema_not_loaded(&mut self, context: JsonValue);
    fn permission_denied(&mut self, context: JsonValue);
    fn operation_error(&mut self, context: JsonValue, error: String);
    fn operation_success(&mut self, context: JsonValue, result: JsonValue);
}

/// Builder for query responses
pub struct QueryResponseBuilder {
    results: Vec<QueryResult>,
}

impl QueryResponseBuilder {
    pub fn new() -> Self {
        QueryResponseBuilder {
            results: Vec::new(),
        }
    }

    pub fn build(self) -> Vec<QueryResult> {
        self.results
    }
}

impl ResponseBuilder for QueryResponseBuilder {
    type Result = QueryResult;

    fn schema_not_loaded(&mut self, context: JsonValue) {
        self.results.push(QueryResult {
            query: context,
            result: json!({"error": "schema not loaded"}),
        });
    }

    fn permission_denied(&mut self, context: JsonValue) {
        self.results.push(QueryResult {
            query: context,
            result: json!({"error": "permission denied"}),
        });
    }

    fn operation_error(&mut self, context: JsonValue, error: String) {
        self.results.push(QueryResult {
            query: context,
            result: json!({"error": error}),
        });
    }

    fn operation_success(&mut self, context: JsonValue, result: JsonValue) {
        self.results.push(QueryResult {
            query: context,
            result,
        });
    }
}

/// Builder for write responses
pub struct WriteResponseBuilder {
    results: Vec<WriteResult>,
}

impl WriteResponseBuilder {
    pub fn new() -> Self {
        WriteResponseBuilder {
            results: Vec::new(),
        }
    }

    pub fn build(self) -> Vec<WriteResult> {
        self.results
    }
}

impl ResponseBuilder for WriteResponseBuilder {
    type Result = WriteResult;

    fn schema_not_loaded(&mut self, context: JsonValue) {
        self.results.push(WriteResult {
            write: context,
            status: "schema not loaded".to_string(),
        });
    }

    fn permission_denied(&mut self, context: JsonValue) {
        self.results.push(WriteResult {
            write: context,
            status: "permission denied".to_string(),
        });
    }

    fn operation_error(&mut self, context: JsonValue, error: String) {
        self.results.push(WriteResult {
            write: context,
            status: format!("error: {}", error),
        });
    }

    fn operation_success(&mut self, context: JsonValue, _result: JsonValue) {
        self.results.push(WriteResult {
            write: context,
            status: "ok".to_string(),
        });
    }
}
