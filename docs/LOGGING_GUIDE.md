# DataFold Logging System Guide

**Version**: 1.0  
**Last Updated**: June 2025  
**Author**: Engineering Team

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Configuration](#configuration)
4. [Feature-Specific Logging](#feature-specific-logging)
5. [Output Types](#output-types)
6. [Runtime Management](#runtime-management)
7. [Performance Monitoring](#performance-monitoring)
8. [Migration Guide](#migration-guide)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)
11. [API Reference](#api-reference)

---

## Overview

The DataFold logging system provides comprehensive, configurable logging capabilities with feature-specific filtering, multiple output formats, and runtime management. Built on top of Rust's `log` and `tracing` crates, it offers both backward compatibility and advanced features for production deployments.

### Key Features

- **Feature-Specific Logging**: Target specific components (transform, network, schema, etc.)
- **Multiple Outputs**: Console, file, web streaming, and structured JSON
- **Runtime Configuration**: Change log levels without restarting
- **Performance Monitoring**: Built-in timing and metrics
- **Environment Integration**: Environment variable overrides
- **Backward Compatibility**: Works with existing web_logger

### Supported Log Levels

- **TRACE**: Very detailed diagnostic information
- **DEBUG**: Detailed information for debugging
- **INFO**: General informational messages
- **WARN**: Warning messages for potential issues
- **ERROR**: Error conditions that need attention

---

## Quick Start

### Basic Setup

1. **Initialize logging in your application:**

```rust
use fold_node::logging::LoggingSystem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with default configuration
    LoggingSystem::init_default().await?;
    
    // Your application code here
    log::info!("Application started");
    
    Ok(())
}
```

2. **Use feature-specific logging:**

```rust
use fold_node::{log_transform_info, log_network_debug, log_schema_error};

// Transform-specific logging
log_transform_info!("Starting data transformation: {}", operation_name);

// Network-specific logging
log_network_debug!("Received connection from {}", peer_addr);

// Schema-specific logging
log_schema_error!("Schema validation failed: {}", error_msg);
```

3. **Access logs via HTTP API:**

```bash
# Get current logs
curl http://localhost:9001/api/logs

# Stream logs in real-time
curl http://localhost:9001/api/logs/stream

# Get current configuration
curl http://localhost:9001/api/logs/config
```

---

## Configuration

### Configuration File

The logging system uses TOML configuration files. Default location: [`config/logging.toml`](../config/logging.toml)

```toml
[general]
default_level = "INFO"
enable_colors = true
enable_correlation_ids = true
max_correlation_id_length = 64

[outputs.console]
enabled = true
level = "INFO"
colors = true
include_timestamp = true
include_module = true
include_thread = false

[outputs.file]
enabled = false
path = "logs/datafold.log"
level = "DEBUG"
max_size = "10MB"
max_files = 5
include_timestamp = true
include_module = true
include_thread = true

[outputs.web]
enabled = true
level = "INFO"
buffer_size = 1000
enable_filtering = true
max_logs = 5000

[outputs.structured]
enabled = false
level = "DEBUG"
path = "logs/datafold-structured.json"
include_context = true
include_metrics = false

[features]
transform = "DEBUG"
network = "INFO"
database = "WARN"
schema = "INFO"
query = "INFO"
mutation = "INFO"
permissions = "INFO"
http_server = "INFO"
tcp_server = "INFO"
ingestion = "INFO"
```

### Environment Variables

Override configuration using environment variables:

```bash
# General settings
export DATAFOLD_LOG_LEVEL=DEBUG
export DATAFOLD_LOG_COLORS=true

# Console output
export DATAFOLD_LOG_CONSOLE_ENABLED=true
export DATAFOLD_LOG_CONSOLE_LEVEL=INFO

# File output
export DATAFOLD_LOG_FILE_ENABLED=true
export DATAFOLD_LOG_FILE_PATH=/var/log/datafold.log
export DATAFOLD_LOG_FILE_LEVEL=DEBUG

# Web output
export DATAFOLD_LOG_WEB_ENABLED=true
export DATAFOLD_LOG_WEB_LEVEL=INFO

# Feature-specific levels
export DATAFOLD_LOG_FEATURE_TRANSFORM=TRACE
export DATAFOLD_LOG_FEATURE_NETWORK=DEBUG
export DATAFOLD_LOG_FEATURE_SCHEMA=WARN
```

### Loading Configuration

```rust
use fold_node::logging::{LoggingSystem, config::LogConfig};

// Load from file
let config = LogConfig::from_file("config/logging.toml")?;
LoggingSystem::init_with_config(config).await?;

// Load from environment only
let config = LogConfig::from_env()?;
LoggingSystem::init_with_config(config).await?;

// Use defaults
LoggingSystem::init_default().await?;
```

---

## Feature-Specific Logging

### Available Features

The logging system provides dedicated macros for different DataFold components:

| Feature | Target | Macros | Use Cases |
|---------|--------|--------|-----------|
| `transform` | `datafold_node::transform` | `log_transform_*!` | Data transformations, DSL execution |
| `network` | `datafold_node::network` | `log_network_*!` | P2P networking, peer discovery |
| `database` | `datafold_node::database` | Standard `log::*!` | Database operations, storage |
| `schema` | `datafold_node::schema` | `log_schema_*!` | Schema validation, management |
| `query` | `datafold_node::query` | Standard `log::*!` | Query execution, optimization |
| `mutation` | `datafold_node::mutation` | Standard `log::*!` | Data mutations, updates |
| `permissions` | `datafold_node::permissions` | Standard `log::*!` | Access control, authorization |
| `http_server` | `datafold_node::http_server` | `log_http_*!` | HTTP API, web interface |
| `tcp_server` | `datafold_node::tcp_server` | Standard `log::*!` | TCP protocol, connections |
| `ingestion` | `datafold_node::ingestion` | Standard `log::*!` | Data ingestion, processing |

### Using Feature-Specific Macros

```rust
use fold_node::{
    log_transform_debug, log_transform_info, log_transform_warn, log_transform_error,
    log_network_debug, log_network_info, log_network_warn, log_network_error,
    log_schema_debug, log_schema_info, log_schema_warn, log_schema_error,
    log_http_debug, log_http_info, log_http_warn, log_http_error,
};

// Transform operations
log_transform_debug!("Parsing transform expression: {}", expr);
log_transform_info!("Transform completed successfully in {}ms", duration);
log_transform_warn!("Transform used deprecated function: {}", func_name);
log_transform_error!("Transform failed: {}", error);

// Network operations
log_network_debug!("Sending heartbeat to peer {}", peer_id);
log_network_info!("Connected to {} peers", peer_count);
log_network_warn!("Peer {} unresponsive", peer_id);
log_network_error!("Network connection failed: {}", error);

// Schema operations
log_schema_debug!("Validating field: {}", field_name);
log_schema_info!("Schema {} loaded successfully", schema_name);
log_schema_warn!("Schema {} has deprecated fields", schema_name);
log_schema_error!("Schema validation failed: {}", error);

// HTTP server operations
log_http_debug!("Processing request: {} {}", method, path);
log_http_info!("Request completed: {} {} - {} in {}ms", method, path, status, duration);
log_http_warn!("Slow request: {} {} took {}ms", method, path, duration);
log_http_error!("Request failed: {} {} - {}", method, path, error);
```

### Debugging Specific Features

To debug a specific feature, temporarily increase its log level:

```bash
# Via HTTP API
curl -X POST http://localhost:9001/api/logs/features \
  -H "Content-Type: application/json" \
  -d '{"feature": "transform", "level": "TRACE"}'

# Via environment variable (requires restart)
export DATAFOLD_LOG_FEATURE_TRANSFORM=TRACE
```

---

## Output Types

### Console Output

**Purpose**: Real-time development and debugging  
**Features**: Colored output, configurable formatting

```toml
[outputs.console]
enabled = true
level = "INFO"
colors = true
include_timestamp = true
include_module = true
include_thread = false
```

**Example Output:**
```
2025-06-02T21:13:03.456Z INFO datafold_node::transform: Transform completed successfully operation=user_score_calc duration=45ms
2025-06-02T21:13:03.457Z DEBUG datafold_node::network: Sending heartbeat to peer peer_id=node_abc123
```

### File Output

**Purpose**: Persistent logging for production environments  
**Features**: Log rotation, detailed formatting

```toml
[outputs.file]
enabled = true
path = "logs/datafold.log"
level = "DEBUG"
max_size = "10MB"
max_files = 5
include_timestamp = true
include_module = true
include_thread = true
```

**Log Rotation**: Files are rotated daily or when size limit is reached:
- `datafold.log` (current)
- `datafold.log.1` (previous day)
- `datafold.log.2` (older)
- ... up to `max_files`

### Web Streaming Output

**Purpose**: Real-time monitoring via web interfaces  
**Features**: Browser-compatible streaming, filtering

```toml
[outputs.web]
enabled = true
level = "INFO"
buffer_size = 1000
enable_filtering = true
max_logs = 5000
```

**Usage:**
```javascript
// Subscribe to log stream
const eventSource = new EventSource('/api/logs/stream');
eventSource.onmessage = function(event) {
    const logLine = event.data;
    console.log(logLine);
};
```

### Structured JSON Output

**Purpose**: Integration with monitoring tools (ELK, Splunk, etc.)  
**Features**: Machine-readable format, rich metadata

```toml
[outputs.structured]
enabled = true
level = "DEBUG"
path = "logs/datafold-structured.json"
include_context = true
include_metrics = true
```

**Example Output:**
```json
{
  "timestamp": "2025-06-02T21:13:03.456Z",
  "level": "INFO",
  "target": "datafold_node::transform",
  "message": "Transform completed successfully",
  "fields": {
    "operation": "user_score_calc",
    "duration": "45ms"
  },
  "span": {
    "name": "execute_transform",
    "correlation_id": "req_abc123"
  }
}
```

---

## Runtime Management

### HTTP API Endpoints

#### Get Current Configuration
```bash
curl http://localhost:9001/api/logs/config
```

**Response:**
```json
{
  "config": {
    "general": {
      "default_level": "INFO",
      "enable_colors": true,
      "enable_correlation_ids": true,
      "max_correlation_id_length": 64
    },
    "outputs": { ... },
    "features": { ... }
  }
}
```

#### Update Feature Log Level
```bash
curl -X POST http://localhost:9001/api/logs/features \
  -H "Content-Type: application/json" \
  -d '{"feature": "transform", "level": "DEBUG"}'
```

**Response:**
```json
{
  "success": true,
  "message": "Updated transform log level to DEBUG"
}
```

#### Get Available Features
```bash
curl http://localhost:9001/api/logs/features
```

**Response:**
```json
{
  "features": {
    "transform": "DEBUG",
    "network": "INFO",
    "schema": "INFO",
    ...
  },
  "available_levels": ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]
}
```

#### Reload Configuration
```bash
curl -X POST http://localhost:9001/api/logs/reload
```

**Response:**
```json
{
  "success": true,
  "message": "Configuration reloaded successfully"
}
```

#### Get Current Logs
```bash
curl http://localhost:9001/api/logs
```

**Response:**
```json
[
  "2025-06-02T21:13:03.456Z INFO Transform completed successfully",
  "2025-06-02T21:13:03.457Z DEBUG Sending heartbeat to peer",
  ...
]
```

#### Stream Logs (Server-Sent Events)
```bash
curl http://localhost:9001/api/logs/stream
```

**Response:**
```
data: 2025-06-02T21:13:03.456Z INFO Transform completed successfully

data: 2025-06-02T21:13:03.457Z DEBUG Sending heartbeat to peer

...
```

### Programmatic Updates

```rust
use fold_node::logging::LoggingSystem;

// Update feature log level
LoggingSystem::update_feature_level("transform", "TRACE").await?;

// Reload configuration from file
LoggingSystem::reload_config_from_file("config/logging.toml").await?;

// Get current configuration
let config = LoggingSystem::get_config().await;

// Get available features
let features = LoggingSystem::get_features().await;
```

---

## Performance Monitoring

### Performance Timer

The logging system includes built-in performance monitoring utilities:

```rust
use fold_node::logging::features::{PerformanceTimer, LogFeature};

// Start timing an operation
let timer = PerformanceTimer::new(
    LogFeature::Transform, 
    "user_score_calculation".to_string()
);

// Perform the operation
let result = perform_transform(data).await?;

// Log completion time
timer.finish(); // Logs: "Operation 'user_score_calculation' completed in 45ms"
```

### Custom Performance Metrics

```rust
use std::time::Instant;
use fold_node::log_transform_info;

let start = Instant::now();

// Perform operation
let result = expensive_operation().await?;

let duration = start.elapsed();
log_transform_info!(
    "Operation completed successfully",
    operation = "expensive_operation",
    duration_ms = duration.as_millis(),
    records_processed = result.len()
);
```

### Correlation IDs

Enable correlation IDs to track related operations across components:

```rust
use fold_node::logging::LoggingSystem;

// Correlation IDs are automatically generated when enabled in config
log_transform_info!("Starting request processing", request_id = "req_123");
log_network_debug!("Fetching data from peer", request_id = "req_123");
log_schema_info!("Validation complete", request_id = "req_123");
```

---

## Migration Guide

### From Basic Logging

**Before (using standard `log` crate):**
```rust
use log::{info, debug, warn, error};

debug!("Processing transform: {}", name);
info!("Transform completed in {}ms", duration);
warn!("Transform used deprecated function");
error!("Transform failed: {}", error);
```

**After (using feature-specific logging):**
```rust
use fold_node::{log_transform_debug, log_transform_info, log_transform_warn, log_transform_error};

log_transform_debug!("Processing transform: {}", name);
log_transform_info!("Transform completed in {}ms", duration);
log_transform_warn!("Transform used deprecated function");
log_transform_error!("Transform failed: {}", error);
```

### From web_logger

**Before:**
```rust
use crate::web_logger;

web_logger::init()?;
log::info!("Application started");
```

**After:**
```rust
use fold_node::logging::LoggingSystem;

LoggingSystem::init_default().await?;
log::info!("Application started"); // Still works!
```

### Migration Script

Use the provided migration script to help convert existing code:

```bash
python scripts/migrate_logging.py /path/to/your/code
```

The script will:
1. Identify log statements that could use feature-specific macros
2. Suggest replacements based on file/module names
3. Generate a migration report

---

## Best Practices

### Development Environment

```toml
[general]
default_level = "DEBUG"
enable_colors = true

[outputs.console]
enabled = true
level = "DEBUG"
colors = true

[outputs.file]
enabled = false

[features]
transform = "TRACE"    # High detail for active development
network = "DEBUG"      # Network debugging
schema = "INFO"        # Moderate detail
database = "WARN"      # Reduce noise
```

### Staging Environment

```toml
[general]
default_level = "INFO"

[outputs.console]
enabled = true
level = "INFO"

[outputs.file]
enabled = true
level = "DEBUG"
path = "/var/log/datafold/staging.log"

[outputs.structured]
enabled = true
level = "INFO"
path = "/var/log/datafold/staging-structured.json"

[features]
transform = "INFO"
network = "INFO"
http_server = "DEBUG"  # API monitoring
```

### Production Environment

```toml
[general]
default_level = "WARN"

[outputs.console]
enabled = false

[outputs.file]
enabled = true
level = "INFO"
path = "/var/log/datafold/production.log"
max_size = "100MB"
max_files = 10

[outputs.structured]
enabled = true
level = "INFO"
path = "/var/log/datafold/production-structured.json"

[features]
transform = "WARN"
network = "WARN"
database = "ERROR"     # Only critical issues
permissions = "WARN"   # Security events
```

### Structured Logging

Use structured fields for better searchability:

```rust
// Good: Structured data
log_transform_info!(
    "Transform execution completed",
    transform_id = %transform.id,
    duration_ms = duration.as_millis(),
    records_in = input_count,
    records_out = output_count,
    status = "success"
);

// Avoid: Unstructured strings
log_transform_info!(
    "Transform {} completed in {}ms processing {} -> {} records", 
    transform.id, 
    duration.as_millis(), 
    input_count, 
    output_count
);
```

### Error Context

Provide rich context for errors:

```rust
use anyhow::Context;

match perform_transform(data).await {
    Ok(result) => {
        log_transform_info!(
            "Transform successful",
            transform_id = %transform.id,
            output_records = result.len()
        );
    }
    Err(e) => {
        log_transform_error!(
            "Transform failed",
            transform_id = %transform.id,
            error = %e,
            input_records = data.len(),
            error_context = ?e.source()
        );
    }
}
```

### Performance Logging

Log performance metrics consistently:

```rust
let timer = PerformanceTimer::new(LogFeature::Database, "bulk_insert".to_string());

let result = database.bulk_insert(records).await?;

timer.finish();

// Additional metrics
log::info!(
    target: "datafold_node::database",
    "Bulk insert metrics",
    records_inserted = result.inserted_count,
    batch_size = records.len(),
    throughput_per_sec = (result.inserted_count as f64 / timer.duration().as_secs_f64()) as u64
);
```

---

## Troubleshooting

### Common Issues

#### 1. Logs Not Appearing

**Problem**: No logs are showing up in console or files.

**Solutions:**
- Check if logging is initialized: `LoggingSystem::init_default().await?`
- Verify log levels: feature level must be >= output level
- Check output configuration: ensure outputs are enabled
- Verify file permissions for file output

```bash
# Debug configuration
curl http://localhost:9001/api/logs/config
```

#### 2. Too Many/Few Logs

**Problem**: Log volume is too high or too low.

**Solutions:**
- Adjust feature-specific levels:
  ```bash
  curl -X POST http://localhost:9001/api/logs/features \
    -H "Content-Type: application/json" \
    -d '{"feature": "database", "level": "ERROR"}'
  ```
- Update output levels in configuration
- Use environment variables for quick testing:
  ```bash
  export DATAFOLD_LOG_FEATURE_NETWORK=ERROR
  ```

#### 3. File Rotation Not Working

**Problem**: Log files are not rotating as expected.

**Solutions:**
- Check disk space and file permissions
- Verify `max_size` and `max_files` configuration
- Monitor log rotation logs in structured output
- Ensure log directory exists and is writable

#### 4. Web Streaming Issues

**Problem**: Web log streaming is not working.

**Solutions:**
- Verify web output is enabled
- Check browser network tools for connection issues
- Ensure firewall allows connections to HTTP server
- Check for authentication requirements

#### 5. Performance Impact

**Problem**: Logging is impacting application performance.

**Solutions:**
- Reduce log levels in production
- Disable unnecessary outputs
- Use structured logging efficiently
- Consider async logging for high-throughput scenarios

### Debug Mode

Enable comprehensive debugging:

```bash
# Set environment variables
export DATAFOLD_LOG_LEVEL=TRACE
export DATAFOLD_LOG_FEATURE_TRANSFORM=TRACE
export DATAFOLD_LOG_FEATURE_NETWORK=TRACE
export DATAFOLD_LOG_FEATURE_SCHEMA=TRACE

# Or via configuration
curl -X POST http://localhost:9001/api/logs/features \
  -H "Content-Type: application/json" \
  -d '{"feature": "transform", "level": "TRACE"}'
```

### Log Analysis

**Find specific operations:**
```bash
# Search for transform operations
grep "transform" logs/datafold.log | grep "ERROR"

# Search for specific request ID
grep "req_123" logs/datafold.log

# Parse structured logs
jq '.level == "ERROR"' logs/datafold-structured.json
```

**Monitor real-time:**
```bash
# Follow logs
tail -f logs/datafold.log

# Filter by feature
curl -s http://localhost:9001/api/logs/stream | grep "transform"
```

---

## API Reference

### HTTP Endpoints

| Method | Endpoint | Description | Request Body | Response |
|--------|----------|-------------|--------------|----------|
| GET | `/api/logs` | Get current logs | None | `String[]` |
| GET | `/api/logs/stream` | Stream logs via SSE | None | `text/event-stream` |
| GET | `/api/logs/config` | Get configuration | None | `LogConfig` |
| POST | `/api/logs/features` | Update feature level | `LogLevelUpdate` | `LogConfigResponse` |
| POST | `/api/logs/reload` | Reload configuration | None | `LogConfigResponse` |
| GET | `/api/logs/features` | Get available features | None | `FeatureResponse` |

### Data Types

#### LogLevelUpdate
```json
{
  "feature": "string",
  "level": "TRACE|DEBUG|INFO|WARN|ERROR"
}
```

#### LogConfigResponse
```json
{
  "success": true,
  "message": "string"
}
```

#### FeatureResponse
```json
{
  "features": {
    "feature_name": "log_level"
  },
  "available_levels": ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]
}
```

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `DATAFOLD_LOG_LEVEL` | Default log level | `INFO` | `DEBUG` |
| `DATAFOLD_LOG_COLORS` | Enable colored output | `true` | `false` |
| `DATAFOLD_LOG_CONSOLE_ENABLED` | Enable console output | `true` | `false` |
| `DATAFOLD_LOG_CONSOLE_LEVEL` | Console log level | `INFO` | `DEBUG` |
| `DATAFOLD_LOG_FILE_ENABLED` | Enable file output | `false` | `true` |
| `DATAFOLD_LOG_FILE_PATH` | Log file path | `logs/datafold.log` | `/var/log/app.log` |
| `DATAFOLD_LOG_FILE_LEVEL` | File log level | `DEBUG` | `INFO` |
| `DATAFOLD_LOG_WEB_ENABLED` | Enable web output | `true` | `false` |
| `DATAFOLD_LOG_WEB_LEVEL` | Web log level | `INFO` | `DEBUG` |
| `DATAFOLD_LOG_FEATURE_*` | Feature-specific level | `INFO` | `TRACE` |

### Configuration Schema

See [`config/logging.toml`](../config/logging.toml) for complete configuration schema and examples.

---

## Integration Examples

### With Monitoring Tools

#### ELK Stack Integration
```yaml
# logstash.conf
input {
  file {
    path => "/var/log/datafold/production-structured.json"
    codec => json
  }
}

filter {
  if [target] == "datafold_node::transform" {
    mutate {
      add_tag => ["transform"]
    }
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "datafold-logs-%{+YYYY.MM.dd}"
  }
}
```

#### Prometheus Metrics
```rust
// Custom metrics based on logs
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref TRANSFORM_COUNTER: Counter = Counter::new(
        "datafold_transforms_total", 
        "Total number of transforms executed"
    ).unwrap();
    
    static ref TRANSFORM_DURATION: Histogram = Histogram::new(
        "datafold_transform_duration_seconds",
        "Transform execution time in seconds"
    ).unwrap();
}

// In your transform code
let timer = PerformanceTimer::new(LogFeature::Transform, "user_calc".to_string());
let result = execute_transform(data).await?;
timer.finish();

TRANSFORM_COUNTER.inc();
TRANSFORM_DURATION.observe(timer.duration().as_secs_f64());
```

#### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "DataFold Logging Dashboard",
    "panels": [
      {
        "title": "Log Levels by Feature",
        "type": "stat",
        "targets": [
          {
            "expr": "sum by (level, target) (rate(datafold_logs_total[5m]))"
          }
        ]
      },
      {
        "title": "Transform Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(datafold_transform_duration_seconds_sum[5m]) / rate(datafold_transform_duration_seconds_count[5m])"
          }
        ]
      }
    ]
  }
}
```

---

For more examples and advanced configuration, see:
- [`examples/logging_examples.rs`](../examples/logging_examples.rs)
- [`examples/logging_performance.rs`](../examples/logging_performance.rs)
- [`examples/logging_config_examples.toml`](../examples/logging_config_examples.toml)
- [Migration Script](../scripts/migrate_logging.py)