//! AI-based schema/data recommendation logic

use crate::ingestion::{
    openrouter_service::{AISchemaResponse, OpenRouterService},
    IngestionConfig, IngestionResult,
};
use log::info;
use serde_json::Value;

pub struct AIRecommendationService {
    openrouter_service: OpenRouterService,
}

impl AIRecommendationService {
    pub fn new(config: IngestionConfig) -> IngestionResult<Self> {
        let openrouter_service = OpenRouterService::new(config)?;
        Ok(Self {
            openrouter_service,
        })
    }

    /// Gets AI recommendation for schema selection/creation.
    pub async fn get_ai_recommendation(
        &self,
        data: &Value,
        available_schemas: &Value,
    ) -> IngestionResult<AISchemaResponse> {
        let ai_response = self
            .get_ai_schema_recommendation(data, available_schemas)
            .await?;
        info!(
            "Received AI recommendation: {} existing schemas, new schema: {}",
            ai_response.existing_schemas.len(),
            ai_response.new_schemas.is_some()
        );
        Ok(ai_response)
    }

    /// Get AI schema recommendation
    async fn get_ai_schema_recommendation(
        &self,
        json_data: &Value,
        available_schemas: &Value,
    ) -> IngestionResult<AISchemaResponse> {
        self.openrouter_service
            .get_schema_recommendation(json_data, available_schemas)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_recommendation_service_creation() {
        let config = IngestionConfig {
            openrouter_api_key: "test-key".to_string(),
            enabled: true,
            ..Default::default()
        };

        let result = AIRecommendationService::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ai_recommendation_service_creation_fails_without_config() {
        let config = IngestionConfig::default();

        let result = AIRecommendationService::new(config);
        assert!(result.is_err());
    }
}