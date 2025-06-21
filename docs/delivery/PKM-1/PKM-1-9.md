# PKM-1-9: Integrate Signature Verification into Mutation Endpoint

[Back to task list](./tasks.md)

## Description

This task involves modifying the backend's `/api/data/mutate` endpoint to require and verify an Ed25519 signature for all incoming data mutations. This is a critical security enhancement to ensure that only authorized users can modify data. The endpoint will be updated to accept a `SignedMessage` object, validate the signature against the payload, and then process the mutation only if the signature is valid.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2024-07-22 21:00:00 | Created | N/A | Proposed | Task created to address security gap in mutation endpoint. | AI_Agent |
| 2024-07-22 22:00:00 | Status Change | Proposed | InProgress | Starting implementation. | AI_Agent |
| 2024-07-22 22:15:00 | Status Change | InProgress | Review | Implementation complete, ready for review. | AI_Agent |

## Requirements

1. **Update `data_mutate` Handler**: Modify the function signature to accept a `SignedMessage` payload.
2. **Signature Verification**: Before processing the mutation, call the `SigningManager` to verify the signature.
3. **Payload Extraction**: If verification succeeds, decode the Base64 payload to get the actual mutation data.
4. **Error Handling**: Return a clear `401 Unauthorized` or `403 Forbidden` error if signature verification fails.
5. **Client-Side Update**: Modify the `MutationTab.jsx` component to sign the mutation payload before sending it.

## Implementation Plan

1. **Backend (`data_mutate` in `http_server.rs`):**
   - Change the function to accept `web::Json<SignedMessage>`. (Note: `SignedMessage` struct will need to be defined in Rust to match the TypeScript version).
   - Inject the `SigningManager`.
   - Call `signing_manager.verify_signature()`.
   - If successful, decode the payload from Base64 and deserialize it into the mutation `Value`.
   - Proceed with the existing mutation logic using the decoded payload.
   - If verification fails, return an appropriate HTTP error.

2. **Frontend (`MutationTab.jsx`):**
   - Integrate the `useKeyGeneration`, `useSigning`, and `useSecurityAPI` hooks (or a new, more specific hook).
   - Before fetching, use the `signPayload` function to create the `SignedMessage` object.
   - Send the `SignedMessage` object as the body of the request to `/api/data/mutate`.
   - Update UI to handle potential signature errors returned from the backend.

## Test Plan
- **Unit Test (Backend)**: Add a test case to ensure a request with an invalid signature to `data_mutate` is rejected.
- **Unit Test (Backend)**: Add a test case to ensure a request with a valid signature is processed correctly.
- **Integration Test**: Create a test that simulates the full flow from the client: signing a mutation, sending it, and verifying that the data is correctly mutated in the database.
- **Manual Test**: Use the UI to perform a signed mutation and verify the outcome.

## Files Modified
- `src/datafold_node/http_server.rs`
- `src/datafold_node/static-react/src/components/tabs/MutationTab.jsx`
- `src/security/signing.rs` (if any changes are needed to `SigningManager`)
- `tests/integration/api_security_integration_test.rs` (or similar) 