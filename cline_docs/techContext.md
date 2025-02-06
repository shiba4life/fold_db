# Technical Context

## Technologies Used
- Rust programming language
- sled embedded database for persistent storage
- serde for serialization/deserialization
- JSON for data representation

## Development Setup
- Rust toolchain required
- No external database dependencies (uses embedded sled)
- File-based storage system

## Technical Constraints
- Immutable data model (all changes create new versions)
- Trust distance must be a positive integer (lower = higher trust)
- Permissions are enforced at field level
- All operations require public key for authentication
- Schema must be loaded before data operations
- Write operations require explicit permissions
- Read operations can use either trust distance or explicit permissions
