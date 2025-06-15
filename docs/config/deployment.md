# Configuration Deployment Guide

**PBI 27: Cross-Platform Configuration Management System**

This document provides comprehensive deployment guides for the cross-platform configuration management system, including platform-specific instructions, migration procedures, troubleshooting, and performance tuning.

## Table of Contents

1. [Deployment Overview](#deployment-overview)
2. [Platform-Specific Deployment](#platform-specific-deployment)
3. [Migration Procedures](#migration-procedures)
4. [Configuration Management](#configuration-management)
5. [Troubleshooting Guide](#troubleshooting-guide)
6. [Performance Tuning](#performance-tuning)
7. [Monitoring and Maintenance](#monitoring-and-maintenance)

## Deployment Overview

The cross-platform configuration system is designed for seamless deployment across Linux, macOS, and Windows environments with minimal platform-specific customization required.

### System Requirements

**Minimum Requirements:**
- Rust 1.70 or later
- 512 MB RAM
- 100 MB disk space
- Network connectivity for keystore services (optional)

**Recommended Requirements:**
- Rust 1.75 or later
- 2 GB RAM
- 500 MB disk space
- SSD storage for optimal performance

### Pre-Deployment Checklist

- [ ] Verify platform compatibility
- [ ] Check Rust version compatibility
- [ ] Ensure sufficient disk space
- [ ] Verify network access for keystore services
- [ ] Backup existing configuration files
- [ ] Test deployment in staging environment

## Platform-Specific Deployment

### Linux Deployment

**Supported Distributions:**
- Ubuntu 20.04 LTS or later
- CentOS 8 or later
- Fedora 35 or later
- Debian 11 or later
- Arch Linux (current)

#### Installation Steps

```bash
# 1. Clone the repository
git clone https://github.com/company/datafold.git
cd datafold

# 2. Build the project
cargo build --release --workspace

# 3. Install system-wide (optional)
sudo cp target/release/datafold_cli /usr/local/bin/
sudo cp target/release/datafold_node /usr/local/bin/

# 4. Create configuration directories
mkdir -p ~/.config/datafold
mkdir -p ~/.local/share/datafold
mkdir -p ~/.cache/datafold
mkdir -p ~/.local/state/datafold/logs

# 5. Initialize configuration
datafold_cli config init
```

#### Linux-Specific Configuration

```toml
# ~/.config/datafold/config.toml
[platform]
name = "linux"
use_xdg_directories = true
enable_keyring = true
keyring_service = "gnome-keyring"  # or "kwallet", "secret-service"

[platform.paths]
config_dir = "${XDG_CONFIG_HOME:-$HOME/.config}/datafold"
data_dir = "${XDG_DATA_HOME:-$HOME/.local/share}/datafold"
cache_dir = "${XDG_CACHE_HOME:-$HOME/.cache}/datafold"
logs_dir = "${XDG_STATE_HOME:-$HOME/.local/state}/datafold/logs"
runtime_dir = "${XDG_RUNTIME_DIR:-/tmp}/datafold-${USER}"

[platform.security]
use_file_permissions = true
config_file_mode = "0600"
data_dir_mode = "0700"
enable_selinux_support = true
```

#### Systemd Service Configuration

```ini
# /etc/systemd/system/datafold-node.service
[Unit]
Description=DataFold Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=datafold
Group=datafold
WorkingDirectory=/opt/datafold
ExecStart=/usr/local/bin/datafold_node --config /etc/datafold/node_config.toml
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/datafold /var/log/datafold

[Install]
WantedBy=multi-user.target
```

### macOS Deployment

**Supported Versions:**
- macOS 11 (Big Sur) or later
- Both Intel and Apple Silicon architectures

#### Installation Steps

```bash
# 1. Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Clone and build
git clone https://github.com/company/datafold.git
cd datafold
cargo build --release --workspace

# 4. Install to Applications (GUI applications)
sudo cp target/release/datafold_cli /usr/local/bin/
sudo cp target/release/datafold_node /usr/local/bin/

# 5. Create configuration directories
mkdir -p "$HOME/Library/Application Support/DataFold"
mkdir -p "$HOME/Library/Caches/DataFold"
mkdir -p "$HOME/Library/Logs/DataFold"

# 6. Initialize configuration
datafold_cli config init
```

#### macOS-Specific Configuration

```toml
# ~/Library/Application Support/DataFold/config.toml
[platform]
name = "macos"
use_apple_guidelines = true
enable_keychain = true
enable_fsevents = true

[platform.paths]
config_dir = "~/Library/Application Support/DataFold"
data_dir = "~/Library/Application Support/DataFold"
cache_dir = "~/Library/Caches/DataFold"
logs_dir = "~/Library/Logs/DataFold"
runtime_dir = "~/Library/Caches/DataFold/tmp"

[platform.security]
use_keychain_services = true
keychain_service_name = "DataFold"
enable_app_sandbox = false
code_signing_required = false

[platform.performance]
use_fsevents = true
enable_memory_pressure_handling = true
respect_thermal_state = true
```

#### LaunchDaemon Configuration

```xml
<!-- /Library/LaunchDaemons/com.company.datafold.node.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.company.datafold.node</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/datafold_node</string>
        <string>--config</string>
        <string>/etc/datafold/node_config.toml</string>
    </array>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>/var/log/datafold/stdout.log</string>
    
    <key>StandardErrorPath</key>
    <string>/var/log/datafold/stderr.log</string>
    
    <key>WorkingDirectory</key>
    <string>/opt/datafold</string>
    
    <key>UserName</key>
    <string>_datafold</string>
</dict>
</plist>
```

### Windows Deployment

**Supported Versions:**
- Windows 10 version 1903 or later
- Windows 11
- Windows Server 2019 or later

#### Installation Steps

```powershell
# 1. Install Rust (run in PowerShell as Administrator)
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe -y
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# 2. Clone and build
git clone https://github.com/company/datafold.git
cd datafold
cargo build --release --workspace

# 3. Create installation directory
New-Item -ItemType Directory -Force -Path "C:\Program Files\DataFold"
Copy-Item "target\release\datafold_cli.exe" "C:\Program Files\DataFold\"
Copy-Item "target\release\datafold_node.exe" "C:\Program Files\DataFold\"

# 4. Add to PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
[Environment]::SetEnvironmentVariable("PATH", $currentPath + ";C:\Program Files\DataFold", "Machine")

# 5. Create configuration directories
New-Item -ItemType Directory -Force -Path "$env:APPDATA\DataFold"
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\DataFold\Cache"
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\DataFold\Logs"

# 6. Initialize configuration
datafold_cli config init
```

#### Windows-Specific Configuration

```toml
# %APPDATA%\DataFold\config.toml
[platform]
name = "windows"
use_windows_conventions = true
enable_credential_manager = true
enable_wmi = true

[platform.paths]
config_dir = "%APPDATA%\\DataFold"
data_dir = "%APPDATA%\\DataFold"
cache_dir = "%LOCALAPPDATA%\\DataFold\\Cache"
logs_dir = "%LOCALAPPDATA%\\DataFold\\Logs"
runtime_dir = "%TEMP%\\DataFold"

[platform.security]
use_credential_manager = true
enable_uac_elevation = false
require_admin_privileges = false

[platform.performance]
use_overlapped_io = true
enable_memory_mapped_files = true
respect_power_settings = true
```

#### Windows Service Configuration

```powershell
# Install as Windows Service
New-Service -Name "DataFoldNode" -BinaryPathName "C:\Program Files\DataFold\datafold_node.exe --config C:\ProgramData\DataFold\node_config.toml" -DisplayName "DataFold Node Service" -Description "DataFold distributed database node" -StartupType Automatic

# Configure service recovery
sc.exe failure DataFoldNode reset= 0 actions= restart/5000/restart/10000/restart/30000

# Start the service
Start-Service -Name "DataFoldNode"
```

## Migration Procedures

### Automated Migration

The configuration system includes comprehensive migration utilities for transitioning from legacy systems.

#### Complete System Migration

```bash
# Run complete migration
datafold_cli migrate --all --backup

# Migration options:
# --all: Migrate all detected configuration systems
# --backup: Create backups before migration
# --dry-run: Preview changes without applying
# --force: Overwrite existing configurations
# --strategy preserve|replace|backup: Migration strategy
```

#### Step-by-Step Migration Process

```rust
// Programmatic migration example
use datafold::config::migration::{ConfigMigrationManager, MigrationStrategy};

async fn perform_migration() -> Result<(), Box<dyn std::error::Error>> {
    let migration_manager = ConfigMigrationManager::new();
    
    // 1. CLI configuration migration
    println!("Step 1: Migrating CLI configuration...");
    let cli_result = migration_manager.migrate_cli_config().await?;
    
    if cli_result.success {
        println!("✓ CLI migration successful ({} items)", cli_result.items_migrated);
    } else {
        eprintln!("✗ CLI migration failed");
        for error in &cli_result.errors {
            eprintln!("  {}", error);
        }
    }
    
    // 2. Logging configuration migration
    println!("Step 2: Migrating logging configuration...");
    let logging_result = migration_manager.migrate_logging_config().await?;
    
    if logging_result.success {
        println!("✓ Logging migration successful ({} items)", logging_result.items_migrated);
    } else {
        eprintln!("✗ Logging migration failed");
        for error in &logging_result.errors {
            eprintln!("  {}", error);
        }
    }
    
    // 3. Unified configuration migration
    println!("Step 3: Migrating unified configuration...");
    let unified_result = migration_manager.migrate_unified_config().await?;
    
    if unified_result.success {
        println!("✓ Unified migration successful ({} items)", unified_result.items_migrated);
    } else {
        eprintln!("✗ Unified migration failed");
        for error in &unified_result.errors {
            eprintln!("  {}", error);
        }
    }
    
    Ok(())
}
```

### Manual Migration Procedures

#### Legacy CLI Configuration

**Before** (`~/.config/datafold/cli_config.json`):
```json
{
  "default_profile": "production",
  "profiles": {
    "production": {
      "name": "production",
      "api_endpoint": "https://api.datafold.com",
      "auth_token": "abc123"
    }
  },
  "settings": {
    "output_format": "table",
    "verbosity": 1
  }
}
```

**After** (`~/.config/datafold/config.toml`):
```toml
[cli]
default_profile = "production"

[cli.profiles.production]
name = "production"
api_endpoint = "https://api.datafold.com"
# auth_token moved to keystore for security

[cli.settings]
output_format = "table"
verbosity = 1

[platform.keystore]
store_credentials = true
service_name = "DataFold CLI"
```

#### Migration Validation

```bash
# Validate migrated configuration
datafold_cli config validate

# Test configuration loading
datafold_cli config show

# Verify keystore integration
datafold_cli auth status
```

## Configuration Management

### Environment-Specific Configurations

#### Development Environment

```toml
# config.toml
[environment]
name = "development"
debug_mode = true

[logging]
level = "debug"
enable_console = true
enable_file = true

[performance]
cache_size_mb = 64
enable_file_watching = true
```

#### Staging Environment

```toml
# config.toml
[environment]
name = "staging"
debug_mode = false

[logging]
level = "info"
enable_console = true
enable_file = true
enable_structured = true

[performance]
cache_size_mb = 128
enable_file_watching = true

[security]
require_encryption = true
audit_changes = true
```

#### Production Environment

```toml
# config.toml
[environment]
name = "production"
debug_mode = false

[logging]
level = "warn"
enable_console = false
enable_file = true
enable_structured = true
enable_remote = true

[performance]
cache_size_mb = 256
enable_file_watching = false
use_memory_mapping = true

[security]
require_encryption = true
require_signatures = true
audit_changes = true
rotate_keys = true

[monitoring]
enable_metrics = true
enable_health_checks = true
alert_on_errors = true
```

### Configuration Templating

```bash
# Generate configuration from template
datafold_cli config generate --template production --output /etc/datafold/config.toml

# Template with variable substitution
datafold_cli config generate --template production --vars environment.env --output config.toml
```

**Template Example** (`templates/production.toml`):
```toml
[environment]
name = "{{ENVIRONMENT_NAME}}"

[database]
host = "{{DB_HOST}}"
port = {{DB_PORT}}
name = "{{DB_NAME}}"

[api]
endpoint = "{{API_ENDPOINT}}"
timeout_seconds = {{API_TIMEOUT}}
```

**Variables File** (`environment.env`):
```env
ENVIRONMENT_NAME=production
DB_HOST=db.company.com
DB_PORT=5432
DB_NAME=datafold_prod
API_ENDPOINT=https://api.company.com
API_TIMEOUT=30
```

## Troubleshooting Guide

### Common Issues and Solutions

#### Issue: Configuration File Not Found

**Symptoms:**
```
Error: Configuration not found at expected location
```

**Diagnosis:**
```bash
# Check expected configuration path
datafold_cli config path

# List configuration directories
datafold_cli config dirs

# Check file permissions
ls -la ~/.config/datafold/
```

**Solutions:**
```bash
# Initialize default configuration
datafold_cli config init

# Or create from template
datafold_cli config generate --template default
```

#### Issue: Permission Denied

**Symptoms:**
```
Error: Access denied writing to configuration directory
```

**Diagnosis:**
```bash
# Check directory permissions
ls -ld ~/.config/datafold/

# Check parent directory permissions
ls -ld ~/.config/

# Check disk space
df -h ~/.config/
```

**Solutions:**
```bash
# Fix directory permissions
chmod 700 ~/.config/datafold/

# Fix file permissions
chmod 600 ~/.config/datafold/config.toml

# Create directories if missing
mkdir -p ~/.config/datafold
```

#### Issue: Keystore Access Failed

**Symptoms:**
```
Error: Failed to access platform keystore
```

**Platform-Specific Solutions:**

**Linux:**
```bash
# Install required keyring
sudo apt install gnome-keyring  # Ubuntu/Debian
sudo dnf install gnome-keyring  # Fedora

# Check keyring service
systemctl --user status gnome-keyring
```

**macOS:**
```bash
# Check Keychain Access
security list-keychains

# Reset keychain if corrupted
security delete-keychain login.keychain
security create-keychain login.keychain
```

**Windows:**
```powershell
# Check Credential Manager service
Get-Service -Name "VaultSvc"

# Restart if needed
Restart-Service -Name "VaultSvc"
```

#### Issue: Configuration Validation Failed

**Symptoms:**
```
Error: Configuration validation failed: Invalid value for field 'port'
```

**Diagnosis:**
```bash
# Validate configuration
datafold_cli config validate --verbose

# Show configuration schema
datafold_cli config schema
```

**Solutions:**
1. Fix validation errors in configuration file
2. Use configuration generator with valid templates
3. Reset to default configuration if corrupted

#### Issue: Migration Failed

**Symptoms:**
```
Error: Migration failed: Source configuration format not supported
```

**Diagnosis:**
```bash
# Check source configuration format
file ~/.config/datafold/old_config.*

# List migration capabilities
datafold_cli migrate --list-formats
```

**Solutions:**
```bash
# Try manual conversion
datafold_cli migrate --source-format json --target-format toml --input old_config.json --output config.toml

# Use backup if available
datafold_cli config restore --backup /path/to/backup.toml
```

### Performance Issues

#### Issue: Slow Configuration Loading

**Diagnosis:**
```bash
# Check configuration load time
time datafold_cli config show > /dev/null

# Check file system performance
time ls -la ~/.config/datafold/

# Check configuration file size
du -h ~/.config/datafold/config.toml
```

**Solutions:**
```bash
# Enable configuration caching
datafold_cli config set performance.enable_caching true

# Reduce configuration size
datafold_cli config optimize

# Use SSD storage for configuration directory
```

#### Issue: High Memory Usage

**Diagnosis:**
```bash
# Check memory usage
ps aux | grep datafold

# Monitor configuration manager memory
datafold_cli config metrics --memory
```

**Solutions:**
```toml
# Reduce cache size in config.toml
[performance]
cache_size_mb = 32  # Reduce from default 64MB
enable_memory_mapping = true
use_compressed_cache = true
```

### Network Issues

#### Issue: Keystore Service Unavailable

**Symptoms:**
```
Error: Failed to connect to keystore service
```

**Solutions:**
```bash
# Disable keystore temporarily
datafold_cli config set security.use_keystore false

# Or configure fallback storage
datafold_cli config set security.fallback_to_file true
```

## Performance Tuning

### Memory Optimization

```toml
# config.toml - Memory-optimized settings
[performance]
# Reduce memory footprint
cache_size_mb = 32
use_memory_mapping = true
enable_compression = true
lazy_loading = true

# Garbage collection tuning
gc_threshold_mb = 64
gc_frequency_seconds = 300

[platform.memory]
# Platform-specific memory settings
use_large_pages = false  # Disable for low-memory systems
memory_pool_size_mb = 16
```

### I/O Optimization

```toml
# config.toml - I/O optimized settings
[performance.io]
# File system optimization
use_direct_io = true
buffer_size_kb = 64
sync_frequency_seconds = 30

# Platform-specific I/O
[platform.io]
use_overlapped_io = true  # Windows
use_sendfile = true       # Linux
use_fsevents = true       # macOS
```

### Network Optimization

```toml
# config.toml - Network optimized settings
[performance.network]
# Connection pooling
max_connections = 10
connection_timeout_seconds = 30
keep_alive_seconds = 300

# Compression
enable_compression = true
compression_level = 6

# Caching
enable_dns_cache = true
dns_cache_ttl_seconds = 3600
```

### Platform-Specific Optimizations

#### Linux Optimizations

```toml
[platform.linux]
# Use epoll for file watching
use_epoll = true

# Memory management
use_transparent_hugepages = true
disable_swap = false

# I/O scheduling
io_scheduler = "deadline"  # or "cfq", "noop"
```

#### macOS Optimizations

```toml
[platform.macos]
# Use Grand Central Dispatch
use_gcd = true

# Memory pressure handling
respect_memory_pressure = true

# Thermal management
respect_thermal_state = true

# FSEvents optimization
fsevent_latency_seconds = 0.1
```

#### Windows Optimizations

```toml
[platform.windows]
# I/O completion ports
use_iocp = true

# Memory management
use_large_page_minimum = false

# Power management
respect_power_scheme = true

# Registry caching
enable_registry_cache = true
```

## Monitoring and Maintenance

### Health Checks

```bash
# System health check
datafold_cli health check

# Configuration-specific health
datafold_cli config health

# Platform capabilities check
datafold_cli platform check
```

### Metrics Collection

```bash
# Configuration metrics
datafold_cli config metrics

# Performance metrics
datafold_cli config metrics --performance

# Platform metrics
datafold_cli platform metrics
```

### Maintenance Tasks

#### Daily Maintenance

```bash
#!/bin/bash
# daily_maintenance.sh

# Check configuration health
datafold_cli config health || echo "Configuration health check failed"

# Validate configuration
datafold_cli config validate || echo "Configuration validation failed"

# Clean cache if needed
CACHE_SIZE=$(du -sm ~/.cache/datafold | cut -f1)
if [ $CACHE_SIZE -gt 100 ]; then
    echo "Cleaning cache (${CACHE_SIZE}MB)"
    datafold_cli config clean-cache
fi

# Rotate logs if needed
LOG_SIZE=$(du -sm ~/.local/state/datafold/logs | cut -f1)
if [ $LOG_SIZE -gt 500 ]; then
    echo "Rotating logs (${LOG_SIZE}MB)"
    datafold_cli logs rotate
fi
```

#### Weekly Maintenance

```bash
#!/bin/bash
# weekly_maintenance.sh

# Backup configuration
BACKUP_DATE=$(date +%Y%m%d)
datafold_cli config backup --output "config_backup_${BACKUP_DATE}.toml"

# Update configuration if needed
datafold_cli config update --check-only

# Performance analysis
datafold_cli config metrics --performance --report weekly_performance_${BACKUP_DATE}.json

# Security audit
datafold_cli config audit --output security_audit_${BACKUP_DATE}.json
```

#### Monthly Maintenance

```bash
#!/bin/bash
# monthly_maintenance.sh

# Clean old backups (keep last 6 months)
find ~/.config/datafold/backups -name "*.toml" -mtime +180 -delete

# Optimize configuration
datafold_cli config optimize

# Update platform capabilities
datafold_cli platform update

# Generate maintenance report
datafold_cli config report --type maintenance --output monthly_report_$(date +%Y%m).json
```

### Monitoring Integration

#### Prometheus Metrics

```toml
# config.toml
[monitoring.prometheus]
enabled = true
port = 9090
metrics_endpoint = "/metrics"

[monitoring.metrics]
collect_config_metrics = true
collect_platform_metrics = true
collect_performance_metrics = true
```

#### Grafana Dashboard

Key metrics to monitor:
- Configuration load time
- Cache hit ratio
- Memory usage
- File system I/O
- Error rates
- Platform capabilities status

#### Alerting Rules

```yaml
# alerts.yml
groups:
  - name: datafold_config
    rules:
      - alert: ConfigurationLoadSlow
        expr: config_load_duration_seconds > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Configuration loading is slow"
          
      - alert: ConfigurationError
        expr: config_errors_total > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Configuration errors detected"
          
      - alert: KeystoreUnavailable
        expr: keystore_available == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Platform keystore unavailable"
```

## Related Documentation

- [Architecture](architecture.md) - System architecture and design
- [API Reference](api.md) - Complete API documentation
- [Integration Guide](integration.md) - Integration patterns and examples
- [Security Guide](security.md) - Security features and best practices