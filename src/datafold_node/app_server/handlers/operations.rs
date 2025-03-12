use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use warp::{Rejection, Reply};
use serde_json::json;
use crate::datafold_node::node::DataFoldNode;
use crate::schema::types::Operation;
use crate::datafold_node::app_server::types::{SignedRequest, ApiSuccessResponse};
use crate::datafold_node::app_server::errors::{AppError, AppErrorType, AppErrorResponse};
use crate::datafold_node::app_server::logging::{AppLogger, LogLevel};

/// Handle a signed query operation
pub async fn handle_signed_operation(
    request: SignedRequest,
    public_key: String,
    client_ip: String,
    request_id: String,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    logger: AppLogger,
) -> Result<impl Reply, Rejection> {
    let start_time = Instant::now();
    
    // Parse the operation
    let operation_str = &request.payload.content;
    let operation: Operation = match serde_json::from_str(operation_str) {
        Ok(op) => op,
        Err(e) => {
            let error = AppError::invalid_payload(format!("Invalid operation format: {}", e));
            
            // Log operation error
            logger.log_operation(
                LogLevel::Warning,
                &request.payload.operation,
                0,
                false,
                Some(AppErrorType::InvalidPayload),
                Some(format!("Invalid operation format: {}", e)),
                &request_id,
                Some(&public_key),
            );
            
            return Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)));
        }
    };
    
    // Execute the operation
    let mut node = node.lock().await;
    let result = node.execute_operation(operation);
    
    // Calculate duration
    let duration = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(result) => {
            // Log successful operation
            logger.log_operation(
                LogLevel::Info,
                &request.payload.operation,
                duration,
                true,
                None,
                None,
                &request_id,
                Some(&public_key),
            );
            
            Ok(warp::reply::json(&ApiSuccessResponse::new(result, &request_id)))
        },
        Err(e) => {
            let error = AppError::operation_error(e.to_string());
            
            // Log operation error
            logger.log_operation(
                LogLevel::Warning,
                &request.payload.operation,
                duration,
                false,
                Some(AppErrorType::OperationError),
                Some(e.to_string()),
                &request_id,
                Some(&public_key),
            );
            
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Handle a request to get API status
pub async fn handle_api_status() -> Result<impl Reply, Rejection> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
        
    let status = json!({
        "status": "ok",
        "version": "1.0.0",
        "timestamp": timestamp,
    });
    
    Ok(warp::reply::json(&status))
}
