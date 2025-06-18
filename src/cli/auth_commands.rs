//! Authentication command definitions
//! 
//! This module contains all CLI commands related to authentication,
//! server integration, and profile management.

use clap::Subcommand;
use std::path::PathBuf;
use super::cli_types::{CliSecurityLevel, HttpMethod, MessageEncoding, CliSigningMode};

/// Authentication-related CLI commands
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Register public key with DataFold server
    RegisterKey {
        /// DataFold server URL
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Key identifier to register (must exist in storage)
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Client ID for registration (optional, will be generated if not provided)
        #[arg(long)]
        client_id: Option<String>,
        /// User ID for registration (optional)
        #[arg(long)]
        user_id: Option<String>,
        /// Human-readable key name (optional)
        #[arg(long)]
        key_name: Option<String>,
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        /// Number of retry attempts on failure
        #[arg(long, default_value = "3")]
        retries: u32,
    },
    /// Check public key registration status on server
    CheckRegistration {
        /// DataFold server URL
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Client ID to check
        #[arg(long, required = true)]
        client_id: String,
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        /// Number of retry attempts on failure
        #[arg(long, default_value = "3")]
        retries: u32,
    },
    /// Sign a message and verify with server
    SignAndVerify {
        /// DataFold server URL
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Key identifier to use for signing (must exist in storage)
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Client ID registered on server
        #[arg(long, required = true)]
        client_id: String,
        /// Message to sign (string)
        #[arg(long)]
        message: Option<String>,
        /// File containing message to sign
        #[arg(long)]
        message_file: Option<PathBuf>,
        /// Message encoding for server verification
        #[arg(long, value_enum, default_value = "utf8")]
        message_encoding: MessageEncoding,
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        /// Number of retry attempts on failure
        #[arg(long, default_value = "3")]
        retries: u32,
    },
    /// Test end-to-end workflow: generate key, register, sign, and verify
    TestServerIntegration {
        /// DataFold server URL
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Test key identifier
        #[arg(long, default_value = "test_integration_key")]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Test message to sign and verify
        #[arg(long, default_value = "Hello, DataFold server integration test!")]
        test_message: String,
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        /// Number of retry attempts on failure
        #[arg(long, default_value = "3")]
        retries: u32,
        /// Security level for key generation
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Cleanup test key after completion
        #[arg(long)]
        cleanup: bool,
    },
    /// Initialize CLI authentication configuration with unified config support
    AuthInit {
        /// Server URL for authentication
        #[arg(long, default_value = "http://localhost:8080")]
        server_url: String,
        /// Profile name for this configuration
        #[arg(long, default_value = "default")]
        profile: String,
        /// Key identifier to use for authentication (must exist in storage)
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// User ID for registration (optional)
        #[arg(long)]
        user_id: Option<String>,
        /// Environment to configure (dev/staging/prod)
        #[arg(long)]
        environment: Option<String>,
        /// Force overwrite existing profile
        #[arg(long)]
        force: bool,
    },
    /// Show CLI authentication status with unified configuration
    AuthStatus {
        /// Show detailed status information
        #[arg(long)]
        verbose: bool,
        /// Profile name to check (defaults to current profile)
        #[arg(long)]
        profile: Option<String>,
        /// Environment to check (dev/staging/prod)
        #[arg(long)]
        environment: Option<String>,
    },
    /// Manage authentication profiles
    AuthProfile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Generate a new key pair specifically for CLI authentication
    AuthKeygen {
        /// Key identifier for the new key
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Security level for key encryption
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
        /// Automatically register the key with the server
        #[arg(long)]
        auto_register: bool,
        /// Server URL for auto-registration
        #[arg(long)]
        server_url: Option<String>,
        /// Passphrase for key encryption (if not provided, will prompt)
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Test authenticated request to server
    AuthTest {
        /// Server endpoint to test (defaults to /api/test)
        #[arg(long, default_value = "/api/test")]
        endpoint: String,
        /// Profile name to use for authentication
        #[arg(long)]
        profile: Option<String>,
        /// HTTP method to use for test
        #[arg(long, value_enum, default_value = "get")]
        method: HttpMethod,
        /// Test payload for POST/PUT requests
        #[arg(long)]
        payload: Option<String>,
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Configure automatic signature injection
    AuthConfigure {
        /// Enable or disable automatic signing globally
        #[arg(long)]
        enable_auto_sign: Option<bool>,
        /// Set default signing mode (auto, manual)
        #[arg(long, value_enum)]
        default_mode: Option<CliSigningMode>,
        /// Set signing mode for a specific command
        #[arg(long)]
        command: Option<String>,
        /// Signing mode to set for the command
        #[arg(long, value_enum, requires = "command")]
        command_mode: Option<CliSigningMode>,
        /// Remove command-specific signing override
        #[arg(long)]
        remove_command_override: Option<String>,
        /// Enable or disable debug logging for signatures
        #[arg(long)]
        debug: Option<bool>,
        /// Set environment variable for signing preference
        #[arg(long)]
        env_var: Option<String>,
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
    /// Setup authentication with interactive configuration
    AuthSetup {
        /// Create default configuration file
        #[arg(long)]
        create_config: bool,
        /// Server URL for default profile
        #[arg(long)]
        server_url: Option<String>,
        /// Interactive setup mode
        #[arg(long)]
        interactive: bool,
    },
}

/// Profile management actions
#[derive(Subcommand, Debug, Clone)]
pub enum ProfileAction {
    /// Create a new authentication profile
    Create {
        /// Profile name
        #[arg(required = true)]
        name: String,
        /// Server URL
        #[arg(long, required = true)]
        server_url: String,
        /// Key identifier
        #[arg(long, required = true)]
        key_id: String,
        /// User ID (optional)
        #[arg(long)]
        user_id: Option<String>,
        /// Set as default profile
        #[arg(long)]
        set_default: bool,
    },
    /// List all authentication profiles
    List {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,
    },
    /// Show details of a specific profile
    Show {
        /// Profile name
        #[arg(required = true)]
        name: String,
    },
    /// Update an existing profile
    Update {
        /// Profile name
        #[arg(required = true)]
        name: String,
        /// New server URL
        #[arg(long)]
        server_url: Option<String>,
        /// New key identifier
        #[arg(long)]
        key_id: Option<String>,
        /// New user ID
        #[arg(long)]
        user_id: Option<String>,
    },
    /// Delete a profile
    Delete {
        /// Profile name
        #[arg(required = true)]
        name: String,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Set default profile
    SetDefault {
        /// Profile name
        #[arg(required = true)]
        name: String,
    },
}

/// Environment management actions
#[derive(Subcommand, Debug, Clone)]
pub enum EnvironmentAction {
    /// List all available environments
    List {},
    /// Show current environment information
    Show {
        /// Specific environment to show (defaults to current)
        #[arg(long)]
        environment: Option<String>,
    },
    /// Switch to a different environment
    Switch {
        /// Environment to switch to
        #[arg(required = true)]
        environment: String,
    },
    /// Compare two environments
    Compare {
        /// First environment
        #[arg(required = true)]
        env1: String,
        /// Second environment
        #[arg(required = true)]
        env2: String,
    },
    /// Validate all environments
    Validate {},
    /// Export environment variables for scripting
    Export {
        /// Environment to export (defaults to current)
        #[arg(long)]
        environment: Option<String>,
    },
}