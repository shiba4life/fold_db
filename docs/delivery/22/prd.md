# PBI-22: Crypto Module Structure Simplification

## Overview

This PBI simplifies crypto module structures across all DataFold platforms (Rust CLI, JavaScript SDK, Python SDK) to provide consistent cryptographic implementations and reduce security risks from code duplication. Currently, each platform has its own crypto patterns and implementations, leading to potential security vulnerabilities, maintenance overhead, and inconsistent cryptographic behavior. This simplification will standardize crypto operations and improve security posture.

[View in Backlog](../backlog.md#user-content-22)

## Problem Statement

The DataFold codebase has fragmented cryptographic implementations:
- **Rust CLI**: Multiple crypto modules in [`src/crypto/`](../../../src/crypto/) with overlapping functionality
- **JavaScript SDK**: Platform-specific cryptographic implementations without shared security patterns
- **Python SDK**: Separate crypto implementations that may not align with other platforms

This fragmentation leads to:
- Inconsistent cryptographic operations across platforms
- Potential security vulnerabilities from implementation differences
- Duplicated crypto code increasing attack surface and maintenance burden
- Varying key management and crypto configuration approaches
- Difficulty auditing and maintaining cryptographic security across platforms

## User Stories

**Primary User Story**: As a crypto engineer, I want simplified crypto module structures so that I can maintain consistent cryptographic implementations and reduce security risks from code duplication.

**Supporting User Stories**:
- As a security auditor, I want unified crypto implementations so I can audit cryptographic operations efficiently
- As a developer, I want shared crypto utilities so I don't need to implement cryptographic operations from scratch
- As a platform engineer, I want consistent key management so all platforms handle cryptographic keys securely

## Technical Approach

### Implementation Strategy

1. **Create Unified Crypto Framework**
   - Shared cryptographic operation implementations
   - Common key management and crypto configuration patterns
   - Standardized crypto error handling and validation

2. **Platform Integration (Security-First)**
   ```rust
   // Rust: Unified crypto adapter
   impl UnifiedCrypto {
       pub fn encrypt(&self, data: &[u8], key: &Key) -> Result<Vec<u8>, CryptoError> {
           self.core_crypto.encrypt(data, key)
       }
   }
   ```

   ```typescript
   // JavaScript: Unified crypto utilities
   export class UnifiedCrypto {
       encrypt(data: Uint8Array, key: Key): Result<Uint8Array, CryptoError> {
           return this.coreCrypto.encrypt(data, key);
       }
   }
   ```

   ```python
   # Python: Unified crypto implementation
   class UnifiedCrypto:
       def encrypt(self, data: bytes, key: Key) -> Result[bytes, CryptoError]:
           return self.core_crypto.encrypt(data, key)
   ```

3. **Crypto Pattern Standardization**
   - Common encryption/decryption workflows
   - Unified key derivation and management patterns
   - Consistent crypto validation and error handling

### Files to be Modified
- [`src/crypto/`](../../../src/crypto/) - Rust crypto module consolidation (multiple files)
- [`js-sdk/src/crypto/`](../../../js-sdk/src/crypto/) - JavaScript crypto utilities (new)
- [`python-sdk/src/datafold_sdk/crypto/`](../../../python-sdk/src/datafold_sdk/crypto/) - Python crypto utilities (new)

### Technical Benefits
- **Reduced Attack Surface**: Consolidated crypto code reduces potential vulnerabilities
- **Consistent Security**: Same cryptographic operations across all platforms
- **Simplified Auditing**: Single crypto codebase to review and audit
- **Better Key Management**: Unified cryptographic key handling across platforms

## UX/UI Considerations

This change primarily affects security and developer experiences:

- **Backward Compatibility**: Existing crypto APIs continue to work through adapters
- **Security Transparency**: Clear crypto error messages and validation across platforms
- **Developer Experience**: Consistent crypto patterns for easier secure development
- **Migration Path**: Smooth transition from platform-specific to unified crypto implementations
- **Documentation**: Comprehensive crypto implementation and security guide

## Acceptance Criteria

- [ ] Unified crypto framework designed and implemented
- [ ] Rust crypto modules consolidated with unified framework integration
- [ ] JavaScript SDK crypto utilities implemented using unified framework
- [ ] Python SDK crypto utilities implemented using unified framework
- [ ] Cross-platform crypto operation consistency verified
- [ ] Unified key management and derivation implemented
- [ ] Consistent crypto error handling and validation across all platforms
- [ ] Crypto operation timing consistency verified across platforms
- [ ] All existing crypto-related tests pass with new unified system
- [ ] Security audit conducted on unified crypto implementation
- [ ] Performance benchmarks show no regression in crypto operations

## Dependencies

- **Prerequisites**: PBI-21 (Middleware Alignment) - unified middleware will work with crypto operations
- **Concurrent**: Can leverage unified middleware and configuration infrastructure
- **Dependent PBIs**: PBI-23 (Documentation Consolidation) will benefit from unified crypto documentation

## Open Questions

1. **Crypto Algorithm Standardization**: Which cryptographic algorithms should be standardized across all platforms?
2. **Key Storage Security**: How should cryptographic keys be securely stored and accessed across platforms?
3. **Crypto Performance**: What's the acceptable performance impact for crypto operation standardization?
4. **Hardware Security Module Support**: Should HSM integration be standardized across platforms?
5. **Crypto Compliance**: How should crypto compliance requirements be handled consistently?

## Related Tasks

Tasks will be created upon PBI approval and moved to "Agreed" status. Expected tasks include:

1. Design unified crypto framework and security patterns
2. Consolidate Rust crypto modules with unified framework integration
3. Create JavaScript SDK unified crypto utilities
4. Create Python SDK unified crypto utilities
5. Implement cross-platform crypto operation consistency
6. Add unified key management and derivation logic
7. Standardize crypto error handling and validation
8. Add crypto operation timing consistency verification
9. Conduct comprehensive security audit of unified crypto system
10. Update documentation with unified crypto implementation guide
11. Performance testing and security benchmarking
12. E2E CoS Test for crypto module simplification