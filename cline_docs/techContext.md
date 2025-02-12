# Technical Context

## Technologies Used
- Rust programming language
- sled embedded database for persistent storage
- serde for serialization/deserialization
- JSON for data representation
- Bitcoin Lightning Network for payments
- tokio for async runtime
- Docker for application containerization

## Development Setup
- Rust toolchain required
- No external database dependencies (uses embedded sled)
- Lightning Network node for payment processing
- File-based storage system
- Docker runtime for application containers

## Technical Constraints

### Application Container Constraints
- All application code must run in sandboxed containers
- Containers must be stateless beyond runtime
- Network access only through Node API
- Data modifications are append-only
- Container lifecycle managed by DataFold Node

### Core System Constraints
- Immutable data model (all changes create new versions)
- Trust distance must be a positive integer (lower = higher trust)
- Permissions are enforced at field level
- All operations require public key for authentication
- Schema must be loaded before data operations
- Write operations require explicit permissions
- Read operations can use either trust distance or explicit permissions
- Payment requirements:
  - All base multipliers must be positive
  - Trust distance scaling factors must be >= 1.0
  - Payment thresholds must be non-negative
  - Lightning Network connection required for paid operations

## Components
1. Core Database
   - Atomic operations
   - Version tracking
   - Schema validation
   - Permission checks

2. Schema System
   - JSON schema definitions
   - Field-level configurations
   - Schema mapping/transformation
   - Schema interpreter for validation

3. Payment System
   - Lightning Network integration
   - Per-field payment calculation
   - Trust distance scaling
   - Payment verification
   - Hold invoice support

4. Permission System
   - Trust-based access control
   - Field-level permissions
   - Explicit policy management
   - Permission wrapper implementation
