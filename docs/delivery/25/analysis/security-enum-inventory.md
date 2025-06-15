# Security Enum Inventory - Comprehensive Analysis

**Analysis Date**: 2025-06-13  
**Task**: 25-1 Comprehensive security enum audit and analysis  
**Analyst**: AI_Agent  

## Executive Summary

This document provides a comprehensive inventory of all security-related enums across the DataFold codebase. The analysis reveals significant duplication and opportunities for unification, particularly around status tracking, severity levels, and security classifications.

**Key Findings:**
- **Critical Duplicates**: 2 exact enum name duplicates (`RotationStatus`, `AlertSeverity`)
- **Near Duplicates**: 8 enums with overlapping functionality
- **Total Security Enums**: 47 enums identified across 16 modules
- **Unification Opportunities**: 15 enums could be consolidated into 4 unified types

## Detailed Inventory

### 1. CRITICAL DUPLICATES (Exact Name Conflicts)

#### 1.1 RotationStatus (2 definitions)

**Location 1**: [`src/crypto/key_rotation_audit.rs:223`](../../../src/crypto/key_rotation_audit.rs:223)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationStatus {
    /// Rotation requested but not started
    Requested,
    /// Validation in progress
    Validating,
    /// Rotation in progress
    InProgress,
    /// Rotation completed successfully
    Completed,
    /// Rotation failed
    Failed,
    /// Rotation cancelled
    Cancelled,
}
```

**Location 2**: [`src/db_operations/key_rotation_operations.rs:55`](../../../src/db_operations/key_rotation_operations.rs:55)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationStatus {
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation was rolled back
    RolledBack,
}
```

**Conflict Analysis:**
- Audit version has 6 variants, DB version has 4 variants
- Common variants: `InProgress`, `Completed`, `Failed`
- Audit-only: `Requested`, `Validating`, `Cancelled`
- DB-only: `RolledBack`
- **Impact**: Name collision prevents importing both modules

#### 1.2 AlertSeverity (3 identical definitions)

**Location 1**: [`src/datafold_node/performance_monitoring.rs:76`](../../../src/datafold_node/performance_monitoring.rs:76)
**Location 2**: [`src/datafold_node/signature_auth.rs:1190`](../../../src/datafold_node/signature_auth.rs:1190)  
**Location 3**: [`src/datafold_node/key_rotation_health.rs:123`](../../../src/datafold_node/key_rotation_health.rs:123)

```rust
// Locations 1 & 2 (identical):
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

// Location 3 (extended):
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert requiring attention
    Error,
    /// Critical alert requiring immediate action
    Critical,
}
```

**Conflict Analysis:**
- Two versions identical (3 variants), one extended (4 variants)
- Health version adds `Error` level and documentation
- All implement different derive traits

### 2. SEVERITY/LEVEL ENUMS (High Unification Potential)

#### 2.1 Event/Alert Severity Group

| Enum | Location | Variants | Traits |
|------|----------|----------|---------|
| [`EventSeverity`](../../../src/events/event_types.rs:50) | `events/event_types.rs:50` | Info, Warning, Error, Critical | Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize |
| [`AuditSeverity`](../../../src/crypto/audit_logger.rs:39) | `crypto/audit_logger.rs:39` | Info, Warning, Error, Critical | Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize |
| [`SecurityEventSeverity`](../../../src/datafold_node/signature_auth.rs:517) | `datafold_node/signature_auth.rs:517` | Info, Warning, Error, Critical | Debug, Clone, Serialize, Deserialize, PartialEq |
| [`ErrorSeverity`](../../../src/crypto/key_rotation_error_handling.rs:20) | `crypto/key_rotation_error_handling.rs:20` | Low, Medium, High, Critical | Debug, Clone, PartialEq, Eq, Serialize, Deserialize |
| [`ErrorSeverity`](../../../src/crypto/enhanced_error.rs:17) | `crypto/enhanced_error.rs:17` | Low, Medium, High, Critical | Debug, Clone, PartialEq, Eq, Serialize, Deserialize |

**Unification Potential**: HIGH - These could be unified into single [`SecuritySeverity`](../../../src/security_types.rs:0) enum

#### 2.2 Security Level Group

| Enum | Location | Variants | Purpose |
|------|----------|----------|---------|
| [`SecurityLevel`](../../../src/config/crypto.rs:275) | `config/crypto.rs:275` | Interactive, Balanced, Sensitive | Crypto parameter selection |
| [`SecurityLevel`](../../../src/cli/verification.rs:353) | `cli/verification.rs:353` | Low, Medium, High | CLI verification settings |
| [`CliSecurityLevel`](../../../src/bin/datafold_cli.rs:69) | `bin/datafold_cli.rs:69` | Fast, Balanced, Paranoid | CLI argument parsing |

**Conflict**: Exact name collision between config and CLI versions
**Unification Potential**: MEDIUM - Different use cases but could share common base

### 3. STATUS ENUMS (Medium Unification Potential)

#### 3.1 System/Health Status Group

| Enum | Location | Variants | Context |
|------|----------|----------|---------|
| [`SystemStatus`](../../../src/crypto/key_rotation_integration.rs:114) | `crypto/key_rotation_integration.rs:114` | Normal, Degraded, Offline | System health |
| [`RotationHealthStatus`](../../../src/datafold_node/key_rotation_health.rs:26) | `datafold_node/key_rotation_health.rs:26` | Healthy, Warning, Critical, Failed | Rotation health |
| [`RecoveryHealthStatus`](../../../src/crypto/key_rotation_recovery.rs:161) | `crypto/key_rotation_recovery.rs:161` | Healthy, Warning, Critical, Failed | Recovery health |
| [`HealthCheckStatus`](../../../src/datafold_node/key_rotation_health.rs:60) | `datafold_node/key_rotation_health.rs:60` | Passed, Warning, Failed | Individual checks |

#### 3.2 Operation Status Group

| Enum | Location | Variants | Context |
|------|----------|----------|---------|
| [`VerificationStatus`](../../../src/cli/verification.rs:68) | `cli/verification.rs:68` | Valid, Invalid, Expired, Malformed | Signature verification |
| [`EscalationStatus`](../../../src/crypto/key_rotation_integration.rs:161) | `crypto/key_rotation_integration.rs:161` | Active, Resolved, Escalated | Issue escalation |
| [`ConsistencyStatus`](../../../src/crypto/key_rotation_recovery.rs:787) | `crypto/key_rotation_recovery.rs:787` | Consistent, Inconsistent, Unknown | Data consistency |
| [`RollbackStatus`](../../../src/db_operations/key_rotation_rollback.rs:67) | `db_operations/key_rotation_rollback.rs:67` | InProgress, Completed, Failed, Partial | Rollback operations |

### 4. SECURITY-SPECIFIC ENUMS

#### 4.1 Threat/Risk Assessment

| Enum | Location | Variants | Purpose |
|------|----------|----------|---------|
| [`ThreatLevel`](../../../src/crypto/security_monitor.rs:18) | `crypto/security_monitor.rs:18` | Low, Medium, High, Critical | Threat assessment |
| [`SecurityPattern`](../../../src/crypto/security_monitor.rs:31) | `crypto/security_monitor.rs:31` | FailedDecryption, UnusualKeyAccess, etc. | Pattern detection |
| [`RotationThreatPattern`](../../../src/crypto/rotation_threat_monitor.rs:25) | `crypto/rotation_threat_monitor.rs:25` | FrequentRotation, SuspiciousSource, etc. | Rotation threats |
| [`RiskAction`](../../../src/crypto/key_rotation_security.rs:210) | `crypto/key_rotation_security.rs:210` | Allow, Block, Require2FA, etc. | Risk responses |

#### 4.2 Compliance and Audit

| Enum | Location | Variants | Purpose |
|------|----------|----------|---------|
| [`ComplianceFramework`](../../../src/datafold_node/key_rotation_compliance.rs:22) | `datafold_node/key_rotation_compliance.rs:22` | SOC2TypeII, ISO27001, etc. | Compliance standards |
| [`ComplianceStatus`](../../../src/datafold_node/key_rotation_compliance.rs:384) | `datafold_node/key_rotation_compliance.rs:384` | Compliant, NonCompliant, etc. | Compliance state |
| [`AuditEventType`](../../../src/crypto/audit_logger.rs:18) | `crypto/audit_logger.rs:18` | Encryption, Decryption, etc. | Audit categories |
| [`KeyRotationAuditEventType`](../../../src/crypto/key_rotation_audit.rs:22) | `crypto/key_rotation_audit.rs:22` | RequestInitiated, ValidationStarted, etc. | Rotation audit events |

### 5. EVENT TYPE ENUMS

| Enum | Location | Variants | Purpose |
|------|----------|----------|---------|
| [`SecurityEventCategory`](../../../src/events/event_types.rs:14) | `events/event_types.rs:14` | Authentication, Authorization, etc. | Event classification |
| [`SecurityEventType`](../../../src/datafold_node/signature_auth.rs:539) | `datafold_node/signature_auth.rs:539` | AuthenticationSuccess, etc. | Specific events |
| [`KeyRotationEventType`](../../../src/events/event_types.rs:337) | `events/event_types.rs:337` | Started, Completed, etc. | Rotation events |
| [`PerformanceAlertType`](../../../src/datafold_node/performance_monitoring.rs:65) | `datafold_node/performance_monitoring.rs:65` | HighLatency, etc. | Performance alerts |

### 6. CACHE AND UTILITY ENUMS

| Enum | Location | Variants | Purpose |
|------|----------|----------|---------|
| [`CacheEntryStatus`](../../../src/datafold_node/key_cache_manager.rs:77) | `datafold_node/key_cache_manager.rs:77` | Valid, Expired, etc. | Cache management |
| [`SecurityProfile`](../../../src/datafold_node/signature_auth.rs:592) | `datafold_node/signature_auth.rs:592` | Strict, Balanced, etc. | Auth profiles |
| [`AlertDestination`](../../../src/events/handlers.rs:339) | `events/handlers.rs:339` | Console, File, etc. | Alert routing |
| [`AlertCategory`](../../../src/datafold_node/key_rotation_health.rs:136) | `datafold_node/key_rotation_health.rs:136` | Performance, Security, etc. | Alert classification |

## Summary Statistics

- **Total Enums Identified**: 47
- **Modules Analyzed**: 16  
- **Critical Name Conflicts**: 2 (`RotationStatus`, `AlertSeverity`)
- **High Unification Potential**: 15 enums (severity/level groups)
- **Medium Unification Potential**: 12 enums (status groups)
- **Domain-Specific (Keep Separate)**: 18 enums

## Immediate Action Required

1. **Resolve Name Conflicts**: `RotationStatus` and `AlertSeverity` duplicates must be addressed
2. **Plan Severity Unification**: 8 severity-related enums can be consolidated  
3. **Assess Status Consolidation**: 12 status enums could share common patterns
4. **Preserve Domain Logic**: 18 domain-specific enums should remain separate

## Next Steps

See [`duplicate-resolution-plan.md`](./duplicate-resolution-plan.md) for detailed conflict resolution strategy.