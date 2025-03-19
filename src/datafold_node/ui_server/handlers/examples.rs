use std::fs;
use std::path::Path;
use warp::{Rejection, Reply};
use serde_json::Value;
use crate::datafold_node::ui_server::types::ApiSuccessResponse;
use crate::datafold_node::ui_server::errors::{UiError, UiErrorResponse};

/// Handler for listing available example files
pub async fn handle_list_examples() -> Result<impl Reply, Rejection> {
    let examples_dir = Path::new("src/datafold_node/examples");
    
    if !examples_dir.exists() || !examples_dir.is_dir() {
        let error = UiError::internal_error("Examples directory not found");
        return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
    }
    
    let entries = match fs::read_dir(examples_dir) {
        Ok(entries) => entries,
        Err(e) => {
            let error = UiError::internal_error(format!("Failed to read examples directory: {}", e));
            return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
        }
    };
    
    let mut examples = Vec::new();
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            // Only include JSON files
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                    examples.push(filename.to_string());
                }
            }
        }
    }
    
    Ok(warp::reply::json(&ApiSuccessResponse::new(examples)))
}

/// Handler for getting a specific example file
pub async fn handle_get_example(filename: String) -> Result<impl Reply, Rejection> {
    let file_path = Path::new("src/datafold_node/examples").join(&filename);
    
    if !file_path.exists() || !file_path.is_file() {
        let error = UiError::internal_error(format!("Example file '{}' not found", filename));
        return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
    }
    
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            let error = UiError::internal_error(format!("Failed to read example file: {}", e));
            return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
        }
    };
    
    // Parse the content as JSON
    let json_content: Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(e) => {
            let error = UiError::internal_error(format!("Failed to parse example file as JSON: {}", e));
            return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
        }
    };
    
    Ok(warp::reply::json(&ApiSuccessResponse::new(json_content)))
}
