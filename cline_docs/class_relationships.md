# FoldDB Class Relationships

## Core Components Overview

### FoldDB (Main Entry Point)
- **Primary Dependencies:**
  - SchemaManager
  - PermissionWrapper
  - Atom
  - AtomRef
- **Responsibilities:**
  - Database operations coordination
  - Schema loading and management
  - Permission validation through PermissionWrapper
  - Atom and AtomRef storage management
  - Query and mutation execution

### SchemaManager
- **Dependencies:**
  - Schema
- **Responsibilities:**
  - Schema validation and storage
  - Schema transformation execution
  - Schema lifecycle management (load/unload)
- **Used By:**
  - FoldDB for schema operations
  - PermissionWrapper for permission validation

### PermissionManager
- **Dependencies:**
  - PermissionsPolicy
  - TrustDistance
- **Responsibilities:**
  - Permission validation
  - Trust distance calculations
  - Read/write permission checks
- **Used By:**
  - PermissionWrapper as the core permission logic

### Atom & AtomRef System
- **Atom**
  - Immutable data container
  - Stores content and metadata
  - Links to previous versions
- **AtomRef**
  - Points to latest version of data
  - Enables version tracking
  - Used by FoldDB for data access

## Data Flow Relationships

1. **Schema Definition Flow**
   ```
   FoldDB -> SchemaManager -> Schema
   ```
   - FoldDB receives schema definitions
   - SchemaManager validates and stores schemas
   - Schema objects define data structure and permissions

2. **Permission Check Flow**
   ```
   FoldDB -> PermissionWrapper -> PermissionManager -> PermissionsPolicy
   ```
   - FoldDB initiates permission checks
   - PermissionWrapper coordinates permission validation
   - PermissionManager performs actual permission checks
   - PermissionsPolicy defines access rules

3. **Data Storage Flow**
   ```
   FoldDB -> Atom -> AtomRef
   ```
   - FoldDB manages data operations
   - Atoms store immutable data versions
   - AtomRefs track latest versions

## Key Relationships

### FoldDB ↔ SchemaManager
- FoldDB delegates all schema operations
- SchemaManager provides schema validation and management
- Two-way communication for schema operations

### FoldDB ↔ PermissionWrapper
- FoldDB requests permission checks
- PermissionWrapper coordinates with PermissionManager
- Enforces access control on operations

### Atom ↔ AtomRef
- AtomRef maintains pointer to current Atom
- Atoms link to previous versions
- Enables version history tracking

### Schema ↔ PermissionsPolicy
- Schema defines field-level permissions
- PermissionsPolicy implements access rules
- Integrated through permission checks

## Implementation Details

### Concurrency Management
- SchemaManager uses Mutex for thread-safe schema access
- Atomic operations through immutable Atoms
- Version control through AtomRef updates

### Error Handling
- Consistent error types across components
- SchemaError for schema-related issues
- Permission validation results
- Atomic operation guarantees

### Data Access Patterns
1. Schema validation first
2. Permission checks second
3. Data access/modification last
4. Version history maintenance

## Extension Points

### Schema Transformation
- Transform pipeline in SchemaManager
- Extensible schema validation
- Custom schema operations

### Permission Models
- Trust distance calculations
- Explicit permission overrides
- Public key based access control

### Storage Layer
- Sled database backend
- In-memory caching
- Versioned data storage
