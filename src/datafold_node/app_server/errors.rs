use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use warp::reject::Reject;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AppErrorType {
    ExpiredTimestamp,
    InvalidSignature,
    UnauthorizedAccess,
    InvalidPayload,
    OperationError,
    RateLimitExceeded,
    InternalError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppError {
    pub error_type: AppErrorType,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

impl Reject for AppError {}

impl AppError {
    pub fn expired_timestamp(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::ExpiredTimestamp,
            message: message.into(),
            details: None,
        }
    }

    pub fn invalid_signature(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::InvalidSignature,
            message: message.into(),
            details: None,
        }
    }

    pub fn unauthorized_access(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::UnauthorizedAccess,
            message: message.into(),
            details: None,
        }
    }

    pub fn invalid_payload(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::InvalidPayload,
            message: message.into(),
            details: None,
        }
    }

    pub fn operation_error(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::OperationError,
            message: message.into(),
            details: None,
        }
    }

    pub fn rate_limit_exceeded(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::RateLimitExceeded,
            message: message.into(),
            details: None,
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            error_type: AppErrorType::InternalError,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Debug, Serialize)]
pub struct AppErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
}

impl AppErrorResponse {
    pub fn from_app_error(error: &AppError) -> Self {
        let (error_str, code) = match error.error_type {
            AppErrorType::ExpiredTimestamp => ("EXPIRED_TIMESTAMP", "AUTH_ERROR"),
            AppErrorType::InvalidSignature => ("INVALID_SIGNATURE", "AUTH_ERROR"),
            AppErrorType::UnauthorizedAccess => ("UNAUTHORIZED_ACCESS", "AUTH_ERROR"),
            AppErrorType::InvalidPayload => ("INVALID_PAYLOAD", "REQUEST_ERROR"),
            AppErrorType::OperationError => ("OPERATION_ERROR", "EXECUTION_ERROR"),
            AppErrorType::RateLimitExceeded => ("RATE_LIMIT_EXCEEDED", "THROTTLE_ERROR"),
            AppErrorType::InternalError => ("INTERNAL_ERROR", "SERVER_ERROR"),
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            error: error_str.to_string(),
            code: code.to_string(),
            message: error.message.clone(),
            timestamp,
            details: error.details.clone(),
        }
    }
}
