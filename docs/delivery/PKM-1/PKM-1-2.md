# PKM-1-2: Implement React key generation component

[Back to task list](./tasks.md)

## Description

Create a React component for Ed25519 keypair generation that operates entirely client-side with secure random number generation. The component will temporarily store private keys in React state while providing a secure user interface for key management operations.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 16:43:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

1. **Client-Side Key Generation**: Ed25519 keypair generation using selected library from PKM-1-1
2. **Secure State Management**: Private keys stored only in React state, never persisted
3. **Public Key Registration**: Integration with existing backend endpoints for public key storage
4. **User Interface**: Clean, accessible interface for key generation operations
5. **Error Handling**: Graceful handling of cryptographic and network errors
6. **Security Indicators**: Clear UI feedback about private key security boundaries

## Implementation Plan

1. **Component Structure**:
   - Create `KeyGenerationComponent.tsx` with React hooks
   - Implement `useKeyGeneration()` custom hook for state management
   - Add TypeScript interfaces for key management state

2. **Cryptographic Integration**:
   - Integrate selected Ed25519 library from PKM-1-1
   - Implement secure random key generation
   - Add key format conversion utilities

3. **Backend Integration**:
   - Connect to existing security routes for public key registration
   - Implement API calls to [`security_routes.rs`](../../src/datafold_node/security_routes.rs)
   - Add error handling for network operations

4. **UI Implementation**:
   - Design key generation interface with security messaging
   - Add progress indicators and success/error states
   - Implement accessibility features (ARIA labels, keyboard navigation)

## Verification

- [ ] React component successfully generates Ed25519 keypairs client-side
- [ ] Private keys stored only in React state, never persisted to storage
- [ ] Public keys successfully registered with backend via existing security routes
- [ ] Component properly handles errors and edge cases
- [ ] UI meets accessibility standards (WCAG 2.1 AA)
- [ ] Security messaging clearly communicates private key boundaries
- [ ] Unit tests cover key generation and state management
- [ ] Integration tests verify backend connectivity

## Files Modified

- `src/datafold_node/static-react/components/KeyGenerationComponent.tsx` (to be created)
- `src/datafold_node/static-react/hooks/useKeyGeneration.ts` (to be created)
- `src/datafold_node/static-react/types/cryptography.ts` (to be created)
- `src/datafold_node/static-react/utils/ed25519.ts` (to be created)
- Related test files (to be created)