# PBIs for Centralized Security Architecture Implementation

## Epic: Centralized Security Layer

---

### PBI 1: Implement Signature Verification Middleware

- **Description**: Create a single Actix middleware that verifies Ed25519 signatures on all incoming authenticated API requests.
- **Acceptance Criteria**:
  - Middleware rejects requests with missing or invalid signatures.
  - Verified identity is attached to request context.
  - All business logic assumes requests are already authenticated.

---

### PBI 2: Implement Central Encryption/Decryption Layer

- **Description**: Create a single encryption/decryption module (e.g., `EncryptedDb` or `EncryptedStore`) that wraps all database access for sensitive data.
- **Acceptance Criteria**:
  - All writes to the database are encrypted before storage.
  - All reads from the database are decrypted before use.
  - No encryption logic exists outside this module.

---

### PBI 3: Refactor API Endpoints to Use Security Layers

- **Description**: Update all API endpoints that require authentication or encryption to use the new middleware and encryption layer.
- **Acceptance Criteria**:
  - Endpoints requiring authentication use the signature verification middleware.
  - Endpoints storing or retrieving sensitive data use the encryption layer.
  - Non-sensitive endpoints bypass these layers as appropriate.

---

### PBI 4: Integration Tests for Security Boundaries

- **Description**: Write integration tests to verify that:
  - Requests without valid signatures are rejected.
  - Data is always encrypted at rest and decrypted on access.
- **Acceptance Criteria**:
  - Tests cover both positive and negative cases for authentication and encryption.
  - Tests are automated and run in CI.

---

### PBI 5: Documentation and Developer Guide

- **Description**: Document the security boundary, usage of the middleware, and the encryption layer.
- **Acceptance Criteria**:
  - Developer guide explains how to use and extend the security layers.
  - Security boundary is clearly defined for future contributors.

---

### PBI 6: Migration Plan for Existing Functionality

- **Description**: Plan and execute the migration of existing business logic and database access to use the new centralized security layers.
- **Acceptance Criteria**:
  - All sensitive operations are routed through the new layers.
  - No legacy security logic remains in business logic or storage code.

---

### PBI 7: Performance and Audit Logging

- **Description**: Add logging and monitoring to the security layers for auditing and performance analysis.
- **Acceptance Criteria**:
  - All authentication failures and encryption errors are logged.
  - Performance metrics are available for signature verification and encryption operations.

---

These PBIs will ensure a maintainable, auditable, and robust centralized security architecture for the project.