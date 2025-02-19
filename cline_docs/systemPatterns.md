# System Patterns

## Core Architecture
- Modular design with clear separation of concerns:
  - Schema management and interpretation
  - Schema transformation and mapping
  - Request/response handling
  - Error management
  - Concurrent operation support

## Key Components

1. SchemaManager
   - Schema lifecycle management
   - Schema persistence
   - Field mapping coordination
   - Schema validation
   - Schema relationship tracking
   - Thread-safe operations

3. DataFold Node
   - Application container management
   - Network access gatekeeper
   - API provider for applications
   - Data transformation coordinator
   - Trust and permission validator
   - Micropayment negotiator

4. FoldDB
   - Main entry point
   - Manages database operations
   - Coordinates between components
   
5. PermissionManager
   - Validates access permissions
   - Handles trust distance calculations
   - Manages explicit permissions
   - Permission check wrapper
   - Query/mutation permission validation

6. PaymentManager
   - Lightning Network integration
   - Payment calculation and scaling
   - Invoice generation and tracking
   - Payment verification
   - Hold invoice management

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

## Data Flow
1. Request is authenticated and validated
2. Operation type is determined (Query/Mutation/Schema)
3. Permissions are checked
4. Payment requirements calculated if applicable
5. Operation is executed
6. Response is formatted and sent

## Schema Management Flow
1. Schema is loaded from JSON definition
2. Fields and relationships are validated
3. Field mappings are processed
4. Reference UUIDs are tracked
5. Schema is persisted to disk
6. Schema is made available for operations

## Error Handling Flow
1. Error is caught at appropriate level
2. Context is added to error
3. Error is categorized by type
4. Response is formatted with error details
5. Client is notified
6. Resources are cleaned up
7. Error is logged if necessary

## Testing Strategy
1. Unit tests for components
2. Integration tests for flows
3. Concurrent operation tests
4. Error handling tests
5. Performance benchmarks
6. Socket communication tests
7. Schema transformation tests
