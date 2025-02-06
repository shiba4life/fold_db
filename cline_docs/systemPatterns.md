# System Patterns

## Core Architecture
- Modular design with clear separation of concerns:
  - Schema management
  - Permissions management
  - Atom/AtomRef storage
  - Database operations

## Key Components
1. FoldDB
   - Main entry point
   - Manages database operations
   - Coordinates between components
   
2. SchemaManager
   - Handles schema validation
   - Manages schema loading
   - Tracks available schemas

3. PermissionManager
   - Validates access permissions
   - Handles trust distance calculations
   - Manages explicit permissions

4. Atoms & AtomRefs
   - Atoms: Immutable data containers
   - AtomRefs: Pointers to latest versions
   - Version history tracking

## Design Patterns
- Repository pattern for data access
- Strategy pattern for permission checks
- Factory pattern for Atom creation
- Immutable data structures
- Builder pattern for schema construction

## Data Flow
1. Operations start with schema validation
2. Permissions are checked before access
3. Data is stored in immutable Atoms
4. AtomRefs track latest versions
5. Version history maintained through linked Atoms
