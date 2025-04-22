use clap::{Parser, Subcommand};
use fold_node::{load_schema_from_file, DataFoldNode, MutationType, NodeConfig, Operation};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

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

        /// Mutation type (create, update, delete)
        #[arg(short, long, required = true)]
        mutation_type: String,

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load node configuration
    println!("Loading config from: {}", cli.config);
    let config_str = fs::read_to_string(&cli.config)?;
    let config: NodeConfig = serde_json::from_str(&config_str)?;

    // Initialize node
    println!("Initializing DataFold Node...");
    let mut node = DataFoldNode::load(config)?;
    println!("Node initialized with ID: {}", node.get_node_id());

    // Process command
    match cli.command {
        Commands::LoadSchema { path } => {
            println!("Loading schema from: {}", path.display());
            load_schema_from_file(path, &mut node)?;
            println!("Schema loaded successfully");
        }
        Commands::ListSchemas {} => {
            let schemas = node.list_schemas()?;
            println!("Loaded schemas:");
            for schema in schemas {
                println!("  - {}", schema.name);
            }
        }
        Commands::Query {
            schema,
            fields,
            filter,
            output,
        } => {
            println!("Executing query on schema: {}", schema);

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
                println!("{}", result);
            } else {
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        Commands::Mutate {
            schema,
            mutation_type,
            data,
        } => {
            println!("Executing mutation on schema: {}", schema);

            let data_value: Value = serde_json::from_str(&data)?;

            let mutation_type = match mutation_type.to_lowercase().as_str() {
                "create" => MutationType::Create,
                "update" => MutationType::Update,
                "delete" => MutationType::Delete,
                s if s.starts_with("add_to_collection:") => {
                    let id = s.split(':').nth(1).unwrap_or_default().to_string();
                    MutationType::AddToCollection(id)
                },
                s if s.starts_with("update_to_collection:") => {
                    let id = s.split(':').nth(1).unwrap_or_default().to_string();
                    MutationType::UpdateToCollection(id)
                },
                s if s.starts_with("delete_from_collection:") => {
                    let id = s.split(':').nth(1).unwrap_or_default().to_string();
                    MutationType::DeleteFromCollection(id)
                },
                _ => return Err("Invalid mutation type. Use 'create', 'update', 'delete', or collection operations".into())
            };

            let operation = Operation::Mutation {
                schema,
                data: data_value,
                mutation_type,
            };

            node.execute_operation(operation)?;
            println!("Mutation executed successfully");
        }
        Commands::Execute { path } => {
            println!("Executing operation from file: {}", path.display());
            let operation_str = fs::read_to_string(path)?;
            let operation: Operation = serde_json::from_str(&operation_str)?;

            let result = node.execute_operation(operation)?;

            if !result.is_null() {
                println!("Result:");
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Operation executed successfully");
            }
        }
    }

    Ok(())
}
