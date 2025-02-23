# System Patterns

## Core Architecture
- Modular design with clear separation of concerns:
  - Schema management and interpretation
  - Schema transformation and mapping
  - Request/response handling
  - Error management
  - Concurrent operation support
  - Version control management

## Key Components

1. SchemaManager
   - Schema lifecycle management
   - Schema persistence and versioning
   - Field mapping coordination
   - Schema validation
   - Schema relationship tracking
   - Thread-safe operations
   - Transformation management
   - Error recovery handling

2. SchemaInterpreter
   - Schema definition parsing
   - Field validation
   - Transformation rules processing
   - Relationship validation
   - Error context preservation
   - Type checking and conversion

3. DataFold Node
   - Application container management
   - Network access gatekeeper
   - API provider for applications
   - Data transformation coordinator
   - Trust and permission validator
   - Micropayment negotiator
   - Error handling and recovery

4. FoldDB
   - Main entry point
   - Manages database operations
   - Coordinates between components
   - Version history tracking
   - Atomic operation management
   
5. PermissionManager
   - Validates access permissions
   - Handles trust distance calculations
   - Manages explicit permissions
   - Permission check wrapper
   - Query/mutation permission validation
   - Thread-safe permission checks

6. PaymentManager
   - Lightning Network integration
   - Payment calculation and scaling
   - Invoice generation and tracking
   - Payment verification
   - Hold invoice management
   - Thread-safe payment processing

## Design Patterns
- Repository pattern for data access
- Strategy pattern for permission checks
- Factory pattern for Atom creation
- Builder pattern for schema construction
- Interpreter pattern for schema definitions
- Observer pattern for payment tracking
- Command pattern for database operations
- Immutable data structures
- Thread-safe concurrency patterns
- Visitor pattern for schema transformations
- Chain of Responsibility for error handling

## Data Flow
1. Request is authenticated and validated
2. Operation type is determined (Query/Mutation/Schema)
3. Permissions are checked
4. Payment requirements calculated if applicable
5. Schema transformations applied if needed
6. Operation is executed
7. Response is formatted and sent
8. Error handling if needed

## Schema Management Flow
1. Schema is loaded from JSON definition
2. Fields and relationships are validated
3. Field mappings are processed
4. Reference UUIDs are tracked
5. Transformation rules are validated
6. Schema is versioned and persisted
7. Schema is made available for operations

## Schema Transformation Flow
1. Source and target schemas are loaded
2. Field mappings are validated
3. Transformation rules are applied
4. References are updated
5. Data is transformed
6. Changes are validated
7. New version is created if needed

## Error Handling Flow
1. Error is caught at appropriate level
2. Context is added to error
3. Error is categorized by type
4. Recovery is attempted if possible
5. Response is formatted with error details
6. Client is notified
7. Resources are cleaned up
8. Error is logged if necessary

## Testing Strategy
1. Unit tests for components
2. Integration tests for flows
3. Concurrent operation tests
4. Error handling tests
5. Performance benchmarks
6. Socket communication tests
7. Schema transformation tests
8. Version control tests
9. Recovery mechanism tests
