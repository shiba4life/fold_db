//! Client implementation for communicating with a DataFold node.
//! 
//! This module provides the client-side interface for interacting with a DataFold node
//! through Unix Domain Sockets. It handles:
//! - Connection management
//! - Request/response serialization
//! - Authentication
//! - Timeout handling
//! 
//! # Example
//! ```no_run
//! use fold_db::{DataFoldClient, ClientConfig};
//! use std::path::PathBuf;
//! use std::time::Duration;
//! 
//! let config = ClientConfig {
//!     socket_path: PathBuf::from("/tmp/folddb.sock"),
//!     timeout: Duration::from_secs(5),
//! };
//! 
//! let client = DataFoldClient::new(config);
//! 
//! // Execute a query
//! let result: serde_json::Value = client.query(serde_json::json!({
//!     "schema": "users",
//!     "filter": { "age": { "gt": 21 } }
//! })).expect("Query failed");
//! ```

use super::types::{ApiRequest, ApiResponse, AuthContext, ClientError, OperationType};
use super::ClientConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::os::unix::net::UnixStream;
use std::io::{BufRead, Write};
use uuid::Uuid;

/// A client for communicating with a DataFold node via Unix Domain Socket.
/// 
/// The client provides a high-level interface for:
/// - Executing queries
/// - Performing mutations
/// - Retrieving schema information
/// - Managing authentication
pub struct DataFoldClient {
    config: ClientConfig,
}

impl DataFoldClient {
    /// Creates a new client with the given configuration.
    /// 
    /// # Arguments
    /// * `config` - Client configuration including socket path and timeout settings
    /// 
    /// # Example
    /// ```no_run
    /// use fold_db::{DataFoldClient, ClientConfig};
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    /// 
    /// let config = ClientConfig {
    ///     socket_path: PathBuf::from("/tmp/folddb.sock"),
    ///     timeout: Duration::from_secs(5),
    /// };
    /// 
    /// let client = DataFoldClient::new(config);
    /// ```
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Executes a query operation against the database.
    /// 
    /// # Type Parameters
    /// * `T` - The expected return type that can be deserialized from JSON
    /// 
    /// # Arguments
    /// * `query` - The query to execute, must be serializable to JSON
    /// 
    /// # Returns
    /// * `Result<T, ClientError>` - The query results or an error
    /// 
    /// # Example
    /// ```no_run
    /// # use fold_db::{DataFoldClient, ClientConfig};
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # let client = DataFoldClient::new(ClientConfig {
    /// #     socket_path: PathBuf::from("/tmp/folddb.sock"),
    /// #     timeout: Duration::from_secs(5),
    /// # });
    /// let query = serde_json::json!({
    ///     "schema": "users",
    ///     "filter": { "age": { "gt": 21 } }
    /// });
    /// 
    /// let results: Vec<serde_json::Value> = client.query(query)
    ///     .expect("Query failed");
    /// ```
    pub fn query<T: DeserializeOwned>(&self, query: impl Serialize) -> Result<T, ClientError> {
        let request = ApiRequest {
            request_id: Uuid::new_v4().to_string(),
            operation_type: OperationType::Query,
            payload: serde_json::to_value(query)?,
            auth: self.get_auth_context()?,
        };

        self.execute_request(request)
    }

    /// Executes a mutation operation on the database.
    /// 
    /// # Type Parameters
    /// * `T` - The expected return type that can be deserialized from JSON
    /// 
    /// # Arguments
    /// * `mutation` - The mutation to execute, must be serializable to JSON
    /// 
    /// # Returns
    /// * `Result<T, ClientError>` - The mutation result or an error
    /// 
    /// # Example
    /// ```no_run
    /// # use fold_db::{DataFoldClient, ClientConfig};
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # let client = DataFoldClient::new(ClientConfig {
    /// #     socket_path: PathBuf::from("/tmp/folddb.sock"),
    /// #     timeout: Duration::from_secs(5),
    /// # });
    /// let mutation = serde_json::json!({
    ///     "schema": "users",
    ///     "operation": "create",
    ///     "data": {
    ///         "name": "Alice",
    ///         "age": 30
    ///     }
    /// });
    /// 
    /// let result: serde_json::Value = client.mutate(mutation)
    ///     .expect("Mutation failed");
    /// ```
    pub fn mutate<T: DeserializeOwned>(&self, mutation: impl Serialize) -> Result<T, ClientError> {
        let request = ApiRequest {
            request_id: Uuid::new_v4().to_string(),
            operation_type: OperationType::Mutation,
            payload: serde_json::to_value(mutation)?,
            auth: self.get_auth_context()?,
        };

        self.execute_request(request)
    }

    /// Retrieves a schema definition by its ID.
    /// 
    /// # Arguments
    /// * `schema_id` - The unique identifier of the schema to retrieve
    /// 
    /// # Returns
    /// * `Result<serde_json::Value, ClientError>` - The schema definition or an error
    /// 
    /// # Example
    /// ```no_run
    /// # use fold_db::{DataFoldClient, ClientConfig};
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # let client = DataFoldClient::new(ClientConfig {
    /// #     socket_path: PathBuf::from("/tmp/folddb.sock"),
    /// #     timeout: Duration::from_secs(5),
    /// # });
    /// let schema = client.get_schema("users")
    ///     .expect("Failed to retrieve schema");
    /// ```
    pub fn get_schema(&self, schema_id: &str) -> Result<serde_json::Value, ClientError> {
        let request = ApiRequest {
            request_id: Uuid::new_v4().to_string(),
            operation_type: OperationType::GetSchema,
            payload: serde_json::json!({ "schema_id": schema_id }),
            auth: self.get_auth_context()?,
        };

        self.execute_request(request)
    }

    /// Executes a request and parses the response.
    /// 
    /// This is an internal method used by the public interface methods.
    /// It handles the low-level details of:
    /// - Establishing socket connection
    /// - Setting timeouts
    /// - Sending request data
    /// - Reading response
    /// - Parsing and error handling
    /// 
    /// # Type Parameters
    /// * `T` - The expected return type that can be deserialized from JSON
    /// 
    /// # Arguments
    /// * `request` - The API request to execute
    /// 
    /// # Returns
    /// * `Result<T, ClientError>` - The parsed response or an error
    fn execute_request<T: DeserializeOwned>(&self, request: ApiRequest) -> Result<T, ClientError> {
        let stream = UnixStream::connect(&self.config.socket_path)
            .map_err(ClientError::ConnectionFailed)?;
        
        // Set read/write timeouts
        stream.set_read_timeout(Some(self.config.timeout))?;
        stream.set_write_timeout(Some(self.config.timeout))?;

        // Split stream into reader and writer
        let mut writer = stream.try_clone()?;
        let mut reader = std::io::BufReader::new(stream);

        // Send request in a new scope so writer is dropped after sending
        {
            let request_json = serde_json::to_string(&request)?;
            writer.write_all(request_json.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }

        // Read response
        let mut response_data = String::new();
        reader.read_line(&mut response_data)?;
        response_data = response_data.trim().to_string();

        let response: ApiResponse = serde_json::from_str(&response_data)?;

        match response.status {
            super::types::ResponseStatus::Success => {
                let data = response.data.ok_or_else(|| {
                    ClientError::InvalidResponse("Missing data in successful response".into())
                })?;
                Ok(serde_json::from_value(data)?)
            }
            super::types::ResponseStatus::Error => {
                let error = response.error.ok_or_else(|| {
                    ClientError::InvalidResponse("Missing error details in error response".into())
                })?;
                Err(ClientError::OperationFailed(error.message))
            }
        }
    }

    /// Retrieves the authentication context for requests.
    /// 
    /// Currently returns a placeholder key. This will be replaced with
    /// proper key management in the future.
    /// 
    /// # Returns
    /// * `Result<AuthContext, ClientError>` - The auth context or an error
    fn get_auth_context(&self) -> Result<AuthContext, ClientError> {
        // TODO: Implement proper key management
        // For now, return a placeholder public key
        Ok(AuthContext {
            public_key: "placeholder_key".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_client_creation() {
        let config = ClientConfig {
            socket_path: PathBuf::from("/tmp/test.sock"),
            timeout: Duration::from_secs(1),
        };
        let client = DataFoldClient::new(config);
        assert_eq!(client.config.timeout, Duration::from_secs(1));
    }
}
