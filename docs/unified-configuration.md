# Unified Configuration Management [DEPRECATED]

**⚠️ DEPRECATED: This document describes the legacy unified configuration system (PBI-16) which has been superseded by the Cross-Platform Configuration Management System (PBI-27).**

## Migration Notice

The unified configuration system described in this document has been **fully replaced** by the new cross-platform configuration management system. Please refer to the new documentation:

- **[Cross-Platform Configuration Architecture](config/architecture.md)** - Complete system overview
- **[Configuration API Reference](config/api.md)** - API documentation and usage examples
- **[Integration Guide](config/integration.md)** - Migration and integration patterns
- **[Deployment Guide](config/deployment.md)** - Deployment and migration procedures
- **[Security Guide](config/security.md)** - Security features and best practices

## Automatic Migration

The new system includes comprehensive migration utilities to automatically convert existing unified configurations:

```bash
# Migrate all configurations automatically
datafold_cli migrate --all --backup

# Migrate specific unified configuration
datafold_cli migrate --system unified --backup
```

## Key Improvements in PBI-27

The new cross-platform configuration system provides significant improvements over the legacy unified system:

### Enhanced Platform Support
- **Native path resolution** following OS-specific conventions (XDG, Apple HIG, Windows)
- **Platform-specific optimizations** for file operations and caching
- **Native keystore integration** (GNOME Keyring, Keychain Services, Credential Manager)

### Improved Security
- **Encrypted configuration sections** for sensitive data
- **Hardware-backed security** where available (Secure Enclave, TPM)
- **Comprehensive audit logging** for compliance requirements
- **Secure defaults** with defense-in-depth architecture

### Better Performance
- **Intelligent caching** with automatic invalidation
- **Real-time file watching** with platform-optimized change detection
- **Memory-efficient operations** with streaming and lazy loading
- **Atomic write operations** preventing configuration corruption

### Developer Experience
- **Type-safe configuration values** with runtime validation
- **Comprehensive error handling** with detailed context
- **Hot reloading** with change notification callbacks
- **Extensive testing framework** with platform compatibility validation

## Legacy Configuration Format

The legacy unified configuration used this JSON format:

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

## New Configuration Format

The new system uses TOML format with enhanced structure and security:

```toml
# config.toml - New cross-platform configuration
[environment]
name = "production"
debug_mode = false

[cli]
default_profile = "production"
output_format = "table"
verbosity = 1

[cli.profiles.production]
name = "production"
api_endpoint = "https://api.datafold.com"
# Credentials stored securely in platform keystore

[security]
encryption_enabled = true
use_keystore = true
audit_logging = true

[performance]
cache_size_mb = 128
enable_file_watching = true
use_memory_mapping = true

[platform]
use_platform_paths = true
keystore_service = "auto"  # Platform-specific
```

## Migration Mapping

| Legacy Section | New Location | Notes |
|----------------|--------------|-------|
| `environments.*.signing` | `[security.signing]` | Enhanced with keystore integration |
| `environments.*.verification` | `[security.verification]` | Improved validation |
| `environments.*.logging` | `[logging]` | Platform-aware path resolution |
| `environments.*.authentication` | `[cli.authentication]` | Keystore-backed credentials |
| `environments.*.performance` | `[performance]` | Platform-specific optimizations |
| `security_profiles` | `[security.*]` | Granular security controls |
| `defaults` | Various sections | Distributed to appropriate sections |

## Support Timeline

- **Legacy system removal**: The unified configuration system has been fully removed as of PBI-27 completion
- **Migration support**: Automated migration tools will continue to be maintained
- **Documentation**: This document serves as historical reference only

## Getting Help

For migration assistance or questions about the new configuration system:

1. **Read the new documentation**: Start with [Configuration Architecture](config/architecture.md)
2. **Use migration tools**: Run `datafold_cli migrate --help` for options
3. **Check examples**: See practical examples in [Integration Guide](config/integration.md)
4. **Report issues**: File migration issues in the project repository

---

**For all new development and deployments, use the [Cross-Platform Configuration System](config/architecture.md) documented in the `docs/config/` directory.**