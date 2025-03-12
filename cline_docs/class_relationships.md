# Class Relationships Documentation

## Core Components

### FoldDB
- Central coordinator for all database operations
- Manages interactions between SchemaManager, PermissionManager, and PaymentManager
- Handles Atom creation and retrieval
- Core entry point for all database operations
- Coordinates with ErrorManager for error handling and recovery

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

### DataFoldNode
- Application container for FoldDB
- Manages schema loading and persistence
- Coordinates network operations
- Provides API for external applications
- Handles operation execution
- Manages network discovery and connections

## Schema Management

### SchemaManager
- Manages schema lifecycle and validation
- Coordinates with SchemaInterpreter for schema loading
- Maintains registry of available schemas
- Provides schema validation services
- Handles schema persistence and version control
- Tracks relationships between schemas
- Manages field mappings and transformations

### Schema
- Represents data structure definition
- Contains collection of SchemaFields
- Handles validation and transformation rules
- Defines data shape and constraints
- Maintains version information
- Tracks relationships with other schemas
- Manages field mappings and references

### SchemaInterpreter
- Interprets JSON schema definitions
- Validates schema structure
- Processes field configurations
- Handles schema transformations
- Provides error handling for interpretation
- Manages field transformations

### SchemaField
- Defines individual field properties
- Links to permission policies
- Contains payment configurations
- Specifies field type and constraints
- Maintains field mappings to other schemas
- Contains validation rules
- Tracks relationships and dependencies

### SchemaPersistence
- Handles schema storage and retrieval
- Manages version control for schemas
- Tracks schema changes over time
- Provides persistence layer abstraction

### SchemaRelationship
- Defines relationships between schema fields
- Specifies relationship types and constraints
- Handles relationship validation
- Supports transformation operations

### FieldMapping
- Defines mappings between fields across schemas
- Contains transformation rules
- Validates mapping consistency
- Applies field-level transformations

## Permissions

### PermissionManager
- Validates access permissions
- Calculates trust distances
- Manages permission policies
- Coordinates with PermissionWrapper
- Handles permission-related errors
- Integrates with error management system

### PermissionWrapper
- Wraps data with permission checks
- Provides permission verification layer
- Ensures consistent permission application
- Validates wrapped data integrity

### PermissionPolicy
- Defines read/write access rules
- Contains policy validation logic
- Specifies access control rules
- Provides policy checking capabilities

## Payment System

### PaymentManager
- Coordinates payment processing
- Manages Lightning Network integration
- Handles invoice generation
- Verifies payment completion
- Manages payment-related errors
- Integrates with error handling system

### PaymentCalculator
- Calculates fees based on trust and schema
- Applies scaling factors
- Determines payment requirements
- Validates payment calculations

### LightningClient
- Interfaces with Lightning Network
- Creates and manages invoices
- Verifies payment status
- Handles network-related errors

## Error Handling

### ErrorManager
- Centralizes error handling logic
- Coordinates error recovery
- Manages error logging
- Provides context-aware error handling

### ErrorContext
- Contains error-specific information
- Maintains error context and stack traces
- Supports error recovery actions
- Enables detailed error reporting

## Server Components

### UiServer
- Provides management UI interface
- Serves static files for web interface
- Handles schema operations
- Manages network operations
- Provides API for UI interactions
- Runs on dedicated port (default 8080)

### AppServer
- Provides 3rd party API access
- Implements signature verification
- Includes comprehensive logging
- Enforces permission checks
- Provides secure API endpoints
- Runs on dedicated port (default 8081)

### AppLogger
- Handles structured logging
- Records operation details
- Logs security events
- Provides debug logging
- Maintains separate log files by category

## Network Components

### NetworkCore
- Central coordinator for network operations
- Manages connections to other nodes
- Routes messages between components
- Handles node discovery
- Coordinates query and schema services
- Manages network lifecycle

### ConnectionManager
- Manages peer connections
- Handles connection lifecycle
- Monitors connection health
- Provides message sending interface
- Manages connection recovery

### MessageRouter
- Routes messages to appropriate handlers
- Manages message type registration
- Coordinates message processing
- Provides unified message handling interface
- Supports extensible message types

### QueryService
- Executes queries locally
- Sends queries to remote nodes
- Manages query callbacks
- Tracks pending queries
- Handles query responses

### SchemaService
- Lists available schemas
- Retrieves schemas from remote nodes
- Manages schema callbacks
- Tracks pending schema requests
- Handles schema responses

## Key Relationships

1. Server Architecture
   - DataFoldNode provides core functionality
   - UiServer handles management interface
   - AppServer provides secure API access
   - Both servers operate concurrently
   - Servers share the same DataFoldNode instance

2. Version Management
   - Atoms link to previous versions creating a chain
   - AtomRefs provide latest version lookup
   - Enables atomic updates and version history
   - Schema versions tracked through SchemaPersistence

3. Schema Management Flow
   - SchemaManager coordinates overall schema operations
   - SchemaPersistence handles storage and versioning
   - SchemaInterpreter processes and validates schemas
   - Schemas contain SchemaFields and SchemaRelationships
   - FieldMappings define transformations between schemas
   - Fields link to PermissionPolicies and PaymentConfigs

4. Schema Transformation Flow
   - SchemaRelationships define field connections
   - FieldMappings specify transformation rules
   - SchemaInterpreter executes transformations
   - SchemaPersistence tracks transformation history

5. Network Communication Flow
   - NetworkCore coordinates all network operations
   - ConnectionManager handles peer connections
   - MessageRouter directs messages to appropriate handlers
   - QueryService and SchemaService handle specific operations
   - NodeDiscovery finds and announces nodes

6. Access Control
   - PermissionManager validates through PermissionPolicies
   - PermissionWrapper provides security layer
   - Integrated with schema-level permissions
   - ErrorManager handles permission failures
   - AppServer enforces signature verification

7. Payment Processing
   - PaymentManager coordinates with PaymentCalculator
   - LightningClient handles payment network interaction
   - Integrated with schema-level payment requirements
   - ErrorManager handles payment failures

8. Error Management
   - ErrorManager coordinates all error handling
   - ErrorContext provides detailed error information
   - Components integrate with error handling system
   - Supports error recovery and logging

This architecture enables:
- Immutable data storage with version tracking
- Schema-based validation and transformation
- Advanced schema relationship management
- Field-level mapping and transformation
- Fine-grained permission control
- Integrated payment processing
- Robust error handling and recovery
- Containerized application management
- Secure API access with signature verification
- Separate management and API interfaces
