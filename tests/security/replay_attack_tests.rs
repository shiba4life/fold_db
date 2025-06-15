//! Comprehensive Replay Attack Validation Test Suite
//!
//! This module implements Task 11-7-2: Validate replay attack prevention
//!
//! Tests all replay attack prevention mechanisms including:
//! - Immediate replay attempts
//! - Delayed replay attacks  
//! - Timestamp manipulation attacks
//! - Nonce collision and prediction attacks
//! - High-frequency replay flooding
//! - Clock skew and synchronization attacks
//!
//! Validates across all security profiles (Strict, Standard, Lenient)

use datafold::datafold_node::signature_auth::{
    SecurityProfile, SignatureAuthConfig, SignatureVerificationState,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use uuid::Uuid;

/// Replay attack simulation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayAttackResult {
    pub attack_type: String,
    pub security_profile: SecurityProfile,
    pub total_attempts: usize,
    pub successful_attacks: usize,
    pub blocked_attempts: usize,
    pub false_positives: usize,
    pub detection_accuracy: f64,
    pub average_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub min_response_time_ms: u64,
    pub memory_impact_bytes: usize,
    pub attack_duration_ms: u64,
}

/// Attack scenario configuration
#[derive(Debug, Clone)]
pub struct AttackScenario {
    pub name: String,
    pub description: String,
    pub security_profile: SecurityProfile,
    pub attack_duration_secs: u64,
    pub attack_frequency_ms: u64,
    pub concurrent_attackers: usize,
    pub use_valid_signatures: bool,
    pub timestamp_manipulation: TimestampManipulation,
    pub nonce_strategy: NonceStrategy,
}

/// Timestamp manipulation strategies
#[derive(Debug, Clone)]
pub enum TimestampManipulation {
    None,
    PastTimestamp { seconds_ago: u64 },
    FutureTimestamp { seconds_ahead: u64 },
    RandomWithinWindow,
    SequentialReplay,
}

/// Nonce attack strategies
#[derive(Debug, Clone)]
pub enum NonceStrategy {
    ExactReplay,
    PredictableSequence,
    RandomCollision,
    ValidUUID4,
    InvalidFormat,
}

/// Performance metrics under attack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPerformanceMetrics {
    pub baseline_response_time_ms: u64,
    pub under_attack_response_time_ms: u64,
    pub performance_degradation_percent: f64,
    pub memory_usage_increase_bytes: usize,
    pub cpu_overhead_percent: f64,
    pub throughput_reduction_percent: f64,
}

/// Cross-platform validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformValidation {
    pub rust_server_results: ReplayAttackResult,
    pub javascript_client_consistency: bool,
    pub python_client_consistency: bool,
    pub cli_client_consistency: bool,
    pub time_sync_variance_ms: u64,
}

/// Main replay attack test runner
pub struct ReplayAttackTestRunner {
    states: HashMap<SecurityProfile, SignatureVerificationState>,
    scenarios: Vec<AttackScenario>,
    results: Vec<ReplayAttackResult>,
}

impl ReplayAttackTestRunner {
    /// Create new test runner with all security profiles
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut states = HashMap::new();

        // Initialize states for all security profiles
        states.insert(
            SecurityProfile::Strict,
            SignatureVerificationState::new(SignatureAuthConfig::strict())?,
        );
        states.insert(
            SecurityProfile::Standard,
            SignatureVerificationState::new(SignatureAuthConfig::default())?,
        );
        states.insert(
            SecurityProfile::Lenient,
            SignatureVerificationState::new(SignatureAuthConfig::lenient())?,
        );

        let scenarios = Self::create_attack_scenarios();

        Ok(Self {
            states,
            scenarios,
            results: Vec::new(),
        })
    }

    /// Create comprehensive attack scenarios
    fn create_attack_scenarios() -> Vec<AttackScenario> {
        vec![
            // Immediate replay attacks
            AttackScenario {
                name: "immediate_replay".to_string(),
                description: "Immediate replay of valid requests".to_string(),
                security_profile: SecurityProfile::Strict,
                attack_duration_secs: 30,
                attack_frequency_ms: 100,
                concurrent_attackers: 1,
                use_valid_signatures: true,
                timestamp_manipulation: TimestampManipulation::None,
                nonce_strategy: NonceStrategy::ExactReplay,
            },
            // Delayed replay attacks
            AttackScenario {
                name: "delayed_replay".to_string(),
                description: "Replay attacks with time delays".to_string(),
                security_profile: SecurityProfile::Standard,
                attack_duration_secs: 60,
                attack_frequency_ms: 1000,
                concurrent_attackers: 3,
                use_valid_signatures: true,
                timestamp_manipulation: TimestampManipulation::SequentialReplay,
                nonce_strategy: NonceStrategy::ExactReplay,
            },
            // Timestamp manipulation attacks
            AttackScenario {
                name: "timestamp_manipulation".to_string(),
                description: "Attacks using manipulated timestamps".to_string(),
                security_profile: SecurityProfile::Strict,
                attack_duration_secs: 45,
                attack_frequency_ms: 200,
                concurrent_attackers: 2,
                use_valid_signatures: true,
                timestamp_manipulation: TimestampManipulation::PastTimestamp { seconds_ago: 300 },
                nonce_strategy: NonceStrategy::ValidUUID4,
            },
            // Future timestamp attacks
            AttackScenario {
                name: "future_timestamp_attack".to_string(),
                description: "Attacks using future timestamps".to_string(),
                security_profile: SecurityProfile::Standard,
                attack_duration_secs: 30,
                attack_frequency_ms: 150,
                concurrent_attackers: 2,
                use_valid_signatures: true,
                timestamp_manipulation: TimestampManipulation::FutureTimestamp {
                    seconds_ahead: 120,
                },
                nonce_strategy: NonceStrategy::ValidUUID4,
            },
            // Nonce prediction attacks
            AttackScenario {
                name: "nonce_prediction".to_string(),
                description: "Attacks using predictable nonce sequences".to_string(),
                security_profile: SecurityProfile::Lenient,
                attack_duration_secs: 60,
                attack_frequency_ms: 300,
                concurrent_attackers: 1,
                use_valid_signatures: false,
                timestamp_manipulation: TimestampManipulation::None,
                nonce_strategy: NonceStrategy::PredictableSequence,
            },
            // High-frequency flooding
            AttackScenario {
                name: "high_frequency_flood".to_string(),
                description: "High-frequency replay flooding attack".to_string(),
                security_profile: SecurityProfile::Strict,
                attack_duration_secs: 20,
                attack_frequency_ms: 10, // Very high frequency
                concurrent_attackers: 5,
                use_valid_signatures: true,
                timestamp_manipulation: TimestampManipulation::RandomWithinWindow,
                nonce_strategy: NonceStrategy::ExactReplay,
            },
            // Nonce collision attacks
            AttackScenario {
                name: "nonce_collision".to_string(),
                description: "Attempts to create nonce collisions".to_string(),
                security_profile: SecurityProfile::Standard,
                attack_duration_secs: 40,
                attack_frequency_ms: 500,
                concurrent_attackers: 3,
                use_valid_signatures: false,
                timestamp_manipulation: TimestampManipulation::None,
                nonce_strategy: NonceStrategy::RandomCollision,
            },
        ]
    }

    /// Run all attack scenarios
    pub async fn run_all_scenarios(
        &mut self,
    ) -> Result<Vec<ReplayAttackResult>, Box<dyn std::error::Error>> {
        println!("🚀 Starting comprehensive replay attack validation");

        for scenario in &self.scenarios.clone() {
            println!("🔍 Running attack scenario: {}", scenario.name);
            let result = self.run_attack_scenario(scenario).await?;

            println!("📊 Scenario '{}' results:", scenario.name);
            println!("  - Total attempts: {}", result.total_attempts);
            println!("  - Blocked: {}", result.blocked_attempts);
            println!(
                "  - Detection accuracy: {:.2}%",
                result.detection_accuracy * 100.0
            );
            println!(
                "  - Avg response time: {}ms",
                result.average_response_time_ms
            );

            self.results.push(result);
        }

        Ok(self.results.clone())
    }

    /// Run a specific attack scenario
    async fn run_attack_scenario(
        &self,
        scenario: &AttackScenario,
    ) -> Result<ReplayAttackResult, Box<dyn std::error::Error>> {
        let state = self
            .states
            .get(&scenario.security_profile)
            .ok_or("Security profile state not found")?;

        let start_time = Instant::now();
        let mut total_attempts = 0;
        let mut blocked_attempts = 0;
        let mut successful_attacks = 0;
        let mut response_times = Vec::new();

        // Get baseline nonce store stats
        let initial_stats = state.get_nonce_store_stats()?;

        // Generate base nonce and timestamp for replay attacks
        let base_nonce = self.generate_nonce(&scenario.nonce_strategy);
        let base_timestamp = self.generate_timestamp(&scenario.timestamp_manipulation);

        // First, establish the base nonce (for replay scenarios)
        if scenario.nonce_strategy == NonceStrategy::ExactReplay {
            let _ = state.check_and_store_nonce(&base_nonce, base_timestamp);
        }

        // Run attack simulation
        let attack_start = Instant::now();
        while attack_start.elapsed().as_secs() < scenario.attack_duration_secs {
            for _attacker in 0..scenario.concurrent_attackers {
                let attack_start_time = Instant::now();

                let nonce = if scenario.nonce_strategy == NonceStrategy::ExactReplay {
                    base_nonce.clone()
                } else {
                    self.generate_nonce(&scenario.nonce_strategy)
                };

                let timestamp = self.generate_timestamp(&scenario.timestamp_manipulation);

                // Attempt replay attack
                let result = state.check_and_store_nonce(&nonce, timestamp);
                let response_time = attack_start_time.elapsed().as_millis() as u64;
                response_times.push(response_time);

                total_attempts += 1;

                match result {
                    Ok(_) => {
                        // This should only happen for the first valid request
                        if total_attempts > 1
                            && scenario.nonce_strategy == NonceStrategy::ExactReplay
                        {
                            successful_attacks += 1; // Security failure!
                        }
                    }
                    Err(_) => {
                        blocked_attempts += 1;
                    }
                }
            }

            // Wait before next attack wave
            sleep(Duration::from_millis(scenario.attack_frequency_ms)).await;
        }

        // Get final nonce store stats for memory impact analysis
        let final_stats = state.get_nonce_store_stats()?;
        let memory_impact =
            (final_stats.total_nonces - initial_stats.total_nonces) * std::mem::size_of::<String>(); // Rough estimate

        // Calculate metrics
        let detection_accuracy = if total_attempts > 0 {
            blocked_attempts as f64 / total_attempts as f64
        } else {
            0.0
        };

        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<u64>() / response_times.len() as u64
        } else {
            0
        };

        let max_response_time = response_times.iter().max().copied().unwrap_or(0);
        let min_response_time = response_times.iter().min().copied().unwrap_or(0);

        Ok(ReplayAttackResult {
            attack_type: scenario.name.clone(),
            security_profile: scenario.security_profile.clone(),
            total_attempts,
            successful_attacks,
            blocked_attempts,
            false_positives: 0, // Would need legitimate requests to calculate
            detection_accuracy,
            average_response_time_ms: avg_response_time,
            max_response_time_ms: max_response_time,
            min_response_time_ms: min_response_time,
            memory_impact_bytes: memory_impact,
            attack_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Generate nonce based on strategy
    fn generate_nonce(&self, strategy: &NonceStrategy) -> String {
        match strategy {
            NonceStrategy::ExactReplay => "replay-test-nonce".to_string(),
            NonceStrategy::ValidUUID4 => Uuid::new_v4().to_string(),
            NonceStrategy::PredictableSequence => {
                format!(
                    "predictable-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                )
            }
            NonceStrategy::RandomCollision => {
                // Attempt to create collision (very unlikely with proper UUIDs)
                "00000000-0000-0000-0000-000000000001".to_string()
            }
            NonceStrategy::InvalidFormat => "invalid-nonce-format!@#".to_string(),
        }
    }

    /// Generate timestamp based on manipulation strategy
    fn generate_timestamp(&self, manipulation: &TimestampManipulation) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match manipulation {
            TimestampManipulation::None => now,
            TimestampManipulation::PastTimestamp { seconds_ago } => now - seconds_ago,
            TimestampManipulation::FutureTimestamp { seconds_ahead } => now + seconds_ahead,
            TimestampManipulation::RandomWithinWindow => {
                // Random timestamp within recent window
                now - (rand::random::<u64>() % 300) // Last 5 minutes
            }
            TimestampManipulation::SequentialReplay => now - 1, // Just in the past
        }
    }

    /// Generate comprehensive validation report
    pub fn generate_validation_report(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        for result in &self.results {
            report.add_result(result.clone());
        }

        report.calculate_summary();
        report
    }
}

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub timestamp: u64,
    pub total_scenarios: usize,
    pub total_attacks_attempted: usize,
    pub total_attacks_blocked: usize,
    pub overall_detection_accuracy: f64,
    pub security_profile_results: HashMap<String, SecurityProfileSummary>,
    pub performance_impact: AttackPerformanceMetrics,
    pub recommendations: Vec<String>,
    pub scenario_results: Vec<ReplayAttackResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProfileSummary {
    pub profile: SecurityProfile,
    pub scenarios_tested: usize,
    pub average_detection_accuracy: f64,
    pub average_response_time_ms: u64,
    pub total_memory_impact_bytes: usize,
    pub effectiveness_rating: String, // "Excellent", "Good", "Needs Improvement"
}

impl ValidationReport {
    fn new() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_scenarios: 0,
            total_attacks_attempted: 0,
            total_attacks_blocked: 0,
            overall_detection_accuracy: 0.0,
            security_profile_results: HashMap::new(),
            performance_impact: AttackPerformanceMetrics {
                baseline_response_time_ms: 0,
                under_attack_response_time_ms: 0,
                performance_degradation_percent: 0.0,
                memory_usage_increase_bytes: 0,
                cpu_overhead_percent: 0.0,
                throughput_reduction_percent: 0.0,
            },
            recommendations: Vec::new(),
            scenario_results: Vec::new(),
        }
    }

    fn add_result(&mut self, result: ReplayAttackResult) {
        self.total_scenarios += 1;
        self.total_attacks_attempted += result.total_attempts;
        self.total_attacks_blocked += result.blocked_attempts;
        self.scenario_results.push(result);
    }

    fn calculate_summary(&mut self) {
        if self.total_attacks_attempted > 0 {
            self.overall_detection_accuracy =
                self.total_attacks_blocked as f64 / self.total_attacks_attempted as f64;
        }

        self.generate_recommendations();
    }

    fn generate_recommendations(&mut self) {
        if self.overall_detection_accuracy < 0.95 {
            self.recommendations.push(
                "Consider using stricter security profile for better replay detection".to_string(),
            );
        }

        if self
            .scenario_results
            .iter()
            .any(|r| r.average_response_time_ms > 500)
        {
            self.recommendations.push(
                "Performance degradation detected under attack - consider optimization".to_string(),
            );
        }

        if self
            .scenario_results
            .iter()
            .any(|r| r.memory_impact_bytes > 1024 * 1024)
        {
            self.recommendations.push(
                "High memory usage detected - consider nonce store cleanup optimization"
                    .to_string(),
            );
        }
    }
}

// Implementation of PartialEq for NonceStrategy to enable comparison
impl PartialEq for NonceStrategy {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Test suite implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_immediate_replay_attack_detection() {
        let config = SignatureAuthConfig::strict();
        let state = SignatureVerificationState::new(config).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = Uuid::new_v4().to_string();

        // First request should succeed
        assert!(state.check_and_store_nonce(&nonce, now).is_ok());

        // Immediate replay should fail
        assert!(state.check_and_store_nonce(&nonce, now).is_err());

        // Same nonce with different timestamp should also fail
        assert!(state.check_and_store_nonce(&nonce, now + 10).is_err());
    }

    #[tokio::test]
    async fn test_timestamp_manipulation_detection() {
        let config = SignatureAuthConfig::strict();
        let state = SignatureVerificationState::new(config).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Future timestamp beyond tolerance should fail
        let future_timestamp = now + 200; // Way beyond strict tolerance
        let nonce = Uuid::new_v4().to_string();
        assert!(state
            .check_and_store_nonce(&nonce, future_timestamp)
            .is_err());

        // Past timestamp beyond window should fail
        let past_timestamp = now - 200; // Beyond strict window
        let nonce2 = Uuid::new_v4().to_string();
        assert!(state
            .check_and_store_nonce(&nonce2, past_timestamp)
            .is_err());
    }

    #[tokio::test]
    async fn test_nonce_format_validation() {
        let config = SignatureAuthConfig::strict();
        let state = SignatureVerificationState::new(config).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Valid UUID4 should work
        let valid_nonce = Uuid::new_v4().to_string();
        assert!(state.check_and_store_nonce(&valid_nonce, now).is_ok());

        // Invalid format should fail
        let invalid_nonce = "not-a-valid-uuid";
        assert!(state.check_and_store_nonce(invalid_nonce, now).is_err());
    }

    #[tokio::test]
    async fn test_high_frequency_attack_handling() {
        let config = SignatureAuthConfig::strict();
        let state = SignatureVerificationState::new(config).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let base_nonce = Uuid::new_v4().to_string();

        // First request succeeds
        assert!(state.check_and_store_nonce(&base_nonce, now).is_ok());

        // Rapid replay attempts should all fail
        for i in 1..100 {
            let result = state.check_and_store_nonce(&base_nonce, now + i);
            assert!(result.is_err(), "Replay attempt {} should have failed", i);
        }
    }

    #[tokio::test]
    async fn test_security_profile_differences() {
        let strict_config = SignatureAuthConfig::strict();
        let lenient_config = SignatureAuthConfig::lenient();

        let strict_state = SignatureVerificationState::new(strict_config).unwrap();
        let lenient_state = SignatureVerificationState::new(lenient_config).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Test timestamp tolerance differences
        let old_timestamp = now - 400; // 400 seconds ago
        let nonce1 = Uuid::new_v4().to_string();
        let nonce2 = Uuid::new_v4().to_string();

        // Should fail in strict mode (60s + 5s = 65s window)
        assert!(strict_state
            .check_and_store_nonce(&nonce1, old_timestamp)
            .is_err());

        // Should succeed in lenient mode (600s + 120s = 720s window)
        assert!(lenient_state
            .check_and_store_nonce(&nonce2, old_timestamp)
            .is_ok());
    }

    #[tokio::test]
    async fn test_attack_scenario_runner() {
        let runner = ReplayAttackTestRunner::new().unwrap();

        // Create a simple test scenario
        let test_scenario = AttackScenario {
            name: "test_immediate_replay".to_string(),
            description: "Test immediate replay detection".to_string(),
            security_profile: SecurityProfile::Strict,
            attack_duration_secs: 1, // Very short for test
            attack_frequency_ms: 100,
            concurrent_attackers: 1,
            use_valid_signatures: true,
            timestamp_manipulation: TimestampManipulation::None,
            nonce_strategy: NonceStrategy::ExactReplay,
        };

        let result = runner.run_attack_scenario(&test_scenario).await.unwrap();

        // Should have blocked most attempts (after the first valid one)
        assert!(result.total_attempts > 0);
        assert!(result.detection_accuracy > 0.8); // Should block most replays
        assert_eq!(result.security_profile, SecurityProfile::Strict);
    }

    #[tokio::test]
    async fn test_validation_report_generation() {
        let mut runner = ReplayAttackTestRunner::new().unwrap();

        // Add a sample result
        let sample_result = ReplayAttackResult {
            attack_type: "test_attack".to_string(),
            security_profile: SecurityProfile::Standard,
            total_attempts: 100,
            successful_attacks: 2,
            blocked_attempts: 98,
            false_positives: 0,
            detection_accuracy: 0.98,
            average_response_time_ms: 50,
            max_response_time_ms: 100,
            min_response_time_ms: 10,
            memory_impact_bytes: 1024,
            attack_duration_ms: 5000,
        };

        runner.results.push(sample_result);
        let report = runner.generate_validation_report();

        assert_eq!(report.total_scenarios, 1);
        assert_eq!(report.total_attacks_attempted, 100);
        assert_eq!(report.total_attacks_blocked, 98);
        assert!((report.overall_detection_accuracy - 0.98).abs() < 0.01);
    }
}
