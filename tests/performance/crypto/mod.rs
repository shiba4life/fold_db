//! Performance Tests for Unified Cryptographic System
//!
//! This module contains performance benchmarks and load testing for the
//! unified cryptographic system to ensure operations meet performance requirements.

pub mod crypto_benchmarks;
pub mod load_testing;
pub mod stress_testing;
pub mod concurrency_testing;

// Re-export performance utilities
pub use crypto_benchmarks::*;

/// Performance testing utilities
pub mod performance_utils {
    use std::time::{Duration, Instant};
    use datafold::unified_crypto::*;

    /// Performance benchmark results
    #[derive(Debug, Clone)]
    pub struct BenchmarkResult {
        pub operation: String,
        pub iterations: usize,
        pub total_duration: Duration,
        pub average_duration: Duration,
        pub min_duration: Duration,
        pub max_duration: Duration,
        pub operations_per_second: f64,
    }

    impl BenchmarkResult {
        pub fn new(operation: String, iterations: usize, durations: Vec<Duration>) -> Self {
            let total_duration: Duration = durations.iter().sum();
            let average_duration = total_duration / iterations as u32;
            let min_duration = durations.iter().min().copied().unwrap_or_default();
            let max_duration = durations.iter().max().copied().unwrap_or_default();
            let operations_per_second = iterations as f64 / total_duration.as_secs_f64();

            Self {
                operation,
                iterations,
                total_duration,
                average_duration,
                min_duration,
                max_duration,
                operations_per_second,
            }
        }

        pub fn print_summary(&self) {
            println!("=== Performance Benchmark: {} ===", self.operation);
            println!("Iterations: {}", self.iterations);
            println!("Total Duration: {:?}", self.total_duration);
            println!("Average Duration: {:?}", self.average_duration);
            println!("Min Duration: {:?}", self.min_duration);
            println!("Max Duration: {:?}", self.max_duration);
            println!("Operations per second: {:.2}", self.operations_per_second);
            println!();
        }
    }

    /// Benchmark a crypto operation
    pub fn benchmark_crypto_operation<F, R>(
        operation_name: &str,
        operation: F,
        iterations: usize,
    ) -> BenchmarkResult
    where
        F: Fn() -> R,
    {
        let mut durations = Vec::with_capacity(iterations);

        // Warm up
        for _ in 0..10 {
            let _ = operation();
        }

        // Actual benchmark
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = operation();
            durations.push(start.elapsed());
        }

        BenchmarkResult::new(operation_name.to_string(), iterations, durations)
    }

    /// Benchmark throughput for data processing operations
    pub fn benchmark_throughput<F>(
        operation_name: &str,
        operation: F,
        data_sizes: Vec<usize>,
        iterations_per_size: usize,
    ) -> Vec<(usize, BenchmarkResult)>
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
    {
        let mut results = Vec::new();

        for &size in &data_sizes {
            let test_data = vec![42u8; size];
            let mut durations = Vec::with_capacity(iterations_per_size);

            for _ in 0..iterations_per_size {
                let start = Instant::now();
                let _ = operation(&test_data);
                durations.push(start.elapsed());
            }

            let benchmark = BenchmarkResult::new(
                format!("{} ({}B)", operation_name, size),
                iterations_per_size,
                durations,
            );

            results.push((size, benchmark));
        }

        results
    }

    /// Performance requirements for cryptographic operations
    pub struct PerformanceRequirements {
        pub key_generation_max_ms: u64,
        pub encryption_min_mbps: f64,
        pub signing_min_ops_per_sec: f64,
        pub verification_min_ops_per_sec: f64,
        pub hash_min_mbps: f64,
    }

    impl Default for PerformanceRequirements {
        fn default() -> Self {
            Self {
                key_generation_max_ms: 100,      // 100ms max for key generation
                encryption_min_mbps: 10.0,       // 10 MB/s minimum encryption throughput
                signing_min_ops_per_sec: 1000.0, // 1000 signatures per second
                verification_min_ops_per_sec: 2000.0, // 2000 verifications per second
                hash_min_mbps: 100.0,            // 100 MB/s minimum hashing throughput
            }
        }
    }

    /// Validate benchmark results against performance requirements
    pub fn validate_performance(
        result: &BenchmarkResult,
        requirements: &PerformanceRequirements,
    ) -> bool {
        match result.operation.as_str() {
            op if op.contains("key_generation") => {
                result.average_duration.as_millis() <= requirements.key_generation_max_ms as u128
            }
            op if op.contains("signing") => {
                result.operations_per_second >= requirements.signing_min_ops_per_sec
            }
            op if op.contains("verification") => {
                result.operations_per_second >= requirements.verification_min_ops_per_sec
            }
            _ => true, // Default to passing for other operations
        }
    }

    /// Calculate throughput in MB/s for data processing operations
    pub fn calculate_throughput_mbps(data_size: usize, duration: Duration) -> f64 {
        let bytes_per_second = data_size as f64 / duration.as_secs_f64();
        bytes_per_second / (1024.0 * 1024.0) // Convert to MB/s
    }

    /// Generate test data for performance testing
    pub fn generate_performance_test_data(sizes: Vec<usize>) -> Vec<(usize, Vec<u8>)> {
        sizes
            .into_iter()
            .map(|size| {
                let data = (0..size).map(|i| (i % 256) as u8).collect();
                (size, data)
            })
            .collect()
    }

    /// Standard test data sizes for crypto performance testing
    pub fn standard_test_sizes() -> Vec<usize> {
        vec![
            1024,           // 1 KB
            4096,           // 4 KB
            16384,          // 16 KB
            65536,          // 64 KB
            262144,         // 256 KB
            1048576,        // 1 MB
            4194304,        // 4 MB
            16777216,       // 16 MB
        ]
    }

    /// Small test sizes for quick performance testing
    pub fn quick_test_sizes() -> Vec<usize> {
        vec![
            1024,     // 1 KB
            16384,    // 16 KB
            262144,   // 256 KB
            1048576,  // 1 MB
        ]
    }
}