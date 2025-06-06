# Fold DB - Distributed Data Platform

Fold DB is a Rust-based distributed data platform that provides schema-driven data storage, querying, and transformation capabilities with peer-to-peer networking, fine-grained permissions, and programmable transforms.

## Quick Start

### Installation

Build all crates and binaries in release mode:

```bash
cargo build --release --workspace
```

### Starting a Node

Start a fold db node with HTTP and TCP servers:

```bash
# Start the HTTP server (includes web UI)
cargo run --bin datafold_http_server -- --port 9001

# Or start the TCP-only node
cargo run --bin datafold_node -- --port 9000 --tcp-port 9000
```

### Basic Usage

1. **Load a Schema**: Define the structure of your data
2. **Insert Data**: Create atoms (data records) in the schema
3. **Query Data**: Retrieve data with field-level permissions
4. **Transform Data**: Use programmable transforms for computed fields

## Core Concepts

### Atoms
Atoms are the fundamental storage units in fold db. Each atom contains:
- A unique UUID identifier
- Field values according to a schema
- Version history for updates
- Permission and payment metadata

### Schemas
Schemas define:
- Field types (Single, Collection, Range)
- Permission policies (read/write access)
- Payment requirements
- Transform definitions
- **Immutable structure** - see [Schema Management](schema-management.md) and [Migration Guide](migration-guide.md)

### Range Fields
Special field type that stores key-value pairs, enabling:
- Hierarchical data storage
- Efficient key-based queries
- Pattern matching and range operations
- Time-series data storage

### Transforms
Programmable computation engine that:
- Executes custom logic on field changes
- Supports a domain-specific language (DSL)
- Provides built-in functions and operators
- Enables real-time data processing

### Networking
Peer-to-peer networking enables:
- Node discovery and connection
- Schema synchronization
- Distributed query execution
- Trust-based access control

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Client    │    │   HTTP Client   │    │   TCP Client    │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────▼─────────────┐
                    │      DataFold Node       │
                    │                          │
                    │  ┌─────────────────────┐ │
                    │  │     FoldDB Core     │ │
                    │  │                     │ │
                    │  │ ┌─────────────────┐ │ │
                    │  │ │  Atom Manager   │ │ │
                    │  │ └─────────────────┘ │ │
                    │  │ ┌─────────────────┐ │ │
                    │  │ │ Schema Manager  │ │ │
                    │  │ └─────────────────┘ │ │
                    │  │ ┌─────────────────┐ │ │
                    │  │ │Transform Engine │ │ │
                    │  │ └─────────────────┘ │ │
                    │  └─────────────────────┘ │
                    │                          │
                    │  ┌─────────────────────┐ │
                    │  │   Network Layer     │ │
                    │  └─────────────────────┘ │
                    │                          │
                    │  ┌─────────────────────┐ │
                    │  │  Storage Engine     │ │
                    │  └─────────────────────┘ │
                    └──────────────────────────┘
```

## Key Features

- **Event-Driven Architecture**: Automatic transform execution on field changes
- **Schema Immutability**: Schemas are immutable once created, ensuring data consistency and integrity
- **Fine-Grained Permissions**: Field-level access control with trust distances
- **Payment System**: Configurable fees for data access and operations
- **Multi-Protocol APIs**: CLI, HTTP REST, and TCP interfaces
- **Distributed Networking**: P2P node discovery and data replication
- **Range Queries**: Efficient key-value storage and retrieval
- **Transform DSL**: Custom computation language for data processing

## Use Cases

- **Analytics Platforms**: Store and process event data with time-series queries
- **Content Management**: Manage structured content with permissions and workflows  
- **IoT Data Collection**: Collect and transform sensor data in real-time
- **Financial Systems**: Handle transactions with fine-grained access control
- **Social Networks**: Manage user profiles and relationships with privacy controls
- **E-commerce**: Track inventory and orders across multiple locations

## Documentation Structure

- **[Architecture](./architecture.md)** - Detailed system architecture and design patterns
- **[Use Cases](./use-cases.md)** - Comprehensive examples and scenarios  
- **[API Reference](./api-reference.md)** - Complete API documentation
- **[Deployment Guide](./deployment-guide.md)** - Deployment patterns and configuration
- **[Schema Management](./schema-management.md)** - Schema system documentation
- **[Transforms](./transforms.md)** - Transform system and DSL documentation
- **[Network Operations](./network-operations.md)** - Distributed functionality
- **[Permissions and Fees](./permissions-and-fees.md)** - Access control and payment system
- **[Developer Guide](./developer-guide.md)** - Integration and embedding guide

## Example: User Profile Management

```bash
# 1. Load a schema
curl -X POST http://localhost:9001/api/schema \
  -H "Content-Type: application/json" \
  -d '{
    "name": "UserProfile",
    "fields": {
      "username": {"field_type": "Single", "permission_policy": {...}},
      "email": {"field_type": "Single", "permission_policy": {...}},
      "profile_data": {"field_type": "Range", "permission_policy": {...}}
    }
  }'

# 2. Create a user
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"create\",\"data\":{\"username\":\"alice\",\"email\":\"alice@example.com\",\"profile_data\":{\"location\":\"San Francisco\",\"bio\":\"Software engineer\"}}}"
  }'

# 3. Query users
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"query\",\"schema\":\"UserProfile\",\"fields\":[\"username\",\"email\"],\"filter\":{\"username\":\"alice\"}}"
  }'
```

## Getting Help

- Check the [Architecture documentation](./architecture.md) for system design details
- See [Use Cases](./use-cases.md) for practical examples
- Refer to [API Reference](./api-reference.md) for complete API documentation
- Review [Developer Guide](./developer-guide.md) for integration patterns

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Submit a pull request

---

**Next Steps**: Explore the [Architecture documentation](./architecture.md) to understand fold db's design principles and components.