//! HTTP client for CLI with automatic request signing
//!
//! This module provides an HTTP client that automatically signs requests using
//! RFC 9421 HTTP Message Signatures when authentication is configured.

use crate::cli::auth::{CliAuthError, CliAuthProfile, CliAuthStatus, CliRequestSigner, CliSigningConfig};
use crate::crypto::ed25519::MasterKeyPair;
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

/// HTTP client with automatic request signing for DataFold CLI
pub struct AuthenticatedHttpClient {
    /// Underlying HTTP client
    client: Client,
    /// Request signer (if authentication is configured)
    signer: Option<CliRequestSigner>,
    /// Retry configuration
    retry_config: RetryConfig,
}

/// Configuration for HTTP request retries
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries (will be exponentially backed off)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries
    pub max_delay_ms: u64,
    /// Whether to retry on server errors (5xx)
    pub retry_server_errors: bool,
    /// Whether to retry on network errors
    pub retry_network_errors: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            retry_server_errors: true,
            retry_network_errors: true,
        }
    }
}

/// Error wrapper for HTTP client operations
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    #[error("Authentication error: {0}")]
    Auth(#[from] CliAuthError),
    
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Server error: HTTP {status}: {message}")]
    Server { status: u16, message: String },
    
    #[error("All retry attempts failed: {last_error}")]
    RetryExhausted { last_error: String },
}

pub type HttpClientResult<T> = Result<T, HttpClientError>;

impl AuthenticatedHttpClient {
    /// Create a new HTTP client without authentication
    pub fn new(timeout_secs: u64) -> HttpClientResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("datafold-cli/1.0.0")
            .build()?;
        
        Ok(Self {
            client,
            signer: None,
            retry_config: RetryConfig::default(),
        })
    }
    
    /// Create a new HTTP client with authentication
    pub fn with_authentication(
        timeout_secs: u64,
        keypair: MasterKeyPair,
        profile: CliAuthProfile,
        signing_config: Option<CliSigningConfig>,
    ) -> HttpClientResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("datafold-cli/1.0.0")
            .build()?;
        
        let config = signing_config.unwrap_or_default();
        let signer = CliRequestSigner::new(keypair, profile, config);
        
        Ok(Self {
            client,
            signer: Some(signer),
            retry_config: RetryConfig::default(),
        })
    }
    
    /// Set retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
    
    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.signer.is_some()
    }
    
    /// Get authentication status
    pub fn auth_status(&self) -> CliAuthStatus {
        match &self.signer {
            Some(signer) => CliAuthStatus {
                configured: true,
                client_id: Some(signer.profile().client_id.clone()),
                key_id: Some(signer.profile().key_id.clone()),
                server_url: Some(signer.profile().server_url.clone()),
                last_attempt: None,
                last_success: None,
            },
            None => CliAuthStatus::default(),
        }
    }
    
    /// Make a GET request
    pub async fn get(&self, url: &str) -> HttpClientResult<Response> {
        let request = self.client.get(url);
        self.execute_request_with_retry(request).await
    }
    
    /// Make a POST request with JSON body
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> HttpClientResult<Response> {
        let request = self.client
            .post(url)
            .header("content-type", "application/json")
            .json(body);
        self.execute_request_with_retry(request).await
    }
    
    /// Make a PUT request with JSON body
    pub async fn put_json<T: Serialize>(&self, url: &str, body: &T) -> HttpClientResult<Response> {
        let request = self.client
            .put(url)
            .header("content-type", "application/json")
            .json(body);
        self.execute_request_with_retry(request).await
    }
    
    /// Make a PATCH request with JSON body
    pub async fn patch_json<T: Serialize>(&self, url: &str, body: &T) -> HttpClientResult<Response> {
        let request = self.client
            .patch(url)
            .header("content-type", "application/json")
            .json(body);
        self.execute_request_with_retry(request).await
    }
    
    /// Make a DELETE request
    pub async fn delete(&self, url: &str) -> HttpClientResult<Response> {
        let request = self.client.delete(url);
        self.execute_request_with_retry(request).await
    }
    
    /// Make a request and deserialize the JSON response
    pub async fn request_json<T: DeserializeOwned>(&self, request_builder: RequestBuilder) -> HttpClientResult<T> {
        let response = self.execute_request_with_retry(request_builder).await?;
        let status = response.status();
        let text = response.text().await?;
        
        if status.is_success() {
            serde_json::from_str(&text).map_err(HttpClientError::JsonSerialization)
        } else {
            Err(HttpClientError::Server {
                status: status.as_u16(),
                message: text,
            })
        }
    }
    
    /// Execute a request with automatic signing and retry logic
    async fn execute_request_with_retry(&self, request_builder: RequestBuilder) -> HttpClientResult<Response> {
        let mut last_error = None;
        
        for attempt in 0..=self.retry_config.max_retries {
            if attempt > 0 {
                let delay_ms = std::cmp::min(
                    self.retry_config.initial_delay_ms * 2_u64.pow(attempt - 1),
                    self.retry_config.max_delay_ms,
                );
                println!("Retrying request (attempt {}/{}) after {}ms", 
                         attempt + 1, self.retry_config.max_retries + 1, delay_ms);
                sleep(Duration::from_millis(delay_ms)).await;
            }
            
            // Clone the request builder for each attempt
            let request = match request_builder.try_clone() {
                Some(req) => req,
                None => return Err(HttpClientError::Configuration(
                    "Failed to clone request for retry".to_string()
                )),
            };
            
            // Build the request
            let mut built_request = match request.build() {
                Ok(req) => req,
                Err(e) => {
                    last_error = Some(HttpClientError::Request(e));
                    continue;
                }
            };
            
            // Sign the request if authentication is configured
            if let Some(signer) = &self.signer {
                if let Err(e) = signer.sign_request(&mut built_request) {
                    return Err(HttpClientError::Auth(e));
                }
            }
            
            // Execute the request
            match self.client.execute(built_request).await {
                Ok(response) => {
                    let status = response.status();
                    
                    // Check if we should retry based on status code
                    if status.is_server_error() && self.retry_config.retry_server_errors && attempt < self.retry_config.max_retries {
                        last_error = Some(HttpClientError::Server {
                            status: status.as_u16(),
                            message: format!("Server error on attempt {}", attempt + 1),
                        });
                        continue;
                    }
                    
                    // Return the response (success or client error)
                    return Ok(response);
                }
                Err(e) => {
                    let is_network_error = e.is_timeout() || e.is_connect() || e.is_request();
                    
                    if is_network_error && self.retry_config.retry_network_errors && attempt < self.retry_config.max_retries {
                        last_error = Some(HttpClientError::Request(e));
                        continue;
                    } else {
                        return Err(HttpClientError::Request(e));
                    }
                }
            }
        }
        
        // All retries exhausted
        Err(HttpClientError::RetryExhausted {
            last_error: last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string()),
        })
    }
    
    /// Create a request builder for custom requests
    pub fn request(&self, method: Method, url: &str) -> HttpClientResult<RequestBuilder> {
        let url = url.parse::<Url>()
            .map_err(|e| HttpClientError::Configuration(format!("Invalid URL: {}", e)))?;
        Ok(self.client.request(method, url))
    }
}

/// Builder for creating authenticated HTTP clients from CLI configuration
pub struct HttpClientBuilder {
    timeout_secs: u64,
    retry_config: RetryConfig,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            retry_config: RetryConfig::default(),
        }
    }
}

impl HttpClientBuilder {
    /// Create a new HTTP client builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set request timeout
    pub fn timeout_secs(mut self, timeout: u64) -> Self {
        self.timeout_secs = timeout;
        self
    }
    
    /// Set retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
    
    /// Build an unauthenticated HTTP client
    pub fn build(self) -> HttpClientResult<AuthenticatedHttpClient> {
        AuthenticatedHttpClient::new(self.timeout_secs)
            .map(|client| client.with_retry_config(self.retry_config))
    }
    
    /// Build an authenticated HTTP client
    pub fn build_authenticated(
        self,
        keypair: MasterKeyPair,
        profile: CliAuthProfile,
        signing_config: Option<CliSigningConfig>,
    ) -> HttpClientResult<AuthenticatedHttpClient> {
        AuthenticatedHttpClient::with_authentication(
            self.timeout_secs,
            keypair,
            profile,
            signing_config,
        ).map(|client| client.with_retry_config(self.retry_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::generate_master_keypair;
    use std::collections::HashMap;
    
    fn create_test_profile() -> CliAuthProfile {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        CliAuthProfile {
            client_id: "test-client-123".to_string(),
            key_id: "test-key".to_string(),
            user_id: Some("test-user".to_string()),
            server_url: "https://api.example.com".to_string(),
            metadata,
        }
    }
    
    #[test]
    fn test_unauthenticated_client_creation() {
        let client = AuthenticatedHttpClient::new(30).unwrap();
        assert!(!client.is_authenticated());
        
        let status = client.auth_status();
        assert!(!status.configured);
        assert!(status.client_id.is_none());
    }
    
    #[test]
    fn test_authenticated_client_creation() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        let profile = create_test_profile();
        
        let client = AuthenticatedHttpClient::with_authentication(
            30,
            keypair,
            profile.clone(),
            None,
        ).unwrap();
        
        assert!(client.is_authenticated());
        
        let status = client.auth_status();
        assert!(status.configured);
        assert_eq!(status.client_id.as_ref().unwrap(), &profile.client_id);
        assert_eq!(status.key_id.as_ref().unwrap(), &profile.key_id);
        assert_eq!(status.server_url.as_ref().unwrap(), &profile.server_url);
    }
    
    #[test]
    fn test_http_client_builder() {
        let builder = HttpClientBuilder::new()
            .timeout_secs(60)
            .retry_config(RetryConfig {
                max_retries: 5,
                initial_delay_ms: 500,
                max_delay_ms: 10000,
                retry_server_errors: true,
                retry_network_errors: true,
            });
        
        let client = builder.build().unwrap();
        assert!(!client.is_authenticated());
        assert_eq!(client.retry_config.max_retries, 5);
        assert_eq!(client.retry_config.initial_delay_ms, 500);
    }
    
    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert!(config.retry_server_errors);
        assert!(config.retry_network_errors);
    }
}