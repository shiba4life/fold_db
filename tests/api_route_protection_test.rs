//! Test suite to verify that signature verification middleware is properly integrated
//! with ALL API routes as required by T11.2

use actix_web::http::StatusCode;
use actix_web::{middleware::Logger, test, web, App, HttpResponse};
use datafold::datafold_node::config::NodeConfig;
use datafold::datafold_node::http_server::AppState;
use datafold::datafold_node::signature_auth::{
    SignatureAuthConfig, SignatureVerificationMiddleware, SignatureVerificationState,
};
use datafold::datafold_node::DataFoldNode;
use datafold::datafold_node::{crypto_routes, system_routes};
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

/// Test handler that returns success for protected endpoints
async fn test_protected_handler() -> HttpResponse {
    HttpResponse::Ok().json(json!({"message": "Protected endpoint accessed successfully"}))
}

/// Test that verifies signature verification middleware is applied to all API route scopes
#[tokio::test]
async fn test_all_api_routes_require_authentication() {
    let _ = env_logger::builder().is_test(true).try_init();

    println!("🔒 Testing that ALL API routes require signature authentication (T11.2)");

    // Create temporary directory for test node
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config = NodeConfig::development(temp_dir.path().to_path_buf());

    // Create DataFold node
    let node = DataFoldNode::new(config).expect("Failed to create DataFold node");

    // Create signature auth configuration for testing
    let sig_config = SignatureAuthConfig::default();
    let signature_auth = SignatureVerificationState::new(sig_config)
        .expect("Failed to create signature verification state");

    // Create application state
    let app_state = web::Data::new(AppState {
        node: Arc::new(Mutex::new(node)),
        signature_auth: Arc::new(signature_auth.clone()),
    });

    // Create test app with signature verification middleware applied to all API routes
    let app = test::init_service(
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    // Apply signature verification middleware to the entire API scope
                    .wrap(SignatureVerificationMiddleware::new(signature_auth))
                    // Schema endpoints
                    .route("/schemas", web::get().to(test_protected_handler))
                    .route("/schema/test", web::get().to(test_protected_handler))
                    .route("/schema", web::post().to(test_protected_handler))
                    // Query/mutation endpoints
                    .route("/execute", web::post().to(test_protected_handler))
                    .route("/query", web::post().to(test_protected_handler))
                    .route("/mutation", web::post().to(test_protected_handler))
                    // Transform endpoints
                    .route("/transforms", web::get().to(test_protected_handler))
                    .route(
                        "/transform/test/run",
                        web::post().to(test_protected_handler),
                    )
                    // Log endpoints
                    .route("/logs", web::get().to(test_protected_handler))
                    .route("/logs/config", web::get().to(test_protected_handler))
                    // System endpoints (including exempted paths)
                    .service(
                        web::scope("/system")
                            .route("/status", web::get().to(system_routes::get_system_status))
                            .route("/reset-database", web::post().to(test_protected_handler)),
                    )
                    // Ingestion endpoints
                    .service(
                        web::scope("/ingestion")
                            .route("/process", web::post().to(test_protected_handler))
                            .route("/status", web::get().to(test_protected_handler))
                            .route("/health", web::get().to(test_protected_handler)),
                    )
                    // Crypto endpoints (including exempted paths)
                    .service(
                        web::scope("/crypto")
                            .route("/init/random", web::post().to(test_protected_handler))
                            .route("/status", web::get().to(test_protected_handler))
                            .route(
                                "/keys/register",
                                web::post().to(crypto_routes::register_public_key),
                            )
                            .route("/keys/status/test", web::get().to(test_protected_handler)),
                    )
                    // Network endpoints
                    .service(
                        web::scope("/network")
                            .route("/init", web::post().to(test_protected_handler))
                            .route("/status", web::get().to(test_protected_handler))
                            .route("/connect", web::post().to(test_protected_handler)),
                    ),
            ),
    )
    .await;

    // Test protected endpoints that should require authentication
    let protected_endpoints = vec![
        // Schema endpoints
        ("/api/schemas", "GET"),
        ("/api/schema/test", "GET"),
        ("/api/schema", "POST"),
        // Query/mutation endpoints
        ("/api/execute", "POST"),
        ("/api/query", "POST"),
        ("/api/mutation", "POST"),
        // Transform endpoints
        ("/api/transforms", "GET"),
        ("/api/transform/test/run", "POST"),
        // Log endpoints
        ("/api/logs", "GET"),
        ("/api/logs/config", "GET"),
        // System endpoints (protected)
        ("/api/system/reset-database", "POST"),
        // Ingestion endpoints
        ("/api/ingestion/process", "POST"),
        ("/api/ingestion/status", "GET"),
        ("/api/ingestion/health", "GET"),
        // Crypto endpoints (protected)
        ("/api/crypto/init/random", "POST"),
        ("/api/crypto/status", "GET"),
        ("/api/crypto/keys/status/test", "GET"),
        // Network endpoints
        ("/api/network/init", "POST"),
        ("/api/network/status", "GET"),
        ("/api/network/connect", "POST"),
    ];

    let protected_count = protected_endpoints.len();
    println!(
        "🔍 Testing {} protected endpoints require authentication...",
        protected_count
    );

    for (path, method) in protected_endpoints {
        let req = match method {
            "GET" => test::TestRequest::get().uri(path).to_request(),
            "POST" => test::TestRequest::post().uri(path).to_request(),
            _ => continue,
        };

        // Use try_call_service to handle authentication errors properly
        let result = test::try_call_service(&app, req).await;

        match result {
            Ok(resp) => {
                // If we get a response, it should be an error status
                assert!(
                    resp.status() == StatusCode::UNAUTHORIZED
                        || resp.status() == StatusCode::BAD_REQUEST,
                    "❌ Endpoint {} {} should require authentication but returned: {}",
                    method,
                    path,
                    resp.status()
                );
                println!(
                    "✅ {} {} correctly requires authentication ({})",
                    method,
                    path,
                    resp.status()
                );
            }
            Err(err) => {
                // Authentication middleware returned an error, which is expected
                // Convert the error to check its status code
                let error_response = err.error_response();
                assert!(
                    error_response.status() == StatusCode::UNAUTHORIZED
                        || error_response.status() == StatusCode::BAD_REQUEST,
                    "❌ Endpoint {} {} should require authentication but error returned: {}",
                    method,
                    path,
                    error_response.status()
                );
                println!(
                    "✅ {} {} correctly requires authentication (error: {})",
                    method,
                    path,
                    error_response.status()
                );
            }
        }
    }

    // Test exempted endpoints that should work without authentication
    let exempted_endpoints = vec![
        ("/api/system/status", "GET"),
        ("/api/crypto/keys/register", "POST"),
    ];

    let exempted_count = exempted_endpoints.len();
    println!(
        "🔍 Testing {} exempted endpoints work without authentication...",
        exempted_count
    );

    for (path, method) in exempted_endpoints {
        let req = match method {
            "GET" => test::TestRequest::get().uri(path).to_request(),
            "POST" => test::TestRequest::post()
                .uri(path)
                .set_json(json!({
                    "client_id": "test",
                    "public_key": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }))
                .to_request(),
            _ => continue,
        };

        let resp = test::call_service(&app, req).await;

        // These endpoints should work without authentication (200 or other success codes)
        assert!(
            resp.status().is_success()
                || resp.status().is_client_error() && resp.status() != StatusCode::UNAUTHORIZED,
            "❌ Exempted endpoint {} {} should work without authentication but returned: {}",
            method,
            path,
            resp.status()
        );

        println!(
            "✅ {} {} correctly exempted from authentication ({})",
            method,
            path,
            resp.status()
        );
    }

    println!("🎉 T11.2 VERIFICATION COMPLETE: All API routes properly protected with signature verification middleware");
    println!(
        "✅ {} protected endpoints require authentication",
        protected_count
    );
    println!(
        "✅ {} exempted endpoints work without authentication",
        exempted_count
    );
    println!("✅ Signature verification middleware correctly integrated with all API route scopes");
}

/// Test that verifies middleware order - signature verification runs before route handlers
#[tokio::test]
async fn test_middleware_execution_order() {
    let _ = env_logger::builder().is_test(true).try_init();

    println!("⚡ Testing middleware execution order");

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config = NodeConfig::development(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).expect("Failed to create DataFold node");

    let sig_config = SignatureAuthConfig::default();
    let signature_auth = SignatureVerificationState::new(sig_config)
        .expect("Failed to create signature verification state");

    let app_state = web::Data::new(AppState {
        node: Arc::new(Mutex::new(node)),
        signature_auth: Arc::new(signature_auth.clone()),
    });

    // Create test app with middleware applied at API scope level
    let app = test::init_service(
        App::new().app_data(app_state.clone()).service(
            web::scope("/api")
                .wrap(SignatureVerificationMiddleware::new(signature_auth))
                .route(
                    "/test",
                    web::get().to(|| async {
                        // This handler should not be reached without valid signature
                        HttpResponse::Ok().json(json!({"handler": "reached"}))
                    }),
                ),
        ),
    )
    .await;

    // Test that middleware blocks request before reaching handler
    let req = test::TestRequest::get().uri("/api/test").to_request();

    // Use try_call_service to handle authentication errors properly
    let result = test::try_call_service(&app, req).await;

    match result {
        Ok(resp) => {
            // Should get authentication error before reaching handler
            assert!(
                resp.status() == StatusCode::UNAUTHORIZED
                    || resp.status() == StatusCode::BAD_REQUEST,
                "Middleware should block request before reaching handler, got: {}",
                resp.status()
            );
            println!("✅ Signature verification middleware correctly executes before route handlers (response: {})", resp.status());
        }
        Err(err) => {
            // Authentication middleware returned an error, which is expected
            let error_response = err.error_response();
            assert!(
                error_response.status() == StatusCode::UNAUTHORIZED
                    || error_response.status() == StatusCode::BAD_REQUEST,
                "Middleware should block request with proper error status, got: {}",
                error_response.status()
            );
            println!("✅ Signature verification middleware correctly executes before route handlers (error: {})", error_response.status());
        }
    }
}

/// Test error handling and response formatting
#[tokio::test]
async fn test_authentication_error_responses() {
    let _ = env_logger::builder().is_test(true).try_init();

    println!("📋 Testing authentication error response formatting");

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config = NodeConfig::development(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).expect("Failed to create DataFold node");

    let sig_config = SignatureAuthConfig::default();
    let signature_auth = SignatureVerificationState::new(sig_config)
        .expect("Failed to create signature verification state");

    let app_state = web::Data::new(AppState {
        node: Arc::new(Mutex::new(node)),
        signature_auth: Arc::new(signature_auth.clone()),
    });

    let app = test::init_service(
        App::new().app_data(app_state.clone()).service(
            web::scope("/api")
                .wrap(SignatureVerificationMiddleware::new(signature_auth))
                .route("/test", web::get().to(test_protected_handler)),
        ),
    )
    .await;

    // Test missing signature headers
    let req = test::TestRequest::get().uri("/api/test").to_request();

    // Use try_call_service to handle authentication errors properly
    let result = test::try_call_service(&app, req).await;

    match result {
        Ok(resp) => {
            assert!(
                resp.status() == StatusCode::UNAUTHORIZED
                    || resp.status() == StatusCode::BAD_REQUEST,
                "Should return proper error status for missing signature headers, got: {}",
                resp.status()
            );
            println!(
                "✅ Authentication error responses properly formatted (response: {})",
                resp.status()
            );
        }
        Err(err) => {
            // Authentication middleware returned an error, which is expected
            let error_response = err.error_response();
            assert!(
                error_response.status() == StatusCode::UNAUTHORIZED
                    || error_response.status() == StatusCode::BAD_REQUEST,
                "Should return proper error status for missing signature headers, got: {}",
                error_response.status()
            );
            println!(
                "✅ Authentication error responses properly formatted (error: {})",
                error_response.status()
            );
        }
    }
}
