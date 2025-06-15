# PBI-25: Unify security-related enums across modules

[View in Backlog](../backlog.md#user-content-25)

## Overview

This PBI consolidates scattered security-related enums across the DataFold codebase into a unified shared module. Currently, security enums like `RotationStatus`, `ThreatLevel`, `ComplianceStatus`, `RiskAction`, and `SecurityLevel` are duplicated or inconsistently defined across multiple modules, leading to code duplication, maintenance burden, and potential inconsistencies.

The solution involves creating a centralized `security_types.rs` module that defines all security-related enums and refactoring existing modules to use these shared types.

## Problem Statement

**Current Issues:**
1. **Duplication**: `RotationStatus` is defined identically in both `src/db_operations/key_rotation_operations.rs` and `src/crypto/key_rotation_audit.rs`
2. **Inconsistent Usage**: Security enums are scattered across multiple modules without a clear organizational structure
3. **Maintenance Burden**: Changes to security enum definitions require updates in multiple locations
4. **Type Safety Risks**: Different modules may evolve their enum definitions independently, leading to incompatibilities
5. **Documentation Fragmentation**: Security type documentation is spread across multiple files

**Impact:**
- Increased development time due to code duplication
- Higher risk of bugs from inconsistent enum usage
- Difficulty in maintaining security-related functionality
- Reduced code readability and discoverability

## User Stories

**Primary User Story:**
As a security architect, I want all security-related enums to be unified in a shared module so that all components use consistent types and reduce duplication.

**Supporting User Stories:**
- As a developer, I want to import security types from a single location so that I can easily discover and use the correct types
- As a maintainer, I want security enum changes to be made in one place so that updates are consistent across all modules
- As a code reviewer, I want clear separation of security types so that I can easily verify correct usage

## Technical Approach

### 1. Create Shared Security Types Module
- **Location**: `src/security/types.rs` or `src/security_types.rs`
- **Content**: All security-related enums consolidated into a single module
- **Documentation**: Comprehensive documentation for each enum and its variants

### 2. Enum Consolidation Strategy
- **Identify**: All security-related enums across the codebase
- **Analyze**: Differences between duplicate definitions
- **Merge**: Combine compatible definitions, resolve conflicts
- **Standardize**: Ensure consistent naming and documentation

### 3. Module Refactoring
- **Update Imports**: Replace local enum definitions with imports from shared module
- **Remove Duplicates**: Delete redundant enum definitions
- **Update References**: Ensure all usage points reference the shared types
- **Verify Compatibility**: Ensure no breaking changes to existing APIs

### 4. Target Enums for Unification
- `RotationStatus` (currently duplicated)
- `ThreatLevel` (widely used)
- `ComplianceStatus` (compliance module)
- `SecurityLevel` (crypto config)
- `RiskAction` (security assessment)
- `SecurityPattern` (monitoring)
- `SecurityEventSeverity` (event handling)
- `SecurityEventType` (event classification)
- `SecurityProfile` (authentication)

## UX/UI Considerations

This is primarily an internal refactoring with no direct user-facing changes. However:

**Developer Experience:**
- Clear import paths for security types
- Consistent enum naming conventions
- Comprehensive documentation
- IDE auto-completion support

**API Stability:**
- Maintain backward compatibility where possible
- Deprecation warnings for any breaking changes
- Clear migration documentation

## Acceptance Criteria

✅ **All security-related enums are defined in a single shared module**
- Single source of truth for all security enum definitions
- Clear module organization and documentation

✅ **All modules refactored to use shared enums**
- No local security enum definitions remain
- All imports updated to use shared module
- Compilation successful across all modules

✅ **No duplicate or conflicting enum definitions remain**
- Code analysis confirms no security enum duplication
- Consistent enum variant naming and semantics
- Merged conflicting definitions appropriately

✅ **Technical documentation updated to reflect new shared types**
- Module-level documentation for security types
- Updated code comments and examples
- Developer guide for using security types

## Dependencies

**Internal Dependencies:**
- Access to all security modules for analysis and refactoring
- Understanding of existing enum usage patterns
- Coordination with any ongoing security-related development

**External Dependencies:**
- None identified

**Potential Blockers:**
- Active development in security modules that might conflict with refactoring
- Need to ensure backward compatibility for external API consumers

## Open Questions

1. **Module Organization**: Should we create `src/security/types.rs` or use `src/security_types.rs`?
2. **Breaking Changes**: Are there any external APIs that depend on current enum locations?
3. **Enum Variants**: Should we standardize enum variant naming (e.g., PascalCase vs SCREAMING_SNAKE_CASE)?
4. **Future Extensions**: Should we include non-enum security types in this module?

## Related Tasks

Tasks will be created in [`tasks.md`](./tasks.md) and include:

1. **Analysis and Planning**
   - Comprehensive audit of all security enums
   - Conflict resolution for duplicate definitions
   - Module structure design

2. **Implementation**
   - Create shared security types module
   - Refactor individual modules to use shared types
   - Update imports and remove duplicates

3. **Verification and Documentation**
   - Comprehensive testing of refactored code
   - Update technical documentation
   - Verify no breaking changes

See [Tasks for PBI 25](./tasks.md) for detailed implementation breakdown.