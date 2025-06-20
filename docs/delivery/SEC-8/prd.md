# PBI-SEC-8: Implement Public Key Persistence

[View in Backlog](../backlog.md#user-content-pbi-sec-8-implement-public-key-persistence)

## Overview

This PBI addresses a critical security gap where Ed25519 public keys are only stored in memory and are lost when the node restarts. This prevents reliable authentication across node restarts and makes the system unsuitable for production deployment.

## Problem Statement

Currently, the `MessageVerifier` stores all registered public keys in an in-memory `HashMap`:

```rust
/// Registered public keys
public_keys: Arc<RwLock<HashMap<String, PublicKeyInfo>>>,
```

This creates several problems:
- All registered public keys are lost on node restart
- Clients must re-register keys after every restart
- No persistent authentication state across sessions
- Production deployment is not viable

## User Stories

- As a **node operator**, I want public keys to persist across restarts so that I don't lose authentication state
- As a **client developer**, I want my registered keys to remain valid after node restarts so that I don't need to re-register
- As a **system administrator**, I want reliable authentication persistence for production deployments

## Technical Approach

### Database Schema
- Add `public_keys` tree to `DbOperations` 
- Store keys using format: `key_id -> PublicKeyInfo`
- Leverage existing sled database infrastructure

### Implementation Strategy
1. **Extend DbOperations**: Add public key storage/retrieval methods
2. **Update MessageVerifier**: Load persisted keys on startup, save on registration
3. **Migration Support**: Handle existing in-memory keys during upgrade
4. **Maintain Compatibility**: All existing public key operations continue working

### Key Components
- `src/db_operations/public_key_operations.rs` - New database operations
- `src/security/signing.rs` - Update MessageVerifier persistence
- Migration utilities for existing deployments

## UX/UI Considerations

- No user-facing changes required
- All existing APIs continue to work
- Transparent persistence for end users

## Acceptance Criteria

- [ ] Public keys are stored in the sled database using `DbOperations`
- [ ] `MessageVerifier` loads persisted keys on startup
- [ ] Key registration persists keys immediately to database
- [ ] All existing in-memory public key operations continue to work
- [ ] Node restart preserves all registered public keys
- [ ] Migration path exists for existing in-memory keys

## Dependencies

- Existing `DbOperations` infrastructure
- Current `MessageVerifier` implementation
- Sled database integration

## Open Questions

- Should we implement key expiration cleanup during startup?
- How should we handle corrupted key data in the database?
- Should we add key usage tracking/metrics?

## Related Tasks

- [SEC-8-1: Add Public Key Database Operations](./SEC-8-1.md)
- [SEC-8-2: Update MessageVerifier for Persistence](./SEC-8-2.md) 
- [SEC-8-3: Add Migration for Existing Keys](./SEC-8-3.md)
- [SEC-8-4: Integration Tests for Key Persistence](./SEC-8-4.md)