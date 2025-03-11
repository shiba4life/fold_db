# Progress

## Completed Features

### Core Database
- Schema management
- Query execution
- Mutation handling
- Atom-based storage
- Permission system

### Network Layer
- Node discovery
- Peer-to-peer communication
- Schema synchronization
- Query routing

### Sandbox Environment
- Docker-based sandbox for third-party containers
- Network isolation with internal Docker network
- Security measures (capability restrictions, privilege escalation prevention)
- Resource limits (CPU, memory, process limits)
- Sandboxed API Docker access
- Network-based communication between sandboxed containers and Datafold API
- Unix socket communication for maximum isolation

### Applications
- FoldSocial: A simple social media app using DataFold client
  - Post creation and viewing
  - Chronological timeline
  - Responsive UI
  - Simplified DataFold Node server with HTTP API
  - Data persistence using JSON files
  - Error handling for node connectivity

## In Progress

### Sandbox Environment
- Improved error handling and logging
- Volume mount support for sandboxed containers

### API Layer
- Additional API endpoints
- API versioning
- Rate limiting
- Authentication and authorization

## Planned Features

### Sandbox Environment
- Fine-grained permission system for API access
- Audit logging for all API requests
- Resource usage monitoring and reporting
- Automatic container cleanup

### API Layer
- GraphQL support
- Websocket support for real-time updates
- Batch operations
- Query caching
