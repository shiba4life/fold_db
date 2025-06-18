//! Verification command definitions
//! 
//! This module contains all CLI commands related to signature verification,
//! signature inspection, and verification configuration.

use clap::Subcommand;
use std::path::PathBuf;
use super::cli_types::{HttpMethod, VerificationOutputFormat};

/// Verification-related CLI commands
#[derive(Subcommand, Debug)]
pub enum VerificationCommands {
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