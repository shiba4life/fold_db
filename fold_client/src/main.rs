//! FoldClient - A client for DataFold that provides sandboxed access to the node API
//!
//! This is the main executable for the FoldClient.

use clap::{Parser, Subcommand};
use fold_client::{FoldClient, FoldClientConfig, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FoldClient
    Start {
        /// Path to the DataFold node socket
        #[arg(long)]
        node_socket: Option<String>,

        /// Host for the DataFold node TCP connection
        #[arg(long)]
        node_host: Option<String>,

        /// Port for the DataFold node TCP connection
        #[arg(long)]
        node_port: Option<u16>,

        /// Path to the directory where app sockets will be created
        #[arg(long)]
        app_socket_dir: Option<PathBuf>,

        /// Path to the directory where app data will be stored
        #[arg(long)]
        app_data_dir: Option<PathBuf>,

        /// Whether to allow network access for apps
        #[arg(long)]
        allow_network: bool,

        /// Whether to allow file system access for apps
        #[arg(long)]
        allow_filesystem: bool,

        /// Maximum memory usage for apps (in MB)
        #[arg(long)]
        max_memory: Option<u64>,

        /// Maximum CPU usage for apps (in percent)
        #[arg(long)]
        max_cpu: Option<u32>,
    },
    /// Register a new app
    RegisterApp {
        /// Name of the app
        #[arg(short, long)]
        name: String,

        /// Permissions for the app (comma-separated)
        #[arg(short, long)]
        permissions: String,
    },
    /// Launch an app
    LaunchApp {
        /// ID of the app
        #[arg(short, long)]
        id: String,

        /// Path to the program to launch
        #[arg(short, long)]
        program: String,

        /// Arguments for the program
        #[arg(short, long)]
        args: Vec<String>,
    },
    /// Terminate an app
    TerminateApp {
        /// ID of the app
        #[arg(short, long)]
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Load configuration
    let mut config = if let Some(config_path) = cli.config {
        // Load configuration from file
        let config_str = std::fs::read_to_string(config_path)?;
        let config: FoldClientConfig = serde_json::from_str(&config_str)
            .map_err(|e| fold_client::FoldClientError::Config(format!("Invalid configuration: {}", e)))?;
        config
    } else {
        // Use default configuration
        FoldClientConfig::default()
    };

    // Process command
    match cli.command {
        Commands::Start {
            node_socket,
            node_host,
            node_port,
            app_socket_dir,
            app_data_dir,
            allow_network,
            allow_filesystem,
            max_memory,
            max_cpu,
        } => {
            // Update configuration with command-line options
            if let Some(node_socket) = node_socket {
                config.node_socket_path = Some(node_socket);
                config.node_tcp_address = None;
            } else if let (Some(host), Some(port)) = (node_host, node_port) {
                config.node_tcp_address = Some((host, port));
                config.node_socket_path = None;
            }

            if let Some(app_socket_dir) = app_socket_dir {
                config.app_socket_dir = app_socket_dir;
            }

            if let Some(app_data_dir) = app_data_dir {
                config.app_data_dir = app_data_dir;
            }

            config.allow_network_access = allow_network;
            config.allow_filesystem_access = allow_filesystem;

            if let Some(max_memory) = max_memory {
                config.max_memory_mb = Some(max_memory);
            }

            if let Some(max_cpu) = max_cpu {
                config.max_cpu_percent = Some(max_cpu);
            }

            // Create and start the FoldClient
            let mut client = FoldClient::with_config(config)?;
            client.start().await?;

            // Wait for Ctrl+C
            tokio::signal::ctrl_c().await?;

            // Stop the FoldClient
            client.stop().await?;
        }
        Commands::RegisterApp { name, permissions } => {
            // Create the FoldClient
            let mut client = FoldClient::with_config(config)?;

            // Parse permissions
            let permissions: Vec<&str> = permissions.split(',').collect();

            // Register the app
            client.register_app(&name, &permissions).await?;
        }
        Commands::LaunchApp { id, program, args } => {
            // Create the FoldClient
            let client = FoldClient::with_config(config)?;

            // Convert args to &[&str]
            let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

            // Launch the app
            client.launch_app(&id, &program, &args).await?;
        }
        Commands::TerminateApp { id } => {
            // Create the FoldClient
            let client = FoldClient::with_config(config)?;

            // Terminate the app
            client.terminate_app(&id).await?;
        }
    }

    Ok(())
}
