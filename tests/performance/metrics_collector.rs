//! Performance Metrics Collector
//!
//! This module provides real-time performance metrics collection, monitoring,
//! and data aggregation for the performance benchmarking system.

use super::{PerformanceMeasurement, BenchmarkTimer};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::interval;

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCollectorConfig {
    /// Collection interval in milliseconds
    pub collection_interval_ms: u64,
    /// Maximum number of samples to retain in memory
    pub max_samples_in_memory: usize,
    /// Enable system metrics collection
    pub collect_system_metrics: bool,
    /// Enable memory profiling
    pub enable_memory_profiling: bool,
    /// Enable CPU profiling
    pub enable_cpu_profiling: bool,
    /// Enable network metrics
    pub enable_network_metrics: bool,
    /// Metrics aggregation window size
    pub aggregation_window_size: usize,
    /// Enable real-time alerting
    pub enable_real_time_alerting: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for MetricsCollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_ms: 100,
            max_samples_in_memory: 10000,
            collect_system_metrics: true,
            enable_memory_profiling: true,
            enable_cpu_profiling: true,
            enable_network_metrics: true,
            aggregation_window_size: 100,
            enable_real_time_alerting: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Alert thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: f64,
    /// Minimum acceptable throughput (ops/sec)
    pub min_throughput: f64,
    /// Maximum acceptable error rate percentage
    pub max_error_rate_percent: f64,
    /// Maximum acceptable memory usage in MB
    pub max_memory_mb: f64,
    /// Maximum acceptable CPU usage percentage
    pub max_cpu_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_latency_ms: 100.0,
            min_throughput: 100.0,
            max_error_rate_percent: 5.0,
            max_memory_mb: 100.0,
            max_cpu_percent: 80.0,
        }
    }
}

/// Real-time performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    /// Timestamp of measurement
    pub timestamp: SystemTime,
    /// Current latency in milliseconds
    pub current_latency_ms: f64,
    /// Current throughput (operations per second)
    pub current_throughput: f64,
    /// Current error rate percentage
    pub current_error_rate_percent: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Network bytes received
    pub network_bytes_received: u64,
    /// Active connections count
    pub active_connections: u32,
    /// Queue depth
    pub queue_depth: u32,
}

/// Aggregated metrics over a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Time window start
    pub window_start: SystemTime,
    /// Time window end
    pub window_end: SystemTime,
    /// Total operations processed
    pub total_operations: u64,
    /// Average latency
    pub avg_latency_ms: f64,
    /// Median latency
    pub median_latency_ms: f64,
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    /// 99th percentile latency
    pub p99_latency_ms: f64,
    /// Maximum latency
    pub max_latency_ms: f64,
    /// Minimum latency
    pub min_latency_ms: f64,
    /// Average throughput
    pub avg_throughput: f64,
    /// Peak throughput
    pub peak_throughput: f64,
    /// Total errors
    pub total_errors: u64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Average memory usage
    pub avg_memory_usage_mb: f64,
    /// Peak memory usage
    pub peak_memory_usage_mb: f64,
    /// Average CPU usage
    pub avg_cpu_usage_percent: f64,
    /// Peak CPU usage
    pub peak_cpu_usage_percent: f64,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Alert timestamp
    pub timestamp: SystemTime,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert message
    pub message: String,
    /// Current value that triggered the alert
    pub current_value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// Benchmark or component that triggered the alert
    pub source: String,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    HighLatency,
    LowThroughput,
    HighErrorRate,
    HighMemoryUsage,
    HighCpuUsage,
    NetworkIssue,
    SystemResource,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Timestamp
    pub timestamp: SystemTime,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Available memory in MB
    pub available_memory_mb: f64,
    /// Disk I/O read bytes per second
    pub disk_read_bps: u64,
    /// Disk I/O write bytes per second
    pub disk_write_bps: u64,
    /// Network bytes sent per second
    pub network_sent_bps: u64,
    /// Network bytes received per second
    pub network_received_bps: u64,
    /// Load average (1 minute)
    pub load_average_1m: f64,
    /// Load average (5 minutes)
    pub load_average_5m: f64,
    /// Load average (15 minutes)
    pub load_average_15m: f64,
}

/// Memory profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Heap allocated bytes
    pub heap_allocated_bytes: u64,
    /// Heap capacity bytes
    pub heap_capacity_bytes: u64,
    /// Stack size bytes
    pub stack_size_bytes: u64,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
    /// Garbage collection count
    pub gc_count: u64,
    /// Time spent in garbage collection (ms)
    pub gc_time_ms: f64,
}

/// Performance metrics collector
pub struct PerformanceMetricsCollector {
    config: MetricsCollectorConfig,
    
    // Real-time metrics
    current_metrics: Arc<Mutex<RealTimeMetrics>>,
    
    // Counters
    operation_counter: Arc<AtomicUsize>,
    error_counter: Arc<AtomicUsize>,
    bytes_sent_counter: Arc<AtomicU64>,
    bytes_received_counter: Arc<AtomicU64>,
    
    // Sample collections
    latency_samples: Arc<Mutex<VecDeque<f64>>>,
    throughput_samples: Arc<Mutex<VecDeque<f64>>>,
    system_metrics: Arc<Mutex<VecDeque<SystemMetrics>>>,
    memory_profiles: Arc<Mutex<VecDeque<MemoryProfile>>>,
    
    // Aggregated data
    aggregated_metrics: Arc<Mutex<Vec<AggregatedMetrics>>>,
    
    // Alerts
    active_alerts: Arc<Mutex<Vec<PerformanceAlert>>>,
    
    // Timing
    start_time: Instant,
    last_aggregation: Arc<Mutex<Instant>>,
    
    // Control flags
    is_collecting: Arc<Mutex<bool>>,
}

impl PerformanceMetricsCollector {
    /// Create new metrics collector
    pub fn new(config: MetricsCollectorConfig) -> Self {
        let now = Instant::now();
        
        Self {
            config,
            current_metrics: Arc::new(Mutex::new(RealTimeMetrics {
                timestamp: SystemTime::now(),
                current_latency_ms: 0.0,
                current_throughput: 0.0,
                current_error_rate_percent: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                active_connections: 0,
                queue_depth: 0,
            })),
            operation_counter: Arc::new(AtomicUsize::new(0)),
            error_counter: Arc::new(AtomicUsize::new(0)),
            bytes_sent_counter: Arc::new(AtomicU64::new(0)),
            bytes_received_counter: Arc::new(AtomicU64::new(0)),
            latency_samples: Arc::new(Mutex::new(VecDeque::new())),
            throughput_samples: Arc::new(Mutex::new(VecDeque::new())),
            system_metrics: Arc::new(Mutex::new(VecDeque::new())),
            memory_profiles: Arc::new(Mutex::new(VecDeque::new())),
            aggregated_metrics: Arc::new(Mutex::new(Vec::new())),
            active_alerts: Arc::new(Mutex::new(Vec::new())),
            start_time: now,
            last_aggregation: Arc::new(Mutex::new(now)),
            is_collecting: Arc::new(Mutex::new(false)),
        }
    }

    /// Start metrics collection
    pub async fn start_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut collecting = self.is_collecting.lock().unwrap();
            if *collecting {
                return Ok(()); // Already collecting
            }
            *collecting = true;
        }

        println!("📊 Starting performance metrics collection");

        // Start collection task
        let collector = self.clone_for_task();
        tokio::spawn(async move {
            collector.collection_loop().await;
        });

        Ok(())
    }

    /// Stop metrics collection
    pub async fn stop_collection(&self) {
        {
            let mut collecting = self.is_collecting.lock().unwrap();
            *collecting = false;
        }
        
        println!("⏹️ Stopping performance metrics collection");
        
        // Perform final aggregation
        self.aggregate_metrics();
    }

    /// Record operation timing
    pub fn record_operation(&self, latency_ms: f64, success: bool) {
        self.operation_counter.fetch_add(1, Ordering::Relaxed);
        
        if !success {
            self.error_counter.fetch_add(1, Ordering::Relaxed);
        }

        // Add to latency samples
        {
            let mut samples = self.latency_samples.lock().unwrap();
            samples.push_back(latency_ms);
            
            // Limit sample size
            while samples.len() > self.config.max_samples_in_memory {
                samples.pop_front();
            }
        }

        // Update current metrics
        self.update_current_metrics();
        
        // Check for alerts
        if self.config.enable_real_time_alerting {
            self.check_alerts();
        }
    }

    /// Record network traffic
    pub fn record_network_traffic(&self, bytes_sent: u64, bytes_received: u64) {
        self.bytes_sent_counter.fetch_add(bytes_sent, Ordering::Relaxed);
        self.bytes_received_counter.fetch_add(bytes_received, Ordering::Relaxed);
    }

    /// Get current real-time metrics
    pub fn get_current_metrics(&self) -> RealTimeMetrics {
        self.current_metrics.lock().unwrap().clone()
    }

    /// Get aggregated metrics
    pub fn get_aggregated_metrics(&self) -> Vec<AggregatedMetrics> {
        self.aggregated_metrics.lock().unwrap().clone()
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
        self.active_alerts.lock().unwrap().clone()
    }

    /// Get system metrics history
    pub fn get_system_metrics(&self) -> Vec<SystemMetrics> {
        self.system_metrics.lock().unwrap().iter().cloned().collect()
    }

    /// Get memory profile history
    pub fn get_memory_profiles(&self) -> Vec<MemoryProfile> {
        self.memory_profiles.lock().unwrap().iter().cloned().collect()
    }

    /// Convert to performance measurements
    pub fn to_performance_measurements(&self, benchmark_name: String, category: String) -> Vec<PerformanceMeasurement> {
        let aggregated = self.get_aggregated_metrics();
        
        aggregated.into_iter().map(|agg| {
            let total_duration = agg.window_end.duration_since(agg.window_start)
                .unwrap_or_default();
            
            PerformanceMeasurement {
                benchmark_name: benchmark_name.clone(),
                category: category.clone(),
                operation_type: "collected_metrics".to_string(),
                operation_count: agg.total_operations as usize,
                total_duration,
                avg_operation_time_ms: agg.avg_latency_ms,
                median_operation_time_ms: agg.median_latency_ms,
                p95_operation_time_ms: agg.p95_latency_ms,
                p99_operation_time_ms: agg.p99_latency_ms,
                min_operation_time_ms: agg.min_latency_ms,
                max_operation_time_ms: agg.max_latency_ms,
                operations_per_second: agg.avg_throughput,
                error_count: agg.total_errors as usize,
                success_rate_percent: 100.0 - agg.error_rate_percent,
                memory_usage_mb: agg.avg_memory_usage_mb,
                cpu_usage_percent: agg.avg_cpu_usage_percent,
                additional_metrics: {
                    let mut metrics = HashMap::new();
                    metrics.insert("peak_throughput".to_string(), agg.peak_throughput);
                    metrics.insert("peak_memory_mb".to_string(), agg.peak_memory_usage_mb);
                    metrics.insert("peak_cpu_percent".to_string(), agg.peak_cpu_usage_percent);
                    metrics
                },
                timestamp: agg.window_start,
            }
        }).collect()
    }

    // Private methods

    /// Clone for async task
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            current_metrics: Arc::clone(&self.current_metrics),
            operation_counter: Arc::clone(&self.operation_counter),
            error_counter: Arc::clone(&self.error_counter),
            bytes_sent_counter: Arc::clone(&self.bytes_sent_counter),
            bytes_received_counter: Arc::clone(&self.bytes_received_counter),
            latency_samples: Arc::clone(&self.latency_samples),
            throughput_samples: Arc::clone(&self.throughput_samples),
            system_metrics: Arc::clone(&self.system_metrics),
            memory_profiles: Arc::clone(&self.memory_profiles),
            aggregated_metrics: Arc::clone(&self.aggregated_metrics),
            active_alerts: Arc::clone(&self.active_alerts),
            start_time: self.start_time,
            last_aggregation: Arc::clone(&self.last_aggregation),
            is_collecting: Arc::clone(&self.is_collecting),
        }
    }

    /// Main collection loop
    async fn collection_loop(&self) {
        let mut interval = interval(Duration::from_millis(self.config.collection_interval_ms));
        
        while *self.is_collecting.lock().unwrap() {
            interval.tick().await;
            
            // Update current metrics
            self.update_current_metrics();
            
            // Collect system metrics
            if self.config.collect_system_metrics {
                self.collect_system_metrics();
            }
            
            // Collect memory profile
            if self.config.enable_memory_profiling {
                self.collect_memory_profile();
            }
            
            // Aggregate metrics periodically
            let should_aggregate = {
                let last_agg = self.last_aggregation.lock().unwrap();
                last_agg.elapsed() >= Duration::from_secs(1) // Aggregate every second
            };
            
            if should_aggregate {
                self.aggregate_metrics();
                *self.last_aggregation.lock().unwrap() = Instant::now();
            }
        }
    }

    /// Update current real-time metrics
    fn update_current_metrics(&self) {
        let mut current = self.current_metrics.lock().unwrap();
        
        current.timestamp = SystemTime::now();
        
        // Calculate current throughput
        let elapsed = self.start_time.elapsed();
        let total_ops = self.operation_counter.load(Ordering::Relaxed);
        current.current_throughput = if elapsed.as_secs_f64() > 0.0 {
            total_ops as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        // Calculate current error rate
        let total_errors = self.error_counter.load(Ordering::Relaxed);
        current.current_error_rate_percent = if total_ops > 0 {
            (total_errors as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        };
        
        // Get latest latency
        {
            let samples = self.latency_samples.lock().unwrap();
            if let Some(&latest_latency) = samples.back() {
                current.current_latency_ms = latest_latency;
            }
        }
        
        // Update network metrics
        current.network_bytes_sent = self.bytes_sent_counter.load(Ordering::Relaxed);
        current.network_bytes_received = self.bytes_received_counter.load(Ordering::Relaxed);
    }

    /// Collect system metrics
    fn collect_system_metrics(&self) {
        let metrics = SystemMetrics {
            timestamp: SystemTime::now(),
            cpu_usage_percent: self.get_cpu_usage(),
            memory_usage_mb: self.get_memory_usage(),
            available_memory_mb: self.get_available_memory(),
            disk_read_bps: self.get_disk_read_rate(),
            disk_write_bps: self.get_disk_write_rate(),
            network_sent_bps: self.get_network_sent_rate(),
            network_received_bps: self.get_network_received_rate(),
            load_average_1m: self.get_load_average_1m(),
            load_average_5m: self.get_load_average_5m(),
            load_average_15m: self.get_load_average_15m(),
        };
        
        // Update current metrics
        {
            let mut current = self.current_metrics.lock().unwrap();
            current.cpu_usage_percent = metrics.cpu_usage_percent;
            current.memory_usage_mb = metrics.memory_usage_mb;
        }
        
        // Store in history
        {
            let mut system_metrics = self.system_metrics.lock().unwrap();
            system_metrics.push_back(metrics);
            
            // Limit history size
            while system_metrics.len() > self.config.max_samples_in_memory {
                system_metrics.pop_front();
            }
        }
    }

    /// Collect memory profile
    fn collect_memory_profile(&self) {
        let profile = MemoryProfile {
            timestamp: SystemTime::now(),
            heap_allocated_bytes: self.get_heap_allocated(),
            heap_capacity_bytes: self.get_heap_capacity(),
            stack_size_bytes: self.get_stack_size(),
            allocation_count: self.get_allocation_count(),
            deallocation_count: self.get_deallocation_count(),
            gc_count: self.get_gc_count(),
            gc_time_ms: self.get_gc_time(),
        };
        
        {
            let mut memory_profiles = self.memory_profiles.lock().unwrap();
            memory_profiles.push_back(profile);
            
            // Limit history size
            while memory_profiles.len() > self.config.max_samples_in_memory {
                memory_profiles.pop_front();
            }
        }
    }

    /// Aggregate metrics over time window
    fn aggregate_metrics(&self) {
        let window_size = self.config.aggregation_window_size;
        
        let (latency_stats, throughput_stats) = {
            let latency_samples = self.latency_samples.lock().unwrap();
            let throughput_samples = self.throughput_samples.lock().unwrap();
            
            if latency_samples.len() < window_size {
                return; // Not enough data
            }
            
            let recent_latencies: Vec<f64> = latency_samples.iter()
                .rev()
                .take(window_size)
                .cloned()
                .collect();
            
            let recent_throughputs: Vec<f64> = throughput_samples.iter()
                .rev()
                .take(window_size.min(throughput_samples.len()))
                .cloned()
                .collect();
            
            (recent_latencies, recent_throughputs)
        };
        
        let mut sorted_latencies = latency_stats.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let total_ops = self.operation_counter.load(Ordering::Relaxed) as u64;
        let total_errors = self.error_counter.load(Ordering::Relaxed) as u64;
        
        let avg_latency = latency_stats.iter().sum::<f64>() / latency_stats.len() as f64;
        let median_latency = sorted_latencies[sorted_latencies.len() / 2];
        let p95_latency = sorted_latencies[(sorted_latencies.len() * 95) / 100];
        let p99_latency = sorted_latencies[(sorted_latencies.len() * 99) / 100];
        
        let aggregated = AggregatedMetrics {
            window_start: SystemTime::now() - Duration::from_secs(1),
            window_end: SystemTime::now(),
            total_operations: total_ops,
            avg_latency_ms: avg_latency,
            median_latency_ms: median_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            max_latency_ms: sorted_latencies.last().copied().unwrap_or(0.0),
            min_latency_ms: sorted_latencies.first().copied().unwrap_or(0.0),
            avg_throughput: throughput_stats.iter().sum::<f64>() / throughput_stats.len().max(1) as f64,
            peak_throughput: throughput_stats.iter().fold(0.0, |a, &b| a.max(b)),
            total_errors,
            error_rate_percent: if total_ops > 0 { (total_errors as f64 / total_ops as f64) * 100.0 } else { 0.0 },
            avg_memory_usage_mb: self.get_avg_memory_usage(),
            peak_memory_usage_mb: self.get_peak_memory_usage(),
            avg_cpu_usage_percent: self.get_avg_cpu_usage(),
            peak_cpu_usage_percent: self.get_peak_cpu_usage(),
        };
        
        {
            let mut aggregated_metrics = self.aggregated_metrics.lock().unwrap();
            aggregated_metrics.push(aggregated);
            
            // Limit aggregated data
            while aggregated_metrics.len() > 1000 {
                aggregated_metrics.remove(0);
            }
        }
    }

    /// Check for performance alerts
    fn check_alerts(&self) {
        let current = self.get_current_metrics();
        let mut new_alerts = Vec::new();
        
        // Check latency threshold
        if current.current_latency_ms > self.config.alert_thresholds.max_latency_ms {
            new_alerts.push(PerformanceAlert {
                timestamp: SystemTime::now(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::HighLatency,
                message: format!("High latency detected: {:.2}ms > {:.2}ms", 
                    current.current_latency_ms, self.config.alert_thresholds.max_latency_ms),
                current_value: current.current_latency_ms,
                threshold: self.config.alert_thresholds.max_latency_ms,
                source: "metrics_collector".to_string(),
            });
        }
        
        // Check throughput threshold
        if current.current_throughput < self.config.alert_thresholds.min_throughput {
            new_alerts.push(PerformanceAlert {
                timestamp: SystemTime::now(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::LowThroughput,
                message: format!("Low throughput detected: {:.2} ops/s < {:.2} ops/s", 
                    current.current_throughput, self.config.alert_thresholds.min_throughput),
                current_value: current.current_throughput,
                threshold: self.config.alert_thresholds.min_throughput,
                source: "metrics_collector".to_string(),
            });
        }
        
        // Check error rate threshold
        if current.current_error_rate_percent > self.config.alert_thresholds.max_error_rate_percent {
            new_alerts.push(PerformanceAlert {
                timestamp: SystemTime::now(),
                severity: AlertSeverity::Critical,
                alert_type: AlertType::HighErrorRate,
                message: format!("High error rate detected: {:.2}% > {:.2}%", 
                    current.current_error_rate_percent, self.config.alert_thresholds.max_error_rate_percent),
                current_value: current.current_error_rate_percent,
                threshold: self.config.alert_thresholds.max_error_rate_percent,
                source: "metrics_collector".to_string(),
            });
        }
        
        // Check memory usage threshold
        if current.memory_usage_mb > self.config.alert_thresholds.max_memory_mb {
            new_alerts.push(PerformanceAlert {
                timestamp: SystemTime::now(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::HighMemoryUsage,
                message: format!("High memory usage detected: {:.2}MB > {:.2}MB", 
                    current.memory_usage_mb, self.config.alert_thresholds.max_memory_mb),
                current_value: current.memory_usage_mb,
                threshold: self.config.alert_thresholds.max_memory_mb,
                source: "metrics_collector".to_string(),
            });
        }
        
        // Check CPU usage threshold
        if current.cpu_usage_percent > self.config.alert_thresholds.max_cpu_percent {
            new_alerts.push(PerformanceAlert {
                timestamp: SystemTime::now(),
                severity: AlertSeverity::Warning,
                alert_type: AlertType::HighCpuUsage,
                message: format!("High CPU usage detected: {:.2}% > {:.2}%", 
                    current.cpu_usage_percent, self.config.alert_thresholds.max_cpu_percent),
                current_value: current.cpu_usage_percent,
                threshold: self.config.alert_thresholds.max_cpu_percent,
                source: "metrics_collector".to_string(),
            });
        }
        
        // Add new alerts
        if !new_alerts.is_empty() {
            let mut active_alerts = self.active_alerts.lock().unwrap();
            active_alerts.extend(new_alerts);
            
            // Limit alert history
            while active_alerts.len() > 100 {
                active_alerts.remove(0);
            }
        }
    }

    // System metric collection methods (simplified implementations)

    fn get_cpu_usage(&self) -> f64 {
        // Simplified CPU usage calculation
        std::process::Command::new("ps")
            .args(&["-A", "-o", "%cpu"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .skip(1)
                    .filter_map(|line| line.trim().parse::<f64>().ok())
                    .sum::<f64>()
            })
            .unwrap_or(0.0)
    }

    fn get_memory_usage(&self) -> f64 {
        // Simplified memory usage in MB
        std::process::Command::new("ps")
            .args(&["-A", "-o", "rss"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .skip(1)
                    .filter_map(|line| line.trim().parse::<u64>().ok())
                    .sum::<u64>() as f64 / 1024.0 // Convert KB to MB
            })
            .unwrap_or(0.0)
    }

    fn get_available_memory(&self) -> f64 {
        // Simplified available memory calculation
        1024.0 // Default 1GB available
    }

    fn get_disk_read_rate(&self) -> u64 { 0 }
    fn get_disk_write_rate(&self) -> u64 { 0 }
    fn get_network_sent_rate(&self) -> u64 { 0 }
    fn get_network_received_rate(&self) -> u64 { 0 }

    fn get_load_average_1m(&self) -> f64 {
        // Read from /proc/loadavg on Linux
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|content| content.split_whitespace().next()?.parse().ok())
            .unwrap_or(0.0)
    }

    fn get_load_average_5m(&self) -> f64 {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|content| content.split_whitespace().nth(1)?.parse().ok())
            .unwrap_or(0.0)
    }

    fn get_load_average_15m(&self) -> f64 {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|content| content.split_whitespace().nth(2)?.parse().ok())
            .unwrap_or(0.0)
    }

    fn get_heap_allocated(&self) -> u64 { 0 }
    fn get_heap_capacity(&self) -> u64 { 0 }
    fn get_stack_size(&self) -> u64 { 0 }
    fn get_allocation_count(&self) -> u64 { 0 }
    fn get_deallocation_count(&self) -> u64 { 0 }
    fn get_gc_count(&self) -> u64 { 0 }
    fn get_gc_time(&self) -> f64 { 0.0 }

    fn get_avg_memory_usage(&self) -> f64 {
        let system_metrics = self.system_metrics.lock().unwrap();
        if system_metrics.is_empty() {
            return 0.0;
        }
        system_metrics.iter().map(|m| m.memory_usage_mb).sum::<f64>() / system_metrics.len() as f64
    }

    fn get_peak_memory_usage(&self) -> f64 {
        let system_metrics = self.system_metrics.lock().unwrap();
        system_metrics.iter().map(|m| m.memory_usage_mb).fold(0.0, |a, b| a.max(b))
    }

    fn get_avg_cpu_usage(&self) -> f64 {
        let system_metrics = self.system_metrics.lock().unwrap();
        if system_metrics.is_empty() {
            return 0.0;
        }
        system_metrics.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / system_metrics.len() as f64
    }

    fn get_peak_cpu_usage(&self) -> f64 {
        let system_metrics = self.system_metrics.lock().unwrap();
        system_metrics.iter().map(|m| m.cpu_usage_percent).fold(0.0, |a, b| a.max(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsCollectorConfig::default();
        let collector = PerformanceMetricsCollector::new(config);
        
        let metrics = collector.get_current_metrics();
        assert_eq!(metrics.current_throughput, 0.0);
    }

    #[tokio::test]
    async fn test_operation_recording() {
        let config = MetricsCollectorConfig::default();
        let collector = PerformanceMetricsCollector::new(config);
        
        collector.record_operation(10.0, true);
        collector.record_operation(20.0, false);
        
        let metrics = collector.get_current_metrics();
        assert_eq!(metrics.current_error_rate_percent, 50.0);
    }

    #[test]
    fn test_metrics_conversion() {
        let config = MetricsCollectorConfig::default();
        let collector = PerformanceMetricsCollector::new(config);
        
        let measurements = collector.to_performance_measurements(
            "test_benchmark".to_string(),
            "test_category".to_string(),
        );
        
        assert!(measurements.is_empty()); // No aggregated data yet
    }
}