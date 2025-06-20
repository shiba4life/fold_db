# Product Backlog

> This backlog contains all Product Backlog Items (PBIs) for the project, ordered by priority.  
> **Location:** docs/delivery/backlog.md  
> **Policy:** Maintained in accordance with [.cursorrules](../../.cursorrules) and project policy.  
> **Purpose:** The backlog is the single source of truth for all PBIs, each of which must be explicitly linked to tasks and implementation plans.

---

## Structure

Each PBI must include:
- **PBI ID**: Unique identifier (e.g., PBI-SEC-1)
- **Title**: Short, descriptive name
- **Description**: What the PBI is and why it matters
- **Acceptance Criteria**: Bullet list of what must be true for the PBI to be considered complete
- **Linked Plan**: Reference to the implementation plan or proposal
- **Associated Tasks**: (If applicable) List of task IDs or links

---

## PBIs

### PBI-SEC-1: Implement Signature Verification Middleware

**Description:**  
Create a single Actix middleware that verifies Ed25519 signatures on all incoming authenticated API requests.

**Acceptance Criteria:**
- Middleware rejects requests with missing or invalid signatures.
- Verified identity is attached to request context.
- All business logic assumes requests are already authenticated.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

### PBI-SEC-2: Implement Central Encryption/Decryption Layer

**Description:**  
Create a single encryption/decryption module (e.g., `EncryptedDb` or `EncryptedStore`) that wraps all database access for sensitive data.

**Acceptance Criteria:**
- All writes to the database are encrypted before storage.
- All reads from the database are decrypted before use.
- No encryption logic exists outside this module.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

### PBI-SEC-3: Refactor API Endpoints to Use Security Layers

**Description:**  
Update all API endpoints that require authentication or encryption to use the new middleware and encryption layer.

**Acceptance Criteria:**
- Endpoints requiring authentication use the signature verification middleware.
- Endpoints storing or retrieving sensitive data use the encryption layer.
- Non-sensitive endpoints bypass these layers as appropriate.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

### PBI-SEC-4: Integration Tests for Security Boundaries

**Description:**  
Write integration tests to verify that:
- Requests without valid signatures are rejected.
- Data is always encrypted at rest and decrypted on access.

**Acceptance Criteria:**
- Tests cover both positive and negative cases for authentication and encryption.
- Tests are automated and run in CI.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

### PBI-SEC-5: Documentation and Developer Guide

**Description:**  
Document the security boundary, usage of the middleware, and the encryption layer.

**Acceptance Criteria:**
- Developer guide explains how to use and extend the security layers.
- Security boundary is clearly defined for future contributors.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

### PBI-SEC-6: Migration Plan for Existing Functionality

**Description:**  
Plan and execute the migration of existing business logic and database access to use the new centralized security layers.

**Acceptance Criteria:**
- All sensitive operations are routed through the new layers.
- No legacy security logic remains in business logic or storage code.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

### PBI-SEC-7: Performance and Audit Logging

**Description:**  
Add logging and monitoring to the security layers for auditing and performance analysis.

**Acceptance Criteria:**
- All authentication failures and encryption errors are logged.
- Performance metrics are available for signature verification and encryption operations.

**Linked Plan:** [Centralized Security Architecture 2025-06-20](../proposals/centralized_security_architecture_2025-06-20.md)

---

*All PBIs must be explicitly linked to tasks and implementation plans. This backlog is the authoritative source for all work in the project.*