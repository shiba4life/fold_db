use clap::{Parser, Subcommand, ValueEnum};
use datafold::schema::SchemaHasher;
use datafold::{load_node_config, DataFoldNode, MutationType, Operation, SchemaState};
use datafold::config::crypto::{CryptoConfig, MasterKeyConfig, KeyDerivationConfig, SecurityLevel};
use datafold::datafold_node::crypto_init::{
    initialize_database_crypto, get_crypto_init_status, validate_crypto_config_for_init
};
use log::{info, warn, error};
use rpassword::read_password;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};

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

/// Crypto initialization method
#[derive(Debug, Clone, ValueEnum)]
enum CryptoMethod {
    /// Generate a random master key pair (highest security, no password recovery)
    Random,
    /// Derive master key from user passphrase (allows password recovery)
    Passphrase,
}

/// Security level enum for CLI (wrapper around the config SecurityLevel)
#[derive(Debug, Clone, ValueEnum)]
enum CliSecurityLevel {
    /// Fast parameters for interactive use
    Interactive,
    /// Balanced parameters for general use
    Balanced,
    /// High security parameters for sensitive operations
    Sensitive,
}

impl From<CliSecurityLevel> for SecurityLevel {
    fn from(cli_level: CliSecurityLevel) -> Self {
        match cli_level {
            CliSecurityLevel::Interactive => SecurityLevel::Interactive,
            CliSecurityLevel::Balanced => SecurityLevel::Balanced,
            CliSecurityLevel::Sensitive => SecurityLevel::Sensitive,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database cryptography
    CryptoInit {
        /// Crypto initialization method
        #[arg(long, value_enum, default_value = "random")]
        method: CryptoMethod,
        /// Security level for key derivation (when using passphrase)
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Force re-initialization even if crypto is already initialized
        #[arg(long)]
        force: bool,
    },
    /// Check database crypto initialization status
    CryptoStatus {},
    /// Validate crypto configuration
    CryptoValidate {
        /// Path to configuration file to validate (defaults to CLI config)
        #[arg(long)]
        config_file: Option<PathBuf>,
    },
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
fn handle_crypto_init(
    method: CryptoMethod,
    security_level: CliSecurityLevel,
    force: bool,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting database crypto initialization");
    
    // Get the actual SecurityLevel from the CLI wrapper
    let security_level: SecurityLevel = security_level.into();
    
    // Check if crypto is already initialized
    let fold_db = node.get_fold_db()?;
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops.clone()).map_err(|e| format!("Failed to check crypto status: {}", e))?;
    
    if status.initialized && !force {
        info!("Database crypto is already initialized: {}", status.summary());
        if status.is_healthy() {
            info!("Crypto initialization is healthy and verified");
            return Ok(());
        } else {
            warn!("Crypto initialization exists but integrity check failed");
            info!("Use --force to re-initialize if needed");
            return Err("Crypto initialization integrity check failed".into());
        }
    } else if status.initialized && force {
        warn!("Forcing crypto re-initialization on already initialized database");
    }
    
    // Get passphrase if needed
    let passphrase = match method {
        CryptoMethod::Random => None,
        CryptoMethod::Passphrase => Some(get_secure_passphrase()?),
    };
    
    // Create crypto configuration
    let crypto_config = match method {
        CryptoMethod::Random => {
            info!("Using random key generation");
            CryptoConfig {
                enabled: true,
                master_key: MasterKeyConfig::Random,
                key_derivation: KeyDerivationConfig::for_security_level(security_level),
            }
        }
        CryptoMethod::Passphrase => {
            let passphrase = passphrase.unwrap(); // Safe since we just set it
            info!("Using passphrase-based key derivation with {} security level", security_level.as_str());
            CryptoConfig {
                enabled: true,
                master_key: MasterKeyConfig::Passphrase { passphrase },
                key_derivation: KeyDerivationConfig::for_security_level(security_level),
            }
        }
    };
    
    // Validate configuration
    validate_crypto_config_for_init(&crypto_config)
        .map_err(|e| format!("Crypto configuration validation failed: {}", e))?;
    info!("Crypto configuration validated successfully");
    
    // Perform initialization
    match initialize_database_crypto(db_ops, &crypto_config) {
        Ok(context) => {
            info!("✅ Database crypto initialization completed successfully!");
            info!("Derivation method: {}", context.derivation_method);
            info!("Master public key stored in database metadata");
            
            // Verify the initialization was successful
            let fold_db = node.get_fold_db()?;
            let final_status = get_crypto_init_status(fold_db.db_ops())
                .map_err(|e| format!("Failed to verify crypto initialization: {}", e))?;
            
            if final_status.is_healthy() {
                info!("✅ Crypto initialization verified successfully");
            } else {
                error!("❌ Crypto initialization verification failed");
                return Err("Crypto initialization verification failed".into());
            }
        }
        Err(e) => {
            error!("❌ Crypto initialization failed: {}", e);
            return Err(format!("Crypto initialization failed: {}", e).into());
        }
    }
    
    Ok(())
}

fn handle_crypto_status(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking database crypto initialization status");
    
    let fold_db = node.get_fold_db()?;
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops)
        .map_err(|e| format!("Failed to get crypto status: {}", e))?;
    
    info!("Crypto Status: {}", status.summary());
    
    if status.initialized {
        info!("  Initialized: ✅ Yes");
        info!("  Algorithm: {}", status.algorithm.as_deref().unwrap_or("Unknown"));
        info!("  Derivation Method: {}", status.derivation_method.as_deref().unwrap_or("Unknown"));
        info!("  Version: {}", status.version.unwrap_or(0));
        
        if let Some(created_at) = status.created_at {
            info!("  Created: {}", created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        match status.integrity_verified {
            Some(true) => info!("  Integrity: ✅ Verified"),
            Some(false) => warn!("  Integrity: ❌ Failed verification"),
            None => info!("  Integrity: ⚠️  Not checked"),
        }
        
        if status.is_healthy() {
            info!("🟢 Overall Status: Healthy");
        } else {
            warn!("🟡 Overall Status: Issues detected");
        }
    } else {
        info!("  Initialized: ❌ No");
        info!("🔴 Overall Status: Not initialized");
    }
    
    Ok(())
}

fn handle_crypto_validate(
    config_file: Option<PathBuf>,
    default_config_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = config_file
        .as_deref()
        .map(|p| p.to_str().unwrap_or(default_config_path))
        .unwrap_or(default_config_path);
    
    info!("Validating crypto configuration in: {}", config_path);
    
    // Load the node configuration
    let node_config = load_node_config(Some(config_path), None)?;
    
    // Check if crypto configuration exists
    if let Some(crypto_config) = &node_config.crypto {
        info!("Found crypto configuration");
        
        // Validate the configuration
        match validate_crypto_config_for_init(crypto_config) {
            Ok(()) => {
                info!("✅ Crypto configuration is valid");
                
                // Show configuration details
                info!("Configuration details:");
                info!("  Enabled: {}", crypto_config.enabled);
                
                if crypto_config.enabled {
                    match &crypto_config.master_key {
                        MasterKeyConfig::Random => {
                            info!("  Master Key: Random generation");
                        }
                        MasterKeyConfig::Passphrase { .. } => {
                            info!("  Master Key: Passphrase-based derivation");
                            
                            if let Some(preset) = &crypto_config.key_derivation.preset {
                                info!("  Security Level: {}", preset.as_str());
                            } else {
                                info!("  Key Derivation: Custom parameters");
                                info!("    Memory Cost: {} KB", crypto_config.key_derivation.memory_cost);
                                info!("    Time Cost: {} iterations", crypto_config.key_derivation.time_cost);
                                info!("    Parallelism: {} threads", crypto_config.key_derivation.parallelism);
                            }
                        }
                        MasterKeyConfig::External { key_source } => {
                            info!("  Master Key: External source ({})", key_source);
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Crypto configuration validation failed: {}", e);
                return Err(format!("Crypto configuration validation failed: {}", e).into());
            }
        }
    } else {
        info!("No crypto configuration found in node config");
        info!("ℹ️  Crypto will be disabled by default");
    }
    
    Ok(())
}

fn get_secure_passphrase() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        print!("Enter passphrase for master key derivation: ");
        io::stdout().flush()?;
        
        let passphrase = read_password()?;
        
        if passphrase.len() < 6 {
            error!("Passphrase must be at least 6 characters long");
            continue;
        }
        
        if passphrase.len() > 1024 {
            error!("Passphrase is too long (maximum 1024 characters)");
            continue;
        }
        
        // Confirm passphrase
        print!("Confirm passphrase: ");
        io::stdout().flush()?;
        
        let confirmation = read_password()?;
        
        if passphrase != confirmation {
            error!("Passphrases do not match. Please try again.");
            continue;
        }
        
        // Clear confirmation from memory
        drop(confirmation);
        
        info!("✅ Passphrase accepted");
        return Ok(passphrase);
    }
}

fn handle_load_schema(
    path: PathBuf,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading schema from: {}", path.display());
    let path_str = path.to_str().ok_or("Invalid file path")?;
    node.load_schema_from_file(path_str)?;
    info!("Schema loaded successfully");
    Ok(())
}

fn handle_add_schema(
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

fn handle_hash_schemas(verify: bool) -> Result<(), Box<dyn std::error::Error>> {
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

fn handle_list_schemas(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    let schemas = node.list_schemas()?;
    info!("Loaded schemas:");
    for schema in schemas {
        info!("  - {}", schema);
    }
    Ok(())
}

fn handle_list_available_schemas(
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let names = node.list_available_schemas()?;
    info!("Available schemas:");
    for name in names {
        info!("  - {}", name);
    }
    Ok(())
}

fn handle_unload_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.unload_schema(&name)?;
    info!("Schema '{}' unloaded", name);
    Ok(())
}

fn handle_allow_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.allow_schema(&name)?;
    info!("Schema '{}' allowed", name);
    Ok(())
}

fn handle_approve_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.approve_schema(&name)?;
    info!("Schema '{}' approved successfully", name);
    Ok(())
}

fn handle_block_schema(
    name: String,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    node.block_schema(&name)?;
    info!("Schema '{}' blocked successfully", name);
    Ok(())
}

fn handle_get_schema_state(
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

fn handle_list_schemas_by_state(
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

fn handle_execute(
    path: PathBuf,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    datafold::web_logger::init().ok();
    let cli = Cli::parse();

    // Handle commands that don't need the node first
    match &cli.command {
        Commands::HashSchemas { verify } => {
            return handle_hash_schemas(*verify);
        }
        Commands::CryptoValidate { config_file } => {
            return handle_crypto_validate(config_file.clone(), &cli.config);
        }
        _ => {}
    }

    // Load node configuration
    info!("Loading config from: {}", cli.config);
    let config = load_node_config(Some(&cli.config), None)?;

    // Initialize node
    info!("Initializing DataFold Node...");
    let mut node = DataFoldNode::load(config).await?;
    info!("Node initialized with ID: {}", node.get_node_id());

    // Process command
    match cli.command {
        Commands::LoadSchema { path } => handle_load_schema(path, &mut node)?,
        Commands::AddSchema { path, name } => handle_add_schema(path, name, &mut node)?,
        Commands::HashSchemas { .. } => unreachable!(), // Already handled above
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
        Commands::ApproveSchema { name } => handle_approve_schema(name, &mut node)?,
        Commands::BlockSchema { name } => handle_block_schema(name, &mut node)?,
        Commands::GetSchemaState { name } => handle_get_schema_state(name, &mut node)?,
        Commands::ListSchemasByState { state } => handle_list_schemas_by_state(state, &mut node)?,
        Commands::CryptoInit { method, security_level, force } => {
            handle_crypto_init(method, security_level, force, &mut node)?
        }
        Commands::CryptoStatus {} => handle_crypto_status(&mut node)?,
        Commands::CryptoValidate { .. } => unreachable!(), // Already handled above
        Commands::Execute { path } => handle_execute(path, &mut node)?,
    }

    Ok(())
}
