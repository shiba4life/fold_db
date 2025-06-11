# PBI-19: Authentication/Signing Logic Unification

## Overview

This PBI unifies authentication and signing logic across all DataFold platforms (Rust CLI, JavaScript SDK, Python SDK). Currently, each platform implements authentication and signing mechanisms independently, leading to inconsistent security implementations, potential vulnerabilities, and maintenance overhead. This unification will provide consistent security behavior, reduce attack surface, and simplify security auditing.

[View in Backlog](../backlog.md#user-content-19)

## Problem Statement

The DataFold codebase has fragmented authentication and signing implementations:
- **Rust CLI**: Custom signing logic in [`src/cli/signing_config.rs`](../../../src/cli/signing_config.rs) and [`src/datafold_node/signature_auth.rs`](../../../src/datafold_node/signature_auth.rs)
- **JavaScript SDK**: Platform-specific authentication patterns without shared security utilities
- **Python SDK**: Separate authentication implementations that may not align with other platforms

This fragmentation leads to:
- Inconsistent security implementations across platforms
- Potential security vulnerabilities from implementation differences
- Difficulty maintaining and auditing security code across platforms
- Risk of authentication bypasses due to platform-specific inconsistencies
- Maintenance overhead from duplicated security logic

## User Stories

**Primary User Story**: As a security engineer, I want unified authentication and signing logic across all platforms so that I can maintain consistent security implementations and reduce attack surface.

**Supporting User Stories**:
- As a security auditor, I want consistent authentication patterns so I can audit security implementations efficiently
- As a developer, I want shared authentication utilities so I don't need to implement security logic from scratch
- As a platform engineer, I want unified key management so all platforms handle cryptographic operations consistently

## Technical Approach

### Implementation Strategy

1. **Create Unified Authentication Framework**
   - Shared signing and verification logic across platforms
   - Common key management and validation patterns
   - Standardized authentication error handling and logging

2. **Platform Integration (Security-First)**
   ```rust
   // Rust: Unified authentication adapter
   impl UnifiedAuth {
       pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> AuthResult {
           self.core_verifier.verify(message, signature)
       }
   }
   ```

   ```typescript
   // JavaScript: Unified auth utilities
   export class UnifiedAuth {
       verifySignature(message: Uint8Array, signature: Signature): AuthResult {
           return this.coreVerifier.verify(message, signature);
       }
   }
   ```

   ```python
   # Python: Unified auth implementation
   class UnifiedAuth:
       def verify_signature(self, message: bytes, signature: Signature) -> AuthResult:
           return self.core_verifier.verify(message, signature)
   ```

3. **Security Pattern Standardization**
   - Common signature verification workflows
   - Unified key derivation and management
   - Consistent authentication timing and error handling

### Files to be Modified
- [`src/cli/signing_config.rs`](../../../src/cli/signing_config.rs) - Rust signing configuration
- [`src/datafold_node/signature_auth.rs`](../../../src/datafold_node/signature_auth.rs) - Rust authentication logic
- [`js-sdk/src/auth/`](../../../js-sdk/src/auth/) - JavaScript authentication utilities (new)
- [`python-sdk/src/datafold_sdk/auth/`](../../../python-sdk/src/datafold_sdk/auth/) - Python authentication utilities (new)

### Technical Benefits
- **Consistent Security**: Same authentication logic across all platforms
- **Reduced Attack Surface**: Unified implementation reduces security vulnerabilities
- **Simplified Auditing**: Single authentication codebase to review and audit
- **Better Key Management**: Consistent cryptographic operations across platforms

## UX/UI Considerations

This change primarily affects security and developer experiences:

- **Backward Compatibility**: Existing authentication APIs continue to work during migration
- **Security Transparency**: Clear security error messages and logging across platforms
- **Developer Experience**: Consistent authentication patterns for easier development
- **Migration Path**: Smooth transition from platform-specific to unified authentication
- **Documentation**: Comprehensive security implementation guide

## Acceptance Criteria

- [ ] Unified authentication framework designed and implemented
- [ ] Rust CLI integrated with unified authentication with backward compatibility
- [ ] JavaScript SDK authentication utilities implemented using unified framework
- [ ] Python SDK authentication utilities implemented using unified framework
- [ ] Cross-platform signature verification consistency verified
- [ ] Unified key management and derivation implemented
- [ ] Consistent authentication error handling across all platforms
- [ ] Security timing attack protection standardized across platforms
- [ ] All existing authentication tests pass with new unified system
- [ ] Security audit conducted on unified authentication implementation
- [ ] Performance benchmarks show no regression in authentication operations

## Dependencies

- **Prerequisites**: PBI-18 (Configuration Consolidation) - unified configuration will support authentication settings
- **Concurrent**: Can leverage unified configuration infrastructure
- **Dependent PBIs**: PBI-20 (HTTP Client Simplification) will benefit from unified authentication patterns

## Open Questions

1. **Key Storage Security**: How should private keys be securely stored and accessed across different platforms?
2. **Authentication Caching**: Should authentication results be cached, and if so, what are the security implications?
3. **Signature Algorithm Migration**: How should we handle migration between different signature algorithms in the future?
4. **Multi-Key Support**: Should the unified system support multiple keys per user/client?
5. **Authentication Logging**: What level of authentication logging is appropriate without compromising security?

## Related Tasks

Tasks will be created upon PBI approval and moved to "Agreed" status. Expected tasks include:

1. Design unified authentication and signing framework
2. Implement Rust authentication adapter and integration
3. Create JavaScript SDK unified authentication utilities
4. Create Python SDK unified authentication utilities
5. Implement cross-platform signature verification consistency
6. Add unified key management and derivation logic
7. Standardize authentication error handling and logging
8. Add security timing attack protection across platforms
9. Conduct security audit of unified authentication system
10. Update documentation with unified authentication guide
11. Performance testing and security benchmarking
12. E2E CoS Test for authentication unification