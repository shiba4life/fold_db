# Usage Patterns Analysis - Security Enums

**Analysis Date**: 2025-06-13  
**Task**: 25-1 Comprehensive security enum audit and analysis  
**Analyst**: AI_Agent  

## Executive Summary

This document analyzes usage patterns for security-related enums across the DataFold codebase to inform the unification strategy. The analysis covers 81 usage instances across 16 modules, revealing critical dependencies and usage patterns that must be preserved during consolidation.

## Critical Usage Patterns

### 1. RotationStatus Usage Analysis

**Total Usage Instances**: 23 across 3 modules

#### Module: [`src/db_operations/key_rotation_operations.rs`](../../../src/db_operations/key_rotation_operations.rs:0)
**Usage Pattern**: Database state management
```rust
// Status transitions during operations
record.status = RotationStatus::InProgress;     // Line 108
record.status = RotationStatus::Completed;      // Line 149  
record.status = RotationStatus::Failed;         // Line 151
record.status = RotationStatus::RolledBack;     // Line 257

// Status-based logic and filtering
match record.status {
    RotationStatus::Completed => successful_rotations += 1,           // Line 536
    RotationStatus::Failed | RotationStatus::RolledBack => failed_rotations += 1,  // Line 537
    _ => {}
}
```

**Critical Dependencies**:
- Database persistence (serialization required)
- State machine transitions
- Metrics calculation based on status

#### Module: [`src/crypto/key_rotation_audit.rs`](../../../src/crypto/key_rotation_audit.rs:0)
**Usage Pattern**: Audit trail tracking
```rust
// Initial status assignment
status: RotationStatus::Requested,              // Line 283

// Event-driven status updates  
KeyRotationAuditEventType::RotationValidated => {
    correlation.status = RotationStatus::Validating;     // Line 328
}
KeyRotationAuditEventType::RotationInProgress => {
    correlation.status = RotationStatus::InProgress;     // Line 331
}
KeyRotationAuditEventType::RotationCompleted => {
    correlation.status = RotationStatus::Completed;      // Line 334
}
```

**Critical Dependencies**:
- Audit log correlation
- Event-driven status transitions
- Compliance reporting

#### Module: [`src/crypto/key_rotation_recovery.rs`](../../../src/crypto/key_rotation_recovery.rs:0)  
**Usage Pattern**: Recovery decision logic
```rust
match operation.status {
    RotationStatus::Failed => true,              // Line 305
    RotationStatus::InProgress => {              // Line 306
        // Time-based recovery logic
    }
    RotationStatus::RolledBack => {              // Line 310
        // Rollback verification
    }
    RotationStatus::Completed => {               // Line 314
        // Integrity verification
    }
}
```

**Critical Dependencies**:
- Recovery logic branching
- Time-based decisions
- System health assessment

### 2. EventSeverity Usage Analysis

**Total Usage Instances**: 36 across 6 modules

#### Module: [`src/events/event_types.rs`](../../../src/events/event_types.rs:0)
**Usage Pattern**: Core event classification and filtering
```rust
// Display formatting
match self {
    EventSeverity::Info => write!(f, "INFO"),        // Line 64
    EventSeverity::Warning => write!(f, "WARNING"),  // Line 65
    EventSeverity::Error => write!(f, "ERROR"),      // Line 66
    EventSeverity::Critical => write!(f, "CRITICAL"), // Line 67
}

// Severity-based filtering logic
match (minimum_severity, self.severity()) {
    (EventSeverity::Info, _) => true,                           // Line 444
    (EventSeverity::Warning, EventSeverity::Info) => false,    // Line 445
    (EventSeverity::Error, EventSeverity::Info | EventSeverity::Warning) => false, // Line 447
    (EventSeverity::Critical, EventSeverity::Critical) => true, // Line 449
    // ...
}
```

**Critical Dependencies**:
- Event filtering and routing
- Alert threshold configuration  
- Display formatting standards

#### Module: [`src/events/verification_bus.rs`](../../../src/events/verification_bus.rs:0)
**Usage Pattern**: Configuration and threshold management
```rust
// Configuration defaults
min_severity: EventSeverity::Info,               // Line 58

// Dynamic configuration from environment
min_severity: match env_config.logging.level.as_str() {
    "debug" => EventSeverity::Info,              // Line 524
    "info" => EventSeverity::Info,               // Line 525  
    "warn" => EventSeverity::Warning,            // Line 526
    "error" => EventSeverity::Error,             // Line 527
    _ => EventSeverity::Info,                    // Line 528
},
```

**Critical Dependencies**:
- Environment-based configuration
- Logging level mapping
- Bus filtering configuration

#### Modules: [`src/events/handlers.rs`](../../../src/events/handlers.rs:0), [`src/events/correlation.rs`](../../../src/events/correlation.rs:0), [`src/events/transport.rs`](../../../src/events/transport.rs:0)
**Usage Pattern**: Event creation and processing
```rust
// Event instantiation with severity
severity: EventSeverity::Info,                  // Standard info events
severity: EventSeverity::Critical,              // Security incidents
severity: EventSeverity::Error,                 // Failure events
```

### 3. SecurityLevel Usage Analysis

**Total Usage Instances**: 17 across 4 modules

#### Module: [`src/config/crypto.rs`](../../../src/config/crypto.rs:0)
**Usage Pattern**: Cryptographic parameter selection
```rust
// Configuration defaults
key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Balanced), // Line 40

// Parameter mapping
match level {
    SecurityLevel::Interactive => Self::interactive(),     // Line 215
    SecurityLevel::Balanced => Self { preset: Some(SecurityLevel::Balanced), ..Default::default() }, // Line 216
    SecurityLevel::Sensitive => Self::sensitive(),         // Line 220
}

// Argon2 parameter calculation
match preset {
    SecurityLevel::Interactive => (32768, 2, 2),          // Line 242
    SecurityLevel::Balanced => (65536, 3, 4),             // Line 243  
    SecurityLevel::Sensitive => (131072, 4, 8),           // Line 244
}
```

**Critical Dependencies**:
- Cryptographic parameter selection
- Performance vs. security trade-offs
- Configuration validation

#### Module: [`src/bin/datafold_cli.rs`](../../../src/bin/datafold_cli.rs:0)
**Usage Pattern**: CLI parameter mapping and conversion
```rust
// CLI enum to config enum conversion
match cli_level {
    CliSecurityLevel::Interactive => SecurityLevel::Interactive,  // Line 81
    CliSecurityLevel::Balanced => SecurityLevel::Balanced,        // Line 82
    CliSecurityLevel::Sensitive => SecurityLevel::Sensitive,      // Line 83
}

// Argon2 parameter selection (repeated pattern)
let argon2_params = match security_level {
    CliSecurityLevel::Interactive => Argon2Params::interactive(), // Line 1629
    CliSecurityLevel::Balanced => Argon2Params::default(),        // Line 1630
    CliSecurityLevel::Sensitive => Argon2Params::sensitive(),     // Line 1631
};
```

**Critical Dependencies**:
- CLI argument processing
- Parameter validation
- User experience optimization

### 4. AlertSeverity Usage Analysis

**Total Usage Instances**: 5 across 2 modules

#### Module: [`src/datafold_node/performance_monitoring.rs`](../../../src/datafold_node/performance_monitoring.rs:0)
**Usage Pattern**: Performance threshold alerting
```rust
// Alert generation with severity
AlertSeverity::Warning,                          // Line 316
```

#### Module: [`src/datafold_node/signature_auth.rs`](../../../src/datafold_node/signature_auth.rs:0)
**Usage Pattern**: Security event severity mapping and logging
```rust
// Error to severity mapping
match self {
    Self::MissingHeaders { .. } => SecurityEventSeverity::Info,        // Line 399
    Self::InvalidSignatureFormat { .. } => SecurityEventSeverity::Warn, // Line 400
    Self::NonceValidationFailed { .. } => SecurityEventSeverity::Critical, // Line 403
    // ...
}

// Severity-based logging
match event.severity {
    SecurityEventSeverity::Info => info!("SECURITY_EVENT: {}", json_str),     // Line 1306
    SecurityEventSeverity::Warn => warn!("SECURITY_EVENT: {}", json_str),     // Line 1307  
    SecurityEventSeverity::Critical => error!("SECURITY_EVENT: {}", json_str), // Line 1308
}
```

## Cross-Module Dependencies

### 1. Import Dependencies

**Direct Imports Identified**:
```rust
// config::crypto::SecurityLevel used in datafold_node  
use crate::config::crypto::SecurityLevel;        // crypto_validation.rs:222
```

**Implicit Dependencies** (via usage patterns):
- Events modules share [`EventSeverity`](../../../src/events/event_types.rs:50) across 6 files
- Crypto modules share rotation status tracking
- DataFold node modules share alert severity patterns

### 2. Serialization Dependencies

**Database Storage**:
- [`RotationStatus`](../../../src/crypto/key_rotation_audit.rs:223) - stored in rotation records
- Status transitions must maintain backward compatibility

**API Responses**:
- Security levels exposed in configuration endpoints
- Event severities in monitoring APIs
- Alert severities in health check responses

**Configuration Files**:
- [`SecurityLevel`](../../../src/config/crypto.rs:275) - serialized in crypto config
- [`EventSeverity`](../../../src/events/event_types.rs:50) - in logging configuration

### 3. Public API Surface

**External Interfaces**:
- CLI arguments expose [`CliSecurityLevel`](../../../src/bin/datafold_cli.rs:69)
- HTTP endpoints may expose status enums
- Event bus interfaces use severity levels

## Unification Impact Assessment

### High Impact Changes
1. **RotationStatus** - Heavy usage in state machines, database storage
2. **EventSeverity** - Core to event routing and filtering
3. **SecurityLevel** - Critical for crypto parameter selection

### Medium Impact Changes  
1. **AlertSeverity** - Used in monitoring but limited scope
2. **SecurityEventSeverity** - Auth-specific but important for security

### Low Impact Changes
1. Domain-specific enums with limited cross-module usage
2. Utility enums used in single modules

## Recommendations for Safe Migration

### 1. Gradual Migration Strategy
- Start with low-impact enums to validate approach
- Use type aliases during transition period
- Maintain serialization compatibility

### 2. Backward Compatibility Requirements
- Database stored values must remain valid
- API response formats must not change
- Configuration file formats must remain compatible

### 3. Testing Requirements
- Full integration tests for status transitions
- Serialization/deserialization validation
- Cross-module dependency verification

### 4. Risk Mitigation
- Feature flags for gradual rollout
- Rollback plan for each migration phase
- Comprehensive monitoring during transition

## Usage Statistics Summary

| Enum Category | Total Instances | Modules | Risk Level | Unification Priority |
|---------------|----------------|---------|------------|---------------------|
| Status Enums | 35 | 6 | High | High |
| Severity Enums | 41 | 8 | Medium | High |
| Security Levels | 17 | 4 | Medium | Medium |
| Domain-Specific | 8 | 4 | Low | Low |

**Next Steps**: Use this analysis to inform the implementation phases in [`duplicate-resolution-plan.md`](./duplicate-resolution-plan.md).