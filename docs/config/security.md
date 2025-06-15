# Configuration Security Guide

**PBI 27: Cross-Platform Configuration Management System**

This document provides comprehensive security guidance for the cross-platform configuration management system, including security features, best practices, threat mitigation, and compliance considerations.

## Table of Contents

1. [Security Overview](#security-overview)
2. [Keystore Integration](#keystore-integration)
3. [Encrypted Configuration](#encrypted-configuration)
4. [Cryptographic Features](#cryptographic-features)
5. [Access Control](#access-control)
6. [Security Best Practices](#security-best-practices)
7. [Audit and Compliance](#audit-and-compliance)
8. [Threat Model](#threat-model)
9. [Security Monitoring](#security-monitoring)

## Security Overview

The cross-platform configuration system implements defense-in-depth security with multiple layers of protection:

- **Platform-native keystore integration** for secure credential storage
- **Encrypted configuration sections** for sensitive data
- **Granular access controls** with file system permissions
- **Cryptographic protection** with industry-standard algorithms
- **Audit logging** for security event tracking
- **Secure defaults** for all configuration options

### Security Architecture

```mermaid
flowchart TD
    A[Application] --> B[Configuration Manager]
    B --> C[Access Control Layer]
    C --> D[Encryption Layer]
    D --> E[Storage Layer]
    
    E --> F[File System]
    E --> G[Platform Keystore]
    
    F --> F1[Encrypted Files]
    F --> F2[Protected Directories]
    
    G --> G1[Linux: Secret Service]
    G --> G2[macOS: Keychain]
    G --> G3[Windows: Credential Manager]
    
    H[Audit System] --> C
    H --> D
    H --> E
```

### Security Principles

1. **Principle of Least Privilege**: Minimal required permissions for all operations
2. **Defense in Depth**: Multiple security layers with independent failure modes
3. **Secure by Default**: Safe configuration options enabled by default
4. **Data Minimization**: Only store necessary sensitive information
5. **Transparency**: Clear audit trails for all security-relevant operations

## Keystore Integration

The system provides unified access to platform-specific secure storage mechanisms.

### Platform Keystore Support

#### Linux: Secret Service API

**Implementation**: [`LinuxKeystore`](../../src/config/platform/linux.rs)

**Features:**
- GNOME Keyring integration
- KWallet support
- Secret Service API compliance
- D-Bus communication security

**Configuration:**
```toml
[security.keystore.linux]
service = "secret-service"  # or "gnome-keyring", "kwallet"
application_name = "DataFold"
keyring_name = "default"

# Security settings
encrypt_in_memory = true
lock_on_idle = true
idle_timeout_minutes = 30
```

**Security Features:**
- Encrypted storage with user session key
- Authentication required for access
- Process isolation
- Automatic lock on user session end

#### macOS: Keychain Services

**Implementation**: [`MacOSKeystore`](../../src/config/platform/macos.rs)

**Features:**
- Native Keychain Services integration
- Touch ID/Face ID authentication support
- Secure Enclave hardware protection
- App-specific keychain access

**Configuration:**
```toml
[security.keystore.macos]
keychain = "login"  # or "system", "dynamic"
service_name = "DataFold"
access_group = "com.company.datafold"

# Security settings
require_authentication = true
allow_touch_id = true
allow_face_id = true
secure_enclave_required = false
```

**Security Features:**
- Hardware-backed encryption (Secure Enclave)
- Biometric authentication
- App sandbox isolation
- System-level access controls

#### Windows: Credential Manager

**Implementation**: [`WindowsKeystore`](../../src/config/platform/windows.rs)

**Features:**
- Windows Credential Manager integration
- DPAPI (Data Protection API) encryption
- User session isolation
- Enterprise credential roaming

**Configuration:**
```toml
[security.keystore.windows]
credential_type = "generic"  # or "domain", "certificate"
target_name = "DataFold"
persist = "local_machine"  # or "session", "enterprise"

# Security settings
require_user_interaction = false
encrypt_with_user_key = true
allow_credential_roaming = false
```

**Security Features:**
- DPAPI encryption with user key
- Active Directory integration
- Group Policy enforcement
- Hardware security module support

### Keystore API Usage

```rust
use datafold::config::platform::keystore::{create_platform_keystore, PlatformKeystore};

async fn secure_credential_management() -> Result<(), Box<dyn std::error::Error>> {
    let keystore = create_platform_keystore().await?;
    
    // Store sensitive configuration
    keystore.store_secret("api_key", b"sensitive_api_key").await?;
    keystore.store_secret("db_password", b"database_password").await?;
    
    // Retrieve when needed
    if let Some(api_key) = keystore.get_secret("api_key").await? {
        let api_key_str = String::from_utf8(api_key)?;
        // Use API key securely...
    }
    
    // Clean up sensitive data
    keystore.delete_secret("api_key").await?;
    
    Ok(())
}
```

### Keystore Security Best Practices

1. **Rotate credentials regularly**
2. **Use application-specific service names**
3. **Implement proper error handling for keystore unavailability**
4. **Clean up unused credentials**
5. **Monitor keystore access patterns**

## Encrypted Configuration

The system supports encrypted configuration sections for storing sensitive data in configuration files.

### Encrypted Section Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSection {
    pub encrypted_data: Vec<u8>,
    pub metadata: EncryptionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    pub algorithm: String,
    pub key_derivation: String,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### Configuration Example

```toml
# config.toml - Non-sensitive configuration
[database]
host = "localhost"
port = 5432
ssl_enabled = true

[api]
endpoint = "https://api.company.com"
timeout_seconds = 30

# Encrypted sections stored separately
[encrypted_sections]
database_credentials = "encrypted_section_id_1"
api_credentials = "encrypted_section_id_2"
```

**Encrypted Section Storage** (`.datafold/encrypted/section_id_1.enc`):
```json
{
  "encrypted_data": "base64_encoded_encrypted_data",
  "metadata": {
    "algorithm": "ChaCha20Poly1305",
    "key_derivation": "Argon2id",
    "salt": "base64_encoded_salt",
    "nonce": "base64_encoded_nonce",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

### Encryption Implementation

```rust
use datafold::config::{EnhancedConfigurationManager, EncryptedSection};

async fn manage_encrypted_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let manager = EnhancedConfigurationManager::new().await?;
    
    // Store sensitive configuration in encrypted section
    let sensitive_config = serde_json::json!({
        "username": "admin",
        "password": "super_secret_password",
        "private_key": "-----BEGIN PRIVATE KEY-----..."
    });
    
    // Encrypt and store
    manager.store_encrypted_section("database_credentials", &sensitive_config).await?;
    
    // Retrieve and decrypt
    let decrypted_config = manager.load_encrypted_section("database_credentials").await?;
    
    Ok(())
}
```

### Encryption Algorithms

**Supported Algorithms:**
- **ChaCha20Poly1305**: Modern AEAD cipher (default)
- **AES-256-GCM**: NIST-approved AEAD cipher
- **XSalsa20Poly1305**: Extended nonce AEAD cipher

**Key Derivation:**
- **Argon2id**: Memory-hard key derivation (default)
- **scrypt**: Alternative memory-hard function
- **PBKDF2**: NIST-approved key derivation

## Cryptographic Features

The system includes comprehensive cryptographic configuration for database encryption and secure operations.

### Crypto Configuration

```rust
use datafold::config::crypto::{CryptoConfig, MasterKeyConfig, KeyDerivationConfig};

// Passphrase-based encryption
let crypto_config = CryptoConfig::with_passphrase("strong_passphrase".to_string());

// Random key generation
let crypto_config = CryptoConfig::with_random_key();

// Enhanced security configuration
let crypto_config = CryptoConfig::with_enhanced_security("strong_passphrase".to_string());
```

### Master Key Configuration

```toml
[crypto]
enabled = true

[crypto.master_key]
type = "passphrase"  # or "random", "derived", "external"
# passphrase stored securely in keystore

[crypto.key_derivation]
preset = "interactive"  # or "sensitive", "standard"
memory_cost_kb = 65536
time_cost = 2
parallelism = 1

[crypto.algorithms]
symmetric = "ChaCha20Poly1305"  # or "AES-256-GCM"
key_derivation = "Argon2id"     # or "scrypt", "PBKDF2"
hash = "BLAKE3"                 # or "SHA-256", "SHA-3"
```

### Security Levels

```rust
use datafold::security_types::SecurityLevel;

// Configure based on security requirements
let config = match security_level {
    SecurityLevel::Minimal => KeyDerivationConfig::for_security_level(SecurityLevel::Minimal),
    SecurityLevel::Standard => KeyDerivationConfig::for_security_level(SecurityLevel::Standard),
    SecurityLevel::Strict => KeyDerivationConfig::for_security_level(SecurityLevel::Strict),
};
```

**Security Level Comparison:**

| Level | Memory Cost | Time Cost | Use Case |
|-------|-------------|-----------|----------|
| Minimal | 32 MB | 1 | Development, testing |
| Standard | 64 MB | 2 | Production applications |
| Strict | 128 MB | 3 | High-security environments |

## Access Control

The system implements comprehensive access control mechanisms at multiple levels.

### File System Permissions

```toml
[security.file_permissions]
# Configuration directory permissions
config_dir_mode = "0700"  # rwx------
config_file_mode = "0600" # rw-------

# Data directory permissions
data_dir_mode = "0700"
data_file_mode = "0600"

# Log file permissions
log_file_mode = "0640"    # rw-r-----
log_dir_mode = "0750"     # rwxr-x---
```

### User and Group Management

```bash
# Create dedicated user for DataFold services
sudo useradd -r -s /bin/false -d /var/lib/datafold datafold

# Set up directory ownership
sudo chown -R datafold:datafold /var/lib/datafold
sudo chown -R datafold:datafold /etc/datafold

# Configure sudo access for management
echo "admin ALL=(datafold) NOPASSWD: /usr/local/bin/datafold_cli" | sudo tee /etc/sudoers.d/datafold
```

### Process Isolation

```toml
[security.process]
# Run with minimal privileges
drop_privileges = true
target_user = "datafold"
target_group = "datafold"

# Restrict system calls (Linux)
enable_seccomp = true
allowed_syscalls = ["read", "write", "open", "close", "mmap", "munmap"]

# Resource limits
max_memory_mb = 512
max_file_descriptors = 1024
max_cpu_percent = 50
```

### Network Security

```toml
[security.network]
# TLS configuration
tls_min_version = "1.3"
cipher_suites = ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"]
require_client_certificates = true

# Connection limits
max_connections = 100
connection_timeout_seconds = 30
rate_limit_requests_per_minute = 1000

# IP restrictions
allowed_ips = ["10.0.0.0/8", "192.168.0.0/16"]
blocked_ips = []
```

## Security Best Practices

### Configuration Security

1. **Use Strong Passwords**
   ```toml
   [security.password_policy]
   min_length = 12
   require_uppercase = true
   require_lowercase = true
   require_numbers = true
   require_symbols = true
   prohibit_common_passwords = true
   ```

2. **Enable Encryption**
   ```toml
   [security.encryption]
   encrypt_configuration = true
   encrypt_at_rest = true
   encrypt_in_transit = true
   ```

3. **Regular Key Rotation**
   ```toml
   [security.key_rotation]
   enabled = true
   rotation_interval_days = 90
   keep_old_keys_count = 3
   automatic_rotation = true
   ```

4. **Secure Defaults**
   ```toml
   [security.defaults]
   deny_by_default = true
   require_authentication = true
   enable_audit_logging = true
   disable_debug_in_production = true
   ```

### Development Security

```rust
// Secure configuration loading
async fn load_secure_config() -> Result<Config, ConfigError> {
    let manager = EnhancedConfigurationManager::new().await?;
    
    // Validate security settings
    let config = manager.get_enhanced().await?;
    if !config.security_enhanced.encryption_enabled {
        return Err(ConfigError::Security("Encryption required in production".to_string()));
    }
    
    // Check for insecure configurations
    if config.base.get_value("debug.enabled")?.as_bool().unwrap_or(false) {
        log::warn!("Debug mode enabled - ensure this is not production");
    }
    
    Ok(config.base)
}

// Secure credential handling
async fn handle_credentials_securely() -> Result<(), Box<dyn std::error::Error>> {
    let keystore = create_platform_keystore().await?;
    
    // Never log sensitive data
    let api_key = keystore.get_secret("api_key").await?;
    log::info!("API key loaded (length: {} bytes)", api_key.as_ref().map(|k| k.len()).unwrap_or(0));
    
    // Clear sensitive data from memory
    if let Some(mut key) = api_key {
        // Use the key...
        
        // Explicitly zero out sensitive memory
        key.fill(0);
    }
    
    Ok(())
}
```

### Deployment Security

```bash
#!/bin/bash
# secure_deployment.sh

# Set strict file permissions
umask 077

# Create secure configuration
datafold_cli config generate --security-level strict --output /etc/datafold/config.toml
sudo chown root:datafold /etc/datafold/config.toml
sudo chmod 640 /etc/datafold/config.toml

# Enable security features
datafold_cli config set security.encryption.enabled true
datafold_cli config set security.audit.enabled true
datafold_cli config set security.access_control.strict_mode true

# Configure keystore
datafold_cli keystore init --require-authentication
datafold_cli keystore set api_key --prompt-secure

# Verify security configuration
datafold_cli security audit
```

## Audit and Compliance

### Audit Logging

```toml
[security.audit]
enabled = true
log_level = "info"
include_success = true
include_failures = true
include_access_attempts = true

[security.audit.events]
# Configuration events
config_read = true
config_write = true
config_delete = true
config_validate = true

# Security events
authentication_success = true
authentication_failure = true
authorization_failure = true
keystore_access = true
encryption_operations = true

# Administrative events
user_creation = true
permission_changes = true
key_rotation = true
security_policy_changes = true
```

### Audit Log Format

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "event_type": "config_read",
  "severity": "info",
  "user": "admin",
  "process_id": 1234,
  "session_id": "session_abc123",
  "source_ip": "192.168.1.100",
  "resource": "/etc/datafold/config.toml",
  "operation": "read_configuration",
  "result": "success",
  "details": {
    "sections_accessed": ["database", "api"],
    "duration_ms": 45,
    "bytes_read": 2048
  },
  "security_context": {
    "encryption_used": true,
    "authentication_method": "keystore",
    "authorization_granted": true
  }
}
```

### Compliance Features

#### SOX Compliance
- **Audit trails** for all configuration changes
- **Access controls** with principle of least privilege
- **Data integrity** through cryptographic hashing
- **Change management** with approval workflows

#### GDPR Compliance
- **Data minimization** - only necessary data stored
- **Right to erasure** - secure deletion capabilities
- **Data portability** - standard configuration formats
- **Privacy by design** - encrypted storage by default

#### SOC 2 Compliance
- **Security monitoring** with real-time alerts
- **Availability controls** with redundancy
- **Processing integrity** through validation
- **Confidentiality** via encryption
- **Privacy controls** with access logging

### Compliance Configuration

```toml
[security.compliance]
# Enable compliance features
sox_compliance = true
gdpr_compliance = true
soc2_compliance = true

[security.compliance.sox]
audit_trail_retention_days = 2555  # 7 years
require_change_approval = true
segregation_of_duties = true

[security.compliance.gdpr]
data_retention_days = 730  # 2 years
enable_right_to_erasure = true
log_data_access = true
pseudonymization = true

[security.compliance.soc2]
continuous_monitoring = true
incident_response_enabled = true
vulnerability_scanning = true
penetration_testing = true
```

## Threat Model

### Threat Analysis

#### 1. Configuration File Exposure

**Threat**: Unauthorized access to configuration files
**Impact**: Credential theft, system compromise
**Mitigation**:
- File system permissions (0600)
- Configuration encryption
- Keystore integration
- Access monitoring

#### 2. Memory Attacks

**Threat**: Sensitive data extraction from memory
**Impact**: Credential exposure, data breach
**Mitigation**:
- Memory encryption
- Secure memory clearing
- Process isolation
- Core dump prevention

#### 3. Network Interception

**Threat**: Configuration data interception in transit
**Impact**: Credential theft, man-in-the-middle attacks
**Mitigation**:
- TLS 1.3 encryption
- Certificate pinning
- Mutual authentication
- Network segmentation

#### 4. Privilege Escalation

**Threat**: Unauthorized privilege elevation
**Impact**: System compromise, data breach
**Mitigation**:
- Principle of least privilege
- Role-based access control
- Process sandboxing
- Regular permission audits

#### 5. Supply Chain Attacks

**Threat**: Compromised dependencies or build process
**Impact**: Code injection, backdoors
**Mitigation**:
- Dependency scanning
- Code signing verification
- Build reproducibility
- Security scanning

### Security Controls Matrix

| Threat Category | Control Type | Implementation | Effectiveness |
|-----------------|--------------|----------------|---------------|
| Data at Rest | Encryption | File-level encryption | High |
| Data in Transit | TLS | TLS 1.3 with strong ciphers | High |
| Access Control | Authentication | Keystore + MFA | High |
| Audit | Logging | Comprehensive audit logs | Medium |
| Code Integrity | Signing | Digital signatures | High |
| Process Security | Isolation | Sandboxing + containers | Medium |

## Security Monitoring

### Real-time Monitoring

```toml
[security.monitoring]
enabled = true
real_time_alerts = true
alert_thresholds = "strict"

[security.monitoring.metrics]
# Failed authentication attempts
max_auth_failures_per_minute = 5
max_auth_failures_per_hour = 20

# Configuration access patterns
max_config_reads_per_minute = 100
unusual_access_pattern_detection = true

# Keystore operations
monitor_keystore_access = true
alert_on_keystore_failures = true

# File system monitoring
monitor_config_file_changes = true
alert_on_permission_changes = true
```

### Security Dashboards

Key metrics to monitor:
- Authentication success/failure rates
- Configuration access patterns
- Keystore operation statistics
- Encryption/decryption performance
- File permission changes
- Network connection attempts
- Resource usage anomalies

### Incident Response

```toml
[security.incident_response]
enabled = true
automatic_response = true

[security.incident_response.actions]
# Automatic lockout after failed attempts
lockout_after_failures = 5
lockout_duration_minutes = 30

# Configuration protection
freeze_config_on_breach = true
backup_config_on_incident = true

# Alerting
email_alerts = ["security@company.com"]
sms_alerts = ["+1234567890"]
webhook_alerts = ["https://alerting.company.com/webhook"]

# Forensics
enable_forensic_logging = true
capture_memory_dumps = false
preserve_evidence = true
```

### Security Testing

```bash
#!/bin/bash
# security_test_suite.sh

echo "Running security tests..."

# Test file permissions
echo "Testing file permissions..."
datafold_cli security test-permissions

# Test encryption
echo "Testing encryption..."
datafold_cli security test-encryption

# Test keystore integration
echo "Testing keystore..."
datafold_cli security test-keystore

# Test access controls
echo "Testing access controls..."
datafold_cli security test-access-control

# Vulnerability scanning
echo "Running vulnerability scan..."
datafold_cli security scan-vulnerabilities

# Generate security report
echo "Generating security report..."
datafold_cli security report --output security_report_$(date +%Y%m%d).json

echo "Security tests completed."
```

### Penetration Testing

Regular security assessments should include:

1. **Configuration Security Testing**
   - File permission bypasses
   - Configuration injection attacks
   - Credential extraction attempts

2. **Keystore Security Testing**
   - Platform keystore bypasses
   - Memory extraction attacks
   - Side-channel analysis

3. **Network Security Testing**
   - TLS configuration testing
   - Certificate validation bypasses
   - Protocol downgrade attacks

4. **Access Control Testing**
   - Privilege escalation attempts
   - Authentication bypasses
   - Authorization flaws

## Related Documentation

- [Architecture](architecture.md) - System architecture and design
- [API Reference](api.md) - Complete API documentation
- [Integration Guide](integration.md) - Integration patterns and examples
- [Deployment Guide](deployment.md) - Deployment and migration procedures