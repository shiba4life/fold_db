//! Logging and status reporting for ingestion operations

use crate::ingestion::{IngestionConfig, IngestionResult};
use log::info;
use serde_json::Value;

pub struct IngestionLogger {
    config: IngestionConfig,
}

impl IngestionLogger {
    pub fn new(config: IngestionConfig) -> Self {
        Self { config }
    }

    /// Logs the completion of the ingestion process.
    pub fn log_completion(&self, schema_name: &str, mutations_count: usize, mutations_executed: usize) {
        info!(
            "Ingestion completed successfully: schema '{}', {} mutations generated, {} executed",
            schema_name, mutations_count, mutations_executed
        );
    }

    /// Get ingestion status
    pub fn get_status(&self) -> IngestionResult<Value> {
        Ok(serde_json::json!({
            "enabled": self.config.enabled,
            "configured": self.config.is_ready(),
            "model": self.config.openrouter_model,
            "auto_execute_mutations": self.config.auto_execute_mutations,
            "default_trust_distance": self.config.default_trust_distance
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_service_creation() {
        let config = IngestionConfig {
            openrouter_api_key: "test-key".to_string(),
            enabled: true,
            ..Default::default()
        };

        let logger = IngestionLogger::new(config);
        let status = logger.get_status().unwrap();
        
        assert_eq!(status["enabled"], true);
        assert_eq!(status["configured"], true);
        assert_eq!(status["model"], "anthropic/claude-3.5-sonnet");
    }

    #[test]
    fn test_log_completion() {
        let config = IngestionConfig {
            openrouter_api_key: "test-key".to_string(),
            enabled: true,
            ..Default::default()
        };

        let logger = IngestionLogger::new(config);
        
        // This should not panic
        logger.log_completion("test_schema", 5, 3);
    }
}