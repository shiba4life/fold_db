# PBI-PKM-1: React UI for Ed25519 Key Management with Existing Backend Integration

[View in Backlog](../backlog.md#user-content-PKM-1)

## Overview

This PBI implements a React-based user interface for Ed25519 key management that leverages Datafold's existing security infrastructure. The implementation focuses exclusively on client-side cryptographic operations and UI components, integrating with the already-available backend security routes, cryptographic libraries, and database operations.

## Problem Statement

Datafold has comprehensive backend security infrastructure including Ed25519 key operations, signature verification, public key storage, and security routes. However, there is no user interface to expose this functionality to end users. A React-based UI is needed that:

- Generates Ed25519 keypairs client-side using browser cryptography
- Stores private keys temporarily in React state (never persisted or transmitted)  
- Integrates with existing [`security_routes.rs`](../../src/datafold_node/security_routes.rs) endpoints
- Leverages existing [`signing.rs`](../../src/security/signing.rs) verification infrastructure
- Uses existing [`public_key_operations.rs`](../../src/db_operations/public_key_operations.rs) for public key storage

## User Stories

**Primary User Story:**
As a developer, I want to implement React UI components for Ed25519 key management with client-side cryptography and existing backend integration.

**Detailed User Stories:**
- As a user, I want to generate Ed25519 keypairs in my browser so that my private keys never leave my client
- As a user, I want to sign data client-side and have the server verify signatures using existing infrastructure
- As a user, I want a clean React interface for key management operations
- As a developer, I want to leverage existing security infrastructure rather than rebuilding backend functionality

## Technical Approach

### Client-Side Focus
Since the backend already provides comprehensive security infrastructure, this PBI focuses on:

1. **React UI Components**: Key generation interface, signing operations, data management
2. **Client-Side Cryptography**: Browser-based Ed25519 operations using `@noble/ed25519` or similar
3. **State Management**: Secure temporary storage of private keys in React state
4. **API Integration**: Connection to existing security endpoints

### Existing Backend Infrastructure (Already Available)
- ✅ **Ed25519 Operations**: [`src/security/keys.rs`](../../src/security/keys.rs)
- ✅ **Signature Verification**: [`src/security/signing.rs`](../../src/security/signing.rs)  
- ✅ **Public Key Storage**: [`src/db_operations/public_key_operations.rs`](../../src/db_operations/public_key_operations.rs)
- ✅ **Security Routes**: [`src/datafold_node/security_routes.rs`](../../src/datafold_node/security_routes.rs)
- ✅ **Audit Logging**: [`src/security/audit.rs`](../../src/security/audit.rs)

## UX/UI Considerations

- **Security-First Design**: Clear indication that private keys never leave the browser
- **Session Management**: Visual feedback for key lifecycle and automatic cleanup
- **Integration**: Consistent with existing Datafold React UI patterns
- **Accessibility**: WCAG 2.1 AA compliance for key management operations
- **Error Handling**: Graceful handling of cryptographic and network errors

## Acceptance Criteria

1. **Client-Side Key Generation**: Ed25519 keypairs generated in browser using secure random
2. **Private Key Security**: Private keys stored only in React state, never persisted or transmitted
3. **Backend Integration**: Successfully integrates with existing security routes and signature verification
4. **UI Components**: Complete React components for key management operations
5. **Session Management**: Automatic cleanup of private keys on logout/session expiry
6. **Testing**: Unit tests for crypto operations, integration tests for API calls, E2E tests for user workflows
7. **Security Validation**: Zero server-side private key exposure verified through testing

## Dependencies

### External Dependencies
- Browser cryptography library (`@noble/ed25519` recommended)
- React 18+ with hooks support
- TypeScript for type safety

### Internal Dependencies
- Existing security infrastructure (already implemented)
- React build system integration
- Existing authentication/session management

## Open Questions

1. **Crypto Library**: Confirm `@noble/ed25519` vs other Ed25519 browser implementations
2. **Session Integration**: How to integrate with existing Datafold session management
3. **UI Framework**: Confirm Tailwind CSS usage and component patterns
4. **Testing Environment**: Browser testing setup for cryptographic operations

## Related Tasks

See [Tasks for PBI PKM-1](./tasks.md) for detailed implementation tasks.