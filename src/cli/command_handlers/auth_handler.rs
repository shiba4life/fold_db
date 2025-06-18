//! Authentication command handling for DataFold CLI
//! 
//! This module handles all authentication-related commands including
//! key registration, authentication initialization, profiles, and
//! server integration testing.

use crate::cli::args::Commands;
use crate::cli::commands::auth_commands;

/// Handle authentication-related commands
pub async fn handle_auth_command(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
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
            auth_commands::handle_register_key(
                server_url.clone(),
                key_id.clone(),
                storage_dir.clone(),
                client_id.clone(),
                user_id.clone(),
                key_name.clone(),
                *timeout,
                *retries,
            )
            .await
        }
        Commands::CheckRegistration {
            server_url,
            client_id,
            timeout,
            retries,
        } => {
            auth_commands::handle_check_registration(
                server_url.clone(),
                client_id.clone(),
                *timeout,
                *retries,
            )
            .await
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
            auth_commands::handle_sign_and_verify(
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
            .await
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
            auth_commands::handle_test_server_integration(
                server_url.clone(),
                key_id.clone(),
                storage_dir.clone(),
                test_message.clone(),
                *timeout,
                *retries,
                security_level.clone(),
                *cleanup,
            )
            .await
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
            auth_commands::handle_auth_init(
                server_url.clone(),
                profile.clone(),
                key_id.clone(),
                storage_dir.clone(),
                user_id.clone(),
                environment.clone(),
                *force,
            )
        }
        Commands::AuthStatus {
            verbose,
            profile,
            environment,
        } => {
            auth_commands::handle_auth_status(*verbose, profile.clone(), environment.clone())
        }
        Commands::AuthProfile { action } => {
            auth_commands::handle_auth_profile((*action).clone()).await
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
            auth_commands::handle_auth_keygen(
                key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                *force,
                *auto_register,
                server_url.clone(),
                passphrase.clone(),
            )
            .await
        }
        Commands::AuthTest {
            endpoint,
            profile,
            method,
            payload,
            timeout,
        } => {
            auth_commands::handle_auth_test(
                endpoint.clone(),
                profile.clone(),
                method.clone(),
                payload.clone(),
                *timeout,
            )
            .await
        }
        Commands::AuthConfigure {
            enable_auto_sign: _,
            default_mode: _,
            command: _,
            command_mode: _,
            remove_command_override: _,
            debug: _,
            env_var: _,
            show: _,
        } => {
            eprintln!("AuthConfigure command not yet implemented");
            Err("AuthConfigure command not yet implemented".into())
        }
        Commands::AuthSetup {
            create_config: _,
            server_url: _,
            interactive: _,
        } => {
            eprintln!("AuthSetup command not yet implemented");
            Err("AuthSetup command not yet implemented".into())
        }
        _ => {
            Err("Command is not an authentication command".into())
        }
    }
}

/// Check if a command is an authentication command
pub fn is_auth_command(command: &Commands) -> bool {
    matches!(command,
        Commands::RegisterKey { .. }
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
    )
}