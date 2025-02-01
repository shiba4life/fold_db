# Technical Context

## Technologies Used
- Rust (primary implementation language)
- sled (embedded key-value store)
- async-graphql (GraphQL server implementation)
- tokio (async runtime)
- serde (serialization/deserialization)
- chrono (datetime handling)
- uuid (UUID generation)

## Development Setup
1. Project Structure:
   - src/
     - main.rs (GraphQL server setup)
     - folddb.rs (Core FoldDB implementation)
   - Cargo.toml (dependencies and project config)

2. Key Dependencies:
```toml
[dependencies]
sled = "*"
async-graphql = "*"
tokio = { version = "*", features = ["full"] }
serde = { version = "*", features = ["derive"] }
serde_json = "*"
chrono = { version = "*", features = ["serde"] }
uuid = { version = "*", features = ["v4", "serde"] }
```

## Technical Constraints
1. Data Storage:
   - All data must be stored as immutable atoms
   - Atoms form chains with prev references
   - AtomRefs maintain latest atom pointers

2. Schema Management:
   - Internal schemas map fields to aref_uuids
   - Schemas are stored in memory for fast access
   - Field values must be JSON-encoded

3. GraphQL Interface:
   - Exposes simplified data access
   - Hides internal UUID complexity
   - Returns JSON-formatted field values
