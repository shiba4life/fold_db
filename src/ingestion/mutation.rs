//! Mutation generation and execution logic

use crate::fold_db_core::FoldDB;
use crate::ingestion::{
    mutation_generator::MutationGenerator,
    openrouter_service::AISchemaResponse,
    request::IngestionRequest,
    IngestionConfig, IngestionError, IngestionResult,
};
use crate::schema::types::Mutation;
use log::{error, info};
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub struct MutationService {
    mutation_generator: MutationGenerator,
    fold_db: Arc<Mutex<FoldDB>>,
    pub config: IngestionConfig,
}

impl MutationService {
    pub fn new(fold_db: Arc<Mutex<FoldDB>>, config: IngestionConfig) -> Self {
        Self {
            mutation_generator: MutationGenerator::new(),
            fold_db,
            config,
        }
    }

    /// Generates mutations for the data using the determined schema.
    pub fn generate_mutations(
        &self,
        schema_name: &str,
        request: &IngestionRequest,
        ai_response: &AISchemaResponse,
    ) -> IngestionResult<Vec<Mutation>> {
        let mutations = self.generate_mutations_for_data(
            schema_name,
            &request.data,
            &ai_response.mutation_mappers,
            request
                .trust_distance
                .unwrap_or(self.config.default_trust_distance),
            request
                .pub_key
                .clone()
                .unwrap_or_else(|| "default".to_string()),
        )?;

        info!("Generated {} mutations", mutations.len());
        Ok(mutations)
    }

    /// Executes mutations if auto-execution is enabled.
    pub async fn execute_mutations_if_requested(
        &self,
        request: &IngestionRequest,
        mutations: &[Mutation],
    ) -> IngestionResult<usize> {
        if request
            .auto_execute
            .unwrap_or(self.config.auto_execute_mutations)
        {
            self.execute_mutations(mutations).await
        } else {
            Ok(0)
        }
    }

    /// Generate mutations for the data
    fn generate_mutations_for_data(
        &self,
        schema_name: &str,
        json_data: &Value,
        mutation_mappers: &std::collections::HashMap<String, String>,
        trust_distance: u32,
        pub_key: String,
    ) -> IngestionResult<Vec<Mutation>> {
        self.mutation_generator.generate_mutations(
            schema_name,
            json_data,
            mutation_mappers,
            trust_distance,
            pub_key,
        )
    }

    /// Execute mutations
    async fn execute_mutations(&self, mutations: &[Mutation]) -> IngestionResult<usize> {
        let mut executed_count = 0;

        for mutation in mutations {
            match self.execute_single_mutation(mutation).await {
                Ok(()) => {
                    executed_count += 1;
                    info!(
                        "Successfully executed mutation for schema '{}'",
                        mutation.schema_name
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to execute mutation for schema '{}': {}",
                        mutation.schema_name, e
                    );
                    // Continue with other mutations even if one fails
                }
            }
        }

        Ok(executed_count)
    }

    /// Execute a single mutation
    async fn execute_single_mutation(&self, mutation: &Mutation) -> IngestionResult<()> {
        let mut db = self.fold_db.lock().map_err(|_| {
            IngestionError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        db.write_schema(mutation.clone())
            .map_err(IngestionError::SchemaSystemError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold_db_core::FoldDB;
    use tempfile::TempDir;

    #[test]
    fn test_mutation_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir
            .path()
            .join("test_mutation")
            .to_string_lossy()
            .to_string();

        let config = IngestionConfig {
            openrouter_api_key: "test-key".to_string(),
            enabled: true,
            ..Default::default()
        };

        let fold_db = match FoldDB::new(&test_path) {
            Ok(db) => Arc::new(Mutex::new(db)),
            Err(_) => {
                eprintln!("Skipping test_mutation_service_creation: Could not create database");
                return;
            }
        };

        let service = MutationService::new(fold_db, config);
        assert_eq!(service.config.enabled, true);
    }
}