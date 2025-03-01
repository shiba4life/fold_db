# System Patterns

## Core Architecture
- Modular design with clear separation of concerns:
  - Schema management and interpretation
  - Schema transformation and mapping
  - Request/response handling
  - Error management
  - Concurrent operation support
  - Version control management
  - Field-level mapping and validation
  - Schema relationship tracking

## Key Components

1. SchemaCore
   - Unified schema management and interpretation
   - Schema lifecycle management
   - Schema persistence and versioning
   - Schema definition parsing and validation
   - Field mapping coordination
   - Schema relationship tracking
   - Thread-safe operations
   - Transformation management
   - Type checking and conversion
   - Field mapping validation

2. NetworkCore
   - Unified network layer management
   - Connection coordination
   - Message routing
   - Query handling
   - Schema listing
   - Node discovery
   - Error recovery
   - Thread-safe operations

3. ConnectionManager
   - Connection lifecycle management
   - Connection health monitoring
   - Connection recovery
   - Thread-safe connection handling

4. MessageRouter
   - Message type registration
   - Handler registration
   - Message routing to appropriate handlers
   - Error handling for message processing

5. QueryService
   - Query execution (server-side)
   - Remote query execution (client-side)
   - Trust validation
   - Response formatting
   - Pending query tracking
   - Error handling for queries

6. SchemaService
   - Schema listing (server-side)
   - Remote schema listing (client-side)
   - Schema information management
   - Response formatting
   - Pending request tracking
   - Error handling for schema operations

7. DataFold Node
   - Application container management
   - Network access gatekeeper
   - API provider for applications
   - Data transformation coordinator
   - Trust and permission validator
   - Micropayment negotiator
   - Error handling and recovery
   - Schema version management

8. FoldDB
   - Main entry point
   - Manages database operations
   - Coordinates between components
   - Version history tracking
   - Atomic operation management
   - Error management coordination
   - Schema transformation orchestration
   
9. PermissionManager
   - Validates access permissions
   - Handles trust distance calculations
   - Manages explicit permissions
   - Permission check wrapper
   - Query/mutation permission validation
   - Thread-safe permission checks
   - Error handling integration

10. PaymentManager
    - Lightning Network integration
    - Payment calculation and scaling
    - Invoice generation and tracking
    - Payment verification
    - Hold invoice management
    - Thread-safe payment processing
    - Error handling integration

11. Error Handling System
    - Centralized FoldDbError type
    - Error categorization by subsystem
    - Error context management
    - Direct error propagation
    - Recovery coordination
    - Error logging and tracking
    - Component-specific error handling
    - Recovery strategy execution

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
- Template pattern for error recovery
- Decorator pattern for error context
- Mediator pattern for error management

## Data Flow
1. Request is authenticated and validated
2. Operation type is determined (Query/Mutation/Schema)
3. Permissions are checked
4. Payment requirements calculated if applicable
5. Schema transformations applied if needed
6. Operation is executed
7. Response is formatted and sent
8. Error handling if needed
9. Recovery attempted if error occurs
10. Error context preserved and logged

## Schema Management Flow
1. Schema is loaded from JSON definition
2. Fields and relationships are validated
3. Field mappings are processed
4. Reference UUIDs are tracked
5. Transformation rules are validated
6. Schema is versioned and persisted
7. Schema relationships are tracked
8. Field mappings are validated
9. Schema is made available for operations
10. Version control is updated

## Schema Transformation Flow
1. Source and target schemas are loaded
2. Field mappings are validated
3. Transformation rules are applied
4. References are updated
5. Data is transformed
6. Changes are validated
7. New version is created if needed
8. Relationships are updated
9. Field mappings are persisted
10. Version control is updated

## Error Handling Flow
1. Error is caught at appropriate level
2. Error is converted to FoldDbError with context
3. Error is categorized by subsystem (Schema, Network, Database, etc.)
4. Recovery is attempted if possible
5. Response is formatted with error details
6. Client is notified
7. Resources are cleaned up
8. Error is logged if necessary
9. Recovery status is tracked
10. Error context is preserved

## Field Mapping Flow
1. Source and target fields identified
2. Mapping rules validated
3. Transformation rules checked
4. References validated
5. Mapping applied
6. Results validated
7. Changes persisted
8. Version control updated
9. Relationships tracked
10. Error handling performed

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
10. Field mapping tests
11. Relationship tracking tests
12. Error recovery tests

## Error Recovery Strategy
1. Error context analysis
2. Recovery action determination
3. Resource state verification
4. Backup data consultation
5. Recovery action execution
6. State validation
7. Client notification
8. Resource cleanup
9. Log update
10. System state verification
