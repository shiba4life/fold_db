//! # Network Layer
//!
//! The network module implements peer-to-peer communication between DataFold nodes.
//! It provides discovery, connection management, and protocol handling for distributed operations.
//!
//! ## Components
//!
//! * `config` - Network configuration settings
//! * `core` - Core network functionality and peer management
//! * `error` - Network-specific error types
//! * `schema_protocol` - Protocol definition for schema operations
//! * `schema_service` - Service for handling schema-related network requests
//!
//! ## Architecture
//!
//! The network layer uses libp2p for peer-to-peer communication, with custom protocols
//! for schema operations. It supports mDNS discovery for local network peers and
//! direct connections to known peers.
//!
//! Communication between nodes follows a request-response pattern, with each node
//! able to act as both client and server. The network layer handles connection
//! management, peer discovery, and message routing.

pub mod config;
pub mod core;
pub mod connections;
pub mod discovery;
pub mod error;
pub mod schema_protocol;
pub mod schema_service;

pub use config::NetworkConfig;
pub use core::NetworkCore;
pub use error::{NetworkError, NetworkResult};
pub use schema_service::SchemaService;

// Re-export types needed for public API
pub use libp2p::PeerId;
