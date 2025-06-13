//! REMOVED: CLI administrative commands for key rotation management
//!
//! This file contained comprehensive administrative tools that have been removed to ensure
//! there is only one secure key rotation path: user-signed rotation requests.
//!
//! The following admin override functionality has been removed:
//! - Bulk key rotation operations (bypassed individual user consent)
//! - Emergency rotation procedures (included bypass_validation: true)
//! - System-wide emergency rotation (rotated all keys without consent)
//! - Policy management that could override normal checks
//! - Administrative commands that skipped signature validation
//!
//! The single remaining rotation path is the normal user-initiated rotation in
//! src/datafold_node/key_rotation_routes.rs which requires users to sign
//! rotation requests with their old private key.

use crate::cli::config::CliConfigManager;
use crate::cli::auth::CliAuthProfile;

/// All admin commands have been removed
/// This function returns an error directing users to the normal rotation path
pub async fn handle_admin_command(
    _command: (),  // No admin commands exist anymore
    _config_manager: &CliConfigManager,
    _profile: Option<&CliAuthProfile>,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("Admin key rotation commands have been removed for security. Please use the normal user rotation endpoint which requires signing with your old private key.".into())
}