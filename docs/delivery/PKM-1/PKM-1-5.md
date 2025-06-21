# PKM-1-5: Implement secure session management

[Back to task list](./tasks.md)

## Description

This task focuses on the **automated** lifecycle management of the client-side private key. The goal is to ensure the key is automatically cleared from memory when it's no longer needed, such as on session expiry or user logout, to minimize its exposure.

## Status History

| Timestamp | Event Type | From Status | To Status | Details | User |
|-----------|------------|-------------|-----------|---------|------|
| 2025-06-20 16:43:00 | Created | N/A | Proposed | Task file created | User |

## Requirements

1. **Private Key Lifecycle**: Manage private key creation, storage, and destruction
2. **Automatic Cleanup**: Clear private keys on logout, session expiry, and tab close
3. **Memory Security**: Ensure zero persistence to localStorage, sessionStorage, or cookies
4. **Session Integration**: Work with existing Datafold session management
5. **Security Boundaries**: Validate no private key leakage outside React state
6. **User Feedback**: Clear indication of key status and security state

## Implementation Plan

**Note**: A manual `clearKeys` function was implemented in [PKM-1-2](./PKM-1-2.md). This task will build on that by creating an automated cleanup process.

1. **Lifecycle Management**:
   - Create `useKeyLifecycle()` hook for key state management
   - Implement automatic key generation on session start
   - Add key destruction on session end

2. **Cleanup Mechanisms**:
   - Add `beforeunload` event handlers for tab/window close
   - Implement logout cleanup integration
   - Add session timeout cleanup
   - Create memory clearing utilities

3. **Security Validation**:
   - Add runtime checks for storage persistence
   - Implement key state validation
   - Add security boundary testing

4. **User Interface**:
   - Create key status indicators
   - Add security messaging about temporary storage
   - Implement key regeneration controls

5. **Session Integration**:
   - Hook into the application's session management or authentication context.
   - Identify the events that signify the end of a user session (e.g., logout action, session cookie expiry).

6. **Automated Cleanup**:
   - When a session-ending event is detected, automatically call the `clearKeys` function from the `useKeyGeneration` hook/`KeyGenerationComponent` state.

7. **UI State Reset**:
   - Ensure that when keys are cleared, the `KeyManagementTab` and any other related components are reset to their initial state.

## Verification

- [ ] Private keys automatically cleared on logout
- [ ] Keys cleared on session timeout/expiry
- [ ] Keys cleared on tab/window close
- [ ] Zero persistence to browser storage verified
- [ ] Integration with existing session management works
- [ ] User receives clear feedback about key security
- [ ] Memory clearing functions work correctly
- [ ] Security boundary validation passes all tests

## Files Modified

- `src/datafold_node/static-react/hooks/useKeyLifecycle.ts` (to be created)
- `src/datafold_node/static-react/utils/sessionSecurity.ts` (to be created)
- `src/datafold_node/static-react/components/KeyStatusIndicator.tsx` (to be created)
- Related test files (to be created)