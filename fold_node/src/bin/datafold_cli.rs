use clap::{Parser, Subcommand};
use fold_node::{
    load_schema_from_file, load_node_config, DataFoldNode, MutationType,
    Operation,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use log::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the node configuration file
    #[arg(short, long, default_value = "config/node_config.json")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load a schema from a JSON file
    LoadSchema {
        /// Path to the schema JSON file
        #[arg(required = true)]
        path: PathBuf,
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

fn handle_load_schema(path: PathBuf, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading schema from: {}", path.display());
    load_schema_from_file(path, node)?;
    info!("Schema loaded successfully");
    Ok(())
}

fn handle_list_schemas(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let schemas = node.list_schemas()?;
    info!("Loaded schemas:");
    for schema in schemas {
        info!("  - {}", schema.name);
    }
    Ok(())
}

fn handle_list_available_schemas(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let names = node.list_available_schemas()?;
    info!("Available schemas:");
    for name in names {
        info!("  - {}", name);
    }
    Ok(())
}

fn handle_unload_schema(name: String, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    node.unload_schema(&name)?;
    info!("Schema '{}' unloaded", name);
    Ok(())
}

fn handle_allow_schema(name: String, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    node.allow_schema(&name)?;
    info!("Schema '{}' allowed", name);
    Ok(())
}

fn handle_query(
    node: &mut DataFoldNode,
    schema: String,
    fields: Vec<String>,
    filter: Option<String>,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing query on schema: {}", schema);

    let filter_value = if let Some(filter_str) = filter {
        Some(serde_json::from_str(&filter_str)?)
    } else {
        None
    };

    let operation = Operation::Query {
        schema,
        fields,
        filter: filter_value,
    };

    let result = node.execute_operation(operation)?;

    if output == "json" {
        info!("{}", result);
    } else {
        info!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

fn handle_mutate(
    node: &mut DataFoldNode,
    schema: String,
    mutation_type: MutationType,
    data: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing mutation on schema: {}", schema);

    let data_value: Value = serde_json::from_str(&data)?;

    let operation = Operation::Mutation {
        schema,
        data: data_value,
        mutation_type,
    };

    node.execute_operation(operation)?;
    info!("Mutation executed successfully");

    Ok(())
}

fn handle_execute(path: PathBuf, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing operation from file: {}", path.display());
    let operation_str = fs::read_to_string(path)?;
    let operation: Operation = serde_json::from_str(&operation_str)?;

    let result = node.execute_operation(operation)?;

    if !result.is_null() {
        info!("Result:");
        info!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        info!("Operation executed successfully");
    }

    Ok(())
}

/// Main entry point for the DataFold CLI.
///
/// This function parses command-line arguments, initializes a DataFold node,
/// and executes the requested command. It supports various operations such as
/// loading schemas, listing schemas, executing queries and mutations, and more.
///
/// # Command-Line Arguments
///
/// * `-c, --config <PATH>` - Path to the node configuration file (default: config/node_config.json)
/// * Subcommands:
///   * `load-schema <PATH>` - Load a schema from a JSON file
///   * `list-schemas` - List all loaded schemas
///   * `list-available-schemas` - List schemas stored on disk
///   * `unload-schema --name <NAME>` - Unload a schema
///   * `query` - Execute a query operation
///   * `mutate` - Execute a mutation operation
///   * `execute <PATH>` - Load an operation from a JSON file
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Errors
///
/// Returns an error if:
/// * The configuration file cannot be read or parsed
/// * The node cannot be initialized
/// * There is an error executing the requested command
fn main() -> Result<(), Box<dyn std::error::Error>> {
    fold_node::web_logger::init().ok();
    let cli = Cli::parse();

    // Load node configuration
    info!("Loading config from: {}", cli.config);
    let config = load_node_config(Some(&cli.config), None);

    // Initialize node
    info!("Initializing DataFold Node...");
    let mut node = DataFoldNode::load(config)?;
    info!("Node initialized with ID: {}", node.get_node_id());

    // Process command
    match cli.command {
        Commands::LoadSchema { path } => handle_load_schema(path, &mut node)?,
        Commands::ListSchemas {} => handle_list_schemas(&mut node)?,
        Commands::ListAvailableSchemas {} => handle_list_available_schemas(&mut node)?,
        Commands::AllowSchema { name } => handle_allow_schema(name, &mut node)?,
        Commands::Query {
            schema,
            fields,
            filter,
            output,
        } => handle_query(&mut node, schema, fields, filter, output)?,
        Commands::Mutate {
            schema,
            mutation_type,
            data,
        } => handle_mutate(&mut node, schema, mutation_type, data)?,
        Commands::UnloadSchema { name } => handle_unload_schema(name, &mut node)?,
        Commands::Execute { path } => handle_execute(path, &mut node)?,
    }

    Ok(())
}
