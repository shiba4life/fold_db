//! Node module for FoldClient
//!
//! This module provides functionality for communicating with the DataFold node.

use crate::Result;
use crate::FoldClientError;
use serde_json::Value;
use std::path::PathBuf;

pub mod client;

pub use client::NodeClient;

/// Connection to a DataFold node
#[derive(Debug, Clone)]
pub enum NodeConnection {
    /// Unix socket connection
    UnixSocket(PathBuf),
    /// TCP connection
    TcpSocket(String, u16),
}

impl NodeConnection {
    /// Create a new Unix socket connection
    pub fn unix_socket<P: Into<PathBuf>>(path: P) -> Self {
        Self::UnixSocket(path.into())
    }

    /// Create a new TCP socket connection
    pub fn tcp_socket<S: Into<String>>(host: S, port: u16) -> Self {
        Self::TcpSocket(host.into(), port)
    }
}

/// Request to the DataFold node
#[derive(Debug, Clone)]
pub struct NodeRequest {
    /// App identifier
    pub app_id: String,
    /// Operation to perform
    pub operation: String,
    /// Operation parameters
    pub params: Value,
    /// Signature of the request
    pub signature: Vec<u8>,
}

/// Response from the DataFold node
#[derive(Debug, Clone)]
pub struct NodeResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Result of the operation
    pub result: Option<Value>,
    /// Error message if the request failed
    pub error: Option<String>,
}
