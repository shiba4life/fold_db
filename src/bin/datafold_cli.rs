//! Main CLI entry point and command routing for DataFold CLI
//! 
//! This module contains the main function and high-level command routing,
//! delegating specific command handling to dedicated handler modules.

use clap::Parser;
use datafold::cli::args::{Cli, Commands};
use datafold::{load_node_config, DataFoldNode};
use datafold::cli::command_handlers::{
    auth_handler, crypto_handler, schema_handler
};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    datafold::logging::init().ok();
    let cli = Cli::parse();

    // Handle commands that don't need the node first
    match &cli.command {
        // Standalone schema commands
        cmd if schema_handler::is_standalone_schema_command(cmd) => {
            return schema_handler::handle_schema_command_standalone(cmd);
        }
        
        // Standalone crypto commands
        cmd if crypto_handler::is_standalone_crypto_command(cmd) => {
            return crypto_handler::handle_crypto_command_standalone(cmd, &cli.config).await;
        }
        
        // Authentication commands
        cmd if auth_handler::is_auth_command(cmd) => {
            return auth_handler::handle_auth_command(cmd).await;
        }
        
        // Verification commands
        cmd if schema_handler::is_verification_command(cmd) => {
            return schema_handler::handle_verification_command(cmd).await;
        }
        
        // Environment commands
        cmd if schema_handler::is_environment_command(cmd) => {
            return schema_handler::handle_environment_command(cmd);
        }
        
        // Commands that require a node will be handled below
        _ => {}
    }

    // Load node configuration for commands that require it
    info!("Loading config from: {}", cli.config);
    let config = load_node_config(Some(&cli.config), None)?;

    // Initialize node
    info!("Initializing DataFold Node...");
    let mut node = DataFoldNode::load(config).await?;
    info!("Node initialized with ID: {}", node.get_node_id());

    // Process commands that require a node
    match &cli.command {
        // Node-based schema commands
        cmd if schema_handler::is_node_schema_command(cmd) => {
            schema_handler::handle_schema_command_with_node(cmd, &mut node)?;
        }
        
        // Node-based crypto commands
        cmd if crypto_handler::is_node_crypto_command(cmd) => {
            crypto_handler::handle_crypto_command_with_node(cmd, &mut node)?;
        }
        
        // Handle unreachable patterns for already-processed commands
        Commands::HashSchemas { .. } 
        | Commands::CryptoValidate { .. }
        | Commands::GenerateKey { .. }
        | Commands::DeriveKey { .. }
        | Commands::ExtractPublicKey { .. }
        | Commands::VerifyKey { .. }
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
        | Commands::Environment { .. } => {
            // These commands were already handled above
            unreachable!("Command should have been handled in the first match block")
        }
        
        // If we get here, there might be a command that wasn't properly categorized
        _ => {
            return Err(format!("Unhandled command: {:?}", cli.command).into());
        }
    }

    Ok(())
}
