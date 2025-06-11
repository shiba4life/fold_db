# Unified Configuration Management

**PBI-16: Security Architecture Configuration Unification**

This document describes the unified configuration system that provides consistent configuration management across all DataFold components (Rust CLI, JavaScript SDK, and Python SDK).

## Overview

The unified configuration system eliminates the fragmented configuration patterns that previously existed across platforms. All components now use a single, unified configuration format with environment-specific sections as the primary and only configuration system.

## Configuration File Structure

The unified configuration is stored in [`config/unified-datafold-config.json`](../config/unified-datafold-config.json) and follows this structure:

```json
{
  "config_format_version": "1.0",
  "environments": {
    "development": { /* dev-specific settings */ },
    "staging": { /* staging-specific settings */ },
    "production": { /* production-specific settings */ }
  },
  "security_profiles": {
    "minimal": { /* basic security settings */ },
    "standard": { /* balanced security settings */ },
    "strict": { /* maximum security settings */ }
  },
  "defaults": {
    "environment": "development",
    "signing_mode": "manual",
    "output_format": "table",
    "verbosity": 1
  }
}
```

## Environment Configuration

Each environment contains these configuration sections:

### Signing Configuration
- **policy**: Security profile to use (minimal, standard, strict)
- **timeout_ms**: Signing timeout in milliseconds
- **required_components**: List of required signature components
- **include_content_digest**: Whether to include content digest
- **include_timestamp**: Whether to include timestamp
- **include_nonce**: Whether to include nonce
- **max_body_size_mb**: Maximum body size for digest calculation
- **debug**: Debug logging configuration

### Verification Configuration
- **strict_timing**: Whether to enforce strict timing checks
- **allow_clock_skew_seconds**: Allowed clock skew in seconds
- **require_nonce**: Whether to require nonce in signatures
- **max_signature_age_seconds**: Maximum signature age in seconds

### Logging Configuration
- **level**: Log level (debug, info, warn, error)
- **colored_output**: Whether to use colored output
- **structured**: Whether to use structured logging

### Authentication Configuration
- **store_tokens**: Whether to store authentication tokens
- **auto_update_check**: Whether to automatically check for updates
- **prompt_on_first_sign**: Whether to prompt on first signature

### Performance Configuration
- **cache_keys**: Whether to cache keys
- **max_concurrent_signs**: Maximum concurrent signing operations
- **default_timeout_secs**: Default timeout in seconds
- **default_max_retries**: Default maximum retries

## Platform Integration

### Rust CLI

The Rust CLI uses the [`UnifiedConfigManager`](../src/config/unified_config.rs) directly for configuration management:

```rust
use datafold::config::unified_config::UnifiedConfigManager;

// Load unified configuration
let manager = UnifiedConfigManager::load_default()?;

// Access environment configuration
let env_config = manager.current_environment_config()?;

// Switch environments
manager.set_environment("production".to_string())?;
```

#### Environment Management Commands

The CLI provides environment management commands:

```bash
# List all environments
datafold environment list

# Show current environment
datafold environment show

# Switch to production environment
datafold environment switch production

# Compare environments
datafold environment compare development production

# Validate all environments
datafold environment validate

# Export environment variables
datafold environment export --environment production
```

### JavaScript SDK

The JavaScript SDK uses the [`UnifiedConfigManager`](../js-sdk/src/config/unified-config.ts) class:

```typescript
import { UnifiedConfigManager, loadDefaultUnifiedConfig } from '@datafold/sdk';

// Load unified configuration
const configManager = await loadDefaultUnifiedConfig('production');

// Convert to signing configuration
const signingConfig = configManager.toSigningConfig(keyId, privateKey);

// Get environment-specific settings
const loggingConfig = configManager.getLoggingConfig();
const performanceConfig = configManager.getPerformanceConfig();
```

### Python SDK

The Python SDK uses the [`UnifiedConfigManager`](../python-sdk/src/datafold_sdk/config/unified_config.py) class:

```python
from datafold_sdk.config import load_default_unified_config

# Load unified configuration
config_manager = load_default_unified_config('production')

# Convert to signing configuration
signing_config = config_manager.to_signing_config(key_id, private_key)

# Get environment-specific settings
logging_config = config_manager.get_logging_config()
performance_config = config_manager.get_performance_config()
```

## Environment Management

### Switching Environments

You can switch between environments programmatically:

```rust
// Rust
manager.set_environment("production".to_string())?;
```

```typescript
// TypeScript
configManager.setEnvironment('production');
```

```python
# Python
config_manager.set_environment('production')
```

### Environment Variables

Export environment-specific variables for CI/CD and scripting:

```bash
# Export current environment variables
datafold environment export

# Export specific environment
datafold environment export --environment production
```

Output example:
```bash
export DATAFOLD_ENVIRONMENT=production
export DATAFOLD_SIGNING_POLICY=strict
export DATAFOLD_SIGNING_TIMEOUT_MS=2000
export DATAFOLD_CONTENT_DIGEST=true
export DATAFOLD_STRICT_TIMING=true
export DATAFOLD_LOG_LEVEL=warn
export DATAFOLD_CACHE_KEYS=false
```

## Security Profiles

Security profiles define reusable security configurations:

### Minimal Profile
- Basic signing for low-latency scenarios
- Required components: `@method`, `@target-uri`
- No content digest
- Custom nonces allowed

### Standard Profile  
- Balanced security for most applications
- Required components: `@method`, `@target-uri`, `content-type`
- Content digest included
- Nonce validation enabled

### Strict Profile
- Maximum security with comprehensive coverage
- Required components: `@method`, `@target-uri`, `content-type`, `content-length`, `user-agent`
- SHA-512 content digest
- Strict nonce validation

## Configuration Validation

The unified configuration system includes comprehensive validation:

- **Environment validation**: Ensures all environments reference valid security profiles
- **Performance validation**: Validates timeout and concurrency settings
- **Security validation**: Ensures security profiles are properly configured
- **Cross-platform consistency**: Validates configuration works across all platforms

## Simplified Configuration Architecture

The unified configuration system has been simplified to be the primary and only configuration system:

- All components now use unified configuration directly
- Legacy backward compatibility adapters have been removed
- Simplified codebase with unified configuration as the single source of truth
- Environment-specific configuration is the standard approach across all platforms

## Benefits

1. **Environment Management**: Easy switching between dev/staging/prod configurations
2. **Cross-Platform Consistency**: Same configuration format across all platforms
3. **Simplified Deployment**: Single configuration file for all components
4. **Configuration as Code**: Version-controlled, auditable configuration changes
5. **Reduced Complexity**: Eliminates configuration drift and synchronization issues

## Testing

The unified configuration system includes comprehensive integration tests:

- Cross-platform configuration loading
- Environment switching functionality
- Configuration validation
- CLI adapter integration
- Performance benchmarks

Run tests with:
```bash
cargo test integration_unified_config_test
```

## Implementation Status

- ✅ Unified configuration schema and file format
- ✅ Environment-specific configuration sections
- ✅ Rust CLI adapter and environment utilities
- ✅ JavaScript SDK configuration loader
- ✅ Python SDK configuration loader
- ✅ Configuration validation across all platforms
- ✅ Backward compatibility with existing configurations
- ✅ Environment switching utilities
- ✅ Cross-platform integration tests
- ✅ CLI environment management commands
- ✅ Documentation and usage guides

PBI-16 is complete with all acceptance criteria met.