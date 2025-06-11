//! Performance Benchmark Runner
//!
//! This module provides the main orchestrator for running comprehensive performance
//! benchmarks across all DataFold signature authentication components.

use super::{
    PerformanceBenchmarkConfig, PerformanceTargets, PerformanceBenchmarkResult,
    PerformanceMeasurement, PerformanceSummary, ComponentSummary, PerformanceTrends,
    TrendDirection, RegressionAnalysis, PerformanceRecommendation, BenchmarkMetadata,
    EnvironmentInfo, RecommendationPriority, RecommendationCategory, EffortEstimate
};
use super::server_benchmarks::ServerPerformanceBenchmarks;
use super::client_benchmarks::ClientPerformanceBenchmarks;
use super::sdk_benchmarks::SdkPerformanceBenchmarks;
use super::cli_benchmarks::CliPerformanceBenchmarks;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

/// Main performance benchmark runner
pub struct PerformanceBenchmarkRunner {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
}

impl PerformanceBenchmarkRunner {
    /// Create new performance benchmark runner
    pub fn new(config: PerformanceBenchmarkConfig, targets: PerformanceTargets) -> Self {
        Self { config, targets }
    }

    /// Create runner with default configuration
    pub fn with_defaults() -> Self {
        Self::new(
            PerformanceBenchmarkConfig::default(),
            PerformanceTargets::default(),
        )
    }

    /// Create runner optimized for CI/CD environments
    pub fn for_ci_cd() -> Self {
        let mut config = PerformanceBenchmarkConfig::default();
        config.test_duration_seconds = 30; // Shorter tests for CI
        config.warmup_duration_seconds = 5;
        config.micro_benchmark_iterations = 1000; // Fewer iterations
        config.concurrent_user_counts = vec![1, 5, 10]; // Fewer concurrency levels
        config.enable_regression_testing = true;
        
        Self::new(config, PerformanceTargets::default())
    }

    /// Create runner for comprehensive local testing
    pub fn for_local_testing() -> Self {
        let mut config = PerformanceBenchmarkConfig::default();
        config.test_duration_seconds = 120; // Longer comprehensive tests
        config.warmup_duration_seconds = 15;
        config.micro_benchmark_iterations = 10000;
        config.enable_memory_profiling = true;
        config.enable_cpu_monitoring = true;
        config.enable_latency_analysis = true;
        
        Self::new(config, PerformanceTargets::default())
    }

    /// Run all performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<PerformanceBenchmarkResult, Box<dyn std::error::Error>> {
        println!("🚀 Starting comprehensive performance benchmark suite");
        println!("================================================");
        
        let overall_start = Instant::now();
        let mut all_measurements = Vec::new();
        
        // Run server-side benchmarks
        if let Ok(mut server_benchmarks) = ServerPerformanceBenchmarks::new(
            self.config.clone(), 
            self.targets.clone()
        ) {
            println!("\n🔧 Running server-side benchmarks...");
            match server_benchmarks.run_all_benchmarks().await {
                Ok(measurements) => {
                    println!("  ✅ Server benchmarks completed: {} measurements", measurements.len());
                    all_measurements.extend(measurements);
                }
                Err(e) => {
                    println!("  ⚠️  Server benchmarks failed: {}", e);
                }
            }
        }
        
        // Run client-side benchmarks
        if let Ok(mut client_benchmarks) = ClientPerformanceBenchmarks::new(
            self.config.clone(),
            self.targets.clone()
        ) {
            println!("\n💻 Running client-side benchmarks...");
            match client_benchmarks.run_all_benchmarks().await {
                Ok(measurements) => {
                    println!("  ✅ Client benchmarks completed: {} measurements", measurements.len());
                    all_measurements.extend(measurements);
                }
                Err(e) => {
                    println!("  ⚠️  Client benchmarks failed: {}", e);
                }
            }
        }
        
        // Run SDK benchmarks
        if let Ok(mut sdk_benchmarks) = SdkPerformanceBenchmarks::new(
            self.config.clone(),
            self.targets.clone()
        ) {
            println!("\n📦 Running SDK benchmarks...");
            match sdk_benchmarks.run_all_benchmarks().await {
                Ok(measurements) => {
                    println!("  ✅ SDK benchmarks completed: {} measurements", measurements.len());
                    all_measurements.extend(measurements);
                }
                Err(e) => {
                    println!("  ⚠️  SDK benchmarks failed: {}", e);
                }
            }
        }
        
        // Run CLI benchmarks
        if let Ok(mut cli_benchmarks) = CliPerformanceBenchmarks::new(
            self.config.clone(),
            self.targets.clone()
        ) {
            println!("\n⌨️  Running CLI benchmarks...");
            match cli_benchmarks.run_all_benchmarks().await {
                Ok(measurements) => {
                    println!("  ✅ CLI benchmarks completed: {} measurements", measurements.len());
                    all_measurements.extend(measurements);
                }
                Err(e) => {
                    println!("  ⚠️  CLI benchmarks failed: {}", e);
                }
            }
        }
        
        let total_execution_time = overall_start.elapsed();
        
        println!("\n📊 Analyzing benchmark results...");
        
        // Generate performance summary
        let summary = self.generate_performance_summary(&all_measurements, total_execution_time);
        
        // Perform regression analysis if enabled
        let regression_analysis = if self.config.enable_regression_testing {
            self.perform_regression_analysis(&all_measurements).await.ok()
        } else {
            None
        };
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&all_measurements, &summary);
        
        // Create benchmark metadata
        let metadata = self.create_benchmark_metadata(total_execution_time).await;
        
        let result = PerformanceBenchmarkResult {
            config: self.config.clone(),
            targets: self.targets.clone(),
            measurements: all_measurements,
            summary,
            regression_analysis,
            recommendations,
            metadata,
        };
        
        println!("\n✅ Performance benchmark suite completed");
        println!("   Total measurements: {}", result.measurements.len());
        println!("   Execution time: {:.1}s", total_execution_time.as_secs_f64());
        println!("   Performance score: {:.1}/100", result.summary.performance_score);
        
        if let Some(ref regression) = result.regression_analysis {
            if !regression.regressions.is_empty() {
                println!("   ⚠️  {} performance regressions detected", regression.regressions.len());
            }
            if !regression.improvements.is_empty() {
                println!("   🎉 {} performance improvements detected", regression.improvements.len());
            }
        }
        
        Ok(result)
    }

    /// Run specific benchmark category
    pub async fn run_category_benchmarks(
        &mut self,
        category: BenchmarkCategory,
    ) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        match category {
            BenchmarkCategory::Server => {
                let mut benchmarks = ServerPerformanceBenchmarks::new(
                    self.config.clone(),
                    self.targets.clone(),
                )?;
                benchmarks.run_all_benchmarks().await
            }
            BenchmarkCategory::Client => {
                let mut benchmarks = ClientPerformanceBenchmarks::new(
                    self.config.clone(),
                    self.targets.clone(),
                )?;
                benchmarks.run_all_benchmarks().await
            }
            BenchmarkCategory::SDK => {
                let mut benchmarks = SdkPerformanceBenchmarks::new(
                    self.config.clone(),
                    self.targets.clone(),
                )?;
                benchmarks.run_all_benchmarks().await
            }
            BenchmarkCategory::CLI => {
                let mut benchmarks = CliPerformanceBenchmarks::new(
                    self.config.clone(),
                    self.targets.clone(),
                )?;
                benchmarks.run_all_benchmarks().await
            }
        }
    }

    /// Generate performance summary
    fn generate_performance_summary(
        &self,
        measurements: &[PerformanceMeasurement],
        total_duration: Duration,
    ) -> PerformanceSummary {
        let total_operations: usize = measurements.iter().map(|m| m.operation_count).sum();
        let total_errors: usize = measurements.iter().map(|m| m.error_count).sum();
        let overall_success_rate = if total_operations > 0 {
            ((total_operations - total_errors) as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        // Count targets met
        let mut targets_met = 0;
        let total_targets = measurements.len();
        
        for measurement in measurements {
            if measurement.meets_targets(&self.targets) {
                targets_met += 1;
            }
        }
        
        // Calculate performance score (0-100)
        let performance_score = self.calculate_performance_score(measurements);
        
        // Generate component summaries
        let component_summaries = self.generate_component_summaries(measurements);
        
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

    /// Calculate overall performance score
    fn calculate_performance_score(&self, measurements: &[PerformanceMeasurement]) -> f64 {
        if measurements.is_empty() {
            return 0.0;
        }
        
        let mut score_sum = 0.0;
        let mut weight_sum = 0.0;
        
        for measurement in measurements {
            let weight = self.get_measurement_weight(measurement);
            let individual_score = self.calculate_individual_score(measurement);
            
            score_sum += individual_score * weight;
            weight_sum += weight;
        }
        
        if weight_sum > 0.0 {
            (score_sum / weight_sum).min(100.0).max(0.0)
        } else {
            0.0
        }
    }

    /// Get weight for a measurement based on its importance
    fn get_measurement_weight(&self, measurement: &PerformanceMeasurement) -> f64 {
        match measurement.component.as_str() {
            "server" => 2.0, // Server performance is most critical
            "client" => 1.5,
            "sdk" => 1.0,
            "cli" => 0.8,
            _ => 1.0,
        }
    }

    /// Calculate individual performance score for a measurement
    fn calculate_individual_score(&self, measurement: &PerformanceMeasurement) -> f64 {
        let mut score = 100.0;
        
        // Deduct points for high latency
        let target_time = match measurement.component.as_str() {
            "server" => self.targets.server_verification_ms,
            "client" | "sdk" | "cli" => self.targets.client_signing_ms,
            _ => 10.0,
        };
        
        if measurement.avg_operation_time_ms > target_time {
            let latency_penalty = ((measurement.avg_operation_time_ms - target_time) / target_time) * 50.0;
            score -= latency_penalty.min(50.0);
        }
        
        // Deduct points for low throughput
        let min_throughput = self.targets.min_throughput_rps / 10.0; // Adjusted per component
        if measurement.operations_per_second < min_throughput {
            let throughput_penalty = ((min_throughput - measurement.operations_per_second) / min_throughput) * 30.0;
            score -= throughput_penalty.min(30.0);
        }
        
        // Deduct points for errors
        if measurement.error_count > 0 {
            let error_rate = (measurement.error_count as f64 / measurement.operation_count as f64) * 100.0;
            score -= error_rate.min(20.0);
        }
        
        // Deduct points for high memory usage
        if let Some(memory_usage) = measurement.memory_usage_bytes {
            if memory_usage > self.targets.max_memory_overhead_bytes {
                let memory_penalty = 10.0;
                score -= memory_penalty;
            }
        }
        
        score.max(0.0)
    }

    /// Generate component-specific summaries
    fn generate_component_summaries(&self, measurements: &[PerformanceMeasurement]) -> HashMap<String, ComponentSummary> {
        let mut summaries = HashMap::new();
        
        // Group measurements by component
        let mut component_groups: HashMap<String, Vec<&PerformanceMeasurement>> = HashMap::new();
        for measurement in measurements {
            component_groups.entry(measurement.component.clone())
                .or_insert_with(Vec::new)
                .push(measurement);
        }
        
        for (component_name, component_measurements) in component_groups {
            let measurement_count = component_measurements.len();
            
            // Calculate average performance score for component
            let mut score_sum = 0.0;
            for measurement in &component_measurements {
                score_sum += self.calculate_individual_score(measurement);
            }
            let avg_performance_score = score_sum / measurement_count as f64;
            
            // Find best and worst performing operations
            let best_operation = component_measurements.iter()
                .min_by(|a, b| a.avg_operation_time_ms.partial_cmp(&b.avg_operation_time_ms).unwrap())
                .map(|m| m.operation.clone())
                .unwrap_or_default();
            
            let worst_operation = component_measurements.iter()
                .max_by(|a, b| a.avg_operation_time_ms.partial_cmp(&b.avg_operation_time_ms).unwrap())
                .map(|m| m.operation.clone())
                .unwrap_or_default();
            
            // Analyze performance trends (simplified)
            let trends = self.analyze_component_trends(&component_measurements);
            
            let summary = ComponentSummary {
                component_name: component_name.clone(),
                measurement_count,
                avg_performance_score,
                best_operation,
                worst_operation,
                trends,
            };
            
            summaries.insert(component_name, summary);
        }
        
        summaries
    }

    /// Analyze performance trends for a component
    fn analyze_component_trends(&self, measurements: &[&PerformanceMeasurement]) -> PerformanceTrends {
        // Simplified trend analysis - in a real implementation, this would compare
        // against historical data to determine trends
        
        PerformanceTrends {
            latency_trend: TrendDirection::Stable,
            throughput_trend: TrendDirection::Stable,
            memory_trend: TrendDirection::Stable,
            cpu_trend: TrendDirection::Stable,
        }
    }

    /// Perform regression analysis against baseline data
    async fn perform_regression_analysis(
        &self,
        measurements: &[PerformanceMeasurement],
    ) -> Result<RegressionAnalysis, Box<dyn std::error::Error>> {
        // Load baseline data if available
        let baseline_data = if let Some(ref baseline_path) = self.config.baseline_data_path {
            self.load_baseline_data(baseline_path).await.ok()
        } else {
            None
        };
        
        let mut regressions = Vec::new();
        let mut improvements = Vec::new();
        
        if let Some(baseline) = baseline_data {
            // Compare current measurements against baseline
            for current in measurements {
                if let Some(baseline_measurement) = baseline.iter().find(|b| 
                    b.test_name == current.test_name && b.component == current.component
                ) {
                    let current_time = current.avg_operation_time_ms;
                    let baseline_time = baseline_measurement.avg_operation_time_ms;
                    
                    let change_percent = ((current_time - baseline_time) / baseline_time) * 100.0;
                    
                    if change_percent > 5.0 { // Regression threshold
                        regressions.push(super::PerformanceRegression {
                            test_name: current.test_name.clone(),
                            metric: "avg_operation_time_ms".to_string(),
                            baseline_value: baseline_time,
                            current_value: current_time,
                            regression_percent: change_percent,
                            severity: self.classify_regression_severity(change_percent),
                        });
                    } else if change_percent < -5.0 { // Improvement threshold
                        improvements.push(super::PerformanceImprovement {
                            test_name: current.test_name.clone(),
                            metric: "avg_operation_time_ms".to_string(),
                            baseline_value: baseline_time,
                            current_value: current_time,
                            improvement_percent: -change_percent,
                        });
                    }
                }
            }
        }
        
        let regression_score = if measurements.is_empty() {
            0.0
        } else {
            ((measurements.len() - regressions.len()) as f64 / measurements.len() as f64) * 100.0
        };
        
        Ok(RegressionAnalysis {
            baseline_timestamp: "current".to_string(), // Would be actual timestamp in real implementation
            regressions,
            improvements,
            regression_score,
        })
    }

    /// Classify regression severity
    fn classify_regression_severity(&self, regression_percent: f64) -> super::RegressionSeverity {
        if regression_percent >= 50.0 {
            super::RegressionSeverity::Critical
        } else if regression_percent >= 25.0 {
            super::RegressionSeverity::High
        } else if regression_percent >= 10.0 {
            super::RegressionSeverity::Medium
        } else if regression_percent >= 5.0 {
            super::RegressionSeverity::Low
        } else {
            super::RegressionSeverity::Minimal
        }
    }

    /// Load baseline performance data
    async fn load_baseline_data(&self, path: &str) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let baseline: Vec<PerformanceMeasurement> = serde_json::from_str(&content)?;
        Ok(baseline)
    }

    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        measurements: &[PerformanceMeasurement],
        summary: &PerformanceSummary,
    ) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        
        // Check for high latency issues
        for measurement in measurements {
            let target_time = match measurement.component.as_str() {
                "server" => self.targets.server_verification_ms,
                _ => self.targets.client_signing_ms,
            };
            
            if measurement.avg_operation_time_ms > target_time * 2.0 {
                recommendations.push(PerformanceRecommendation {
                    component: measurement.component.clone(),
                    priority: RecommendationPriority::High,
                    category: RecommendationCategory::Algorithm,
                    description: format!(
                        "Operation '{}' has high latency: {:.2}ms vs target {:.2}ms",
                        measurement.operation, measurement.avg_operation_time_ms, target_time
                    ),
                    recommendation: "Consider optimizing the cryptographic implementation or adding caching".to_string(),
                    expected_improvement: "20-50% latency reduction".to_string(),
                    effort_estimate: EffortEstimate::Medium,
                });
            }
        }
        
        // Check for low throughput
        for measurement in measurements {
            if measurement.operations_per_second < self.targets.min_throughput_rps / 10.0 {
                recommendations.push(PerformanceRecommendation {
                    component: measurement.component.clone(),
                    priority: RecommendationPriority::Medium,
                    category: RecommendationCategory::Concurrency,
                    description: format!(
                        "Low throughput detected: {:.1} ops/sec",
                        measurement.operations_per_second
                    ),
                    recommendation: "Consider implementing parallel processing or connection pooling".to_string(),
                    expected_improvement: "2-5x throughput increase".to_string(),
                    effort_estimate: EffortEstimate::High,
                });
            }
        }
        
        // Check for memory usage issues
        for measurement in measurements {
            if let Some(memory_usage) = measurement.memory_usage_bytes {
                if memory_usage > self.targets.max_memory_overhead_bytes {
                    recommendations.push(PerformanceRecommendation {
                        component: measurement.component.clone(),
                        priority: RecommendationPriority::Medium,
                        category: RecommendationCategory::Memory,
                        description: format!(
                            "High memory usage: {} bytes vs target {} bytes",
                            memory_usage, self.targets.max_memory_overhead_bytes
                        ),
                        recommendation: "Optimize data structures and implement memory pooling".to_string(),
                        expected_improvement: "30-60% memory reduction".to_string(),
                        effort_estimate: EffortEstimate::Medium,
                    });
                }
            }
        }
        
        // Check overall performance score
        if summary.performance_score < 70.0 {
            recommendations.push(PerformanceRecommendation {
                component: "overall".to_string(),
                priority: RecommendationPriority::Critical,
                category: RecommendationCategory::Configuration,
                description: format!(
                    "Overall performance score is low: {:.1}/100",
                    summary.performance_score
                ),
                recommendation: "Review and optimize the entire authentication pipeline".to_string(),
                expected_improvement: "Significant overall performance improvement".to_string(),
                effort_estimate: EffortEstimate::VeryHigh,
            });
        }
        
        // Add caching recommendations for frequently accessed operations
        let high_volume_ops: Vec<&PerformanceMeasurement> = measurements.iter()
            .filter(|m| m.operation_count > 1000)
            .collect();
        
        if !high_volume_ops.is_empty() {
            recommendations.push(PerformanceRecommendation {
                component: "caching".to_string(),
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Caching,
                description: "High-volume operations detected that could benefit from caching".to_string(),
                recommendation: "Implement signature and key caching for frequently used operations".to_string(),
                expected_improvement: "50-80% latency reduction for cached operations".to_string(),
                effort_estimate: EffortEstimate::Low,
            });
        }
        
        recommendations
    }

    /// Create benchmark metadata
    async fn create_benchmark_metadata(&self, execution_duration: Duration) -> BenchmarkMetadata {
        let environment = self.gather_environment_info().await;
        
        BenchmarkMetadata {
            timestamp: Utc::now().to_rfc3339(),
            environment,
            git_commit: self.get_git_commit().await,
            execution_duration,
            system_load: self.get_system_load().await,
        }
    }

    /// Gather environment information
    async fn gather_environment_info(&self) -> EnvironmentInfo {
        EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            cpu: self.get_cpu_info().unwrap_or_else(|| "Unknown".to_string()),
            memory_gb: self.get_memory_info().unwrap_or(0.0),
            rust_version: self.get_rust_version().unwrap_or_else(|| "Unknown".to_string()),
            nodejs_version: self.get_nodejs_version().await,
            python_version: self.get_python_version().await,
        }
    }

    /// Get CPU information
    fn get_cpu_info(&self) -> Option<String> {
        // Simplified CPU info - in a real implementation, this would use system APIs
        Some(format!("{} architecture", std::env::consts::ARCH))
    }

    /// Get memory information
    fn get_memory_info(&self) -> Option<f64> {
        // Simplified memory info - in a real implementation, this would use system APIs
        Some(8.0) // Assume 8GB for now
    }

    /// Get Rust version
    fn get_rust_version(&self) -> Option<String> {
        option_env!("CARGO_PKG_RUST_VERSION").map(|v| v.to_string())
            .or_else(|| Some("Unknown".to_string()))
    }

    /// Get Node.js version
    async fn get_nodejs_version(&self) -> Option<String> {
        if let Ok(output) = tokio::process::Command::new("node")
            .arg("--version")
            .output()
            .await
        {
            String::from_utf8(output.stdout).ok().map(|v| v.trim().to_string())
        } else {
            None
        }
    }

    /// Get Python version
    async fn get_python_version(&self) -> Option<String> {
        if let Ok(output) = tokio::process::Command::new("python3")
            .arg("--version")
            .output()
            .await
        {
            String::from_utf8(output.stdout).ok().map(|v| v.trim().to_string())
        } else {
            None
        }
    }

    /// Get Git commit hash
    async fn get_git_commit(&self) -> Option<String> {
        if let Ok(output) = tokio::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .await
        {
            String::from_utf8(output.stdout).ok().map(|v| v.trim().to_string())
        } else {
            None
        }
    }

    /// Get system load
    async fn get_system_load(&self) -> Option<f64> {
        // Simplified system load - in a real implementation, this would use system APIs
        Some(0.5) // Assume moderate load
    }

    /// Save benchmark results to file
    pub async fn save_results(
        &self,
        result: &PerformanceBenchmarkResult,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(result)?;
        fs::write(output_path, json_content)?;
        println!("💾 Benchmark results saved to: {}", output_path.display());
        Ok(())
    }

    /// Save baseline data for regression testing
    pub async fn save_baseline(
        &self,
        measurements: &[PerformanceMeasurement],
        baseline_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(measurements)?;
        fs::write(baseline_path, json_content)?;
        println!("📊 Baseline data saved to: {}", baseline_path.display());
        Ok(())
    }
}

/// Benchmark categories
#[derive(Debug, Clone)]
pub enum BenchmarkCategory {
    Server,
    Client,
    SDK,
    CLI,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_runner_creation() {
        let runner = PerformanceBenchmarkRunner::with_defaults();
        assert_eq!(runner.config.test_duration_seconds, 60);
    }

    #[tokio::test]
    async fn test_ci_cd_configuration() {
        let runner = PerformanceBenchmarkRunner::for_ci_cd();
        assert_eq!(runner.config.test_duration_seconds, 30);
        assert!(runner.config.enable_regression_testing);
    }

    #[tokio::test]
    async fn test_local_testing_configuration() {
        let runner = PerformanceBenchmarkRunner::for_local_testing();
        assert_eq!(runner.config.test_duration_seconds, 120);
        assert!(runner.config.enable_memory_profiling);
    }

    #[test]
    fn test_performance_score_calculation() {
        let runner = PerformanceBenchmarkRunner::with_defaults();
        
        let mut measurement = super::super::PerformanceMeasurement::new(
            "test".to_string(),
            "server".to_string(),
            "test_op".to_string(),
        );
        measurement.avg_operation_time_ms = 0.5; // Under target
        measurement.operations_per_second = 1000.0; // Good throughput
        measurement.error_count = 0;
        measurement.operation_count = 1000;
        
        let score = runner.calculate_individual_score(&measurement);
        assert!(score > 90.0);
    }

    #[test]
    fn test_measurement_weight() {
        let runner = PerformanceBenchmarkRunner::with_defaults();
        
        let server_measurement = super::super::PerformanceMeasurement::new(
            "test".to_string(),
            "server".to_string(),
            "test_op".to_string(),
        );
        
        let client_measurement = super::super::PerformanceMeasurement::new(
            "test".to_string(),
            "client".to_string(),
            "test_op".to_string(),
        );
        
        assert!(runner.get_measurement_weight(&server_measurement) > 
                runner.get_measurement_weight(&client_measurement));
    }
}