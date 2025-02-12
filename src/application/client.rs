use super::types::{ApiRequest, ApiResponse, AuthContext, ClientError, OperationType};
use super::ClientConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::os::unix::net::UnixStream;
use std::io::{BufRead, BufReader, Read, Write};
use uuid::Uuid;

/// Client for communicating with DataFold Node via Unix Domain Socket
pub struct DataFoldClient {
    config: ClientConfig,
}

impl DataFoldClient {
    /// Create a new client with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Execute a query operation
    pub fn query<T: DeserializeOwned>(&self, query: impl Serialize) -> Result<T, ClientError> {
        let request = ApiRequest {
            request_id: Uuid::new_v4().to_string(),
            operation_type: OperationType::Query,
            payload: serde_json::to_value(query)?,
            auth: self.get_auth_context()?,
        };

        self.execute_request(request)
    }

    /// Execute a mutation operation
    pub fn mutate<T: DeserializeOwned>(&self, mutation: impl Serialize) -> Result<T, ClientError> {
        let request = ApiRequest {
            request_id: Uuid::new_v4().to_string(),
            operation_type: OperationType::Mutation,
            payload: serde_json::to_value(mutation)?,
            auth: self.get_auth_context()?,
        };

        self.execute_request(request)
    }

    /// Get schema by ID
    pub fn get_schema(&self, schema_id: &str) -> Result<serde_json::Value, ClientError> {
        let request = ApiRequest {
            request_id: Uuid::new_v4().to_string(),
            operation_type: OperationType::GetSchema,
            payload: serde_json::json!({ "schema_id": schema_id }),
            auth: self.get_auth_context()?,
        };

        self.execute_request(request)
    }

    /// Execute a request and parse the response
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

    /// Get authentication context for requests
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
