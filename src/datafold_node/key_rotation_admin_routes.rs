//! REMOVED: Admin key rotation routes file
//!
//! This file contained admin override functionality that has been removed to ensure
//! there is only one secure key rotation path: user-signed rotation requests.
//!
//! All admin-only endpoints for bulk operations, emergency procedures, and policy
//! management have been removed as they provided alternative paths that could
//! bypass normal signature validation.
//!
//! The single remaining rotation path is in src/datafold_node/key_rotation_routes.rs
//! which requires users to sign rotation requests with their old private key.

// This file is intentionally left minimal to prevent compilation errors
// while removing all admin override functionality.