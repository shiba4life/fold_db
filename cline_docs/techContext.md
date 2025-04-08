# Technical Context

## Technologies Used
- Rust programming language
- sled embedded database for persistent storage
- serde for serialization/deserialization
- JSON for data and schema representation
- tokio for async runtime
- Docker for application containerization
- Bitcoin Lightning Network for payments

## Development Setup
- Rust toolchain required
- Docker runtime for application containerization
- No external database dependencies (uses embedded sled)
- Lightning Network node for payment processing
- File-based storage system

## Technical Constraints

### Schema System Constraints
- JSON-based schema definitions
- Field mapping relationships must be explicit
- No circular field mappings allowed
- Schema names must be unique
- All schema files stored in dedicated directory
- Thread-safe schema operations required
- Persistent schema storage with version control
- Schema transformations must preserve data integrity
- Field relationships must be explicitly tracked
- Type conversions must be explicitly defined
- Schema validation must occur before transformation
- Error recovery mechanisms required for transformations

### Core System Constraints
- Immutable data model (all changes create new versions)
- Trust distance must be a positive integer (lower = higher trust)
- Permissions are enforced at field level
- All operations require public key for authentication
- Schema must be loaded before data operations
- Write operations require explicit permissions
- Read operations can use either trust distance or explicit permissions
- Thread-safe concurrent operations
- Version history must be maintained
- Atomic operations for all data changes

### Payment Requirements
- All base multipliers must be positive
- Trust distance scaling factors must be >= 1.0
- Payment thresholds must be non-negative
- Lightning Network connection required for paid operations
- Hold invoices for complex operations
- Payment verification before operation execution
- Thread-safe payment processing
- Payment state must be tracked

## Components

1. Schema System
   - JSON schema definitions
   - Field-level configurations
   - Schema mapping/transformation
   - Schema persistence with versioning
   - Field relationship tracking
   - Thread-safe operations
   - Error recovery mechanisms
   - Validation rules
   - Type conversion handling

2. Payment System
   - Lightning Network integration
   - Per-field payment calculation
   - Trust distance scaling
   - Payment verification
   - Hold invoice support
   - Thread-safe payment processing
   - Payment state management
   - Error handling

3. Permission System
   - Trust-based access control
   - Field-level permissions
   - Explicit policy management
   - Permission wrapper implementation
   - Thread-safe permission checks
   - Policy validation
   - Error handling

## Performance Considerations
- Thread-safe concurrent processing
- Resource cleanup
- Error recovery
- Schema caching
- Payment state management
- Version control overhead
- Transformation overhead
- Validation performance
- Memory usage optimization

## Security Considerations
- Public key authentication
- Trust distance validation
- Permission enforcement
- Payment verification
- Error handling
- Resource cleanup
- Data validation
- Schema validation
- Transformation validation
- Version control integrity
- Concurrent access control
