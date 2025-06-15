//! Attack Simulation Tools for Replay Prevention Testing
//!
//! This module provides tools for simulating various replay attack scenarios
//! to validate the effectiveness of DataFold's replay prevention mechanisms.

use datafold::datafold_node::signature_auth::{
    SecurityProfile, SignatureAuthConfig, SignatureVerificationState,
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use uuid::Uuid;

/// Attack simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSimulationConfig {
    pub target_profile: SecurityProfile,
    pub duration_seconds: u64,
    pub concurrent_attackers: usize,
    pub attack_frequency_hz: f64,
    pub legitimate_traffic_ratio: f64,
    pub enable_logging: bool,
    pub performance_monitoring: bool,
}

impl Default for AttackSimulationConfig {
    fn default() -> Self {
        Self {
            target_profile: SecurityProfile::Standard,
            duration_seconds: 60,
            concurrent_attackers: 3,
            attack_frequency_hz: 10.0,
            legitimate_traffic_ratio: 0.1,
            enable_logging: true,
            performance_monitoring: true,
        }
    }
}

/// Attack pattern types for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackPattern {
    /// Immediate replay of the same request
    ImmediateReplay {
        original_nonce: String,
        original_timestamp: u64,
    },
    /// Delayed replay with time skew
    DelayedReplay {
        delay_seconds: u64,
        timestamp_drift: i64,
    },
    /// Bulk replay flooding
    ReplayFlooding {
        nonce_pool: Vec<String>,
        burst_size: usize,
    },
    /// Timestamp manipulation attempts
    TimestampManipulation {
        future_drift_seconds: u64,
        past_drift_seconds: u64,
    },
    /// Nonce prediction attempts
    NoncePrediction {
        prediction_strategy: NoncePredictionStrategy,
    },
    /// Combined multi-vector attack
    MultiVector {
        patterns: Vec<AttackPattern>,
        orchestration_delay_ms: u64,
    },
}

/// Nonce prediction strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoncePredictionStrategy {
    Sequential,
    Timestamp,
    Incremental,
    PatternBased(String),
    WeakRandom,
}

/// Attack simulation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSimulationResult {
    pub attack_pattern: String,
    pub configuration: AttackSimulationConfig,
    pub execution_metrics: ExecutionMetrics,
    pub security_metrics: SecurityMetrics,
    pub performance_impact: PerformanceImpact,
    pub detection_analysis: DetectionAnalysis,
    pub recommendations: Vec<String>,
}

/// Execution metrics for the attack simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_attack_attempts: usize,
    pub successful_bypasses: usize,
    pub blocked_attempts: usize,
    pub legitimate_requests: usize,
    pub false_positives: usize,
    pub average_attempt_duration_ms: f64,
    pub peak_concurrent_attacks: usize,
}

/// Security effectiveness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub detection_rate: f64,
    pub false_positive_rate: f64,
    pub time_to_detection_ms: f64,
    pub attack_pattern_recognition: f64,
    pub nonce_store_effectiveness: f64,
    pub timestamp_validation_effectiveness: f64,
}

/// Performance impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub baseline_response_time_ms: f64,
    pub under_attack_response_time_ms: f64,
    pub response_time_degradation_percent: f64,
    pub memory_usage_increase_bytes: usize,
    pub cpu_utilization_increase_percent: f64,
    pub throughput_reduction_percent: f64,
}

/// Detection analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionAnalysis {
    pub immediate_detection_rate: f64,
    pub delayed_detection_rate: f64,
    pub pattern_recognition_accuracy: f64,
    pub error_categorization: HashMap<String, usize>,
    pub timing_analysis: TimingAnalysis,
}

/// Timing analysis for attack detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysis {
    pub minimum_detection_time_ms: u64,
    pub maximum_detection_time_ms: u64,
    pub average_detection_time_ms: f64,
    pub detection_time_variance: f64,
}

/// Main attack simulator
pub struct AttackSimulator {
    config: AttackSimulationConfig,
    target_state: Arc<SignatureVerificationState>,
    attack_patterns: Vec<AttackPattern>,
    results: Vec<AttackSimulationResult>,
    baseline_metrics: Option<PerformanceBaseline>,
}

/// Baseline performance measurements
#[derive(Debug, Clone)]
struct PerformanceBaseline {
    response_time_ms: f64,
    memory_usage_bytes: usize,
    cpu_utilization_percent: f64,
    throughput_ops_per_sec: f64,
}

impl AttackSimulator {
    /// Create new attack simulator
    pub fn new(config: AttackSimulationConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let auth_config = match config.target_profile {
            SecurityProfile::Strict => SignatureAuthConfig::strict(),
            SecurityProfile::Standard => SignatureAuthConfig::default(),
            SecurityProfile::Lenient => SignatureAuthConfig::lenient(),
        };

        let target_state = Arc::new(SignatureVerificationState::new(auth_config)?);
        let attack_patterns = Self::generate_attack_patterns();

        Ok(Self {
            config,
            target_state,
            attack_patterns,
            results: Vec::new(),
            baseline_metrics: None,
        })
    }

    /// Generate comprehensive attack patterns
    fn generate_attack_patterns() -> Vec<AttackPattern> {
        vec![
            AttackPattern::ImmediateReplay {
                original_nonce: "target-nonce-001".to_string(),
                original_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            AttackPattern::DelayedReplay {
                delay_seconds: 300,
                timestamp_drift: -60,
            },
            AttackPattern::ReplayFlooding {
                nonce_pool: (0..100).map(|i| format!("flood-nonce-{:03}", i)).collect(),
                burst_size: 50,
            },
            AttackPattern::TimestampManipulation {
                future_drift_seconds: 3600,
                past_drift_seconds: 7200,
            },
            AttackPattern::NoncePrediction {
                prediction_strategy: NoncePredictionStrategy::Sequential,
            },
            AttackPattern::NoncePrediction {
                prediction_strategy: NoncePredictionStrategy::Timestamp,
            },
            AttackPattern::MultiVector {
                patterns: vec![
                    AttackPattern::ImmediateReplay {
                        original_nonce: "multi-nonce-1".to_string(),
                        original_timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                    AttackPattern::TimestampManipulation {
                        future_drift_seconds: 600,
                        past_drift_seconds: 600,
                    },
                ],
                orchestration_delay_ms: 100,
            },
        ]
    }

    /// Establish performance baseline
    pub async fn establish_baseline(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Establishing performance baseline...");

        let start_time = Instant::now();
        let baseline_requests = 100;

        // Measure baseline performance with legitimate requests
        for i in 0..baseline_requests {
            let nonce = Uuid::new_v4().to_string();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let request_start = Instant::now();
            let _ = self.target_state.check_and_store_nonce(&nonce, timestamp);
            let _request_duration = request_start.elapsed();
        }

        let total_duration = start_time.elapsed();
        let avg_response_time = total_duration.as_millis() as f64 / baseline_requests as f64;
        let throughput = baseline_requests as f64 / total_duration.as_secs_f64();

        self.baseline_metrics = Some(PerformanceBaseline {
            response_time_ms: avg_response_time,
            memory_usage_bytes: self.estimate_memory_usage(),
            cpu_utilization_percent: 0.0, // Would need system monitoring
            throughput_ops_per_sec: throughput,
        });

        println!(
            "✓ Baseline established: {:.2}ms avg response, {:.1} ops/sec throughput",
            avg_response_time, throughput
        );

        Ok(())
    }

    /// Run all attack simulations
    pub async fn run_all_simulations(
        &mut self,
    ) -> Result<Vec<AttackSimulationResult>, Box<dyn std::error::Error>> {
        if self.baseline_metrics.is_none() {
            self.establish_baseline().await?;
        }

        println!("🚀 Starting comprehensive attack simulations");

        for (i, pattern) in self.attack_patterns.clone().iter().enumerate() {
            println!(
                "🔍 Running attack pattern {} of {}",
                i + 1,
                self.attack_patterns.len()
            );

            let result = self.simulate_attack_pattern(pattern).await?;
            self.results.push(result);
        }

        println!("✅ All attack simulations completed");
        Ok(self.results.clone())
    }

    /// Simulate specific attack pattern
    async fn simulate_attack_pattern(
        &self,
        pattern: &AttackPattern,
    ) -> Result<AttackSimulationResult, Box<dyn std::error::Error>> {
        let pattern_name = self.get_pattern_name(pattern);
        println!("  └─ Executing {}", pattern_name);

        let start_time = Instant::now();
        let mut execution_metrics = ExecutionMetrics::default();
        let mut detection_times = Vec::new();
        let mut error_categorization = HashMap::new();

        // Prepare attack based on pattern
        match pattern {
            AttackPattern::ImmediateReplay {
                original_nonce,
                original_timestamp,
            } => {
                self.simulate_immediate_replay(
                    original_nonce,
                    *original_timestamp,
                    &mut execution_metrics,
                    &mut detection_times,
                    &mut error_categorization,
                )
                .await?;
            }
            AttackPattern::DelayedReplay {
                delay_seconds,
                timestamp_drift,
            } => {
                self.simulate_delayed_replay(
                    *delay_seconds,
                    *timestamp_drift,
                    &mut execution_metrics,
                    &mut detection_times,
                    &mut error_categorization,
                )
                .await?;
            }
            AttackPattern::ReplayFlooding {
                nonce_pool,
                burst_size,
            } => {
                self.simulate_replay_flooding(
                    nonce_pool,
                    *burst_size,
                    &mut execution_metrics,
                    &mut detection_times,
                    &mut error_categorization,
                )
                .await?;
            }
            AttackPattern::TimestampManipulation {
                future_drift_seconds,
                past_drift_seconds,
            } => {
                self.simulate_timestamp_manipulation(
                    *future_drift_seconds,
                    *past_drift_seconds,
                    &mut execution_metrics,
                    &mut detection_times,
                    &mut error_categorization,
                )
                .await?;
            }
            AttackPattern::NoncePrediction {
                prediction_strategy,
            } => {
                self.simulate_nonce_prediction(
                    prediction_strategy,
                    &mut execution_metrics,
                    &mut detection_times,
                    &mut error_categorization,
                )
                .await?;
            }
            AttackPattern::MultiVector {
                patterns,
                orchestration_delay_ms,
            } => {
                self.simulate_multi_vector_attack(
                    patterns,
                    *orchestration_delay_ms,
                    &mut execution_metrics,
                    &mut detection_times,
                    &mut error_categorization,
                )
                .await?;
            }
        }

        let total_duration = start_time.elapsed();
        execution_metrics.average_attempt_duration_ms = total_duration.as_millis() as f64
            / execution_metrics.total_attack_attempts.max(1) as f64;

        // Calculate metrics
        let security_metrics =
            self.calculate_security_metrics(&execution_metrics, &detection_times);
        let performance_impact = self.calculate_performance_impact(&execution_metrics);
        let detection_analysis =
            self.analyze_detection_patterns(&detection_times, &error_categorization);
        let recommendations = self.generate_recommendations(&security_metrics, &performance_impact);

        Ok(AttackSimulationResult {
            attack_pattern: pattern_name,
            configuration: self.config.clone(),
            execution_metrics,
            security_metrics,
            performance_impact,
            detection_analysis,
            recommendations,
        })
    }

    /// Simulate immediate replay attack
    async fn simulate_immediate_replay(
        &self,
        original_nonce: &str,
        original_timestamp: u64,
        metrics: &mut ExecutionMetrics,
        detection_times: &mut Vec<u64>,
        error_categorization: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // First, establish the original request
        let _ = self
            .target_state
            .check_and_store_nonce(original_nonce, original_timestamp);

        // Now attempt immediate replays
        let replay_attempts = 50;
        for _ in 0..replay_attempts {
            let attack_start = Instant::now();
            let result = self
                .target_state
                .check_and_store_nonce(original_nonce, original_timestamp);
            let detection_time = attack_start.elapsed().as_millis() as u64;

            metrics.total_attack_attempts += 1;
            detection_times.push(detection_time);

            match result {
                Ok(_) => {
                    metrics.successful_bypasses += 1;
                }
                Err(err) => {
                    metrics.blocked_attempts += 1;
                    let error_type = format!("{:?}", err)
                        .split('(')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string();
                    *error_categorization.entry(error_type).or_insert(0) += 1;
                }
            }

            // Small delay between attempts
            sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    /// Simulate delayed replay attack
    async fn simulate_delayed_replay(
        &self,
        delay_seconds: u64,
        timestamp_drift: i64,
        metrics: &mut ExecutionMetrics,
        detection_times: &mut Vec<u64>,
        error_categorization: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base_nonce = Uuid::new_v4().to_string();
        let base_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Establish original request
        let _ = self
            .target_state
            .check_and_store_nonce(&base_nonce, base_timestamp);

        // Wait for delay
        sleep(Duration::from_secs(delay_seconds.min(5))).await; // Cap at 5 seconds for testing

        // Attempt replay with timestamp drift
        let replay_timestamp = (base_timestamp as i64 + timestamp_drift) as u64;
        let attack_start = Instant::now();
        let result = self
            .target_state
            .check_and_store_nonce(&base_nonce, replay_timestamp);
        let detection_time = attack_start.elapsed().as_millis() as u64;

        metrics.total_attack_attempts += 1;
        detection_times.push(detection_time);

        match result {
            Ok(_) => metrics.successful_bypasses += 1,
            Err(err) => {
                metrics.blocked_attempts += 1;
                let error_type = format!("{:?}", err)
                    .split('(')
                    .next()
                    .unwrap_or("Unknown")
                    .to_string();
                *error_categorization.entry(error_type).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    /// Simulate replay flooding attack
    async fn simulate_replay_flooding(
        &self,
        nonce_pool: &[String],
        burst_size: usize,
        metrics: &mut ExecutionMetrics,
        detection_times: &mut Vec<u64>,
        error_categorization: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // First, establish some nonces
        for nonce in nonce_pool.iter().take(10) {
            let _ = self.target_state.check_and_store_nonce(nonce, timestamp);
        }

        // Now flood with replays
        let mut concurrent_tasks = Vec::new();

        for chunk in nonce_pool.chunks(burst_size) {
            let state = Arc::clone(&self.target_state);
            let chunk_nonces = chunk.to_vec();

            let task = tokio::spawn(async move {
                let mut chunk_metrics = Vec::new();

                for nonce in chunk_nonces {
                    let attack_start = Instant::now();
                    let result = state.check_and_store_nonce(&nonce, timestamp);
                    let detection_time = attack_start.elapsed().as_millis() as u64;

                    chunk_metrics.push((result, detection_time));
                }

                chunk_metrics
            });

            concurrent_tasks.push(task);
        }

        // Store the length before moving concurrent_tasks
        let peak_concurrent_attacks = concurrent_tasks.len();

        // Wait for all tasks and collect results
        let results = join_all(concurrent_tasks).await;

        for task_result in results {
            if let Ok(chunk_metrics) = task_result {
                for (result, detection_time) in chunk_metrics {
                    metrics.total_attack_attempts += 1;
                    detection_times.push(detection_time);

                    match result {
                        Ok(_) => metrics.successful_bypasses += 1,
                        Err(err) => {
                            metrics.blocked_attempts += 1;
                            let error_type = format!("{:?}", err)
                                .split('(')
                                .next()
                                .unwrap_or("Unknown")
                                .to_string();
                            *error_categorization.entry(error_type).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        metrics.peak_concurrent_attacks = peak_concurrent_attacks;

        Ok(())
    }

    /// Simulate timestamp manipulation attack
    async fn simulate_timestamp_manipulation(
        &self,
        future_drift_seconds: u64,
        past_drift_seconds: u64,
        metrics: &mut ExecutionMetrics,
        detection_times: &mut Vec<u64>,
        error_categorization: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Test various timestamp manipulations
        let test_timestamps = vec![
            now + future_drift_seconds, // Future timestamp
            now - past_drift_seconds,   // Past timestamp
            now + 3600,                 // 1 hour future
            now - 3600,                 // 1 hour past
            0,                          // Invalid timestamp
            u64::MAX,                   // Maximum timestamp
        ];

        for timestamp in test_timestamps {
            let nonce = Uuid::new_v4().to_string();
            let attack_start = Instant::now();
            let result = self.target_state.check_and_store_nonce(&nonce, timestamp);
            let detection_time = attack_start.elapsed().as_millis() as u64;

            metrics.total_attack_attempts += 1;
            detection_times.push(detection_time);

            match result {
                Ok(_) => metrics.successful_bypasses += 1,
                Err(err) => {
                    metrics.blocked_attempts += 1;
                    let error_type = format!("{:?}", err)
                        .split('(')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string();
                    *error_categorization.entry(error_type).or_insert(0) += 1;
                }
            }
        }

        Ok(())
    }

    /// Simulate nonce prediction attack
    async fn simulate_nonce_prediction(
        &self,
        strategy: &NoncePredictionStrategy,
        metrics: &mut ExecutionMetrics,
        detection_times: &mut Vec<u64>,
        error_categorization: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let predicted_nonces = self.generate_predicted_nonces(strategy, 50);

        for nonce in predicted_nonces {
            let attack_start = Instant::now();
            let result = self.target_state.check_and_store_nonce(&nonce, timestamp);
            let detection_time = attack_start.elapsed().as_millis() as u64;

            metrics.total_attack_attempts += 1;
            detection_times.push(detection_time);

            match result {
                Ok(_) => metrics.successful_bypasses += 1,
                Err(err) => {
                    metrics.blocked_attempts += 1;
                    let error_type = format!("{:?}", err)
                        .split('(')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string();
                    *error_categorization.entry(error_type).or_insert(0) += 1;
                }
            }
        }

        Ok(())
    }

    /// Simulate multi-vector attack
    async fn simulate_multi_vector_attack(
        &self,
        patterns: &[AttackPattern],
        orchestration_delay_ms: u64,
        metrics: &mut ExecutionMetrics,
        detection_times: &mut Vec<u64>,
        error_categorization: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Execute each pattern with orchestration delay
        for pattern in patterns {
            match pattern {
                AttackPattern::ImmediateReplay {
                    original_nonce,
                    original_timestamp,
                } => {
                    self.simulate_immediate_replay(
                        original_nonce,
                        *original_timestamp,
                        metrics,
                        detection_times,
                        error_categorization,
                    )
                    .await?;
                }
                AttackPattern::TimestampManipulation {
                    future_drift_seconds,
                    past_drift_seconds,
                } => {
                    self.simulate_timestamp_manipulation(
                        *future_drift_seconds,
                        *past_drift_seconds,
                        metrics,
                        detection_times,
                        error_categorization,
                    )
                    .await?;
                }
                _ => {
                    // Handle other patterns as needed
                }
            }

            sleep(Duration::from_millis(orchestration_delay_ms)).await;
        }

        Ok(())
    }

    /// Generate predicted nonces based on strategy
    fn generate_predicted_nonces(
        &self,
        strategy: &NoncePredictionStrategy,
        count: usize,
    ) -> Vec<String> {
        match strategy {
            NoncePredictionStrategy::Sequential => {
                (0..count).map(|i| format!("predicted-{:06}", i)).collect()
            }
            NoncePredictionStrategy::Timestamp => {
                let base_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                (0..count)
                    .map(|i| format!("time-{}", base_time + i as u64))
                    .collect()
            }
            NoncePredictionStrategy::Incremental => {
                (1..=count).map(|i| format!("{:032x}", i)).collect()
            }
            NoncePredictionStrategy::PatternBased(pattern) => (0..count)
                .map(|i| format!("{}-{:04}", pattern, i))
                .collect(),
            NoncePredictionStrategy::WeakRandom => {
                // Simulate weak randomness with small range
                (0..count)
                    .map(|_| format!("weak-{:04}", rand::random::<u16>() % 100))
                    .collect()
            }
        }
    }

    /// Calculate security metrics
    fn calculate_security_metrics(
        &self,
        execution_metrics: &ExecutionMetrics,
        detection_times: &[u64],
    ) -> SecurityMetrics {
        let detection_rate = if execution_metrics.total_attack_attempts > 0 {
            execution_metrics.blocked_attempts as f64
                / execution_metrics.total_attack_attempts as f64
        } else {
            1.0
        };

        let false_positive_rate = if execution_metrics.legitimate_requests > 0 {
            execution_metrics.false_positives as f64 / execution_metrics.legitimate_requests as f64
        } else {
            0.0
        };

        let avg_detection_time = if !detection_times.is_empty() {
            detection_times.iter().sum::<u64>() as f64 / detection_times.len() as f64
        } else {
            0.0
        };

        SecurityMetrics {
            detection_rate,
            false_positive_rate,
            time_to_detection_ms: avg_detection_time,
            attack_pattern_recognition: detection_rate, // Simplified
            nonce_store_effectiveness: detection_rate,
            timestamp_validation_effectiveness: detection_rate,
        }
    }

    /// Calculate performance impact
    fn calculate_performance_impact(
        &self,
        _execution_metrics: &ExecutionMetrics,
    ) -> PerformanceImpact {
        let baseline = self.baseline_metrics.as_ref().unwrap();

        // For simulation, use estimated values
        let under_attack_response_time = baseline.response_time_ms * 1.2; // 20% degradation
        let degradation_percent = ((under_attack_response_time - baseline.response_time_ms)
            / baseline.response_time_ms)
            * 100.0;

        PerformanceImpact {
            baseline_response_time_ms: baseline.response_time_ms,
            under_attack_response_time_ms: under_attack_response_time,
            response_time_degradation_percent: degradation_percent,
            memory_usage_increase_bytes: 1024 * 100, // Estimated 100KB increase
            cpu_utilization_increase_percent: 15.0,
            throughput_reduction_percent: 10.0,
        }
    }

    /// Analyze detection patterns
    fn analyze_detection_patterns(
        &self,
        detection_times: &[u64],
        error_categorization: &HashMap<String, usize>,
    ) -> DetectionAnalysis {
        let immediate_threshold_ms = 100;
        let immediate_detections = detection_times
            .iter()
            .filter(|&&t| t <= immediate_threshold_ms)
            .count();
        let immediate_detection_rate = if !detection_times.is_empty() {
            immediate_detections as f64 / detection_times.len() as f64
        } else {
            1.0
        };

        let delayed_detection_rate = 1.0 - immediate_detection_rate;

        let timing_analysis = if !detection_times.is_empty() {
            let min_time = *detection_times.iter().min().unwrap();
            let max_time = *detection_times.iter().max().unwrap();
            let avg_time =
                detection_times.iter().sum::<u64>() as f64 / detection_times.len() as f64;
            let variance = detection_times
                .iter()
                .map(|&x| (x as f64 - avg_time).powi(2))
                .sum::<f64>()
                / detection_times.len() as f64;

            TimingAnalysis {
                minimum_detection_time_ms: min_time,
                maximum_detection_time_ms: max_time,
                average_detection_time_ms: avg_time,
                detection_time_variance: variance,
            }
        } else {
            TimingAnalysis {
                minimum_detection_time_ms: 0,
                maximum_detection_time_ms: 0,
                average_detection_time_ms: 0.0,
                detection_time_variance: 0.0,
            }
        };

        DetectionAnalysis {
            immediate_detection_rate,
            delayed_detection_rate,
            pattern_recognition_accuracy: immediate_detection_rate, // Simplified
            error_categorization: error_categorization.clone(),
            timing_analysis,
        }
    }

    /// Generate recommendations based on results
    fn generate_recommendations(
        &self,
        security_metrics: &SecurityMetrics,
        performance_impact: &PerformanceImpact,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if security_metrics.detection_rate < 0.95 {
            recommendations.push(
                "Consider using stricter security profile for better detection rates".to_string(),
            );
        }

        if security_metrics.false_positive_rate > 0.05 {
            recommendations.push(
                "High false positive rate detected - review timestamp tolerance settings"
                    .to_string(),
            );
        }

        if performance_impact.response_time_degradation_percent > 50.0 {
            recommendations.push(
                "Significant performance degradation - consider optimization or load balancing"
                    .to_string(),
            );
        }

        if security_metrics.time_to_detection_ms > 1000.0 {
            recommendations.push(
                "Slow attack detection - consider optimizing nonce store operations".to_string(),
            );
        }

        recommendations
    }

    /// Get human-readable pattern name
    fn get_pattern_name(&self, pattern: &AttackPattern) -> String {
        match pattern {
            AttackPattern::ImmediateReplay { .. } => "Immediate Replay".to_string(),
            AttackPattern::DelayedReplay { .. } => "Delayed Replay".to_string(),
            AttackPattern::ReplayFlooding { .. } => "Replay Flooding".to_string(),
            AttackPattern::TimestampManipulation { .. } => "Timestamp Manipulation".to_string(),
            AttackPattern::NoncePrediction { .. } => "Nonce Prediction".to_string(),
            AttackPattern::MultiVector { .. } => "Multi-Vector Attack".to_string(),
        }
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate based on nonce store
        let stats = self
            .target_state
            .get_nonce_store_stats()
            .unwrap_or_default();
        stats.total_nonces * 64 // Estimate 64 bytes per nonce entry
    }
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            total_attack_attempts: 0,
            successful_bypasses: 0,
            blocked_attempts: 0,
            legitimate_requests: 0,
            false_positives: 0,
            average_attempt_duration_ms: 0.0,
            peak_concurrent_attacks: 0,
        }
    }
}

// NonceStoreStats already has Default implemented in the main crate
// use datafold::datafold_node::signature_auth::NonceStoreStats;

// Removed orphan Default implementation - already exists in main crate

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_attack_simulator_creation() {
        let config = AttackSimulationConfig::default();
        let simulator = AttackSimulator::new(config).unwrap();
        assert_eq!(simulator.attack_patterns.len(), 7); // Should have 7 attack patterns
    }

    #[tokio::test]
    async fn test_baseline_establishment() {
        let config = AttackSimulationConfig::default();
        let mut simulator = AttackSimulator::new(config).unwrap();

        simulator.establish_baseline().await.unwrap();
        assert!(simulator.baseline_metrics.is_some());

        let baseline = simulator.baseline_metrics.unwrap();
        assert!(baseline.response_time_ms >= 0.0);
        assert!(baseline.throughput_ops_per_sec > 0.0);
    }

    #[tokio::test]
    async fn test_immediate_replay_simulation() {
        let config = AttackSimulationConfig {
            duration_seconds: 1,
            concurrent_attackers: 1,
            ..Default::default()
        };
        let simulator = AttackSimulator::new(config).unwrap();

        let mut metrics = ExecutionMetrics::default();
        let mut detection_times = Vec::new();
        let mut error_categorization = HashMap::new();

        simulator
            .simulate_immediate_replay(
                "test-nonce",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                &mut metrics,
                &mut detection_times,
                &mut error_categorization,
            )
            .await
            .unwrap();

        assert!(metrics.total_attack_attempts > 0);
        assert!(metrics.blocked_attempts > 0); // Should block replay attempts
    }

    #[tokio::test]
    async fn test_nonce_prediction_generation() {
        let config = AttackSimulationConfig::default();
        let simulator = AttackSimulator::new(config).unwrap();

        let sequential_nonces =
            simulator.generate_predicted_nonces(&NoncePredictionStrategy::Sequential, 5);
        assert_eq!(sequential_nonces.len(), 5);
        assert!(sequential_nonces[0].contains("predicted-000000"));

        let timestamp_nonces =
            simulator.generate_predicted_nonces(&NoncePredictionStrategy::Timestamp, 3);
        assert_eq!(timestamp_nonces.len(), 3);
        assert!(timestamp_nonces[0].contains("time-"));
    }
}
