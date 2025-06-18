//! Schema and query command definitions
//! 
//! This module contains all CLI commands related to schema management,
//! query operations, and mutation operations.

use clap::Subcommand;
use std::path::PathBuf;
use crate::MutationType;

/// Schema and query-related CLI commands
#[derive(Subcommand, Debug)]
pub enum SchemaCommands {
    /// Load a schema from a JSON file
    LoadSchema {
        /// Path to the schema JSON file
        #[arg(required = true)]
        path: PathBuf,
    },
    /// Add a new schema to the available_schemas directory
    AddSchema {
        /// Path to the schema JSON file to add
        #[arg(required = true)]
        path: PathBuf,
        /// Optional custom name for the schema (defaults to filename)
        #[arg(long, short)]
        name: Option<String>,
    },
    /// Hash all schemas in the available_schemas directory
    HashSchemas {
        /// Verify existing hashes instead of updating them
        #[arg(long, short)]
        verify: bool,
    },
    /// List all loaded schemas
    ListSchemas {},
    /// List all schemas available on disk
    ListAvailableSchemas {},
    /// Unload a schema
    UnloadSchema {
        /// Schema name to unload
        #[arg(long, short, required = true)]
        name: String,
    },
    /// Allow operations on a schema (loads it if unloaded)
    AllowSchema {
        /// Schema name to allow
        #[arg(long, short, required = true)]
        name: String,
    },
    /// Approve a schema for queries and mutations
    ApproveSchema {
        /// Schema name to approve
        #[arg(long, short, required = true)]
        name: String,
    },
    /// Block a schema from queries and mutations
    BlockSchema {
        /// Schema name to block
        #[arg(long, short, required = true)]
        name: String,
    },
    /// Get the current state of a schema
    GetSchemaState {
        /// Schema name to check
        #[arg(long, short, required = true)]
        name: String,
    },
    /// List schemas by state
    ListSchemasByState {
        /// State to filter by (available, approved, blocked)
        #[arg(long, short, required = true)]
        state: String,
    },
    /// Execute a query operation
    Query {
        /// Schema name to query
        #[arg(short, long, required = true)]
        schema: String,

        /// Fields to retrieve (comma-separated)
        #[arg(short, long, required = true, value_delimiter = ',')]
        fields: Vec<String>,

        /// Optional filter in JSON format
        #[arg(short = 'i', long)]
        filter: Option<String>,

        /// Output format (json or pretty)
        #[arg(short, long, default_value = "pretty")]
        output: String,
    },
    /// Execute a mutation operation
    Mutate {
        /// Schema name to mutate
        #[arg(short, long, required = true)]
        schema: String,

        /// Mutation type
        #[arg(short, long, required = true, value_enum)]
        mutation_type: MutationType,

        /// Data in JSON format
        #[arg(short, long, required = true)]
        data: String,
    },
    /// Load an operation from a JSON file
    Execute {
        /// Path to the operation JSON file
        #[arg(required = true)]
        path: PathBuf,
    },
}