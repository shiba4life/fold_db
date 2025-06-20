# PKM-1-4: Integrate with existing security routes

[Back to task list](./tasks.md)

## Description

Connect React components to existing backend security endpoints for signature verification and public key management. This task leverages the comprehensive security infrastructure already implemented in [`security_routes.rs`](../../src/datafold_node/security_routes.rs) and related backend systems.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 16:43:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

1. **API Client Integration**: Connect React components to existing security endpoints
2. **Public Key Registration**: Use existing public key storage infrastructure
3. **Signature Verification**: Integrate with existing signature verification system
4. **Authentication Flow**: Implement proper authentication for security operations
5. **Error Handling**: Handle API errors and network failures gracefully
6. **Audit Integration**: Ensure operations are logged through existing audit system

## Implementation Plan

1. **API Client Setup**:
   - Create API client for security route endpoints
   - Implement request/response type definitions
   - Add authentication headers and CSRF protection

2. **Public Key Operations**:
   - Integrate with existing [`public_key_operations.rs`](../../src/db_operations/public_key_operations.rs)
   - Implement public key registration via security routes
   - Add public key retrieval and listing functionality

3. **Signature Operations**:
   - Connect to existing signature verification in [`signing.rs`](../../src/security/signing.rs)
   - Implement signature submission and verification flow
   - Add signature format compatibility layer

4. **Audit and Logging**:
   - Ensure operations are logged via [`audit.rs`](../../src/security/audit.rs)
   - Add client-side logging for debugging
   - Implement security event tracking

## Verification

- [ ] React components successfully call existing security route endpoints
- [ ] Public keys properly registered using existing infrastructure
- [ ] Signatures verified through existing verification system
- [ ] API errors handled gracefully with user feedback
- [ ] All operations logged through existing audit system
- [ ] Authentication flow works with existing session management
- [ ] Integration tests verify end-to-end connectivity
- [ ] Performance meets UI responsiveness requirements

## Files Modified

- `src/datafold_node/static-react/api/securityClient.ts` (to be created)
- `src/datafold_node/static-react/types/api.ts` (to be created)
- `src/datafold_node/static-react/hooks/useSecurityAPI.ts` (to be created)
- Related test files (to be created)