//! File/directory helpers and utility functions for schema operations
//!
//! This module contains utility functions for:
//! - Schema file discovery and directory operations
//! - Hash-based schema lookup
//! - Unified schema discovery and loading
//! - File system utilities

use crate::schema::core_types::{SchemaCore, SchemaLoadingReport, SchemaSource, SchemaState};
use crate::schema::types::{JsonSchemaDefinition, Schema, SchemaError};
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;

/// Discover schemas from the schemas directory
pub fn discover_schemas_from_files(schema_core: &SchemaCore) -> Result<Vec<Schema>, SchemaError> {
    let mut discovered_schemas = Vec::new();

    info!("Discovering schemas from {}", schema_core.schemas_dir.display());
    if let Ok(entries) = std::fs::read_dir(&schema_core.schemas_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                    if schema_opt.is_none() {
                        if let Ok(json_schema) =
                            serde_json::from_str::<JsonSchemaDefinition>(&contents)
                        {
                            if let Ok(schema) = super::parsing::interpret_schema(schema_core, json_schema) {
                                schema_opt = Some(schema);
                            }
                        }
                    }
                    if let Some(mut schema) = schema_opt {
                        super::transforms::fix_transform_outputs(schema_core, &mut schema);
                        let schema_name = schema.name.clone();
                        discovered_schemas.push(schema);
                        info!("Discovered schema '{}' from file", schema_name);
                    }
                }
            }
        }
    }

    Ok(discovered_schemas)
}

/// Discover schemas from the available_schemas directory
pub fn discover_available_schemas(schema_core: &SchemaCore) -> Result<Vec<Schema>, SchemaError> {
    info!("🔍 DEBUG: Starting discovery from available_schemas directory");
    let mut discovered_schemas = Vec::new();
    let available_schemas_dir = PathBuf::from("available_schemas");

    info!(
        "Discovering available schemas from {}",
        available_schemas_dir.display()
    );
    if let Ok(entries) = std::fs::read_dir(&available_schemas_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    let mut schema_opt = serde_json::from_str::<Schema>(&contents).ok();
                    if schema_opt.is_none() {
                        if let Ok(json_schema) =
                            serde_json::from_str::<JsonSchemaDefinition>(&contents)
                        {
                            if let Ok(schema) = super::parsing::interpret_schema(schema_core, json_schema) {
                                schema_opt = Some(schema);
                            }
                        }
                    }
                    if let Some(mut schema) = schema_opt {
                        super::transforms::fix_transform_outputs(schema_core, &mut schema);
                        let schema_name = schema.name.clone();
                        discovered_schemas.push(schema);
                        info!("Discovered available schema '{}' from file", schema_name);
                    }
                }
            }
        }
    }

    Ok(discovered_schemas)
}

/// Single entry point for all schema discovery and loading
/// Consolidates all existing discovery methods (no sample manager)
pub fn discover_and_load_all_schemas(schema_core: &SchemaCore) -> Result<SchemaLoadingReport, SchemaError> {
    info!("🔍 Starting unified schema discovery and loading");

    let mut discovered_schemas = Vec::new();
    let mut failed_schemas = Vec::new();
    let mut loading_sources = HashMap::new();

    // Get current schemas in memory to avoid unnecessary reloading
    let current_schemas = {
        let available = schema_core.available.lock().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire schema lock".to_string())
        })?;
        available
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<String>>()
    };

    // 1. FIRST: Load existing schema states from sled persistence
    info!("📋 Loading existing schema states from persistence first");
    let schema_states = schema_core.load_states();
    for schema_name in schema_states.keys() {
        loading_sources.insert(schema_name.clone(), SchemaSource::Persistence);
        info!("Loaded persisted schema state for '{}'", schema_name);
    }

    // 2. SECOND: Discover from available_schemas/ directory (without overwriting states)
    match discover_available_schemas(schema_core) {
        Ok(schemas) => {
            for schema in schemas {
                let schema_name = schema.name.clone();
                discovered_schemas.push(schema_name.clone());

                // Only update loading source if not already loaded from persistence
                if !loading_sources.contains_key(&schema_name) {
                    loading_sources
                        .insert(schema_name.clone(), SchemaSource::AvailableDirectory);
                }

                // Only load if not already in memory
                if !current_schemas.contains(&schema_name) {
                    info!(
                        "Loading new schema '{}' from available_schemas/ (preserving persisted state)",
                        schema_name
                    );
                    if let Err(e) = schema_core.load_schema_internal(schema) {
                        failed_schemas.push((schema_name, e.to_string()));
                    }
                } else {
                    info!(
                        "Schema '{}' already in memory, skipping reload",
                        schema_name
                    );
                }
            }
        }
        Err(e) => {
            info!("Failed to discover schemas from available_schemas/: {}", e);
        }
    }

    // 3. THIRD: Discover from data/schemas/ directory (without overwriting states)
    match discover_schemas_from_files(schema_core) {
        Ok(schemas) => {
            for schema in schemas {
                let schema_name = schema.name.clone();
                if !discovered_schemas.contains(&schema_name) {
                    discovered_schemas.push(schema_name.clone());

                    // Only update loading source if not already loaded from persistence
                    if !loading_sources.contains_key(&schema_name) {
                        loading_sources
                            .insert(schema_name.clone(), SchemaSource::DataDirectory);
                    }

                    // Only load if not already in memory
                    if !current_schemas.contains(&schema_name) {
                        info!("Loading new schema '{}' from data/schemas/ (preserving persisted state)", schema_name);
                        if let Err(e) = schema_core.load_schema_internal(schema) {
                            failed_schemas.push((schema_name, e.to_string()));
                        }
                    } else {
                        info!(
                            "Schema '{}' already in memory, skipping reload",
                            schema_name
                        );
                    }
                }
            }
        }
        Err(e) => {
            info!("Failed to discover schemas from data/schemas/: {}", e);
        }
    }

    // 4. Get loaded schemas (approved state)
    let loaded_schemas = schema_core
        .list_schemas_by_state(SchemaState::Approved)
        .unwrap_or_else(|_| Vec::new());

    info!(
        "✅ Schema discovery complete: {} discovered, {} loaded, {} failed",
        discovered_schemas.len(),
        loaded_schemas.len(),
        failed_schemas.len()
    );

    Ok(SchemaLoadingReport {
        discovered_schemas,
        loaded_schemas,
        failed_schemas,
        schema_states,
        loading_sources,
        last_updated: chrono::Utc::now(),
    })
}

/// Find a schema with the same hash (for duplicate detection) in the specified directory
pub fn find_schema_by_hash(
    _schema_core: &SchemaCore,
    target_hash: &str,
    exclude_name: &str,
) -> Result<Option<String>, SchemaError> {
    let available_schemas_dir = std::path::PathBuf::from("available_schemas");

    if let Ok(entries) = std::fs::read_dir(&available_schemas_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                // Skip the file we're trying to create
                if let Some(file_stem) = path.file_stem() {
                    if file_stem == exclude_name {
                        continue;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(schema_json) = serde_json::from_str::<serde_json::Value>(&content)
                    {
                        // Check if schema has a hash field
                        if let Some(existing_hash) =
                            schema_json.get("hash").and_then(|h| h.as_str())
                        {
                            if existing_hash == target_hash {
                                if let Some(name) =
                                    schema_json.get("name").and_then(|n| n.as_str())
                                {
                                    return Ok(Some(name.to_string()));
                                }
                            }
                        } else {
                            // Calculate hash for schemas without hash field
                            if let Ok(calculated_hash) =
                                super::hasher::SchemaHasher::calculate_hash(&schema_json)
                            {
                                if calculated_hash == target_hash {
                                    if let Some(name) =
                                        schema_json.get("name").and_then(|n| n.as_str())
                                    {
                                        return Ok(Some(name.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Create a directory if it doesn't exist
pub fn ensure_directory_exists(path: &std::path::Path) -> Result<(), SchemaError> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to create directory {}: {}", path.display(), e)))?;
    }
    Ok(())
}

/// Check if a file exists
pub fn file_exists(path: &std::path::Path) -> bool {
    path.exists() && path.is_file()
}

/// Get all JSON files in a directory
pub fn get_json_files_in_directory(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, SchemaError> {
    let mut json_files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                json_files.push(path);
            }
        }
    }
    
    Ok(json_files)
}

/// Read schema content from file with error handling
pub fn read_schema_file(path: &std::path::Path) -> Result<String, SchemaError> {
    std::fs::read_to_string(path)
        .map_err(|e| SchemaError::InvalidData(format!("Failed to read schema file {}: {}", path.display(), e)))
}

/// Write schema content to file with error handling
pub fn write_schema_file(path: &std::path::Path, content: &str) -> Result<(), SchemaError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_directory_exists(parent)?;
    }
    
    std::fs::write(path, content)
        .map_err(|e| SchemaError::InvalidData(format!("Failed to write schema file {}: {}", path.display(), e)))
}

impl SchemaCore {
    /// Discover schemas from the schemas directory
    pub fn discover_schemas_from_files(&self) -> Result<Vec<Schema>, SchemaError> {
        discover_schemas_from_files(self)
    }

    /// Discover schemas from the available_schemas directory
    pub fn discover_available_schemas(&self) -> Result<Vec<Schema>, SchemaError> {
        discover_available_schemas(self)
    }

    /// Single entry point for all schema discovery and loading
    pub fn discover_and_load_all_schemas(&self) -> Result<SchemaLoadingReport, SchemaError> {
        discover_and_load_all_schemas(self)
    }

    /// Find a schema with the same hash (for duplicate detection)
    pub fn find_schema_by_hash(
        &self,
        target_hash: &str,
        exclude_name: &str,
    ) -> Result<Option<String>, SchemaError> {
        find_schema_by_hash(self, target_hash, exclude_name)
    }
}