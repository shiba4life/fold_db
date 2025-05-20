use clap::{Parser, Subcommand};
use fold_node::{
    load_schema_from_file, DataFoldNode, MutationType, NodeConfig, Operation,
    load_node_config,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use env_logger;
use log::{info, warn, error};

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
        Commands::LoadSchema { path } => {
            info!("Loading schema from: {}", path.display());
            load_schema_from_file(path, &mut node)?;
            info!("Schema loaded successfully");
        }
        Commands::ListSchemas {} => {
            let schemas = node.list_schemas()?;
            info!("Loaded schemas:");
            for schema in schemas {
                info!("  - {}", schema.name);
            }
        }
        Commands::Query {
            schema,
            fields,
            filter,
            output,
        } => {
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
        }
        Commands::Mutate {
            schema,
            mutation_type,
            data,
        } => {
            info!("Executing mutation on schema: {}", schema);

            let data_value: Value = serde_json::from_str(&data)?;

            let operation = Operation::Mutation {
                schema,
                data: data_value,
                mutation_type,
            };

            node.execute_operation(operation)?;
            info!("Mutation executed successfully");
        }
        Commands::Execute { path } => {
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
        }
    }

    Ok(())
}
