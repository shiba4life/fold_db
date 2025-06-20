# PKM-1-5: Implement secure session management

[Back to task list](./tasks.md)

## Description

Add private key lifecycle management with automatic cleanup on logout/session expiry. This critical security task ensures private keys are properly managed in React state with zero persistence and automatic cleanup when sessions end.

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