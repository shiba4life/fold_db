# PBI-20: HTTP Client Implementation Simplification

## Overview

This PBI simplifies HTTP client implementations across all DataFold platforms (Rust CLI, JavaScript SDK, Python SDK). Currently, each platform has its own HTTP client logic with different request/response handling patterns, error management, and retry mechanisms. This simplification will reduce code complexity, ensure consistent network behavior, and improve maintainability across all SDKs.

[View in Backlog](../backlog.md#user-content-20)

## Problem Statement

The DataFold codebase has fragmented HTTP client implementations:
- **Rust CLI**: Custom HTTP client logic in [`src/cli/http_client.rs`](../../../src/cli/http_client.rs) with Rust-specific patterns
- **JavaScript SDK**: Platform-specific HTTP handling without standardized error management
- **Python SDK**: Separate HTTP client implementations that don't align with other platforms

This fragmentation leads to:
- Inconsistent network behavior across platforms
- Different error handling and retry strategies
- Duplicated HTTP client logic and maintenance overhead
- Varying timeout and connection management approaches
- Platform-specific networking bugs and issues

## User Stories

**Primary User Story**: As a platform engineer, I want simplified HTTP client implementations so that I can reduce code complexity and maintain consistent network behavior across all SDKs.

**Supporting User Stories**:
- As a developer, I want consistent HTTP APIs so I can implement network features uniformly across platforms
- As a DevOps engineer, I want standardized network error handling so I can troubleshoot issues consistently
- As a system administrator, I want unified timeout and retry behavior so network operations are predictable

## Technical Approach

### Implementation Strategy

1. **Create Unified HTTP Client Framework**
   - Shared request/response handling patterns
   - Common error handling and retry mechanisms
   - Standardized timeout and connection management

2. **Platform Integration (Consistent Behavior)**
   ```rust
   // Rust: Unified HTTP client adapter
   impl UnifiedHttpClient {
       pub async fn request(&self, req: UnifiedRequest) -> Result<UnifiedResponse, HttpError> {
           self.core_client.execute(req).await
       }
   }
   ```

   ```typescript
   // JavaScript: Unified HTTP utilities
   export class UnifiedHttpClient {
       async request(req: UnifiedRequest): Promise<UnifiedResponse> {
           return this.coreClient.execute(req);
       }
   }
   ```

   ```python
   # Python: Unified HTTP implementation
   class UnifiedHttpClient:
       async def request(self, req: UnifiedRequest) -> UnifiedResponse:
           return await self.core_client.execute(req)
   ```

3. **Network Pattern Standardization**
   - Common retry logic with exponential backoff
   - Unified timeout and connection pooling strategies
   - Consistent error classification and handling

### Files to be Modified
- [`src/cli/http_client.rs`](../../../src/cli/http_client.rs) - Rust HTTP client integration
- [`js-sdk/src/http/`](../../../js-sdk/src/http/) - JavaScript HTTP utilities (new)
- [`python-sdk/src/datafold_sdk/http/`](../../../python-sdk/src/datafold_sdk/http/) - Python HTTP utilities (new)

### Technical Benefits
- **Reduced Code Duplication**: Eliminate redundant HTTP client implementations
- **Consistent Network Behavior**: Same request/response patterns across platforms
- **Simplified Debugging**: Unified error handling and logging for network issues
- **Better Reliability**: Standardized retry and timeout mechanisms

## UX/UI Considerations

This change primarily affects developer and operational experiences:

- **Backward Compatibility**: Existing HTTP APIs continue to work through adapters
- **Error Consistency**: Standardized HTTP error messages and status codes across platforms
- **Performance Transparency**: Consistent timeout and retry behavior for predictable operations
- **Migration Path**: Clear migration guides for moving to unified HTTP clients
- **Monitoring**: Unified HTTP metrics and logging for better observability

## Acceptance Criteria

- [ ] Unified HTTP client framework designed and implemented
- [ ] Rust CLI integrated with unified HTTP client with backward compatibility
- [ ] JavaScript SDK HTTP utilities implemented using unified framework
- [ ] Python SDK HTTP utilities implemented using unified framework
- [ ] Cross-platform HTTP error handling consistency verified
- [ ] Unified retry logic with exponential backoff implemented
- [ ] Consistent timeout and connection management across all platforms
- [ ] HTTP request/response logging standardized across platforms
- [ ] All existing HTTP-related tests pass with new unified system
- [ ] Performance benchmarks show no regression in HTTP operations
- [ ] Network reliability improved through standardized retry mechanisms

## Dependencies

- **Prerequisites**: PBI-19 (Authentication Unification) - unified authentication will be used in HTTP clients
- **Concurrent**: Can leverage unified authentication and configuration infrastructure
- **Dependent PBIs**: PBI-21 (Middleware Alignment) will benefit from unified HTTP client patterns

## Open Questions

1. **Connection Pooling Strategy**: How should connection pooling be managed across different platforms?
2. **Request Serialization**: Should request/response serialization be standardized across platforms?
3. **HTTP Version Support**: What HTTP versions should be supported consistently across platforms?
4. **Compression Support**: Should compression be handled uniformly across all platforms?
5. **Metrics and Monitoring**: What HTTP metrics should be collected consistently across platforms?

## Related Tasks

Tasks will be created upon PBI approval and moved to "Agreed" status. Expected tasks include:

1. Design unified HTTP client framework and patterns
2. Implement Rust HTTP client adapter and integration
3. Create JavaScript SDK unified HTTP utilities
4. Create Python SDK unified HTTP utilities
5. Implement cross-platform HTTP error handling consistency
6. Add unified retry logic with exponential backoff
7. Standardize timeout and connection management
8. Add consistent HTTP request/response logging
9. Implement unified HTTP metrics collection
10. Update documentation with unified HTTP client guide
11. Performance testing and network reliability benchmarking
12. E2E CoS Test for HTTP client simplification