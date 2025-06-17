//! Environment management command handlers
//! 
//! This module contains handlers for environment-related operations
//! including listing, switching, comparing, and validating environments.

use crate::cli::args::EnvironmentAction;
use crate::cli::environment_utils::commands as env_commands;

/// Handle environment management commands
pub fn handle_environment_command(action: EnvironmentAction) -> Result<(), Box<dyn std::error::Error>> {
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