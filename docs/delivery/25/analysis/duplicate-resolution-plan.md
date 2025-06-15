# Duplicate Resolution Plan - Security Enums

**Analysis Date**: 2025-06-13  
**Task**: 25-1 Comprehensive security enum audit and analysis  
**Analyst**: AI_Agent  

## Executive Summary

This document outlines the resolution strategy for the critical enum duplicates and near-duplicates identified in the security enum inventory. The plan prioritizes backward compatibility while establishing a unified security types module.

## Critical Conflicts Resolution

### 1. RotationStatus Conflict Resolution

**Problem**: Two different [`RotationStatus`](../../../src/crypto/key_rotation_audit.rs:223) enums exist with overlapping but incompatible definitions.

**Current Definitions:**
- **Audit Version** (6 variants): `Requested`, `Validating`, `InProgress`, `Completed`, `Failed`, `Cancelled`
- **DB Version** (4 variants): `InProgress`, `Completed`, `Failed`, `RolledBack`

**Resolution Strategy**: CREATE UNIFIED ENUM with superset of variants

**Proposed Unified Definition**:
```rust
// Location: src/security_types.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationStatus {
    /// Rotation requested but not yet started
    Requested,
    /// Pre-rotation validation in progress
    Validating,
    /// Rotation operation in progress
    InProgress,
    /// Rotation completed successfully
    Completed,
    /// Rotation failed with error
    Failed,
    /// Rotation cancelled by user/system
    Cancelled,
    /// Rotation was rolled back due to issues
    RolledBack,
}
```

**Migration Plan**:
1. **Phase 1**: Create unified enum in [`security_types.rs`](../../../src/security_types.rs:0)
2. **Phase 2**: Update audit module to use unified enum with full variant support
3. **Phase 3**: Update DB module to use unified enum, mapping `RolledBack` as needed
4. **Phase 4**: Remove duplicate definitions

**Backward Compatibility**: 
- All existing variants preserved
- Serialization compatibility maintained via same variant names
- New `RolledBack` and audit-specific variants available to all modules

### 2. AlertSeverity Conflict Resolution

**Problem**: Three [`AlertSeverity`](../../../src/datafold_node/key_rotation_health.rs:123) definitions with slight variations.

**Current Definitions:**
- **Performance/Auth** (3 variants): `Info`, `Warning`, `Critical`
- **Health** (4 variants): `Info`, `Warning`, `Error`, `Critical`

**Resolution Strategy**: UNIFY INTO COMPREHENSIVE SEVERITY ENUM

**Proposed Unified Definition**:
```rust
// Location: src/security_types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert for normal operations
    Info,
    /// Warning alert for potential issues
    Warning,
    /// Error alert requiring attention
    Error,
    /// Critical alert requiring immediate action
    Critical,
}
```

**Migration Plan**:
1. **Phase 1**: Create unified enum with 4-variant version (includes `Error`)
2. **Phase 2**: Update all modules to use unified enum
3. **Phase 3**: Map missing `Error` variant in modules that only used 3 variants
4. **Phase 4**: Remove duplicate definitions

**Backward Compatibility**: All existing variants preserved, `Error` added for completeness

## Near-Duplicate Consolidation

### 3. Severity/Level Enum Unification

**Target Enums for Consolidation**:
- [`EventSeverity`](../../../src/events/event_types.rs:50) (events)
- [`AuditSeverity`](../../../src/crypto/audit_logger.rs:39) (crypto)  
- [`SecurityEventSeverity`](../../../src/datafold_node/signature_auth.rs:517) (auth)
- [`ErrorSeverity`](../../../src/crypto/key_rotation_error_handling.rs:20) (error handling - Low/Medium/High/Critical)
- [`ErrorSeverity`](../../../src/crypto/enhanced_error.rs:17) (enhanced error - Low/Medium/High/Critical)

**Unified Approach**: Create THREE specialized severity enums:

#### 3.1 AlertSeverity (Already defined above)
For alerts, notifications, and monitoring

#### 3.2 EventSeverity  
```rust
// Location: src/security_types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Informational events for normal operations
    Info,
    /// Warning events for potential issues  
    Warning,
    /// Error events for failed operations
    Error,
    /// Critical events requiring immediate attention
    Critical,
}
```

#### 3.3 ErrorSeverity
```rust
// Location: src/security_types.rs  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - operation can continue with degraded functionality
    Low,
    /// Medium severity - operation should be retried with caution
    Medium,
    /// High severity - operation should be aborted and escalated
    High,
    /// Critical severity - immediate intervention required
    Critical,
}
```

### 4. SecurityLevel Conflict Resolution

**Problem**: Two different [`SecurityLevel`](../../../src/config/crypto.rs:275) enums with different purposes:
- **Config**: `Interactive`, `Balanced`, `Sensitive` (crypto parameters)
- **CLI**: `Low`, `Medium`, `High` (verification settings)

**Resolution Strategy**: RENAME FOR CLARITY

**Proposed Resolution**:
```rust
// Location: src/security_types.rs

// For crypto configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoSecurityLevel {
    /// Fast parameters for interactive use
    Interactive,
    /// Balanced parameters for general use  
    Balanced,
    /// High security parameters for sensitive operations
    Sensitive,
}

// For verification/validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationSecurityLevel {
    /// Basic verification requirements
    Low,
    /// Standard verification requirements
    Medium,
    /// Strict verification requirements  
    High,
}
```

## Status Enum Consolidation Strategy

### 5. Health Status Unification

**Target Enums**:
- [`RotationHealthStatus`](../../../src/datafold_node/key_rotation_health.rs:26)
- [`RecoveryHealthStatus`](../../../src/crypto/key_rotation_recovery.rs:161)
- [`SystemStatus`](../../../src/crypto/key_rotation_integration.rs:114)

**Unified Definition**:
```rust
// Location: src/security_types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System/component is operating normally
    Healthy,
    /// System/component has minor issues but is functional
    Warning,
    /// System/component has significant issues affecting operation
    Critical,
    /// System/component has failed and is non-functional
    Failed,
    /// System/component is offline or unreachable
    Offline,
}
```

### 6. Operation Status Patterns

**Pattern Identification**: Many status enums follow common patterns:
- **Basic**: `InProgress`, `Completed`, `Failed`
- **Extended**: Add `Cancelled`, `Pending`, `Retrying`
- **Verification**: `Valid`, `Invalid`, `Expired`, `Malformed`

**Approach**: Create base traits rather than single enum:

```rust
// Location: src/security_types.rs
pub trait OperationStatus {
    fn is_complete(&self) -> bool;
    fn is_failed(&self) -> bool;
    fn is_active(&self) -> bool;
}

// Keep specific enums but ensure they implement common interface
```

## Implementation Priority

### Phase 1: Critical Conflicts (Week 1)
1. **High Priority**: [`RotationStatus`](../../../src/crypto/key_rotation_audit.rs:223) unification
2. **High Priority**: [`AlertSeverity`](../../../src/datafold_node/key_rotation_health.rs:123) unification
3. **Medium Priority**: [`SecurityLevel`](../../../src/config/crypto.rs:275) rename/split

### Phase 2: Severity Consolidation (Week 2)  
1. Create unified severity enums
2. Update imports across modules
3. Remove duplicate definitions

### Phase 3: Status Unification (Week 3)
1. [`HealthStatus`](../../../src/security_types.rs:0) consolidation
2. Operation status trait implementation
3. Verification status standardization

### Phase 4: Cleanup (Week 4)
1. Remove all duplicate definitions
2. Update documentation
3. Verify compilation and tests

## Risk Assessment

### High Risk Items
- **RotationStatus**: Used in serialization, API compatibility critical
- **AlertSeverity**: Multiple modules, potential breaking changes

### Mitigation Strategies
1. **Gradual Migration**: Phase implementation to minimize disruption
2. **Alias Support**: Temporary type aliases during transition
3. **Version Compatibility**: Maintain serialization compatibility
4. **Comprehensive Testing**: Full test suite validation after each phase

### Breaking Change Analysis
- **External APIs**: Need to verify no public APIs expose conflicting enums
- **Serialization**: JSON/serde compatibility must be maintained  
- **Database**: Ensure stored enum values remain valid

## Validation Criteria

### Phase Completion Criteria
- [ ] No compilation errors
- [ ] All tests pass
- [ ] No serialization format changes
- [ ] Documentation updated
- [ ] Import statements verified

### Final Success Criteria  
- [ ] Zero duplicate enum names
- [ ] Consistent severity/level patterns
- [ ] Reduced total enum count by ≥30%
- [ ] All modules compile with new unified types
- [ ] Backward compatibility maintained

## Next Steps

1. Review this plan with stakeholders
2. Create [`security_types.rs`](../../../src/security_types.rs:0) module
3. Begin Phase 1 implementation
4. Monitor for any additional conflicts during implementation

See [`usage-patterns.md`](./usage-patterns.md) for detailed usage analysis supporting these decisions.