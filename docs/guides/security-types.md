# Security Types Guide

This guide explains DataFold's unified security types system, which consolidates all security-related enums into a single [`security_types.rs`](../../src/security_types.rs) module.

## Overview

As of PBI 25, all security-related enums have been unified into consistent types that eliminate duplication and ensure type safety across the entire codebase.

## Core Security Types

### SecurityLevel

Controls cryptographic parameter strength and performance trade-offs:

```rust
use datafold::security_types::SecurityLevel;

// Available levels
let level = SecurityLevel::Low;      // Fast, interactive use (formerly "Interactive")
let level = SecurityLevel::Standard; // Balanced performance/security (formerly "Balanced") 
let level = SecurityLevel::High;     // Maximum security (formerly "Sensitive")

// Get crypto parameters
let (memory, time, parallelism) = level.argon2_params();
let iterations = level.key_derivation_rounds();

// Check characteristics
if level.is_interactive() {
    println!("Suitable for interactive use");
}
if level.is_high_security() {
    println!("Provides maximum security");
}
```

**Migration Note**: The old enum variants have been renamed:
- `Interactive` → `Low` 
- `Balanced` → `Standard`
- `Sensitive` → `High`

### Severity

Universal severity classification for events, errors, and alerts:

```rust
use datafold::security_types::Severity;

// Available levels (replaces EventSeverity, ErrorSeverity, AlertSeverity, etc.)
let severity = Severity::Info;     // Normal operations
let severity = Severity::Warning;  // Potential issues  
let severity = Severity::Error;    // Failed operations
let severity = Severity::Critical; // Immediate action required

// Severity checking
if severity.requires_immediate_action() {
    // Handle critical situations
}
if severity.should_alert() {
    // Trigger alerting system
}

// Severity comparison
if severity.meets_threshold(Severity::Warning) {
    // Process events at or above warning level
}
```

### RotationStatus

Tracks key rotation operation lifecycle:

```rust
use datafold::security_types::RotationStatus;

// Status progression
let status = RotationStatus::Requested;   // Initial request
let status = RotationStatus::Validating; // Pre-rotation checks
let status = RotationStatus::InProgress; // Active rotation
let status = RotationStatus::Completed;  // Success
// Or failure states: Failed, Cancelled, RolledBack

// Status checking
if status.is_active() {
    println!("Rotation in progress");
}
if status.is_successful() {
    println!("Rotation completed successfully");
}
if status.is_terminal() {
    println!("Rotation finished (success or failure)");
}
```

### HealthStatus

System and component health monitoring:

```rust
use datafold::security_types::HealthStatus;

// Health states
let health = HealthStatus::Healthy;  // Normal operation
let health = HealthStatus::Warning;  // Minor issues
let health = HealthStatus::Critical; // Significant problems
let health = HealthStatus::Failed;   // Non-functional
let health = HealthStatus::Offline;  // Unreachable

// Health assessment
if health.is_operational() {
    // Component can handle requests
}
if health.requires_attention() {
    // Take corrective action
}

// Convert to severity for alerting
let alert_severity = health.to_severity();
```

### ThreatLevel

Security threat assessment and response:

```rust
use datafold::security_types::ThreatLevel;

// Threat levels
let threat = ThreatLevel::Low;      // Informational
let threat = ThreatLevel::Medium;   // Suspicious activity
let threat = ThreatLevel::High;     // Probable attack
let threat = ThreatLevel::Critical; // Active attack

// Threat response
if threat.requires_immediate_action() {
    // High or Critical threats need immediate response
}
if threat.is_active_threat() {
    // Critical threats indicate active attacks
}

// Convert to severity for logging
let log_severity = threat.to_severity();
```

## Migration from Old Enums

### Replaced Types

The following old enums have been replaced by unified types:

| Old Enum | Replaced By | Notes |
|----------|-------------|-------|
| `EventSeverity` | `Severity` | Events module |
| `AlertSeverity` | `Severity` | Monitoring and alerts |
| `AuditSeverity` | `Severity` | Audit logging |
| `ErrorSeverity` | `Severity` | Error handling |
| `SecurityEventSeverity` | `Severity` | Authentication events |
| `RotationStatus` (duplicates) | `RotationStatus` | Single unified version |
| `SecurityLevel` (config) | `SecurityLevel` | Crypto configuration |
| `RotationHealthStatus` | `HealthStatus` | Health monitoring |
| `RecoveryHealthStatus` | `HealthStatus` | Recovery operations |
| `SystemStatus` | `HealthStatus` | System monitoring |

### Code Migration Examples

**Old Code (EventSeverity):**
```rust
// OLD - Don't use
use crate::events::event_types::EventSeverity;
let severity = EventSeverity::Critical;
```

**New Code (Severity):**
```rust
// NEW - Use this
use datafold::security_types::Severity;
let severity = Severity::Critical;
```

**Old Code (SecurityLevel variants):**
```rust
// OLD - Don't use  
SecurityLevel::Interactive  // Fast
SecurityLevel::Balanced     // Default
SecurityLevel::Sensitive    // High security
```

**New Code (SecurityLevel variants):**
```rust
// NEW - Use this
SecurityLevel::Low       // Fast (formerly Interactive)
SecurityLevel::Standard  // Default (formerly Balanced) 
SecurityLevel::High      // High security (formerly Sensitive)
```

## Best Practices

### 1. Import from Unified Module
Always import security types from the unified module:

```rust
use datafold::security_types::{SecurityLevel, Severity, HealthStatus, RotationStatus, ThreatLevel};
```

### 2. Use Type Methods
Leverage the built-in methods for common operations:

```rust
// Check severity thresholds
if event_severity.meets_threshold(Severity::Warning) {
    // Handle warning+ events
}

// Convert between types
let alert_severity = health_status.to_severity();
let log_severity = threat_level.to_severity();
```

### 3. Consistent Error Handling
Use `Severity` for all error classification:

```rust
let error_severity = match error_type {
    ErrorType::Network => Severity::Warning,
    ErrorType::Database => Severity::Error,
    ErrorType::Security => Severity::Critical,
    _ => Severity::Info,
};
```

### 4. Health Monitoring
Use `HealthStatus` for all component health tracking:

```rust
let component_health = if can_process_requests() {
    HealthStatus::Healthy
} else if has_minor_issues() {
    HealthStatus::Warning  
} else {
    HealthStatus::Critical
};
```

## API Compatibility

All unified types implement:
- `Serialize` and `Deserialize` for JSON/configuration compatibility
- `Display` for human-readable output
- `PartialOrd` and `Ord` for severity/threat level comparisons
- `Hash` and `Eq` for use in collections

## Configuration Examples

### Security Level in Configuration
```json
{
  "crypto": {
    "security_level": "Standard",
    "key_derivation": {
      "security_level": "High"  
    }
  }
}
```

### Event Filtering by Severity
```json
{
  "logging": {
    "min_severity": "Warning",
    "alert_threshold": "Error"
  }
}
```

## See Also

- [Security Types Module](../../src/security_types.rs) - Implementation details
- [CLI Authentication Guide](./cli-authentication.md) - Security profiles
- [Key Rotation Guide](./key-rotation.md) - Rotation status tracking
- [PBI 25 Documentation](../delivery/25/prd.md) - Unification project details