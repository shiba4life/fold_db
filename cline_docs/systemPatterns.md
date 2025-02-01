# System Patterns

## Architecture Patterns

### 1. Core Data Model
- **Atom Pattern**
  - Immutable data units containing content and metadata
  - Forms chains through prev references
  - Each atom has a unique UUID
  - Content is JSON-encoded for flexibility

- **AtomRef Pattern**
  - Maintains pointer to latest atom in a chain
  - Enables efficient latest value lookup
  - Stored with UUID key in sled database

### 2. Schema Management
- **Internal Schema Pattern**
  - Maps human-readable field names to aref_uuids
  - Stored in memory for performance
  - Organized by schema name (e.g., "user_profile")
  - Enables abstraction of internal UUIDs

### 3. Data Access Patterns
- **GraphQL Interface**
  - Single query type for field access
  - Schema-based field resolution
  - JSON value return format
  - Error handling through GraphQL Result type

### 4. Key Technical Decisions
1. **Embedded Key-Value Store**
   - Using sled for reliable, fast storage
   - Binary-safe value storage
   - ACID compliance for data integrity

2. **In-Memory Schema Mapping**
   - Schemas loaded at startup
   - Fast field-to-uuid resolution
   - Trade-off: memory usage vs. performance

3. **Async GraphQL Implementation**
   - Built on tokio runtime
   - Non-blocking field resolution
   - Efficient handling of concurrent requests

4. **Error Handling Strategy**
   - Custom error types with std::error::Error
   - GraphQL error propagation
   - Clear error messages for clients
