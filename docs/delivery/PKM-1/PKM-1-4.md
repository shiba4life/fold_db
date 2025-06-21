# PKM-1-4: Integrate with existing security routes

[Back to task list](./tasks.md)

## Description

Connect the React UI components to the existing backend security endpoints that handle **signature verification**. The integration for public key registration was completed in task [PKM-1-2](./PKM-1-2.md). This task will focus on sending signed messages to protected endpoints and handling the responses.

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

**Note**: Integration with the `/api/security/register-key` endpoint is already complete. This task will focus on endpoints that require a valid signature for access.

1.  **API Client**:
    *   Create a utility or hook for making authenticated API requests.
    *   This function will take a request payload, sign it using the client-side private key (from `useKeyGeneration` state), and structure it according to the `SignedMessage` format expected by the backend.
2.  **Endpoint Integration**:
    *   Identify a protected backend endpoint (e.g., a test endpoint like `/api/security/protected-endpoint`).
    *   Use the authenticated API client to send a request to this endpoint.
3.  **UI Feedback**:
    *   Update the UI to display the result of the signed request (success or failure).
    *   Provide clear error messages if signature verification fails.

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