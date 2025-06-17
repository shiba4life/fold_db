//! Schema management command handlers
//! 
//! This module contains handlers for all schema-related operations including
//! loading, adding, hashing, listing, and state management.

use crate::schema::SchemaHasher;
use crate::{DataFoldNode, SchemaState};
use log::info;
use std::fs;
use std::path::PathBuf;

/// Handle loading a schema from file
pub fn handle_load_schema(
    path: PathBuf,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading schema from: {}", path.display());
    let path_str = path.to_str().ok_or("Invalid file path")?;
    node.load_schema_from_file(path_str)?;
    info!("Schema loaded successfully");
    Ok(())
}

/// Handle adding a schema to the available directory
pub fn handle_add_schema(
    path: PathBuf,
    name: Option<String>,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Adding schema from: {}", path.display());

    // Read the schema file
    let schema_content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read schema file: {}", e))?;

    // Determine schema name from parameter or filename
    let custom_name = name.or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    });

    info!("Using database-level validation (always enabled)");

    // Use the database-level method which includes full validation
    let final_schema_name = node
        .add_schema_to_available_directory(&schema_content, custom_name)
        .map_err(|e| format!("Schema validation failed: {}", e))?;

    // Reload available schemas
    info!("Reloading available schemas...");
    node.refresh_schemas()
        .map_err(|e| format!("Failed to reload schemas: {}", e))?;

    info!(
        "Schema '{}' is now available for approval and use",
        final_schema_name
    );
    Ok(())
}

/// Handle hashing or verifying schema hashes
pub fn handle_hash_schemas(verify: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verify {
        info!("Verifying schema hashes in available_schemas directory...");

        match SchemaHasher::verify_available_schemas_directory() {
            Ok(results) => {
                let mut all_valid = true;
                info!("Hash verification results:");

                for (filename, is_valid) in results {
                    if is_valid {
                        info!("  ✅ {}: Valid hash", filename);
                    } else {
                        info!("  ❌ {}: Invalid or missing hash", filename);
                        all_valid = false;
                    }
                }

                if all_valid {
                    info!("All schemas have valid hashes!");
                } else {
                    info!("Some schemas have invalid or missing hashes. Run without --verify to update them.");
                }
            }
            Err(e) => {
                return Err(format!("Failed to verify schema hashes: {}", e).into());
            }
        }
    } else {
        info!("Adding/updating hashes for all schemas in available_schemas directory...");

        match SchemaHasher::hash_available_schemas_directory() {
            Ok(results) => {
                info!("Successfully processed {} schema files:", results.len());

                for (filename, hash) in results {
                    info!("  ✅ {}: {}", filename, hash);
                }

                info!("All schemas have been updated with hashes!");
            }
            Err(e) => {
                return Err(format!("Failed to hash schemas: {}", e).into());
            }
        }
    }

    Ok(())
}

/// Handle listing loaded schemas
pub fn handle_list_schemas(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let schemas = node.list_schemas()?;
    info!("Loaded schemas:");
    for schema in schemas {
        info!("  - {}", schema);
    }
    Ok(())
}

/// Handle listing available schemas
pub fn handle_list_available_schemas(
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let names = node.list_available_schemas()?;
    info!("Available schemas:");
    for name in names {
        info!("  - {}", name);
    }
    Ok(())
}

/// Handle unloading a schema
pub fn handle_unload_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.unload_schema(&name)?;
    info!("Schema '{}' unloaded", name);
    Ok(())
}

/// Handle allowing a schema
pub fn handle_allow_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.allow_schema(&name)?;
    info!("Schema '{}' allowed", name);
    Ok(())
}

/// Handle approving a schema
pub fn handle_approve_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.approve_schema(&name)?;
    info!("Schema '{}' approved successfully", name);
    Ok(())
}

/// Handle blocking a schema
pub fn handle_block_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.block_schema(&name)?;
    info!("Schema '{}' blocked successfully", name);
    Ok(())
}

/// Handle getting schema state
pub fn handle_get_schema_state(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = node.get_schema_state(&name)?;
    let state_str = match state {
        SchemaState::Available => "available",
        SchemaState::Approved => "approved",
        SchemaState::Blocked => "blocked",
    };
    info!("Schema '{}' state: {}", name, state_str);
    Ok(())
}

/// Handle listing schemas by state
pub fn handle_list_schemas_by_state(
    state: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema_state = match state.as_str() {
        "available" => SchemaState::Available,
        "approved" => SchemaState::Approved,
        "blocked" => SchemaState::Blocked,
        _ => {
            return Err(format!(
                "Invalid state: {}. Use: available, approved, or blocked",
                state
            )
            .into())
        }
    };

    let schemas = node.list_schemas_by_state(schema_state)?;
    info!("Schemas with state '{}':", state);
    for schema in schemas {
        info!("  - {}", schema);
    }
    Ok(())
}