//! Performance Analysis Module
//!
//! This module provides comprehensive analysis of performance benchmarking data,
//! including statistical analysis, regression detection, trend analysis, and
//! optimization recommendations.

use super::{
    PerformanceMeasurement, PerformanceTargets, PerformanceBenchmarkConfig
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::time::Duration;

/// Performance analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisConfig {
    /// Regression threshold percentage (e.g., 10.0 for 10% regression)
    pub regression_threshold_percent: f64,
    /// Improvement threshold percentage (e.g., 5.0 for 5% improvement)
    pub improvement_threshold_percent: f64,
    /// Minimum sample size for statistical significance
    pub min_sample_size: usize,
    /// Confidence level for statistical tests
    pub confidence_level: f64,
    /// Enable trend analysis
    pub enable_trend_analysis: bool,
    /// Enable outlier detection
    pub enable_outlier_detection: bool,
    /// Historical data retention days
    pub historical_retention_days: u32,
}

impl Default for PerformanceAnalysisConfig {
    fn default() -> Self {
        Self {
            regression_threshold_percent: 10.0,
            improvement_threshold_percent: 5.0,
            min_sample_size: 30,
            confidence_level: 0.95,
            enable_trend_analysis: true,
            enable_outlier_detection: true,
            historical_retention_days: 90,
        }
    }
}

/// Statistical summary of performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    /// Number of measurements
    pub sample_size: usize,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Variance
    pub variance: f64,
    /// Median value
    pub median: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// 25th percentile
    pub p25: f64,
    /// 75th percentile
    pub p75: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Coefficient of variation (std_dev / mean)
    pub coefficient_of_variation: f64,
    /// Skewness (measure of asymmetry)
    pub skewness: f64,
    /// Kurtosis (measure of tail heaviness)
    pub kurtosis: f64,
}

/// Regression analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Benchmark name
    pub benchmark_name: String,
    /// Metric name
    pub metric_name: String,
    /// Previous value
    pub previous_value: f64,
    /// Current value
    pub current_value: f64,
    /// Change percentage (negative = improvement, positive = regression)
    pub change_percent: f64,
    /// Statistical significance (p-value)
    pub p_value: f64,
    /// Whether this is a significant regression
    pub is_regression: bool,
    /// Whether this is a significant improvement
    pub is_improvement: bool,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
    /// Severity level
    pub severity: RegressionSeverity,
}

/// Regression severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Critical,
    Major,
    Minor,
    Negligible,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Benchmark name
    pub benchmark_name: String,
    /// Metric name
    pub metric_name: String,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Trend strength (0-1, where 1 is strongest)
    pub trend_strength: f64,
    /// Linear regression slope
    pub slope: f64,
    /// R-squared value
    pub r_squared: f64,
    /// Predicted next value
    pub predicted_next_value: f64,
    /// Prediction confidence interval
    pub prediction_confidence: (f64, f64),
    /// Number of data points used
    pub data_points: usize,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
    Volatile,
}

/// Outlier detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    /// Benchmark name
    pub benchmark_name: String,
    /// Metric name
    pub metric_name: String,
    /// Detected outliers (value, z-score)
    pub outliers: Vec<(f64, f64)>,
    /// Outlier detection method used
    pub detection_method: OutlierDetectionMethod,
    /// Outlier threshold
    pub threshold: f64,
    /// Impact on statistics
    pub impact_on_mean: f64,
    /// Impact on standard deviation
    pub impact_on_std_dev: f64,
}

/// Outlier detection methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutlierDetectionMethod {
    ZScore,
    IQR,
    ModifiedZScore,
}

/// Performance target compliance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetComplianceAnalysis {
    /// Benchmark name
    pub benchmark_name: String,
    /// Target compliance by metric
    pub metric_compliance: HashMap<String, TargetCompliance>,
    /// Overall compliance score (0-100)
    pub overall_score: f64,
    /// Compliance status
    pub status: ComplianceStatus,
    /// Failed targets with details
    pub failed_targets: Vec<FailedTarget>,
}

/// Individual target compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetCompliance {
    /// Metric name
    pub metric_name: String,
    /// Target value
    pub target_value: f64,
    /// Actual value
    pub actual_value: f64,
    /// Compliance percentage (0-100)
    pub compliance_percent: f64,
    /// Whether target is met
    pub is_compliant: bool,
    /// Margin (how much better/worse than target)
    pub margin: f64,
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    FullyCompliant,
    MostlyCompliant,
    PartiallyCompliant,
    NonCompliant,
}

/// Failed target details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTarget {
    /// Metric name
    pub metric_name: String,
    /// Target value
    pub target_value: f64,
    /// Actual value
    pub actual_value: f64,
    /// Failure margin
    pub failure_margin: f64,
    /// Recommended action
    pub recommended_action: String,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Priority level
    pub priority: RecommendationPriority,
    /// Category
    pub category: RecommendationCategory,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Expected impact
    pub expected_impact: String,
    /// Implementation effort
    pub implementation_effort: ImplementationEffort,
    /// Related metrics
    pub related_metrics: Vec<String>,
    /// Technical details
    pub technical_details: Vec<String>,
}

/// Recommendation priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Recommendation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Algorithm,
    Caching,
    Concurrency,
    Database,
    Network,
    Memory,
    CPU,
    Infrastructure,
    Configuration,
}

/// Implementation effort estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,  // Hours
    Low,      // Days
    Medium,   // Weeks
    High,     // Months
}

/// Complete performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisResult {
    /// Analysis timestamp
    pub timestamp: std::time::SystemTime,
    /// Configuration used
    pub config: PerformanceAnalysisConfig,
    /// Statistical summaries by benchmark
    pub statistics: HashMap<String, PerformanceStatistics>,
    /// Regression analysis results
    pub regressions: Vec<RegressionAnalysis>,
    /// Trend analysis results
    pub trends: Vec<TrendAnalysis>,
    /// Outlier analysis results
    pub outliers: Vec<OutlierAnalysis>,
    /// Target compliance analysis
    pub target_compliance: Vec<TargetComplianceAnalysis>,
    /// Optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
    /// Overall performance score (0-100)
    pub overall_score: f64,
    /// Summary insights
    pub insights: Vec<String>,
}

/// Performance analysis engine
pub struct PerformanceAnalyzer {
    config: PerformanceAnalysisConfig,
    historical_data: BTreeMap<std::time::SystemTime, Vec<PerformanceMeasurement>>,
}

impl PerformanceAnalyzer {
    /// Create new performance analyzer
    pub fn new(config: PerformanceAnalysisConfig) -> Self {
        Self {
            config,
            historical_data: BTreeMap::new(),
        }
    }

    /// Add historical data point
    pub fn add_historical_data(&mut self, timestamp: std::time::SystemTime, measurements: Vec<PerformanceMeasurement>) {
        self.historical_data.insert(timestamp, measurements);
        self.cleanup_old_data();
    }

    /// Perform comprehensive performance analysis
    pub fn analyze(&self, current_measurements: &[PerformanceMeasurement], targets: &PerformanceTargets) -> PerformanceAnalysisResult {
        println!("🔍 Performing comprehensive performance analysis");

        let mut result = PerformanceAnalysisResult {
            timestamp: std::time::SystemTime::now(),
            config: self.config.clone(),
            statistics: HashMap::new(),
            regressions: Vec::new(),
            trends: Vec::new(),
            outliers: Vec::new(),
            target_compliance: Vec::new(),
            recommendations: Vec::new(),
            overall_score: 0.0,
            insights: Vec::new(),
        };

        // Group measurements by benchmark name
        let grouped_measurements = self.group_measurements_by_benchmark(current_measurements);

        // Calculate statistics for each benchmark
        for (benchmark_name, measurements) in &grouped_measurements {
            if let Some(stats) = self.calculate_statistics(measurements) {
                result.statistics.insert(benchmark_name.clone(), stats);
            }
        }

        // Perform regression analysis
        result.regressions = self.detect_regressions(&grouped_measurements);

        // Perform trend analysis
        if self.config.enable_trend_analysis {
            result.trends = self.analyze_trends(&grouped_measurements);
        }

        // Perform outlier detection
        if self.config.enable_outlier_detection {
            result.outliers = self.detect_outliers(&grouped_measurements);
        }

        // Analyze target compliance
        result.target_compliance = self.analyze_target_compliance(&grouped_measurements, targets);

        // Generate optimization recommendations
        result.recommendations = self.generate_recommendations(&result);

        // Calculate overall performance score
        result.overall_score = self.calculate_overall_score(&result);

        // Generate insights
        result.insights = self.generate_insights(&result);

        println!("✅ Performance analysis completed - Score: {:.1}/100", result.overall_score);

        result
    }

    /// Calculate statistical summary
    fn calculate_statistics(&self, measurements: &[PerformanceMeasurement]) -> Option<PerformanceStatistics> {
        if measurements.is_empty() {
            return None;
        }

        // Extract timing values
        let mut values: Vec<f64> = measurements.iter()
            .map(|m| m.avg_operation_time_ms)
            .collect();

        if values.is_empty() {
            return None;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = values.len();
        let mean = values.iter().sum::<f64>() / n as f64;
        
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / n as f64;
        
        let std_dev = variance.sqrt();
        
        let median = if n % 2 == 0 {
            (values[n / 2 - 1] + values[n / 2]) / 2.0
        } else {
            values[n / 2]
        };

        let min = values[0];
        let max = values[n - 1];
        let p25 = values[n / 4];
        let p75 = values[(3 * n) / 4];
        let p90 = values[(9 * n) / 10];
        let p95 = values[(95 * n) / 100];
        let p99 = values[(99 * n) / 100];

        let coefficient_of_variation = if mean != 0.0 { std_dev / mean } else { 0.0 };

        // Calculate skewness
        let skewness = if std_dev != 0.0 {
            values.iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>() / n as f64
        } else {
            0.0
        };

        // Calculate kurtosis
        let kurtosis = if std_dev != 0.0 {
            values.iter()
                .map(|x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>() / n as f64 - 3.0
        } else {
            0.0
        };

        Some(PerformanceStatistics {
            sample_size: n,
            mean,
            std_dev,
            variance,
            median,
            min,
            max,
            p25,
            p75,
            p90,
            p95,
            p99,
            coefficient_of_variation,
            skewness,
            kurtosis,
        })
    }

    /// Detect performance regressions
    fn detect_regressions(&self, grouped_measurements: &HashMap<String, Vec<PerformanceMeasurement>>) -> Vec<RegressionAnalysis> {
        let mut regressions = Vec::new();

        for (benchmark_name, current_measurements) in grouped_measurements {
            if let Some(historical_measurements) = self.get_recent_historical_data(benchmark_name) {
                if let Some(regression) = self.compare_with_baseline(benchmark_name, current_measurements, &historical_measurements) {
                    regressions.push(regression);
                }
            }
        }

        regressions
    }

    /// Compare current measurements with baseline
    fn compare_with_baseline(
        &self,
        benchmark_name: &str,
        current: &[PerformanceMeasurement],
        historical: &[PerformanceMeasurement],
    ) -> Option<RegressionAnalysis> {
        if current.is_empty() || historical.is_empty() {
            return None;
        }

        let current_mean = current.iter().map(|m| m.avg_operation_time_ms).sum::<f64>() / current.len() as f64;
        let historical_mean = historical.iter().map(|m| m.avg_operation_time_ms).sum::<f64>() / historical.len() as f64;

        let change_percent = ((current_mean - historical_mean) / historical_mean) * 100.0;

        // Perform t-test for statistical significance
        let p_value = self.perform_t_test(current, historical);

        let is_regression = change_percent > self.config.regression_threshold_percent && p_value < (1.0 - self.config.confidence_level);
        let is_improvement = change_percent < -self.config.improvement_threshold_percent && p_value < (1.0 - self.config.confidence_level);

        let severity = self.calculate_regression_severity(change_percent);

        // Calculate confidence interval
        let confidence_interval = self.calculate_confidence_interval(current, historical);

        Some(RegressionAnalysis {
            benchmark_name: benchmark_name.to_string(),
            metric_name: "avg_operation_time_ms".to_string(),
            previous_value: historical_mean,
            current_value: current_mean,
            change_percent,
            p_value,
            is_regression,
            is_improvement,
            confidence_interval,
            severity,
        })
    }

    /// Perform t-test between two samples
    fn perform_t_test(&self, current: &[PerformanceMeasurement], historical: &[PerformanceMeasurement]) -> f64 {
        let current_values: Vec<f64> = current.iter().map(|m| m.avg_operation_time_ms).collect();
        let historical_values: Vec<f64> = historical.iter().map(|m| m.avg_operation_time_ms).collect();

        if current_values.len() < 2 || historical_values.len() < 2 {
            return 1.0; // No significance
        }

        let mean1 = current_values.iter().sum::<f64>() / current_values.len() as f64;
        let mean2 = historical_values.iter().sum::<f64>() / historical_values.len() as f64;

        let var1 = current_values.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (current_values.len() - 1) as f64;
        let var2 = historical_values.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (historical_values.len() - 1) as f64;

        let pooled_std = ((var1 / current_values.len() as f64) + (var2 / historical_values.len() as f64)).sqrt();

        if pooled_std == 0.0 {
            return 1.0;
        }

        let t_stat = (mean1 - mean2).abs() / pooled_std;
        
        // Simplified p-value approximation
        let df = current_values.len() + historical_values.len() - 2;
        let p_value = 2.0 * (1.0 - self.student_t_cdf(t_stat, df as f64));

        p_value.max(0.001).min(1.0)
    }

    /// Simplified Student's t-distribution CDF approximation
    fn student_t_cdf(&self, t: f64, df: f64) -> f64 {
        // Very simplified approximation - in practice, use a proper statistical library
        let x = t / (t * t + df).sqrt();
        0.5 + 0.5 * x * (1.0 - x * x / 3.0)
    }

    /// Calculate regression severity
    fn calculate_regression_severity(&self, change_percent: f64) -> RegressionSeverity {
        let abs_change = change_percent.abs();
        if abs_change >= 50.0 {
            RegressionSeverity::Critical
        } else if abs_change >= 25.0 {
            RegressionSeverity::Major
        } else if abs_change >= 10.0 {
            RegressionSeverity::Minor
        } else {
            RegressionSeverity::Negligible
        }
    }

    /// Calculate confidence interval
    fn calculate_confidence_interval(&self, current: &[PerformanceMeasurement], _historical: &[PerformanceMeasurement]) -> (f64, f64) {
        let values: Vec<f64> = current.iter().map(|m| m.avg_operation_time_ms).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std_dev = (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64).sqrt();
        let margin = 1.96 * std_dev / (values.len() as f64).sqrt(); // 95% CI
        
        (mean - margin, mean + margin)
    }

    /// Analyze trends over time
    fn analyze_trends(&self, grouped_measurements: &HashMap<String, Vec<PerformanceMeasurement>>) -> Vec<TrendAnalysis> {
        let mut trends = Vec::new();

        for (benchmark_name, _current_measurements) in grouped_measurements {
            if let Some(trend) = self.calculate_trend(benchmark_name) {
                trends.push(trend);
            }
        }

        trends
    }

    /// Calculate trend for a specific benchmark
    fn calculate_trend(&self, benchmark_name: &str) -> Option<TrendAnalysis> {
        let historical_points = self.get_historical_time_series(benchmark_name);
        
        if historical_points.len() < 3 {
            return None;
        }

        let (slope, r_squared) = self.linear_regression(&historical_points);
        
        let trend_direction = if slope.abs() < 0.01 {
            TrendDirection::Stable
        } else if slope < 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Degrading
        };

        let trend_strength = r_squared;
        
        // Predict next value
        let last_x = historical_points.len() as f64;
        let predicted_next_value = self.predict_value(&historical_points, slope, last_x + 1.0);
        
        // Calculate prediction confidence (simplified)
        let residuals: Vec<f64> = historical_points.iter().enumerate()
            .map(|(i, &(_, y))| y - self.predict_value(&historical_points, slope, i as f64))
            .collect();
        
        let residual_std = (residuals.iter().map(|r| r.powi(2)).sum::<f64>() / residuals.len() as f64).sqrt();
        let prediction_confidence = (predicted_next_value - 2.0 * residual_std, predicted_next_value + 2.0 * residual_std);

        Some(TrendAnalysis {
            benchmark_name: benchmark_name.to_string(),
            metric_name: "avg_operation_time_ms".to_string(),
            trend_direction,
            trend_strength,
            slope,
            r_squared,
            predicted_next_value,
            prediction_confidence,
            data_points: historical_points.len(),
        })
    }

    /// Perform linear regression
    fn linear_regression(&self, points: &[(f64, f64)]) -> (f64, f64) {
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();
        let sum_y2: f64 = points.iter().map(|(_, y)| y * y).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        
        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = points.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
        let ss_res: f64 = points.iter().map(|(x, y)| {
            let predicted = self.predict_value(points, slope, *x);
            (y - predicted).powi(2)
        }).sum();
        
        let r_squared = if ss_tot > 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

        (slope, r_squared)
    }

    /// Predict value using linear regression
    fn predict_value(&self, points: &[(f64, f64)], slope: f64, x: f64) -> f64 {
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        
        let intercept = mean_y - slope * mean_x;
        slope * x + intercept
    }

    /// Detect outliers in measurements
    fn detect_outliers(&self, grouped_measurements: &HashMap<String, Vec<PerformanceMeasurement>>) -> Vec<OutlierAnalysis> {
        let mut outliers = Vec::new();

        for (benchmark_name, measurements) in grouped_measurements {
            if let Some(outlier_analysis) = self.detect_outliers_in_benchmark(benchmark_name, measurements) {
                outliers.push(outlier_analysis);
            }
        }

        outliers
    }

    /// Detect outliers in a specific benchmark
    fn detect_outliers_in_benchmark(&self, benchmark_name: &str, measurements: &[PerformanceMeasurement]) -> Option<OutlierAnalysis> {
        let values: Vec<f64> = measurements.iter().map(|m| m.avg_operation_time_ms).collect();
        
        if values.len() < 10 {
            return None; // Need sufficient data for outlier detection
        }

        let detected_outliers = self.detect_outliers_z_score(&values);
        
        if detected_outliers.is_empty() {
            return None;
        }

        // Calculate impact
        let mean_with_outliers = values.iter().sum::<f64>() / values.len() as f64;
        let filtered_values: Vec<f64> = values.iter().cloned()
            .filter(|v| !detected_outliers.iter().any(|(outlier_val, _)| (v - outlier_val).abs() < 0.001))
            .collect();
        
        let mean_without_outliers = if !filtered_values.is_empty() {
            filtered_values.iter().sum::<f64>() / filtered_values.len() as f64
        } else {
            mean_with_outliers
        };

        let impact_on_mean = ((mean_with_outliers - mean_without_outliers) / mean_without_outliers).abs() * 100.0;
        
        let std_dev_with = (values.iter().map(|x| (x - mean_with_outliers).powi(2)).sum::<f64>() / values.len() as f64).sqrt();
        let std_dev_without = if filtered_values.len() > 1 {
            (filtered_values.iter().map(|x| (x - mean_without_outliers).powi(2)).sum::<f64>() / filtered_values.len() as f64).sqrt()
        } else {
            std_dev_with
        };
        
        let impact_on_std_dev = ((std_dev_with - std_dev_without) / std_dev_without).abs() * 100.0;

        Some(OutlierAnalysis {
            benchmark_name: benchmark_name.to_string(),
            metric_name: "avg_operation_time_ms".to_string(),
            outliers: detected_outliers,
            detection_method: OutlierDetectionMethod::ZScore,
            threshold: 3.0,
            impact_on_mean,
            impact_on_std_dev,
        })
    }

    /// Detect outliers using Z-score method
    fn detect_outliers_z_score(&self, values: &[f64]) -> Vec<(f64, f64)> {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std_dev = (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt();
        
        if std_dev == 0.0 {
            return Vec::new();
        }

        let threshold = 3.0; // Standard z-score threshold
        
        values.iter()
            .map(|&value| {
                let z_score = (value - mean).abs() / std_dev;
                (value, z_score)
            })
            .filter(|(_, z_score)| *z_score > threshold)
            .collect()
    }

    /// Analyze target compliance
    fn analyze_target_compliance(&self, grouped_measurements: &HashMap<String, Vec<PerformanceMeasurement>>, targets: &PerformanceTargets) -> Vec<TargetComplianceAnalysis> {
        let mut compliance_results = Vec::new();

        for (benchmark_name, measurements) in grouped_measurements {
            let compliance = self.check_target_compliance(benchmark_name, measurements, targets);
            compliance_results.push(compliance);
        }

        compliance_results
    }

    /// Check target compliance for a benchmark
    fn check_target_compliance(&self, benchmark_name: &str, measurements: &[PerformanceMeasurement], targets: &PerformanceTargets) -> TargetComplianceAnalysis {
        let mut metric_compliance = HashMap::new();
        let mut failed_targets = Vec::new();
        
        if let Some(measurement) = measurements.first() {
            // Check average operation time
            let avg_time_compliance = TargetCompliance {
                metric_name: "avg_operation_time_ms".to_string(),
                target_value: targets.max_avg_operation_time_ms,
                actual_value: measurement.avg_operation_time_ms,
                compliance_percent: if measurement.avg_operation_time_ms <= targets.max_avg_operation_time_ms {
                    100.0
                } else {
                    (targets.max_avg_operation_time_ms / measurement.avg_operation_time_ms) * 100.0
                },
                is_compliant: measurement.avg_operation_time_ms <= targets.max_avg_operation_time_ms,
                margin: measurement.avg_operation_time_ms - targets.max_avg_operation_time_ms,
            };

            if !avg_time_compliance.is_compliant {
                failed_targets.push(FailedTarget {
                    metric_name: "avg_operation_time_ms".to_string(),
                    target_value: targets.max_avg_operation_time_ms,
                    actual_value: measurement.avg_operation_time_ms,
                    failure_margin: avg_time_compliance.margin,
                    recommended_action: "Optimize algorithm or increase resources".to_string(),
                });
            }

            metric_compliance.insert("avg_operation_time_ms".to_string(), avg_time_compliance);

            // Check throughput
            let throughput_compliance = TargetCompliance {
                metric_name: "operations_per_second".to_string(),
                target_value: targets.min_operations_per_second,
                actual_value: measurement.operations_per_second,
                compliance_percent: if measurement.operations_per_second >= targets.min_operations_per_second {
                    100.0
                } else {
                    (measurement.operations_per_second / targets.min_operations_per_second) * 100.0
                },
                is_compliant: measurement.operations_per_second >= targets.min_operations_per_second,
                margin: measurement.operations_per_second - targets.min_operations_per_second,
            };

            if !throughput_compliance.is_compliant {
                failed_targets.push(FailedTarget {
                    metric_name: "operations_per_second".to_string(),
                    target_value: targets.min_operations_per_second,
                    actual_value: measurement.operations_per_second,
                    failure_margin: throughput_compliance.margin.abs(),
                    recommended_action: "Improve concurrency or reduce per-operation overhead".to_string(),
                });
            }

            metric_compliance.insert("operations_per_second".to_string(), throughput_compliance);
        }

        let overall_score = if metric_compliance.is_empty() {
            0.0
        } else {
            metric_compliance.values().map(|c| c.compliance_percent).sum::<f64>() / metric_compliance.len() as f64
        };

        let status = match overall_score {
            score if score >= 95.0 => ComplianceStatus::FullyCompliant,
            score if score >= 80.0 => ComplianceStatus::MostlyCompliant,
            score if score >= 50.0 => ComplianceStatus::PartiallyCompliant,
            _ => ComplianceStatus::NonCompliant,
        };

        TargetComplianceAnalysis {
            benchmark_name: benchmark_name.to_string(),
            metric_compliance,
            overall_score,
            status,
            failed_targets,
        }
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, analysis_result: &PerformanceAnalysisResult) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze regressions for recommendations
        for regression in &analysis_result.regressions {
            if regression.is_regression {
                let recommendation = match regression.severity {
                    RegressionSeverity::Critical => OptimizationRecommendation {
                        priority: RecommendationPriority::Critical,
                        category: RecommendationCategory::Algorithm,
                        title: format!("Critical Performance Regression in {}", regression.benchmark_name),
                        description: format!(
                            "Performance has degraded by {:.1}% in {}. Immediate investigation required.",
                            regression.change_percent, regression.benchmark_name
                        ),
                        expected_impact: "Restore performance to baseline levels".to_string(),
                        implementation_effort: ImplementationEffort::High,
                        related_metrics: vec![regression.metric_name.clone()],
                        technical_details: vec![
                            "Profile code for bottlenecks".to_string(),
                            "Review recent changes".to_string(),
                            "Check for resource contention".to_string(),
                        ],
                    },
                    RegressionSeverity::Major => OptimizationRecommendation {
                        priority: RecommendationPriority::High,
                        category: RecommendationCategory::Algorithm,
                        title: format!("Major Performance Regression in {}", regression.benchmark_name),
                        description: format!(
                            "Performance has degraded by {:.1}% in {}. Should be addressed soon.",
                            regression.change_percent, regression.benchmark_name
                        ),
                        expected_impact: "Significant performance improvement".to_string(),
                        implementation_effort: ImplementationEffort::Medium,
                        related_metrics: vec![regression.metric_name.clone()],
                        technical_details: vec![
                            "Analyze algorithm complexity".to_string(),
                            "Consider caching strategies".to_string(),
                        ],
                    },
                    _ => continue,
                };
                recommendations.push(recommendation);
            }
        }

        // Analyze failed targets for recommendations
        for compliance in &analysis_result.target_compliance {
            for failed_target in &compliance.failed_targets {
                let recommendation = OptimizationRecommendation {
                    priority: RecommendationPriority::High,
                    category: RecommendationCategory::Configuration,
                    title: format!("Target Compliance Failure: {}", failed_target.metric_name),
                    description: format!(
                        "Metric {} is {:.1}% over target in {}",
                        failed_target.metric_name,
                        (failed_target.failure_margin / failed_target.target_value) * 100.0,
                        compliance.benchmark_name
                    ),
                    expected_impact: failed_target.recommended_action.clone(),
                    implementation_effort: ImplementationEffort::Medium,
                    related_metrics: vec![failed_target.metric_name.clone()],
                    technical_details: vec![
                        "Review performance requirements".to_string(),
                        "Optimize implementation".to_string(),
                    ],
                };
                recommendations.push(recommendation);
            }
        }

        // General recommendations based on statistics
        for (benchmark_name, stats) in &analysis_result.statistics {
            if stats.coefficient_of_variation > 0.5 {
                recommendations.push(OptimizationRecommendation {
                    priority: RecommendationPriority::Medium,
                    category: RecommendationCategory::Concurrency,
                    title: format!("High Performance Variability in {}", benchmark_name),
                    description: "Performance measurements show high variability, indicating inconsistent behavior".to_string(),
                    expected_impact: "More consistent performance".to_string(),
                    implementation_effort: ImplementationEffort::Low,
                    related_metrics: vec!["coefficient_of_variation".to_string()],
                    technical_details: vec![
                        "Reduce resource contention".to_string(),
                        "Implement connection pooling".to_string(),
                        "Optimize garbage collection".to_string(),
                    ],
                });
            }
        }

        recommendations
    }

    /// Calculate overall performance score
    fn calculate_overall_score(&self, analysis_result: &PerformanceAnalysisResult) -> f64 {
        let mut score_components = Vec::new();

        // Target compliance score
        if !analysis_result.target_compliance.is_empty() {
            let avg_compliance = analysis_result.target_compliance.iter()
                .map(|c| c.overall_score)
                .sum::<f64>() / analysis_result.target_compliance.len() as f64;
            score_components.push(avg_compliance * 0.4); // 40% weight
        }

        // Regression penalty
        let regression_penalty = analysis_result.regressions.iter()
            .map(|r| match r.severity {
                RegressionSeverity::Critical => 30.0,
                RegressionSeverity::Major => 20.0,
                RegressionSeverity::Minor => 10.0,
                RegressionSeverity::Negligible => 2.0,
            })
            .sum::<f64>();
        
        score_components.push((100.0 - regression_penalty.min(50.0)) * 0.3); // 30% weight

        // Stability score (based on coefficient of variation)
        if !analysis_result.statistics.is_empty() {
            let avg_cv = analysis_result.statistics.values()
                .map(|s| s.coefficient_of_variation)
                .sum::<f64>() / analysis_result.statistics.len() as f64;
            let stability_score = (1.0 - avg_cv.min(1.0)) * 100.0;
            score_components.push(stability_score * 0.2); // 20% weight
        }

        // Trend score
        if !analysis_result.trends.is_empty() {
            let improving_trends = analysis_result.trends.iter()
                .filter(|t| t.trend_direction == TrendDirection::Improving)
                .count();
            let total_trends = analysis_result.trends.len();
            let trend_score = (improving_trends as f64 / total_trends as f64) * 100.0;
            score_components.push(trend_score * 0.1); // 10% weight
        }

        if score_components.is_empty() {
            50.0 // Neutral score if no data
        } else {
            score_components.iter().sum::<f64>() / score_components.len() as f64
        }
    }

    /// Generate insights from analysis
    fn generate_insights(&self, analysis_result: &PerformanceAnalysisResult) -> Vec<String> {
        let mut insights = Vec::new();

        // Overall performance assessment
        match analysis_result.overall_score {
            score if score >= 90.0 => insights.push("🟢 Overall performance is excellent with minimal issues".to_string()),
            score if score >= 70.0 => insights.push("🟡 Overall performance is good but has room for improvement".to_string()),
            score if score >= 50.0 => insights.push("🟠 Overall performance is moderate with several concerns".to_string()),
            _ => insights.push("🔴 Overall performance is poor and requires immediate attention".to_string()),
        }

        // Regression insights
        let critical_regressions = analysis_result.regressions.iter()
            .filter(|r| r.severity == RegressionSeverity::Critical)
            .count();
        
        if critical_regressions > 0 {
            insights.push(format!("🚨 {} critical performance regression(s) detected", critical_regressions));
        }

        // Target compliance insights
        let non_compliant = analysis_result.target_compliance.iter()
            .filter(|c| c.status == ComplianceStatus::NonCompliant)
            .count();
        
        if non_compliant > 0 {
            insights.push(format!("⚠️ {} benchmark(s) not meeting performance targets", non_compliant));
        }

        // Trend insights
        let degrading_trends = analysis_result.trends.iter()
            .filter(|t| t.trend_direction == TrendDirection::Degrading)
            .count();
        
        if degrading_trends > 0 {
            insights.push(format!("📉 {} benchmark(s) showing degrading performance trends", degrading_trends));
        }

        // Outlier insights
        let benchmarks_with_outliers = analysis_result.outliers.len();
        if benchmarks_with_outliers > 0 {
            insights.push(format!("🎯 {} benchmark(s) have performance outliers affecting consistency", benchmarks_with_outliers));
        }

        // Recommendations insight
        let high_priority_recommendations = analysis_result.recommendations.iter()
            .filter(|r| matches!(r.priority, RecommendationPriority::Critical | RecommendationPriority::High))
            .count();
        
        if high_priority_recommendations > 0 {
            insights.push(format!("🔧 {} high-priority optimization recommendations available", high_priority_recommendations));
        }

        if insights.len() == 1 && analysis_result.overall_score >= 90.0 {
            insights.push("✨ Performance is well-optimized across all measured components".to_string());
        }

        insights
    }

    // Helper methods

    /// Group measurements by benchmark name
    fn group_measurements_by_benchmark(&self, measurements: &[PerformanceMeasurement]) -> HashMap<String, Vec<PerformanceMeasurement>> {
        let mut grouped = HashMap::new();
        
        for measurement in measurements {
            grouped.entry(measurement.benchmark_name.clone())
                .or_insert_with(Vec::new)
                .push(measurement.clone());
        }
        
        grouped
    }

    /// Get recent historical data for a benchmark
    fn get_recent_historical_data(&self, benchmark_name: &str) -> Option<Vec<PerformanceMeasurement>> {
        let recent_cutoff = std::time::SystemTime::now() - Duration::from_secs(7 * 24 * 3600); // 7 days ago
        
        let mut historical_measurements = Vec::new();
        
        for (timestamp, measurements) in &self.historical_data {
            if *timestamp >= recent_cutoff {
                for measurement in measurements {
                    if measurement.benchmark_name == benchmark_name {
                        historical_measurements.push(measurement.clone());
                    }
                }
            }
        }
        
        if historical_measurements.is_empty() {
            None
        } else {
            Some(historical_measurements)
        }
    }

    /// Get historical time series for trend analysis
    fn get_historical_time_series(&self, benchmark_name: &str) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        
        for (timestamp, measurements) in &self.historical_data {
            let avg_time = measurements.iter()
                .filter(|m| m.benchmark_name == benchmark_name)
                .map(|m| m.avg_operation_time_ms)
                .sum::<f64>() / measurements.len() as f64;
            
            if avg_time > 0.0 {
                let time_since_epoch = timestamp.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                points.push((time_since_epoch, avg_time));
            }
        }
        
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Convert to sequential x values for easier regression
        points.into_iter()
            .enumerate()
            .map(|(i, (_, y))| (i as f64, y))
            .collect()
    }

    /// Clean up old historical data
    fn cleanup_old_data(&mut self) {
        let cutoff = std::time::SystemTime::now() - Duration::from_secs(self.config.historical_retention_days as u64 * 24 * 3600);
        
        self.historical_data.retain(|timestamp, _| *timestamp >= cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer_creation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config);
        
        assert!(analyzer.historical_data.is_empty());
    }

    #[test]
    fn test_statistics_calculation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config);
        
        let measurements = vec![
            PerformanceMeasurement {
                benchmark_name: "test".to_string(),
                category: "test".to_string(),
                operation_type: "test".to_string(),
                avg_operation_time_ms: 10.0,
                ..Default::default()
            },
            PerformanceMeasurement {
                benchmark_name: "test".to_string(),
                category: "test".to_string(),
                operation_type: "test".to_string(),
                avg_operation_time_ms: 20.0,
                ..Default::default()
            },
        ];
        
        let stats = analyzer.calculate_statistics(&measurements);
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert_eq!(stats.sample_size, 2);
        assert_eq!(stats.mean, 15.0);
    }

    #[test]
    fn test_regression_severity_calculation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config);
        
        assert_eq!(analyzer.calculate_regression_severity(60.0), RegressionSeverity::Critical);
        assert_eq!(analyzer.calculate_regression_severity(30.0), RegressionSeverity::Major);
        assert_eq!(analyzer.calculate_regression_severity(15.0), RegressionSeverity::Minor);
        assert_eq!(analyzer.calculate_regression_severity(5.0), RegressionSeverity::Negligible);
    }
}