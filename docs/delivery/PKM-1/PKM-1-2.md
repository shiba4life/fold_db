# PKM-1-2: Implement React key generation component

[Back to task list](./tasks.md)

## Description

Create a React component for Ed25519 keypair generation that operates entirely client-side with secure random number generation. The component will temporarily store private keys in React state while providing a secure user interface for key management operations.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 16:43:00 | Created | N/A | Proposed | Task file created | User |
| 2025-01-03 09:15:00 | Status Change | Proposed | InProgress | Starting implementation of React key generation component | AI Agent |
| 2025-01-03 09:45:00 | Status Change | InProgress | Review | Implementation completed - React components, tests, and backend integration working | AI Agent |
| 2025-01-03 10:15:00 | Status Change | Review | Done | User approved the implementation. | AI Agent |

## Requirements

1. **Client-Side Key Generation**: Ed25519 keypair generation using **@noble/ed25519** (selected from PKM-1-1)
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
   - Integrate **@noble/ed25519** library (4KB bundle, async API)
   - Implement secure random key generation using `ed.utils.randomPrivateKey()`
   - Add key format conversion utilities (hex encoding/decoding)
   - Follow patterns from [PKM-1-1-noble-ed25519-guide.md](./PKM-1-1-noble-ed25519-guide.md)

3. **Backend Integration**:
   - Connect to existing security routes for public key registration
   - Implement API calls to [`security_routes.rs`](../../src/datafold_node/security_routes.rs)
   - Add error handling for network operations

4. **UI Implementation**:
   - Design key generation interface with security messaging
   - Add progress indicators and success/error states
   - Implement accessibility features (ARIA labels, keyboard navigation)

## Implementation Guidance

Based on PKM-1-1 research findings, use the following @noble/ed25519 patterns:

**Key Generation:**
```typescript
import * as ed from '@noble/ed25519';

// Generate keypair
const privateKey = ed.utils.randomPrivateKey();
const publicKey = await ed.getPublicKeyAsync(privateKey);
```

**Performance Expectations:**
- Key Generation: ~9,173 ops/sec
- Bundle Size: 4KB gzipped
- Browser Support: Chrome 67+, Firefox 60+, Safari 13+, Edge 18+

**Security Considerations:**
- Use async API for better performance
- Private keys are 32 bytes, public keys are 32 bytes
- Implement proper error handling for cryptographic operations
- Follow security patterns from the comprehensive guide

## Verification

- [x] React component successfully generates Ed25519 keypairs client-side
- [x] Private keys stored only in React state, never persisted to storage
- [x] Public keys successfully registered with backend via existing security routes
- [x] Component properly handles errors and edge cases
- [x] UI meets accessibility standards (WCAG 2.1 AA)
- [x] Security messaging clearly communicates private key boundaries
- [x] Unit tests cover key generation and state management
- [x] Integration tests verify backend connectivity

## Dependencies

- **@noble/ed25519**: Primary cryptographic library (install via `npm install @noble/ed25519`)
- **Reference Implementation**: See [PKM-1-1-noble-ed25519-guide.md](./PKM-1-1-noble-ed25519-guide.md) for React integration patterns

## Files Modified

- `src/datafold_node/static-react/src/components/KeyGenerationComponent.jsx` (created)
- `src/datafold_node/static-react/src/components/tabs/KeyManagementTab.jsx` (created)
- `src/datafold_node/static-react/src/hooks/useKeyGeneration.ts` (created)
- `src/datafold_node/static-react/src/types/cryptography.ts` (created)  
- `src/datafold_node/static-react/src/utils/ed25519.ts` (created)
- `src/datafold_node/static-react/package.json` (added @noble/ed25519 dependency)
- `src/datafold_node/static-react/src/App.jsx` (added Keys tab integration)
- `src/datafold_node/static-react/src/test/components/KeyGenerationComponent.test.jsx` (created)