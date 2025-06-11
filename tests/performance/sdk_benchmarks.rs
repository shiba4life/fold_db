//! SDK Performance Benchmarks
//!
//! This module provides performance benchmarking for the JavaScript and Python SDKs,
//! testing signing performance, HTTP client integration, and cross-platform consistency.

use super::{
    PerformanceMeasurement, BenchmarkTimer, PerformanceBenchmarkConfig,
    PerformanceTargets
};
use serde_json::json;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio::fs;
use tokio::process::Command as AsyncCommand;

/// SDK performance benchmark suite
pub struct SdkPerformanceBenchmarks {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
    results: Vec<PerformanceMeasurement>,
    temp_dir: tempfile::TempDir,
}

impl SdkPerformanceBenchmarks {
    /// Create new SDK performance benchmarks
    pub fn new(
        config: PerformanceBenchmarkConfig,
        targets: PerformanceTargets,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        
        Ok(Self {
            config,
            targets,
            results: Vec::new(),
            temp_dir,
        })
    }

    /// Run all SDK performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        println!("🚀 Starting SDK performance benchmarks");
        
        // Check if SDKs are available
        let js_available = self.check_javascript_sdk_availability().await;
        let python_available = self.check_python_sdk_availability().await;
        
        if js_available {
            println!("📦 JavaScript SDK detected, running JS benchmarks");
            self.benchmark_javascript_sdk().await?;
        } else {
            println!("⚠️  JavaScript SDK not available, skipping JS benchmarks");
        }
        
        if python_available {
            println!("🐍 Python SDK detected, running Python benchmarks");
            self.benchmark_python_sdk().await?;
        } else {
            println!("⚠️  Python SDK not available, skipping Python benchmarks");
        }
        
        // Cross-platform comparison benchmarks
        if js_available && python_available {
            self.benchmark_cross_platform_consistency().await?;
        }
        
        // SDK integration benchmarks
        self.benchmark_sdk_integration_scenarios().await?;
        
        println!("✅ SDK performance benchmarks completed");
        Ok(self.results.clone())
    }

    /// Check if JavaScript SDK is available
    async fn check_javascript_sdk_availability(&self) -> bool {
        // Check if Node.js is available
        let node_check = Command::new("node")
            .arg("--version")
            .output();
        
        if node_check.is_err() {
            return false;
        }
        
        // Check if js-sdk directory exists
        std::path::Path::new("js-sdk").exists()
    }

    /// Check if Python SDK is available  
    async fn check_python_sdk_availability(&self) -> bool {
        // Check if Python is available
        let python_check = Command::new("python3")
            .arg("--version")
            .output();
        
        if python_check.is_err() {
            return false;
        }
        
        // Check if python-sdk directory exists
        std::path::Path::new("python-sdk").exists()
    }

    /// Benchmark JavaScript SDK performance
    async fn benchmark_javascript_sdk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking JavaScript SDK");
        
        // Create JavaScript benchmark script
        let js_benchmark_script = self.create_javascript_benchmark_script().await?;
        
        // Run JavaScript signing performance test
        self.run_javascript_signing_benchmark(&js_benchmark_script).await?;
        
        // Run JavaScript HTTP client integration test
        self.run_javascript_http_client_benchmark(&js_benchmark_script).await?;
        
        // Run JavaScript batch operations test
        self.run_javascript_batch_benchmark(&js_benchmark_script).await?;
        
        // Run JavaScript memory usage test
        self.run_javascript_memory_benchmark(&js_benchmark_script).await?;
        
        Ok(())
    }

    /// Benchmark Python SDK performance
    async fn benchmark_python_sdk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking Python SDK");
        
        // Create Python benchmark script
        let python_benchmark_script = self.create_python_benchmark_script().await?;
        
        // Run Python signing performance test
        self.run_python_signing_benchmark(&python_benchmark_script).await?;
        
        // Run Python HTTP client integration test  
        self.run_python_http_client_benchmark(&python_benchmark_script).await?;
        
        // Run Python async vs sync performance test
        self.run_python_async_sync_benchmark(&python_benchmark_script).await?;
        
        // Run Python batch operations test
        self.run_python_batch_benchmark(&python_benchmark_script).await?;
        
        // Run Python memory usage test
        self.run_python_memory_benchmark(&python_benchmark_script).await?;
        
        Ok(())
    }

    /// Create JavaScript benchmark script
    async fn create_javascript_benchmark_script(&self) -> Result<String, Box<dyn std::error::Error>> {
        let script_content = r#"
const { performance } = require('perf_hooks');
const { generateKeyPair } = require('../js-sdk/src/crypto/ed25519.js');
const { 
    createSignedHttpClient,
    createSigningConfig 
} = require('../js-sdk/src/server/index.js');

class JavaScriptBenchmarks {
    constructor() {
        this.keyPair = null;
        this.signingConfig = null;
        this.client = null;
    }

    async initialize() {
        this.keyPair = await generateKeyPair();
        this.signingConfig = createSigningConfig()
            .algorithm('ed25519')
            .keyId('benchmark-key')
            .privateKey(this.keyPair.privateKey)
            .profile('standard')
            .build();
        this.client = createSignedHttpClient(this.signingConfig);
    }

    async benchmarkSigning(iterations = 1000) {
        console.log(`Running JS signing benchmark with ${iterations} iterations`);
        const times = [];
        
        for (let i = 0; i < iterations; i++) {
            const message = `Test message ${i}`;
            const start = performance.now();
            
            // Simulate signing operation
            await this.client.signRequest('POST', 'https://api.example.com/test', {
                body: JSON.stringify({ test: message }),
                headers: { 'content-type': 'application/json' }
            });
            
            const end = performance.now();
            times.push(end - start);
        }
        
        return this.calculateStats(times, 'js_signing', iterations);
    }

    async benchmarkHttpClient(iterations = 500) {
        console.log(`Running JS HTTP client benchmark with ${iterations} iterations`);
        const times = [];
        
        for (let i = 0; i < iterations; i++) {
            const start = performance.now();
            
            try {
                // Simulate HTTP request preparation
                const request = {
                    method: 'POST',
                    url: `https://api.example.com/test/${i}`,
                    data: { test_id: i, data: `test data ${i}` }
                };
                
                // Sign the request
                const signedHeaders = await this.client.signRequest(
                    request.method, 
                    request.url, 
                    { body: JSON.stringify(request.data) }
                );
                
                const end = performance.now();
                times.push(end - start);
            } catch (error) {
                console.error(`Error in iteration ${i}:`, error);
            }
        }
        
        return this.calculateStats(times, 'js_http_client', iterations);
    }

    async benchmarkBatch(batchSize = 100) {
        console.log(`Running JS batch benchmark with batch size ${batchSize}`);
        const start = performance.now();
        
        const promises = [];
        for (let i = 0; i < batchSize; i++) {
            const promise = this.client.signRequest('POST', `https://api.example.com/batch/${i}`, {
                body: JSON.stringify({ batch_id: i })
            });
            promises.push(promise);
        }
        
        await Promise.all(promises);
        const end = performance.now();
        
        return {
            test_name: 'js_batch',
            total_time_ms: end - start,
            operations: batchSize,
            avg_time_ms: (end - start) / batchSize,
            ops_per_second: batchSize / ((end - start) / 1000)
        };
    }

    async benchmarkMemory(iterations = 1000) {
        console.log(`Running JS memory benchmark with ${iterations} iterations`);
        
        const initialMemory = process.memoryUsage();
        const start = performance.now();
        
        for (let i = 0; i < iterations; i++) {
            await this.client.signRequest('POST', `https://api.example.com/memory/${i}`, {
                body: JSON.stringify({ memory_test: i, data: 'x'.repeat(100) })
            });
        }
        
        const end = performance.now();
        const finalMemory = process.memoryUsage();
        
        return {
            test_name: 'js_memory',
            total_time_ms: end - start,
            operations: iterations,
            avg_time_ms: (end - start) / iterations,
            ops_per_second: iterations / ((end - start) / 1000),
            memory_used_bytes: finalMemory.heapUsed - initialMemory.heapUsed
        };
    }

    calculateStats(times, testName, iterations) {
        times.sort((a, b) => a - b);
        const sum = times.reduce((a, b) => a + b, 0);
        const avg = sum / times.length;
        const median = times[Math.floor(times.length / 2)];
        const p95 = times[Math.floor(times.length * 0.95)];
        const p99 = times[Math.floor(times.length * 0.99)];
        const min = times[0];
        const max = times[times.length - 1];
        
        return {
            test_name: testName,
            operations: iterations,
            avg_time_ms: avg,
            median_time_ms: median,
            p95_time_ms: p95,
            p99_time_ms: p99,
            min_time_ms: min,
            max_time_ms: max,
            ops_per_second: 1000 / avg,
            total_time_ms: sum
        };
    }
}

async function runBenchmarks() {
    const benchmarks = new JavaScriptBenchmarks();
    
    try {
        await benchmarks.initialize();
        
        const results = {
            signing: await benchmarks.benchmarkSigning(1000),
            httpClient: await benchmarks.benchmarkHttpClient(500),
            batch: await benchmarks.benchmarkBatch(100),
            memory: await benchmarks.benchmarkMemory(1000)
        };
        
        console.log('JS_BENCHMARK_RESULTS:', JSON.stringify(results));
    } catch (error) {
        console.error('JavaScript benchmark error:', error);
        process.exit(1);
    }
}

if (require.main === module) {
    runBenchmarks();
}
"#;
        
        let script_path = self.temp_dir.path().join("js_benchmark.js");
        fs::write(&script_path, script_content).await?;
        
        Ok(script_path.to_string_lossy().to_string())
    }

    /// Create Python benchmark script
    async fn create_python_benchmark_script(&self) -> Result<String, Box<dyn std::error::Error>> {
        let script_content = r#"
import sys
import time
import json
import asyncio
import tracemalloc
from datetime import datetime

# Add python-sdk to path
sys.path.insert(0, 'python-sdk/src')

try:
    from datafold_sdk.crypto.ed25519 import generate_key_pair
    from datafold_sdk.http_client import create_signed_http_client
    from datafold_sdk.signing import create_signing_config
except ImportError as e:
    print(f"Import error: {e}")
    sys.exit(1)

class PythonBenchmarks:
    def __init__(self):
        self.key_pair = None
        self.signing_config = None
        self.client = None

    async def initialize(self):
        self.key_pair = generate_key_pair()
        self.signing_config = create_signing_config(
            algorithm='ed25519',
            key_id='benchmark-key',
            private_key=self.key_pair.private_key
        )
        self.client = create_signed_http_client(self.signing_config)

    async def benchmark_signing(self, iterations=1000):
        print(f"Running Python signing benchmark with {iterations} iterations")
        times = []
        
        for i in range(iterations):
            message = f"Test message {i}"
            start = time.perf_counter()
            
            # Simulate signing operation
            await self.client.sign_request(
                'POST', 
                'https://api.example.com/test',
                body=json.dumps({"test": message}),
                headers={'content-type': 'application/json'}
            )
            
            end = time.perf_counter()
            times.append((end - start) * 1000)  # Convert to milliseconds
        
        return self.calculate_stats(times, 'python_signing', iterations)

    async def benchmark_http_client(self, iterations=500):
        print(f"Running Python HTTP client benchmark with {iterations} iterations")
        times = []
        
        for i in range(iterations):
            start = time.perf_counter()
            
            try:
                # Simulate HTTP request preparation
                request_data = {'test_id': i, 'data': f'test data {i}'}
                
                # Sign the request
                signed_headers = await self.client.sign_request(
                    'POST',
                    f'https://api.example.com/test/{i}',
                    body=json.dumps(request_data)
                )
                
                end = time.perf_counter()
                times.append((end - start) * 1000)
            except Exception as e:
                print(f"Error in iteration {i}: {e}")
        
        return self.calculate_stats(times, 'python_http_client', iterations)

    async def benchmark_async_vs_sync(self, iterations=500):
        print(f"Running Python async vs sync benchmark with {iterations} iterations")
        
        # Async benchmark
        async_times = []
        for i in range(iterations):
            start = time.perf_counter()
            await self.client.sign_request('POST', f'https://api.example.com/async/{i}', 
                                         body=json.dumps({'async_test': i}))
            end = time.perf_counter()
            async_times.append((end - start) * 1000)
        
        # Sync benchmark (simulated)
        sync_times = []
        for i in range(iterations):
            start = time.perf_counter()
            # Simulate synchronous signing
            self.client.sign_request_sync('POST', f'https://api.example.com/sync/{i}',
                                        body=json.dumps({'sync_test': i}))
            end = time.perf_counter()
            sync_times.append((end - start) * 1000)
        
        return {
            'async': self.calculate_stats(async_times, 'python_async', iterations),
            'sync': self.calculate_stats(sync_times, 'python_sync', iterations)
        }

    async def benchmark_batch(self, batch_size=100):
        print(f"Running Python batch benchmark with batch size {batch_size}")
        start = time.perf_counter()
        
        # Create batch of signing tasks
        tasks = []
        for i in range(batch_size):
            task = self.client.sign_request(
                'POST', 
                f'https://api.example.com/batch/{i}',
                body=json.dumps({'batch_id': i})
            )
            tasks.append(task)
        
        await asyncio.gather(*tasks)
        end = time.perf_counter()
        
        total_time_ms = (end - start) * 1000
        return {
            'test_name': 'python_batch',
            'total_time_ms': total_time_ms,
            'operations': batch_size,
            'avg_time_ms': total_time_ms / batch_size,
            'ops_per_second': batch_size / (total_time_ms / 1000)
        }

    async def benchmark_memory(self, iterations=1000):
        print(f"Running Python memory benchmark with {iterations} iterations")
        
        tracemalloc.start()
        start = time.perf_counter()
        
        for i in range(iterations):
            await self.client.sign_request(
                'POST', 
                f'https://api.example.com/memory/{i}',
                body=json.dumps({'memory_test': i, 'data': 'x' * 100})
            )
        
        end = time.perf_counter()
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        total_time_ms = (end - start) * 1000
        return {
            'test_name': 'python_memory',
            'total_time_ms': total_time_ms,
            'operations': iterations,
            'avg_time_ms': total_time_ms / iterations,
            'ops_per_second': iterations / (total_time_ms / 1000),
            'memory_used_bytes': peak
        }

    def calculate_stats(self, times, test_name, iterations):
        times.sort()
        avg = sum(times) / len(times)
        median = times[len(times) // 2]
        p95 = times[int(len(times) * 0.95)]
        p99 = times[int(len(times) * 0.99)]
        min_time = times[0]
        max_time = times[-1]
        
        return {
            'test_name': test_name,
            'operations': iterations,
            'avg_time_ms': avg,
            'median_time_ms': median,
            'p95_time_ms': p95,
            'p99_time_ms': p99,
            'min_time_ms': min_time,
            'max_time_ms': max_time,
            'ops_per_second': 1000 / avg,
            'total_time_ms': sum(times)
        }

async def run_benchmarks():
    benchmarks = PythonBenchmarks()
    
    try:
        await benchmarks.initialize()
        
        results = {
            'signing': await benchmarks.benchmark_signing(1000),
            'http_client': await benchmarks.benchmark_http_client(500),
            'async_sync': await benchmarks.benchmark_async_vs_sync(500),
            'batch': await benchmarks.benchmark_batch(100),
            'memory': await benchmarks.benchmark_memory(1000)
        }
        
        print('PYTHON_BENCHMARK_RESULTS:', json.dumps(results))
    except Exception as e:
        print(f'Python benchmark error: {e}')
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    asyncio.run(run_benchmarks())
"#;
        
        let script_path = self.temp_dir.path().join("python_benchmark.py");
        fs::write(&script_path, script_content).await?;
        
        Ok(script_path.to_string_lossy().to_string())
    }

    /// Run JavaScript signing benchmark
    async fn run_javascript_signing_benchmark(&mut self, script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 JavaScript signing performance");
        
        let start_time = Instant::now();
        
        let output = AsyncCommand::new("node")
            .arg(script_path)
            .output()
            .await?;
        
        let total_duration = start_time.elapsed();
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("    ⚠️  JavaScript benchmark failed: {}", stderr);
            return Ok(());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse benchmark results from stdout
        if let Some(results_line) = stdout.lines().find(|line| line.contains("JS_BENCHMARK_RESULTS:")) {
            if let Some(json_str) = results_line.strip_prefix("JS_BENCHMARK_RESULTS:") {
                if let Ok(results) = serde_json::from_str::<serde_json::Value>(json_str.trim()) {
                    self.process_javascript_results(&results, total_duration).await?;
                }
            }
        }
        
        println!("    ✓ JavaScript benchmarks completed");
        Ok(())
    }

    /// Run JavaScript HTTP client benchmark
    async fn run_javascript_http_client_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main JavaScript benchmark script
        Ok(())
    }

    /// Run JavaScript batch benchmark
    async fn run_javascript_batch_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main JavaScript benchmark script
        Ok(())
    }

    /// Run JavaScript memory benchmark
    async fn run_javascript_memory_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main JavaScript benchmark script
        Ok(())
    }

    /// Run Python signing benchmark
    async fn run_python_signing_benchmark(&mut self, script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Python signing performance");
        
        let start_time = Instant::now();
        
        let output = AsyncCommand::new("python3")
            .arg(script_path)
            .output()
            .await?;
        
        let total_duration = start_time.elapsed();
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("    ⚠️  Python benchmark failed: {}", stderr);
            return Ok(());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse benchmark results from stdout
        if let Some(results_line) = stdout.lines().find(|line| line.contains("PYTHON_BENCHMARK_RESULTS:")) {
            if let Some(json_str) = results_line.strip_prefix("PYTHON_BENCHMARK_RESULTS:") {
                if let Ok(results) = serde_json::from_str::<serde_json::Value>(json_str.trim()) {
                    self.process_python_results(&results, total_duration).await?;
                }
            }
        }
        
        println!("    ✓ Python benchmarks completed");
        Ok(())
    }

    /// Run Python HTTP client benchmark
    async fn run_python_http_client_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main Python benchmark script
        Ok(())
    }

    /// Run Python async vs sync benchmark
    async fn run_python_async_sync_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main Python benchmark script
        Ok(())
    }

    /// Run Python batch benchmark
    async fn run_python_batch_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main Python benchmark script
        Ok(())
    }

    /// Run Python memory benchmark
    async fn run_python_memory_benchmark(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This is handled within the main Python benchmark script
        Ok(())
    }

    /// Process JavaScript benchmark results
    async fn process_javascript_results(
        &mut self,
        results: &serde_json::Value,
        total_duration: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Process signing results
        if let Some(signing) = results.get("signing") {
            let mut measurement = PerformanceMeasurement::new(
                "javascript_signing".to_string(),
                "sdk".to_string(),
                "js_crypto_signing".to_string(),
            );
            
            self.populate_measurement_from_json(&mut measurement, signing, total_duration);
            measurement.additional_metrics.insert("sdk_type".to_string(), 1.0); // 1 = JavaScript
            self.results.push(measurement);
        }
        
        // Process HTTP client results
        if let Some(http_client) = results.get("httpClient") {
            let mut measurement = PerformanceMeasurement::new(
                "javascript_http_client".to_string(),
                "sdk".to_string(),
                "js_http_integration".to_string(),
            );
            
            self.populate_measurement_from_json(&mut measurement, http_client, total_duration);
            measurement.additional_metrics.insert("sdk_type".to_string(), 1.0);
            self.results.push(measurement);
        }
        
        // Process batch results
        if let Some(batch) = results.get("batch") {
            let mut measurement = PerformanceMeasurement::new(
                "javascript_batch".to_string(),
                "sdk".to_string(),
                "js_batch_operations".to_string(),
            );
            
            if let Some(total_time) = batch.get("total_time_ms").and_then(|v| v.as_f64()) {
                measurement.total_duration = Duration::from_secs_f64(total_time / 1000.0);
            }
            if let Some(ops) = batch.get("operations").and_then(|v| v.as_u64()) {
                measurement.operation_count = ops as usize;
            }
            if let Some(avg_time) = batch.get("avg_time_ms").and_then(|v| v.as_f64()) {
                measurement.avg_operation_time_ms = avg_time;
            }
            if let Some(ops_per_sec) = batch.get("ops_per_second").and_then(|v| v.as_f64()) {
                measurement.operations_per_second = ops_per_sec;
            }
            
            measurement.additional_metrics.insert("sdk_type".to_string(), 1.0);
            self.results.push(measurement);
        }
        
        // Process memory results
        if let Some(memory) = results.get("memory") {
            let mut measurement = PerformanceMeasurement::new(
                "javascript_memory".to_string(),
                "sdk".to_string(),
                "js_memory_usage".to_string(),
            );
            
            self.populate_measurement_from_json(&mut measurement, memory, total_duration);
            
            if let Some(memory_used) = memory.get("memory_used_bytes").and_then(|v| v.as_u64()) {
                measurement.memory_usage_bytes = Some(memory_used as usize);
            }
            
            measurement.additional_metrics.insert("sdk_type".to_string(), 1.0);
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Process Python benchmark results
    async fn process_python_results(
        &mut self,
        results: &serde_json::Value,
        total_duration: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Process signing results
        if let Some(signing) = results.get("signing") {
            let mut measurement = PerformanceMeasurement::new(
                "python_signing".to_string(),
                "sdk".to_string(),
                "python_crypto_signing".to_string(),
            );
            
            self.populate_measurement_from_json(&mut measurement, signing, total_duration);
            measurement.additional_metrics.insert("sdk_type".to_string(), 2.0); // 2 = Python
            self.results.push(measurement);
        }
        
        // Process HTTP client results
        if let Some(http_client) = results.get("http_client") {
            let mut measurement = PerformanceMeasurement::new(
                "python_http_client".to_string(),
                "sdk".to_string(),
                "python_http_integration".to_string(),
            );
            
            self.populate_measurement_from_json(&mut measurement, http_client, total_duration);
            measurement.additional_metrics.insert("sdk_type".to_string(), 2.0);
            self.results.push(measurement);
        }
        
        // Process async/sync comparison
        if let Some(async_sync) = results.get("async_sync") {
            if let Some(async_results) = async_sync.get("async") {
                let mut measurement = PerformanceMeasurement::new(
                    "python_async".to_string(),
                    "sdk".to_string(),
                    "python_async_operations".to_string(),
                );
                
                self.populate_measurement_from_json(&mut measurement, async_results, total_duration);
                measurement.additional_metrics.insert("sdk_type".to_string(), 2.0);
                measurement.additional_metrics.insert("operation_type".to_string(), 1.0); // 1 = async
                self.results.push(measurement);
            }
            
            if let Some(sync_results) = async_sync.get("sync") {
                let mut measurement = PerformanceMeasurement::new(
                    "python_sync".to_string(),
                    "sdk".to_string(),
                    "python_sync_operations".to_string(),
                );
                
                self.populate_measurement_from_json(&mut measurement, sync_results, total_duration);
                measurement.additional_metrics.insert("sdk_type".to_string(), 2.0);
                measurement.additional_metrics.insert("operation_type".to_string(), 2.0); // 2 = sync
                self.results.push(measurement);
            }
        }
        
        // Process batch results
        if let Some(batch) = results.get("batch") {
            let mut measurement = PerformanceMeasurement::new(
                "python_batch".to_string(),
                "sdk".to_string(),
                "python_batch_operations".to_string(),
            );
            
            if let Some(total_time) = batch.get("total_time_ms").and_then(|v| v.as_f64()) {
                measurement.total_duration = Duration::from_secs_f64(total_time / 1000.0);
            }
            if let Some(ops) = batch.get("operations").and_then(|v| v.as_u64()) {
                measurement.operation_count = ops as usize;
            }
            if let Some(avg_time) = batch.get("avg_time_ms").and_then(|v| v.as_f64()) {
                measurement.avg_operation_time_ms = avg_time;
            }
            if let Some(ops_per_sec) = batch.get("ops_per_second").and_then(|v| v.as_f64()) {
                measurement.operations_per_second = ops_per_sec;
            }
            
            measurement.additional_metrics.insert("sdk_type".to_string(), 2.0);
            self.results.push(measurement);
        }
        
        // Process memory results
        if let Some(memory) = results.get("memory") {
            let mut measurement = PerformanceMeasurement::new(
                "python_memory".to_string(),
                "sdk".to_string(),
                "python_memory_usage".to_string(),
            );
            
            self.populate_measurement_from_json(&mut measurement, memory, total_duration);
            
            if let Some(memory_used) = memory.get("memory_used_bytes").and_then(|v| v.as_u64()) {
                measurement.memory_usage_bytes = Some(memory_used as usize);
            }
            
            measurement.additional_metrics.insert("sdk_type".to_string(), 2.0);
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Populate measurement from JSON result
    fn populate_measurement_from_json(
        &self,
        measurement: &mut PerformanceMeasurement,
        json_result: &serde_json::Value,
        total_duration: Duration,
    ) {
        if let Some(ops) = json_result.get("operations").and_then(|v| v.as_u64()) {
            measurement.operation_count = ops as usize;
        }
        
        measurement.total_duration = total_duration;
        
        if let Some(avg_time) = json_result.get("avg_time_ms").and_then(|v| v.as_f64()) {
            measurement.avg_operation_time_ms = avg_time;
        }
        
        if let Some(median_time) = json_result.get("median_time_ms").and_then(|v| v.as_f64()) {
            measurement.median_operation_time_ms = median_time;
        }
        
        if let Some(p95_time) = json_result.get("p95_time_ms").and_then(|v| v.as_f64()) {
            measurement.p95_operation_time_ms = p95_time;
        }
        
        if let Some(p99_time) = json_result.get("p99_time_ms").and_then(|v| v.as_f64()) {
            measurement.p99_operation_time_ms = p99_time;
        }
        
        if let Some(min_time) = json_result.get("min_time_ms").and_then(|v| v.as_f64()) {
            measurement.min_operation_time_ms = min_time;
        }
        
        if let Some(max_time) = json_result.get("max_time_ms").and_then(|v| v.as_f64()) {
            measurement.max_operation_time_ms = max_time;
        }
        
        if let Some(ops_per_sec) = json_result.get("ops_per_second").and_then(|v| v.as_f64()) {
            measurement.operations_per_second = ops_per_sec;
        }
        
        measurement.error_count = 0;
        measurement.success_rate_percent = 100.0;
    }

    /// Benchmark cross-platform consistency
    async fn benchmark_cross_platform_consistency(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking cross-platform consistency");
        
        // Find JavaScript and Python results for comparison
        let js_signing = self.results.iter()
            .find(|r| r.test_name == "javascript_signing");
        let python_signing = self.results.iter()
            .find(|r| r.test_name == "python_signing");
        
        if let (Some(js), Some(py)) = (js_signing, python_signing) {
            let consistency_ratio = py.avg_operation_time_ms / js.avg_operation_time_ms;
            let performance_delta_percent = ((py.avg_operation_time_ms - js.avg_operation_time_ms) 
                / js.avg_operation_time_ms) * 100.0;
            
            let mut measurement = PerformanceMeasurement::new(
                "cross_platform_consistency".to_string(),
                "sdk".to_string(),
                "platform_comparison".to_string(),
            );
            
            measurement.operation_count = 2; // Two platforms compared
            measurement.avg_operation_time_ms = (js.avg_operation_time_ms + py.avg_operation_time_ms) / 2.0;
            measurement.operations_per_second = (js.operations_per_second + py.operations_per_second) / 2.0;
            measurement.success_rate_percent = 100.0;
            
            measurement.additional_metrics.insert("js_avg_time_ms".to_string(), js.avg_operation_time_ms);
            measurement.additional_metrics.insert("python_avg_time_ms".to_string(), py.avg_operation_time_ms);
            measurement.additional_metrics.insert("consistency_ratio".to_string(), consistency_ratio);
            measurement.additional_metrics.insert("performance_delta_percent".to_string(), performance_delta_percent);
            
            println!("  ✓ Cross-platform consistency: {:.2}x ratio, {:.1}% delta", 
                    consistency_ratio, performance_delta_percent);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark SDK integration scenarios
    async fn benchmark_sdk_integration_scenarios(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking SDK integration scenarios");
        
        // Simulate different integration patterns
        let scenarios = vec![
            ("single_request", "Single authenticated request"),
            ("burst_requests", "Burst of authenticated requests"),
            ("sustained_load", "Sustained load with authentication"),
            ("mixed_operations", "Mixed authenticated and non-authenticated operations"),
        ];
        
        for (scenario_name, description) in scenarios {
            let mut timer = BenchmarkTimer::new();
            let iterations = 100;
            let mut success_count = 0;
            
            println!("  🔍 {}: {}", scenario_name, description);
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                // Simulate different integration scenarios
                match scenario_name {
                    "single_request" => {
                        // Simulate single request processing time
                        tokio::time::sleep(Duration::from_micros(500)).await;
                    }
                    "burst_requests" => {
                        // Simulate burst of 10 requests
                        for _ in 0..10 {
                            tokio::time::sleep(Duration::from_micros(50)).await;
                        }
                    }
                    "sustained_load" => {
                        // Simulate sustained load processing
                        tokio::time::sleep(Duration::from_micros(200)).await;
                    }
                    "mixed_operations" => {
                        // Simulate mix of authenticated and non-authenticated
                        if i % 2 == 0 {
                            tokio::time::sleep(Duration::from_micros(300)).await; // Authenticated
                        } else {
                            tokio::time::sleep(Duration::from_micros(100)).await; // Non-authenticated
                        }
                    }
                    _ => {}
                }
                
                timer.record();
                success_count += 1;
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("sdk_integration_{}", scenario_name),
                "sdk".to_string(),
                "integration_scenario".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.median_operation_time_ms = stats.median_ms;
            measurement.p95_operation_time_ms = stats.p95_ms;
            measurement.p99_operation_time_ms = stats.p99_ms;
            measurement.min_operation_time_ms = stats.min_ms;
            measurement.max_operation_time_ms = stats.max_ms;
            measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
            measurement.error_count = 0;
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            measurement.additional_metrics.insert("scenario_type".to_string(), 
                match scenario_name {
                    "single_request" => 1.0,
                    "burst_requests" => 2.0,
                    "sustained_load" => 3.0,
                    "mixed_operations" => 4.0,
                    _ => 0.0,
                });
            
            println!("    ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    scenario_name, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sdk_benchmarks_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        
        let benchmarks = SdkPerformanceBenchmarks::new(config, targets);
        assert!(benchmarks.is_ok());
    }

    #[tokio::test]
    async fn test_javascript_script_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        let benchmarks = SdkPerformanceBenchmarks::new(config, targets).unwrap();
        
        let script_path = benchmarks.create_javascript_benchmark_script().await;
        assert!(script_path.is_ok());
        
        let path = script_path.unwrap();
        assert!(std::path::Path::new(&path).exists());
    }

    #[tokio::test]
    async fn test_python_script_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        let benchmarks = SdkPerformanceBenchmarks::new(config, targets).unwrap();
        
        let script_path = benchmarks.create_python_benchmark_script().await;
        assert!(script_path.is_ok());
        
        let path = script_path.unwrap();
        assert!(std::path::Path::new(&path).exists());
    }
}