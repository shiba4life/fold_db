# PBI-16: Security Architecture Configuration Unification

## Overview

This PBI implements Phase 2 of the security architecture simplification plan, creating unified configuration management across all DataFold components. Currently, DataFold has 3 separate, overlapping configuration approaches across Rust CLI, JavaScript SDK, and Python SDK. This unification will provide consistent environment management, simplified deployment, and configuration as code capabilities.

[View in Backlog](../backlog.md#user-content-16)

## Problem Statement

The DataFold codebase has fragmented configuration management:
- **Rust CLI**: Custom [`src/cli/signing_config.rs`](../../../src/cli/signing_config.rs) with specific signing configuration structure
- **JS SDK**: Custom VerificationConfig structures with platform-specific options
- **Python SDK**: Separate configuration patterns that don't align with other platforms

This fragmentation leads to:
- Inconsistent configuration patterns across platforms
- Complex deployment procedures requiring multiple config files
- Difficulty managing different environments (dev/staging/prod)
- Manual configuration synchronization between components
- DevOps complexity and potential for configuration drift

## User Stories

**Primary User Story**: As a DevOps engineer, I want unified configuration management across all DataFold components so that I can deploy and manage environments consistently without maintaining separate config systems.

**Supporting User Stories**:
- As a system administrator, I want environment-specific configurations so I can easily manage dev/staging/prod deployments
- As a developer, I want configuration as code so changes are version-controlled and auditable
- As a security engineer, I want consistent security settings across all platforms so there are no configuration gaps

## Technical Approach

### Implementation Strategy

1. **Create Unified Configuration Schema**
   - JSON configuration format in [`config/unified-datafold-config.json`](../../../config/unified-datafold-config.json)
   - Environment-specific sections (development, staging, production)
   - Cross-platform configuration validation

2. **Platform Adapters (Backward Compatible)**
   ```rust
   // Rust CLI adapter
   impl From<UnifiedConfig> for CliSigningConfig {
       fn from(config: UnifiedConfig) -> Self {
           Self {
               required_components: config.environments.production.signing.components.always_include,
               include_content_digest: config.environments.production.signing.components.include_content_digest,
               // ... other mappings
           }
       }
   }
   ```

3. **Cross-Platform Configuration Libraries**
   - [`src/config/unified_config.rs`](../../../src/config/unified_config.rs)
   - [`js-sdk/src/config/unified-config.ts`](../../../js-sdk/src/config/unified-config.ts)
   - [`python-sdk/src/datafold_sdk/config/unified_config.py`](../../../python-sdk/src/datafold_sdk/config/unified_config.py)

### Configuration Structure
```json
{
  "environments": {
    "development": {
      "signing": { "policy": "lenient", "timeout": 60 },
      "verification": { "strict_timing": false },
      "logging": { "level": "debug" }
    },
    "production": {
      "signing": { "policy": "strict", "timeout": 30 },
      "verification": { "strict_timing": true },
      "logging": { "level": "info" }
    }
  }
}
```

### Technical Benefits
- **Environment Management**: Easy dev/staging/prod configuration
- **Cross-Platform Consistency**: Same config format everywhere
- **Simplified Deployment**: Single config file for all components
- **Better DevOps**: Configuration as code with version control

## UX/UI Considerations

This change primarily affects deployment and configuration workflows:

- **Migration Path**: Existing configurations continue to work through adapters
- **Environment Selection**: Clear environment specification in configuration
- **Validation Feedback**: Helpful error messages for configuration validation failures
- **Documentation**: Comprehensive migration and usage guides
- **Tooling**: Configuration validation and environment switching utilities

## Acceptance Criteria

- [ ] Unified configuration schema created and documented
- [ ] Environment-specific configuration sections (dev/staging/prod) implemented
- [ ] Rust CLI adapter from unified config to existing CliSigningConfig
- [ ] JavaScript SDK configuration loader with environment selection
- [ ] Python SDK configuration loader with environment selection
- [ ] Configuration validation across all platforms
- [ ] Backward compatibility with existing configuration files
- [ ] Environment switching utilities for each platform
- [ ] Cross-platform configuration integration tests
- [ ] Migration guide and documentation updated
- [ ] Configuration as code workflows documented
- [ ] Performance benchmarks show no regression in config loading

## Dependencies

- **Prerequisites**: PBI-15 (Policy Consolidation) - unified policies will be referenced in configuration
- **Concurrent**: Can leverage policy loading infrastructure from PBI-15
- **Dependent PBIs**: PBI-17 (Event Bus) will use unified configuration for event routing

## Open Questions

1. **Environment Detection**: Should environment be auto-detected from deployment context or explicitly specified?
2. **Configuration Override**: What's the priority order for configuration sources (file, environment variables, CLI args)?
3. **Secret Management**: How should sensitive configuration values (keys, tokens) be handled?
4. **Hot Reloading**: Should configuration changes trigger automatic reloading in running services?
5. **Validation Strictness**: Should configuration validation be strict (fail on unknown fields) or permissive?

## Related Tasks

Tasks will be created upon PBI approval and moved to "Agreed" status. Expected tasks include:

1. Design unified configuration schema and format
2. Create unified configuration JSON template with environment sections
3. Implement Rust configuration adapter and loading utilities
4. Implement JavaScript SDK configuration loader with environment support
5. Implement Python SDK configuration loader with environment support
6. Create cross-platform configuration validation
7. Develop environment switching and management utilities
8. Add backward compatibility adapters for existing configurations
9. Create configuration migration tools and scripts
10. Update documentation with configuration management guide
11. E2E CoS Test for configuration unification