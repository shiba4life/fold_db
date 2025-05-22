use serde::Serialize;
use serde_json::Value;

/// Unified response type for HTTP and TCP APIs
#[derive(Serialize)]
pub struct UnifiedResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl UnifiedResponse {
    /// Create a success response with optional data
    pub fn success(data: Option<Value>) -> Self {
        Self { success: true, data, error: None }
    }

    /// Create an error response
    pub fn error<E: ToString>(msg: E) -> Self {
        Self { success: false, data: None, error: Some(msg.to_string()) }
    }
}
