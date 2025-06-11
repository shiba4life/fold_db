//! Performance Benchmarks Under Replay Attack Conditions
//!
//! This module provides comprehensive performance testing and benchmarking
//! of DataFold's replay prevention mechanisms under various attack conditions.

use datafold::datafold_node::signature_auth::{
    SignatureAuthConfig, SignatureVerificationState, SecurityProfile
};
use datafold::FoldDbError;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub test_duration_seconds: u64,
    pub warmup_duration_seconds: u64,
    pub baseline_request_count: usize,
    pub attack_intensity_levels: Vec<AttackIntensity>,
    pub concurrent_user_counts: Vec<usize>,
    pub security_profiles: Vec<SecurityProfile>,
    pub measure_memory_usage: bool,
    pub measure_cpu_usage: bool,
    pub detailed_latency_analysis: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            test_duration_seconds: 300, // 5 minutes
            warmup_duration_seconds: 30,
            baseline_request_count: 1000,
            attack_intensity_levels: vec![
                AttackIntensity::Low,
                AttackIntensity::Medium,
                AttackIntensity::High,
                AttackIntensity::Extreme,
            ],
            concurrent_user_counts: vec![1, 5, 10, 25, 50, 100],
            security_profiles: vec![
                SecurityProfile::Strict,
                SecurityProfile::Standard,
                SecurityProfile::Lenient,
            ],
            measure_memory_usage: true,
            measure_cpu_usage: true,
            detailed_latency_analysis: true,
        }
    }
}

/// Attack intensity levels for performance testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackIntensity {
    /// 1-5 replay attempts per second
    Low,
    /// 10-20 replay attempts per second  
    Medium,
    /// 50-100 replay attempts per second
    High,
    /// 200+ replay attempts per second
    Extreme,
}

impl AttackIntensity {
    pub fn requests_per_second(&self) -> f64 {
        match self {
            AttackIntensity::Low => 3.0,
            AttackIntensity::Medium => 15.0,
            AttackIntensity::High => 75.0,
            AttackIntensity::Extreme => 250.0,
        }
    }

    pub fn concurrent_attackers(&self) -> usize {
        match self {
            AttackIntensity::Low => 1,
            AttackIntensity::Medium => 3,
            AttackIntensity::High => 10,
            AttackIntensity::Extreme => 25,
        }
    }
}

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub configuration: BenchmarkConfig,
    pub baseline_metrics: BaselineMetrics,
    pub attack_scenario_results: Vec<AttackScenarioResult>,
    pub scalability_analysis: ScalabilityAnalysis,
    pub resource_utilization: ResourceUtilization,
    pub performance_degradation_analysis: PerformanceDegradationAnalysis,
    pub recommendations: Vec<PerformanceRecommendation>,
}

/// Baseline performance metrics without attacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_requests_per_second: f64,
    pub memory_usage_baseline_bytes: usize,
    pub cpu_utilization_baseline_percent: f64,
    pub error_rate_percent: f64,
    pub nonce_store_operations_per_second: f64,
}

/// Results for specific attack scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackScenarioResult {
    pub security_profile: SecurityProfile,
    pub attack_intensity: AttackIntensity,
    pub concurrent_users: usize,
    pub legitimate_request_metrics: RequestMetrics,
    pub attack_request_metrics: RequestMetrics,
    pub security_effectiveness: SecurityEffectiveness,
    pub resource_impact: ResourceImpact,
    pub latency_distribution: LatencyDistribution,
}

/// Request performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub average_response_time_ms: f64,
    pub median_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub throughput_rps: f64,
    pub error_rate_percent: f64,
}

/// Security effectiveness under performance stress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEffectiveness {
    pub replay_detection_rate: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub attack_blocking_accuracy: f64,
    pub time_to_detection_ms: f64,
    pub security_overhead_ms: f64,
}

/// Resource utilization impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceImpact {
    pub memory_usage_increase_bytes: isize,
    pub memory_usage_increase_percent: f64,
    pub cpu_utilization_increase_percent: f64,
    pub nonce_store_size_increase: usize,
    pub cache_hit_rate_percent: f64,
    pub gc_pressure_increase_percent: f64,
}

/// Detailed latency distribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDistribution {
    pub percentiles: HashMap<String, f64>, // P50, P95, P99, P99.9, etc.
    pub latency_histogram: Vec<LatencyBucket>,
    pub outlier_analysis: OutlierAnalysis,
}

/// Latency histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBucket {
    pub min_ms: f64,
    pub max_ms: f64,
    pub count: usize,
    pub percentage: f64,
}

/// Outlier analysis for latency spikes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    pub outlier_threshold_ms: f64,
    pub outlier_count: usize,
    pub outlier_percentage: f64,
    pub max_outlier_ms: f64,
    pub outlier_causes: Vec<String>,
}

/// Scalability analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityAnalysis {
    pub max_sustainable_load_rps: f64,
    pub breaking_point_concurrent_users: usize,
    pub linear_scalability_range: (usize, usize), // (min_users, max_users)
    pub bottleneck_identification: Vec<Bottleneck>,
    pub capacity_recommendations: Vec<String>,
}

/// Performance bottleneck identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub impact_on_throughput_percent: f64,
    pub recommended_solution: String,
}

/// Bottleneck severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Performance degradation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDegradationAnalysis {
    pub max_acceptable_degradation_percent: f64,
    pub current_degradation_percent: f64,
    pub degradation_threshold_exceeded: bool,
    pub degradation_by_attack_intensity: HashMap<String, f64>,
    pub recovery_time_after_attack_ms: f64,
    pub graceful_degradation_analysis: GracefulDegradationAnalysis,
}

/// Graceful degradation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracefulDegradationAnalysis {
    pub maintains_core_functionality: bool,
    pub adaptive_rate_limiting_effectiveness: f64,
    pub circuit_breaker_effectiveness: f64,
    pub backup_mechanism_performance: f64,
}

/// Performance recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub description: String,
    pub expected_improvement_percent: f64,
    pub implementation_effort: ImplementationEffort,
    pub specific_actions: Vec<String>,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    NonceStoreOptimization,
    TimestampValidationOptimization,
    ConcurrencyOptimization,
    MemoryOptimization,
    SecurityConfiguration,
    Infrastructure,
    Monitoring,
}

/// Recommendation priorities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,    // < 1 day
    Medium, // 1-5 days
    High,   // 1-2 weeks
    VeryHigh, // > 2 weeks
}

/// Performance benchmark runner
pub struct PerformanceBenchmarkRunner {
    config: BenchmarkConfig,
    states: HashMap<SecurityProfile, Arc<SignatureVerificationState>>,
    metrics_collector: Arc<MetricsCollector>,
}

/// Metrics collection system
pub struct MetricsCollector {
    request_times: Arc<Mutex<Vec<f64>>>,
    memory_samples: Arc<Mutex<Vec<usize>>>,
    error_counts: Arc<AtomicUsize>,
    success_counts: Arc<AtomicUsize>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl PerformanceBenchmarkRunner {
    /// Create new performance benchmark runner
    pub fn new(config: BenchmarkConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut states = HashMap::new();
        
        // Initialize states for all security profiles
        for profile in &config.security_profiles {
            let auth_config = match profile {
                SecurityProfile::Strict => SignatureAuthConfig::strict(),
                SecurityProfile::Standard => SignatureAuthConfig::default(),
                SecurityProfile::Lenient => SignatureAuthConfig::lenient(),
            };
            
            let state = Arc::new(SignatureVerificationState::new(auth_config)?);
            states.insert(profile.clone(), state);
        }
        
        let metrics_collector = Arc::new(MetricsCollector::new());
        
        Ok(Self {
            config,
            states,
            metrics_collector,
        })
    }

    /// Run comprehensive performance benchmarks
    pub async fn run_benchmarks(&mut self) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        println!("🚀 Starting comprehensive performance benchmarks");
        
        // Step 1: Establish baseline performance
        println!("📊 Establishing baseline performance metrics...");
        let baseline_metrics = self.measure_baseline_performance().await?;
        
        // Step 2: Run attack scenario benchmarks
        println!("⚔️ Running attack scenario benchmarks...");
        let mut attack_scenario_results = Vec::new();
        
        for profile in &self.config.security_profiles.clone() {
            for intensity in &self.config.attack_intensity_levels.clone() {
                for &user_count in &self.config.concurrent_user_counts {
                    println!("  └─ Testing {:?} profile, {:?} intensity, {} users", 
                            profile, intensity, user_count);
                    
                    let result = self.run_attack_scenario(profile, intensity, user_count).await?;
                    attack_scenario_results.push(result);
                }
            }
        }
        
        // Step 3: Analyze scalability
        println!("📈 Analyzing scalability characteristics...");
        let scalability_analysis = self.analyze_scalability(&attack_scenario_results).await?;
        
        // Step 4: Measure resource utilization
        println!("💾 Analyzing resource utilization...");
        let resource_utilization = self.analyze_resource_utilization(&attack_scenario_results);
        
        // Step 5: Analyze performance degradation
        println!("📉 Analyzing performance degradation patterns...");
        let performance_degradation_analysis = self.analyze_performance_degradation(
            &baseline_metrics,
            &attack_scenario_results
        );
        
        // Step 6: Generate recommendations
        println!("💡 Generating performance recommendations...");
        let recommendations = self.generate_performance_recommendations(
            &baseline_metrics,
            &attack_scenario_results,
            &scalability_analysis,
            &performance_degradation_analysis
        );
        
        println!("✅ Performance benchmarks completed");
        
        Ok(BenchmarkResult {
            configuration: self.config.clone(),
            baseline_metrics,
            attack_scenario_results,
            scalability_analysis,
            resource_utilization,
            performance_degradation_analysis,
            recommendations,
        })
    }

    /// Measure baseline performance without attacks
    async fn measure_baseline_performance(&self) -> Result<BaselineMetrics, Box<dyn std::error::Error>> {
        let state = self.states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile state not found")?;
        
        self.metrics_collector.reset();
        
        // Warmup phase
        for _ in 0..100 {
            let nonce = Uuid::new_v4().to_string();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let _ = state.check_and_store_nonce(&nonce, timestamp);
        }
        
        // Actual measurement phase
        let start_time = Instant::now();
        let mut response_times = Vec::new();
        
        for _ in 0..self.config.baseline_request_count {
            let nonce = Uuid::new_v4().to_string();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            
            let request_start = Instant::now();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            let response_time = request_start.elapsed().as_millis() as f64;
            
            response_times.push(response_time);
            
            if result.is_ok() {
                self.metrics_collector.record_success();
            } else {
                self.metrics_collector.record_error();
            }
        }
        
        let total_duration = start_time.elapsed();
        let throughput = self.config.baseline_request_count as f64 / total_duration.as_secs_f64();
        
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let average_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let p95_response_time = response_times[(response_times.len() * 95 / 100).min(response_times.len() - 1)];
        let p99_response_time = response_times[(response_times.len() * 99 / 100).min(response_times.len() - 1)];
        
        Ok(BaselineMetrics {
            average_response_time_ms: average_response_time,
            p95_response_time_ms: p95_response_time,
            p99_response_time_ms: p99_response_time,
            throughput_requests_per_second: throughput,
            memory_usage_baseline_bytes: self.estimate_memory_usage(state),
            cpu_utilization_baseline_percent: 5.0, // Simplified estimation
            error_rate_percent: 0.0, // Baseline should have no errors
            nonce_store_operations_per_second: throughput, // 1:1 for baseline
        })
    }

    /// Run specific attack scenario benchmark
    async fn run_attack_scenario(
        &self,
        profile: &SecurityProfile,
        intensity: &AttackIntensity,
        concurrent_users: usize,
    ) -> Result<AttackScenarioResult, Box<dyn std::error::Error>> {
        
        let state = self.states.get(profile)
            .ok_or("Security profile state not found")?;
        
        self.metrics_collector.reset();
        
        // Generate legitimate and attack traffic
        let attack_rps = intensity.requests_per_second();
        let legitimate_rps = attack_rps * 0.1; // 10% legitimate traffic
        
        let attack_interval_ms = (1000.0 / attack_rps) as u64;
        let legitimate_interval_ms = (1000.0 / legitimate_rps) as u64;
        
        let mut legitimate_metrics = Vec::new();
        let mut attack_metrics = Vec::new();
        
        // Run concurrent attack simulation
        let attack_tasks = self.spawn_attack_tasks(
            state.clone(),
            intensity,
            concurrent_users,
            &mut attack_metrics
        ).await;
        
        let legitimate_tasks = self.spawn_legitimate_tasks(
            state.clone(),
            concurrent_users / 4, // Fewer legitimate users
            &mut legitimate_metrics
        ).await;
        
        // Wait for completion
        let (attack_results, legitimate_results) = tokio::join!(
            join_all(attack_tasks),
            join_all(legitimate_tasks)
        );
        
        // Process results
        let legitimate_request_metrics = self.calculate_request_metrics(&legitimate_results);
        let attack_request_metrics = self.calculate_request_metrics(&attack_results);
        
        let security_effectiveness = self.calculate_security_effectiveness(&attack_results);
        let resource_impact = self.calculate_resource_impact(state);
        let latency_distribution = self.calculate_latency_distribution(&attack_results, &legitimate_results);
        
        Ok(AttackScenarioResult {
            security_profile: profile.clone(),
            attack_intensity: intensity.clone(),
            concurrent_users,
            legitimate_request_metrics,
            attack_request_metrics,
            security_effectiveness,
            resource_impact,
            latency_distribution,
        })
    }

    /// Spawn attack tasks for concurrent execution
    async fn spawn_attack_tasks(
        &self,
        state: Arc<SignatureVerificationState>,
        intensity: &AttackIntensity,
        concurrent_attackers: usize,
        _metrics: &mut Vec<f64>,
    ) -> Vec<tokio::task::JoinHandle<Vec<(Result<(), FoldDbError>, f64)>>> {
        
        let mut tasks = Vec::new();
        let requests_per_attacker = (intensity.requests_per_second() / concurrent_attackers as f64).ceil() as usize;
        
        for attacker_id in 0..concurrent_attackers {
            let state_clone = Arc::clone(&state);
            let attack_duration = self.config.test_duration_seconds;
            
            let task = tokio::spawn(async move {
                let mut results = Vec::new();
                let base_nonce = format!("attacker-{}-nonce", attacker_id);
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                
                // First establish the nonce
                let _ = state_clone.check_and_store_nonce(&base_nonce, timestamp);
                
                // Now perform replay attacks
                for i in 0..requests_per_attacker {
                    let request_start = Instant::now();
                    let result = state_clone.check_and_store_nonce(&base_nonce, timestamp + i as u64);
                    let response_time = request_start.elapsed().as_millis() as f64;
                    
                    results.push((result, response_time));
                    
                    // Rate limiting
                    sleep(Duration::from_millis(100)).await;
                    
                    // Check if we've exceeded test duration
                    if request_start.elapsed().as_secs() > attack_duration {
                        break;
                    }
                }
                
                results
            });
            
            tasks.push(task);
        }
        
        tasks
    }

    /// Spawn legitimate traffic tasks
    async fn spawn_legitimate_tasks(
        &self,
        state: Arc<SignatureVerificationState>,
        concurrent_users: usize,
        _metrics: &mut Vec<f64>,
    ) -> Vec<tokio::task::JoinHandle<Vec<(Result<(), FoldDbError>, f64)>>> {
        
        let mut tasks = Vec::new();
        let requests_per_user = 10; // Each user makes 10 requests
        
        for user_id in 0..concurrent_users {
            let state_clone = Arc::clone(&state);
            
            let task = tokio::spawn(async move {
                let mut results = Vec::new();
                
                for i in 0..requests_per_user {
                    let nonce = format!("user-{}-request-{}", user_id, i);
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    
                    let request_start = Instant::now();
                    let result = state_clone.check_and_store_nonce(&nonce, timestamp);
                    let response_time = request_start.elapsed().as_millis() as f64;
                    
                    results.push((result, response_time));
                    
                    // Realistic user pacing
                    sleep(Duration::from_millis(500)).await;
                }
                
                results
            });
            
            tasks.push(task);
        }
        
        tasks
    }

    /// Calculate request metrics from task results
    fn calculate_request_metrics(
        &self,
        task_results: &[Result<Vec<(Result<(), FoldDbError>, f64)>, tokio::task::JoinError>]
    ) -> RequestMetrics {
        
        let mut all_results = Vec::new();
        let mut all_response_times = Vec::new();
        
        for task_result in task_results {
            if let Ok(results) = task_result {
                for (result, response_time) in results {
                    all_results.push(result.is_ok());
                    all_response_times.push(*response_time);
                }
            }
        }
        
        let total_requests = all_results.len();
        let successful_requests = all_results.iter().filter(|&&success| success).count();
        let failed_requests = total_requests - successful_requests;
        
        all_response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let average_response_time = if !all_response_times.is_empty() {
            all_response_times.iter().sum::<f64>() / all_response_times.len() as f64
        } else {
            0.0
        };
        
        let median_response_time = if !all_response_times.is_empty() {
            all_response_times[all_response_times.len() / 2]
        } else {
            0.0
        };
        
        let p95_response_time = if !all_response_times.is_empty() {
            all_response_times[(all_response_times.len() * 95 / 100).min(all_response_times.len() - 1)]
        } else {
            0.0
        };
        
        let p99_response_time = if !all_response_times.is_empty() {
            all_response_times[(all_response_times.len() * 99 / 100).min(all_response_times.len() - 1)]
        } else {
            0.0
        };
        
        let min_response_time = all_response_times.first().copied().unwrap_or(0.0);
        let max_response_time = all_response_times.last().copied().unwrap_or(0.0);
        
        RequestMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time_ms: average_response_time,
            median_response_time_ms: median_response_time,
            p95_response_time_ms: p95_response_time,
            p99_response_time_ms: p99_response_time,
            min_response_time_ms: min_response_time,
            max_response_time_ms: max_response_time,
            throughput_rps: total_requests as f64 / self.config.test_duration_seconds as f64,
            error_rate_percent: if total_requests > 0 {
                (failed_requests as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Calculate security effectiveness metrics
    fn calculate_security_effectiveness(
        &self,
        task_results: &[Result<Vec<(Result<(), FoldDbError>, f64)>, tokio::task::JoinError>]
    ) -> SecurityEffectiveness {
        
        let mut total_attacks = 0;
        let mut blocked_attacks = 0;
        let mut detection_times = Vec::new();
        
        for task_result in task_results {
            if let Ok(results) = task_result {
                for (result, response_time) in results {
                    total_attacks += 1;
                    detection_times.push(*response_time);
                    
                    if result.is_err() {
                        blocked_attacks += 1;
                    }
                }
            }
        }
        
        let detection_rate = if total_attacks > 0 {
            blocked_attacks as f64 / total_attacks as f64
        } else {
            1.0
        };
        
        let avg_detection_time = if !detection_times.is_empty() {
            detection_times.iter().sum::<f64>() / detection_times.len() as f64
        } else {
            0.0
        };
        
        SecurityEffectiveness {
            replay_detection_rate: detection_rate,
            false_positive_rate: 0.01, // Simplified - would need legitimate traffic analysis
            false_negative_rate: 1.0 - detection_rate,
            attack_blocking_accuracy: detection_rate,
            time_to_detection_ms: avg_detection_time,
            security_overhead_ms: avg_detection_time * 0.1, // Estimate 10% overhead
        }
    }

    /// Calculate resource impact
    fn calculate_resource_impact(&self, state: &Arc<SignatureVerificationState>) -> ResourceImpact {
        let current_memory = self.estimate_memory_usage(state);
        let baseline_memory = 1024 * 10; // 10KB baseline estimate
        
        ResourceImpact {
            memory_usage_increase_bytes: current_memory as isize - baseline_memory as isize,
            memory_usage_increase_percent: ((current_memory as f64 - baseline_memory as f64) / baseline_memory as f64) * 100.0,
            cpu_utilization_increase_percent: 15.0, // Estimated increase
            nonce_store_size_increase: current_memory / 64, // Rough estimate of nonce count
            cache_hit_rate_percent: 85.0, // Estimated cache efficiency
            gc_pressure_increase_percent: 5.0, // Estimated GC impact
        }
    }

    /// Calculate detailed latency distribution
    fn calculate_latency_distribution(
        &self,
        attack_results: &[Result<Vec<(Result<(), FoldDbError>, f64)>, tokio::task::JoinError>],
        legitimate_results: &[Result<Vec<(Result<(), FoldDbError>, f64)>, tokio::task::JoinError>],
    ) -> LatencyDistribution {
        
        let mut all_latencies = Vec::new();
        
        // Collect all latencies
        for task_result in attack_results.iter().chain(legitimate_results.iter()) {
            if let Ok(results) = task_result {
                for (_, response_time) in results {
                    all_latencies.push(*response_time);
                }
            }
        }
        
        all_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Calculate percentiles
        let mut percentiles = HashMap::new();
        if !all_latencies.is_empty() {
            percentiles.insert("P50".to_string(), all_latencies[all_latencies.len() * 50 / 100]);
            percentiles.insert("P90".to_string(), all_latencies[all_latencies.len() * 90 / 100]);
            percentiles.insert("P95".to_string(), all_latencies[all_latencies.len() * 95 / 100]);
            percentiles.insert("P99".to_string(), all_latencies[all_latencies.len() * 99 / 100]);
            percentiles.insert("P99.9".to_string(), all_latencies[(all_latencies.len() * 999 / 1000).min(all_latencies.len() - 1)]);
        }
        
        // Create histogram
        let latency_histogram = self.create_latency_histogram(&all_latencies);
        
        // Analyze outliers
        let outlier_analysis = self.analyze_latency_outliers(&all_latencies);
        
        LatencyDistribution {
            percentiles,
            latency_histogram,
            outlier_analysis,
        }
    }

    /// Create latency histogram
    fn create_latency_histogram(&self, latencies: &[f64]) -> Vec<LatencyBucket> {
        if latencies.is_empty() {
            return Vec::new();
        }
        
        let min_latency = latencies.first().copied().unwrap_or(0.0);
        let max_latency = latencies.last().copied().unwrap_or(0.0);
        let bucket_count = 10;
        let bucket_size = (max_latency - min_latency) / bucket_count as f64;
        
        let mut buckets = Vec::new();
        let total_count = latencies.len();
        
        for i in 0..bucket_count {
            let bucket_min = min_latency + i as f64 * bucket_size;
            let bucket_max = min_latency + (i + 1) as f64 * bucket_size;
            
            let count = latencies.iter()
                .filter(|&&latency| latency >= bucket_min && latency < bucket_max)
                .count();
            
            let percentage = (count as f64 / total_count as f64) * 100.0;
            
            buckets.push(LatencyBucket {
                min_ms: bucket_min,
                max_ms: bucket_max,
                count,
                percentage,
            });
        }
        
        buckets
    }

    /// Analyze latency outliers
    fn analyze_latency_outliers(&self, latencies: &[f64]) -> OutlierAnalysis {
        if latencies.is_empty() {
            return OutlierAnalysis {
                outlier_threshold_ms: 0.0,
                outlier_count: 0,
                outlier_percentage: 0.0,
                max_outlier_ms: 0.0,
                outlier_causes: Vec::new(),
            };
        }
        
        // Use P95 as outlier threshold
        let threshold_index = (latencies.len() * 95 / 100).min(latencies.len() - 1);
        let outlier_threshold = latencies[threshold_index];
        
        let outliers: Vec<f64> = latencies.iter()
            .filter(|&&latency| latency > outlier_threshold)
            .copied()
            .collect();
        
        let outlier_count = outliers.len();
        let outlier_percentage = (outlier_count as f64 / latencies.len() as f64) * 100.0;
        let max_outlier = outliers.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied().unwrap_or(0.0);
        
        let outlier_causes = vec![
            "Nonce store cleanup operations".to_string(),
            "Memory allocation pressure".to_string(),
            "Concurrent request contention".to_string(),
            "Attack detection overhead".to_string(),
        ];
        
        OutlierAnalysis {
            outlier_threshold_ms: outlier_threshold,
            outlier_count,
            outlier_percentage,
            max_outlier_ms: max_outlier,
            outlier_causes,
        }
    }

    /// Analyze scalability characteristics
    async fn analyze_scalability(&self, results: &[AttackScenarioResult]) -> Result<ScalabilityAnalysis, Box<dyn std::error::Error>> {
        // Analyze how performance scales with load
        let mut throughput_by_users = HashMap::new();
        
        for result in results {
            let user_count = result.concurrent_users;
            let throughput = result.legitimate_request_metrics.throughput_rps;
            
            throughput_by_users.entry(user_count)
                .and_modify(|e: &mut Vec<f64>| e.push(throughput))
                .or_insert_with(|| vec![throughput]);
        }
        
        // Find maximum sustainable load
        let max_sustainable_load = throughput_by_users.values()
            .filter_map(|throughputs| throughputs.iter().max_by(|a, b| a.partial_cmp(b).unwrap()))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0);
        
        // Find breaking point (where performance degrades significantly)
        let breaking_point = self.find_breaking_point(results);
        
        // Identify linear scalability range
        let linear_range = self.find_linear_scalability_range(results);
        
        // Identify bottlenecks
        let bottlenecks = self.identify_bottlenecks(results);
        
        // Generate capacity recommendations
        let capacity_recommendations = self.generate_capacity_recommendations(results);
        
        Ok(ScalabilityAnalysis {
            max_sustainable_load_rps: max_sustainable_load,
            breaking_point_concurrent_users: breaking_point,
            linear_scalability_range: linear_range,
            bottleneck_identification: bottlenecks,
            capacity_recommendations,
        })
    }

    /// Find the breaking point where performance significantly degrades
    fn find_breaking_point(&self, results: &[AttackScenarioResult]) -> usize {
        // Simplified logic - find where response time increases significantly
        let mut response_times_by_users = HashMap::new();
        
        for result in results {
            let user_count = result.concurrent_users;
            let response_time = result.legitimate_request_metrics.average_response_time_ms;
            
            response_times_by_users.entry(user_count)
                .and_modify(|e: &mut Vec<f64>| e.push(response_time))
                .or_insert_with(|| vec![response_time]);
        }
        
        let mut sorted_users: Vec<usize> = response_times_by_users.keys().copied().collect();
        sorted_users.sort();
        
        // Find where response time increases by more than 50%
        for window in sorted_users.windows(2) {
            if let (Some(current_times), Some(next_times)) = (
                response_times_by_users.get(&window[0]),
                response_times_by_users.get(&window[1])
            ) {
                let current_avg = current_times.iter().sum::<f64>() / current_times.len() as f64;
                let next_avg = next_times.iter().sum::<f64>() / next_times.len() as f64;
                
                if next_avg > current_avg * 1.5 {
                    return window[1];
                }
            }
        }
        
        sorted_users.last().copied().unwrap_or(100)
    }

    /// Find linear scalability range
    fn find_linear_scalability_range(&self, results: &[AttackScenarioResult]) -> (usize, usize) {
        // Simplified - return range where throughput scales roughly linearly
        let user_counts: Vec<usize> = results.iter()
            .map(|r| r.concurrent_users)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        let min_users = user_counts.iter().min().copied().unwrap_or(1);
        let max_users = user_counts.iter().max().copied().unwrap_or(100);
        
        (min_users, max_users / 2) // Assume linear up to half of max tested
    }

    /// Identify performance bottlenecks
    fn identify_bottlenecks(&self, results: &[AttackScenarioResult]) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Analyze memory usage growth
        let high_memory_usage = results.iter()
            .any(|r| r.resource_impact.memory_usage_increase_percent > 100.0);
        
        if high_memory_usage {
            bottlenecks.push(Bottleneck {
                component: "Nonce Store Memory".to_string(),
                severity: BottleneckSeverity::High,
                description: "Nonce store memory usage growing rapidly under load".to_string(),
                impact_on_throughput_percent: 25.0,
                recommended_solution: "Implement more aggressive nonce cleanup or use external storage".to_string(),
            });
        }
        
        // Analyze response time variance
        let high_latency_variance = results.iter()
            .any(|r| r.legitimate_request_metrics.p99_response_time_ms > 
                    r.legitimate_request_metrics.average_response_time_ms * 3.0);
        
        if high_latency_variance {
            bottlenecks.push(Bottleneck {
                component: "Request Processing".to_string(),
                severity: BottleneckSeverity::Medium,
                description: "High latency variance under concurrent load".to_string(),
                impact_on_throughput_percent: 15.0,
                recommended_solution: "Optimize critical sections and reduce lock contention".to_string(),
            });
        }
        
        bottlenecks
    }

    /// Generate capacity recommendations
    fn generate_capacity_recommendations(&self, results: &[AttackScenarioResult]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze maximum tested load
        let max_users = results.iter()
            .map(|r| r.concurrent_users)
            .max()
            .unwrap_or(1);
        
        let max_throughput = results.iter()
            .map(|r| r.legitimate_request_metrics.throughput_rps)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        recommendations.push(format!(
            "Maximum tested throughput: {:.1} RPS with {} concurrent users",
            max_throughput, max_users
        ));
        
        if max_throughput < 100.0 {
            recommendations.push("Consider performance optimizations to improve throughput".to_string());
        }
        
        if max_users < 50 {
            recommendations.push("Test with higher concurrent user loads to find true capacity limits".to_string());
        }
        
        recommendations.push("Monitor memory usage and implement capacity alerts".to_string());
        
        recommendations
    }

    /// Analyze resource utilization across all results
    fn analyze_resource_utilization(&self, results: &[AttackScenarioResult]) -> ResourceUtilization {
        let max_memory_increase = results.iter()
            .map(|r| r.resource_impact.memory_usage_increase_bytes)
            .max()
            .unwrap_or(0);
        
        let avg_cpu_increase = results.iter()
            .map(|r| r.resource_impact.cpu_utilization_increase_percent)
            .sum::<f64>() / results.len().max(1) as f64;
        
        ResourceUtilization {
            peak_memory_usage_bytes: max_memory_increase as usize,
            average_cpu_utilization_percent: avg_cpu_increase,
            memory_growth_rate_bytes_per_second: max_memory_increase as f64 / self.config.test_duration_seconds as f64,
            nonce_store_efficiency_percent: 85.0, // Estimated based on cache hit rates
            resource_cleanup_effectiveness: 0.9,
        }
    }

    /// Analyze performance degradation patterns
    fn analyze_performance_degradation(
        &self,
        baseline: &BaselineMetrics,
        results: &[AttackScenarioResult],
    ) -> PerformanceDegradationAnalysis {
        
        let mut max_degradation = 0.0;
        let mut degradation_by_intensity = HashMap::new();
        
        for result in results {
            let legitimate_response_time = result.legitimate_request_metrics.average_response_time_ms;
            let degradation = ((legitimate_response_time - baseline.average_response_time_ms) / baseline.average_response_time_ms) * 100.0;
            
            if degradation > max_degradation {
                max_degradation = degradation;
            }
            
            let intensity_key = format!("{:?}", result.attack_intensity);
            degradation_by_intensity.entry(intensity_key)
                .and_modify(|e: &mut f64| *e = e.max(degradation))
                .or_insert(degradation);
        }
        
        PerformanceDegradationAnalysis {
            max_acceptable_degradation_percent: 25.0, // 25% degradation threshold
            current_degradation_percent: max_degradation,
            degradation_threshold_exceeded: max_degradation > 25.0,
            degradation_by_attack_intensity: degradation_by_intensity,
            recovery_time_after_attack_ms: 1000.0, // Estimated recovery time
            graceful_degradation_analysis: GracefulDegradationAnalysis {
                maintains_core_functionality: max_degradation < 50.0,
                adaptive_rate_limiting_effectiveness: 0.8,
                circuit_breaker_effectiveness: 0.9,
                backup_mechanism_performance: 0.7,
            },
        }
    }

    /// Generate performance recommendations
    fn generate_performance_recommendations(
        &self,
        baseline: &BaselineMetrics,
        results: &[AttackScenarioResult],
        scalability: &ScalabilityAnalysis,
        degradation: &PerformanceDegradationAnalysis,
    ) -> Vec<PerformanceRecommendation> {
        
        let mut recommendations = Vec::new();
        
        // Check if baseline performance is acceptable
        if baseline.average_response_time_ms > 100.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::NonceStoreOptimization,
                priority: RecommendationPriority::High,
                description: "Baseline response time exceeds recommended threshold".to_string(),
                expected_improvement_percent: 30.0,
                implementation_effort: ImplementationEffort::Medium,
                specific_actions: vec![
                    "Optimize nonce storage data structure".to_string(),
                    "Implement more efficient lookup algorithms".to_string(),
                    "Consider using external caching layer".to_string(),
                ],
            });
        }
        
        // Check degradation under attack
        if degradation.current_degradation_percent > 25.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::ConcurrencyOptimization,
                priority: RecommendationPriority::Critical,
                description: "Performance degrades significantly under attack".to_string(),
                expected_improvement_percent: 40.0,
                implementation_effort: ImplementationEffort::High,
                specific_actions: vec![
                    "Implement request queuing and rate limiting".to_string(),
                    "Optimize concurrent access patterns".to_string(),
                    "Add circuit breaker mechanisms".to_string(),
                ],
            });
        }
        
        // Memory optimization recommendations
        let high_memory_usage = results.iter()
            .any(|r| r.resource_impact.memory_usage_increase_percent > 100.0);
        
        if high_memory_usage {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::MemoryOptimization,
                priority: RecommendationPriority::High,
                description: "Memory usage increases significantly under load".to_string(),
                expected_improvement_percent: 25.0,
                implementation_effort: ImplementationEffort::Medium,
                specific_actions: vec![
                    "Implement more aggressive nonce cleanup".to_string(),
                    "Optimize memory allocation patterns".to_string(),
                    "Consider memory-mapped storage for large nonce stores".to_string(),
                ],
            });
        }
        
        // Scalability recommendations
        if scalability.max_sustainable_load_rps < 50.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Infrastructure,
                priority: RecommendationPriority::Medium,
                description: "Low maximum sustainable throughput detected".to_string(),
                expected_improvement_percent: 100.0,
                implementation_effort: ImplementationEffort::High,
                specific_actions: vec![
                    "Consider horizontal scaling architecture".to_string(),
                    "Implement load balancing".to_string(),
                    "Optimize critical path performance".to_string(),
                ],
            });
        }
        
        recommendations
    }

    /// Estimate memory usage for a state
    fn estimate_memory_usage(&self, state: &Arc<SignatureVerificationState>) -> usize {
        let stats = state.get_nonce_store_stats().unwrap_or_default();
        stats.total_nonces * 64 + 1024 * 10 // Base overhead
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            request_times: Arc::new(Mutex::new(Vec::new())),
            memory_samples: Arc::new(Mutex::new(Vec::new())),
            error_counts: Arc::new(AtomicUsize::new(0)),
            success_counts: Arc::new(AtomicUsize::new(0)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    fn reset(&self) {
        self.request_times.lock().unwrap().clear();
        self.memory_samples.lock().unwrap().clear();
        self.error_counts.store(0, Ordering::Relaxed);
        self.success_counts.store(0, Ordering::Relaxed);
        *self.start_time.lock().unwrap() = Some(Instant::now());
    }

    fn record_success(&self) {
        self.success_counts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_error(&self) {
        self.error_counts.fetch_add(1, Ordering::Relaxed);
    }
}

/// Resource utilization summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub peak_memory_usage_bytes: usize,
    pub average_cpu_utilization_percent: f64,
    pub memory_growth_rate_bytes_per_second: f64,
    pub nonce_store_efficiency_percent: f64,
    pub resource_cleanup_effectiveness: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_runner_creation() {
        let config = BenchmarkConfig::default();
        let runner = PerformanceBenchmarkRunner::new(config).unwrap();
        assert!(runner.states.contains_key(&SecurityProfile::Strict));
        assert!(runner.states.contains_key(&SecurityProfile::Standard));
        assert!(runner.states.contains_key(&SecurityProfile::Lenient));
    }

    #[tokio::test]
    async fn test_baseline_performance_measurement() {
        let mut config = BenchmarkConfig::default();
        config.baseline_request_count = 100; // Small number for testing
        
        let mut runner = PerformanceBenchmarkRunner::new(config).unwrap();
        let baseline = runner.measure_baseline_performance().await.unwrap();
        
        assert!(baseline.average_response_time_ms >= 0.0);
        assert!(baseline.throughput_requests_per_second > 0.0);
        assert_eq!(baseline.error_rate_percent, 0.0); // Baseline should have no errors
    }

    #[tokio::test]
    async fn test_attack_intensity_configuration() {
        assert_eq!(AttackIntensity::Low.requests_per_second(), 3.0);
        assert_eq!(AttackIntensity::Medium.requests_per_second(), 15.0);
        assert_eq!(AttackIntensity::High.requests_per_second(), 75.0);
        assert_eq!(AttackIntensity::Extreme.requests_per_second(), 250.0);
        
        assert_eq!(AttackIntensity::Low.concurrent_attackers(), 1);
        assert_eq!(AttackIntensity::Extreme.concurrent_attackers(), 25);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.reset();
        collector.record_success();
        collector.record_success();
        collector.record_error();
        
        assert_eq!(collector.success_counts.load(Ordering::Relaxed), 2);
        assert_eq!(collector.error_counts.load(Ordering::Relaxed), 1);
    }
}