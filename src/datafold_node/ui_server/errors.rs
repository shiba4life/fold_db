use serde::{Deserialize, Serialize};
use warp::reject::Reject;

#[derive(Debug, Serialize, Deserialize)]
pub enum UiErrorType {
    InvalidRequest,
    NetworkError,
    SchemaError,
    DatabaseError,
    InternalError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiError {
    pub error_type: UiErrorType,
    pub message: String,
}

impl Reject for UiError {}

impl UiError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            error_type: UiErrorType::InvalidRequest,
            message: message.into(),
        }
    }

    pub fn network_error(message: impl Into<String>) -> Self {
        Self {
            error_type: UiErrorType::NetworkError,
            message: message.into(),
        }
    }

    pub fn schema_error(message: impl Into<String>) -> Self {
        Self {
            error_type: UiErrorType::SchemaError,
            message: message.into(),
        }
    }

    pub fn database_error(message: impl Into<String>) -> Self {
        Self {
            error_type: UiErrorType::DatabaseError,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            error_type: UiErrorType::InternalError,
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UiErrorResponse {
    pub error: String,
    pub message: String,
}

impl UiErrorResponse {
    pub fn from_ui_error(error: &UiError) -> Self {
        let error_type = match error.error_type {
            UiErrorType::InvalidRequest => "INVALID_REQUEST",
            UiErrorType::NetworkError => "NETWORK_ERROR",
            UiErrorType::SchemaError => "SCHEMA_ERROR",
            UiErrorType::DatabaseError => "DATABASE_ERROR",
            UiErrorType::InternalError => "INTERNAL_ERROR",
        };

        Self {
            error: error_type.to_string(),
            message: error.message.clone(),
        }
    }
}
