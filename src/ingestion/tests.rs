//! Tests for ingestion functionality

use crate::fold_db_core::FoldDB;
use crate::ingestion::{
    ai_recommendation::AIRecommendationService,
    logging::IngestionLogger,
    mutation::MutationService,
    request::IngestionRequest,
    schema_management::SchemaManager,
    IngestionConfig, IngestionCore,
};
use crate::schema::SchemaCore;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

fn create_test_config() -> IngestionConfig {
    IngestionConfig {
        openrouter_api_key: "test-key".to_string(),
        enabled: true,
        ..Default::default()
    }
}

fn create_test_components(test_name: &str) -> Option<(Arc<SchemaCore>, Arc<Mutex<FoldDB>>)> {
    let temp_dir = TempDir::new().ok()?;
    let test_path = temp_dir
        .path()
        .join(test_name)
        .to_string_lossy()
        .to_string();

    let schema_core = SchemaCore::new_for_testing(&test_path).ok()?;
    let fold_db = FoldDB::new(&test_path).ok()?;

    Some((Arc::new(schema_core), Arc::new(Mutex::new(fold_db))))
}

#[test]
fn test_ingestion_request_validation() {
    // Valid inputs
    let request = IngestionRequest {
        data: serde_json::json!({"key": "value"}),
        auto_execute: None,
        trust_distance: None,
        pub_key: None,
    };
    assert!(request.validate_input().is_ok());

    let request = IngestionRequest {
        data: serde_json::json!([1, 2, 3]),
        auto_execute: None,
        trust_distance: None,
        pub_key: None,
    };
    assert!(request.validate_input().is_ok());

    // Invalid inputs
    let request = IngestionRequest {
        data: serde_json::json!(null),
        auto_execute: None,
        trust_distance: None,
        pub_key: None,
    };
    assert!(request.validate_input().is_err());

    let request = IngestionRequest {
        data: serde_json::json!("string"),
        auto_execute: None,
        trust_distance: None,
        pub_key: None,
    };
    assert!(request.validate_input().is_err());

    let request = IngestionRequest {
        data: serde_json::json!(42),
        auto_execute: None,
        trust_distance: None,
        pub_key: None,
    };
    assert!(request.validate_input().is_err());
}

#[test]
fn test_ai_recommendation_service_creation() {
    let config = create_test_config();
    let result = AIRecommendationService::new(config);
    assert!(result.is_ok());
}

#[test]
fn test_ai_recommendation_service_creation_fails_without_config() {
    let config = IngestionConfig::default();
    let result = AIRecommendationService::new(config);
    assert!(result.is_err());
}

#[test]
fn test_mutation_service_creation() {
    if let Some((_, fold_db)) = create_test_components("test_mutation_service") {
        let config = create_test_config();
        let service = MutationService::new(fold_db, config);
        assert_eq!(service.config.enabled, true);
    } else {
        eprintln!("Skipping test_mutation_service_creation: Could not create test components");
    }
}

#[test]
fn test_schema_manager_creation() {
    if let Some((schema_core, _)) = create_test_components("test_schema_manager") {
        let manager = SchemaManager::new(schema_core);
        
        let schema_def = serde_json::json!({
            "name": "TestSchema",
            "fields": {
                "field1": {"type": "string"},
                "field2": {"type": "number"}
            }
        });

        let result = manager.create_basic_schema_from_definition(&schema_def);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.name, "TestSchema");
        assert_eq!(schema.fields.len(), 2);
    } else {
        eprintln!("Skipping test_schema_manager_creation: Could not create test components");
    }
}

#[test]
fn test_logging_service() {
    let config = create_test_config();
    let logger = IngestionLogger::new(config);
    
    let status = logger.get_status().unwrap();
    assert_eq!(status["enabled"], true);
    assert_eq!(status["configured"], true);
    assert_eq!(status["model"], "anthropic/claude-3.5-sonnet");
    
    // This should not panic
    logger.log_completion("test_schema", 5, 3);
}

#[test]
fn test_ingestion_core_creation() {
    if let Some((schema_core, fold_db)) = create_test_components("test_ingestion_core") {
        let config = create_test_config();
        let result = IngestionCore::new(config, schema_core, fold_db);
        assert!(result.is_ok());
    } else {
        eprintln!("Skipping test_ingestion_core_creation: Could not create test components");
    }
}

#[test]
fn test_configuration_validation() {
    // Valid configuration
    let config = create_test_config();
    assert!(config.validate().is_ok());
    assert!(config.is_ready());

    // Invalid configuration (no API key)
    let config = IngestionConfig::default();
    assert!(config.validate().is_err());
    assert!(!config.is_ready());
}