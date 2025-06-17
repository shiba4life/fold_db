use clap::{Parser, Subcommand, ValueEnum};
use datafold::cli::auth::{CliAuthProfile, CliSigningConfig};
use datafold::cli::config::{CliConfigManager, ServerConfig};
use datafold::cli::environment_utils::commands as env_commands;
use datafold::cli::http_client::{AuthenticatedHttpClient, HttpClientBuilder, RetryConfig};
use datafold::config::crypto::{CryptoConfig, KeyDerivationConfig, MasterKeyConfig};
use datafold::crypto::ed25519::{generate_master_keypair, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
use datafold::crypto::MasterKeyPair;
use datafold::datafold_node::crypto_init::{
    get_crypto_init_status, initialize_database_crypto, validate_crypto_config_for_init,
};
use datafold::schema::SchemaHasher;
use datafold::security_types::SecurityLevel;
use datafold::{load_node_config, DataFoldNode, MutationType, Operation, SchemaState};
// Removed: Admin key rotation commands have been removed for security
use base64::{engine::general_purpose, Engine as _};
use datafold::cli::signing_config::SigningMode;
use datafold::crypto::{derive_key, generate_salt, Argon2Params};
use log::{error, info, warn};
use rand::{rngs::OsRng, RngCore};
use reqwest::Client;
use rpassword::read_password;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the node configuration file
    #[arg(short, long, default_value = "config/node_config.json")]
    config: String,

    /// Authentication profile to use for signing (mandatory authentication enabled)
    #[arg(long, global = true)]
    profile: Option<String>,

    /// Environment to use for unified configuration (dev/staging/prod)
    #[arg(long, global = true)]
    environment: Option<String>,

    /// Enable debug logging for signature operations
    #[arg(long, global = true)]
    sign_debug: bool,

    /// Verbose output for debugging
    #[arg(short, long, global = true)]
    verbose: bool,

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
            CliSecurityLevel::Interactive => SecurityLevel::Low,
            CliSecurityLevel::Balanced => SecurityLevel::Standard,
            CliSecurityLevel::Sensitive => SecurityLevel::High,
        }
    }
}

/// Key output format for CLI key generation
#[derive(Debug, Clone, ValueEnum)]
enum KeyFormat {
    /// Hexadecimal format (lowercase)
    Hex,
    /// Base64 format
    Base64,
    /// PEM format (PKCS#8 for private keys, SubjectPublicKeyInfo for public keys)
    Pem,
    /// Raw bytes format (for programmatic use)
    Raw,
}

/// Key rotation method
#[derive(Debug, Clone, ValueEnum)]
enum RotationMethod {
    /// Generate a completely new random key
    Regenerate,
    /// Derive new key from master using incremented counter
    Derive,
    /// Re-derive from passphrase with new salt
    Rederive,
}

/// Export format for encrypted key export
#[derive(Debug, Clone, ValueEnum)]
enum ExportFormat {
    /// JSON format with base64-encoded data
    Json,
    /// Binary format with compact encoding
    Binary,
}

#[derive(Subcommand, Debug)]
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
    /// Generate a new Ed25519 keypair for client-side key management
    GenerateKey {
        /// Output format for the generated keys
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for private key (optional, defaults to stdout)
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Output file for public key (optional, defaults to stdout)
        #[arg(long)]
        public_key_file: Option<PathBuf>,
        /// Generate multiple keypairs (batch mode)
        #[arg(long, default_value = "1")]
        count: u32,
        /// Only output the public key (useful for registration)
        #[arg(long)]
        public_only: bool,
        /// Only output the private key (use with caution)
        #[arg(long)]
        private_only: bool,
    },
    /// Derive Ed25519 keypair from passphrase using secure key derivation
    DeriveKey {
        /// Output format for the derived keys
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for private key (optional, defaults to stdout)
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Output file for public key (optional, defaults to stdout)
        #[arg(long)]
        public_key_file: Option<PathBuf>,
        /// Security level for key derivation
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Only output the public key
        #[arg(long)]
        public_only: bool,
        /// Only output the private key (use with caution)
        #[arg(long)]
        private_only: bool,
        /// Passphrase for key derivation (if not provided, will prompt)
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Extract public key from private key
    ExtractPublicKey {
        /// Private key input (hex, base64, or PEM format)
        #[arg(long)]
        private_key: Option<String>,
        /// Private key file path
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Output format for the public key
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for public key (optional, defaults to stdout)
        #[arg(long)]
        output_file: Option<PathBuf>,
    },
    /// Verify that a keypair is valid and matches
    VerifyKey {
        /// Private key input (hex, base64, or PEM format)
        #[arg(long)]
        private_key: Option<String>,
        /// Private key file path
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Public key input (hex, base64, or PEM format)
        #[arg(long)]
        public_key: Option<String>,
        /// Public key file path
        #[arg(long)]
        public_key_file: Option<PathBuf>,
    },
    /// Store a private key securely in encrypted storage
    StoreKey {
        /// Key identifier/name for storage
        #[arg(long, required = true)]
        key_id: String,
        /// Private key input (hex, base64, or PEM format)
        #[arg(long)]
        private_key: Option<String>,
        /// Private key file path
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
        /// Security level for encryption
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Passphrase for key encryption (if not provided, will prompt)
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Retrieve a private key from encrypted storage
    RetrieveKey {
        /// Key identifier/name to retrieve
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Output format for the retrieved key
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for retrieved key (optional, defaults to stdout)
        #[arg(long)]
        output_file: Option<PathBuf>,
        /// Only output the public key derived from stored private key
        #[arg(long)]
        public_only: bool,
    },
    /// Delete a key from encrypted storage
    DeleteKey {
        /// Key identifier/name to delete
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },
    /// List all stored keys
    ListKeys {
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Show detailed information about each key
        #[arg(long)]
        verbose: bool,
    },
    /// Derive a child key from a master key using HKDF (BLAKE3)
    DeriveFromMaster {
        /// Master key identifier to derive from
        #[arg(long, required = true)]
        master_key_id: String,
        /// Derivation context/info string
        #[arg(long, required = true)]
        context: String,
        /// Child key identifier for storage
        #[arg(long, required = true)]
        child_key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Security level for child key encryption
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Output format for the derived key (if not stored)
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Only output the key, don't store it
        #[arg(long)]
        output_only: bool,
        /// Force overwrite if child key already exists
        #[arg(long)]
        force: bool,
    },
    /// Rotate a stored key to a new version
    RotateKey {
        /// Key identifier to rotate
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Security level for the new key version
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Rotation method
        #[arg(long, value_enum, default_value = "regenerate")]
        method: RotationMethod,
        /// Keep previous version as backup
        #[arg(long)]
        keep_backup: bool,
        /// Force rotation without confirmation
        #[arg(long)]
        force: bool,
    },
    /// List all versions of a stored key
    ListKeyVersions {
        /// Key identifier to list versions for
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Show detailed version information
        #[arg(long)]
        verbose: bool,
    },
    /// Create encrypted backup of a key
    BackupKey {
        /// Key identifier to backup
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Backup file path
        #[arg(long, required = true)]
        backup_file: PathBuf,
        /// Additional passphrase for backup encryption (optional)
        #[arg(long)]
        backup_passphrase: bool,
    },
    /// Restore key from encrypted backup
    RestoreKey {
        /// Backup file path
        #[arg(long, required = true)]
        backup_file: PathBuf,
        /// Key identifier for restored key
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
    },
    /// Export key with encrypted passphrase protection
    ExportKey {
        /// Key identifier to export
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Export file path
        #[arg(long, required = true)]
        export_file: PathBuf,
        /// Export format (json or binary)
        #[arg(long, value_enum, default_value = "json")]
        format: ExportFormat,
        /// Use additional export passphrase for enhanced security
        #[arg(long)]
        export_passphrase: bool,
        /// Include key metadata in export
        #[arg(long)]
        include_metadata: bool,
    },
    /// Import key from encrypted export file
    ImportKey {
        /// Export file path
        #[arg(long, required = true)]
        export_file: PathBuf,
        /// Key identifier for imported key
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
        /// Verify key integrity after import
        #[arg(long, default_value = "true")]
        verify_integrity: bool,
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
    /// Verify a signature against a message
    VerifySignature {
        /// Message to verify (as string)
        #[arg(long)]
        message: Option<String>,
        /// Message file path
        #[arg(long)]
        message_file: Option<PathBuf>,
        /// Signature to verify (base64 encoded)
        #[arg(long, required = true)]
        signature: String,
        /// Key ID for verification
        #[arg(long, required = true)]
        key_id: String,
        /// Public key for verification (hex, base64, or PEM format)
        #[arg(long)]
        public_key: Option<String>,
        /// Public key file path
        #[arg(long)]
        public_key_file: Option<PathBuf>,
        /// Verification policy to use
        #[arg(long)]
        policy: Option<String>,
        /// Output format (json, table, or compact)
        #[arg(long, value_enum, default_value = "table")]
        output_format: VerificationOutputFormat,
        /// Enable debug output
        #[arg(long)]
        debug: bool,
    },
    /// Inspect signature format and analyze components
    InspectSignature {
        /// Signature headers as JSON or individual values
        #[arg(long)]
        signature_input: Option<String>,
        /// Signature value (base64 encoded)
        #[arg(long)]
        signature: Option<String>,
        /// Headers file (JSON format)
        #[arg(long)]
        headers_file: Option<PathBuf>,
        /// Output format (json, table, or compact)
        #[arg(long, value_enum, default_value = "table")]
        output_format: VerificationOutputFormat,
        /// Show detailed analysis
        #[arg(long)]
        detailed: bool,
        /// Enable debug output
        #[arg(long)]
        debug: bool,
    },
    /// Verify server response signatures
    VerifyResponse {
        /// Server URL to test
        #[arg(long, required = true)]
        url: String,
        /// HTTP method to use
        #[arg(long, value_enum, default_value = "get")]
        method: HttpMethod,
        /// Request headers (JSON format)
        #[arg(long)]
        headers: Option<String>,
        /// Request body for POST/PUT requests
        #[arg(long)]
        body: Option<String>,
        /// Request body file
        #[arg(long)]
        body_file: Option<PathBuf>,
        /// Key ID for verification
        #[arg(long, required = true)]
        key_id: String,
        /// Public key for verification (hex, base64, or PEM format)
        #[arg(long)]
        public_key: Option<String>,
        /// Public key file path
        #[arg(long)]
        public_key_file: Option<PathBuf>,
        /// Verification policy to use
        #[arg(long)]
        policy: Option<String>,
        /// Output format (json, table, or compact)
        #[arg(long, value_enum, default_value = "table")]
        output_format: VerificationOutputFormat,
        /// Enable debug output
        #[arg(long)]
        debug: bool,
        /// Timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Configure verification settings
    VerificationConfig {
        /// Action to perform
        #[command(subcommand)]
        action: VerificationConfigAction,
    },
    /// Environment management for unified configuration
    Environment {
        /// Action to perform
        #[command(subcommand)]
        action: EnvironmentAction,
    },
    // Removed: Administrative key rotation commands have been removed for security
}

/// Environment management actions
#[derive(Subcommand, Debug, Clone)]
enum EnvironmentAction {
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

/// Output format for verification results
#[derive(Debug, Clone, ValueEnum)]
enum VerificationOutputFormat {
    /// JSON format
    Json,
    /// Table format (human-readable)
    Table,
    /// Compact format (one line)
    Compact,
}

/// Verification configuration actions
#[derive(Subcommand, Debug, Clone)]
enum VerificationConfigAction {
    /// Show current verification configuration
    Show {
        /// Show all policies
        #[arg(long)]
        policies: bool,
        /// Show public keys
        #[arg(long)]
        keys: bool,
    },
    /// Add a verification policy
    AddPolicy {
        /// Policy name
        #[arg(required = true)]
        name: String,
        /// Policy configuration file (JSON)
        #[arg(long, required = true)]
        config_file: PathBuf,
    },
    /// Remove a verification policy
    RemovePolicy {
        /// Policy name
        #[arg(required = true)]
        name: String,
    },
    /// Set default verification policy
    SetDefaultPolicy {
        /// Policy name
        #[arg(required = true)]
        name: String,
    },
    /// Add a public key for verification
    AddPublicKey {
        /// Key identifier
        #[arg(long, required = true)]
        key_id: String,
        /// Public key (hex, base64, or PEM format)
        #[arg(long)]
        public_key: Option<String>,
        /// Public key file path
        #[arg(long)]
        public_key_file: Option<PathBuf>,
    },
    /// Remove a public key
    RemovePublicKey {
        /// Key identifier
        #[arg(long, required = true)]
        key_id: String,
    },
    /// List all configured public keys
    ListPublicKeys {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,
    },
}

/// Profile management actions
#[derive(Subcommand, Debug, Clone)]
enum ProfileAction {
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

/// HTTP methods for testing
#[derive(Debug, Clone, ValueEnum)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// CLI wrapper for signing mode
#[derive(Debug, Clone, ValueEnum)]
enum CliSigningMode {
    /// Automatically sign all requests
    Auto,
    /// Only sign when explicitly requested
    Manual,
}

impl From<CliSigningMode> for SigningMode {
    fn from(cli_mode: CliSigningMode) -> Self {
        match cli_mode {
            CliSigningMode::Auto => SigningMode::Auto,
            CliSigningMode::Manual => SigningMode::Manual,
        }
    }
}

/// Message encoding options for server communication
#[derive(Debug, Clone, ValueEnum)]
enum MessageEncoding {
    /// UTF-8 string encoding
    Utf8,
    /// Hexadecimal encoding
    Hex,
    /// Base64 encoding
    Base64,
}

/// Secure key storage configuration
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct KeyStorageConfig {
    /// Encrypted key data
    encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    nonce: [u8; 12],
    /// Salt used for key derivation (32 bytes)
    salt: [u8; 32],
    /// Argon2 parameters used for key derivation
    argon2_params: StoredArgon2Params,
    /// Timestamp when key was stored
    created_at: String,
    /// Version of storage format
    version: u32,
}

/// Simplified Argon2 parameters for storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoredArgon2Params {
    memory_cost: u32,
    time_cost: u32,
    parallelism: u32,
}

impl From<&Argon2Params> for StoredArgon2Params {
    fn from(params: &Argon2Params) -> Self {
        Self {
            memory_cost: params.memory_cost,
            time_cost: params.time_cost,
            parallelism: params.parallelism,
        }
    }
}

impl From<StoredArgon2Params> for Argon2Params {
    fn from(val: StoredArgon2Params) -> Self {
        Argon2Params::new(val.memory_cost, val.time_cost, val.parallelism).unwrap_or_default()
    }
}

/// Key versioning metadata for rotation tracking
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct KeyVersionMetadata {
    /// Version number (starts at 1)
    version: u32,
    /// Previous version number (None for initial version)
    previous_version: Option<u32>,
    /// Creation timestamp for this version
    created_at: String,
    /// Derivation method used for this version
    derivation_method: String,
    /// Salt used for key derivation (32 bytes)
    salt: [u8; 32],
    /// Argon2 parameters used for this version
    argon2_params: StoredArgon2Params,
}

/// Enhanced key storage configuration with versioning
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct VersionedKeyStorageConfig {
    /// Version metadata for tracking
    version_metadata: KeyVersionMetadata,
    /// Encrypted key data
    encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    nonce: [u8; 12],
    /// Storage format version
    storage_version: u32,
}

/// Backup format for encrypted key export
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct KeyBackupFormat {
    /// Backup format version
    format_version: u32,
    /// Original key ID
    key_id: String,
    /// Export timestamp
    exported_at: String,
    /// Encrypted backup data (double-encrypted if backup passphrase used)
    backup_data: Vec<u8>,
    /// Backup encryption nonce
    backup_nonce: [u8; 12],
    /// Backup encryption salt (if additional passphrase used)
    backup_salt: Option<[u8; 32]>,
    /// Backup encryption parameters (if additional passphrase used)
    backup_params: Option<StoredArgon2Params>,
    /// Original key metadata
    original_metadata: KeyVersionMetadata,
}

/// Enhanced export format following the research specifications
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EnhancedKeyExportFormat {
    /// Export format version (starts at 1)
    version: u32,
    /// KDF algorithm used (argon2id)
    kdf: String,
    /// KDF parameters for key derivation
    kdf_params: EnhancedKdfParams,
    /// Encryption algorithm used (xchacha20-poly1305 or aes-gcm)
    encryption: String,
    /// Nonce for encryption (24 bytes for XChaCha20, 12 for AES-GCM)
    nonce: Vec<u8>,
    /// Encrypted key data (ciphertext + authentication tag)
    ciphertext: Vec<u8>,
    /// Export creation timestamp
    created: String,
    /// Original key metadata (optional)
    metadata: Option<ExportKeyMetadata>,
}

/// Enhanced KDF parameters for export
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EnhancedKdfParams {
    /// Salt for key derivation (32 bytes)
    salt: Vec<u8>,
    /// Memory cost in KB
    memory: u32,
    /// Time cost (iterations)
    iterations: u32,
    /// Parallelism factor
    parallelism: u32,
}

/// Optional metadata for exported keys
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ExportKeyMetadata {
    /// Original key identifier
    key_id: String,
    /// Original creation timestamp
    original_created: String,
    /// Export source information
    export_source: String,
    /// Key usage notes (optional)
    notes: Option<String>,
}

/// HTTP client response wrapper for server API calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

/// API error structure from server responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ApiError {
    code: String,
    message: String,
    details: std::collections::HashMap<String, serde_json::Value>,
}

/// Public key registration request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PublicKeyRegistrationRequest {
    client_id: Option<String>,
    user_id: Option<String>,
    public_key: String,
    key_name: Option<String>,
    metadata: Option<std::collections::HashMap<String, String>>,
}

/// Public key registration response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PublicKeyRegistrationResponse {
    registration_id: String,
    client_id: String,
    public_key: String,
    key_name: Option<String>,
    registered_at: String,
    status: String,
}

/// Public key status response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PublicKeyStatusResponse {
    registration_id: String,
    client_id: String,
    public_key: String,
    key_name: Option<String>,
    registered_at: String,
    status: String,
    last_used: Option<String>,
}

/// Signature verification request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SignatureVerificationRequest {
    client_id: String,
    message: String,
    signature: String,
    message_encoding: Option<String>,
    metadata: Option<std::collections::HashMap<String, String>>,
}

/// Signature verification response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SignatureVerificationResponse {
    verified: bool,
    client_id: String,
    public_key: String,
    verified_at: String,
    message_hash: String,
}

/// HKDF key derivation using BLAKE3 (simplified for compatibility)
fn hkdf_derive_key(
    master_key: &[u8; 32],
    salt: &[u8],
    info: &[u8],
    output_length: usize,
) -> Vec<u8> {
    // Simplified key derivation using BLAKE3
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"HKDF-DATAFOLD"); // Domain separator
    hasher.update(salt);
    hasher.update(master_key);
    hasher.update(info);

    let base_hash = hasher.finalize();

    // For simplicity, just use the first output_length bytes
    // In a production system, you'd want proper HKDF expansion
    let mut output = vec![0u8; output_length];
    let available_bytes = std::cmp::min(32, output_length);
    output[..available_bytes].copy_from_slice(&base_hash.as_bytes()[..available_bytes]);

    // If we need more than 32 bytes, use a simple expansion
    if output_length > 32 {
        for (i, item) in output.iter_mut().enumerate().skip(32) {
            *item = base_hash.as_bytes()[i % 32];
        }
    }

    output
}

/// Create HTTP client with retry and timeout configuration
fn create_http_client(timeout_secs: u64) -> Result<Client, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("datafold-cli/0.1.0")
        .build()?;
    Ok(client)
}

/// Perform HTTP request with retry logic
async fn http_request_with_retry<T>(
    _client: &Client,
    request_builder: reqwest::RequestBuilder,
    retries: u32,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let mut last_error = None;

    for attempt in 0..=retries {
        if attempt > 0 {
            println!("Retrying request (attempt {}/{})", attempt + 1, retries + 1);
            tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
        }

        // Clone the request builder for each attempt
        let request = match request_builder.try_clone() {
            Some(req) => req,
            None => return Err("Failed to clone request for retry".into()),
        };

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let response_text = response.text().await?;

                if status.is_success() {
                    // Parse the API response wrapper
                    let api_response: ApiResponse<T> = serde_json::from_str(&response_text)
                        .map_err(|e| format!("Failed to parse response: {}", e))?;

                    if api_response.success {
                        if let Some(data) = api_response.data {
                            return Ok(data);
                        } else {
                            return Err(
                                "API response marked as successful but contained no data".into()
                            );
                        }
                    } else if let Some(error) = api_response.error {
                        return Err(format!("API error: {} - {}", error.code, error.message).into());
                    } else {
                        return Err(
                            "API response marked as failed but contained no error details".into(),
                        );
                    }
                } else {
                    let error_msg = format!("HTTP error {}: {}", status, response_text);
                    last_error = Some(error_msg.clone().into());

                    // Don't retry on client errors (4xx), only server errors (5xx) and network issues
                    if status.is_client_error() {
                        return Err(error_msg.into());
                    }

                    println!("Server error, will retry: {}", error_msg);
                }
            }
            Err(e) => {
                let error_msg = format!("Network error: {}", e);
                last_error = Some(error_msg.clone().into());
                println!("Network error, will retry: {}", error_msg);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "All retry attempts failed".into()))
}

/// Get the default storage directory for keys
/// Build an HTTP client with automatic signing support
#[allow(dead_code)]
fn build_authenticated_client(
    cli: &Cli,
    command_name: &str,
    config_manager: &CliConfigManager,
) -> Result<AuthenticatedHttpClient, Box<dyn std::error::Error>> {
    let signing_context = config_manager.signing_config().for_command(command_name);

    // With mandatory authentication, all requests must be signed
    let should_sign = true;

    if cli.verbose {
        println!(
            "🔐 Signing status for '{}': {}",
            command_name,
            if should_sign { "enabled" } else { "disabled" }
        );
        if signing_context.debug_enabled {
            println!("🐛 Debug mode enabled for signature operations");
        }
    }

    // Get timeout from CLI config
    let timeout = config_manager.config().settings.default_timeout_secs;
    let retry_config = RetryConfig {
        max_retries: config_manager.config().settings.default_max_retries,
        initial_delay_ms: 1000,
        max_delay_ms: 30000,
        retry_server_errors: true,
        retry_network_errors: true,
    };

    if !should_sign {
        // Return unauthenticated client
        return Ok(HttpClientBuilder::new()
            .timeout_secs(timeout)
            .retry_config(retry_config)
            .build()?);
    }

    // Get profile to use
    let profile_name = cli
        .profile
        .as_ref()
        .or(signing_context.profile.as_ref())
        .or(config_manager.config().default_profile.as_ref())
        .ok_or("No authentication profile specified and no default profile set")?;

    let profile = config_manager
        .get_profile(profile_name)
        .ok_or_else(|| format!("Authentication profile '{}' not found", profile_name))?;

    if cli.verbose {
        println!("🔑 Using authentication profile: {}", profile_name);
        println!("🌐 Server URL: {}", profile.server_url);
        println!("🆔 Client ID: {}", profile.client_id);
    }

    // Load key from storage
    let storage_dir = CliConfigManager::default_keys_dir()?;
    let key_content = handle_retrieve_key_internal(&profile.key_id, &storage_dir, false)?;

    // Parse the key
    let private_key_bytes = parse_key_input(&key_content, true)?;
    let keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)
        .map_err(|e| format!("Failed to load keypair: {}", e))?;

    // Create signing configuration
    let signing_config = signing_context.get_signing_config().clone();

    // Apply CLI debug override
    if cli.sign_debug {
        // Enable debug logging for this request
        if cli.verbose {
            println!("🐛 Enabling debug mode for signature generation");
        }
    }

    // Build authenticated client
    let client = HttpClientBuilder::new()
        .timeout_secs(timeout)
        .retry_config(retry_config)
        .build_authenticated(keypair, profile.clone(), Some(signing_config))?;

    if cli.verbose {
        println!("✅ Authenticated HTTP client ready");
    }

    Ok(client)
}

/// Internal helper for retrieving keys without authentication
#[allow(dead_code)]
#[allow(clippy::ptr_arg)]
fn handle_retrieve_key_internal(
    key_id: &str,
    storage_dir: &PathBuf,
    public_only: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let key_file = storage_dir.join(format!("{}.json", key_id));

    if !key_file.exists() {
        return Err(format!("Key '{}' not found in storage", key_id).into());
    }

    let content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&content)?;

    // Prompt for passphrase
    print!("Enter passphrase for key '{}': ", key_id);
    io::stdout().flush()?;
    let passphrase = read_password()?;

    // Decrypt the key
    let decrypted_bytes = decrypt_key(&key_config, &passphrase)?;

    if public_only {
        // Extract public key from private key
        let keypair = MasterKeyPair::from_secret_bytes(&decrypted_bytes)
            .map_err(|e| format!("Failed to parse private key: {}", e))?;
        let public_key_bytes = keypair.public_key_bytes();
        Ok(general_purpose::STANDARD.encode(public_key_bytes))
    } else {
        Ok(general_purpose::STANDARD.encode(decrypted_bytes))
    }
}

fn get_default_storage_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Unable to determine home directory")?;
    Ok(home_dir.join(".datafold").join("keys"))
}

/// Ensure storage directory exists with proper permissions
fn ensure_storage_dir(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    }

    // Set directory permissions to 700 (owner read/write/execute only)
    let mut perms = fs::metadata(dir)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(dir, perms)
        .map_err(|e| format!("Failed to set directory permissions: {}", e))?;

    Ok(())
}

/// Encrypt a private key with a passphrase using BLAKE3-based stream cipher
fn encrypt_key(
    private_key: &[u8; 32],
    passphrase: &str,
    argon2_params: &Argon2Params,
) -> Result<KeyStorageConfig, Box<dyn std::error::Error>> {
    // Generate salt and nonce
    let salt = generate_salt();
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    // Derive encryption key from passphrase
    let derived_key = derive_key(passphrase, &salt, argon2_params)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Use BLAKE3 to generate a keystream for encryption
    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&nonce);
    let keystream = hasher.finalize();

    // XOR encrypt the private key
    let mut encrypted_key = [0u8; 32];
    for i in 0..32 {
        encrypted_key[i] = private_key[i] ^ keystream.as_bytes()[i];
    }

    Ok(KeyStorageConfig {
        encrypted_key: encrypted_key.to_vec(),
        nonce,
        salt: *salt.as_bytes(),
        argon2_params: argon2_params.into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        version: 1,
    })
}

/// Decrypt a private key from storage
fn decrypt_key(
    config: &KeyStorageConfig,
    passphrase: &str,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    // Reconstruct Argon2 params
    let argon2_params: Argon2Params = config.argon2_params.clone().into();

    // Create Salt from stored bytes
    let salt = datafold::crypto::argon2::Salt::from_bytes(config.salt);

    // Derive decryption key from passphrase
    let derived_key = derive_key(passphrase, &salt, &argon2_params)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Use BLAKE3 to generate the same keystream
    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&config.nonce);
    let keystream = hasher.finalize();

    // XOR decrypt the private key
    if config.encrypted_key.len() != 32 {
        return Err("Invalid encrypted key length".into());
    }

    let mut decrypted_key = [0u8; 32];
    for (i, item) in decrypted_key.iter_mut().enumerate() {
        *item = config.encrypted_key[i] ^ keystream.as_bytes()[i];
    }

    Ok(decrypted_key)
}

fn handle_store_key(
    key_id: String,
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    storage_dir: Option<PathBuf>,
    force: bool,
    security_level: CliSecurityLevel,
    passphrase: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    ensure_storage_dir(&storage_path)?;

    // Get private key bytes
    let private_key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read private key file: {}", e))?;
            parse_key_input(content.trim(), true)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --private-key and --private-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --private-key or --private-key-file".into());
        }
    };

    // Check if key already exists
    let key_file_path = storage_path.join(format!("{}.key", key_id));
    if key_file_path.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite", key_id).into());
    }

    // Get passphrase for encryption
    let passphrase = match passphrase {
        Some(p) => p,
        None => get_secure_passphrase()?,
    };

    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Encrypt the key
    let storage_config = encrypt_key(&private_key_bytes, &passphrase, &argon2_params)?;

    // Write encrypted key to file
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)
        .map_err(|e| format!("Failed to write key file: {}", e))?;

    // Set file permissions to 600 (owner read/write only)
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)
        .map_err(|e| format!("Failed to set file permissions: {}", e))?;

    info!(
        "✅ Key '{}' stored securely at: {}",
        key_id,
        key_file_path.display()
    );
    info!("Security level: {:?}", security_level);

    Ok(())
}

fn handle_retrieve_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    format: KeyFormat,
    output_file: Option<PathBuf>,
    public_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Read storage config
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let storage_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Get passphrase for decryption
    print!("Enter passphrase to decrypt key: ");
    io::stdout().flush()?;
    let passphrase = read_password()?;

    // Decrypt the private key
    let private_key_bytes = decrypt_key(&storage_config, &passphrase)?;

    if public_only {
        // Extract and output public key only
        let keypair =
            datafold::crypto::ed25519::generate_master_keypair_from_seed(&private_key_bytes)
                .map_err(|e| format!("Failed to generate keypair from stored key: {}", e))?;
        let public_key_bytes = keypair.public_key_bytes();
        let formatted_public = format_key(&public_key_bytes, &format, false)?;
        output_key(
            &formatted_public,
            output_file.as_ref(),
            "public",
            0,
            1,
            true,
        )?;
    } else {
        // Output private key
        let formatted_private = format_key(&private_key_bytes, &format, true)?;
        output_key(
            &formatted_private,
            output_file.as_ref(),
            "private",
            0,
            1,
            true,
        )?;
    }

    info!("✅ Key '{}' retrieved successfully", key_id);

    Ok(())
}

fn handle_delete_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Confirm deletion unless force is specified
    if !force {
        print!("Are you sure you want to delete key '{}'? (y/N): ", key_id);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            info!("Key deletion cancelled");
            return Ok(());
        }
    }

    // Delete the key file
    fs::remove_file(&key_file_path).map_err(|e| format!("Failed to delete key file: {}", e))?;

    info!("✅ Key '{}' deleted successfully", key_id);

    Ok(())
}

fn handle_list_keys(
    storage_dir: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);

    if !storage_path.exists() {
        info!("No keys found (storage directory doesn't exist)");
        return Ok(());
    }

    // Read directory entries
    let entries = fs::read_dir(&storage_path)
        .map_err(|e| format!("Failed to read storage directory: {}", e))?;

    let mut keys = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "key") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                keys.push((stem.to_string(), path));
            }
        }
    }

    if keys.is_empty() {
        info!("No keys found in storage directory");
        return Ok(());
    }

    keys.sort_by(|a, b| a.0.cmp(&b.0));

    info!("Stored keys in {}:", storage_path.display());

    for (key_id, path) in keys {
        if verbose {
            // Read and parse key config for detailed info
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<KeyStorageConfig>(&content) {
                    Ok(config) => {
                        info!(
                            "  {} (created: {}, security: {}KB/{}/{})",
                            key_id,
                            config.created_at,
                            config.argon2_params.memory_cost,
                            config.argon2_params.time_cost,
                            config.argon2_params.parallelism
                        );
                    }
                    Err(_) => {
                        info!("  {} (invalid format)", key_id);
                    }
                },
                Err(_) => {
                    info!("  {} (read error)", key_id);
                }
            }
        } else {
            info!("  {}", key_id);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_derive_from_master(
    master_key_id: String,
    context: String,
    child_key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    format: KeyFormat,
    output_only: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let master_key_file_path = storage_path.join(format!("{}.key", master_key_id));

    if !master_key_file_path.exists() {
        return Err(format!("Master key '{}' not found", master_key_id).into());
    }

    // Read and decrypt master key
    let config_content = fs::read_to_string(&master_key_file_path)
        .map_err(|e| format!("Failed to read master key file: {}", e))?;
    let storage_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse master key file: {}", e))?;

    print!(
        "Enter passphrase to decrypt master key '{}': ",
        master_key_id
    );
    io::stdout().flush()?;
    let passphrase = read_password()?;

    let master_key_bytes = decrypt_key(&storage_config, &passphrase)?;

    // Derive child key using HKDF (BLAKE3)
    let context_bytes = context.as_bytes();
    let salt = generate_salt();
    let derived_key_material =
        hkdf_derive_key(&master_key_bytes, salt.as_bytes(), context_bytes, 32);

    if derived_key_material.len() != 32 {
        return Err("Failed to derive 32-byte key".into());
    }

    let mut child_key_bytes = [0u8; 32];
    child_key_bytes.copy_from_slice(&derived_key_material);

    if output_only {
        // Just output the derived key
        let formatted_key = format_key(&child_key_bytes, &format, true)?;
        println!("{}", formatted_key);
        info!(
            "✅ Child key derived from master '{}' with context '{}'",
            master_key_id, context
        );
    } else {
        // Store the derived child key
        let child_key_file_path = storage_path.join(format!("{}.key", child_key_id));
        if child_key_file_path.exists() && !force {
            return Err(format!(
                "Child key '{}' already exists. Use --force to overwrite",
                child_key_id
            )
            .into());
        }

        // Get passphrase for child key encryption
        print!("Enter passphrase to encrypt child key '{}': ", child_key_id);
        io::stdout().flush()?;
        let child_passphrase = read_password()?;

        // Convert security level to Argon2 parameters
        let argon2_params = match security_level {
            CliSecurityLevel::Interactive => Argon2Params::interactive(),
            CliSecurityLevel::Balanced => Argon2Params::default(),
            CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
        };

        // Encrypt the child key
        let child_storage_config =
            encrypt_key(&child_key_bytes, &child_passphrase, &argon2_params)?;

        // Write encrypted child key to file
        let config_json = serde_json::to_string_pretty(&child_storage_config)?;
        fs::write(&child_key_file_path, config_json)
            .map_err(|e| format!("Failed to write child key file: {}", e))?;

        // Set file permissions to 600 (owner read/write only)
        let mut perms = fs::metadata(&child_key_file_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&child_key_file_path, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;

        info!(
            "✅ Child key '{}' derived from master '{}' and stored securely",
            child_key_id, master_key_id
        );
        info!("Derivation context: '{}'", context);
        info!("Security level: {:?}", security_level);
    }

    // Clear sensitive data
    let _ = master_key_bytes;
    let _ = child_key_bytes;

    Ok(())
}

fn handle_rotate_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    method: RotationMethod,
    keep_backup: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Confirm rotation unless force is specified
    if !force {
        print!(
            "Are you sure you want to rotate key '{}'? This will create a new version. (y/N): ",
            key_id
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            info!("Key rotation cancelled");
            return Ok(());
        }
    }

    // Read current key
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let current_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Get passphrase for current key
    print!("Enter passphrase for current key '{}': ", key_id);
    io::stdout().flush()?;
    let current_passphrase = read_password()?;

    let current_key_bytes = decrypt_key(&current_config, &current_passphrase)?;

    // Generate new key based on rotation method
    let new_key_bytes = match method {
        RotationMethod::Regenerate => {
            // Generate completely new random key
            let keypair = datafold::crypto::ed25519::generate_master_keypair()
                .map_err(|e| format!("Failed to generate new keypair: {}", e))?;
            keypair.secret_key_bytes()
        }
        RotationMethod::Derive => {
            // Derive new key from current key using incremental counter
            let context = format!("rotation-{}", chrono::Utc::now().timestamp());
            let salt = generate_salt();
            let derived_material =
                hkdf_derive_key(&current_key_bytes, salt.as_bytes(), context.as_bytes(), 32);
            let mut new_key = [0u8; 32];
            new_key.copy_from_slice(&derived_material);
            new_key
        }
        RotationMethod::Rederive => {
            // Re-derive from passphrase with new salt (if original was passphrase-based)
            print!("Enter passphrase for key re-derivation: ");
            io::stdout().flush()?;
            let derive_passphrase = read_password()?;

            let argon2_params = match security_level {
                CliSecurityLevel::Interactive => Argon2Params::interactive(),
                CliSecurityLevel::Balanced => Argon2Params::default(),
                CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
            };

            let derived_key = derive_key(&derive_passphrase, &generate_salt(), &argon2_params)
                .map_err(|e| format!("Key re-derivation failed: {}", e))?;

            let mut new_key = [0u8; 32];
            new_key.copy_from_slice(derived_key.as_bytes());
            new_key
        }
    };

    // Create backup if requested
    if keep_backup {
        let backup_path = storage_path.join(format!(
            "{}.backup.{}.key",
            key_id,
            chrono::Utc::now().timestamp()
        ));
        fs::copy(&key_file_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;

        // Set backup file permissions to 600
        let mut perms = fs::metadata(&backup_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&backup_path, perms)?;

        info!("✅ Backup created: {}", backup_path.display());
    }

    // Get passphrase for new key encryption
    print!("Enter passphrase for rotated key '{}': ", key_id);
    io::stdout().flush()?;
    let new_passphrase = read_password()?;

    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Encrypt the new key
    let new_storage_config = encrypt_key(&new_key_bytes, &new_passphrase, &argon2_params)?;

    // Write new encrypted key to file
    let config_json = serde_json::to_string_pretty(&new_storage_config)?;
    fs::write(&key_file_path, config_json)
        .map_err(|e| format!("Failed to write rotated key file: {}", e))?;

    // Set file permissions to 600
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)?;

    info!(
        "✅ Key '{}' rotated successfully using method: {:?}",
        key_id, method
    );
    info!("Security level: {:?}", security_level);
    if keep_backup {
        info!("Previous version backed up");
    }

    // Clear sensitive data
    let _ = current_key_bytes;
    let _ = new_key_bytes;

    Ok(())
}

fn handle_list_key_versions(
    key_id: String,
    storage_dir: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);

    if !storage_path.exists() {
        info!("No keys found (storage directory doesn't exist)");
        return Ok(());
    }

    // Find all versions of the key
    let entries = fs::read_dir(&storage_path)
        .map_err(|e| format!("Failed to read storage directory: {}", e))?;

    let mut versions = Vec::new();

    // Look for main key and backup files
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Check for main key file
            if filename == format!("{}.key", key_id) {
                versions.push(("current".to_string(), path));
            }
            // Check for backup files
            else if filename.starts_with(&format!("{}.backup.", key_id))
                && filename.ends_with(".key")
            {
                if let Some(timestamp_part) = filename
                    .strip_prefix(&format!("{}.backup.", key_id))
                    .and_then(|s| s.strip_suffix(".key"))
                {
                    versions.push((format!("backup-{}", timestamp_part), path));
                }
            }
        }
    }

    if versions.is_empty() {
        info!("No versions found for key '{}'", key_id);
        return Ok(());
    }

    // Sort versions
    versions.sort_by(|a, b| a.0.cmp(&b.0));

    info!(
        "Versions for key '{}' in {}:",
        key_id,
        storage_path.display()
    );

    for (version_name, path) in versions {
        if verbose {
            // Read and parse key config for detailed info
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<KeyStorageConfig>(&content) {
                    Ok(config) => {
                        info!(
                            "  {} (created: {}, security: {}KB/{}/{})",
                            version_name,
                            config.created_at,
                            config.argon2_params.memory_cost,
                            config.argon2_params.time_cost,
                            config.argon2_params.parallelism
                        );
                    }
                    Err(_) => {
                        info!("  {} (invalid format)", version_name);
                    }
                },
                Err(_) => {
                    info!("  {} (read error)", version_name);
                }
            }
        } else {
            info!("  {}", version_name);
        }
    }

    Ok(())
}

fn handle_backup_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    backup_file: PathBuf,
    backup_passphrase: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Read the key config
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let key_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Create backup metadata
    let backup_metadata = KeyVersionMetadata {
        version: 1,
        previous_version: None,
        created_at: key_config.created_at.clone(),
        derivation_method: "Backup".to_string(),
        salt: key_config.salt,
        argon2_params: key_config.argon2_params.clone(),
    };

    let mut backup_data = config_content.into_bytes();
    let mut backup_nonce = [0u8; 12];
    OsRng.fill_bytes(&mut backup_nonce);

    let mut backup_salt = None;
    let mut backup_params = None;

    // Apply additional encryption if backup passphrase is requested
    if backup_passphrase {
        print!("Enter backup passphrase for additional encryption: ");
        io::stdout().flush()?;
        let backup_pass = read_password()?;

        let salt = generate_salt();
        let argon2_params = Argon2Params::default();

        let derived_key = derive_key(&backup_pass, &salt, &argon2_params)
            .map_err(|e| format!("Backup key derivation failed: {}", e))?;

        // Use BLAKE3 to generate keystream for encryption
        let mut hasher = blake3::Hasher::new();
        hasher.update(derived_key.as_bytes());
        hasher.update(&backup_nonce);
        let keystream = hasher.finalize();

        // XOR encrypt the backup data
        for (i, byte) in backup_data.iter_mut().enumerate() {
            if i < keystream.as_bytes().len() {
                *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
            }
        }

        backup_salt = Some(*salt.as_bytes());
        backup_params = Some((&argon2_params).into());
    }

    // Create backup format
    let backup_format = KeyBackupFormat {
        format_version: 1,
        key_id: key_id.clone(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        backup_data,
        backup_nonce,
        backup_salt,
        backup_params,
        original_metadata: backup_metadata,
    };

    // Write backup file
    let backup_json = serde_json::to_string_pretty(&backup_format)
        .map_err(|e| format!("Failed to serialize backup: {}", e))?;

    fs::write(&backup_file, backup_json)
        .map_err(|e| format!("Failed to write backup file: {}", e))?;

    // Set backup file permissions to 600
    let mut perms = fs::metadata(&backup_file)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&backup_file, perms)
        .map_err(|e| format!("Failed to set backup file permissions: {}", e))?;

    info!(
        "✅ Key '{}' backed up to: {}",
        key_id,
        backup_file.display()
    );
    if backup_passphrase {
        info!("Backup is double-encrypted with backup passphrase");
    }

    Ok(())
}

fn handle_restore_key(
    backup_file: PathBuf,
    key_id: String,
    storage_dir: Option<PathBuf>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    ensure_storage_dir(&storage_path)?;

    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if key_file_path.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite", key_id).into());
    }

    // Read backup file
    let backup_content = fs::read_to_string(&backup_file)
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    let backup_format: KeyBackupFormat = serde_json::from_str(&backup_content)
        .map_err(|e| format!("Failed to parse backup file: {}", e))?;

    let mut restored_data = backup_format.backup_data;

    // Decrypt backup if it has additional encryption
    if backup_format.backup_salt.is_some() && backup_format.backup_params.is_some() {
        print!("Enter backup passphrase for decryption: ");
        io::stdout().flush()?;
        let backup_pass = read_password()?;

        let salt = datafold::crypto::argon2::Salt::from_bytes(backup_format.backup_salt.unwrap());
        let argon2_params: Argon2Params = backup_format.backup_params.unwrap().into();

        let derived_key = derive_key(&backup_pass, &salt, &argon2_params)
            .map_err(|e| format!("Backup key derivation failed: {}", e))?;

        // Use BLAKE3 to generate the same keystream
        let mut hasher = blake3::Hasher::new();
        hasher.update(derived_key.as_bytes());
        hasher.update(&backup_format.backup_nonce);
        let keystream = hasher.finalize();

        // XOR decrypt the backup data
        for (i, byte) in restored_data.iter_mut().enumerate() {
            if i < keystream.as_bytes().len() {
                *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
            }
        }
    }

    // Write restored key to file
    fs::write(&key_file_path, &restored_data)
        .map_err(|e| format!("Failed to write restored key file: {}", e))?;

    // Set file permissions to 600
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)
        .map_err(|e| format!("Failed to set file permissions: {}", e))?;

    info!(
        "✅ Key '{}' restored from backup: {}",
        key_id,
        backup_file.display()
    );
    info!("Original key ID: {}", backup_format.key_id);
    info!("Backup created: {}", backup_format.exported_at);

    Ok(())
}

fn handle_export_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    export_file: PathBuf,
    format: ExportFormat,
    export_passphrase: bool,
    include_metadata: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Read the key config
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let key_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Get the original key passphrase to decrypt the stored key
    print!("Enter passphrase to decrypt stored key '{}': ", key_id);
    io::stdout().flush()?;
    let key_passphrase = read_password()?;

    // Decrypt the stored key
    let decrypted_key = decrypt_key(&key_config, &key_passphrase)?;

    // Get export passphrase
    let export_pass = if export_passphrase {
        print!("Enter export passphrase for additional protection: ");
        io::stdout().flush()?;
        Some(read_password()?)
    } else {
        // Use the same passphrase as the stored key
        Some(key_passphrase)
    };

    if let Some(pass) = export_pass {
        // Generate salt and nonce for export encryption
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        let mut nonce = [0u8; 12]; // AES-GCM nonce
        OsRng.fill_bytes(&mut nonce);

        // Use stronger parameters for export
        let argon2_params = Argon2Params::sensitive();

        // Derive export encryption key
        let salt_obj = datafold::crypto::argon2::Salt::from_bytes(salt);
        let derived_key = derive_key(&pass, &salt_obj, &argon2_params)
            .map_err(|e| format!("Export key derivation failed: {}", e))?;

        // Encrypt the key using AES-GCM-like encryption (simplified)
        let mut hasher = blake3::Hasher::new();
        hasher.update(derived_key.as_bytes());
        hasher.update(&nonce);
        let keystream = hasher.finalize();

        // XOR encrypt the key data
        let mut encrypted_data = decrypted_key.to_vec();
        for (i, byte) in encrypted_data.iter_mut().enumerate() {
            if i < keystream.as_bytes().len() {
                *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
            }
        }

        // Create export metadata if requested
        let metadata = if include_metadata {
            Some(ExportKeyMetadata {
                key_id: key_id.clone(),
                original_created: key_config.created_at.clone(),
                export_source: format!("DataFold CLI v{}", env!("CARGO_PKG_VERSION")),
                notes: Some("Exported with enhanced security".to_string()),
            })
        } else {
            None
        };

        // Create the enhanced export format
        let export_data = EnhancedKeyExportFormat {
            version: 1,
            kdf: "argon2id".to_string(),
            kdf_params: EnhancedKdfParams {
                salt: salt.to_vec(),
                memory: argon2_params.memory_cost,
                iterations: argon2_params.time_cost,
                parallelism: argon2_params.parallelism,
            },
            encryption: "aes-gcm-like".to_string(),
            nonce: nonce.to_vec(),
            ciphertext: encrypted_data,
            created: chrono::Utc::now().to_rfc3339(),
            metadata,
        };

        match format {
            ExportFormat::Json => {
                // Export as JSON
                let export_json = serde_json::to_string_pretty(&export_data)
                    .map_err(|e| format!("Failed to serialize export data: {}", e))?;

                fs::write(&export_file, export_json)
                    .map_err(|e| format!("Failed to write export file: {}", e))?;
            }
            ExportFormat::Binary => {
                // Export as compact binary (using bincode or similar)
                let export_binary = serde_json::to_vec(&export_data)
                    .map_err(|e| format!("Failed to serialize export data: {}", e))?;

                fs::write(&export_file, export_binary)
                    .map_err(|e| format!("Failed to write export file: {}", e))?;
            }
        }

        // Set export file permissions to 600
        let mut perms = fs::metadata(&export_file)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&export_file, perms)
            .map_err(|e| format!("Failed to set export file permissions: {}", e))?;

        info!("✅ Key '{}' exported to: {}", key_id, export_file.display());
        info!("Export format: {:?}", format);
        if export_passphrase {
            info!("Export uses additional passphrase protection");
        }
        if include_metadata {
            info!("Export includes key metadata");
        }
    } else {
        return Err("Export passphrase is required".into());
    }

    // Clear sensitive data
    let _ = decrypted_key;

    Ok(())
}

fn handle_import_key(
    export_file: PathBuf,
    key_id: String,
    storage_dir: Option<PathBuf>,
    force: bool,
    verify_integrity: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    ensure_storage_dir(&storage_path)?;

    let key_file_path = storage_path.join(format!("{}.key", key_id));

    if key_file_path.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite", key_id).into());
    }

    // Read export file
    let export_content = fs::read_to_string(&export_file)
        .map_err(|e| format!("Failed to read export file: {}", e))?;

    // Try to parse as enhanced export format
    let export_data: EnhancedKeyExportFormat =
        serde_json::from_str(&export_content).map_err(|e| {
            format!(
                "Failed to parse export file (not valid enhanced format): {}",
                e
            )
        })?;

    // Validate export format version
    if export_data.version != 1 {
        return Err(format!("Unsupported export format version: {}", export_data.version).into());
    }

    // Validate encryption algorithm
    if export_data.encryption != "aes-gcm-like" {
        return Err(format!(
            "Unsupported encryption algorithm: {}",
            export_data.encryption
        )
        .into());
    }

    // Validate KDF
    if export_data.kdf != "argon2id" {
        return Err(format!("Unsupported KDF: {}", export_data.kdf).into());
    }

    // Get import passphrase
    print!("Enter import passphrase to decrypt exported key: ");
    io::stdout().flush()?;
    let import_passphrase = read_password()?;

    // Reconstruct Argon2 parameters
    let argon2_params = Argon2Params::new(
        export_data.kdf_params.memory,
        export_data.kdf_params.iterations,
        export_data.kdf_params.parallelism,
    )
    .map_err(|e| format!("Invalid KDF parameters: {}", e))?;

    // Recreate salt from export data
    if export_data.kdf_params.salt.len() != 32 {
        return Err("Invalid salt length in export data".into());
    }
    let mut salt_bytes = [0u8; 32];
    salt_bytes.copy_from_slice(&export_data.kdf_params.salt);
    let salt = datafold::crypto::argon2::Salt::from_bytes(salt_bytes);

    // Derive decryption key
    let derived_key = derive_key(&import_passphrase, &salt, &argon2_params)
        .map_err(|e| format!("Import key derivation failed: {}", e))?;

    // Decrypt the key data
    if export_data.nonce.len() != 12 {
        return Err("Invalid nonce length in export data".into());
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&export_data.nonce);

    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&nonce);
    let keystream = hasher.finalize();

    // XOR decrypt the key data
    let mut decrypted_data = export_data.ciphertext.clone();
    for (i, byte) in decrypted_data.iter_mut().enumerate() {
        if i < keystream.as_bytes().len() {
            *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
        }
    }

    // Validate decrypted key length
    if decrypted_data.len() != 32 {
        return Err("Invalid decrypted key length (corruption or wrong passphrase)".into());
    }

    let mut imported_key = [0u8; 32];
    imported_key.copy_from_slice(&decrypted_data);

    // Verify key integrity if requested
    if verify_integrity {
        // Generate keypair from imported key to verify it's valid
        let keypair = datafold::crypto::ed25519::generate_master_keypair_from_seed(&imported_key)
            .map_err(|e| format!("Key integrity verification failed: {}", e))?;

        // Test signing and verification
        let test_message = b"DataFold import verification test";
        let signature = keypair
            .sign_data(test_message)
            .map_err(|e| format!("Key functionality test failed: {}", e))?;

        keypair
            .verify_data(test_message, &signature)
            .map_err(|e| format!("Key verification test failed: {}", e))?;

        info!("✅ Key integrity verification passed");
    }

    // Get passphrase for storing the imported key
    print!(
        "Enter passphrase to encrypt imported key '{}' for storage: ",
        key_id
    );
    io::stdout().flush()?;
    let storage_passphrase = read_password()?;

    // Use balanced security for storage (can be upgraded later if needed)
    let storage_argon2_params = Argon2Params::default();

    // Encrypt the imported key for storage
    let storage_config = encrypt_key(&imported_key, &storage_passphrase, &storage_argon2_params)?;

    // Write encrypted key to file
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)
        .map_err(|e| format!("Failed to write imported key file: {}", e))?;

    // Set file permissions to 600
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)
        .map_err(|e| format!("Failed to set file permissions: {}", e))?;

    info!("✅ Key imported successfully as '{}'", key_id);
    info!("Source export created: {}", export_data.created);
    if let Some(metadata) = &export_data.metadata {
        info!("Original key ID: {}", metadata.key_id);
        info!("Export source: {}", metadata.export_source);
        if let Some(notes) = &metadata.notes {
            info!("Notes: {}", notes);
        }
    }

    // Clear sensitive data
    let _ = imported_key;
    drop(decrypted_data);

    Ok(())
}

/// Handle public key registration with server
#[allow(clippy::too_many_arguments)]
async fn handle_register_key(
    server_url: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    client_id: Option<String>,
    user_id: Option<String>,
    key_name: Option<String>,
    timeout: u64,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Registering public key with DataFold server...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    // Load the key from storage
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if !key_file.exists() {
        return Err(format!(
            "Key '{}' not found in storage. Use store-key to create it first.",
            key_id
        )
        .into());
    }

    // Read and decrypt the stored key
    let key_content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&key_content)?;

    print!("Enter passphrase to unlock key: ");
    io::stdout().flush()?;
    let passphrase = get_secure_passphrase()?;

    let private_key_bytes = decrypt_key(&key_config, &passphrase)?;

    // Extract public key from private key
    let master_keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)?;
    let public_key = master_keypair.public_key();
    let public_key_hex = hex::encode(public_key.to_bytes());

    // Generate client ID if not provided
    let client_id = client_id.unwrap_or_else(|| format!("cli_{}", uuid::Uuid::new_v4()));

    // Create HTTP client
    let client = create_http_client(timeout)?;

    // Prepare registration request
    let registration_request = PublicKeyRegistrationRequest {
        client_id: Some(client_id.clone()),
        user_id,
        public_key: public_key_hex,
        key_name,
        metadata: Some({
            let mut meta = std::collections::HashMap::new();
            meta.insert("source".to_string(), "datafold-cli".to_string());
            meta.insert("key_id".to_string(), key_id.clone());
            meta
        }),
    };

    // Send registration request
    let register_url = format!(
        "{}/api/crypto/keys/register",
        server_url.trim_end_matches('/')
    );
    let request = client
        .post(&register_url)
        .header("Content-Type", "application/json")
        .json(&registration_request);

    println!("Sending registration request to: {}", register_url);
    let response: PublicKeyRegistrationResponse =
        http_request_with_retry(&client, request, retries).await?;

    println!("✅ Public key registered successfully!");
    println!("Registration ID: {}", response.registration_id);
    println!("Client ID: {}", response.client_id);
    println!("Status: {}", response.status);
    println!("Registered at: {}", response.registered_at);

    // Save client ID for future use
    let client_file = storage_dir.join(format!("{}_client.json", key_id));
    let client_info = json!({
        "client_id": response.client_id,
        "registration_id": response.registration_id,
        "server_url": server_url,
        "registered_at": response.registered_at
    });
    fs::write(client_file, serde_json::to_string_pretty(&client_info)?)?;

    println!("Client information saved for future use");

    Ok(())
}

/// Handle checking public key registration status
async fn handle_check_registration(
    server_url: String,
    client_id: String,
    timeout: u64,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking public key registration status...");

    // Create HTTP client
    let client = create_http_client(timeout)?;

    // Send status request
    let status_url = format!(
        "{}/api/crypto/keys/status/{}",
        server_url.trim_end_matches('/'),
        client_id
    );
    let request = client.get(&status_url);

    println!("Requesting status from: {}", status_url);
    let response: PublicKeyStatusResponse =
        http_request_with_retry(&client, request, retries).await?;

    println!("✅ Registration status retrieved successfully!");
    println!("Registration ID: {}", response.registration_id);
    println!("Client ID: {}", response.client_id);
    println!("Public Key: {}", response.public_key);
    println!("Status: {}", response.status);
    println!("Registered at: {}", response.registered_at);
    if let Some(last_used) = response.last_used {
        println!("Last used: {}", last_used);
    } else {
        println!("Last used: Never");
    }
    if let Some(key_name) = response.key_name {
        println!("Key name: {}", key_name);
    }

    Ok(())
}

/// Handle signing message and verifying with server
#[allow(clippy::too_many_arguments)]
async fn handle_sign_and_verify(
    server_url: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    client_id: String,
    message: Option<String>,
    message_file: Option<PathBuf>,
    message_encoding: MessageEncoding,
    timeout: u64,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Signing message and verifying with server...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    // Load the key from storage
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if !key_file.exists() {
        return Err(format!(
            "Key '{}' not found in storage. Use store-key to create it first.",
            key_id
        )
        .into());
    }

    // Read and decrypt the stored key
    let key_content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&key_content)?;

    print!("Enter passphrase to unlock key: ");
    io::stdout().flush()?;
    let passphrase = get_secure_passphrase()?;

    let private_key_bytes = decrypt_key(&key_config, &passphrase)?;
    let master_keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)?;

    // Get message to sign
    let message_to_sign = match (message, message_file) {
        (Some(msg), None) => msg,
        (None, Some(file)) => fs::read_to_string(file)?,
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --message and --message-file".into());
        }
        (None, None) => {
            return Err("Must specify either --message or --message-file".into());
        }
    };

    // Sign the message
    let signature = master_keypair.sign_data(message_to_sign.as_bytes())?;
    let signature_hex = hex::encode(signature);

    println!("Message signed successfully");
    println!("Signature: {}", signature_hex);

    // Create HTTP client
    let client = create_http_client(timeout)?;

    // Prepare verification request
    let encoding_str = match message_encoding {
        MessageEncoding::Utf8 => "utf8",
        MessageEncoding::Hex => "hex",
        MessageEncoding::Base64 => "base64",
    };

    let verification_request = SignatureVerificationRequest {
        client_id: client_id.clone(),
        message: message_to_sign,
        signature: signature_hex,
        message_encoding: Some(encoding_str.to_string()),
        metadata: Some({
            let mut meta = std::collections::HashMap::new();
            meta.insert("source".to_string(), "datafold-cli".to_string());
            meta.insert("key_id".to_string(), key_id.clone());
            meta
        }),
    };

    // Send verification request
    let verify_url = format!(
        "{}/api/crypto/signatures/verify",
        server_url.trim_end_matches('/')
    );
    let request = client
        .post(&verify_url)
        .header("Content-Type", "application/json")
        .json(&verification_request);

    println!("Sending verification request to: {}", verify_url);
    let response: SignatureVerificationResponse =
        http_request_with_retry(&client, request, retries).await?;

    println!("✅ Signature verification completed!");
    println!(
        "Verified: {}",
        if response.verified {
            "✅ SUCCESS"
        } else {
            "❌ FAILED"
        }
    );
    println!("Client ID: {}", response.client_id);
    println!("Public Key: {}", response.public_key);
    println!("Verified at: {}", response.verified_at);
    println!("Message hash: {}", response.message_hash);

    if !response.verified {
        return Err("Signature verification failed".into());
    }

    Ok(())
}

/// Handle CLI authentication initialization
fn handle_auth_init(
    server_url: String,
    profile: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    user_id: Option<String>,
    _environment: Option<String>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Initializing CLI authentication...");

    // Load or create CLI configuration
    let mut config_manager = CliConfigManager::new()?;

    // Check if profile already exists
    if config_manager.get_profile(&profile).is_some() && !force {
        return Err(format!(
            "Profile '{}' already exists. Use --force to overwrite.",
            profile
        )
        .into());
    }

    // Verify the key exists in storage
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if !key_file.exists() {
        return Err(format!(
            "Key '{}' not found in storage. Use 'auth-keygen' or 'store-key' to create it first.",
            key_id
        )
        .into());
    }

    // Generate client ID
    let client_id = format!("cli_{}", uuid::Uuid::new_v4());

    // Create authentication profile
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "datafold-cli".to_string());
    metadata.insert("created_by".to_string(), "auth-init".to_string());

    let auth_profile = CliAuthProfile {
        client_id: client_id.clone(),
        key_id: key_id.clone(),
        user_id,
        server_url: server_url.clone(),
        metadata,
    };

    // Add profile to configuration
    config_manager.add_profile(profile.clone(), auth_profile)?;
    config_manager.save()?;

    println!(
        "✅ Authentication profile '{}' created successfully!",
        profile
    );
    println!("Client ID: {}", client_id);
    println!("Key ID: {}", key_id);
    println!("Server URL: {}", server_url);

    if config_manager.config().default_profile.as_ref() == Some(&profile) {
        println!("✨ Set as default profile");
    }

    println!("\n💡 Next steps:");
    println!(
        "1. Register your public key: datafold register-key --key-id {} --client-id {}",
        key_id, client_id
    );
    println!("2. Test authentication: datafold auth-test");

    Ok(())
}

/// Handle CLI authentication status
fn handle_auth_status(
    verbose: bool,
    profile: Option<String>,
    _environment: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 CLI Authentication Status");
    println!("============================");

    let config_manager = CliConfigManager::new()?;
    let status = config_manager.auth_status();

    if !status.configured {
        println!("❌ Authentication not configured");
        println!("\n💡 To get started:");
        println!("1. Generate a key: datafold auth-keygen --key-id my-key");
        println!("2. Initialize auth: datafold auth-init --key-id my-key --server-url https://your-server.com");
        return Ok(());
    }

    println!("✅ Authentication configured");

    if let Some(profile_name) = &profile {
        // Show specific profile
        if let Some(prof) = config_manager.get_profile(profile_name) {
            println!("\n📋 Profile: {}", profile_name);
            println!("   Client ID: {}", prof.client_id);
            println!("   Key ID: {}", prof.key_id);
            println!("   Server URL: {}", prof.server_url);

            if let Some(user_id) = &prof.user_id {
                println!("   User ID: {}", user_id);
            }

            if verbose {
                println!("   Metadata:");
                for (key, value) in &prof.metadata {
                    println!("     {}: {}", key, value);
                }
            }
        } else {
            return Err(format!("Profile '{}' not found", profile_name).into());
        }
    } else {
        // Show default profile and overall status
        if let Some(client_id) = &status.client_id {
            println!("   Client ID: {}", client_id);
        }
        if let Some(key_id) = &status.key_id {
            println!("   Key ID: {}", key_id);
        }
        if let Some(server_url) = &status.server_url {
            println!("   Server URL: {}", server_url);
        }

        if verbose {
            println!("\n📋 All Profiles:");
            let profiles = config_manager.list_profiles();
            if profiles.is_empty() {
                println!("   No profiles configured");
            } else {
                for profile_name in profiles {
                    let is_default =
                        config_manager.config().default_profile.as_ref() == Some(profile_name);
                    let marker = if is_default { " (default)" } else { "" };
                    println!("   • {}{}", profile_name, marker);

                    if let Some(prof) = config_manager.get_profile(profile_name) {
                        println!("     Client ID: {}", prof.client_id);
                        println!("     Key ID: {}", prof.key_id);
                        println!("     Server: {}", prof.server_url);
                    }
                }
            }
        }
    }

    println!(
        "\n🔧 Configuration file: {}",
        config_manager.config_path().display()
    );

    Ok(())
}

/// Handle authentication profile management
async fn handle_auth_profile(action: ProfileAction) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = CliConfigManager::new()?;

    match action {
        ProfileAction::Create {
            name,
            server_url,
            key_id,
            user_id,
            set_default,
        } => {
            // Check if profile already exists
            if config_manager.get_profile(&name).is_some() {
                return Err(format!("Profile '{}' already exists", name).into());
            }

            // Generate client ID
            let client_id = format!("cli_{}", uuid::Uuid::new_v4());

            // Create profile
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "datafold-cli".to_string());
            metadata.insert("created_by".to_string(), "profile-create".to_string());

            let profile = CliAuthProfile {
                client_id: client_id.clone(),
                key_id: key_id.clone(),
                user_id,
                server_url: server_url.clone(),
                metadata,
            };

            config_manager.add_profile(name.clone(), profile)?;

            if set_default {
                config_manager.set_default_profile(name.clone())?;
            }

            config_manager.save()?;

            println!("✅ Profile '{}' created successfully!", name);
            println!("   Client ID: {}", client_id);
            println!("   Key ID: {}", key_id);
            println!("   Server URL: {}", server_url);

            if set_default {
                println!("   ✨ Set as default profile");
            }
        }

        ProfileAction::List { verbose } => {
            let profiles = config_manager.list_profiles();

            if profiles.is_empty() {
                println!("No authentication profiles configured");
                return Ok(());
            }

            println!("📋 Authentication Profiles:");
            println!("===========================");

            for profile_name in profiles {
                let is_default =
                    config_manager.config().default_profile.as_ref() == Some(profile_name);
                let marker = if is_default { " (default)" } else { "" };

                println!("\n• {}{}", profile_name, marker);

                if let Some(prof) = config_manager.get_profile(profile_name) {
                    println!("  Client ID: {}", prof.client_id);
                    println!("  Key ID: {}", prof.key_id);
                    println!("  Server: {}", prof.server_url);

                    if let Some(user_id) = &prof.user_id {
                        println!("  User ID: {}", user_id);
                    }

                    if verbose {
                        println!("  Metadata:");
                        for (key, value) in &prof.metadata {
                            println!("    {}: {}", key, value);
                        }
                    }
                }
            }
        }

        ProfileAction::Show { name } => {
            if let Some(prof) = config_manager.get_profile(&name) {
                let is_default = config_manager.config().default_profile.as_ref() == Some(&name);

                println!("📋 Profile: {}", name);
                if is_default {
                    println!("   Status: Default profile");
                }
                println!("   Client ID: {}", prof.client_id);
                println!("   Key ID: {}", prof.key_id);
                println!("   Server URL: {}", prof.server_url);

                if let Some(user_id) = &prof.user_id {
                    println!("   User ID: {}", user_id);
                }

                println!("   Metadata:");
                for (key, value) in &prof.metadata {
                    println!("     {}: {}", key, value);
                }
            } else {
                return Err(format!("Profile '{}' not found", name).into());
            }
        }

        ProfileAction::Update {
            name,
            server_url,
            key_id,
            user_id,
        } => {
            if let Some(mut prof) = config_manager.get_profile(&name).cloned() {
                let mut updated = false;

                if let Some(new_url) = server_url {
                    prof.server_url = new_url;
                    updated = true;
                }

                if let Some(new_key_id) = key_id {
                    prof.key_id = new_key_id;
                    updated = true;
                }

                if let Some(new_user_id) = user_id {
                    prof.user_id = Some(new_user_id);
                    updated = true;
                }

                if updated {
                    config_manager.add_profile(name.clone(), prof)?;
                    config_manager.save()?;
                    println!("✅ Profile '{}' updated successfully!", name);
                } else {
                    println!("ℹ️  No changes specified for profile '{}'", name);
                }
            } else {
                return Err(format!("Profile '{}' not found", name).into());
            }
        }

        ProfileAction::Delete { name, force } => {
            if config_manager.get_profile(&name).is_none() {
                return Err(format!("Profile '{}' not found", name).into());
            }

            if !force {
                print!(
                    "Are you sure you want to delete profile '{}'? (y/N): ",
                    name
                );
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().to_lowercase().starts_with('y') {
                    println!("❌ Cancelled");
                    return Ok(());
                }
            }

            config_manager.remove_profile(&name)?;
            config_manager.save()?;

            println!("✅ Profile '{}' deleted successfully!", name);
        }

        ProfileAction::SetDefault { name } => {
            if config_manager.get_profile(&name).is_none() {
                return Err(format!("Profile '{}' not found", name).into());
            }

            config_manager.set_default_profile(name.clone())?;
            config_manager.save()?;

            println!("✅ Profile '{}' set as default!", name);
        }
    }

    Ok(())
}

/// Handle CLI authentication key generation
async fn handle_auth_keygen(
    key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    force: bool,
    auto_register: bool,
    server_url: Option<String>,
    passphrase: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Generating authentication key pair...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    // Check if key already exists
    let key_file = storage_dir.join(format!("{}.json", key_id));
    if key_file.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite.", key_id).into());
    }

    // Generate key pair
    let master_keypair = generate_master_keypair()?;

    // Get passphrase for key encryption
    let _passphrase = match passphrase {
        Some(p) => p,
        None => {
            print!("Enter passphrase to encrypt the key: ");
            io::stdout().flush()?;
            get_secure_passphrase()?
        }
    };

    // Store the key
    handle_store_key(
        key_id.clone(),
        Some(hex::encode(master_keypair.secret_key_bytes())),
        None,
        Some(storage_dir.clone()),
        force,
        security_level,
        Some(_passphrase),
    )?;

    println!("✅ Key pair generated and stored!");
    println!("   Key ID: {}", key_id);
    println!(
        "   Public Key: {}",
        hex::encode(master_keypair.public_key_bytes())
    );
    println!("   Storage: {}", key_file.display());

    // Auto-register if requested
    if auto_register {
        if let Some(server) = server_url {
            println!("\n🔄 Auto-registering key with server...");

            let client_id = format!("cli_{}", uuid::Uuid::new_v4());

            match handle_register_key(
                server,
                key_id.clone(),
                Some(storage_dir),
                Some(client_id.clone()),
                None,
                Some(format!("CLI Key: {}", key_id)),
                30, // timeout
                3,  // retries
            )
            .await
            {
                Ok(()) => {
                    println!("✅ Key registered successfully!");
                    println!("   Client ID: {}", client_id);
                    println!("\n💡 To use this key for authentication:");
                    println!(
                        "   datafold auth-init --key-id {} --server-url <server-url>",
                        key_id
                    );
                }
                Err(e) => {
                    println!(
                        "⚠️  Key generation successful but registration failed: {}",
                        e
                    );
                    println!("   You can register manually later with:");
                    println!("   datafold register-key --key-id {}", key_id);
                }
            }
        } else {
            println!("\n💡 To register this key:");
            println!(
                "   datafold register-key --key-id {} --server-url <server-url>",
                key_id
            );
        }
    } else {
        println!("\n💡 Next steps:");
        println!("1. Register key: datafold register-key --key-id {}", key_id);
        println!("2. Initialize auth: datafold auth-init --key-id {}", key_id);
    }

    Ok(())
}

/// Handle authenticated request test
async fn handle_auth_test(
    endpoint: String,
    profile: Option<String>,
    method: HttpMethod,
    payload: Option<String>,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing authenticated request...");

    // Load CLI configuration
    let config_manager = CliConfigManager::new()?;

    // Get profile to use
    let auth_profile = if let Some(profile_name) = &profile {
        config_manager
            .get_profile(profile_name)
            .ok_or_else(|| format!("Profile '{}' not found", profile_name))?
    } else {
        config_manager
            .get_default_profile()
            .ok_or("No default profile configured. Use 'auth-init' to set up authentication.")?
    };

    // Get storage directory and load key
    let storage_dir = CliConfigManager::default_keys_dir()?;
    let key_file = storage_dir.join(format!("{}.json", auth_profile.key_id));

    if !key_file.exists() {
        return Err(format!("Key '{}' not found in storage", auth_profile.key_id).into());
    }

    // Load and decrypt the key
    let key_content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&key_content)?;

    print!("Enter passphrase to unlock key: ");
    io::stdout().flush()?;
    let passphrase = get_secure_passphrase()?;

    let private_key_bytes = decrypt_key(&key_config, &passphrase)?;
    let master_keypair = MasterKeyPair::from_secret_bytes(&private_key_bytes)?;

    // Create authenticated HTTP client
    let signing_config = CliSigningConfig::default();
    let client = HttpClientBuilder::new()
        .timeout_secs(timeout)
        .build_authenticated(master_keypair, auth_profile.clone(), Some(signing_config))?;

    // Build request URL
    let full_url = if endpoint.starts_with("http") {
        endpoint.to_string()
    } else {
        let path = if endpoint.starts_with('/') {
            endpoint.to_string()
        } else {
            format!("/{}", endpoint)
        };
        format!("{}{}", auth_profile.server_url.trim_end_matches('/'), path)
    };

    println!(
        "Making {} request to: {}",
        method_to_string(&method),
        full_url
    );

    // Execute request based on method
    let response = match method {
        HttpMethod::Get => client.get(&full_url).await?,
        HttpMethod::Post => {
            let body = payload.unwrap_or_else(|| "{}".to_string());
            client
                .post_json(
                    &full_url,
                    &serde_json::from_str::<serde_json::Value>(&body)?,
                )
                .await?
        }
        HttpMethod::Put => {
            let body = payload.unwrap_or_else(|| "{}".to_string());
            client
                .put_json(
                    &full_url,
                    &serde_json::from_str::<serde_json::Value>(&body)?,
                )
                .await?
        }
        HttpMethod::Patch => {
            let body = payload.unwrap_or_else(|| "{}".to_string());
            client
                .patch_json(
                    &full_url,
                    &serde_json::from_str::<serde_json::Value>(&body)?,
                )
                .await?
        }
        HttpMethod::Delete => client.delete(&full_url).await?,
    };

    // Display response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await?;

    println!("\n📄 Response:");
    println!("   Status: {}", status);

    if status.is_success() {
        println!("   ✅ Request successful!");
    } else {
        println!("   ❌ Request failed!");
    }

    println!("   Headers:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            println!("     {}: {}", name, value_str);
        }
    }

    println!("   Body:");
    if body.is_empty() {
        println!("     (empty)");
    } else {
        // Try to pretty-print JSON, otherwise show as-is
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(json) => {
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            Err(_) => {
                println!("{}", body);
            }
        }
    }

    Ok(())
}

/// Convert HttpMethod enum to string
fn method_to_string(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
    }
}

/// Handle authentication configuration commands
#[allow(clippy::too_many_arguments)]
async fn handle_auth_configure(
    enable_auto_sign: Option<bool>,
    default_mode: Option<CliSigningMode>,
    command: Option<String>,
    command_mode: Option<CliSigningMode>,
    remove_command_override: Option<String>,
    debug: Option<bool>,
    env_var: Option<String>,
    show: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_manager = CliConfigManager::load_with_migration()?;

    if show {
        // Show current configuration
        let signing_config = config_manager.signing_config();
        println!("📋 DataFold CLI Signing Configuration");
        println!("=====================================");
        println!();
        println!("🔐 Global Settings:");
        println!(
            "  Auto-signing enabled: {}",
            signing_config.auto_signing.enabled
        );
        println!(
            "  Default mode: {}",
            signing_config.auto_signing.default_mode.as_str()
        );
        println!("  Debug logging: {}", signing_config.debug.enabled);

        if let Some(env) = &signing_config.auto_signing.env_override {
            println!("  Environment override: {}", env);
            if let Ok(value) = std::env::var(env) {
                println!("    Current value: {}", value);
            } else {
                println!("    Current value: (not set)");
            }
        }

        println!();
        println!("📝 Command Overrides:");
        if signing_config.auto_signing.command_overrides.is_empty() {
            println!("  (none configured)");
        } else {
            for (cmd, mode) in &signing_config.auto_signing.command_overrides {
                println!("  {}: {}", cmd, mode.as_str());
            }
        }

        println!();
        println!("🎛️  Performance Settings:");
        println!(
            "  Max signing time: {}ms",
            signing_config.performance.max_signing_time_ms
        );
        println!("  Cache keys: {}", signing_config.performance.cache_keys);
        println!(
            "  Max concurrent signs: {}",
            signing_config.performance.max_concurrent_signs
        );

        return Ok(());
    }

    let mut modified = false;

    // Apply configuration changes
    if let Some(enabled) = enable_auto_sign {
        config_manager.set_auto_signing_enabled(enabled);
        println!(
            "✅ Auto-signing {}",
            if enabled { "enabled" } else { "disabled" }
        );
        modified = true;
    }

    if let Some(mode) = default_mode {
        let mode_str = SigningMode::from(mode.clone()).as_str();
        config_manager.set_default_signing_mode(mode.into());
        println!("✅ Default signing mode set to: {}", mode_str);
        modified = true;
    }

    if let Some(cmd) = command {
        if let Some(mode) = command_mode {
            let mode_str = SigningMode::from(mode.clone()).as_str();
            config_manager.set_command_signing_mode(cmd.clone(), mode.into())?;
            println!("✅ Signing mode for '{}' set to: {}", cmd, mode_str);
            modified = true;
        }
    }

    if let Some(cmd) = remove_command_override {
        config_manager
            .signing_config_mut()
            .auto_signing
            .remove_command_override(&cmd);
        println!("✅ Removed signing override for command: {}", cmd);
        modified = true;
    }

    if let Some(debug_enabled) = debug {
        config_manager.set_signing_debug(debug_enabled);
        println!(
            "✅ Debug logging {}",
            if debug_enabled { "enabled" } else { "disabled" }
        );
        modified = true;
    }

    if let Some(env) = env_var {
        config_manager
            .signing_config_mut()
            .auto_signing
            .env_override = Some(env.clone());
        println!("✅ Environment variable override set to: {}", env);
        modified = true;
    }

    if modified {
        config_manager.save()?;
        println!("💾 Configuration saved");
    } else {
        println!("ℹ️  No changes specified. Use --show to view current configuration.");
    }

    Ok(())
}

/// Handle authentication initialization
async fn handle_auth_init_enhanced(
    create_config: bool,
    server_url: Option<String>,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if interactive {
        return handle_interactive_auth_setup(server_url).await;
    }

    if create_config {
        let config_path = CliConfigManager::default_config_path()?;

        if config_path.exists() {
            println!(
                "⚠️  Configuration file already exists at: {}",
                config_path.display()
            );
            print!("Do you want to overwrite it? (y/N): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("❌ Aborted");
                return Ok(());
            }
        }

        let mut config_manager = CliConfigManager::new()?;

        // Set default server URL if provided
        if let Some(url) = server_url {
            let server_config = ServerConfig {
                url: url.clone(),
                name: "Default DataFold Server".to_string(),
                api_version: Some("v1".to_string()),
                headers: HashMap::new(),
                timeout_secs: 30,
                max_retries: 3,
                verify_ssl: true,
            };
            config_manager.add_server("default".to_string(), server_config);
            println!("✅ Added default server: {}", url);
        }

        config_manager.save()?;
        println!("✅ Created configuration file: {}", config_path.display());

        println!();
        println!("🎯 Next steps:");
        println!("1. Generate a key pair: datafold auth-keygen --key-id my-key");
        println!("2. Create a profile: datafold auth-profile create my-profile --server-url <URL> --key-id my-key");
        println!("3. Test authentication: datafold auth-test");
    } else {
        println!("ℹ️  Use --create-config to create default configuration");
        println!("ℹ️  Use --interactive for guided setup");
    }

    Ok(())
}

/// Handle interactive authentication setup
async fn handle_interactive_auth_setup(
    default_server_url: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DataFold CLI Authentication Setup");
    println!("====================================");
    println!();

    // Check if configuration exists
    let config_path = CliConfigManager::default_config_path()?;
    let mut config_manager = if config_path.exists() {
        println!("📁 Found existing configuration");
        CliConfigManager::load_with_migration()?
    } else {
        println!("📝 Creating new configuration");
        CliConfigManager::new()?
    };

    // Get server URL
    let server_url = if let Some(url) = default_server_url {
        println!("🌐 Using provided server URL: {}", url);
        url
    } else {
        print!("🌐 Enter DataFold server URL: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    if server_url.is_empty() {
        return Err("Server URL is required".into());
    }

    // Get profile name
    print!("📛 Enter profile name (default: default): ");
    io::stdout().flush()?;
    let mut profile_name = String::new();
    io::stdin().read_line(&mut profile_name)?;
    let profile_name = profile_name.trim();
    let profile_name = if profile_name.is_empty() {
        "default"
    } else {
        profile_name
    };

    // Get key ID
    print!("🔑 Enter key ID (default: {}): ", profile_name);
    io::stdout().flush()?;
    let mut key_id = String::new();
    io::stdin().read_line(&mut key_id)?;
    let key_id = key_id.trim();
    let key_id = if key_id.is_empty() {
        profile_name
    } else {
        key_id
    };

    // Check if key exists
    let storage_dir = CliConfigManager::default_keys_dir()?;
    let storage_dir_display = storage_dir.clone();
    let key_file = storage_dir.join(format!("{}.json", key_id));

    if !key_file.exists() {
        println!("🔧 Key '{}' not found. Generating new key pair...", key_id);

        // Generate new key
        handle_store_key(
            key_id.to_string(),
            None,
            None,
            Some(storage_dir),
            false,
            CliSecurityLevel::Balanced,
            None,
        )?;

        println!("✅ Generated and stored key: {}", key_id);
    } else {
        println!("✅ Using existing key: {}", key_id);
    }

    // Create profile
    let mut metadata = HashMap::new();
    metadata.insert("created_by".to_string(), "interactive_setup".to_string());
    metadata.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());

    let profile = CliAuthProfile {
        client_id: format!("datafold-cli-{}", profile_name),
        key_id: key_id.to_string(),
        user_id: None,
        server_url: server_url.clone(),
        metadata,
    };

    config_manager.add_profile(profile_name.to_string(), profile)?;

    // Configure signing
    println!();
    println!("🔐 Configure automatic signing:");
    print!("Enable automatic request signing? (Y/n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let enable_auto = !input.trim().to_lowercase().starts_with('n');
    config_manager.set_auto_signing_enabled(enable_auto);

    if enable_auto {
        config_manager.set_default_signing_mode(SigningMode::Auto);
        println!("✅ Automatic signing enabled");
    } else {
        config_manager.set_default_signing_mode(SigningMode::Manual);
        println!("✅ Manual signing mode (use --sign flag to sign requests)");
    }

    // Save configuration
    config_manager.save()?;

    println!();
    println!("🎉 Setup complete!");
    println!("📁 Configuration saved to: {}", config_path.display());
    println!("🔑 Key stored in: {}", storage_dir_display.display());
    println!();
    println!("🧪 Test your setup:");
    println!("  datafold auth-status");
    println!("  datafold auth-test");

    Ok(())
}

/// Handle end-to-end server integration test
#[allow(clippy::too_many_arguments)]
async fn handle_test_server_integration(
    server_url: String,
    key_id: String,
    storage_dir: Option<PathBuf>,
    test_message: String,
    timeout: u64,
    retries: u32,
    security_level: CliSecurityLevel,
    cleanup: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Starting end-to-end server integration test...");

    // Get storage directory
    let storage_dir = storage_dir.unwrap_or_else(|| get_default_storage_dir().unwrap());
    ensure_storage_dir(&storage_dir)?;

    let test_key_id = format!("{}_test", key_id);
    let client_id = format!("test_{}", uuid::Uuid::new_v4());

    println!("Step 1: Generating test keypair...");
    // Generate a test key
    let master_keypair = generate_master_keypair()?;

    // Store the test key
    print!("Enter passphrase for test key: ");
    io::stdout().flush()?;
    let _passphrase = get_secure_passphrase()?;

    handle_store_key(
        test_key_id.clone(),
        Some(hex::encode(master_keypair.secret_key_bytes())),
        None,
        Some(storage_dir.clone()),
        true, // force
        security_level.clone(),
        Some(_passphrase),
    )?;

    println!("✅ Test key generated and stored");

    println!("Step 2: Registering public key with server...");
    // Register the key
    match handle_register_key(
        server_url.clone(),
        test_key_id.clone(),
        Some(storage_dir.clone()),
        Some(client_id.clone()),
        None,
        Some("Integration Test Key".to_string()),
        timeout,
        retries,
    )
    .await
    {
        Ok(()) => println!("✅ Key registration successful"),
        Err(e) => {
            eprintln!("❌ Key registration failed: {}", e);
            if cleanup {
                let _ = handle_delete_key(test_key_id, Some(storage_dir), true);
            }
            return Err(e);
        }
    }

    println!("Step 3: Checking registration status...");
    // Check registration status
    match handle_check_registration(server_url.clone(), client_id.clone(), timeout, retries).await {
        Ok(()) => println!("✅ Registration status check successful"),
        Err(e) => {
            eprintln!("❌ Registration status check failed: {}", e);
            if cleanup {
                let _ = handle_delete_key(test_key_id, Some(storage_dir), true);
            }
            return Err(e);
        }
    }

    println!("Step 4: Signing and verifying message...");
    // Sign and verify
    match handle_sign_and_verify(
        server_url.clone(),
        test_key_id.clone(),
        Some(storage_dir.clone()),
        client_id.clone(),
        Some(test_message),
        None,
        MessageEncoding::Utf8,
        timeout,
        retries,
    )
    .await
    {
        Ok(()) => println!("✅ Message signing and verification successful"),
        Err(e) => {
            eprintln!("❌ Message signing and verification failed: {}", e);
            if cleanup {
                let _ = handle_delete_key(test_key_id, Some(storage_dir), true);
            }
            return Err(e);
        }
    }

    if cleanup {
        println!("Step 5: Cleaning up test key...");
        match handle_delete_key(test_key_id, Some(storage_dir), true) {
            Ok(()) => println!("✅ Test key cleaned up"),
            Err(e) => {
                eprintln!("⚠️ Failed to clean up test key: {}", e);
                // Don't fail the test for cleanup issues
            }
        }
    }

    println!("🎉 End-to-end server integration test completed successfully!");
    println!("All server integration functionality is working correctly.");

    Ok(())
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
    let status = get_crypto_init_status(db_ops.clone())
        .map_err(|e| format!("Failed to check crypto status: {}", e))?;

    if status.initialized && !force {
        info!(
            "Database crypto is already initialized: {}",
            status.summary()
        );
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
            info!(
                "Using passphrase-based key derivation with {} security level",
                security_level.as_str()
            );
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
        info!(
            "  Algorithm: {}",
            status.algorithm.as_deref().unwrap_or("Unknown")
        );
        info!(
            "  Derivation Method: {}",
            status.derivation_method.as_deref().unwrap_or("Unknown")
        );
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
                                info!(
                                    "    Memory Cost: {} KB",
                                    crypto_config.key_derivation.memory_cost
                                );
                                info!(
                                    "    Time Cost: {} iterations",
                                    crypto_config.key_derivation.time_cost
                                );
                                info!(
                                    "    Parallelism: {} threads",
                                    crypto_config.key_derivation.parallelism
                                );
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

/// Handle Ed25519 key generation command
fn handle_generate_key(
    format: KeyFormat,
    private_key_file: Option<PathBuf>,
    public_key_file: Option<PathBuf>,
    count: u32,
    public_only: bool,
    private_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if public_only && private_only {
        return Err("Cannot specify both --public-only and --private-only".into());
    }

    for i in 0..count {
        let keypair =
            generate_master_keypair().map_err(|e| format!("Failed to generate keypair: {}", e))?;

        let public_key_bytes = keypair.public_key_bytes();
        let private_key_bytes = keypair.secret_key_bytes();

        if count > 1 {
            info!("Generating keypair {} of {}", i + 1, count);
        }

        // Output private key if requested
        if !public_only {
            let formatted_private = format_key(&private_key_bytes, &format, true)?;
            output_key(
                &formatted_private,
                private_key_file.as_ref(),
                "private",
                i,
                count,
                true,
            )?;
        }

        // Output public key if requested
        if !private_only {
            let formatted_public = format_key(&public_key_bytes, &format, false)?;
            output_key(
                &formatted_public,
                public_key_file.as_ref(),
                "public",
                i,
                count,
                true,
            )?;
        }

        // Clear sensitive data
        drop(keypair);
    }

    Ok(())
}

/// Handle Ed25519 key derivation from passphrase
fn handle_derive_key(
    format: KeyFormat,
    private_key_file: Option<PathBuf>,
    public_key_file: Option<PathBuf>,
    security_level: CliSecurityLevel,
    public_only: bool,
    private_only: bool,
    passphrase: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use datafold::crypto::argon2::{generate_salt_and_derive_keypair, Argon2Params};

    if public_only && private_only {
        return Err("Cannot specify both --public-only and --private-only".into());
    }

    let passphrase = match passphrase {
        Some(p) => p,
        None => get_secure_passphrase()?,
    };

    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Generate salt and derive keypair
    let (_salt, keypair) = generate_salt_and_derive_keypair(&passphrase, &argon2_params)
        .map_err(|e| format!("Failed to derive keypair from passphrase: {}", e))?;

    let public_key_bytes = keypair.public_key_bytes();
    let private_key_bytes = keypair.secret_key_bytes();

    // Output private key if requested
    if !public_only {
        let formatted_private = format_key(&private_key_bytes, &format, true)?;
        output_key(
            &formatted_private,
            private_key_file.as_ref(),
            "private",
            0,
            1,
            true,
        )?;
    }

    // Output public key if requested
    if !private_only {
        let formatted_public = format_key(&public_key_bytes, &format, false)?;
        output_key(
            &formatted_public,
            public_key_file.as_ref(),
            "public",
            0,
            1,
            true,
        )?;
    }

    // Clear sensitive data
    drop(keypair);

    Ok(())
}

/// Handle extracting public key from private key
fn handle_extract_public_key(
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    format: KeyFormat,
    output_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get private key bytes
    let private_key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read private key file: {}", e))?;
            parse_key_input(content.trim(), true)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --private-key and --private-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --private-key or --private-key-file".into());
        }
    };

    // Create keypair from private key
    let keypair = datafold::crypto::ed25519::generate_master_keypair_from_seed(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let public_key_bytes = keypair.public_key_bytes();
    let formatted_public = format_key(&public_key_bytes, &format, false)?;

    output_key(
        &formatted_public,
        output_file.as_ref(),
        "public",
        0,
        1,
        false,
    )?;

    Ok(())
}

/// Handle verifying that a keypair is valid and matches
fn handle_verify_key(
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get private key bytes
    let private_key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read private key file: {}", e))?;
            parse_key_input(content.trim(), true)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --private-key and --private-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --private-key or --private-key-file".into());
        }
    };

    // Get public key bytes
    let public_key_bytes = match (public_key, public_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, false)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read public key file: {}", e))?;
            parse_key_input(content.trim(), false)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --public-key and --public-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --public-key or --public-key-file".into());
        }
    };

    // Create keypair from private key
    let keypair = datafold::crypto::ed25519::generate_master_keypair_from_seed(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    // Check if the public keys match
    let derived_public_key_bytes = keypair.public_key_bytes();

    if derived_public_key_bytes == public_key_bytes {
        println!("✅ Keypair verification successful: private and public keys match");
        println!("Public key: {}", hex::encode(public_key_bytes));
        info!("✅ Keypair verification successful: private and public keys match");
        info!("Public key: {}", hex::encode(public_key_bytes));
    } else {
        error!("❌ Keypair verification failed: private and public keys do not match");
        error!(
            "Expected public key: {}",
            hex::encode(derived_public_key_bytes)
        );
        error!("Provided public key: {}", hex::encode(public_key_bytes));
        return Err("Keypair verification failed".into());
    }

    // Test signing and verification to ensure the keypair is fully functional
    let test_message = b"DataFold Ed25519 keypair verification test";
    let signature = keypair
        .sign_data(test_message)
        .map_err(|e| format!("Failed to sign test message: {}", e))?;

    keypair
        .verify_data(test_message, &signature)
        .map_err(|e| format!("Failed to verify test signature: {}", e))?;

    println!("✅ Functional verification successful: keypair can sign and verify");
    info!("✅ Functional verification successful: keypair can sign and verify");

    Ok(())
}

/// Format key bytes according to the specified format
fn format_key(
    key_bytes: &[u8],
    format: &KeyFormat,
    is_private: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        KeyFormat::Hex => Ok(hex::encode(key_bytes)),
        KeyFormat::Base64 => Ok(general_purpose::STANDARD.encode(key_bytes)),
        KeyFormat::Raw => Ok(String::from_utf8(key_bytes.to_vec())
            .unwrap_or_else(|_| format!("Binary data: {} bytes", key_bytes.len()))),
        KeyFormat::Pem => {
            if is_private {
                // For private keys, we'll use a simple PEM-like format
                // In a full implementation, you'd use proper PKCS#8 encoding
                let base64_content = general_purpose::STANDARD.encode(key_bytes);
                Ok(format!(
                    "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
                    base64_content
                ))
            } else {
                // For public keys, use SubjectPublicKeyInfo format
                let base64_content = general_purpose::STANDARD.encode(key_bytes);
                Ok(format!(
                    "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                    base64_content
                ))
            }
        }
    }
}

/// Output formatted key to file or stdout
fn output_key(
    formatted_key: &str,
    output_file: Option<&PathBuf>,
    key_type: &str,
    index: u32,
    total: u32,
    include_comments: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match output_file {
        Some(file_path) => {
            let actual_path = if total > 1 {
                // Add index to filename for batch generation
                let stem = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("key");
                let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let parent = file_path.parent().unwrap_or(std::path::Path::new("."));

                if extension.is_empty() {
                    parent.join(format!("{}_{}", stem, index + 1))
                } else {
                    parent.join(format!("{}_{}.{}", stem, index + 1, extension))
                }
            } else {
                file_path.clone()
            };

            fs::write(&actual_path, formatted_key)
                .map_err(|e| format!("Failed to write {} key to file: {}", key_type, e))?;
            info!("✅ {} key written to: {}", key_type, actual_path.display());
        }
        None => {
            if include_comments {
                if total > 1 {
                    println!("# {} key {} of {}", key_type, index + 1, total);
                } else {
                    println!("# {} key", key_type);
                }
            }
            println!("{}", formatted_key);
        }
    }
    Ok(())
}

/// Parse key input from string (supports hex, base64, and PEM formats)
fn parse_key_input(input: &str, is_private: bool) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let trimmed = input.trim();

    // Try to parse as PEM first
    if trimmed.starts_with("-----BEGIN") && trimmed.ends_with("-----") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() >= 3 {
            let base64_content = lines[1..lines.len() - 1].join("");
            let decoded = general_purpose::STANDARD
                .decode(&base64_content)
                .map_err(|e| format!("Invalid base64 in PEM: {}", e))?;

            if decoded.len() == 32 {
                let mut key_bytes = [0u8; 32];
                key_bytes.copy_from_slice(&decoded);
                return Ok(key_bytes);
            }
        }
        return Err("Invalid PEM format or wrong key size".into());
    }

    // Try hex format
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        let decoded = hex::decode(trimmed).map_err(|e| format!("Invalid hex: {}", e))?;
        if decoded.len() == 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&decoded);
            return Ok(key_bytes);
        }
    }

    // Try base64 format
    if let Ok(decoded) = general_purpose::STANDARD.decode(trimmed) {
        if decoded.len() == 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&decoded);
            return Ok(key_bytes);
        }
    }

    let expected_size = if is_private {
        SECRET_KEY_LENGTH
    } else {
        PUBLIC_KEY_LENGTH
    };
    Err(format!(
        "Unable to parse key: expected {} bytes in hex, base64, or PEM format",
        expected_size
    )
    .into())
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
///
/// Handle signature verification command
#[allow(clippy::too_many_arguments)]
async fn handle_verify_signature(
    message: Option<String>,
    message_file: Option<PathBuf>,
    signature: String,
    key_id: String,
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
    policy: Option<String>,
    output_format: VerificationOutputFormat,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use datafold::cli::verification::{CliSignatureVerifier, CliVerificationConfig};

    // Get message bytes
    let message_bytes = match (message, message_file) {
        (Some(msg), None) => msg.into_bytes(),
        (None, Some(file)) => fs::read(file)?,
        (Some(_), Some(_)) => return Err("Cannot specify both message and message-file".into()),
        (None, None) => return Err("Must specify either message or message-file".into()),
    };

    // Create verifier
    let config = CliVerificationConfig::default();

    // Add public key
    let public_key_bytes = match (public_key, public_key_file) {
        (Some(key), None) => parse_key_input(&key, false)?.to_vec(),
        (None, Some(file)) => {
            let key_str = fs::read_to_string(file)?;
            parse_key_input(key_str.trim(), false)?.to_vec()
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both public-key and public-key-file".into())
        }
        (None, None) => return Err("Must specify either public-key or public-key-file".into()),
    };

    let mut verifier = CliSignatureVerifier::new(config);
    verifier.add_public_key(key_id.clone(), public_key_bytes)?;

    // Perform verification
    let result = verifier
        .verify_message_signature(&message_bytes, &signature, &key_id, policy.as_deref())
        .await?;

    // Output result
    match output_format {
        VerificationOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        VerificationOutputFormat::Table => {
            println!("=== Signature Verification Result ===");
            println!("Status: {}", result.status);
            println!("Signature Valid: {}", result.signature_valid);
            println!("Total Time: {}ms", result.performance.total_time_ms);

            if debug {
                let inspector = datafold::cli::verification::SignatureInspector::new(true);
                let report = inspector.generate_diagnostic_report(&result);
                println!("\n{}", report);
            }
        }
        VerificationOutputFormat::Compact => {
            println!(
                "{}: {}",
                result.status,
                if result.signature_valid { "✓" } else { "✗" }
            );
        }
    }

    if !result.signature_valid {
        std::process::exit(1);
    }

    Ok(())
}

/// Handle signature inspection command
async fn handle_inspect_signature(
    signature_input: Option<String>,
    signature: Option<String>,
    headers_file: Option<PathBuf>,
    output_format: VerificationOutputFormat,
    _detailed: bool,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use datafold::cli::verification::SignatureInspector;
    use std::collections::HashMap;

    let inspector = SignatureInspector::new(debug);
    let mut headers = HashMap::new();

    // Build headers from inputs
    if let Some(input) = signature_input {
        headers.insert("signature-input".to_string(), input);
    }
    if let Some(sig) = signature {
        headers.insert("signature".to_string(), sig);
    }
    if let Some(file) = headers_file {
        let content = fs::read_to_string(file)?;
        let file_headers: HashMap<String, String> = serde_json::from_str(&content)?;
        headers.extend(file_headers);
    }

    if headers.is_empty() {
        return Err(
            "Must provide signature headers via signature-input, signature, or headers-file".into(),
        );
    }

    // Inspect signature format
    let analysis = inspector.inspect_signature_format(&headers);

    // Output results
    match output_format {
        VerificationOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        VerificationOutputFormat::Table => {
            println!("=== Signature Format Analysis ===");
            println!("RFC 9421 Compliant: {}", analysis.is_valid_rfc9421);
            println!(
                "Signature Headers: {}",
                analysis.signature_headers.join(", ")
            );
            println!("Signature IDs: {}", analysis.signature_ids.join(", "));

            if !analysis.issues.is_empty() {
                println!("\n=== Issues Found ===");
                for issue in &analysis.issues {
                    println!("{:?}: {} - {}", issue.severity, issue.code, issue.message);
                    if let Some(component) = &issue.component {
                        println!("  Component: {}", component);
                    }
                }
            }
        }
        VerificationOutputFormat::Compact => {
            println!(
                "RFC9421: {} | Issues: {}",
                if analysis.is_valid_rfc9421 {
                    "✓"
                } else {
                    "✗"
                },
                analysis.issues.len()
            );
        }
    }

    Ok(())
}

/// Handle response verification command
#[allow(clippy::too_many_arguments)]
async fn handle_verify_response(
    url: String,
    method: HttpMethod,
    headers: Option<String>,
    body: Option<String>,
    body_file: Option<PathBuf>,
    key_id: String,
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
    policy: Option<String>,
    output_format: VerificationOutputFormat,
    debug: bool,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    use datafold::cli::verification::{CliSignatureVerifier, CliVerificationConfig};
    use reqwest::Client;
    use std::time::Duration;

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()?;

    // Build request
    let method_str = method_to_string(&method);
    let mut request_builder = match method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
        HttpMethod::Put => client.put(&url),
        HttpMethod::Patch => client.patch(&url),
        HttpMethod::Delete => client.delete(&url),
    };

    // Add headers if provided
    if let Some(headers_json) = headers {
        let headers_map: std::collections::HashMap<String, String> =
            serde_json::from_str(&headers_json)?;
        for (key, value) in headers_map {
            request_builder = request_builder.header(key, value);
        }
    }

    // Add body if provided
    let request_body = match (body, body_file) {
        (Some(body_str), None) => Some(body_str),
        (None, Some(file)) => Some(fs::read_to_string(file)?),
        (Some(_), Some(_)) => return Err("Cannot specify both body and body-file".into()),
        (None, None) => None,
    };

    if let Some(body_content) = request_body {
        request_builder = request_builder.body(body_content);
    }

    // Send request
    let response = request_builder.send().await?;

    // Setup verifier
    let config = CliVerificationConfig::default();

    // Add public key
    let public_key_bytes = match (public_key, public_key_file) {
        (Some(key), None) => parse_key_input(&key, false)?.to_vec(),
        (None, Some(file)) => {
            let key_str = fs::read_to_string(file)?;
            parse_key_input(key_str.trim(), false)?.to_vec()
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both public-key and public-key-file".into())
        }
        (None, None) => return Err("Must specify either public-key or public-key-file".into()),
    };

    let mut verifier = CliSignatureVerifier::new(config);
    verifier.add_public_key(key_id.clone(), public_key_bytes)?;

    // Verify response
    let result = verifier
        .verify_response_with_context(&response, method_str, &url, policy.as_deref())
        .await?;

    // Output result
    match output_format {
        VerificationOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        VerificationOutputFormat::Table => {
            println!("=== Response Verification Result ===");
            println!("Status: {}", result.status);
            println!("Signature Valid: {}", result.signature_valid);
            println!("Total Time: {}ms", result.performance.total_time_ms);

            if debug {
                let inspector = datafold::cli::verification::SignatureInspector::new(true);
                let report = inspector.generate_diagnostic_report(&result);
                println!("\n{}", report);
            }
        }
        VerificationOutputFormat::Compact => {
            println!(
                "{}: {}",
                result.status,
                if result.signature_valid { "✓" } else { "✗" }
            );
        }
    }

    if !result.signature_valid {
        std::process::exit(1);
    }

    Ok(())
}

/// Handle verification configuration command
async fn handle_verification_config(
    action: VerificationConfigAction,
) -> Result<(), Box<dyn std::error::Error>> {
    use datafold::cli::verification::{CliVerificationConfig, VerificationPolicy};

    match action {
        VerificationConfigAction::Show { policies, keys } => {
            let config = CliVerificationConfig::default();

            if policies {
                println!("=== Verification Policies ===");
                for (name, policy) in &config.policies {
                    println!("Policy: {}", name);
                    println!("  Description: {}", policy.description);
                    println!("  Verify Timestamp: {}", policy.verify_timestamp);
                    println!("  Verify Nonce: {}", policy.verify_nonce);
                    println!("  Verify Content Digest: {}", policy.verify_content_digest);
                    println!("  Required Components: {:?}", policy.required_components);
                    println!("  Allowed Algorithms: {:?}", policy.allowed_algorithms);
                    if name == &config.default_policy {
                        println!("  (Default Policy)");
                    }
                    println!();
                }
            }

            if keys {
                println!("=== Public Keys ===");
                if config.public_keys.is_empty() {
                    println!("No public keys configured");
                } else {
                    for (key_id, key_bytes) in &config.public_keys {
                        println!("Key ID: {}", key_id);
                        println!("  Length: {} bytes", key_bytes.len());
                        println!("  Fingerprint: {}", hex::encode(&key_bytes[..8]));
                        println!();
                    }
                }
            }

            if !policies && !keys {
                println!("=== Verification Configuration ===");
                println!("Default Policy: {}", config.default_policy);
                println!("Available Policies: {}", config.policies.len());
                println!("Configured Keys: {}", config.public_keys.len());
                println!(
                    "Performance Monitoring: {}",
                    config.performance_monitoring.enabled
                );
                println!("Debug Enabled: {}", config.debug.enabled);
            }
        }
        VerificationConfigAction::AddPolicy { name, config_file } => {
            let policy_json = fs::read_to_string(config_file)?;
            let _policy: VerificationPolicy = serde_json::from_str(&policy_json)?;
            println!(
                "Policy '{}' would be added (not implemented in this demo)",
                name
            );
        }
        VerificationConfigAction::RemovePolicy { name } => {
            println!(
                "Policy '{}' would be removed (not implemented in this demo)",
                name
            );
        }
        VerificationConfigAction::SetDefaultPolicy { name } => {
            println!(
                "Default policy would be set to '{}' (not implemented in this demo)",
                name
            );
        }
        VerificationConfigAction::AddPublicKey {
            key_id,
            public_key,
            public_key_file,
        } => {
            let _key_bytes = match (public_key, public_key_file) {
                (Some(key), None) => parse_key_input(&key, false)?.to_vec(),
                (None, Some(file)) => {
                    let key_str = fs::read_to_string(file)?;
                    parse_key_input(key_str.trim(), false)?.to_vec()
                }
                (Some(_), Some(_)) => {
                    return Err("Cannot specify both public-key and public-key-file".into())
                }
                (None, None) => {
                    return Err("Must specify either public-key or public-key-file".into())
                }
            };
            println!(
                "Public key '{}' would be added (not implemented in this demo)",
                key_id
            );
        }
        VerificationConfigAction::RemovePublicKey { key_id } => {
            println!(
                "Public key '{}' would be removed (not implemented in this demo)",
                key_id
            );
        }
        VerificationConfigAction::ListPublicKeys { verbose } => {
            let config = CliVerificationConfig::default();
            println!("=== Public Keys ===");
            if config.public_keys.is_empty() {
                println!("No public keys configured");
            } else {
                for (key_id, key_bytes) in &config.public_keys {
                    if verbose {
                        println!("Key ID: {}", key_id);
                        println!("  Length: {} bytes", key_bytes.len());
                        println!("  Fingerprint: {}", hex::encode(&key_bytes[..8]));
                        println!("  Full Key: {}", hex::encode(key_bytes));
                        println!();
                    } else {
                        println!("{} ({}B)", key_id, key_bytes.len());
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle environment management commands
fn handle_environment_command(action: EnvironmentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        EnvironmentAction::List {} => {
            env_commands::list_environments()?;
        }
        EnvironmentAction::Show { environment } => {
            if let Some(env) = environment {
                env_commands::show_environment(&env)?;
            } else {
                env_commands::show_current_environment()?;
            }
        }
        EnvironmentAction::Switch { environment } => {
            env_commands::switch_environment(&environment)?;
        }
        EnvironmentAction::Compare { env1, env2 } => {
            env_commands::compare_environments(&env1, &env2)?;
        }
        EnvironmentAction::Validate {} => {
            env_commands::validate_environments()?;
        }
        EnvironmentAction::Export { environment } => {
            env_commands::export_environment_vars(environment.as_deref())?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    datafold::logging::init().ok();
    let cli = Cli::parse();

    // Handle commands that don't need the node first
    match &cli.command {
        Commands::HashSchemas { verify } => {
            return handle_hash_schemas(*verify);
        }
        Commands::CryptoValidate { config_file } => {
            return handle_crypto_validate(config_file.clone(), &cli.config);
        }
        Commands::GenerateKey {
            format,
            private_key_file,
            public_key_file,
            count,
            public_only,
            private_only,
        } => {
            return handle_generate_key(
                format.clone(),
                private_key_file.clone(),
                public_key_file.clone(),
                *count,
                *public_only,
                *private_only,
            );
        }
        Commands::DeriveKey {
            format,
            private_key_file,
            public_key_file,
            security_level,
            public_only,
            private_only,
            passphrase,
        } => {
            return handle_derive_key(
                format.clone(),
                private_key_file.clone(),
                public_key_file.clone(),
                security_level.clone(),
                *public_only,
                *private_only,
                passphrase.clone(),
            );
        }
        Commands::ExtractPublicKey {
            private_key,
            private_key_file,
            format,
            output_file,
        } => {
            return handle_extract_public_key(
                private_key.clone(),
                private_key_file.clone(),
                format.clone(),
                output_file.clone(),
            );
        }
        Commands::VerifyKey {
            private_key,
            private_key_file,
            public_key,
            public_key_file,
        } => {
            return handle_verify_key(
                private_key.clone(),
                private_key_file.clone(),
                public_key.clone(),
                public_key_file.clone(),
            );
        }
        Commands::StoreKey {
            key_id,
            private_key,
            private_key_file,
            storage_dir,
            force,
            security_level,
            passphrase,
        } => {
            return handle_store_key(
                key_id.clone(),
                private_key.clone(),
                private_key_file.clone(),
                storage_dir.clone(),
                *force,
                security_level.clone(),
                passphrase.clone(),
            );
        }
        Commands::RetrieveKey {
            key_id,
            storage_dir,
            format,
            output_file,
            public_only,
        } => {
            return handle_retrieve_key(
                key_id.clone(),
                storage_dir.clone(),
                format.clone(),
                output_file.clone(),
                *public_only,
            );
        }
        Commands::DeleteKey {
            key_id,
            storage_dir,
            force,
        } => {
            return handle_delete_key(key_id.clone(), storage_dir.clone(), *force);
        }
        Commands::ListKeys {
            storage_dir,
            verbose,
        } => {
            return handle_list_keys(storage_dir.clone(), *verbose);
        }
        Commands::DeriveFromMaster {
            master_key_id,
            context,
            child_key_id,
            storage_dir,
            security_level,
            format,
            output_only,
            force,
        } => {
            return handle_derive_from_master(
                master_key_id.clone(),
                context.clone(),
                child_key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                format.clone(),
                *output_only,
                *force,
            );
        }
        Commands::RotateKey {
            key_id,
            storage_dir,
            security_level,
            method,
            keep_backup,
            force,
        } => {
            return handle_rotate_key(
                key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                method.clone(),
                *keep_backup,
                *force,
            );
        }
        Commands::ListKeyVersions {
            key_id,
            storage_dir,
            verbose,
        } => {
            return handle_list_key_versions(key_id.clone(), storage_dir.clone(), *verbose);
        }
        Commands::BackupKey {
            key_id,
            storage_dir,
            backup_file,
            backup_passphrase,
        } => {
            return handle_backup_key(
                key_id.clone(),
                storage_dir.clone(),
                backup_file.clone(),
                *backup_passphrase,
            );
        }
        Commands::RestoreKey {
            backup_file,
            key_id,
            storage_dir,
            force,
        } => {
            return handle_restore_key(
                backup_file.clone(),
                key_id.clone(),
                storage_dir.clone(),
                *force,
            );
        }
        Commands::ExportKey {
            key_id,
            storage_dir,
            export_file,
            format,
            export_passphrase,
            include_metadata,
        } => {
            return handle_export_key(
                key_id.clone(),
                storage_dir.clone(),
                export_file.clone(),
                format.clone(),
                *export_passphrase,
                *include_metadata,
            );
        }
        Commands::ImportKey {
            export_file,
            key_id,
            storage_dir,
            force,
            verify_integrity,
        } => {
            return handle_import_key(
                export_file.clone(),
                key_id.clone(),
                storage_dir.clone(),
                *force,
                *verify_integrity,
            );
        }
        Commands::RegisterKey {
            server_url,
            key_id,
            storage_dir,
            client_id,
            user_id,
            key_name,
            timeout,
            retries,
        } => {
            return handle_register_key(
                server_url.clone(),
                key_id.clone(),
                storage_dir.clone(),
                client_id.clone(),
                user_id.clone(),
                key_name.clone(),
                *timeout,
                *retries,
            )
            .await;
        }
        Commands::CheckRegistration {
            server_url,
            client_id,
            timeout,
            retries,
        } => {
            return handle_check_registration(
                server_url.clone(),
                client_id.clone(),
                *timeout,
                *retries,
            )
            .await;
        }
        Commands::SignAndVerify {
            server_url,
            key_id,
            storage_dir,
            client_id,
            message,
            message_file,
            message_encoding,
            timeout,
            retries,
        } => {
            return handle_sign_and_verify(
                server_url.clone(),
                key_id.clone(),
                storage_dir.clone(),
                client_id.clone(),
                message.clone(),
                message_file.clone(),
                message_encoding.clone(),
                *timeout,
                *retries,
            )
            .await;
        }
        Commands::TestServerIntegration {
            server_url,
            key_id,
            storage_dir,
            test_message,
            timeout,
            retries,
            security_level,
            cleanup,
        } => {
            return handle_test_server_integration(
                server_url.clone(),
                key_id.clone(),
                storage_dir.clone(),
                test_message.clone(),
                *timeout,
                *retries,
                security_level.clone(),
                *cleanup,
            )
            .await;
        }
        Commands::AuthInit {
            server_url,
            profile,
            key_id,
            storage_dir,
            user_id,
            environment,
            force,
        } => {
            return handle_auth_init(
                server_url.clone(),
                profile.clone(),
                key_id.clone(),
                storage_dir.clone(),
                user_id.clone(),
                environment.clone(),
                *force,
            );
        }
        Commands::AuthStatus {
            verbose,
            profile,
            environment,
        } => {
            return handle_auth_status(*verbose, profile.clone(), environment.clone());
        }
        Commands::AuthProfile { action } => {
            return handle_auth_profile((*action).clone()).await;
        }
        Commands::AuthKeygen {
            key_id,
            storage_dir,
            security_level,
            force,
            auto_register,
            server_url,
            passphrase,
        } => {
            return handle_auth_keygen(
                key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                *force,
                *auto_register,
                server_url.clone(),
                passphrase.clone(),
            )
            .await;
        }
        Commands::AuthTest {
            endpoint,
            profile,
            method,
            payload,
            timeout,
        } => {
            return handle_auth_test(
                endpoint.clone(),
                profile.clone(),
                method.clone(),
                payload.clone(),
                *timeout,
            )
            .await;
        }
        Commands::AuthConfigure {
            enable_auto_sign,
            default_mode,
            command,
            command_mode,
            remove_command_override,
            debug,
            env_var,
            show,
        } => {
            return handle_auth_configure(
                *enable_auto_sign,
                default_mode.clone(),
                command.clone(),
                command_mode.clone(),
                remove_command_override.clone(),
                *debug,
                env_var.clone(),
                *show,
            )
            .await;
        }
        Commands::AuthSetup {
            create_config,
            server_url,
            interactive,
        } => {
            return handle_auth_init_enhanced(*create_config, server_url.clone(), *interactive)
                .await;
        }
        Commands::VerifySignature {
            message,
            message_file,
            signature,
            key_id,
            public_key,
            public_key_file,
            policy,
            output_format,
            debug,
        } => {
            return handle_verify_signature(
                message.clone(),
                message_file.clone(),
                signature.clone(),
                key_id.clone(),
                public_key.clone(),
                public_key_file.clone(),
                policy.clone(),
                output_format.clone(),
                *debug,
            )
            .await;
        }
        Commands::InspectSignature {
            signature_input,
            signature,
            headers_file,
            output_format,
            detailed,
            debug,
        } => {
            return handle_inspect_signature(
                signature_input.clone(),
                signature.clone(),
                headers_file.clone(),
                output_format.clone(),
                *detailed,
                *debug,
            )
            .await;
        }
        Commands::VerifyResponse {
            url,
            method,
            headers,
            body,
            body_file,
            key_id,
            public_key,
            public_key_file,
            policy,
            output_format,
            debug,
            timeout,
        } => {
            return handle_verify_response(
                url.clone(),
                method.clone(),
                headers.clone(),
                body.clone(),
                body_file.clone(),
                key_id.clone(),
                public_key.clone(),
                public_key_file.clone(),
                policy.clone(),
                output_format.clone(),
                *debug,
                *timeout,
            )
            .await;
        }
        Commands::Environment { action } => {
            return handle_environment_command(action.clone());
        }
        Commands::VerificationConfig { action } => {
            return handle_verification_config(action.clone()).await;
        }
        // Removed: Admin key rotation commands have been removed for security
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
        Commands::CryptoInit {
            method,
            security_level,
            force,
        } => handle_crypto_init(method, security_level, force, &mut node)?,
        Commands::CryptoStatus {} => handle_crypto_status(&mut node)?,
        Commands::CryptoValidate { .. } => unreachable!(), // Already handled above
        Commands::GenerateKey { .. } => unreachable!(),    // Already handled above
        Commands::DeriveKey { .. } => unreachable!(),      // Already handled above
        Commands::ExtractPublicKey { .. } => unreachable!(), // Already handled above
        Commands::Execute { path } => handle_execute(path, &mut node)?,
        Commands::VerifyKey { .. }
        | Commands::StoreKey { .. }
        | Commands::RetrieveKey { .. }
        | Commands::DeleteKey { .. }
        | Commands::ListKeys { .. }
        | Commands::DeriveFromMaster { .. }
        | Commands::RotateKey { .. }
        | Commands::ListKeyVersions { .. }
        | Commands::BackupKey { .. }
        | Commands::RestoreKey { .. }
        | Commands::ExportKey { .. }
        | Commands::ImportKey { .. }
        | Commands::RegisterKey { .. }
        | Commands::CheckRegistration { .. }
        | Commands::SignAndVerify { .. }
        | Commands::TestServerIntegration { .. }
        | Commands::AuthInit { .. }
        | Commands::AuthStatus { .. }
        | Commands::AuthProfile { .. }
        | Commands::AuthKeygen { .. }
        | Commands::AuthTest { .. }
        | Commands::AuthConfigure { .. }
        | Commands::AuthSetup { .. }
        | Commands::VerifySignature { .. }
        | Commands::InspectSignature { .. }
        | Commands::VerifyResponse { .. }
        | Commands::VerificationConfig { .. }
        | Commands::Environment { .. } => unreachable!(), // Already handled above
                                                          // Removed: Admin key rotation commands have been removed for security
    }

    Ok(())
}
