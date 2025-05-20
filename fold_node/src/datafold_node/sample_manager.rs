use crate::error::{FoldDbError, FoldDbResult};
use log::error;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Sample data manager for the HTTP server.
///
/// `SampleManager` provides access to sample schemas, queries, and mutations
/// for one-click loading in the UI.
#[derive(Clone)]
pub struct SampleManager {
    /// Sample schemas
    pub(crate) schemas: HashMap<String, Value>,
    /// Sample queries
    pub(crate) queries: HashMap<String, Value>,
    /// Sample mutations
    pub(crate) mutations: HashMap<String, Value>,
}

impl SampleManager {
    /// Create a new sample manager and load samples from disk.
    pub async fn new() -> FoldDbResult<Self> {
        let mut manager = Self {
            schemas: HashMap::new(),
            queries: HashMap::new(),
            mutations: HashMap::new(),
        };

        // Load sample data
        manager.load_samples().await?;

        Ok(manager)
    }

    /// Load sample data from files.
    async fn load_samples(&mut self) -> FoldDbResult<()> {
        let samples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/datafold_node/samples/data");

        let mut entries = match fs::read_dir(&samples_dir).await {
            Ok(e) => e,
            Err(e) => {
                error!(
                    "Failed to read samples directory {}: {}",
                    samples_dir.display(),
                    e
                );
                return Err(FoldDbError::Io(e));
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(ft) = entry.file_type().await {
                if !ft.is_file() {
                    continue;
                }
            }

            if let Ok(content) = fs::read_to_string(entry.path()).await {
                if let Ok(value) = serde_json::from_str::<Value>(&content) {
                    let name = entry
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".json")
                        .to_string();

                    match value.get("type").and_then(|v| v.as_str()) {
                        Some("query") => {
                            self.queries.insert(name, value);
                        }
                        Some("mutation") => {
                            self.mutations.insert(name, value);
                        }
                        _ => {
                            self.schemas.insert(name, value);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a sample schema by name.
    pub fn get_schema_sample(&self, name: &str) -> Option<&Value> {
        self.schemas.get(name)
    }

    /// Get a sample query by name.
    pub fn get_query_sample(&self, name: &str) -> Option<&Value> {
        self.queries.get(name)
    }

    /// Get a sample mutation by name.
    pub fn get_mutation_sample(&self, name: &str) -> Option<&Value> {
        self.mutations.get(name)
    }

    /// List all sample schemas.
    pub fn list_schema_samples(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// List all sample queries.
    pub fn list_query_samples(&self) -> Vec<String> {
        self.queries.keys().cloned().collect()
    }

    /// List all sample mutations.
    pub fn list_mutation_samples(&self) -> Vec<String> {
        self.mutations.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::SampleManager;

    #[tokio::test]
    async fn sample_manager_loads_schemas() {
        let manager = SampleManager::new().await.expect("failed to load samples");
        let schemas = manager.list_schema_samples();
        assert!(schemas.contains(&"UserProfile".to_string()));
        assert!(schemas.contains(&"ProductCatalog".to_string()));
    }
}

