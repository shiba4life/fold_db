use clap::Parser;
// Import CLI argument types from the new args module
use datafold::cli::args::{
    Cli, Commands,
};
use datafold::{load_node_config, DataFoldNode};
// Import command modules
use datafold::cli::commands::{
    auth_commands, crypto, environment, keys, query, schemas, verification,
};
use log::info;

// All CLI argument types have been moved to src/cli/args.rs
// The function implementations continue below

// Core CLI argument types have been moved to src/cli/args.rs
// Complex API structures and data types remain here for now

/// API error structure from server responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ApiError {
    code: String,
    message: String,
    details: std::collections::HashMap<String, serde_json::Value>,
}

/// API response wrapper
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    datafold::logging::init().ok();
    let cli = Cli::parse();

    // Handle commands that don't need the node first
    match &cli.command {
        Commands::HashSchemas { verify } => {
            return schemas::handle_hash_schemas(*verify);
        }
        Commands::CryptoValidate { config_file } => {
            return crypto::handle_crypto_validate(config_file.clone(), &cli.config);
        }
        Commands::GenerateKey {
            format,
            private_key_file,
            public_key_file,
            count,
            public_only,
            private_only,
        } => {
            return keys::handle_generate_key(
                format.clone(),
                private_key_file.clone(),
                public_key_file.clone(),
                *count,
                *public_only,
                *private_only,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
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
            return keys::handle_derive_key(
                format.clone(),
                private_key_file.clone(),
                public_key_file.clone(),
                security_level.clone(),
                *public_only,
                *private_only,
                passphrase.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::ExtractPublicKey {
            private_key,
            private_key_file,
            format,
            output_file,
        } => {
            return keys::handle_extract_public_key(
                private_key.clone(),
                private_key_file.clone(),
                format.clone(),
                output_file.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::VerifyKey {
            private_key,
            private_key_file,
            public_key,
            public_key_file,
        } => {
            return keys::handle_verify_key(
                private_key.clone(),
                private_key_file.clone(),
                public_key.clone(),
                public_key_file.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
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
            return keys::handle_store_key(
                key_id.clone(),
                private_key.clone(),
                private_key_file.clone(),
                storage_dir.clone(),
                *force,
                security_level.clone(),
                passphrase.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::RetrieveKey {
            key_id,
            storage_dir,
            format,
            output_file,
            public_only,
        } => {
            return keys::handle_retrieve_key(
                key_id.clone(),
                storage_dir.clone(),
                format.clone(),
                output_file.clone(),
                *public_only,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::DeleteKey {
            key_id,
            storage_dir,
            force,
        } => {
            return keys::handle_delete_key(key_id.clone(), storage_dir.clone(), *force).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::ListKeys {
            storage_dir,
            verbose,
        } => {
            return keys::handle_list_keys(storage_dir.clone(), *verbose).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
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
            return keys::handle_derive_from_master(
                master_key_id.clone(),
                context.clone(),
                child_key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                format.clone(),
                *output_only,
                *force,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::RotateKey {
            key_id,
            storage_dir,
            security_level,
            method,
            keep_backup,
            force,
        } => {
            return keys::handle_rotate_key(
                key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                method.clone(),
                *keep_backup,
                *force,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::ListKeyVersions {
            key_id,
            storage_dir,
            verbose,
        } => {
            return keys::handle_list_key_versions(key_id.clone(), storage_dir.clone(), *verbose).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::BackupKey {
            key_id,
            storage_dir,
            backup_file,
            backup_passphrase,
        } => {
            return keys::handle_backup_key(
                key_id.clone(),
                storage_dir.clone(),
                backup_file.clone(),
                *backup_passphrase,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::RestoreKey {
            backup_file,
            key_id,
            storage_dir,
            force,
        } => {
            return keys::handle_restore_key(
                backup_file.clone(),
                key_id.clone(),
                storage_dir.clone(),
                *force,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::ExportKey {
            key_id,
            storage_dir,
            export_file,
            format,
            export_passphrase,
            include_metadata,
        } => {
            return keys::handle_export_key(
                key_id.clone(),
                storage_dir.clone(),
                export_file.clone(),
                format.clone(),
                *export_passphrase,
                *include_metadata,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Commands::ImportKey {
            export_file,
            key_id,
            storage_dir,
            force,
            verify_integrity,
        } => {
            return keys::handle_import_key(
                export_file.clone(),
                key_id.clone(),
                storage_dir.clone(),
                *force,
                *verify_integrity,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
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
            return auth_commands::handle_register_key(
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
            return auth_commands::handle_check_registration(
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
            return auth_commands::handle_sign_and_verify(
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
            return auth_commands::handle_test_server_integration(
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
            return auth_commands::handle_auth_init(
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
            return auth_commands::handle_auth_status(*verbose, profile.clone(), environment.clone());
        }
        Commands::AuthProfile { action } => {
            return auth_commands::handle_auth_profile((*action).clone()).await;
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
            return auth_commands::handle_auth_keygen(
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
            return auth_commands::handle_auth_test(
                endpoint.clone(),
                profile.clone(),
                method.clone(),
                payload.clone(),
                *timeout,
            )
            .await;
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
            return Err("AuthConfigure command not yet implemented".into());
        }
        Commands::AuthSetup {
            create_config: _,
            server_url: _,
            interactive: _,
        } => {
            eprintln!("AuthSetup command not yet implemented");
            return Err("AuthSetup command not yet implemented".into());
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
            return verification::handle_verify_signature(
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
            return verification::handle_inspect_signature(
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
            return verification::handle_verify_response(
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
            return environment::handle_environment_command(action.clone());
        }
        Commands::VerificationConfig { action } => {
            return verification::handle_verification_config(action.clone()).await;
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
        Commands::LoadSchema { path } => schemas::handle_load_schema(path, &mut node)?,
        Commands::AddSchema { path, name } => schemas::handle_add_schema(path, name, &mut node)?,
        Commands::HashSchemas { .. } => unreachable!(), // Already handled above
        Commands::ListSchemas {} => schemas::handle_list_schemas(&mut node)?,
        Commands::ListAvailableSchemas {} => schemas::handle_list_available_schemas(&mut node)?,
        Commands::AllowSchema { name } => schemas::handle_allow_schema(name, &mut node)?,
        Commands::Query {
            schema,
            fields,
            filter,
            output,
        } => query::handle_query(&mut node, schema, fields, filter, output)?,
        Commands::Mutate {
            schema,
            mutation_type,
            data,
        } => query::handle_mutate(&mut node, schema, mutation_type, data)?,
        Commands::UnloadSchema { name } => schemas::handle_unload_schema(name, &mut node)?,
        Commands::ApproveSchema { name } => schemas::handle_approve_schema(name, &mut node)?,
        Commands::BlockSchema { name } => schemas::handle_block_schema(name, &mut node)?,
        Commands::GetSchemaState { name } => schemas::handle_get_schema_state(name, &mut node)?,
        Commands::ListSchemasByState { state } => schemas::handle_list_schemas_by_state(state, &mut node)?,
        Commands::CryptoInit {
            method,
            security_level,
            force,
        } => crypto::handle_crypto_init(method, security_level, force, &mut node)?,
        Commands::CryptoStatus {} => crypto::handle_crypto_status(&mut node)?,
        Commands::CryptoValidate { .. } => unreachable!(), // Already handled above
        Commands::GenerateKey { .. } => unreachable!(),    // Already handled above
        Commands::DeriveKey { .. } => unreachable!(),      // Already handled above
        Commands::ExtractPublicKey { .. } => unreachable!(), // Already handled above
        Commands::Execute { path } => query::handle_execute(path, &mut node)?,
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
