//! Schema and query command handling for DataFold CLI
//! 
//! This module handles all schema-related operations including loading,
//! managing, querying, and executing operations on schemas.

use crate::cli::args::Commands;
use crate::cli::commands::{schemas, query, verification, environment};
use crate::DataFoldNode;

/// Handle schema commands that don't require a node
pub fn handle_schema_command_standalone(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::HashSchemas { verify } => {
            schemas::handle_hash_schemas(*verify)
        }
        _ => {
            Err("Command is not a standalone schema command".into())
        }
    }
}

/// Handle schema and query commands that require a node
pub fn handle_schema_command_with_node(command: &Commands, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::LoadSchema { path } => {
            schemas::handle_load_schema(path.clone(), node)?;
            Ok(())
        }
        Commands::AddSchema { path, name } => {
            schemas::handle_add_schema(path.clone(), name.clone(), node)?;
            Ok(())
        }
        Commands::ListSchemas {} => {
            schemas::handle_list_schemas(node)?;
            Ok(())
        }
        Commands::ListAvailableSchemas {} => {
            schemas::handle_list_available_schemas(node)?;
            Ok(())
        }
        Commands::AllowSchema { name } => {
            schemas::handle_allow_schema(name.clone(), node)?;
            Ok(())
        }
        Commands::Query {
            schema,
            fields,
            filter,
            output,
        } => {
            query::handle_query(node, schema.clone(), fields.clone(), filter.clone(), output.clone())?;
            Ok(())
        }
        Commands::Mutate {
            schema,
            mutation_type,
            data,
        } => {
            query::handle_mutate(node, schema.clone(), mutation_type.clone(), data.clone())?;
            Ok(())
        }
        Commands::Execute { path } => {
            query::handle_execute(path.clone(), node)?;
            Ok(())
        }
        Commands::UnloadSchema { name } => {
            schemas::handle_unload_schema(name.clone(), node)?;
            Ok(())
        }
        Commands::ApproveSchema { name } => {
            schemas::handle_approve_schema(name.clone(), node)?;
            Ok(())
        }
        Commands::BlockSchema { name } => {
            schemas::handle_block_schema(name.clone(), node)?;
            Ok(())
        }
        Commands::GetSchemaState { name } => {
            schemas::handle_get_schema_state(name.clone(), node)?;
            Ok(())
        }
        Commands::ListSchemasByState { state } => {
            schemas::handle_list_schemas_by_state(state.clone(), node)?;
            Ok(())
        }
        _ => {
            Err("Command is not a node-based schema command".into())
        }
    }
}

/// Handle verification commands
pub async fn handle_verification_command(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
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
            verification::handle_verify_signature(
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
            .await
        }
        Commands::InspectSignature {
            signature_input,
            signature,
            headers_file,
            output_format,
            detailed,
            debug,
        } => {
            verification::handle_inspect_signature(
                signature_input.clone(),
                signature.clone(),
                headers_file.clone(),
                output_format.clone(),
                *detailed,
                *debug,
            )
            .await
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
            verification::handle_verify_response(
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
            .await
        }
        Commands::VerificationConfig { action } => {
            verification::handle_verification_config(action.clone()).await
        }
        _ => {
            Err("Command is not a verification command".into())
        }
    }
}

/// Handle environment commands
pub fn handle_environment_command(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Environment { action } => {
            environment::handle_environment_command(action.clone())
        }
        _ => {
            Err("Command is not an environment command".into())
        }
    }
}

/// Check if a command is a standalone schema command (doesn't need node)
pub fn is_standalone_schema_command(command: &Commands) -> bool {
    matches!(command,
        Commands::HashSchemas { .. }
    )
}

/// Check if a command is a node-based schema command
pub fn is_node_schema_command(command: &Commands) -> bool {
    matches!(command,
        Commands::LoadSchema { .. }
        | Commands::AddSchema { .. }
        | Commands::ListSchemas { .. }
        | Commands::ListAvailableSchemas { .. }
        | Commands::AllowSchema { .. }
        | Commands::Query { .. }
        | Commands::Mutate { .. }
        | Commands::Execute { .. }
        | Commands::UnloadSchema { .. }
        | Commands::ApproveSchema { .. }
        | Commands::BlockSchema { .. }
        | Commands::GetSchemaState { .. }
        | Commands::ListSchemasByState { .. }
    )
}

/// Check if a command is a verification command
pub fn is_verification_command(command: &Commands) -> bool {
    matches!(command,
        Commands::VerifySignature { .. }
        | Commands::InspectSignature { .. }
        | Commands::VerifyResponse { .. }
        | Commands::VerificationConfig { .. }
    )
}

/// Check if a command is an environment command
pub fn is_environment_command(command: &Commands) -> bool {
    matches!(command,
        Commands::Environment { .. }
    )
}