use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::datafold_node::app_server::errors::AppErrorType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityLogEntry {
    pub timestamp: u64,
    pub level: String,
    pub public_key: String,
    pub error_type: Option<String>,
    pub ip_address: String,
    pub request_id: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationLogEntry {
    pub timestamp: u64,
    pub level: String,
    pub operation_type: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub request_id: String,
    pub public_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppLogger {
    security_log_path: String,
    operation_log_path: String,
    security_level: LogLevel,
    operation_level: LogLevel,
    debug_level: LogLevel,
}

impl AppLogger {
    pub fn new(log_dir: &str) -> Self {
        // Create log directory if it doesn't exist
        std::fs::create_dir_all(log_dir).unwrap_or_else(|e| {
            eprintln!("Warning: Could not create log directory: {}", e);
        });

        let security_log_path = format!("{}/security.log", log_dir);
        let operation_log_path = format!("{}/operation.log", log_dir);

        Self {
            security_log_path,
            operation_log_path,
            security_level: LogLevel::Warning,
            operation_level: LogLevel::Info,
            debug_level: LogLevel::Debug,
        }
    }

    pub fn set_security_level(&mut self, level: LogLevel) {
        self.security_level = level;
    }

    pub fn set_operation_level(&mut self, level: LogLevel) {
        self.operation_level = level;
    }

    pub fn set_debug_level(&mut self, level: LogLevel) {
        self.debug_level = level;
    }

    pub fn log_security_event(
        &self,
        level: LogLevel,
        public_key: &str,
        error_type: Option<AppErrorType>,
        ip_address: &str,
        request_id: &str,
        details: HashMap<String, String>,
    ) {
        // Skip if level is below threshold
        if level < self.security_level {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = SecurityLogEntry {
            timestamp,
            level: level.as_str().to_string(),
            public_key: public_key.to_string(),
            error_type: error_type.map(|e| format!("{:?}", e)),
            ip_address: ip_address.to_string(),
            request_id: request_id.to_string(),
            details,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&entry).unwrap_or_else(|e| {
            eprintln!("Error serializing security log entry: {}", e);
            "{}".to_string()
        });

        // Write to log file
        self.write_to_log(&self.security_log_path, &json);

        // Print to console for critical errors
        if level >= LogLevel::Error {
            eprintln!("SECURITY: {}", json);
        }
    }

    pub fn log_operation(
        &self,
        level: LogLevel,
        operation_type: &str,
        duration_ms: u64,
        success: bool,
        error_type: Option<AppErrorType>,
        error_message: Option<String>,
        request_id: &str,
        public_key: Option<&str>,
    ) {
        // Skip if level is below threshold
        if level < self.operation_level {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = OperationLogEntry {
            timestamp,
            level: level.as_str().to_string(),
            operation_type: operation_type.to_string(),
            duration_ms,
            success,
            error_type: error_type.map(|e| format!("{:?}", e)),
            error_message,
            request_id: request_id.to_string(),
            public_key: public_key.map(|s| s.to_string()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&entry).unwrap_or_else(|e| {
            eprintln!("Error serializing operation log entry: {}", e);
            "{}".to_string()
        });

        // Write to log file
        self.write_to_log(&self.operation_log_path, &json);

        // Print to console for errors
        if level >= LogLevel::Error {
            eprintln!("OPERATION: {}", json);
        }
    }

    pub fn log_debug(&self, message: &str, context: HashMap<String, String>) {
        // Skip if level is below threshold
        if LogLevel::Debug < self.debug_level {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = json!({
            "timestamp": timestamp,
            "level": "DEBUG",
            "message": message,
            "context": context,
        });

        // Print to console
        println!("DEBUG: {}", entry);
    }

    fn write_to_log(&self, log_path: &str, message: &str) {
        let path = Path::new(log_path);
        let file_result = if path.exists() {
            OpenOptions::new().append(true).open(path)
        } else {
            File::create(path)
        };

        match file_result {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "{}", message) {
                    eprintln!("Error writing to log file {}: {}", log_path, e);
                }
            }
            Err(e) => {
                eprintln!("Error opening log file {}: {}", log_path, e);
            }
        }
    }
}

// Helper macro for creating context maps
#[macro_export]
macro_rules! log_context {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}
