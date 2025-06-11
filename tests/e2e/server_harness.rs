//! Test Server Harness for E2E Integration Testing
//!
//! This module provides a test harness for managing DataFold server instances
//! during end-to-end integration testing, including server lifecycle management,
//! configuration, and testing utilities.

use datafold::datafold_node::config::NodeConfig;
use datafold::datafold_node::signature_auth::{SignatureAuthConfig, SecurityProfile};
use datafold::datafold_node::DataFoldNode;
use datafold::datafold_node::http_server::AppState;
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Test server harness for E2E testing
pub struct TestServerHarness {
    config: NodeConfig,
    temp_dir: TempDir,
    server_handle: Option<JoinHandle<Result<(), std::io::Error>>>,
    server_running: Arc<AtomicBool>,
    port: u16,
    base_url: String,
}

impl TestServerHarness {
    /// Create a new test server harness
    pub async fn new(security_profile: SecurityProfile) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let port = Self::find_available_port().await?;
        
        let mut config = match security_profile {
            SecurityProfile::Strict => NodeConfig::production_with_signature_auth(temp_dir.path().to_path_buf()),
            SecurityProfile::Standard => NodeConfig::with_optional_signature_auth(temp_dir.path().to_path_buf()),
            SecurityProfile::Lenient => NodeConfig::development_with_signature_auth(temp_dir.path().to_path_buf()),
        };

        // Configure for testing
        Self::configure_for_testing(&mut config)?;

        let base_url = format!("http://localhost:{}", port);

        Ok(Self {
            config,
            temp_dir,
            server_handle: None,
            server_running: Arc::new(AtomicBool::new(false)),
            port,
            base_url,
        })
    }

    /// Create a server harness with custom configuration
    pub async fn with_config(mut config: NodeConfig) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let port = Self::find_available_port().await?;
        
        Self::configure_for_testing(&mut config)?;
        
        let base_url = format!("http://localhost:{}", port);

        Ok(Self {
            config,
            temp_dir,
            server_handle: None,
            server_running: Arc::new(AtomicBool::new(false)),
            port,
            base_url,
        })
    }

    /// Start the test server
    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.server_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        log::info!("🚀 Starting test server on port {}", self.port);

        // Create DataFold node
        let node = DataFoldNode::new(self.config.clone())?;
        let app_state = web::Data::new(AppState {
            node: Arc::new(Mutex::new(node)),
            signature_auth: None,
        });

        let server_running = self.server_running.clone();
        let port = self.port;

        // Start HTTP server
        let server_handle = tokio::spawn(async move {
            let server = HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .app_data(app_state.clone())
                    .service(
                        web::scope("/api")
                            .service(Self::create_test_routes())
                            .service(Self::create_crypto_routes())
                            .service(Self::create_system_routes())
                    )
            })
            .bind(("127.0.0.1", port))?
            .workers(1)
            .run();

            server_running.store(true, Ordering::Relaxed);
            
            server.await
        });

        self.server_handle = Some(server_handle);

        // Wait for server to start
        self.wait_for_startup().await?;

        log::info!("✅ Test server started successfully on {}", self.base_url);
        Ok(())
    }

    /// Stop the test server
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.server_handle.take() {
            self.server_running.store(false, Ordering::Relaxed);
            handle.abort();
            
            // Give the server time to shut down gracefully
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            log::info!("🛑 Test server stopped");
        }
        Ok(())
    }

    /// Get the server base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the server port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }

    /// Wait for server to start up
    async fn wait_for_startup(&self) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let health_url = format!("{}/api/system/status", self.base_url);
        
        for attempt in 1..=30 {
            if let Ok(response) = client.get(&health_url).send().await {
                if response.status().is_success() {
                    log::info!("✓ Server health check passed on attempt {}", attempt);
                    return Ok(());
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(anyhow::anyhow!("Server failed to start within timeout"))
    }

    /// Simulate temporary server failure for resilience testing
    pub async fn simulate_temporary_failure(&mut self) -> anyhow::Result<()> {
        log::info!("🔄 Simulating temporary server failure");
        
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Restart the server
        self.start().await?;
        
        log::info!("✅ Server recovered from simulated failure");
        Ok(())
    }

    /// Configure node for testing
    fn configure_for_testing(config: &mut NodeConfig) -> anyhow::Result<()> {
        // Enable signature authentication if not already enabled
        if config.signature_auth_config.is_none() {
            config.signature_auth_config = Some(SignatureAuthConfig::default());
        }

        if let Some(ref mut auth_config) = config.signature_auth_config {
            auth_config.enabled = true;
            auth_config.log_replay_attempts = true;
            auth_config.security_logging.log_successful_auth = true;
            auth_config.security_logging.include_correlation_ids = true;
            
            // Adjust for testing environment
            auth_config.allowed_time_window_secs = 600; // Generous for test timing
            auth_config.clock_skew_tolerance_secs = 60;
            auth_config.nonce_ttl_secs = 600;
            auth_config.max_nonce_store_size = 1000;
        }

        Ok(())
    }

    /// Find an available port for testing
    async fn find_available_port() -> anyhow::Result<u16> {
        use std::net::{TcpListener, SocketAddr};

        // Try to bind to port 0 to get an available port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        drop(listener);

        Ok(addr.port())
    }

    /// Create test-specific routes
    fn create_test_routes() -> actix_web::Scope {
        use actix_web::{HttpResponse, Result};

        web::scope("/test")
            .route("/authenticated", web::post().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Authenticated endpoint accessed successfully",
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }))
            }))
            .route("/client/{id}", web::get().to(|path: web::Path<usize>| async move {
                let client_id = path.into_inner();
                HttpResponse::Ok().json(serde_json::json!({
                    "client_id": client_id,
                    "message": format!("Response for client {}", client_id),
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }))
            }))
            .route("/before_rotation", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Request processed before key rotation"
                }))
            }))
            .route("/after_rotation", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Request processed after key rotation"
                }))
            }))
            .route("/old_key_still_works", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Old key still functional"
                }))
            }))
            .route("/unregistered_key", web::get().to(|| async {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unregistered key"
                }))
            }))
            .route("/after_recovery", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Server recovered successfully"
                }))
            }))
            .route("/activity/{id}", web::get().to(|path: web::Path<usize>| async move {
                let activity_id = path.into_inner();
                HttpResponse::Ok().json(serde_json::json!({
                    "activity_id": activity_id,
                    "message": format!("Activity {} recorded", activity_id)
                }))
            }))
    }

    /// Create crypto-related routes
    fn create_crypto_routes() -> actix_web::Scope {
        use actix_web::{HttpResponse, Result};
        use datafold::datafold_node::crypto_routes::PublicKeyRegistrationRequest;

        web::scope("/crypto")
            .route("/keys/register", web::post().to(|req: web::Json<PublicKeyRegistrationRequest>| async move {
                let registration_id = uuid::Uuid::new_v4().to_string();
                HttpResponse::Ok().json(serde_json::json!({
                    "registration_id": registration_id,
                    "client_id": req.client_id,
                    "status": "registered",
                    "message": "Public key registered successfully"
                }))
            }))
            .route("/keys/status/{id}", web::get().to(|path: web::Path<String>| async move {
                let registration_id = path.into_inner();
                HttpResponse::Ok().json(serde_json::json!({
                    "registration_id": registration_id,
                    "status": "active",
                    "registered_at": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }))
            }))
            .route("/status", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "crypto_status": "operational",
                    "signature_auth_enabled": true,
                    "supported_algorithms": ["ed25519"]
                }))
            }))
    }

    /// Create system routes
    fn create_system_routes() -> actix_web::Scope {
        use actix_web::HttpResponse;

        web::scope("/system")
            .route("/status", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy",
                    "version": "test-harness-1.0.0",
                    "uptime_seconds": 0,
                    "features": {
                        "signature_auth": true,
                        "test_mode": true
                    }
                }))
            }))
    }

    /// Create session management routes
    fn create_session_routes() -> actix_web::Scope {
        use actix_web::{HttpResponse, web::Json};
        use serde_json::Value;

        web::scope("/session")
            .route("/establish", web::post().to(|_req: Json<Value>| async {
                let session_id = uuid::Uuid::new_v4().to_string();
                HttpResponse::Ok().json(serde_json::json!({
                    "session_id": session_id,
                    "status": "established",
                    "expires_at": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() + 1800 // 30 minutes
                }))
            }))
            .route("/validate", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "valid": true,
                    "remaining_time_seconds": 1500
                }))
            }))
            .route("/terminate", web::post().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "terminated"
                }))
            }))
    }
}

impl Drop for TestServerHarness {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

/// Server configuration builder for specific test scenarios
pub struct TestServerConfigBuilder {
    security_profile: SecurityProfile,
    custom_auth_config: Option<SignatureAuthConfig>,
    enable_attack_simulation: bool,
    custom_port: Option<u16>,
}

impl TestServerConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            security_profile: SecurityProfile::Standard,
            custom_auth_config: None,
            enable_attack_simulation: false,
            custom_port: None,
        }
    }

    /// Set security profile
    pub fn security_profile(mut self, profile: SecurityProfile) -> Self {
        self.security_profile = profile;
        self
    }

    /// Set custom authentication configuration
    pub fn auth_config(mut self, config: SignatureAuthConfig) -> Self {
        self.custom_auth_config = Some(config);
        self
    }

    /// Enable attack simulation features
    pub fn enable_attack_simulation(mut self) -> Self {
        self.enable_attack_simulation = true;
        self
    }

    /// Set custom port
    pub fn port(mut self, port: u16) -> Self {
        self.custom_port = Some(port);
        self
    }

    /// Build the test server harness
    pub async fn build(self) -> anyhow::Result<TestServerHarness> {
        let mut harness = TestServerHarness::new(self.security_profile).await?;

        if let Some(auth_config) = self.custom_auth_config {
            if let Some(ref mut config_auth) = harness.config.signature_auth_config {
                *config_auth = auth_config;
            }
        }

        if let Some(port) = self.custom_port {
            harness.port = port;
            harness.base_url = format!("http://localhost:{}", port);
        }

        Ok(harness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::e2e::init_e2e_environment;

    #[tokio::test]
    async fn test_server_harness_lifecycle() {
        init_e2e_environment();

        let mut harness = TestServerHarness::new(SecurityProfile::Standard).await.unwrap();
        
        assert!(!harness.is_running());
        
        harness.start().await.unwrap();
        assert!(harness.is_running());
        
        harness.stop().await.unwrap();
        assert!(!harness.is_running());
    }

    #[tokio::test]
    async fn test_server_health_check() {
        init_e2e_environment();

        let mut harness = TestServerHarness::new(SecurityProfile::Lenient).await.unwrap();
        harness.start().await.unwrap();

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/api/system/status", harness.base_url()))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        harness.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_server_config_builder() {
        init_e2e_environment();

        let harness = TestServerConfigBuilder::new()
            .security_profile(SecurityProfile::Strict)
            .enable_attack_simulation()
            .build()
            .await
            .unwrap();

        assert!(!harness.is_running());
        assert!(harness.base_url().starts_with("http://localhost:"));
    }
}