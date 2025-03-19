use clap::{Parser, Subcommand};
use fold_db::{
    DataFoldNode, NodeConfig, FoldDbResult, FoldDbError,
    schema::types::{Query, Operation, MutationType},
    datafold_node::network::NetworkConfig,
};
use serde_json::Value;
use std::{fs, path::PathBuf};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "fold_cli")]
#[command(about = "Command line interface for FoldDB", long_about = None)]
struct Cli {
    /// Optional config file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new node
    Init {
        /// Storage path for the node
        #[arg(short, long)]
        storage_path: PathBuf,
    },
    
    /// Load a schema from a file
    LoadSchema {
        /// Path to the schema file
        #[arg(short, long)]
        file: PathBuf,
    },
    
    /// List all loaded schemas
    ListSchemas,
    
    /// Execute a query
    Query {
        /// Schema name
        #[arg(short, long)]
        schema: String,
        
        /// Fields to query (comma-separated)
        #[arg(short = 'F', long)]
        fields: Option<String>,
        
        /// Filter expression (optional)
        #[arg(short, long)]
        filter: Option<String>,
        
        /// JSON file containing the query
        #[arg(short = 'j', long)]
        json_file: Option<PathBuf>,
    },
    
    /// Execute a mutation
    Mutate {
        /// Schema name
        #[arg(short, long)]
        schema: String,
        
        /// Mutation type (create, update, delete)
        #[arg(short, long)]
        mutation_type: String,
        
        /// JSON data for the mutation
        #[arg(short, long)]
        data: Option<String>,
        
        /// JSON file containing the mutation data
        #[arg(short = 'j', long)]
        json_file: Option<PathBuf>,
    },
    
    /// Network operations
    Network {
        #[command(subcommand)]
        command: NetworkCommands,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Initialize the network
    Init {
        /// Listen address
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        listen_addr: String,
    },
    
    /// Start the network
    Start,
    
    /// Stop the network
    Stop,
    
    /// Discover nodes on the network
    Discover,
    
    /// Connect to a node
    Connect {
        /// Node ID to connect to
        #[arg(short, long)]
        node_id: String,
    },
    
    /// List connected nodes
    ListConnected,
    
    /// List known nodes
    ListKnown,
    
    /// Query a remote node
    QueryNode {
        /// Node ID to query
        #[arg(short, long)]
        node_id: String,
        
        /// Schema name
        #[arg(short, long)]
        schema: String,
        
        /// Fields to query (comma-separated)
        #[arg(short, long)]
        fields: String,
    },
    
    /// List schemas on a remote node
    ListNodeSchemas {
        /// Node ID to query
        #[arg(short, long)]
        node_id: String,
    },
}

fn load_config(config_path: Option<PathBuf>) -> FoldDbResult<NodeConfig> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("config/node_config.json"));
    
    println!("Loading config from: {}", config_path.display());
    
    // Check if config file exists
    if !config_path.exists() {
        return Err(FoldDbError::Config(format!(
            "Config file not found: {}. Run 'fold_cli init' first to create a configuration.",
            config_path.display()
        )));
    }
    
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| FoldDbError::Config(format!("Failed to read config file: {}", e)))?;
    
    let config: NodeConfig = serde_json::from_str(&config_str)
        .map_err(|e| FoldDbError::Config(format!("Failed to parse config file: {}", e)))?;
    
    println!("Config loaded successfully");
    
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cli_config = cli.config.clone();
    
    match &cli.command {
        Commands::Init { storage_path } => {
            println!("Initializing node with storage path: {}", storage_path.display());
            
            // Create storage directory if it doesn't exist
            println!("Creating storage directory: {}", storage_path.display());
            fs::create_dir_all(storage_path)?;
            
            // Create a basic config
            let config = NodeConfig {
                storage_path: storage_path.clone(),
                default_trust_distance: 1,
            };
            
            // Create the node
            let node = DataFoldNode::new(config.clone())?;
            
            println!("Node initialized successfully with ID: {}", node.get_node_id());
            
            // Save the config to a file
            let config_dir = PathBuf::from("config");
            fs::create_dir_all(&config_dir)?;
            
            let config_path = config_dir.join("node_config.json");
            let config_json = serde_json::to_string_pretty(&config)?;
            fs::write(&config_path, config_json)?;
            
            println!("Config saved to: {}", config_path.display());
        },
        
        Commands::LoadSchema { file } => {
            println!("Loading schema from file: {}", file.display());
            
            // Check if file exists
            if !file.exists() {
                return Err(Box::new(FoldDbError::Config(format!("Schema file not found: {}", file.display()))));
            }
            
            // Load the config
            let config = load_config(cli_config.clone())?;
            
            // Create the node
            let mut node = DataFoldNode::load(config)?;
            
            // Load the schema from file
            fold_db::datafold_node::loader::load_schema_from_file(file, &mut node)?;
            
            println!("Schema loaded successfully");
        },
        
        Commands::ListSchemas => {
            // Load the config
            let config = load_config(cli_config.clone())?;
            
            // Create the node
            let node = DataFoldNode::load(config)?;
            
            // List schemas
            let schemas = node.list_schemas()?;
            
            println!("Loaded schemas:");
            for schema in schemas {
                println!("  - {} (fields: {})", schema.name, schema.fields.len());
            }
        },
        
        Commands::Query { schema, fields, filter, json_file } => {
            // Load the config
            let config = load_config(cli_config.clone())?;
            
            // Create the node
            let mut node = DataFoldNode::load(config)?;
            
            // Determine if we're using JSON file or command line args
            let operation = if let Some(json_path) = json_file {
                // Check if file exists
                if !json_path.exists() {
                    return Err(Box::new(FoldDbError::Config(format!("Query JSON file not found: {}", json_path.display()))));
                }
                
                // Read and parse the JSON file
                println!("Reading query from file: {}", json_path.display());
                let json_str = fs::read_to_string(&json_path)
                    .map_err(|e| FoldDbError::Config(format!("Failed to read query file: {}", e)))?;
                
                let query_json: serde_json::Value = serde_json::from_str(&json_str)
                    .map_err(|e| FoldDbError::Config(format!("Failed to parse query JSON: {}", e)))?;
                
                // Extract fields from JSON
                let fields = match query_json.get("fields") {
                    Some(Value::Array(arr)) => {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    },
                    _ => return Err(Box::new(FoldDbError::Config("Query JSON must contain a 'fields' array".to_string())))
                };
                
                // Extract filter from JSON (optional)
                let filter = query_json.get("filter").cloned();
                
                // Create the operation
                Operation::Query {
                    schema: schema.clone(),
                    fields,
                    filter,
                }
            } else {
                // Use command line arguments
                if let Some(field_str) = fields {
                    println!("Parsing fields: {}", field_str);
                    let field_vec: Vec<String> = field_str.split(',').map(|s| s.trim().to_string()).collect();
                    println!("Parsed fields: {:?}", field_vec);
                    
                    // Create the operation
                    Operation::Query {
                        schema: schema.clone(),
                        fields: field_vec,
                        filter: filter.as_ref().map(|f| serde_json::Value::String(f.clone())),
                    }
                } else {
                    return Err(Box::new(FoldDbError::Config("Either --fields or --json-file must be provided".to_string())));
                }
            };
            
            
            // Execute the operation
            let result = node.execute_operation(operation)?;
            
            println!("Query result:");
            println!("{}", serde_json::to_string_pretty(&result)?);
        },
        
        Commands::Mutate { schema, mutation_type, data, json_file } => {
            // Load the config
            let config = load_config(cli_config.clone())?;
            
            // Create the node
            let mut node = DataFoldNode::load(config)?;
            
            // Parse mutation type
            let mutation_type = match mutation_type.to_lowercase().as_str() {
                "create" => MutationType::Create,
                "update" => MutationType::Update,
                "delete" => MutationType::Delete,
                _ => return Err(Box::new(FoldDbError::Config(format!("Invalid mutation type: {}", mutation_type)))),
            };
            
            // Determine if we're using JSON file or command line args
            let data_value = if let Some(json_path) = json_file {
                // Check if file exists
                if !json_path.exists() {
                    return Err(Box::new(FoldDbError::Config(format!("Mutation JSON file not found: {}", json_path.display()))));
                }
                
                // Read and parse the JSON file
                println!("Reading mutation data from file: {}", json_path.display());
                let json_str = fs::read_to_string(&json_path)
                    .map_err(|e| FoldDbError::Config(format!("Failed to read mutation file: {}", e)))?;
                
                serde_json::from_str(&json_str)
                    .map_err(|e| FoldDbError::Config(format!("Failed to parse mutation JSON: {}", e)))?
            } else if let Some(data_str) = data {
                // Parse data from command line
                serde_json::from_str(data_str)
                    .map_err(|e| FoldDbError::Config(format!("Failed to parse data JSON: {}", e)))?
            } else {
                return Err(Box::new(FoldDbError::Config("Either --data or --json-file must be provided".to_string())));
            };
            
            // Create the operation
            let operation = Operation::Mutation {
                schema: schema.clone(),
                data: data_value,
                mutation_type,
            };
            
            // Execute the operation
            let result = node.execute_operation(operation)?;
            
            println!("Mutation executed successfully");
            if !result.is_null() {
                println!("Result: {}", serde_json::to_string_pretty(&result)?);
            }
        },
        
        Commands::Network { command } => {
            // Load the config
            let config = load_config(cli_config.clone())?;
            
            // Create the node
            let mut node = DataFoldNode::load(config)?;
            
            match command {
                NetworkCommands::Init { listen_addr } => {
                    println!("Initializing network with listen address: {}", listen_addr);
                    
                    // Parse the listen address
                    let listen_address: SocketAddr = listen_addr.parse()
                        .map_err(|e| FoldDbError::Config(format!("Invalid listen address: {}", e)))?;
                    
                    // Create network config
                    let network_config = NetworkConfig {
                        listen_address,
                        discovery_port: listen_address.port() + 1,
                        max_connections: 50,
                        connection_timeout: Duration::from_secs(10),
                        enable_discovery: true,
                    };
                    
                    // Initialize the network
                    node.init_network(network_config)?;
                    
                    println!("Network initialized successfully");
                },
                
                NetworkCommands::Start => {
                    println!("Starting network");
                    
                    // Start the network
                    node.start_network()?;
                    
                    println!("Network started successfully");
                },
                
                NetworkCommands::Stop => {
                    println!("Stopping network");
                    
                    // Stop the network
                    node.stop_network()?;
                    
                    println!("Network stopped successfully");
                },
                
                NetworkCommands::Discover => {
                    println!("Discovering nodes");
                    
                    // Discover nodes
                    let nodes = node.discover_nodes()?;
                    
                    println!("Discovered nodes:");
                    for node_info in nodes {
                        println!("  - {} ({})", node_info.node_id, node_info.address);
                    }
                },
                
                NetworkCommands::Connect { node_id } => {
                    println!("Connecting to node: {}", node_id);
                    
                    // Connect to the node
                    node.connect_to_node(node_id)?;
                    
                    println!("Connected successfully");
                },
                
                NetworkCommands::ListConnected => {
                    // List connected nodes
                    let connected_nodes = node.get_connected_nodes()?;
                    
                    println!("Connected nodes:");
                    for node_id in connected_nodes {
                        println!("  - {}", node_id);
                    }
                },
                
                NetworkCommands::ListKnown => {
                    // List known nodes
                    let known_nodes = node.get_known_nodes()?;
                    
                    println!("Known nodes:");
                    for (node_id, node_info) in known_nodes {
                        println!("  - {} ({})", node_id, node_info.address);
                    }
                },
                
                NetworkCommands::QueryNode { node_id, schema, fields } => {
                    println!("Querying node {} for schema {}", node_id, schema);
                    
                    // Parse fields
                    let fields: Vec<String> = fields.split(',').map(|s| s.trim().to_string()).collect();
                    
                    // Create the query
                    let query = Query {
                        schema_name: schema.clone(),
                        fields,
                        pub_key: String::new(),
                        trust_distance: 1,
                    };
                    
                    // Query the node
                    let results = node.query_node(node_id, query)?;
                    
                    println!("Query results:");
                    for (i, result) in results.iter().enumerate() {
                        match result {
                            Ok(value) => println!("  {}. {}", i + 1, serde_json::to_string_pretty(value)?),
                            Err(e) => println!("  {}. Error: {}", i + 1, e),
                        }
                    }
                },
                
                NetworkCommands::ListNodeSchemas { node_id } => {
                    println!("Listing schemas on node: {}", node_id);
                    
                    // List schemas on the node
                    let schemas = node.list_node_schemas(node_id)?;
                    
                    println!("Available schemas:");
                    for schema in schemas {
                        println!("  - {} (version: {})", schema.name, schema.version);
                        if let Some(desc) = schema.description {
                            println!("    Description: {}", desc);
                        }
                    }
                },
            }
        },
    }
    
    Ok(())
}
