# System-Wide Public Key Management Proposal for Datafold

## Overview

This proposal describes a minimal, secure, and user-friendly system for managing a single, system-wide Ed25519 public key in Datafold. This key is used to verify all signed requests to the node. **CRITICAL SECURITY PRINCIPLE: The private key is handled exclusively on the client-side and is NEVER persisted or transmitted to the backend.** The system supports three essential flows:

1.  **Generate/Import Key**: An administrator generates or imports an Ed25519 key pair. The public key is registered with the Datafold node as the single, system-wide verification key.
2.  **Sign Requests**: Clients use the corresponding private key to sign all API requests.
3.  **Verify Signatures**: The Datafold node uses the stored system-wide public key to verify the signature on every incoming request.

This single-key model simplifies key management by removing the need to handle multiple keys for different users or services, while still ensuring the integrity and authenticity of requests.

## Existing System Components

Before implementing new functionality, it's important to catalog the existing infrastructure that can be leveraged:

### Core Security Infrastructure
- **Ed25519 Keypair Generation**: [`src/security/keys.rs`](src/security/keys.rs:18) - Contains cryptographic key generation capabilities
- **Security Routes**: [`src/datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1) - HTTP endpoints for security operations
- **Encryption Capabilities**: [`src/security/encryption.rs`](src/security/encryption.rs:1) - Existing encryption/decryption functionality
- **Signing and Verification**: [`src/security/signing.rs`](src/security/signing.rs:1) - Digital signature capabilities, adapted for a single system key.
- **Audit Logging**: [`src/security/audit.rs`](src/security/audit.rs:1) - Security event logging system

### Database and Storage Infrastructure
- **Database Initialization**: [`src/fold_db_core/infrastructure/init.rs`](src/fold_db_core/infrastructure/init.rs:1) - Database setup and configuration
- **Schema Operations**: [`src/db_operations/schema_operations.rs`](src/db_operations/schema_operations.rs) - Database schema management
- **Public Key Operations**: [`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs) - Public key storage, refactored to enforce a single key.

### HTTP Server Infrastructure
- **HTTP Server Endpoints**: [`src/datafold_node/http_server.rs`](src/datafold_node/http_server.rs:1) - Existing HTTP server framework
- **System Routes**: [`src/datafold_node/system_routes.rs`](src/datafold_node/system_routes.rs) - System management endpoints
- **Query Routes**: [`src/datafold_node/query_routes.rs`](src/datafold_node/query_routes.rs) - Data query endpoints

## Architecture

The system leverages existing Datafold infrastructure with a **client-side cryptographic architecture** and a **single server-side public key**.

-   **Client-Side**: Responsible for holding the private key and signing all requests.
-   **Server-Side**:
    -   **Simplified Storage**: Utilizes the existing database infrastructure ([`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs)) but is refactored to store only a **single system-wide public key**.
    -   **Simplified API**: Exposes a minimal set of endpoints via [`src/datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1) for managing this single key.

## UI Workflow and User Experience

Management of the system-wide key is intended for administrators.

### Key Management UI
A dedicated section in the admin UI allows an administrator to:
-   **Register Key**: Provide a public key to be used as the system-wide verification key. This action overwrites any existing key.
-   **View Key**: Display the current system-wide public key and its metadata.
-   **Delete Key**: Remove the system-wide public key. After deletion, signature verification will fail for all requests until a new key is registered.

## Security Principles

### Private Key Handling
**ZERO SERVER-SIDE PRIVATE KEY EXPOSURE:** The private key corresponding to the system-wide public key is never stored or transmitted to the server. It is the responsibility of the administrator to securely manage the private key.

### Key Flow
-   **Key Registration**: An administrator provides a new public key via the UI or API. The backend stores this as the single system-wide key, identified by the constant `SYSTEM_WIDE_PUBLIC_KEY`. Any previously stored key is overwritten.
-   **Signing Operations**: A client (e.g., another service, a script) uses the private key to sign a payload. The resulting `SignedMessage` is sent to the Datafold node. The `public_key_id` in the message is ignored by the server, as it will always use the system-wide key for verification.
-   **Signature Verification**: The Datafold node receives a `SignedMessage`. It retrieves the single system-wide public key from storage and uses it to verify the message's signature. If the key does not exist or the signature is invalid, the request is rejected.

## Technical Implementation Plan

The backend has been refactored to support the single-key model.

### Backend Implementation
- **Constants**: A new constant `SINGLE_PUBLIC_KEY_ID` has been added to `src/constants.rs` to serve as the database key for the system-wide public key.
- **Database Operations (`src/db_operations/public_key_operations.rs`)**:
    -   Functions have been refactored to manage a single key. `store_public_key` is now `set_system_public_key`, which clears all other keys and stores the new one under the constant ID.
    -   Functions for listing or getting keys by a variable ID have been removed or replaced with functions like `get_system_public_key`.
- **Signing Logic (`src/security/signing.rs`)**:
    -   `MessageVerifier` has been updated to cache and verify against only the single system-wide public key.
    -   It ignores the `public_key_id` field in incoming `SignedMessage` objects during verification.
- **API Endpoints (`src/datafold_node/security_routes.rs` and `src/datafold_node/http_server.rs`)**:
    -   The API for key management has been simplified. The old routes (`/api/security/keys/...`) have been replaced with a single endpoint:
    -   `/api/security/system-key`:
        -   `POST`: Registers or updates the system-wide public key.
        -   `GET`: Retrieves the current system-wide public key.
        -   `DELETE`: Removes the system-wide public key.

### Frontend Development
-   **React UI Components**: A new component, `SecurityKeyManager.jsx`, has been created in `ui/src/components/` to manage the system-wide security key. This component handles all interactions with the `/api/security/system-key` endpoint, including registering, viewing, and deleting the key. It is ready to be integrated into the main application UI.

---

*Prepared by Datafold Engineering – 2025-06-20*
*Updated to reflect single-key architecture.*