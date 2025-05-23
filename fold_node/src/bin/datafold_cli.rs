use clap::{Parser, Subcommand};
use fold_node::{
    load_schema_from_file, load_node_config, DataFoldNode, MutationType, NodeConfig,
    Operation, Fold,
};
use fold_node::schema::types::JsonFoldDefinition;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use env_logger;
use log::{error, info, warn};

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
    /// Load a fold from a JSON file
    LoadFold {
        /// Path to the fold JSON file
        #[arg(required = true)]
        path: PathBuf,
    },
    /// List all loaded folds
    ListFolds {},
    /// List all folds available on disk
    ListAvailableFolds {},
    /// Get a fold by name
    GetFold {
        /// Fold name to retrieve
        #[arg(long, short, required = true)]
        name: String,
    },
    /// Unload a fold
    UnloadFold {
        /// Fold name to unload
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

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn parse_load_fold() {
        let cli = Cli::parse_from(["test", "load-fold", "foo.json"]);
        match cli.command {
            Commands::LoadFold { path } => {
                assert_eq!(path, PathBuf::from("foo.json"));
            }
            _ => panic!("expected LoadFold"),
        }
    }

    #[test]
    fn parse_list_folds() {
        let cli = Cli::parse_from(["test", "list-folds"]);
        matches!(cli.command, Commands::ListFolds {});
    }

    #[test]
    fn parse_list_available_folds() {
        let cli = Cli::parse_from(["test", "list-available-folds"]);
        matches!(cli.command, Commands::ListAvailableFolds {});
    }

    #[test]
    fn parse_get_fold() {
        let cli = Cli::parse_from(["test", "get-fold", "--name", "foo"]);
        match cli.command {
            Commands::GetFold { name } => assert_eq!(name, "foo"),
            _ => panic!("expected GetFold"),
        }
    }
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

fn handle_load_fold(path: PathBuf, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading fold from: {}", path.display());
    let json = fs::read_to_string(path)?;
    let fold: Fold = match serde_json::from_str(&json) {
        Ok(f) => f,
        Err(_) => {
            let def: JsonFoldDefinition = serde_json::from_str(&json)?;
            Fold::try_from(def)?
        }
    };
    node.load_fold(fold)?;
    info!("Fold loaded successfully");
    Ok(())
}

fn handle_list_folds(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let folds = node.list_folds()?;
    info!("Loaded folds:");
    for name in folds {
        info!("  - {}", name);
    }
    Ok(())
}

fn handle_list_available_folds(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let names = node.list_available_folds()?;
    info!("Available folds:");
    for name in names {
        info!("  - {}", name);
    }
    Ok(())
}

fn handle_get_fold(name: String, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    match node.get_fold(&name)? {
        Some(fold) => info!("{}", serde_json::to_string_pretty(&fold)?),
        None => warn!("Fold '{}' not found", name),
    }
    Ok(())
}

fn handle_unload_fold(name: String, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    node.unload_fold(&name)?;
    info!("Fold '{}' unloaded", name);
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
///   * `load-fold <PATH>` - Load a fold from a JSON file
///   * `list-folds` - List all loaded folds
///   * `get-fold --name <NAME>` - Get a fold by name
///   * `unload-fold --name <NAME>` - Unload a fold
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
        Commands::LoadFold { path } => handle_load_fold(path, &mut node)?,
        Commands::ListFolds {} => handle_list_folds(&mut node)?,
        Commands::ListAvailableFolds {} => handle_list_available_folds(&mut node)?,
        Commands::GetFold { name } => handle_get_fold(name, &mut node)?,
        Commands::UnloadFold { name } => handle_unload_fold(name, &mut node)?,
        Commands::Execute { path } => handle_execute(path, &mut node)?,
    }

    Ok(())
}
