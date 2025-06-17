# API Reference Index

Fold DB provides three main interfaces for interacting with the system: CLI, HTTP REST API, and TCP protocol. This index provides navigation to all API documentation organized by topic and interface.

## Quick Navigation

### By Interface
- **[CLI Interface](./cli-interface.md)** - Command-line interface for all operations
- **[HTTP REST API](./http-rest-api-overview.md)** - RESTful web API endpoints
- **[TCP Protocol](./tcp-protocol.md)** - Binary TCP protocol with JSON payloads

### By Topic Area

#### Core Data Operations
- **[Schema Management API](./schema-management-api.md)** - Create, load, and manage schemas
- **[Data Operations API](./data-operations-api.md)** - Query and mutation operations
- **[Request/Response Formats](./request-response-formats.md)** - Data format specifications

#### Network & System
- **[Network API](./network-api.md)** - Peer discovery and connection management
- **[System Monitoring API](./system-monitoring-api.md)** - Health, status, metrics, and logging
- **[Transform API](./transform-api.md)** - Transform registration and management

#### Security & Access
- **[Authentication](./authentication.md)** - Authentication methods and security
- **[Permissions & Payments API](./permissions-payments-api.md)** - Access control and payment systems
- **[Error Handling](./error-handling.md)** - Error codes and troubleshooting

## Installation & Getting Started

The CLI tool is built as part of the main build process:

```bash
cargo build --release --workspace
# Binary available at target/release/datafold_cli
```

**Default Endpoints:**
- HTTP REST API: `http://localhost:9001`
- TCP Protocol: `localhost:9000`

## Integration Guides

For implementation examples and integration patterns, see:
- [CLI Integration Guide](../guides/cli-authentication.md)
- [HTTP API Integration](../guides/integration/)
- [Python SDK](./sdks/python/README.md)
- [JavaScript SDK](./sdks/javascript/README.md)

## Related Documentation

- [Architecture Overview](../architecture.md)
- [Schema Management Guide](../schema-management.md)
- [Network Operations](../network-operations.md)
- [Deployment Guide](../deployment-guide.md)