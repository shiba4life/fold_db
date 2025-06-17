//! CLI argument definitions and parsing structures
//! 
//! This module contains all the argument-related types, enums, and structs
//! used by the DataFold CLI. These were extracted from the main CLI binary
//! to improve code organization and maintainability.

use clap::{Parser, Subcommand, ValueEnum};
use crate::security_types::SecurityLevel;
use crate::cli::signing_config::SigningMode;
use crate::crypto::Argon2Params;
use crate::MutationType;
use std::path::PathBuf;

/// Main CLI structure containing global arguments and subcommands
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the node configuration file
    #[arg(short, long, default_value = "config/node_config.json")]
    pub config: String,

    /// Authentication profile to use for signing (mandatory authentication enabled)
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Environment to use for unified configuration (dev/staging/prod)
    #[arg(long, global = true)]
    pub environment: Option<String>,

    /// Enable debug logging for signature operations
    #[arg(long, global = true)]
    pub sign_debug: bool,

    /// Verbose output for debugging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Crypto initialization method
#[derive(Debug, Clone, ValueEnum)]
pub enum CryptoMethod {
    /// Generate a random master key pair (highest security, no password recovery)
    Random,
    /// Derive master key from user passphrase (allows password recovery)
    Passphrase,
}

/// Security level enum for CLI (wrapper around the config SecurityLevel)
#[derive(Debug, Clone, ValueEnum)]
pub enum CliSecurityLevel {
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
pub enum KeyFormat {
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
pub enum RotationMethod {
    /// Generate a completely new random key
    Regenerate,
    /// Derive new key from master using incremented counter
    Derive,
    /// Re-derive from passphrase with new salt
    Rederive,
}

/// Export format for encrypted key export
#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    /// JSON format with base64-encoded data
    Json,
    /// Binary format with compact encoding
    Binary,
}

/// HTTP methods for testing
#[derive(Debug, Clone, ValueEnum)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// CLI wrapper for signing mode
#[derive(Debug, Clone, ValueEnum)]
pub enum CliSigningMode {
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
pub enum MessageEncoding {
    /// UTF-8 string encoding
    Utf8,
    /// Hexadecimal encoding
    Hex,
    /// Base64 encoding
    Base64,
}

/// Output format for verification results
#[derive(Debug, Clone, ValueEnum)]
pub enum VerificationOutputFormat {
    /// JSON format
    Json,
    /// Table format (human-readable)
    Table,
    /// Compact format (one line)
    Compact,
}

/// Main Commands enum containing all CLI subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
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

/// Verification configuration actions
#[derive(Subcommand, Debug, Clone)]
pub enum VerificationConfigAction {
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

/// Secure key storage configuration
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyStorageConfig {
    /// Encrypted key data
    pub encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    pub nonce: [u8; 12],
    /// Salt used for key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters used for key derivation
    pub argon2_params: StoredArgon2Params,
    /// Timestamp when key was stored
    pub created_at: String,
    /// Version of storage format
    pub version: u32,
}

/// Simplified Argon2 parameters for storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredArgon2Params {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
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
pub struct KeyVersionMetadata {
    /// Version number (starts at 1)
    pub version: u32,
    /// Previous version number (None for initial version)
    pub previous_version: Option<u32>,
    /// Creation timestamp for this version
    pub created_at: String,
    /// Derivation method used for this version
    pub derivation_method: String,
    /// Salt used for key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters used for this version
    pub argon2_params: StoredArgon2Params,
}

/// Enhanced key storage with versioning support
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VersionedKeyStorageConfig {
    /// Current active version metadata
    pub current_version: KeyVersionMetadata,
    /// Encrypted key data for current version
    pub encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    pub nonce: [u8; 12],
    /// Version of storage format
    pub version: u32,
}

/// Key backup format for secure export/import
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyBackupFormat {
    /// Encrypted key storage configuration
    pub key_config: KeyStorageConfig,
    /// Additional backup metadata
    pub backup_metadata: BackupMetadata,
    /// Backup format version
    pub backup_version: u32,
    /// Optional additional encryption layer
    pub additional_encryption: Option<AdditionalEncryption>,
}

/// Backup metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BackupMetadata {
    /// Original key identifier
    pub key_id: String,
    /// Backup creation timestamp
    pub created_at: String,
    /// Source system identifier
    pub source_system: String,
    /// Backup description
    pub description: Option<String>,
}

/// Additional encryption layer for enhanced security
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AdditionalEncryption {
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    /// Nonce for additional encryption (12 bytes)
    pub nonce: [u8; 12],
    /// Salt for additional key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters for additional encryption
    pub argon2_params: StoredArgon2Params,
}

/// Enhanced key export format with metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnhancedKeyExportFormat {
    /// Key data (encrypted)
    pub key_data: Vec<u8>,
    /// Export metadata
    pub metadata: ExportKeyMetadata,
    /// Enhanced KDF parameters
    pub kdf_params: EnhancedKdfParams,
    /// Export format version
    pub version: u32,
}

/// Enhanced KDF parameters for export
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnhancedKdfParams {
    /// Primary salt (32 bytes)
    pub primary_salt: [u8; 32],
    /// Secondary salt for export passphrase (32 bytes)
    pub export_salt: [u8; 32],
    /// Argon2 parameters for primary encryption
    pub primary_argon2: StoredArgon2Params,
    /// Argon2 parameters for export encryption
    pub export_argon2: StoredArgon2Params,
}

/// Export key metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportKeyMetadata {
    /// Original key identifier
    pub key_id: String,
    /// Export timestamp
    pub exported_at: String,
    /// Key version (if versioned)
    pub key_version: Option<u32>,
    /// Export description
    pub description: Option<String>,
}

/// API response wrapper
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

/// API error information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Public key registration request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyRegistrationRequest {
    pub client_id: String,
    pub public_key: String,
    pub user_id: Option<String>,
    pub key_name: Option<String>,
}

/// Public key registration response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyRegistrationResponse {
    pub client_id: String,
    pub public_key: String,
    pub registered_at: String,
    pub status: String,
    pub key_id: Option<String>,
}

/// Public key status response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyStatusResponse {
    pub client_id: String,
    pub registered: bool,
    pub public_key: Option<String>,
    pub registered_at: Option<String>,
    pub status: String,
}

/// Signature verification request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignatureVerificationRequest {
    pub client_id: String,
    pub message: String,
    pub signature: String,
    pub message_encoding: String,
}

/// Signature verification response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignatureVerificationResponse {
    pub valid: bool,
    pub message: String,
    pub verified_at: String,
}