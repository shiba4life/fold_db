//! Core ingestion orchestrator

use crate::fold_db_core::FoldDB;
use crate::ingestion::{
    ai_recommendation::AIRecommendationService,
    logging::IngestionLogger,
    mutation::MutationService,
    request::IngestionRequest,
    schema_management::SchemaManager,
    IngestionConfig, IngestionError, IngestionResponse, IngestionResult,
};
use crate::schema::SchemaCore;
use log::info;
use serde_json::Value;
use std::sync::Arc;

/// Core ingestion service that orchestrates the entire ingestion process
pub struct IngestionCore {
    config: IngestionConfig,
    ai_service: AIRecommendationService,
    schema_manager: SchemaManager,
    mutation_service: MutationService,
    logger: IngestionLogger,
}

impl IngestionCore {
    /// Create a new ingestion core
    pub fn new(
        config: IngestionConfig,
        schema_core: Arc<SchemaCore>,
        fold_db: Arc<std::sync::Mutex<FoldDB>>,
    ) -> IngestionResult<Self> {
        let ai_service = AIRecommendationService::new(config.clone())?;
        let schema_manager = SchemaManager::new(schema_core);
        let mutation_service = MutationService::new(fold_db, config.clone());
        let logger = IngestionLogger::new(config.clone());

        Ok(Self {
            config,
            ai_service,
            schema_manager,
            mutation_service,
            logger,
        })
    }

    /// Process JSON ingestion request
    pub async fn process_json_ingestion(
        &self,
        request: IngestionRequest,
    ) -> IngestionResult<IngestionResponse> {
        info!("Starting JSON ingestion process");

        // Step 1: Validate configuration
        self.validate_configuration()?;

        // Step 2: Validate input
        request.validate_input()?;

        // Step 3: Prepare schemas
        let available_schemas = self.schema_manager.prepare_schemas().await?;

        // Step 4: Get AI recommendation
        let ai_response = self
            .ai_service
            .get_ai_recommendation(&request.data, &available_schemas)
            .await?;

        // Step 5: Determine and setup schema
        let (schema_name, new_schema_created) = self.schema_manager.setup_schema(&ai_response).await?;

        // Step 6: Generate mutations
        let mutations = self.mutation_service.generate_mutations(&schema_name, &request, &ai_response)?;

        // Step 7: Execute mutations if requested
        let mutations_executed = self
            .mutation_service
            .execute_mutations_if_requested(&request, &mutations[..])
            .await?;

        self.logger.log_completion(&schema_name, mutations.len(), mutations_executed);

        Ok(IngestionResponse::success(
            schema_name,
            new_schema_created,
            mutations.len(),
            mutations_executed,
        ))
    }

    /// Validates that the ingestion configuration is ready.
    fn validate_configuration(&self) -> IngestionResult<()> {
        if !self.config.is_ready() {
            return Err(IngestionError::configuration_error(
                "Ingestion module is not properly configured or disabled",
            ));
        }
        Ok(())
    }

    /// Get ingestion status
    pub fn get_status(&self) -> IngestionResult<Value> {
        self.logger.get_status()
    }

    /// Validate JSON input
    pub fn validate_input(&self, data: &Value) -> IngestionResult<()> {
        let request = IngestionRequest {
            data: data.clone(),
            auto_execute: None,
            trust_distance: None,
            pub_key: None,
        };
        request.validate_input()
    }
}
