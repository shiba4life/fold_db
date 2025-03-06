use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::config::SandboxConfig;
use super::types::{OperationType, Request, Response, SandboxError, SandboxResult};

/// Security middleware for sandboxed containers
pub struct SecurityMiddleware {
    /// Allowed operations for containers
    allowed_operations: HashSet<OperationType>,
    /// Rate limiter for API calls
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// Configuration
    config: SandboxConfig,
}

impl SecurityMiddleware {
    /// Creates a new security middleware
    pub fn new(config: SandboxConfig) -> Self {
        // Convert allowed operations strings to enum values
        let mut allowed_operations = HashSet::new();
        for op in &config.allowed_operations {
            match op.to_lowercase().as_str() {
                "query" => allowed_operations.insert(OperationType::Query),
                "mutation" => allowed_operations.insert(OperationType::Mutation),
                "schema" => allowed_operations.insert(OperationType::Schema),
                "node" => allowed_operations.insert(OperationType::Node),
                "app" => allowed_operations.insert(OperationType::App),
                "system" => allowed_operations.insert(OperationType::System),
                _ => false,
            };
        }

        Self {
            allowed_operations,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(
                config.rate_limits.requests_per_minute,
                config.rate_limits.max_concurrent_requests,
            ))),
            config,
        }
    }

    /// Validates a request from a container
    pub fn validate_request(&self, request: &Request) -> SandboxResult<()> {
        // Determine operation type from request path
        let operation_type = self.determine_operation_type(request)?;

        // Check if operation is allowed
        if !self.allowed_operations.contains(&operation_type) {
            return Err(SandboxError::Security(format!(
                "Operation type {:?} is not allowed",
                operation_type
            )));
        }

        // Check request size
        if let Some(body) = &request.body {
            if body.len() as u64 > self.config.rate_limits.max_request_size {
                return Err(SandboxError::Security(format!(
                    "Request size {} exceeds maximum allowed size {}",
                    body.len(),
                    self.config.rate_limits.max_request_size
                )));
            }
        }

        Ok(())
    }

    /// Checks rate limits for a container
    pub fn check_rate_limit(&self, container_id: &str) -> SandboxResult<()> {
        let mut rate_limiter = self.rate_limiter
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock rate limiter".to_string()))?;

        rate_limiter.check_rate_limit(container_id)
    }

    /// Determines the operation type from a request
    fn determine_operation_type(&self, request: &Request) -> SandboxResult<OperationType> {
        // Extract operation type from path
        let path = request.path.to_lowercase();
        
        if path.starts_with("/query") || path.starts_with("/api/query") {
            Ok(OperationType::Query)
        } else if path.starts_with("/mutation") || path.starts_with("/api/mutation") {
            Ok(OperationType::Mutation)
        } else if path.starts_with("/schema") || path.starts_with("/api/schema") {
            Ok(OperationType::Schema)
        } else if path.starts_with("/node") || path.starts_with("/api/node") {
            Ok(OperationType::Node)
        } else if path.starts_with("/app") || path.starts_with("/api/app") {
            Ok(OperationType::App)
        } else if path.starts_with("/system") || path.starts_with("/api/system") {
            Ok(OperationType::System)
        } else {
            Err(SandboxError::Security(format!(
                "Unknown operation type for path: {}",
                path
            )))
        }
    }

    /// Creates an error response
    pub fn create_error_response(&self, error: SandboxError) -> Response {
        let body = format!("{{\"error\": \"{}\"}}", error);
        
        Response {
            status: 403,
            headers: HashMap::new(),
            body: Some(body.into_bytes()),
        }
    }
}

/// Rate limiter for API calls
struct RateLimiter {
    /// Maximum number of requests per minute
    requests_per_minute: u32,
    /// Maximum number of concurrent requests
    max_concurrent_requests: u32,
    /// Request counts per container
    request_counts: HashMap<String, Vec<Instant>>,
    /// Concurrent request counts per container
    concurrent_requests: HashMap<String, u32>,
}

impl RateLimiter {
    /// Creates a new rate limiter
    pub fn new(requests_per_minute: u32, max_concurrent_requests: u32) -> Self {
        Self {
            requests_per_minute,
            max_concurrent_requests,
            request_counts: HashMap::new(),
            concurrent_requests: HashMap::new(),
        }
    }

    /// Checks rate limits for a container
    pub fn check_rate_limit(&mut self, container_id: &str) -> SandboxResult<()> {
        // Check concurrent requests
        let concurrent = self.concurrent_requests.entry(container_id.to_string()).or_insert(0);
        if *concurrent >= self.max_concurrent_requests {
            return Err(SandboxError::Security(format!(
                "Too many concurrent requests for container {}",
                container_id
            )));
        }
        *concurrent += 1;

        // Check requests per minute
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);

        // Get or create request timestamps for this container
        let timestamps = self.request_counts.entry(container_id.to_string()).or_insert_with(Vec::new);
        
        // Remove timestamps older than a minute
        timestamps.retain(|&timestamp| timestamp > minute_ago);
        
        // Check if we've exceeded the rate limit
        if timestamps.len() as u32 >= self.requests_per_minute {
            return Err(SandboxError::Security(format!(
                "Rate limit exceeded for container {}",
                container_id
            )));
        }
        
        // Add the current timestamp
        timestamps.push(now);
        
        Ok(())
    }

    /// Completes a request for a container
    pub fn complete_request(&mut self, container_id: &str) {
        if let Some(concurrent) = self.concurrent_requests.get_mut(container_id) {
            if *concurrent > 0 {
                *concurrent -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_middleware() {
        let config = SandboxConfig::default();
        let middleware = SecurityMiddleware::new(config);

        // Test allowed operation
        let request = Request {
            container_id: "test".to_string(),
            path: "/query".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        assert!(middleware.validate_request(&request).is_ok());

        // Test disallowed operation (mutation is not in default allowed_operations)
        let request = Request {
            container_id: "test".to_string(),
            path: "/mutation".to_string(),
            method: "POST".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        assert!(middleware.validate_request(&request).is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let mut rate_limiter = RateLimiter::new(5, 2);

        // Test concurrent requests
        assert!(rate_limiter.check_rate_limit("test").is_ok());
        assert!(rate_limiter.check_rate_limit("test").is_ok());
        assert!(rate_limiter.check_rate_limit("test").is_err()); // Exceeds max_concurrent_requests

        // Complete a request
        rate_limiter.complete_request("test");
        assert!(rate_limiter.check_rate_limit("test").is_ok());

        // Test requests per minute
        let mut rate_limiter = RateLimiter::new(3, 10);
        assert!(rate_limiter.check_rate_limit("test2").is_ok());
        assert!(rate_limiter.check_rate_limit("test2").is_ok());
        assert!(rate_limiter.check_rate_limit("test2").is_ok());
        assert!(rate_limiter.check_rate_limit("test2").is_err()); // Exceeds requests_per_minute
    }
}
