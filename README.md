# FoldDB

FoldDB is a Rust-based database system that provides schema-based data storage with atomic operations, fine-grained permissions control, and version history tracking.

## Features

- Schema-based data storage and validation
- Atomic operations with version history tracking
- Field-level permissions control
- Trust-based access control with explicit permissions and trust distance
- Immutable data model with version history

## How It Works

- Data is stored in Atoms (immutable units containing content and metadata)
- AtomRefs provide references to the latest version of data
- Schemas define the structure and permissions of data fields
- Permissions are controlled through:
  - Trust distance (lower means higher trust)
  - Explicit read/write policies per field
  - Public key based access control

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fold_db = "0.1.0"
```

## Requirements

- Rust toolchain (2021 edition)
- No external database dependencies (uses embedded sled)

## Technical Constraints

- Trust distance must be a positive integer (lower = higher trust)
- Permissions are enforced at field level
- All operations require public key for authentication
- Schema must be loaded before data operations
- Write operations require explicit permissions
- Read operations can use either trust distance or explicit permissions

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## Dependencies

- serde (1.0) - Serialization/deserialization
- sled (0.34) - Embedded database
- uuid (1.0) - Unique identifiers
- chrono (0.4) - Time handling

## License

This project is currently unlicensed.

## Contributing

Please ensure all tests pass before submitting pull requests:
```bash
cargo test
