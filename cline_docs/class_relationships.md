# Class Relationships Documentation

## Core Components

### FoldDB
- Central coordinator for all database operations
- Manages interactions between SchemaManager, PermissionManager, and PaymentManager
- Handles Atom creation and retrieval
- Core entry point for all database operations

### Atom
- Immutable data container with version tracking
- Contains actual content and metadata
- Links to previous versions through prev_atom_uuid
- Includes schema and source information
- Timestamps for auditing

### AtomRef
- Mutable reference to latest version of an Atom
- Provides indirection for version management
- Tracks update timestamps
- Enables atomic updates through reference switching

## Schema Management

### SchemaManager
- Manages schema lifecycle and validation
- Coordinates with SchemaInterpreter for schema loading
- Maintains registry of available schemas
- Provides schema validation services

### Schema
- Represents data structure definition
- Contains collection of SchemaFields
- Handles validation and transformation rules
- Defines data shape and constraints

### SchemaInterpreter
- Interprets JSON schema definitions
- Validates schema structure
- Processes field configurations
- Handles schema transformations

### SchemaField
- Defines individual field properties
- Links to permission policies
- Contains payment configurations
- Specifies field type and constraints

## Permissions

### PermissionManager
- Validates access permissions
- Calculates trust distances
- Manages permission policies
- Coordinates with PermissionWrapper

### PermissionWrapper
- Wraps data with permission checks
- Provides permission verification layer
- Ensures consistent permission application

### PermissionPolicy
- Defines read/write access rules
- Contains policy validation logic
- Specifies access control rules

## Payment System

### PaymentManager
- Coordinates payment processing
- Manages Lightning Network integration
- Handles invoice generation
- Verifies payment completion

### PaymentCalculator
- Calculates fees based on trust and schema
- Applies scaling factors
- Determines payment requirements

### LightningClient
- Interfaces with Lightning Network
- Creates and manages invoices
- Verifies payment status

## Application Layer

### DataFoldNode
- Manages application containers
- Provides API access
- Coordinates network access
- Handles client connections

### SocketServer
- Manages network communications
- Handles connection lifecycle
- Provides socket-based API

### DataFoldClient
- Client interface to FoldDB
- Handles queries and mutations
- Manages connection state

## Key Relationships

1. Version Management
   - Atoms link to previous versions creating a chain
   - AtomRefs provide latest version lookup
   - Enables atomic updates and version history

2. Schema Validation Flow
   - SchemaManager coordinates with SchemaInterpreter
   - Schemas contain SchemaFields
   - Fields link to PermissionPolicies and PaymentConfigs

3. Access Control
   - PermissionManager validates through PermissionPolicies
   - PermissionWrapper provides security layer
   - Integrated with schema-level permissions

4. Payment Processing
   - PaymentManager coordinates with PaymentCalculator
   - LightningClient handles payment network interaction
   - Integrated with schema-level payment requirements

5. Application Integration
   - DataFoldNode provides container management
   - SocketServer handles communications
   - DataFoldClient provides API access

This architecture enables:
- Immutable data storage with version tracking
- Schema-based validation and transformation
- Fine-grained permission control
- Integrated payment processing
- Containerized application management
