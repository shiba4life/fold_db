use fold_client::{FoldClient, FoldClientConfig, DockerConfig};
use std::path::PathBuf;
use std::process;
use tokio::signal;
use log::{info, error};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return;
    }

    // Load configuration
    let config = load_config().unwrap_or_else(|e| {
        error!("Failed to load configuration: {}", e);
        process::exit(1);
    });

    // Create the FoldClient
    let mut client = FoldClient::with_config(config).unwrap_or_else(|e| {
        error!("Failed to create FoldClient: {}", e);
        process::exit(1);
    });

    // Start the FoldClient
    if let Err(e) = client.start().await {
        error!("Failed to start FoldClient: {}", e);
        process::exit(1);
    }

    info!("FoldClient started successfully");

    // Wait for Ctrl+C
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutting down...");
        }
        Err(e) => {
            error!("Failed to listen for Ctrl+C: {}", e);
        }
    }

    // Stop the FoldClient
    if let Err(e) = client.stop().await {
        error!("Failed to stop FoldClient: {}", e);
        process::exit(1);
    }

    info!("FoldClient stopped successfully");
}

fn print_usage() {
    println!("FoldClient - A client for DataFold that provides Docker-sandboxed access to the node API");
    println!();
    println!("USAGE:");
    println!("    fold_client [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help    Print this help message");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    FOLD_CLIENT_NODE_SOCKET_PATH    Path to the DataFold node socket");
    println!("    FOLD_CLIENT_NODE_TCP_HOST       Host for the DataFold node TCP connection");
    println!("    FOLD_CLIENT_NODE_TCP_PORT       Port for the DataFold node TCP connection");
    println!("    FOLD_CLIENT_APP_SOCKET_DIR      Path to the directory where app sockets will be created");
    println!("    FOLD_CLIENT_APP_DATA_DIR        Path to the directory where app data will be stored");
    println!("    FOLD_CLIENT_DOCKER_HOST         Docker API URL (e.g., unix:///var/run/docker.sock)");
    println!("    FOLD_CLIENT_DOCKER_NETWORK      Docker network to use for containers");
    println!("    FOLD_CLIENT_DOCKER_CPU_LIMIT    Default CPU limit for containers (in CPU shares)");
    println!("    FOLD_CLIENT_DOCKER_MEM_LIMIT    Default memory limit for containers (in MB)");
    println!("    FOLD_CLIENT_DOCKER_STORAGE_LIMIT Default storage limit for containers (in MB)");
    println!("    FOLD_CLIENT_DOCKER_ALLOW_NETWORK Whether to enable network access for containers by default");
    println!("    FOLD_CLIENT_DOCKER_BASE_IMAGE   Base image to use for containers");
    println!("    FOLD_CLIENT_DOCKER_AUTO_REMOVE  Whether to auto-remove containers when they exit");
}

fn load_config() -> Result<FoldClientConfig, Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    let _ = dotenv::dotenv();

    // Get the home directory
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let _datafold_dir = home_dir.join(".datafold");

    // Create a default configuration
    let mut config = FoldClientConfig::default();

    // Override with environment variables
    if let Ok(socket_path) = std::env::var("FOLD_CLIENT_NODE_SOCKET_PATH") {
        config.node_socket_path = Some(socket_path);
        config.node_tcp_address = None;
    } else if let (Ok(host), Ok(port_str)) = (
        std::env::var("FOLD_CLIENT_NODE_TCP_HOST"),
        std::env::var("FOLD_CLIENT_NODE_TCP_PORT"),
    ) {
        if let Ok(port) = port_str.parse::<u16>() {
            config.node_tcp_address = Some((host, port));
            config.node_socket_path = None;
        }
    }

    if let Ok(app_socket_dir) = std::env::var("FOLD_CLIENT_APP_SOCKET_DIR") {
        config.app_socket_dir = PathBuf::from(app_socket_dir);
    }

    if let Ok(app_data_dir) = std::env::var("FOLD_CLIENT_APP_DATA_DIR") {
        config.app_data_dir = PathBuf::from(app_data_dir);
    }

    // Docker configuration
    let mut docker_config = DockerConfig::default();

    if let Ok(docker_host) = std::env::var("FOLD_CLIENT_DOCKER_HOST") {
        docker_config.docker_host = Some(docker_host);
    }

    if let Ok(network) = std::env::var("FOLD_CLIENT_DOCKER_NETWORK") {
        docker_config.network = network;
    }

    if let Ok(cpu_limit_str) = std::env::var("FOLD_CLIENT_DOCKER_CPU_LIMIT") {
        if let Ok(cpu_limit) = cpu_limit_str.parse::<u64>() {
            docker_config.default_cpu_limit = cpu_limit;
        }
    }

    if let Ok(mem_limit_str) = std::env::var("FOLD_CLIENT_DOCKER_MEM_LIMIT") {
        if let Ok(mem_limit) = mem_limit_str.parse::<u64>() {
            docker_config.default_memory_limit = mem_limit;
        }
    }

    if let Ok(storage_limit_str) = std::env::var("FOLD_CLIENT_DOCKER_STORAGE_LIMIT") {
        if let Ok(storage_limit) = storage_limit_str.parse::<u64>() {
            docker_config.default_storage_limit = storage_limit;
        }
    }

    if let Ok(allow_network_str) = std::env::var("FOLD_CLIENT_DOCKER_ALLOW_NETWORK") {
        if let Ok(allow_network) = allow_network_str.parse::<bool>() {
            docker_config.default_allow_network = allow_network;
        }
    }

    if let Ok(base_image) = std::env::var("FOLD_CLIENT_DOCKER_BASE_IMAGE") {
        docker_config.base_image = base_image;
    }

    if let Ok(auto_remove_str) = std::env::var("FOLD_CLIENT_DOCKER_AUTO_REMOVE") {
        if let Ok(auto_remove) = auto_remove_str.parse::<bool>() {
            docker_config.auto_remove = auto_remove;
        }
    }

    config.docker_config = docker_config;

    // Create the necessary directories
    std::fs::create_dir_all(&config.app_socket_dir)?;
    std::fs::create_dir_all(&config.app_data_dir)?;

    Ok(config)
}
