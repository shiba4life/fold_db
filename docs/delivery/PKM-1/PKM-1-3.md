# PKM-1-3: Implement client-side signing functionality

[Back to task list](./tasks.md)

## Description

Create signing utilities and React hooks for client-side Ed25519 signature generation using private keys stored in React state. This task focuses on the cryptographic signing operations that enable authenticated API requests to the existing backend security infrastructure.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 16:43:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

1. **Client-Side Signing**: Ed25519 signature generation using private keys from React state
2. **Message Formatting**: Proper message formatting for compatibility with backend verification
3. **React Hook Integration**: Custom hooks for signing operations in React components
4. **Error Handling**: Comprehensive error handling for cryptographic operations
5. **Backend Compatibility**: Signatures compatible with existing [`signing.rs`](../../src/security/signing.rs) verification
6. **Performance**: Efficient signing operations suitable for interactive UI

## Implementation Plan

**Note**: Key generation is complete (see [PKM-1-2](./PKM-1-2.md)). This task will focus on *using* the generated private key (held in React state) to create signatures.

1. **Signing Utility**:
   - Create `signMessage` utility function in `src/datafold_node/static-react/src/utils/ed25519.ts`.
   - The function should take a message payload and a private key (`Uint8Array`) as input.
   - It will use `ed.signAsync` from `@noble/ed25519` to generate the signature.
   - The output should be a hex-encoded signature string.

2. **Signing Utilities**:
   - Implement message formatting and encoding utilities
   - Add signature format conversion for backend compatibility

3. **React Integration**:
   - Create `useSigning()` hook for React components
   - Implement state management for signing operations
   - Add loading states and error handling

4. **API Integration**:
   - Format signatures for existing security route endpoints
   - Add request signing utilities for authenticated API calls
   - Implement retry logic for failed signing operations

5. **Testing**:
   - Unit tests for signing functions
   - Integration tests with backend verification
   - Performance benchmarks for signing operations

## Verification

- [ ] Ed25519 signatures generated correctly client-side
- [ ] Signatures compatible with existing backend verification in [`signing.rs`](../../src/security/signing.rs)
- [ ] React hooks provide clean interface for signing operations
- [ ] Error handling covers all cryptographic edge cases
- [ ] Performance meets interactive UI requirements (<100ms for typical operations)
- [ ] Unit tests achieve >95% code coverage
- [ ] Integration tests verify backend compatibility

## Files Modified

- `src/datafold_node/static-react/utils/signing.ts` (to be created)
- `src/datafold_node/static-react/hooks/useSigning.ts` (to be created)
- `src/datafold_node/static-react/types/signatures.ts` (to be created)
- Related test files (to be created)