# PBI-15: Security Architecture Policy Consolidation

## Overview

This PBI implements Phase 1 of the security architecture simplification plan, consolidating verification policy definitions into a unified schema. Currently, DataFold has ~828 lines of duplicated policy logic across JavaScript SDK, Python SDK, and Rust CLI platforms. This consolidation will eliminate 94% of duplicated code while maintaining backward compatibility and enabling runtime policy updates.

[View in Backlog](../backlog.md#user-content-15)

## Problem Statement

The DataFold codebase contains significant policy duplication:
- **JS SDK**: [`js-sdk/src/verification/policies.ts`](../../../js-sdk/src/verification/policies.ts) - 385 lines of policy definitions
- **Python SDK**: [`python-sdk/src/datafold_sdk/verification/policies.py`](../../../python-sdk/src/datafold_sdk/verification/policies.py) - 443 lines of policy definitions
- **Verification Rules**: ~149 lines of identical custom rules duplicated across platforms

This duplication leads to:
- Inconsistent policy behavior across platforms
- Maintenance overhead requiring changes in multiple locations
- Risk of policy drift between implementations
- Inability to update policies without code deployment

## User Stories

**Primary User Story**: As a security architect, I want to consolidate verification policy definitions into a unified schema so that we eliminate 94% of duplicated code and maintain consistency across all platforms.

**Supporting User Stories**:
- As a developer, I want a single source of truth for policies so I don't have to maintain multiple implementations
- As a DevOps engineer, I want to update security policies without deploying code changes
- As a security administrator, I want consistent policy behavior across all client platforms

## Technical Approach

### Implementation Strategy

1. **Create Unified Policy Schema**
   - JSON schema definition in [`schemas/unified-policy-schema.json`](../../../schemas/unified-policy-schema.json)
   - Policy definitions in [`config/verification-policies.json`](../../../config/verification-policies.json)

2. **Platform Migration (Backward Compatible)**
   ```typescript
   // Phase 1: Load policies from JSON (backward compatible)
   export const STRICT_VERIFICATION_POLICY = loadPolicyFromJSON('strict');
   
   // Phase 2: Deprecation warnings  
   export const STRICT_VERIFICATION_POLICY = deprecatedPolicyWrapper(loadPolicyFromJSON('strict'));
   
   // Phase 3: Remove old APIs (breaking change - major version)
   ```

3. **Policy Loader Implementation**
   - Cross-platform policy loading utilities
   - Runtime validation against unified schema
   - Caching and performance optimization

### Files to be Modified
- [`js-sdk/src/verification/policies.ts`](../../../js-sdk/src/verification/policies.ts)
- [`python-sdk/src/datafold_sdk/verification/policies.py`](../../../python-sdk/src/datafold_sdk/verification/policies.py)
- [`src/cli/signing_config.rs`](../../../src/cli/signing_config.rs)

### Technical Benefits
- **Code Reduction**: ~828 lines → ~50 lines (94% reduction)
- **Single Source of Truth**: Policies defined once, used everywhere
- **Runtime Updates**: Change policies without code deployment
- **Consistency**: Identical behavior across all platforms

## UX/UI Considerations

This is primarily a backend/infrastructure change with minimal direct user interface impact. Key considerations:

- **Backward Compatibility**: Existing policy APIs continue to work during migration
- **Error Messages**: Clear validation errors when policy JSON is malformed
- **Documentation**: Migration guides for developers updating their code
- **Deprecation Timeline**: Clear communication about API deprecation schedule

## Acceptance Criteria

- [ ] Unified JSON schema created and validated
- [ ] Policy configuration file with all existing policies (STRICT, STANDARD, LENIENT, LEGACY)
- [ ] JavaScript SDK loads policies from unified config with backward compatibility
- [ ] Python SDK loads policies from unified config with backward compatibility
- [ ] Rust CLI integrates with unified policy system
- [ ] All existing tests pass with new policy loading mechanism
- [ ] Runtime policy validation implemented
- [ ] Deprecation warnings added to old policy APIs
- [ ] Code reduction of at least 90% achieved in policy definitions
- [ ] Documentation updated with migration guide
- [ ] Performance benchmarks show no regression in policy loading

## Dependencies

- **Prerequisites**: None - this is foundational work
- **Concurrent**: Can be developed independently of other PBIs
- **Dependent PBIs**: PBI-16 (Configuration Unification) and PBI-17 (Event Bus) will build on this foundation

## Open Questions

1. **Policy Versioning**: Should we implement semantic versioning for policy schemas to handle future changes?
2. **Hot Reloading**: Should runtime policy updates trigger automatic reload, or require explicit refresh?
3. **Validation Performance**: What's the acceptable performance overhead for JSON schema validation?
4. **Migration Timeline**: What's the appropriate deprecation period for old policy APIs?

## Related Tasks

Tasks will be created upon PBI approval and moved to "Agreed" status. Expected tasks include:

1. Create unified policy JSON schema
2. Implement policy configuration file with existing policies
3. Create JavaScript SDK policy loader with backward compatibility
4. Create Python SDK policy loader with backward compatibility  
5. Integrate Rust CLI with unified policy system
6. Add deprecation warnings to existing policy APIs
7. Update documentation and create migration guide
8. Performance testing and optimization
9. E2E CoS Test for policy consolidation