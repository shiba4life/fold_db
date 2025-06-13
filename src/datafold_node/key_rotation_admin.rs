//! REMOVED: Administrative management logic for key rotation operations
//!
//! This file contained admin override functionality that has been removed to ensure
//! there is only one secure key rotation path: user-signed rotation requests.
//!
//! The following admin override functionality has been removed:
//! - Bulk processing that bypassed individual user consent
//! - Emergency procedures with bypass_validation capabilities
//! - Policy management that could override normal checks
//! - System monitoring for admin operations
//! - Alternative validation paths for admin users
//!
//! The single remaining rotation path is the normal user-initiated rotation in
//! src/datafold_node/key_rotation_routes.rs which requires users to sign
//! rotation requests with their old private key.

// This file is intentionally left minimal to prevent compilation errors
// while removing all admin override functionality.