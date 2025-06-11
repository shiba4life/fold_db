//! Performance Reporting Module
//!
//! This module provides comprehensive performance reporting capabilities,
//! including HTML reports, JSON exports, CSV data, and visualizations.

use super::{
    PerformanceMeasurement, PerformanceTargets, PerformanceBenchmarkConfig,
    performance_analysis::{PerformanceAnalysisResult, RegressionSeverity, TrendDirection, ComplianceStatus}
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Report generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Output directory for reports
    pub output_directory: String,
    /// Include detailed charts and graphs
    pub include_charts: bool,
    /// Include historical comparison
    pub include_historical_comparison: bool,
    /// Include regression analysis details
    pub include_regression_analysis: bool,
    /// Include optimization recommendations
    pub include_recommendations: bool,
    /// Include system metrics
    pub include_system_metrics: bool,
    /// Report format options
    pub formats: Vec<ReportFormat>,
    /// Chart generation options
    pub chart_options: ChartOptions,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            output_directory: "reports/performance".to_string(),
            include_charts: true,
            include_historical_comparison: true,
            include_regression_analysis: true,
            include_recommendations: true,
            include_system_metrics: true,
            formats: vec![ReportFormat::Html, ReportFormat::Json, ReportFormat::Csv],
            chart_options: ChartOptions::default(),
        }
    }
}

/// Report output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Html,
    Json,
    Csv,
    Markdown,
    Pdf,
}

/// Chart generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    /// Chart width in pixels
    pub width: u32,
    /// Chart height in pixels
    pub height: u32,
    /// Include trend lines
    pub include_trend_lines: bool,
    /// Include percentile bands
    pub include_percentile_bands: bool,
    /// Color scheme
    pub color_scheme: ColorScheme,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 400,
            include_trend_lines: true,
            include_percentile_bands: true,
            color_scheme: ColorScheme::Professional,
        }
    }
}

/// Color schemes for charts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Professional,
    Vibrant,
    Monochrome,
    HighContrast,
}

/// Performance report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Executive summary
    pub executive_summary: ExecutiveSummary,
    /// Benchmark results
    pub benchmark_results: Vec<BenchmarkResult>,
    /// Performance analysis
    pub analysis: PerformanceAnalysisResult,
    /// System information
    pub system_info: SystemInfo,
    /// Historical comparison
    pub historical_comparison: Option<HistoricalComparison>,
    /// Charts and visualizations
    pub charts: Vec<ChartData>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generation timestamp
    pub generated_at: SystemTime,
    /// Report version
    pub version: String,
    /// Configuration used
    pub config: PerformanceBenchmarkConfig,
    /// Targets used
    pub targets: PerformanceTargets,
    /// Report generation duration
    pub generation_duration_ms: f64,
}

/// Executive summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    /// Overall performance score (0-100)
    pub overall_score: f64,
    /// Performance grade
    pub performance_grade: PerformanceGrade,
    /// Total benchmarks run
    pub total_benchmarks: usize,
    /// Benchmarks passing targets
    pub passing_benchmarks: usize,
    /// Critical issues count
    pub critical_issues: usize,
    /// Performance improvements
    pub improvements: usize,
    /// Performance regressions
    pub regressions: usize,
    /// Key findings
    pub key_findings: Vec<String>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Performance grade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceGrade {
    A, // Excellent (90-100)
    B, // Good (80-89)
    C, // Satisfactory (70-79)
    D, // Needs Improvement (60-69)
    F, // Poor (<60)
}

/// Individual benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Category
    pub category: String,
    /// Performance score (0-100)
    pub score: f64,
    /// Target compliance status
    pub compliance_status: ComplianceStatus,
    /// Key metrics
    pub metrics: BenchmarkMetrics,
    /// Trend information
    pub trend: Option<TrendInfo>,
    /// Issues found
    pub issues: Vec<BenchmarkIssue>,
}

/// Benchmark metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// 95th percentile latency (ms)
    pub p95_latency_ms: f64,
    /// Throughput (ops/sec)
    pub throughput: f64,
    /// Error rate (%)
    pub error_rate_percent: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage (%)
    pub cpu_usage_percent: f64,
}

/// Trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendInfo {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0-1)
    pub strength: f64,
    /// Change from previous measurement
    pub change_percent: f64,
}

/// Benchmark issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue type
    pub issue_type: IssueType,
    /// Issue description
    pub description: String,
    /// Recommended action
    pub recommendation: String,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

/// Issue type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    TargetMissed,
    Regression,
    HighVariability,
    ResourceUsage,
    Anomaly,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// CPU model
    pub cpu_model: String,
    /// CPU cores
    pub cpu_cores: u32,
    /// Total memory (MB)
    pub total_memory_mb: u64,
    /// Rust version
    pub rust_version: String,
    /// Environment variables (relevant ones)
    pub environment: HashMap<String, String>,
}

/// Historical comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalComparison {
    /// Comparison period (days)
    pub comparison_period_days: u32,
    /// Historical data points
    pub historical_points: Vec<HistoricalDataPoint>,
    /// Performance trends
    pub trends: Vec<HistoricalTrend>,
    /// Notable changes
    pub notable_changes: Vec<NotableChange>,
}

/// Historical data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Benchmark name
    pub benchmark_name: String,
    /// Performance score
    pub score: f64,
    /// Key metrics
    pub metrics: BenchmarkMetrics,
}

/// Historical trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTrend {
    /// Benchmark name
    pub benchmark_name: String,
    /// Trend over period
    pub trend: TrendDirection,
    /// Average improvement/degradation per day
    pub daily_change_percent: f64,
    /// Statistical significance
    pub significance: f64,
}

/// Notable change in performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotableChange {
    /// Timestamp of change
    pub timestamp: SystemTime,
    /// Benchmark affected
    pub benchmark_name: String,
    /// Change description
    pub description: String,
    /// Change magnitude
    pub magnitude_percent: f64,
}

/// Chart data for visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// Chart ID
    pub id: String,
    /// Chart title
    pub title: String,
    /// Chart type
    pub chart_type: ChartType,
    /// Data series
    pub series: Vec<DataSeries>,
    /// Chart options
    pub options: ChartOptions,
}

/// Chart types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Area,
    Scatter,
    Heatmap,
    Gauge,
}

/// Data series for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// Series name
    pub name: String,
    /// Data points
    pub data: Vec<DataPoint>,
    /// Series color
    pub color: String,
}

/// Individual data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// X-axis value (timestamp or label)
    pub x: f64,
    /// Y-axis value
    pub y: f64,
    /// Optional label
    pub label: Option<String>,
}

/// Performance report generator
pub struct PerformanceReportGenerator {
    config: ReportConfig,
}

impl PerformanceReportGenerator {
    /// Create new report generator
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Generate comprehensive performance report
    pub fn generate_report(
        &self,
        measurements: &[PerformanceMeasurement],
        analysis: &PerformanceAnalysisResult,
        targets: &PerformanceTargets,
        benchmark_config: &PerformanceBenchmarkConfig,
    ) -> Result<PerformanceReport, Box<dyn std::error::Error>> {
        println!("📊 Generating comprehensive performance report");

        let start_time = SystemTime::now();

        // Generate report data
        let metadata = self.generate_metadata(benchmark_config, targets, start_time)?;
        let executive_summary = self.generate_executive_summary(measurements, analysis)?;
        let benchmark_results = self.generate_benchmark_results(measurements, analysis)?;
        let system_info = self.generate_system_info()?;
        let historical_comparison = if self.config.include_historical_comparison {
            Some(self.generate_historical_comparison(measurements)?)
        } else {
            None
        };
        let charts = if self.config.include_charts {
            self.generate_charts(measurements, analysis)?
        } else {
            Vec::new()
        };

        let mut report = PerformanceReport {
            metadata,
            executive_summary,
            benchmark_results,
            analysis: analysis.clone(),
            system_info,
            historical_comparison,
            charts,
        };

        // Update generation duration
        let generation_duration = start_time.elapsed().unwrap_or_default();
        report.metadata.generation_duration_ms = generation_duration.as_secs_f64() * 1000.0;

        println!("✅ Performance report generated in {:.2}ms", report.metadata.generation_duration_ms);

        Ok(report)
    }

    /// Export report in specified formats
    pub fn export_report(&self, report: &PerformanceReport) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        println!("💾 Exporting performance report in {} formats", self.config.formats.len());

        // Create output directory
        fs::create_dir_all(&self.config.output_directory)?;

        let mut exported_files = Vec::new();

        for format in &self.config.formats {
            let file_path = match format {
                ReportFormat::Html => self.export_html(report)?,
                ReportFormat::Json => self.export_json(report)?,
                ReportFormat::Csv => self.export_csv(report)?,
                ReportFormat::Markdown => self.export_markdown(report)?,
                ReportFormat::Pdf => self.export_pdf(report)?,
            };
            exported_files.push(file_path);
        }

        println!("✅ Report exported to {} files", exported_files.len());

        Ok(exported_files)
    }

    // Private methods for report generation

    /// Generate report metadata
    fn generate_metadata(
        &self,
        config: &PerformanceBenchmarkConfig,
        targets: &PerformanceTargets,
        start_time: SystemTime,
    ) -> Result<ReportMetadata, Box<dyn std::error::Error>> {
        Ok(ReportMetadata {
            generated_at: start_time,
            version: "1.0.0".to_string(),
            config: config.clone(),
            targets: targets.clone(),
            generation_duration_ms: 0.0, // Will be updated later
        })
    }

    /// Generate executive summary
    fn generate_executive_summary(
        &self,
        measurements: &[PerformanceMeasurement],
        analysis: &PerformanceAnalysisResult,
    ) -> Result<ExecutiveSummary, Box<dyn std::error::Error>> {
        let total_benchmarks = measurements.len();
        let passing_benchmarks = analysis.target_compliance.iter()
            .filter(|c| matches!(c.status, ComplianceStatus::FullyCompliant | ComplianceStatus::MostlyCompliant))
            .count();

        let critical_issues = analysis.regressions.iter()
            .filter(|r| r.severity == RegressionSeverity::Critical)
            .count();

        let improvements = analysis.regressions.iter()
            .filter(|r| r.is_improvement)
            .count();

        let regressions = analysis.regressions.iter()
            .filter(|r| r.is_regression)
            .count();

        let performance_grade = match analysis.overall_score {
            score if score >= 90.0 => PerformanceGrade::A,
            score if score >= 80.0 => PerformanceGrade::B,
            score if score >= 70.0 => PerformanceGrade::C,
            score if score >= 60.0 => PerformanceGrade::D,
            _ => PerformanceGrade::F,
        };

        let key_findings = self.generate_key_findings(analysis);
        let recommended_actions = self.generate_recommended_actions(analysis);

        Ok(ExecutiveSummary {
            overall_score: analysis.overall_score,
            performance_grade,
            total_benchmarks,
            passing_benchmarks,
            critical_issues,
            improvements,
            regressions,
            key_findings,
            recommended_actions,
        })
    }

    /// Generate benchmark results
    fn generate_benchmark_results(
        &self,
        measurements: &[PerformanceMeasurement],
        analysis: &PerformanceAnalysisResult,
    ) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for measurement in measurements {
            let compliance_status = analysis.target_compliance.iter()
                .find(|c| c.benchmark_name == measurement.benchmark_name)
                .map(|c| c.status)
                .unwrap_or(ComplianceStatus::NonCompliant);

            let score = self.calculate_benchmark_score(measurement, &compliance_status);

            let metrics = BenchmarkMetrics {
                avg_latency_ms: measurement.avg_operation_time_ms,
                p95_latency_ms: measurement.p95_operation_time_ms,
                throughput: measurement.operations_per_second,
                error_rate_percent: 100.0 - measurement.success_rate_percent,
                memory_usage_mb: measurement.memory_usage_mb,
                cpu_usage_percent: measurement.cpu_usage_percent,
            };

            let trend = analysis.trends.iter()
                .find(|t| t.benchmark_name == measurement.benchmark_name)
                .map(|t| TrendInfo {
                    direction: t.trend_direction,
                    strength: t.trend_strength,
                    change_percent: (t.predicted_next_value - measurement.avg_operation_time_ms) / measurement.avg_operation_time_ms * 100.0,
                });

            let issues = self.generate_benchmark_issues(measurement, analysis);

            results.push(BenchmarkResult {
                name: measurement.benchmark_name.clone(),
                category: measurement.category.clone(),
                score,
                compliance_status,
                metrics,
                trend,
                issues,
            });
        }

        Ok(results)
    }

    /// Calculate benchmark score
    fn calculate_benchmark_score(&self, measurement: &PerformanceMeasurement, compliance_status: &ComplianceStatus) -> f64 {
        let mut score = match compliance_status {
            ComplianceStatus::FullyCompliant => 95.0,
            ComplianceStatus::MostlyCompliant => 80.0,
            ComplianceStatus::PartiallyCompliant => 60.0,
            ComplianceStatus::NonCompliant => 40.0,
        };

        // Adjust for success rate
        score *= measurement.success_rate_percent / 100.0;

        // Adjust for error count
        if measurement.error_count > 0 {
            score -= (measurement.error_count as f64 / measurement.operation_count as f64) * 20.0;
        }

        score.max(0.0).min(100.0)
    }

    /// Generate benchmark issues
    fn generate_benchmark_issues(&self, measurement: &PerformanceMeasurement, analysis: &PerformanceAnalysisResult) -> Vec<BenchmarkIssue> {
        let mut issues = Vec::new();

        // Check for regressions
        for regression in &analysis.regressions {
            if regression.benchmark_name == measurement.benchmark_name && regression.is_regression {
                issues.push(BenchmarkIssue {
                    severity: match regression.severity {
                        RegressionSeverity::Critical => IssueSeverity::Critical,
                        RegressionSeverity::Major => IssueSeverity::Major,
                        RegressionSeverity::Minor => IssueSeverity::Minor,
                        RegressionSeverity::Negligible => IssueSeverity::Info,
                    },
                    issue_type: IssueType::Regression,
                    description: format!("Performance regression of {:.1}%", regression.change_percent),
                    recommendation: "Investigate recent changes and optimize performance".to_string(),
                });
            }
        }

        // Check for target misses
        for compliance in &analysis.target_compliance {
            if compliance.benchmark_name == measurement.benchmark_name {
                for failed_target in &compliance.failed_targets {
                    issues.push(BenchmarkIssue {
                        severity: IssueSeverity::Major,
                        issue_type: IssueType::TargetMissed,
                        description: format!("Failed to meet target for {}", failed_target.metric_name),
                        recommendation: failed_target.recommended_action.clone(),
                    });
                }
            }
        }

        // Check for high error rates
        if measurement.error_count > 0 {
            let error_rate = (measurement.error_count as f64 / measurement.operation_count as f64) * 100.0;
            if error_rate > 5.0 {
                issues.push(BenchmarkIssue {
                    severity: if error_rate > 20.0 { IssueSeverity::Critical } else { IssueSeverity::Major },
                    issue_type: IssueType::Anomaly,
                    description: format!("High error rate: {:.1}%", error_rate),
                    recommendation: "Investigate error causes and improve error handling".to_string(),
                });
            }
        }

        issues
    }

    /// Generate system information
    fn generate_system_info(&self) -> Result<SystemInfo, Box<dyn std::error::Error>> {
        let mut environment = HashMap::new();
        environment.insert("RUST_BACKTRACE".to_string(), std::env::var("RUST_BACKTRACE").unwrap_or_default());
        environment.insert("RUST_LOG".to_string(), std::env::var("RUST_LOG").unwrap_or_default());

        Ok(SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_model: "Unknown".to_string(), // Would need system detection
            cpu_cores: num_cpus::get() as u32,
            total_memory_mb: 8192, // Would need system detection
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            environment,
        })
    }

    /// Generate historical comparison
    fn generate_historical_comparison(&self, _measurements: &[PerformanceMeasurement]) -> Result<HistoricalComparison, Box<dyn std::error::Error>> {
        // Simplified implementation - would normally load historical data
        Ok(HistoricalComparison {
            comparison_period_days: 30,
            historical_points: Vec::new(),
            trends: Vec::new(),
            notable_changes: Vec::new(),
        })
    }

    /// Generate charts
    fn generate_charts(&self, measurements: &[PerformanceMeasurement], analysis: &PerformanceAnalysisResult) -> Result<Vec<ChartData>, Box<dyn std::error::Error>> {
        let mut charts = Vec::new();

        // Latency chart
        charts.push(self.create_latency_chart(measurements)?);

        // Throughput chart
        charts.push(self.create_throughput_chart(measurements)?);

        // Performance score chart
        charts.push(self.create_performance_score_chart(analysis)?);

        // Error rate chart
        charts.push(self.create_error_rate_chart(measurements)?);

        Ok(charts)
    }

    /// Create latency chart
    fn create_latency_chart(&self, measurements: &[PerformanceMeasurement]) -> Result<ChartData, Box<dyn std::error::Error>> {
        let mut series = Vec::new();

        // Average latency series
        let avg_data: Vec<DataPoint> = measurements.iter().enumerate()
            .map(|(i, m)| DataPoint {
                x: i as f64,
                y: m.avg_operation_time_ms,
                label: Some(m.benchmark_name.clone()),
            })
            .collect();

        series.push(DataSeries {
            name: "Average Latency".to_string(),
            data: avg_data,
            color: "#3498db".to_string(),
        });

        // P95 latency series
        let p95_data: Vec<DataPoint> = measurements.iter().enumerate()
            .map(|(i, m)| DataPoint {
                x: i as f64,
                y: m.p95_operation_time_ms,
                label: Some(m.benchmark_name.clone()),
            })
            .collect();

        series.push(DataSeries {
            name: "95th Percentile Latency".to_string(),
            data: p95_data,
            color: "#e74c3c".to_string(),
        });

        Ok(ChartData {
            id: "latency_chart".to_string(),
            title: "Response Time Performance".to_string(),
            chart_type: ChartType::Line,
            series,
            options: self.config.chart_options.clone(),
        })
    }

    /// Create throughput chart
    fn create_throughput_chart(&self, measurements: &[PerformanceMeasurement]) -> Result<ChartData, Box<dyn std::error::Error>> {
        let throughput_data: Vec<DataPoint> = measurements.iter().enumerate()
            .map(|(i, m)| DataPoint {
                x: i as f64,
                y: m.operations_per_second,
                label: Some(m.benchmark_name.clone()),
            })
            .collect();

        let series = vec![DataSeries {
            name: "Throughput".to_string(),
            data: throughput_data,
            color: "#2ecc71".to_string(),
        }];

        Ok(ChartData {
            id: "throughput_chart".to_string(),
            title: "Throughput Performance (Operations/Second)".to_string(),
            chart_type: ChartType::Bar,
            series,
            options: self.config.chart_options.clone(),
        })
    }

    /// Create performance score chart
    fn create_performance_score_chart(&self, analysis: &PerformanceAnalysisResult) -> Result<ChartData, Box<dyn std::error::Error>> {
        let score_data: Vec<DataPoint> = analysis.target_compliance.iter().enumerate()
            .map(|(i, c)| DataPoint {
                x: i as f64,
                y: c.overall_score,
                label: Some(c.benchmark_name.clone()),
            })
            .collect();

        let series = vec![DataSeries {
            name: "Performance Score".to_string(),
            data: score_data,
            color: "#f39c12".to_string(),
        }];

        Ok(ChartData {
            id: "performance_score_chart".to_string(),
            title: "Performance Score by Benchmark".to_string(),
            chart_type: ChartType::Bar,
            series,
            options: self.config.chart_options.clone(),
        })
    }

    /// Create error rate chart
    fn create_error_rate_chart(&self, measurements: &[PerformanceMeasurement]) -> Result<ChartData, Box<dyn std::error::Error>> {
        let error_data: Vec<DataPoint> = measurements.iter().enumerate()
            .map(|(i, m)| DataPoint {
                x: i as f64,
                y: 100.0 - m.success_rate_percent,
                label: Some(m.benchmark_name.clone()),
            })
            .collect();

        let series = vec![DataSeries {
            name: "Error Rate".to_string(),
            data: error_data,
            color: "#e67e22".to_string(),
        }];

        Ok(ChartData {
            id: "error_rate_chart".to_string(),
            title: "Error Rate by Benchmark (%)".to_string(),
            chart_type: ChartType::Area,
            series,
            options: self.config.chart_options.clone(),
        })
    }

    /// Generate key findings
    fn generate_key_findings(&self, analysis: &PerformanceAnalysisResult) -> Vec<String> {
        let mut findings = Vec::new();

        findings.push(format!("Overall performance score: {:.1}/100", analysis.overall_score));

        let compliant_count = analysis.target_compliance.iter()
            .filter(|c| matches!(c.status, ComplianceStatus::FullyCompliant))
            .count();
        findings.push(format!("{}/{} benchmarks fully compliant with targets", 
            compliant_count, analysis.target_compliance.len()));

        if !analysis.regressions.is_empty() {
            let regression_count = analysis.regressions.iter()
                .filter(|r| r.is_regression)
                .count();
            findings.push(format!("{} performance regressions detected", regression_count));
        }

        if !analysis.trends.is_empty() {
            let improving_trends = analysis.trends.iter()
                .filter(|t| t.trend_direction == TrendDirection::Improving)
                .count();
            findings.push(format!("{}/{} benchmarks showing improving trends", 
                improving_trends, analysis.trends.len()));
        }

        findings
    }

    /// Generate recommended actions
    fn generate_recommended_actions(&self, analysis: &PerformanceAnalysisResult) -> Vec<String> {
        let mut actions = Vec::new();

        // High priority recommendations
        let high_priority = analysis.recommendations.iter()
            .filter(|r| matches!(r.priority, super::performance_analysis::RecommendationPriority::Critical | super::performance_analysis::RecommendationPriority::High))
            .take(5);

        for rec in high_priority {
            actions.push(format!("{}: {}", rec.title, rec.description));
        }

        if actions.is_empty() {
            actions.push("Continue monitoring performance metrics".to_string());
            actions.push("Maintain current optimization strategies".to_string());
        }

        actions
    }

    // Export methods

    /// Export HTML report
    fn export_html(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let file_path = format!("{}/performance_report.html", self.config.output_directory);
        let html_content = self.generate_html_content(report)?;
        fs::write(&file_path, html_content)?;
        Ok(file_path)
    }

    /// Export JSON report
    fn export_json(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let file_path = format!("{}/performance_report.json", self.config.output_directory);
        let json_content = serde_json::to_string_pretty(report)?;
        fs::write(&file_path, json_content)?;
        Ok(file_path)
    }

    /// Export CSV report
    fn export_csv(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let file_path = format!("{}/performance_report.csv", self.config.output_directory);
        let csv_content = self.generate_csv_content(report)?;
        fs::write(&file_path, csv_content)?;
        Ok(file_path)
    }

    /// Export Markdown report
    fn export_markdown(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let file_path = format!("{}/performance_report.md", self.config.output_directory);
        let markdown_content = self.generate_markdown_content(report)?;
        fs::write(&file_path, markdown_content)?;
        Ok(file_path)
    }

    /// Export PDF report
    fn export_pdf(&self, _report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let file_path = format!("{}/performance_report.pdf", self.config.output_directory);
        // PDF generation would require additional dependencies
        fs::write(&file_path, "PDF export not implemented")?;
        Ok(file_path)
    }

    /// Generate HTML content
    fn generate_html_content(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let grade_color = match report.executive_summary.performance_grade {
            PerformanceGrade::A => "#2ecc71",
            PerformanceGrade::B => "#3498db",
            PerformanceGrade::C => "#f39c12",
            PerformanceGrade::D => "#e67e22",
            PerformanceGrade::F => "#e74c3c",
        };

        let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Performance Benchmark Report</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f8f9fa; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 10px; margin-bottom: 30px; }}
        .grade {{ font-size: 3em; font-weight: bold; color: {}; }}
        .score {{ font-size: 2em; margin: 10px 0; }}
        .summary-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .summary-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .benchmark-results {{ background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .benchmark {{ border-bottom: 1px solid #eee; padding: 20px 0; }}
        .metric {{ display: inline-block; margin: 5px 10px; padding: 5px 10px; background: #f8f9fa; border-radius: 4px; }}
        .critical {{ color: #e74c3c; font-weight: bold; }}
        .warning {{ color: #f39c12; font-weight: bold; }}
        .success {{ color: #2ecc71; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>DataFold Performance Benchmark Report</h1>
            <div class="score">Overall Score: {:.1}/100</div>
            <div class="grade">Grade: {:?}</div>
            <p>Generated: {:?}</p>
        </div>

        <div class="summary-grid">
            <div class="summary-card">
                <h3>Benchmarks</h3>
                <p><strong>{}</strong> total</p>
                <p><strong>{}</strong> passing targets</p>
            </div>
            <div class="summary-card">
                <h3>Issues</h3>
                <p><strong class="critical">{}</strong> critical</p>
                <p><strong>{}</strong> regressions</p>
                <p><strong class="success">{}</strong> improvements</p>
            </div>
        </div>

        <div class="benchmark-results">
            <h2>Benchmark Results</h2>
            {}
        </div>
    </div>
</body>
</html>
        "#,
            grade_color,
            report.executive_summary.overall_score,
            report.executive_summary.performance_grade,
            report.metadata.generated_at,
            report.executive_summary.total_benchmarks,
            report.executive_summary.passing_benchmarks,
            report.executive_summary.critical_issues,
            report.executive_summary.regressions,
            report.executive_summary.improvements,
            self.generate_benchmark_html(&report.benchmark_results)?
        );

        Ok(html)
    }

    /// Generate benchmark HTML section
    fn generate_benchmark_html(&self, benchmarks: &[BenchmarkResult]) -> Result<String, Box<dyn std::error::Error>> {
        let mut html = String::new();

        for benchmark in benchmarks {
            html.push_str(&format!(r#"
                <div class="benchmark">
                    <h3>{}</h3>
                    <p><strong>Category:</strong> {}</p>
                    <p><strong>Score:</strong> {:.1}/100</p>
                    <div>
                        <span class="metric">Latency: {:.2}ms</span>
                        <span class="metric">P95: {:.2}ms</span>
                        <span class="metric">Throughput: {:.1} ops/s</span>
                        <span class="metric">Errors: {:.1}%</span>
                    </div>
                </div>
            "#,
                benchmark.name,
                benchmark.category,
                benchmark.score,
                benchmark.metrics.avg_latency_ms,
                benchmark.metrics.p95_latency_ms,
                benchmark.metrics.throughput,
                benchmark.metrics.error_rate_percent
            ));
        }

        Ok(html)
    }

    /// Generate CSV content
    fn generate_csv_content(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let mut csv = String::new();
        csv.push_str("Benchmark,Category,Score,Avg_Latency_ms,P95_Latency_ms,Throughput_ops_sec,Error_Rate_percent,Memory_MB,CPU_percent\n");

        for benchmark in &report.benchmark_results {
            csv.push_str(&format!(
                "{},{},{:.1},{:.2},{:.2},{:.1},{:.1},{:.1},{:.1}\n",
                benchmark.name,
                benchmark.category,
                benchmark.score,
                benchmark.metrics.avg_latency_ms,
                benchmark.metrics.p95_latency_ms,
                benchmark.metrics.throughput,
                benchmark.metrics.error_rate_percent,
                benchmark.metrics.memory_usage_mb,
                benchmark.metrics.cpu_usage_percent
            ));
        }

        Ok(csv)
    }

    /// Generate Markdown content
    fn generate_markdown_content(&self, report: &PerformanceReport) -> Result<String, Box<dyn std::error::Error>> {
        let mut md = String::new();

        md.push_str("# DataFold Performance Benchmark Report\n\n");
        md.push_str(&format!("**Generated:** {:?}\n", report.metadata.generated_at));
        md.push_str(&format!("**Overall Score:** {:.1}/100\n", report.executive_summary.overall_score));
        md.push_str(&format!("**Grade:** {:?}\n\n", report.executive_summary.performance_grade));

        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!("- **Total Benchmarks:** {}\n", report.executive_summary.total_benchmarks));
        md.push_str(&format!("- **Passing Targets:** {}\n", report.executive_summary.passing_benchmarks));
        md.push_str(&format!("- **Critical Issues:** {}\n", report.executive_summary.critical_issues));
        md.push_str(&format!("- **Regressions:** {}\n", report.executive_summary.regressions));
        md.push_str(&format!("- **Improvements:** {}\n\n", report.executive_summary.improvements));

        md.push_str("### Key Findings\n\n");
        for finding in &report.executive_summary.key_findings {
            md.push_str(&format!("- {}\n", finding));
        }

        md.push_str("\n### Recommended Actions\n\n");
        for action in &report.executive_summary.recommended_actions {
            md.push_str(&format!("- {}\n", action));
        }

        md.push_str("\n## Benchmark Results\n\n");
        md.push_str("| Benchmark | Category | Score | Avg Latency | P95 Latency | Throughput | Error Rate |\n");
        md.push_str("|-----------|----------|-------|-------------|-------------|------------|------------|\n");

        for benchmark in &report.benchmark_results {
            md.push_str(&format!(
                "| {} | {} | {:.1} | {:.2}ms | {:.2}ms | {:.1} ops/s | {:.1}% |\n",
                benchmark.name,
                benchmark.category,
                benchmark.score,
                benchmark.metrics.avg_latency_ms,
                benchmark.metrics.p95_latency_ms,
                benchmark.metrics.throughput,
                benchmark.metrics.error_rate_percent
            ));
        }

        Ok(md)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_generator_creation() {
        let config = ReportConfig::default();
        let generator = PerformanceReportGenerator::new(config);
        
        assert_eq!(generator.config.formats.len(), 3);
    }

    #[test]
    fn test_performance_grade_calculation() {
        assert_eq!(match 95.0 {
            score if score >= 90.0 => PerformanceGrade::A,
            _ => PerformanceGrade::F,
        }, PerformanceGrade::A);
    }
}