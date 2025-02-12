# System Patterns

## Core Architecture
- Modular design with clear separation of concerns:
  - Schema management and interpretation
  - Permissions management
  - Payment processing
  - Atom/AtomRef storage
  - Database operations
  - Sandboxed application containers

## Application Layer
- Containerized application runtime (Docker)
- Node-mediated network access
- API-based data interaction
- Stateless container design
- Append-only data modifications

## Key Components

1. DataFold Node
   - Application container management
   - Network access gatekeeper
   - API provider for applications
   - Data transformation coordinator
   - Trust and permission validator
   - Micropayment negotiator

2. FoldDB
   - Main entry point
   - Manages database operations
   - Coordinates between components
   
2. SchemaManager & Interpreter
   - Handles schema validation
   - Manages schema loading
   - Tracks available schemas
   - Interprets JSON schema definitions
   - Validates schema constraints
   - Manages schema transformations

3. PermissionManager
   - Validates access permissions
   - Handles trust distance calculations
   - Manages explicit permissions
   - Permission check wrapper
   - Query/mutation permission validation

4. PaymentManager
   - Lightning Network integration
   - Payment calculation and scaling
   - Invoice generation and tracking
   - Payment verification
   - Hold invoice management

5. Atoms & AtomRefs
   - Atoms: Immutable data containers
   - AtomRefs: Pointers to latest versions
   - Version history tracking

## Design Patterns
- Repository pattern for data access
- Strategy pattern for permission checks
- Factory pattern for Atom creation
- Builder pattern for schema construction
- Interpreter pattern for schema definitions
- Observer pattern for payment tracking
- Command pattern for database operations
- Immutable data structures

## Data Flow
1. Operations start with schema validation
2. Permissions are checked before access
3. Payment requirements calculated if applicable
4. Payment verification if required
5. Data is stored in immutable Atoms
6. AtomRefs track latest versions
7. Version history maintained through linked Atoms

## Payment Flow
1. Calculate required payment based on:
   - Global base rate
   - Schema multiplier
   - Field multipliers
   - Trust distance scaling
2. Generate Lightning invoice
3. Monitor payment status
4. Verify payment completion
5. Process operation on success

## Schema Interpretation Flow
1. Load JSON schema definition
2. Validate structure and constraints
3. Process field configurations
4. Set up permission policies
5. Configure payment requirements
6. Apply schema transformations
7. Register schema with database
