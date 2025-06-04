//! # Permissions System
//!
//! The permissions module implements access control for DataFold operations.
//! It combines trust-based and explicit permission models to control access to data.
//!
//! ## Components
//!
//! * `permission_manager` - Core permission checking and enforcement
//! * `permission_wrapper` - Wrapper for permission-controlled objects
//! * `types` - Permission-related data structures and policies
//!
//! ## Architecture
//!
//! The permissions system uses a hybrid approach combining:
//!
//! 1. Trust-based access control using trust distances between nodes
//! 2. Explicit permission grants through public keys
//!
//! This dual approach provides flexibility in access control:
//! - Trust distances enable relationship-based access control
//! - Explicit permissions allow fine-grained access management
//! - Both mechanisms can work independently or in combination
//!
//! Each schema field has associated permission policies that define
//! who can read and write to that field, based on trust distance
//! and explicit permissions.

// permissions module

pub mod permission_manager;
pub mod permission_wrapper;
pub mod types;
pub use permission_wrapper::PermissionWrapper;
