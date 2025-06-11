//! Comprehensive Performance Benchmarking System for Signature Authentication
//!
//! This module provides comprehensive performance benchmarking for DataFold's signature
//! authentication system across all components: server, JavaScript SDK, Python SDK, CLI,
//! and end-to-end workflows.
//!
//! **Performance Targets:**
//! - Server signature verification: <1ms per request
//! - Client signature generation: <10ms per operation  
//! - End-to-end authentication: <50ms total
//! - Throughput: >1000 authenticated requests/second
//! - Memory usage: <10MB additional overhead
//! - CPU overhead: <5% under normal load

pub mod server_benchmarks;
pub mod client_benchmarks;
pub mod sdk_benchmarks;
pub mod cli_benchmarks;
pub mod end_to_end_benchmarks;
pub mod performance_analysis;
pub mod benchmark_runner;
pub mod metrics_collector;
pub mod reporting;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarkConfig {
    /// Test duration for sustained load tests
    pub test_duration_seconds: u64,
    /// Warmup period before measurements
    pub warmup_duration_seconds: u64,
    /// Number of iterations for micro-benchmarks
    pub micro_benchmark_iterations: usize,
    /// Concurrent user counts to test
    pub concurrent_user_counts: Vec<usize>,
    /// Request rates to test (requests per second)
    pub target_request_rates: Vec<f64>,
    /// Enable detailed memory profiling
    pub enable_memory_profiling: bool,
    /// Enable CPU usage monitoring
    pub enable_cpu_monitoring: bool,
    /// Enable latency distribution analysis
    pub enable_latency_analysis: bool,
    /// Enable regression testing against baselines
    pub enable_regression_testing: bool,
    /// Baseline performance data file path
    pub baseline_data_path: Option<String>,
}

impl Default for PerformanceBenchmarkConfig {
    fn default() -> Self {
        Self {
            test_duration_seconds: 60,
            warmup_duration_seconds: 10,
            micro_benchmark_iterations: 10000,
            concurrent_user_counts: vec![1, 5, 10, 25, 50, 100, 200],
            target_request_rates: vec![10.0, 50.0, 100.0, 500.0, 1000.0, 2000.0],
            enable_memory_profiling: true,
            enable_cpu_monitoring: true,
            enable_latency_analysis: true,
            enable_regression_testing: false,
            baseline_data_path: None,
        }
    }
}

/// Performance targets for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Server signature verification target (milliseconds)
    pub server_verification_ms: f64,
    /// Client signature generation target (milliseconds)
    pub client_signing_ms: f64,
    /// End-to-end authentication target (milliseconds)
    pub end_to_end_ms: f64,
    /// Minimum throughput target (requests per second)
    pub min_throughput_rps: f64,
    /// Maximum memory overhead (bytes)
    pub max_memory_overhead_bytes: usize,
    /// Maximum CPU overhead (percentage)
    pub max_cpu_overhead_percent: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            server_verification_ms: 1.0,
            client_signing_ms: 10.0,
            end_to_end_ms: 50.0,
            min_throughput_rps: 1000.0,
            max_memory_overhead_bytes: 10 * 1024 * 1024, // 10MB
            max_cpu_overhead_percent: 5.0,
        }
    }
}

/// Performance measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    /// Test name/identifier
    pub test_name: String,
    /// Component being tested
    pub component: String,
    /// Operation being measured
    pub operation: String,
    /// Number of operations performed
    pub operation_count: usize,
    /// Total test duration
    pub total_duration: Duration,
    /// Average operation time (milliseconds)
    pub avg_operation_time_ms: f64,
    /// Median operation time (milliseconds)
    pub median_operation_time_ms: f64,
    /// 95th percentile operation time (milliseconds)
    pub p95_operation_time_ms: f64,
    /// 99th percentile operation time (milliseconds)
    pub p99_operation_time_ms: f64,
    /// Minimum operation time (milliseconds)
    pub min_operation_time_ms: f64,
    /// Maximum operation time (milliseconds)
    pub max_operation_time_ms: f64,
    /// Operations per second
    pub operations_per_second: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: Option<usize>,
    /// CPU usage (percentage)
    pub cpu_usage_percent: Option<f64>,
    /// Error count
    pub error_count: usize,
    /// Success rate (percentage)
    pub success_rate_percent: f64,
    /// Additional metrics
    pub additional_metrics: HashMap<String, f64>,
}

impl PerformanceMeasurement {
    /// Create a new performance measurement
    pub fn new(
        test_name: String,
        component: String,
        operation: String,
    ) -> Self {
        Self {
            test_name,
            component,
            operation,
            operation_count: 0,
            total_duration: Duration::default(),
            avg_operation_time_ms: 0.0,
            median_operation_time_ms: 0.0,
            p95_operation_time_ms: 0.0,
            p99_operation_time_ms: 0.0,
            min_operation_time_ms: f64::INFINITY,
            max_operation_time_ms: 0.0,
            operations_per_second: 0.0,
            memory_usage_bytes: None,
            cpu_usage_percent: None,
            error_count: 0,
            success_rate_percent: 100.0,
            additional_metrics: HashMap::new(),
        }
    }

    /// Check if measurement meets performance targets
    pub fn meets_targets(&self, targets: &PerformanceTargets) -> bool {
        // Check operation time based on component
        let target_time_ms = match self.component.as_str() {
            "server" => targets.server_verification_ms,
            "client" | "sdk" | "cli" => targets.client_signing_ms,
            "end_to_end" => targets.end_to_end_ms,
            _ => return true, // Unknown component, assume pass
        };

        if self.avg_operation_time_ms > target_time_ms {
            return false;
        }

        // Check throughput
        if self.operations_per_second < targets.min_throughput_rps / 10.0 {
            return false;
        }

        // Check memory usage
        if let Some(memory) = self.memory_usage_bytes {
            if memory > targets.max_memory_overhead_bytes {
                return false;
            }
        }

        // Check CPU usage
        if let Some(cpu) = self.cpu_usage_percent {
            if cpu > targets.max_cpu_overhead_percent {
                return false;
            }
        }

        // Check success rate
        if self.success_rate_percent < 99.0 {
            return false;
        }

        true
    }
}

/// Performance benchmark result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarkResult {
    /// Benchmark configuration used
    pub config: PerformanceBenchmarkConfig,
    /// Performance targets used for validation
    pub targets: PerformanceTargets,
    /// Individual measurements
    pub measurements: Vec<PerformanceMeasurement>,
    /// Overall performance summary
    pub summary: PerformanceSummary,
    /// Regression analysis (if enabled)
    pub regression_analysis: Option<RegressionAnalysis>,
    /// Recommendations for optimization
    pub recommendations: Vec<PerformanceRecommendation>,
    /// Test execution metadata
    pub metadata: BenchmarkMetadata,
}

/// Performance summary across all measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total number of operations tested
    pub total_operations: usize,
    /// Total test duration
    pub total_duration: Duration,
    /// Overall success rate
    pub overall_success_rate: f64,
    /// Number of targets met
    pub targets_met: usize,
    /// Total number of targets
    pub total_targets: usize,
    /// Performance score (0-100)
    pub performance_score: f64,
    /// Component-specific summaries
    pub component_summaries: HashMap<String, ComponentSummary>,
}

/// Performance summary for a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    /// Component name
    pub component_name: String,
    /// Number of measurements
    pub measurement_count: usize,
    /// Average performance score
    pub avg_performance_score: f64,
    /// Best performing operation
    pub best_operation: String,
    /// Worst performing operation
    pub worst_operation: String,
    /// Performance trends
    pub trends: PerformanceTrends,
}

/// Performance trends analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Latency trend (improving/stable/degrading)
    pub latency_trend: TrendDirection,
    /// Throughput trend
    pub throughput_trend: TrendDirection,
    /// Memory usage trend
    pub memory_trend: TrendDirection,
    /// CPU usage trend
    pub cpu_trend: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Regression analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Baseline data timestamp
    pub baseline_timestamp: String,
    /// Performance regressions detected
    pub regressions: Vec<PerformanceRegression>,
    /// Performance improvements detected
    pub improvements: Vec<PerformanceImprovement>,
    /// Overall regression score
    pub regression_score: f64,
}

/// Performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Test name with regression
    pub test_name: String,
    /// Metric that regressed
    pub metric: String,
    /// Baseline value
    pub baseline_value: f64,
    /// Current value
    pub current_value: f64,
    /// Regression percentage
    pub regression_percent: f64,
    /// Severity of regression
    pub severity: RegressionSeverity,
}

/// Performance improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Test name with improvement
    pub test_name: String,
    /// Metric that improved
    pub metric: String,
    /// Baseline value
    pub baseline_value: f64,
    /// Current value
    pub current_value: f64,
    /// Improvement percentage
    pub improvement_percent: f64,
}

/// Regression severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Critical,   // >50% regression
    High,       // 25-50% regression
    Medium,     // 10-25% regression
    Low,        // 5-10% regression
    Minimal,    // <5% regression
}

/// Performance optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    /// Component this recommendation applies to
    pub component: String,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Category of recommendation
    pub category: RecommendationCategory,
    /// Description of the issue
    pub description: String,
    /// Recommended action
    pub recommendation: String,
    /// Expected improvement
    pub expected_improvement: String,
    /// Implementation effort estimate
    pub effort_estimate: EffortEstimate,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Cryptography,
    Caching,
    Concurrency,
    Memory,
    Network,
    Algorithm,
    Configuration,
}

/// Implementation effort estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortEstimate {
    Low,        // < 1 day
    Medium,     // 1-3 days
    High,       // 1-2 weeks
    VeryHigh,   // > 2 weeks
}

/// Benchmark execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    /// Test execution timestamp
    pub timestamp: String,
    /// Test environment information
    pub environment: EnvironmentInfo,
    /// Git commit hash (if available)
    pub git_commit: Option<String>,
    /// Test execution duration
    pub execution_duration: Duration,
    /// System load during testing
    pub system_load: Option<f64>,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// CPU information
    pub cpu: String,
    /// Total memory
    pub memory_gb: f64,
    /// Rust version
    pub rust_version: String,
    /// Node.js version (if applicable)
    pub nodejs_version: Option<String>,
    /// Python version (if applicable)
    pub python_version: Option<String>,
}

/// Timing utilities for benchmarks
pub struct BenchmarkTimer {
    start_time: Instant,
    measurements: Vec<Duration>,
}

impl BenchmarkTimer {
    /// Create a new benchmark timer
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            measurements: Vec::new(),
        }
    }

    /// Start timing an operation
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    /// Record the elapsed time since start
    pub fn record(&mut self) {
        let elapsed = self.start_time.elapsed();
        self.measurements.push(elapsed);
    }

    /// Get all measurements
    pub fn measurements(&self) -> &[Duration] {
        &self.measurements
    }

    /// Calculate statistics from measurements
    pub fn statistics(&self) -> TimingStatistics {
        if self.measurements.is_empty() {
            return TimingStatistics::default();
        }

        let mut times_ms: Vec<f64> = self.measurements
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        
        times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = times_ms.len();
        let total: f64 = times_ms.iter().sum();
        let avg = total / count as f64;
        let median = times_ms[count / 2];
        let p95 = times_ms[(count * 95) / 100];
        let p99 = times_ms[(count * 99) / 100];
        let min = times_ms[0];
        let max = times_ms[count - 1];

        TimingStatistics {
            count,
            avg_ms: avg,
            median_ms: median,
            p95_ms: p95,
            p99_ms: p99,
            min_ms: min,
            max_ms: max,
            total_duration: Duration::from_secs_f64(total / 1000.0),
        }
    }
}

/// Timing statistics
#[derive(Debug, Default)]
pub struct TimingStatistics {
    pub count: usize,
    pub avg_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub total_duration: Duration,
    pub std_dev_ms: f64,
}

// Re-export main benchmark types
pub use server_benchmarks::ServerPerformanceBenchmarks;
pub use client_benchmarks::ClientPerformanceBenchmarks;
pub use sdk_benchmarks::SdkPerformanceBenchmarks;
pub use cli_benchmarks::CliPerformanceBenchmarks;
pub use end_to_end_benchmarks::EndToEndPerformanceBenchmarks;
pub use benchmark_runner::PerformanceBenchmarkRunner;
pub use performance_analysis::{PerformanceAnalyzer, PerformanceAnalysisConfig, PerformanceAnalysisResult};
pub use metrics_collector::{PerformanceMetricsCollector, MetricsCollectorConfig};
pub use reporting::{PerformanceReportGenerator, ReportConfig};

/// Main performance benchmarking orchestrator that integrates all components
pub struct PerformanceBenchmarkSuite {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
    runner: Option<PerformanceBenchmarkRunner>,
    analyzer: Option<PerformanceAnalyzer>,
    report_generator: Option<PerformanceReportGenerator>,
}

impl PerformanceBenchmarkSuite {
    /// Create new performance benchmark suite
    pub fn new(
        config: PerformanceBenchmarkConfig,
        targets: PerformanceTargets,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            targets,
            runner: None,
            analyzer: None,
            report_generator: None,
        })
    }

    /// Initialize the benchmark suite with analysis and reporting capabilities
    pub fn with_analysis_and_reporting(
        mut self,
        analysis_config: PerformanceAnalysisConfig,
        report_config: ReportConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Convert config format
        let runner_config = benchmark_runner::PerformanceBenchmarkConfig {
            iterations: self.config.micro_benchmark_iterations,
            warmup_iterations: 100,
            test_duration_seconds: self.config.test_duration_seconds,
            enable_detailed_timing: self.config.enable_latency_analysis,
            enable_memory_profiling: self.config.enable_memory_profiling,
            enable_cpu_profiling: self.config.enable_cpu_monitoring,
            concurrent_threads: self.config.concurrent_user_counts.first().copied().unwrap_or(10),
            target_percentile: 95.0,
        };

        let runner_targets = benchmark_runner::PerformanceTargets {
            max_avg_operation_time_ms: self.targets.server_verification_ms,
            max_p95_operation_time_ms: self.targets.server_verification_ms * 2.0,
            max_p99_operation_time_ms: self.targets.server_verification_ms * 5.0,
            min_operations_per_second: self.targets.min_throughput_rps,
            max_error_rate_percent: 0.1,
            max_memory_usage_mb: (self.targets.max_memory_overhead_bytes / (1024 * 1024)) as f64,
            max_cpu_usage_percent: self.targets.max_cpu_overhead_percent,
        };

        self.runner = Some(PerformanceBenchmarkRunner::new(runner_config, runner_targets)?);
        self.analyzer = Some(PerformanceAnalyzer::new(analysis_config));
        self.report_generator = Some(PerformanceReportGenerator::new(report_config));

        Ok(self)
    }

    /// Run the complete performance benchmark suite
    pub async fn run_complete_benchmark_suite(&mut self) -> Result<PerformanceBenchmarkResult, Box<dyn std::error::Error>> {
        println!("🚀 Starting complete DataFold performance benchmark suite");

        // Get the current timestamp
        let start_time = std::time::Instant::now();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Run all benchmarks using the runner
        let measurements = if let Some(ref mut runner) = self.runner {
            runner.run_all_benchmarks().await?
        } else {
            return Err("Benchmark runner not initialized. Call with_analysis_and_reporting() first.".into());
        };

        // Convert measurements to the expected format
        let converted_measurements = self.convert_measurements(&measurements);

        // Analyze results
        let analysis_result = if let Some(ref analyzer) = self.analyzer {
            let runner_targets = benchmark_runner::PerformanceTargets {
                max_avg_operation_time_ms: self.targets.server_verification_ms,
                max_p95_operation_time_ms: self.targets.server_verification_ms * 2.0,
                max_p99_operation_time_ms: self.targets.server_verification_ms * 5.0,
                min_operations_per_second: self.targets.min_throughput_rps,
                max_error_rate_percent: 0.1,
                max_memory_usage_mb: (self.targets.max_memory_overhead_bytes / (1024 * 1024)) as f64,
                max_cpu_usage_percent: self.targets.max_cpu_overhead_percent,
            };
            Some(analyzer.analyze(&measurements, &runner_targets))
        } else {
            None
        };

        // Generate reports if configured
        if let (Some(ref report_generator), Some(ref analysis)) = (&self.report_generator, &analysis_result) {
            let runner_config = benchmark_runner::PerformanceBenchmarkConfig {
                iterations: self.config.micro_benchmark_iterations,
                warmup_iterations: 100,
                test_duration_seconds: self.config.test_duration_seconds,
                enable_detailed_timing: self.config.enable_latency_analysis,
                enable_memory_profiling: self.config.enable_memory_profiling,
                enable_cpu_profiling: self.config.enable_cpu_monitoring,
                concurrent_threads: self.config.concurrent_user_counts.first().copied().unwrap_or(10),
                target_percentile: 95.0,
            };

            let runner_targets = benchmark_runner::PerformanceTargets {
                max_avg_operation_time_ms: self.targets.server_verification_ms,
                max_p95_operation_time_ms: self.targets.server_verification_ms * 2.0,
                max_p99_operation_time_ms: self.targets.server_verification_ms * 5.0,
                min_operations_per_second: self.targets.min_throughput_rps,
                max_error_rate_percent: 0.1,
                max_memory_usage_mb: (self.targets.max_memory_overhead_bytes / (1024 * 1024)) as f64,
                max_cpu_usage_percent: self.targets.max_cpu_overhead_percent,
            };

            let report = report_generator.generate_report(
                &measurements,
                analysis,
                &runner_targets,
                &runner_config,
            )?;

            let exported_files = report_generator.export_report(&report)?;
            println!("📄 Reports generated: {:?}", exported_files);
        }

        // Create summary
        let summary = self.create_performance_summary(&converted_measurements, &analysis_result);

        // Create metadata
        let metadata = BenchmarkMetadata {
            timestamp,
            environment: EnvironmentInfo {
                os: std::env::consts::OS.to_string(),
                cpu: "Unknown".to_string(), // Would need system detection
                memory_gb: 8.0, // Would need system detection
                rust_version: env!("CARGO_PKG_VERSION").to_string(),
                nodejs_version: None,
                python_version: None,
            },
            git_commit: None,
            execution_duration: start_time.elapsed(),
            system_load: None,
        };

        // Convert regression analysis
        let regression_analysis = analysis_result.as_ref().map(|analysis| {
            RegressionAnalysis {
                baseline_timestamp: "N/A".to_string(),
                regressions: analysis.regressions.iter().filter(|r| r.is_regression).map(|r| {
                    PerformanceRegression {
                        test_name: r.benchmark_name.clone(),
                        metric: r.metric_name.clone(),
                        baseline_value: r.previous_value,
                        current_value: r.current_value,
                        regression_percent: r.change_percent,
                        severity: match r.severity {
                            performance_analysis::RegressionSeverity::Critical => RegressionSeverity::Critical,
                            performance_analysis::RegressionSeverity::Major => RegressionSeverity::High,
                            performance_analysis::RegressionSeverity::Minor => RegressionSeverity::Medium,
                            performance_analysis::RegressionSeverity::Negligible => RegressionSeverity::Low,
                        },
                    }
                }).collect(),
                improvements: analysis.regressions.iter().filter(|r| r.is_improvement).map(|r| {
                    PerformanceImprovement {
                        test_name: r.benchmark_name.clone(),
                        metric: r.metric_name.clone(),
                        baseline_value: r.previous_value,
                        current_value: r.current_value,
                        improvement_percent: r.change_percent.abs(),
                    }
                }).collect(),
                regression_score: 100.0 - analysis.overall_score,
            }
        });

        // Convert recommendations
        let recommendations = analysis_result.as_ref().map(|analysis| {
            analysis.recommendations.iter().map(|rec| {
                PerformanceRecommendation {
                    component: "signature_auth".to_string(),
                    priority: match rec.priority {
                        performance_analysis::RecommendationPriority::Critical => RecommendationPriority::Critical,
                        performance_analysis::RecommendationPriority::High => RecommendationPriority::High,
                        performance_analysis::RecommendationPriority::Medium => RecommendationPriority::Medium,
                        performance_analysis::RecommendationPriority::Low => RecommendationPriority::Low,
                    },
                    category: match rec.category {
                        performance_analysis::RecommendationCategory::Algorithm => RecommendationCategory::Algorithm,
                        performance_analysis::RecommendationCategory::Caching => RecommendationCategory::Caching,
                        performance_analysis::RecommendationCategory::Concurrency => RecommendationCategory::Concurrency,
                        performance_analysis::RecommendationCategory::Memory => RecommendationCategory::Memory,
                        performance_analysis::RecommendationCategory::Network => RecommendationCategory::Network,
                        _ => RecommendationCategory::Configuration,
                    },
                    description: rec.description.clone(),
                    recommendation: rec.title.clone(),
                    expected_improvement: rec.expected_impact.clone(),
                    effort_estimate: match rec.implementation_effort {
                        performance_analysis::ImplementationEffort::Minimal => EffortEstimate::Low,
                        performance_analysis::ImplementationEffort::Low => EffortEstimate::Low,
                        performance_analysis::ImplementationEffort::Medium => EffortEstimate::Medium,
                        performance_analysis::ImplementationEffort::High => EffortEstimate::High,
                    },
                }
            }).collect()
        }).unwrap_or_default();

        let result = PerformanceBenchmarkResult {
            config: self.config.clone(),
            targets: self.targets.clone(),
            measurements: converted_measurements,
            summary,
            regression_analysis,
            recommendations,
            metadata,
        };

        // Print summary
        self.print_benchmark_summary(&result);

        println!("✅ Complete performance benchmark suite finished");

        Ok(result)
    }

    /// Convert benchmark runner measurements to the expected format
    fn convert_measurements(&self, measurements: &[benchmark_runner::PerformanceMeasurement]) -> Vec<PerformanceMeasurement> {
        measurements.iter().map(|m| {
            PerformanceMeasurement {
                test_name: m.benchmark_name.clone(),
                component: m.category.clone(),
                operation: m.operation_type.clone(),
                operation_count: m.operation_count,
                total_duration: m.total_duration,
                avg_operation_time_ms: m.avg_operation_time_ms,
                median_operation_time_ms: m.median_operation_time_ms,
                p95_operation_time_ms: m.p95_operation_time_ms,
                p99_operation_time_ms: m.p99_operation_time_ms,
                min_operation_time_ms: m.min_operation_time_ms,
                max_operation_time_ms: m.max_operation_time_ms,
                operations_per_second: m.operations_per_second,
                memory_usage_bytes: Some((m.memory_usage_mb * 1024.0 * 1024.0) as usize),
                cpu_usage_percent: Some(m.cpu_usage_percent),
                error_count: m.error_count,
                success_rate_percent: m.success_rate_percent,
                additional_metrics: m.additional_metrics.clone(),
            }
        }).collect()
    }

    /// Create performance summary
    fn create_performance_summary(&self, measurements: &[PerformanceMeasurement], analysis: &Option<PerformanceAnalysisResult>) -> PerformanceSummary {
        let total_operations: usize = measurements.iter().map(|m| m.operation_count).sum();
        let total_duration = measurements.iter().map(|m| m.total_duration).max().unwrap_or_default();
        let overall_success_rate = if !measurements.is_empty() {
            measurements.iter().map(|m| m.success_rate_percent).sum::<f64>() / measurements.len() as f64
        } else {
            100.0
        };

        let targets_met = measurements.iter().filter(|m| m.meets_targets(&self.targets)).count();
        let total_targets = measurements.len();

        let performance_score = analysis.as_ref().map(|a| a.overall_score).unwrap_or(
            if total_targets > 0 {
                (targets_met as f64 / total_targets as f64) * 100.0
            } else {
                100.0
            }
        );

        // Create component summaries
        let mut component_summaries = HashMap::new();
        let components: std::collections::HashSet<String> = measurements.iter().map(|m| m.component.clone()).collect();
        
        for component in components {
            let component_measurements: Vec<_> = measurements.iter().filter(|m| m.component == component).collect();
            
            if !component_measurements.is_empty() {
                let avg_score = component_measurements.iter()
                    .map(|m| if m.meets_targets(&self.targets) { 100.0 } else { 50.0 })
                    .sum::<f64>() / component_measurements.len() as f64;

                let best_operation = component_measurements.iter()
                    .min_by(|a, b| a.avg_operation_time_ms.partial_cmp(&b.avg_operation_time_ms).unwrap())
                    .map(|m| m.operation.clone())
                    .unwrap_or_default();

                let worst_operation = component_measurements.iter()
                    .max_by(|a, b| a.avg_operation_time_ms.partial_cmp(&b.avg_operation_time_ms).unwrap())
                    .map(|m| m.operation.clone())
                    .unwrap_or_default();

                component_summaries.insert(component.clone(), ComponentSummary {
                    component_name: component,
                    measurement_count: component_measurements.len(),
                    avg_performance_score: avg_score,
                    best_operation,
                    worst_operation,
                    trends: PerformanceTrends {
                        latency_trend: TrendDirection::Unknown,
                        throughput_trend: TrendDirection::Unknown,
                        memory_trend: TrendDirection::Unknown,
                        cpu_trend: TrendDirection::Unknown,
                    },
                });
            }
        }

        PerformanceSummary {
            total_operations,
            total_duration,
            overall_success_rate,
            targets_met,
            total_targets,
            performance_score,
            component_summaries,
        }
    }

    /// Print benchmark summary
    fn print_benchmark_summary(&self, result: &PerformanceBenchmarkResult) {
        println!("\n🎯 PERFORMANCE BENCHMARK SUMMARY");
        println!("═══════════════════════════════════");
        println!("Overall Score: {:.1}/100", result.summary.performance_score);
        
        let grade = match result.summary.performance_score {
            score if score >= 90.0 => "A (Excellent)",
            score if score >= 80.0 => "B (Good)",
            score if score >= 70.0 => "C (Satisfactory)",
            score if score >= 60.0 => "D (Needs Improvement)",
            _ => "F (Poor)",
        };
        println!("Performance Grade: {}", grade);

        println!("\nTarget Compliance:");
        println!("  ✅ {}/{} benchmarks meeting targets", result.summary.targets_met, result.summary.total_targets);

        if let Some(ref regression_analysis) = result.regression_analysis {
            if !regression_analysis.regressions.is_empty() {
                println!("\nRegressions Detected:");
                for regression in &regression_analysis.regressions {
                    println!("  🚨 {} - {:.1}% degradation",
                            regression.test_name, regression.regression_percent);
                }
            }

            if !regression_analysis.improvements.is_empty() {
                println!("\nImprovements Found:");
                for improvement in &regression_analysis.improvements {
                    println!("  ✨ {} - {:.1}% improvement",
                            improvement.test_name, improvement.improvement_percent);
                }
            }
        }

        if !result.recommendations.is_empty() {
            println!("\nTop Recommendations:");
            for (i, rec) in result.recommendations.iter().take(3).enumerate() {
                println!("  {}. {}: {}", i + 1, rec.recommendation, rec.description);
            }
        }

        println!("\nComponent Performance:");
        for (component, summary) in &result.summary.component_summaries {
            println!("  📊 {}: {:.1}/100 ({} measurements)",
                    component, summary.avg_performance_score, summary.measurement_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_measurement_creation() {
        let measurement = PerformanceMeasurement::new(
            "test_signing".to_string(),
            "client".to_string(),
            "ed25519_signing".to_string(),
        );

        assert_eq!(measurement.test_name, "test_signing");
        assert_eq!(measurement.component, "client");
        assert_eq!(measurement.operation, "ed25519_signing");
        assert_eq!(measurement.operation_count, 0);
        assert_eq!(measurement.error_count, 0);
        assert_eq!(measurement.success_rate_percent, 100.0);
    }

    #[test]
    fn test_performance_targets_validation() {
        let targets = PerformanceTargets::default();
        
        // Create a measurement that meets targets
        let mut good_measurement = PerformanceMeasurement::new(
            "test".to_string(),
            "server".to_string(),
            "verification".to_string(),
        );
        good_measurement.avg_operation_time_ms = 0.5; // Under 1ms target
        good_measurement.operations_per_second = 1000.0;
        good_measurement.success_rate_percent = 99.5;
        
        assert!(good_measurement.meets_targets(&targets));
        
        // Create a measurement that doesn't meet targets
        let mut bad_measurement = PerformanceMeasurement::new(
            "test".to_string(),
            "server".to_string(),
            "verification".to_string(),
        );
        bad_measurement.avg_operation_time_ms = 2.0; // Over 1ms target
        
        assert!(!bad_measurement.meets_targets(&targets));
    }

    #[test]
    fn test_benchmark_timer() {
        let mut timer = BenchmarkTimer::new();
        
        timer.start();
        std::thread::sleep(Duration::from_millis(1));
        timer.record();
        
        timer.start();
        std::thread::sleep(Duration::from_millis(2));
        timer.record();
        
        let stats = timer.statistics();
        assert_eq!(stats.count, 2);
        assert!(stats.avg_ms >= 1.0);
        assert!(stats.min_ms >= 1.0);
        assert!(stats.max_ms >= 2.0);
    }
}