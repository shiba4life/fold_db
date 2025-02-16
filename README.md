# FoldDB

FoldDB is a schema-based database system that provides atomic operations, fine-grained permissions control, and version history tracking. It's built in Rust with a focus on data integrity and access control.

## Key Features

- **Schema-Based Storage**: Define and validate data structure with JSON schemas
- **Field-Level Permissions**: Fine-grained access control at the individual field level
- **Trust-Based Access**: Flexible permissions model using trust distance and explicit policies
- **Atomic Operations**: All data changes are atomic and create new versions
- **Version History**: Track and access the complete history of data changes

## Core Concepts

### Atoms & AtomRefs

- **Atoms**: Immutable data containers that store content and metadata
- **AtomRefs**: References that always point to the latest version of data
- **Version History**: Maintained through linked Atoms, allowing access to previous versions

### Permissions Model

- **Trust Distance**: Lower numbers indicate higher trust levels
- **Field-Level Control**: Permissions can be set for individual data fields
- **Access Policies**: Explicit read/write permissions using public key authentication

### Schema System

- **JSON Schemas**: Define data structure and validation rules
- **Field Definitions**: Specify data types and constraints
- **Permission Rules**: Integrate access control with schema definitions

## Technical Details

- Built in Rust for performance and safety
- Uses sled embedded database for persistent storage
- JSON-based data representation
- No external database dependencies

## Setup

1. Requirements:
   - Rust toolchain
   - Cargo package manager

2. Installation:
   ```bash
   cargo install folddb
   ```

## Usage

```bash
cargo run --bin datafold_node
```

```rust
use folddb::{FoldDB, Schema};

// Initialize database
let db = FoldDB::new("path/to/db")?;

// Load schema
let schema = Schema::from_json(schema_json)?;
db.load_schema("user_profile", schema)?;

// Write data
let data = json!({
    "name": "Alice",
    "email": "alice@example.com"
});
db.write("user_profile", data, public_key)?;

// Read data
let user = db.read("user_profile", "user123")?;
```

## Architecture

FoldDB follows a modular design with clear separation of concerns:

- **FoldDB**: Main entry point and operation coordinator
- **SchemaManager**: Handles schema validation and management
- **PermissionManager**: Controls access and trust calculations
- **Atom Storage**: Manages immutable data storage and versioning

## Technical Constraints

- All data changes create new versions (immutable data model)
- Trust distance must be a positive integer
- Schema must be loaded before data operations
- Write operations require explicit permissions
- Public key required for authentication

## Development

```bash
# Build project
cargo build

# Run tests
cargo test

# Run with example configuration
cargo run --example basic_usage
```

## Best Practices

1. **Schema Design**:
   - Define clear field-level permissions
   - Use appropriate data types
   - Consider versioning requirements

2. **Permissions**:
   - Set appropriate trust distances
   - Use explicit permissions for sensitive data
   - Review access patterns regularly

3. **Data Operations**:
   - Validate data before writing
   - Handle version history appropriately
   - Consider atomic operation boundaries

## License

[License details to be added]
