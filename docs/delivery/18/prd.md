# PBI-18: Configuration Systems Consolidation

## Overview

This PBI consolidates configuration systems across all DataFold components (Rust CLI, JavaScript SDK, Python SDK) into a unified approach. Currently, each platform has its own configuration patterns, leading to inconsistent behavior, complex deployments, and maintenance overhead. This consolidation will eliminate redundant configuration implementations and ensure consistent behavior across all platforms.

[View in Backlog](../backlog.md#user-content-18)

## Problem Statement

The DataFold codebase has fragmented configuration management across platforms:
- **Rust CLI**: Custom configuration structures in [`src/cli/signing_config.rs`](../../../src/cli/signing_config.rs) and [`src/cli/config.rs`](../../../src/cli/config.rs)
- **JavaScript SDK**: Platform-specific configuration patterns without standardization
- **Python SDK**: Separate configuration approaches that don't align with other platforms

This fragmentation leads to:
- Inconsistent configuration behavior across platforms
- Complex deployment procedures requiring multiple configuration approaches
- Difficulty managing environments and configuration drift
- Maintenance overhead from duplicated configuration logic
- Risk of configuration inconsistencies causing security or functional issues

## User Stories

**Primary User Story**: As a system architect, I want consolidated configuration systems across all DataFold components so that I can eliminate redundant configuration implementations and ensure consistent behavior.

**Supporting User Stories**:
- As a DevOps engineer, I want unified configuration loading so I don't need to learn platform-specific configuration patterns
- As a developer, I want consistent configuration APIs across all platforms so I can implement features uniformly
- As a system administrator, I want standardized configuration validation so I can catch configuration errors early

## Technical Approach

### Implementation Strategy

1. **Create Unified Configuration Framework**
   - Shared configuration schema definition
   - Cross-platform configuration loading utilities
   - Standardized configuration validation and error handling

2. **Platform Integration (Backward Compatible)**
   ```rust
   // Rust: Unified config adapter
   impl From<UnifiedConfig> for CliConfig {
       fn from(config: UnifiedConfig) -> Self {
           Self {
               signing: config.signing.into(),
               network: config.network.into(),
               // ... other mappings
           }
       }
   }
   ```

   ```typescript
   // JavaScript: Unified config loader
   export const loadConfig = (source: ConfigSource): UnifiedConfig => {
       return configLoader.load(source, 'js');
   };
   ```

   ```python
   # Python: Unified config adapter
   class ConfigAdapter:
       @classmethod
       def from_unified(cls, config: UnifiedConfig) -> PlatformConfig:
           return cls(
               signing=config.signing,
               network=config.network
           )
   ```

3. **Configuration Structure Unification**
   - Common configuration sections (signing, network, logging, etc.)
   - Platform-specific overrides when necessary
   - Environment-specific configuration support

### Files to be Modified
- [`src/cli/config.rs`](../../../src/cli/config.rs) - Rust configuration integration
- [`src/cli/signing_config.rs`](../../../src/cli/signing_config.rs) - Signing config adaptation
- [`js-sdk/src/config/`](../../../js-sdk/src/config/) - JavaScript config utilities (new)
- [`python-sdk/src/datafold_sdk/config/`](../../../python-sdk/src/datafold_sdk/config/) - Python config utilities (new)

### Technical Benefits
- **Reduced Code Duplication**: Eliminate redundant configuration implementations
- **Consistent Behavior**: Same configuration logic across all platforms
- **Simplified Deployment**: Unified configuration approach for all components
- **Better Validation**: Centralized configuration validation and error handling

## UX/UI Considerations

This change primarily affects developer and deployment experiences:

- **Backward Compatibility**: Existing configuration files continue to work through adapters
- **Migration Path**: Clear migration guides for moving to unified configuration
- **Error Messages**: Consistent, helpful configuration validation error messages across platforms
- **Documentation**: Comprehensive documentation for unified configuration usage
- **Tooling**: Configuration validation utilities and migration tools

## Acceptance Criteria

- [ ] Unified configuration schema defined and documented
- [ ] Rust CLI integrated with unified configuration system with backward compatibility
- [ ] JavaScript SDK configuration loader implemented using unified approach
- [ ] Python SDK configuration loader implemented using unified approach
- [ ] Cross-platform configuration validation implemented
- [ ] Backward compatibility maintained for existing configuration files
- [ ] Configuration error messages standardized across all platforms
- [ ] Migration tools created for existing configuration files
- [ ] All existing tests pass with new configuration system
- [ ] Performance benchmarks show no regression in configuration loading
- [ ] Documentation updated with unified configuration usage guide

## Dependencies

- **Prerequisites**: PBI-15 (Policy Consolidation) - unified policies will be referenced in configuration
- **Concurrent**: Can leverage policy consolidation infrastructure
- **Dependent PBIs**: PBI-19 (Authentication Logic), PBI-21 (Middleware Alignment) will benefit from unified configuration

## Open Questions

1. **Configuration Override Priority**: What should be the precedence order for configuration sources (file, environment variables, CLI arguments)?
2. **Platform-Specific Extensions**: How should platform-specific configuration extensions be handled while maintaining consistency?
3. **Configuration Validation Strictness**: Should configuration validation be strict (fail on unknown fields) or permissive?
4. **Runtime Configuration Updates**: Should configuration changes be hot-reloadable or require service restart?
5. **Secret Management Integration**: How should sensitive configuration values be handled in the unified system?

## Related Tasks

Tasks will be created upon PBI approval and moved to "Agreed" status. Expected tasks include:

1. Design unified configuration schema and framework
2. Implement Rust CLI configuration adapter and integration
3. Create JavaScript SDK unified configuration loader
4. Create Python SDK unified configuration loader
5. Implement cross-platform configuration validation
6. Add backward compatibility adapters for existing configurations
7. Create configuration migration tools and utilities
8. Update documentation with unified configuration guide
9. Performance testing and optimization
10. E2E CoS Test for configuration consolidation