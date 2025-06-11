//! DataFold Performance Benchmark System Example
//!
//! This example demonstrates the complete performance benchmarking system for
//! DataFold's signature authentication across all components.
//!
//! Run with: `cargo run --example performance_benchmark_example`

use datafold::tests::performance::{
    PerformanceBenchmarkSuite, PerformanceBenchmarkConfig, PerformanceTargets,
    PerformanceAnalysisConfig, ReportConfig, ReportFormat
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 DataFold Performance Benchmark System");
    println!("==========================================");
    
    // Configure the benchmark parameters
    let benchmark_config = PerformanceBenchmarkConfig {
        test_duration_seconds: 30,        // Shorter for demo
        warmup_duration_seconds: 5,       // Quick warmup
        micro_benchmark_iterations: 1000, // Moderate iterations
        concurrent_user_counts: vec![1, 5, 10, 25], // Test various loads
        target_request_rates: vec![100.0, 500.0, 1000.0], // Target rates
        enable_memory_profiling: true,
        enable_cpu_monitoring: true,
        enable_latency_analysis: true,
        enable_regression_testing: false,
        baseline_data_path: None,
    };
    
    // Set performance targets based on specification
    let performance_targets = PerformanceTargets {
        server_verification_ms: 1.0,      // <1ms server verification
        client_signing_ms: 10.0,          // <10ms client signing
        end_to_end_ms: 50.0,              // <50ms end-to-end
        min_throughput_rps: 1000.0,       // >1000 requests/second
        max_memory_overhead_bytes: 10 * 1024 * 1024, // <10MB memory
        max_cpu_overhead_percent: 5.0,    // <5% CPU overhead
    };
    
    // Configure performance analysis
    let analysis_config = PerformanceAnalysisConfig {
        regression_threshold_percent: 10.0,
        improvement_threshold_percent: 5.0,
        min_sample_size: 30,
        confidence_level: 0.95,
        enable_trend_analysis: true,
        enable_outlier_detection: true,
        historical_retention_days: 90,
    };
    
    // Configure reporting
    let report_config = ReportConfig {
        output_directory: "reports/performance".to_string(),
        include_charts: true,
        include_historical_comparison: false, // Disabled for demo
        include_regression_analysis: true,
        include_recommendations: true,
        include_system_metrics: true,
        formats: vec![ReportFormat::Html, ReportFormat::Json, ReportFormat::Markdown],
        chart_options: Default::default(),
    };
    
    println!("📋 Configuration:");
    println!("  • Test Duration: {}s", benchmark_config.test_duration_seconds);
    println!("  • Micro-benchmark Iterations: {}", benchmark_config.micro_benchmark_iterations);
    println!("  • Concurrent Users: {:?}", benchmark_config.concurrent_user_counts);
    println!("  • Target Request Rates: {:?}", benchmark_config.target_request_rates);
    println!("  • Performance Targets:");
    println!("    - Server Verification: <{}ms", performance_targets.server_verification_ms);
    println!("    - Client Signing: <{}ms", performance_targets.client_signing_ms);
    println!("    - End-to-End: <{}ms", performance_targets.end_to_end_ms);
    println!("    - Throughput: >{}rps", performance_targets.min_throughput_rps);
    println!("    - Memory: <{}MB", performance_targets.max_memory_overhead_bytes / (1024 * 1024));
    println!("    - CPU: <{}%", performance_targets.max_cpu_overhead_percent);
    println!();
    
    // Create and initialize the benchmark suite
    let mut benchmark_suite = PerformanceBenchmarkSuite::new(
        benchmark_config,
        performance_targets,
    )?;
    
    // Initialize with analysis and reporting capabilities
    benchmark_suite = benchmark_suite.with_analysis_and_reporting(
        analysis_config,
        report_config,
    )?;
    
    println!("🔧 Benchmark suite initialized successfully");
    println!("🎯 Starting comprehensive performance benchmarks...");
    println!();
    
    // Run the complete benchmark suite
    match benchmark_suite.run_complete_benchmark_suite().await {
        Ok(results) => {
            println!("\n✅ Benchmark suite completed successfully!");
            
            // Display detailed results
            display_detailed_results(&results);
            
            // Show recommendations if any
            if !results.recommendations.is_empty() {
                println!("\n💡 OPTIMIZATION RECOMMENDATIONS");
                println!("═══════════════════════════════════");
                for (i, rec) in results.recommendations.iter().enumerate() {
                    println!("{}. [{}] {}", 
                        i + 1, 
                        format!("{:?}", rec.priority), 
                        rec.recommendation
                    );
                    println!("   Category: {:?}", rec.category);
                    println!("   Description: {}", rec.description);
                    println!("   Expected: {}", rec.expected_improvement);
                    println!("   Effort: {:?}", rec.effort_estimate);
                    println!();
                }
            }
            
            // Show regression analysis if available
            if let Some(ref regression) = results.regression_analysis {
                if !regression.regressions.is_empty() || !regression.improvements.is_empty() {
                    println!("\n📈 REGRESSION ANALYSIS");
                    println!("═══════════════════════");
                    println!("Overall Regression Score: {:.1}/100", regression.regression_score);
                    
                    if !regression.regressions.is_empty() {
                        println!("\nRegressions:");
                        for reg in &regression.regressions {
                            println!("  🚨 {}: {:.1}% slower ({})", 
                                reg.test_name, reg.regression_percent, reg.metric);
                        }
                    }
                    
                    if !regression.improvements.is_empty() {
                        println!("\nImprovements:");
                        for imp in &regression.improvements {
                            println!("  ✨ {}: {:.1}% faster ({})", 
                                imp.test_name, imp.improvement_percent, imp.metric);
                        }
                    }
                }
            }
            
            println!("\n📊 SYSTEM INFORMATION");
            println!("════════════════════");
            println!("OS: {}", results.metadata.environment.os);
            println!("CPU: {}", results.metadata.environment.cpu);
            println!("Memory: {:.1}GB", results.metadata.environment.memory_gb);
            println!("Rust Version: {}", results.metadata.environment.rust_version);
            println!("Execution Time: {:.2}s", results.metadata.execution_duration.as_secs_f64());
            
            println!("\n🎉 Performance benchmarking completed!");
            println!("📁 Reports have been generated in the reports/performance directory");
            
        }
        Err(e) => {
            eprintln!("❌ Benchmark suite failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Display detailed benchmark results
fn display_detailed_results(results: &datafold::tests::performance::PerformanceBenchmarkResult) {
    println!("📊 DETAILED RESULTS");
    println!("══════════════════");
    
    println!("Overall Summary:");
    println!("  • Total Operations: {}", results.summary.total_operations);
    println!("  • Test Duration: {:.2}s", results.summary.total_duration.as_secs_f64());
    println!("  • Success Rate: {:.2}%", results.summary.overall_success_rate);
    println!("  • Targets Met: {}/{}", results.summary.targets_met, results.summary.total_targets);
    println!("  • Performance Score: {:.1}/100", results.summary.performance_score);
    
    println!("\nComponent Performance:");
    for (component, summary) in &results.summary.component_summaries {
        println!("  📦 {}:", component);
        println!("    Score: {:.1}/100", summary.avg_performance_score);
        println!("    Measurements: {}", summary.measurement_count);
        println!("    Best Operation: {}", summary.best_operation);
        println!("    Worst Operation: {}", summary.worst_operation);
    }
    
    if !results.measurements.is_empty() {
        println!("\nIndividual Benchmark Results:");
        for measurement in &results.measurements {
            let status = if measurement.meets_targets(&results.targets) { "✅" } else { "❌" };
            println!("  {} {} ({})", status, measurement.test_name, measurement.component);
            println!("    Avg Latency: {:.3}ms", measurement.avg_operation_time_ms);
            println!("    P95 Latency: {:.3}ms", measurement.p95_operation_time_ms);
            println!("    P99 Latency: {:.3}ms", measurement.p99_operation_time_ms);
            println!("    Throughput: {:.1} ops/sec", measurement.operations_per_second);
            println!("    Success Rate: {:.1}%", measurement.success_rate_percent);
            if let Some(memory) = measurement.memory_usage_bytes {
                println!("    Memory Usage: {:.1}MB", memory as f64 / (1024.0 * 1024.0));
            }
            if let Some(cpu) = measurement.cpu_usage_percent {
                println!("    CPU Usage: {:.1}%", cpu);
            }
            println!();
        }
    }
}

/// Example of how to run specific benchmarks
#[allow(dead_code)]
async fn run_specific_benchmarks_example() -> Result<(), Box<dyn Error>> {
    use datafold::tests::performance::{
        ServerPerformanceBenchmarks, ClientPerformanceBenchmarks,
        SdkPerformanceBenchmarks, CliPerformanceBenchmarks,
        EndToEndPerformanceBenchmarks, PerformanceBenchmarkConfig as RunnerConfig,
        PerformanceTargets as RunnerTargets
    };
    
    let config = RunnerConfig::default();
    let targets = RunnerTargets::default();
    
    println!("🔧 Running specific benchmark components:");
    
    // Server benchmarks
    println!("📡 Server Performance Benchmarks");
    let mut server_benchmarks = ServerPerformanceBenchmarks::new(config.clone(), targets.clone())?;
    let server_results = server_benchmarks.run_all_benchmarks().await?;
    println!("  ✅ Server benchmarks completed: {} measurements", server_results.len());
    
    // Client benchmarks
    println!("💻 Client Performance Benchmarks");
    let mut client_benchmarks = ClientPerformanceBenchmarks::new(config.clone(), targets.clone())?;
    let client_results = client_benchmarks.run_all_benchmarks().await?;
    println!("  ✅ Client benchmarks completed: {} measurements", client_results.len());
    
    // SDK benchmarks
    println!("📦 SDK Performance Benchmarks");
    let mut sdk_benchmarks = SdkPerformanceBenchmarks::new(config.clone(), targets.clone())?;
    let sdk_results = sdk_benchmarks.run_all_benchmarks().await?;
    println!("  ✅ SDK benchmarks completed: {} measurements", sdk_results.len());
    
    // CLI benchmarks
    println!("⚡ CLI Performance Benchmarks");
    let mut cli_benchmarks = CliPerformanceBenchmarks::new(config.clone(), targets.clone())?;
    let cli_results = cli_benchmarks.run_all_benchmarks().await?;
    println!("  ✅ CLI benchmarks completed: {} measurements", cli_results.len());
    
    // End-to-end benchmarks
    println!("🔄 End-to-End Performance Benchmarks");
    let mut e2e_benchmarks = EndToEndPerformanceBenchmarks::new(config.clone(), targets.clone())?;
    let e2e_results = e2e_benchmarks.run_all_benchmarks().await?;
    println!("  ✅ End-to-end benchmarks completed: {} measurements", e2e_results.len());
    
    let total_measurements = server_results.len() + client_results.len() + 
                           sdk_results.len() + cli_results.len() + e2e_results.len();
    
    println!("🎯 All specific benchmarks completed: {} total measurements", total_measurements);
    
    Ok(())
}

/// Example of how to use the metrics collector
#[allow(dead_code)]
async fn metrics_collector_example() -> Result<(), Box<dyn Error>> {
    use datafold::tests::performance::{PerformanceMetricsCollector, MetricsCollectorConfig};
    
    let config = MetricsCollectorConfig::default();
    let collector = PerformanceMetricsCollector::new(config);
    
    println!("📊 Starting metrics collection...");
    collector.start_collection().await?;
    
    // Simulate some operations
    for i in 0..100 {
        let latency_ms = 0.5 + (i as f64 * 0.01); // Simulate increasing latency
        let success = i % 10 != 0; // 10% failure rate
        collector.record_operation(latency_ms, success);
        
        if i % 10 == 0 {
            collector.record_network_traffic(1024, 512); // Some network activity
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    
    collector.stop_collection().await;
    
    // Get metrics
    let current_metrics = collector.get_current_metrics();
    let aggregated_metrics = collector.get_aggregated_metrics();
    let alerts = collector.get_active_alerts();
    
    println!("📈 Metrics Collection Results:");
    println!("  Current Latency: {:.3}ms", current_metrics.current_latency_ms);
    println!("  Current Throughput: {:.1} ops/sec", current_metrics.current_throughput);
    println!("  Error Rate: {:.1}%", current_metrics.current_error_rate_percent);
    println!("  Aggregated Measurements: {}", aggregated_metrics.len());
    println!("  Active Alerts: {}", alerts.len());
    
    for alert in &alerts {
        println!("    🚨 [{:?}] {}", alert.severity, alert.message);
    }
    
    Ok(())
}