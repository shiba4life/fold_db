//! Real-World Scenario Testing
//!
//! This module tests the authentication system under realistic conditions
//! including concurrent clients, high-load scenarios, network failures,
//! and time synchronization challenges.

use super::test_utils::{E2ETestConfig, TestCredentials, E2EHttpClient, E2ETestResults, utils};
use super::server_harness::TestServerHarness;
use super::sdk_harness::{JavaScriptSDKHarness, PythonSDKHarness, CLIHarness};
use datafold::datafold_node::signature_auth::SecurityProfile;
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tokio::time::{timeout, sleep};

/// Real-world scenario test suite
pub struct RealWorldScenarios {
    config: E2ETestConfig,
    results: E2ETestResults,
}

impl RealWorldScenarios {
    /// Create new real-world scenario test suite
    pub fn new(config: E2ETestConfig) -> Self {
        Self {
            config,
            results: E2ETestResults::default(),
        }
    }

    /// Run all real-world scenario tests
    pub async fn run_all_tests(&mut self) -> anyhow::Result<E2ETestResults> {
        log::info!("🌍 Starting Real-World Scenario Tests");

        // Test 1: Concurrent client authentication
        self.test_concurrent_client_authentication().await?;

        // Test 2: High-load authentication scenarios
        self.test_high_load_authentication().await?;

        // Test 3: Network failure and retry testing
        self.test_network_failure_scenarios().await?;

        // Test 4: Time synchronization and clock skew
        self.test_time_synchronization_scenarios().await?;

        // Test 5: Long-running session management
        self.test_long_running_sessions().await?;

        // Test 6: Mixed traffic scenarios
        self.test_mixed_traffic_scenarios().await?;

        // Test 7: Resource exhaustion scenarios
        self.test_resource_exhaustion_scenarios().await?;

        log::info!("✅ Real-World Scenario Tests Complete: {}/{} passed", 
                  self.results.passed_tests, self.results.total_tests);

        Ok(self.results.clone())
    }

    /// Test concurrent client authentication under load
    async fn test_concurrent_client_authentication(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing concurrent client authentication");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs * 2), async {
            // Step 1: Start test server
            let mut server_harness = TestServerHarness::new(SecurityProfile::Standard).await?;
            server_harness.start().await?;

            // Step 2: Generate multiple client credentials
            let mut credentials_list = Vec::new();
            for i in 0..self.config.concurrent_clients {
                let credentials = TestCredentials::generate()?;
                
                // Register each client's public key
                let mut server_client = E2EHttpClient::new(server_harness.base_url().to_string());
                server_client.wait_for_server(10).await?;
                
                let registration_id = server_client.register_public_key(&credentials).await?;
                log::info!("✓ Client {} registered: {}", i + 1, registration_id);
                
                credentials_list.push(credentials);
            }

            // Step 3: Concurrent authentication stress test
            let success_counter = Arc::new(AtomicUsize::new(0));
            let error_counter = Arc::new(AtomicUsize::new(0));
            let semaphore = Arc::new(Semaphore::new(self.config.concurrent_clients));

            let mut handles = Vec::new();

            for (i, credentials) in credentials_list.into_iter().enumerate() {
                let server_harness_url = server_harness.base_url().to_string();
                let success_counter = success_counter.clone();
                let error_counter = error_counter.clone();
                let semaphore = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Each client makes multiple requests
                    for request_num in 1..=10 {
                        let client = E2EHttpClient::new(server_harness_url.clone())
                            .with_credentials(credentials.clone());

                        let result = client.authenticated_request(
                            "POST",
                            &format!("/api/test/concurrent/{}/{}", i, request_num),
                            Some(json!({
                                "client_id": credentials.client_id,
                                "request_number": request_num,
                                "timestamp": utils::current_timestamp()
                            })),
                        ).await;

                        match result {
                            Ok(response) if response.status().is_success() => {
                                success_counter.fetch_add(1, Ordering::Relaxed);
                            }
                            Ok(_) | Err(_) => {
                                error_counter.fetch_add(1, Ordering::Relaxed);
                            }
                        }

                        // Small delay to simulate real-world usage
                        sleep(Duration::from_millis(50)).await;
                    }
                });

                handles.push(handle);
            }

            // Step 4: Wait for all concurrent operations to complete
            let start_time = SystemTime::now();
            
            for handle in handles {
                if let Err(e) = handle.await {
                    log::warn!("Concurrent client task failed: {}", e);
                }
            }

            let duration = start_time.elapsed().unwrap();
            let successful_requests = success_counter.load(Ordering::Relaxed);
            let failed_requests = error_counter.load(Ordering::Relaxed);
            let total_requests = successful_requests + failed_requests;

            log::info!("✓ Concurrent test completed: {}/{} requests successful in {:?}", 
                      successful_requests, total_requests, duration);

            // Record performance metrics
            self.results.add_metric("concurrent_requests_per_second", 
                total_requests as f64 / duration.as_secs_f64());
            self.results.add_metric("concurrent_success_rate", 
                successful_requests as f64 / total_requests as f64);

            // Success criteria: at least 90% success rate
            if (successful_requests as f64 / total_requests as f64) < 0.9 {
                return Err(anyhow::anyhow!("Concurrent authentication success rate too low: {:.2}%",
                    (successful_requests as f64 / total_requests as f64) * 100.0));
            }

            // Step 5: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("concurrent_client_authentication", true, None);
                log::info!("✅ Concurrent client authentication test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("concurrent_client_authentication", false, Some(e.to_string()));
                log::error!("❌ Concurrent client authentication test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("concurrent_client_authentication", false, Some("Test timeout".to_string()));
                log::error!("❌ Concurrent client authentication test timed out");
            }
        }

        Ok(())
    }

    /// Test high-load authentication scenarios
    async fn test_high_load_authentication(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing high-load authentication scenarios");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs * 3), async {
            // Step 1: Start test server with high-performance configuration
            let mut server_harness = TestServerHarness::new(SecurityProfile::Standard).await?;
            server_harness.start().await?;

            // Step 2: Generate test credentials
            let credentials = TestCredentials::generate()?;
            let mut server_client = E2EHttpClient::new(server_harness.base_url().to_string());
            server_client.wait_for_server(20).await?;
            server_client.register_public_key(&credentials).await?;

            // Step 3: Sustained high-load test
            let load_test_duration = Duration::from_secs(30);
            let target_rps = 100; // 100 requests per second
            let request_interval = Duration::from_millis(1000 / target_rps);

            let mut request_count = 0;
            let mut success_count = 0;
            let mut error_count = 0;
            let start_time = SystemTime::now();

            log::info!("Starting sustained load test for {:?} at {} RPS", load_test_duration, target_rps);

            while start_time.elapsed().unwrap() < load_test_duration {
                let client = E2EHttpClient::new(server_harness.base_url().to_string())
                    .with_credentials(credentials.clone());

                let request_start = SystemTime::now();
                let result = client.authenticated_request(
                    "GET",
                    &format!("/api/test/load/{}", request_count),
                    None,
                ).await;

                let request_duration = request_start.elapsed().unwrap();
                request_count += 1;

                match result {
                    Ok(response) if response.status().is_success() => {
                        success_count += 1;
                        
                        // Track response times
                        if request_count % 100 == 0 {
                            self.results.add_metric(
                                &format!("load_test_response_time_ms_{}", request_count),
                                request_duration.as_millis() as f64
                            );
                        }
                    }
                    Ok(_) | Err(_) => {
                        error_count += 1;
                    }
                }

                // Rate limiting to maintain target RPS
                if request_start.elapsed().unwrap() < request_interval {
                    let sleep_duration = request_interval - request_start.elapsed().unwrap();
                    sleep(sleep_duration).await;
                }
            }

            let total_duration = start_time.elapsed().unwrap();
            let actual_rps = request_count as f64 / total_duration.as_secs_f64();
            let success_rate = success_count as f64 / request_count as f64;

            log::info!("✓ Load test completed: {} requests in {:?} ({:.1} RPS, {:.1}% success)", 
                      request_count, total_duration, actual_rps, success_rate * 100.0);

            // Record performance metrics
            self.results.add_metric("load_test_rps", actual_rps);
            self.results.add_metric("load_test_success_rate", success_rate);
            self.results.add_metric("load_test_total_requests", request_count as f64);

            // Success criteria: at least 95% success rate under load
            if success_rate < 0.95 {
                return Err(anyhow::anyhow!("High-load success rate too low: {:.2}%", success_rate * 100.0));
            }

            // Step 4: Burst load test
            log::info!("Starting burst load test...");
            let burst_size = 50;
            let burst_start = SystemTime::now();
            
            let mut burst_handles = Vec::new();
            for i in 0..burst_size {
                let client = E2EHttpClient::new(server_harness.base_url().to_string())
                    .with_credentials(credentials.clone());
                
                let handle = tokio::spawn(async move {
                    client.authenticated_request(
                        "POST",
                        &format!("/api/test/burst/{}", i),
                        Some(json!({
                            "burst_request": i,
                            "timestamp": utils::current_timestamp()
                        })),
                    ).await
                });
                
                burst_handles.push(handle);
            }

            let mut burst_success_count = 0;
            for handle in burst_handles {
                match handle.await {
                    Ok(Ok(response)) if response.status().is_success() => {
                        burst_success_count += 1;
                    }
                    _ => {}
                }
            }

            let burst_duration = burst_start.elapsed().unwrap();
            let burst_success_rate = burst_success_count as f64 / burst_size as f64;

            log::info!("✓ Burst test completed: {}/{} successful in {:?} ({:.1}% success)", 
                      burst_success_count, burst_size, burst_duration, burst_success_rate * 100.0);

            self.results.add_metric("burst_test_success_rate", burst_success_rate);
            self.results.add_metric("burst_test_duration_ms", burst_duration.as_millis() as f64);

            if burst_success_rate < 0.8 {
                return Err(anyhow::anyhow!("Burst load success rate too low: {:.2}%", burst_success_rate * 100.0));
            }

            // Step 5: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("high_load_authentication", true, None);
                log::info!("✅ High-load authentication test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("high_load_authentication", false, Some(e.to_string()));
                log::error!("❌ High-load authentication test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("high_load_authentication", false, Some("Test timeout".to_string()));
                log::error!("❌ High-load authentication test timed out");
            }
        }

        Ok(())
    }

    /// Test network failure and retry scenarios
    async fn test_network_failure_scenarios(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing network failure and retry scenarios");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs), async {
            // Step 1: Start test server
            let mut server_harness = TestServerHarness::new(SecurityProfile::Standard).await?;
            server_harness.start().await?;

            // Step 2: Generate test credentials
            let credentials = TestCredentials::generate()?;
            let mut server_client = E2EHttpClient::new(server_harness.base_url().to_string());
            server_client.wait_for_server(20).await?;
            server_client.register_public_key(&credentials).await?;

            // Step 3: Test successful request before failure
            let response = server_client.authenticated_request(
                "GET",
                "/api/test/before-failure",
                None,
            ).await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Initial request failed"));
            }
            log::info!("✓ Initial request successful");

            // Step 4: Simulate temporary server failure
            server_harness.simulate_temporary_failure().await?;
            log::info!("✓ Simulated temporary server failure");

            // Step 5: Test client resilience during failure
            let mut retry_attempts = 0;
            let max_retries = 5;
            let mut recovery_successful = false;

            for attempt in 1..=max_retries {
                sleep(Duration::from_millis(500 * attempt)).await;
                
                let recovery_client = E2EHttpClient::new(server_harness.base_url().to_string())
                    .with_credentials(credentials.clone());

                match recovery_client.health_check().await {
                    Ok(true) => {
                        log::info!("✓ Server recovered after {} attempts", attempt);
                        recovery_successful = true;
                        retry_attempts = attempt;
                        break;
                    }
                    _ => {
                        log::info!("  Retry attempt {}/{} - server still unavailable", attempt, max_retries);
                    }
                }
            }

            if !recovery_successful {
                return Err(anyhow::anyhow!("Server did not recover within {} attempts", max_retries));
            }

            // Step 6: Test successful request after recovery
            let recovery_client = E2EHttpClient::new(server_harness.base_url().to_string())
                .with_credentials(credentials.clone());

            let response = recovery_client.authenticated_request(
                "POST",
                "/api/test/after-recovery",
                Some(json!({
                    "recovery_test": true,
                    "retry_attempts": retry_attempts
                })),
            ).await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Post-recovery request failed"));
            }
            log::info!("✓ Post-recovery request successful");

            // Step 7: Test gradual recovery under load
            let recovery_load_test_size = 20;
            let mut recovery_handles = Vec::new();

            for i in 0..recovery_load_test_size {
                let recovery_client = E2EHttpClient::new(server_harness.base_url().to_string())
                    .with_credentials(credentials.clone());
                
                let handle = tokio::spawn(async move {
                    sleep(Duration::from_millis(i * 50)).await; // Stagger requests
                    
                    recovery_client.authenticated_request(
                        "GET",
                        &format!("/api/test/recovery-load/{}", i),
                        None,
                    ).await
                });

                recovery_handles.push(handle);
            }

            let mut recovery_success_count = 0;
            for handle in recovery_handles {
                match handle.await {
                    Ok(Ok(response)) if response.status().is_success() => {
                        recovery_success_count += 1;
                    }
                    _ => {}
                }
            }

            let recovery_success_rate = recovery_success_count as f64 / recovery_load_test_size as f64;
            log::info!("✓ Recovery load test: {}/{} successful ({:.1}% success)", 
                      recovery_success_count, recovery_load_test_size, recovery_success_rate * 100.0);

            // Record metrics
            self.results.add_metric("network_failure_recovery_attempts", retry_attempts as f64);
            self.results.add_metric("recovery_load_success_rate", recovery_success_rate);

            if recovery_success_rate < 0.8 {
                return Err(anyhow::anyhow!("Recovery load test success rate too low"));
            }

            // Step 8: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("network_failure_scenarios", true, None);
                log::info!("✅ Network failure scenarios test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("network_failure_scenarios", false, Some(e.to_string()));
                log::error!("❌ Network failure scenarios test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("network_failure_scenarios", false, Some("Test timeout".to_string()));
                log::error!("❌ Network failure scenarios test timed out");
            }
        }

        Ok(())
    }

    /// Test time synchronization and clock skew scenarios
    async fn test_time_synchronization_scenarios(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing time synchronization and clock skew scenarios");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs), async {
            // Step 1: Start test server with lenient time settings
            let mut server_harness = TestServerHarness::new(SecurityProfile::Lenient).await?;
            server_harness.start().await?;

            // Step 2: Generate test credentials
            let credentials = TestCredentials::generate()?;
            let mut server_client = E2EHttpClient::new(server_harness.base_url().to_string());
            server_client.wait_for_server(20).await?;
            server_client.register_public_key(&credentials).await?;

            // Step 3: Test normal timestamp (current time)
            let normal_request = server_client.authenticated_request(
                "POST",
                "/api/test/normal-timestamp",
                Some(json!({
                    "test_type": "normal_timestamp",
                    "timestamp": utils::current_timestamp()
                })),
            ).await?;

            if !normal_request.status().is_success() {
                return Err(anyhow::anyhow!("Normal timestamp request failed"));
            }
            log::info!("✓ Normal timestamp request successful");

            // Step 4: Test with small clock skew (should succeed)
            let skewed_timestamp = utils::current_timestamp() - 30; // 30 seconds in the past
            let skew_request = server_client.authenticated_request(
                "POST",
                "/api/test/small-clock-skew",
                Some(json!({
                    "test_type": "small_clock_skew",
                    "timestamp": skewed_timestamp,
                    "skew_seconds": 30
                })),
            ).await?;

            if !skew_request.status().is_success() {
                return Err(anyhow::anyhow!("Small clock skew request should have succeeded"));
            }
            log::info!("✓ Small clock skew handled correctly");

            // Step 5: Test with large clock skew (should fail with lenient settings still allowing some skew)
            let large_skewed_timestamp = utils::current_timestamp() - 1200; // 20 minutes in the past
            let large_skew_request = server_client.authenticated_request(
                "POST",
                "/api/test/large-clock-skew",
                Some(json!({
                    "test_type": "large_clock_skew",
                    "timestamp": large_skewed_timestamp,
                    "skew_seconds": 1200
                })),
            ).await?;

            if large_skew_request.status().is_success() {
                return Err(anyhow::anyhow!("Large clock skew request should have failed"));
            }
            log::info!("✓ Large clock skew properly rejected");

            // Step 6: Test future timestamps within tolerance
            let future_timestamp = utils::current_timestamp() + 60; // 1 minute in future
            let future_request = server_client.authenticated_request(
                "POST",
                "/api/test/future-timestamp",
                Some(json!({
                    "test_type": "future_timestamp",
                    "timestamp": future_timestamp,
                    "future_seconds": 60
                })),
            ).await?;

            // With lenient settings, small future timestamps should be allowed
            if !future_request.status().is_success() {
                log::warn!("Future timestamp request failed (may be expected depending on configuration)");
            } else {
                log::info!("✓ Future timestamp within tolerance accepted");
            }

            // Step 7: Test timezone handling scenarios
            let timezone_scenarios = vec![
                ("UTC", 0),
                ("EST", -5 * 3600),
                ("PST", -8 * 3600),
                ("JST", 9 * 3600),
            ];

            for (timezone_name, offset_seconds) in timezone_scenarios {
                let timezone_timestamp = utils::current_timestamp() + offset_seconds as u64;
                
                // Adjust back to reasonable range
                let adjusted_timestamp = if timezone_timestamp > utils::current_timestamp() + 3600 {
                    utils::current_timestamp() - 10 // 10 seconds ago instead
                } else if timezone_timestamp < utils::current_timestamp() - 3600 {
                    utils::current_timestamp() - 10 // 10 seconds ago instead
                } else {
                    timezone_timestamp
                };

                let tz_request = server_client.authenticated_request(
                    "POST",
                    &format!("/api/test/timezone/{}", timezone_name.to_lowercase()),
                    Some(json!({
                        "test_type": "timezone_test",
                        "timezone": timezone_name,
                        "timestamp": adjusted_timestamp
                    })),
                ).await?;

                if tz_request.status().is_success() {
                    log::info!("✓ Timezone {} handling successful", timezone_name);
                } else {
                    log::warn!("Timezone {} handling failed", timezone_name);
                }
            }

            // Step 8: Test clock drift simulation
            let mut drift_successful_requests = 0;
            let drift_test_count = 10;

            for i in 1..=drift_test_count {
                // Simulate gradual clock drift
                let drift_seconds = i * 5; // 5 seconds drift per iteration
                let drifted_timestamp = utils::current_timestamp() - drift_seconds;

                let drift_request = server_client.authenticated_request(
                    "POST",
                    &format!("/api/test/clock-drift/{}", i),
                    Some(json!({
                        "test_type": "clock_drift",
                        "iteration": i,
                        "drift_seconds": drift_seconds,
                        "timestamp": drifted_timestamp
                    })),
                ).await?;

                if drift_request.status().is_success() {
                    drift_successful_requests += 1;
                }

                sleep(Duration::from_millis(100)).await;
            }

            let drift_success_rate = drift_successful_requests as f64 / drift_test_count as f64;
            log::info!("✓ Clock drift test: {}/{} successful ({:.1}% success)", 
                      drift_successful_requests, drift_test_count, drift_success_rate * 100.0);

            self.results.add_metric("clock_drift_success_rate", drift_success_rate);

            // Step 9: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("time_synchronization_scenarios", true, None);
                log::info!("✅ Time synchronization scenarios test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("time_synchronization_scenarios", false, Some(e.to_string()));
                log::error!("❌ Time synchronization scenarios test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("time_synchronization_scenarios", false, Some("Test timeout".to_string()));
                log::error!("❌ Time synchronization scenarios test timed out");
            }
        }

        Ok(())
    }

    /// Test long-running session management
    async fn test_long_running_sessions(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing long-running session management");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs), async {
            // Step 1: Start test server
            let mut server_harness = TestServerHarness::new(SecurityProfile::Standard).await?;
            server_harness.start().await?;

            // Step 2: Generate test credentials
            let credentials = TestCredentials::generate()?;
            let mut server_client = E2EHttpClient::new(server_harness.base_url().to_string());
            server_client.wait_for_server(20).await?;
            server_client.register_public_key(&credentials).await?;

            // Step 3: Establish long-running session
            let session_duration_minutes = 5;
            let request_interval = Duration::from_secs(10);
            let session_start = SystemTime::now();
            let session_end = session_start + Duration::from_secs(session_duration_minutes * 60);

            log::info!("Starting long-running session test for {} minutes", session_duration_minutes);

            let mut request_count = 0;
            let mut successful_requests = 0;
            let mut consecutive_failures = 0;
            let max_consecutive_failures = 3;

            while SystemTime::now() < session_end {
                request_count += 1;

                let request_result = server_client.authenticated_request(
                    "GET",
                    &format!("/api/test/long-session/{}", request_count),
                    None,
                ).await;

                match request_result {
                    Ok(response) if response.status().is_success() => {
                        successful_requests += 1;
                        consecutive_failures = 0;
                        
                        if request_count % 10 == 0 {
                            log::info!("  Session request {}: successful", request_count);
                        }
                    }
                    _ => {
                        consecutive_failures += 1;
                        log::warn!("  Session request {}: failed (consecutive failures: {})", 
                                  request_count, consecutive_failures);
                        
                        if consecutive_failures >= max_consecutive_failures {
                            return Err(anyhow::anyhow!("Too many consecutive failures in long-running session"));
                        }
                    }
                }

                sleep(request_interval).await;
            }

            let session_duration = SystemTime::now().duration_since(session_start).unwrap();
            let success_rate = successful_requests as f64 / request_count as f64;

            log::info!("✓ Long-running session completed: {}/{} requests successful in {:?} ({:.1}% success)", 
                      successful_requests, request_count, session_duration, success_rate * 100.0);

            // Record metrics
            self.results.add_metric("long_session_duration_minutes", session_duration.as_secs() as f64 / 60.0);
            self.results.add_metric("long_session_success_rate", success_rate);
            self.results.add_metric("long_session_total_requests", request_count as f64);

            if success_rate < 0.95 {
                return Err(anyhow::anyhow!("Long-running session success rate too low: {:.2}%", success_rate * 100.0));
            }

            // Step 4: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("long_running_sessions", true, None);
                log::info!("✅ Long-running sessions test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("long_running_sessions", false, Some(e.to_string()));
                log::error!("❌ Long-running sessions test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("long_running_sessions", false, Some("Test timeout".to_string()));
                log::error!("❌ Long-running sessions test timed out");
            }
        }

        Ok(())
    }

    /// Test mixed traffic scenarios (authenticated and unauthenticated)
    async fn test_mixed_traffic_scenarios(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing mixed traffic scenarios");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs), async {
            // Step 1: Start test server
            let mut server_harness = TestServerHarness::new(SecurityProfile::Standard).await?;
            server_harness.start().await?;

            // Step 2: Generate authenticated and unauthenticated clients
            let authenticated_credentials = TestCredentials::generate()?;
            let mut auth_client = E2EHttpClient::new(server_harness.base_url().to_string());
            auth_client.wait_for_server(20).await?;
            auth_client.register_public_key(&authenticated_credentials).await?;

            let unauth_client = E2EHttpClient::new(server_harness.base_url().to_string());

            // Step 3: Mixed traffic simulation
            let mixed_traffic_duration = Duration::from_secs(60);
            let start_time = SystemTime::now();

            let mut auth_requests = 0;
            let mut auth_successes = 0;
            let mut unauth_requests = 0;
            let mut unauth_successes = 0;

            while start_time.elapsed().unwrap() < mixed_traffic_duration {
                // Alternate between authenticated and unauthenticated requests
                if auth_requests + unauth_requests % 2 == 0 {
                    // Authenticated request
                    auth_requests += 1;
                    let result = auth_client.authenticated_request(
                        "GET",
                        &format!("/api/test/mixed/auth/{}", auth_requests),
                        None,
                    ).await;

                    if result.is_ok() && result.unwrap().status().is_success() {
                        auth_successes += 1;
                    }
                } else {
                    // Unauthenticated request (should work for public endpoints)
                    unauth_requests += 1;
                    let result = unauth_client.authenticated_request(
                        "GET",
                        &format!("/api/system/status"),
                        None,
                    ).await;

                    if result.is_ok() && result.unwrap().status().is_success() {
                        unauth_successes += 1;
                    }
                }

                sleep(Duration::from_millis(100)).await;
            }

            let auth_success_rate = if auth_requests > 0 { auth_successes as f64 / auth_requests as f64 } else { 0.0 };
            let unauth_success_rate = if unauth_requests > 0 { unauth_successes as f64 / unauth_requests as f64 } else { 0.0 };

            log::info!("✓ Mixed traffic test completed:");
            log::info!("  Authenticated: {}/{} successful ({:.1}%)", auth_successes, auth_requests, auth_success_rate * 100.0);
            log::info!("  Unauthenticated: {}/{} successful ({:.1}%)", unauth_successes, unauth_requests, unauth_success_rate * 100.0);

            // Record metrics
            self.results.add_metric("mixed_traffic_auth_success_rate", auth_success_rate);
            self.results.add_metric("mixed_traffic_unauth_success_rate", unauth_success_rate);

            if auth_success_rate < 0.9 {
                return Err(anyhow::anyhow!("Mixed traffic authenticated success rate too low"));
            }

            // Step 4: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("mixed_traffic_scenarios", true, None);
                log::info!("✅ Mixed traffic scenarios test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("mixed_traffic_scenarios", false, Some(e.to_string()));
                log::error!("❌ Mixed traffic scenarios test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("mixed_traffic_scenarios", false, Some("Test timeout".to_string()));
                log::error!("❌ Mixed traffic scenarios test timed out");
            }
        }

        Ok(())
    }

    /// Test resource exhaustion scenarios
    async fn test_resource_exhaustion_scenarios(&mut self) -> anyhow::Result<()> {
        log::info!("🧪 Testing resource exhaustion scenarios");

        let test_result = timeout(Duration::from_secs(self.config.test_timeout_secs), async {
            // This test is intentionally simplified to avoid actually exhausting system resources
            // In a real implementation, this would test memory limits, connection limits, etc.
            
            // Step 1: Start test server
            let mut server_harness = TestServerHarness::new(SecurityProfile::Standard).await?;
            server_harness.start().await?;

            // Step 2: Test with reasonable load that won't exhaust resources
            let credentials = TestCredentials::generate()?;
            let mut server_client = E2EHttpClient::new(server_harness.base_url().to_string());
            server_client.wait_for_server(20).await?;
            server_client.register_public_key(&credentials).await?;

            // Step 3: Simulate resource pressure (limited scope for safety)
            let pressure_test_size = 20;
            let mut handles = Vec::new();

            for i in 0..pressure_test_size {
                let client = E2EHttpClient::new(server_harness.base_url().to_string())
                    .with_credentials(credentials.clone());
                
                let handle = tokio::spawn(async move {
                    // Send a request with some payload
                    let payload = utils::test_json_payload(1024); // 1KB payload
                    
                    client.authenticated_request(
                        "POST",
                        &format!("/api/test/pressure/{}", i),
                        Some(payload),
                    ).await
                });

                handles.push(handle);
            }

            let mut successful_pressure_requests = 0;
            for handle in handles {
                match handle.await {
                    Ok(Ok(response)) if response.status().is_success() => {
                        successful_pressure_requests += 1;
                    }
                    _ => {}
                }
            }

            let pressure_success_rate = successful_pressure_requests as f64 / pressure_test_size as f64;
            log::info!("✓ Resource pressure test: {}/{} successful ({:.1}% success)", 
                      successful_pressure_requests, pressure_test_size, pressure_success_rate * 100.0);

            self.results.add_metric("resource_pressure_success_rate", pressure_success_rate);

            if pressure_success_rate < 0.8 {
                return Err(anyhow::anyhow!("Resource pressure test success rate too low"));
            }

            // Step 4: Cleanup
            server_harness.stop().await?;

            Ok::<(), anyhow::Error>(())
        }).await;

        match test_result {
            Ok(Ok(())) => {
                self.results.add_result("resource_exhaustion_scenarios", true, None);
                log::info!("✅ Resource exhaustion scenarios test passed");
            }
            Ok(Err(e)) => {
                self.results.add_result("resource_exhaustion_scenarios", false, Some(e.to_string()));
                log::error!("❌ Resource exhaustion scenarios test failed: {}", e);
            }
            Err(_) => {
                self.results.add_result("resource_exhaustion_scenarios", false, Some("Test timeout".to_string()));
                log::error!("❌ Resource exhaustion scenarios test timed out");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::e2e::init_e2e_environment;

    #[tokio::test]
    async fn test_real_world_scenarios_suite() {
        init_e2e_environment();
        
        let config = E2ETestConfig {
            server_url: "http://localhost:8080".to_string(),
            test_timeout_secs: 30,
            concurrent_clients: 5,
            enable_attack_simulation: false,
            temp_dir: std::env::temp_dir(),
        };

        let mut real_world_tests = RealWorldScenarios::new(config);
        
        // For now, just test that the structure works
        assert_eq!(real_world_tests.results.total_tests, 0);
    }
}