# Private Key UI Management Proposal for Datafold

## Overview

This proposal describes a minimal, secure, and user-friendly system for managing Ed25519 private keys in Datafold. **CRITICAL SECURITY PRINCIPLE: Private keys are generated or imported client-side and may be stored temporarily in memory (including React stores) but are NEVER persisted or transmitted to the backend.** The system supports four essential flows:

1. **Generate Key**: Users generate Ed25519 keys locally in the browser using deterministic derivation from identity. Private keys can be held temporarily in memory/React state.
2. **Import Key**: Users import existing Ed25519 private keys through secure input validation and format conversion. Private keys remain client-side only.
3. **Sign Requests**: UI generates signatures client-side using private keys from memory. Only signatures and public keys are sent to the backend.
4. **Verify Signatures**: Backend verifies signatures using stored public keys. No private key material ever reaches the server.

## Existing System Components

Before implementing new functionality, it's important to catalog the existing infrastructure that can be leveraged:

### Core Security Infrastructure
- **Ed25519 Keypair Generation**: [`src/security/keys.rs`](src/security/keys.rs:18) - Contains cryptographic key generation capabilities
- **Security Routes**: [`src/datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1) - HTTP endpoints for security operations
- **Encryption Capabilities**: [`src/security/encryption.rs`](src/security/encryption.rs:1) - Existing encryption/decryption functionality
- **Signing and Verification**: [`src/security/signing.rs`](src/security/signing.rs:1) - Digital signature capabilities
- **Audit Logging**: [`src/security/audit.rs`](src/security/audit.rs:1) - Security event logging system

### Database and Storage Infrastructure
- **Database Initialization**: [`src/fold_db_core/infrastructure/init.rs`](src/fold_db_core/infrastructure/init.rs:1) - Database setup and configuration
- **Schema Operations**: [`src/db_operations/schema_operations.rs`](src/db_operations/schema_operations.rs) - Database schema management
- **Public Key Operations**: [`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs) - Public key storage and retrieval

### HTTP Server Infrastructure
- **HTTP Server Endpoints**: [`src/datafold_node/http_server.rs`](src/datafold_node/http_server.rs:1) - Existing HTTP server framework
- **System Routes**: [`src/datafold_node/system_routes.rs`](src/datafold_node/system_routes.rs) - System management endpoints
- **Query Routes**: [`src/datafold_node/query_routes.rs`](src/datafold_node/query_routes.rs) - Data query endpoints

### Permission and Configuration Management
- **Permission Management System**: [`src/permissions/permission_manager.rs`](src/permissions/permission_manager.rs) - User permission handling
- **Configuration Management**: [`src/datafold_node/config.rs`](src/datafold_node/config.rs) - System configuration
- **React UI Infrastructure**: Existing frontend framework for user interactions

### Logging and Monitoring
- **Structured Logging**: [`src/logging/outputs/structured.rs`](src/logging/outputs/structured.rs) - Comprehensive logging system
- **Event Monitoring**: [`src/fold_db_core/infrastructure/event_monitor.rs`](src/fold_db_core/infrastructure/event_monitor.rs) - System event tracking

## Architecture

The system leverages existing Datafold infrastructure with a **client-side cryptographic architecture**:

- **Enhanced React UI**: Extends the existing React UI infrastructure to handle client-side key generation and signing operations. **Private keys may be stored temporarily in React state/memory for cryptographic operations but are NEVER persisted to localStorage, sessionStorage, cookies, or transmitted to the backend.**
- **Extended HTTP API**: Builds upon the existing HTTP server framework ([`src/datafold_node/http_server.rs`](src/datafold_node/http_server.rs:1)) and security routes ([`src/datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1)) to expose endpoints for signature verification, public key registration, and encrypted data storage.
- **Enhanced Storage**: Utilizes existing database infrastructure ([`src/fold_db_core/infrastructure/init.rs`](src/fold_db_core/infrastructure/init.rs:1)) and public key operations ([`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs)) to store public keys, key metadata, and encrypted blobs. **No private key material is ever stored server-side.**

## UI Workflow and User Experience

### Private Key Storage Security

**TEMPORARY MEMORY STORAGE ALLOWED:**
- **Private keys may be stored temporarily in React state/memory** - for the duration of user sessions to enable cryptographic operations
- **Private keys are NEVER persisted to browser storage** - no localStorage, sessionStorage, IndexedDB, WebSQL, or cookies
- **Private keys are NEVER transmitted to the server** - no API endpoint ever receives private key material
- **Only public keys and signatures are sent to the UI/server** - clients send only cryptographic outputs, never secret inputs

### Secure Workflow Design
1. **Key Generation Flow**: User requests key generation → Browser generates Ed25519 keypair → Private key stored temporarily in React state → Public key sent to server for registration
2. **Key Import Flow**: User selects import option → Input field accepts private key in supported formats → Client-side validation and format conversion → Private key stored temporarily in React state → Public key derived and sent to server for registration
3. **Signing Flow**: User initiates signing → Private key retrieved from React state → Signature generated client-side → Only signature sent to server
4. **Session Management**: Private keys cleared from memory when user logs out or session expires

### Key Management UI Options

The interface provides users with two primary options for key management:

#### Create New Key
- **One-click generation**: Simple button to generate new Ed25519 keypair
- **Deterministic derivation**: Keys derived from user identity for reproducibility
- **Secure randomness**: Uses cryptographically secure random number generation
- **Immediate availability**: Generated keys ready for use in current session

#### Import Existing Key
- **Multiple format support**: Accepts common Ed25519 private key formats (hex, base64, PEM)
- **Real-time validation**: Immediate feedback on key format and validity
- **Format conversion**: Automatic conversion to internal Ed25519 format
- **Security warnings**: Clear messaging about private key handling and security
- **Error handling**: User-friendly error messages for invalid or malformed keys

### Import Security Considerations

**Private Key Import Validation:**
- **Format validation**: Verify input matches supported Ed25519 private key formats
- **Key validation**: Cryptographic verification that imported key is valid Ed25519 private key
- **Length validation**: Ensure key meets Ed25519 32-byte private key requirements
- **Public key derivation**: Derive public key from imported private key for verification
- **Error sanitization**: Prevent exposure of private key material in error messages
- **Input clearing**: Immediate clearing of input fields after successful import

## Security Principles

### Private Key Storage Security

**ZERO SERVER-SIDE PRIVATE KEY EXPOSURE:**
- **Private keys are NEVER stored on the server** - all private key material remains client-side only
- **Private keys are NEVER transmitted to the server** - no API endpoint ever receives private key data
- **Only public keys and key IDs are sent to the server** - server receives only metadata needed for verification
- **Private keys may exist temporarily in browser memory/React state** - acceptable for cryptographic operations during user sessions

### Clear Separation of Concerns
- **Client-side**: Key generation, temporary storage in memory, signing operations
- **Server-side**: Public key storage, signature verification, metadata management
- **Network boundary**: Only public keys, signatures, and metadata cross the client-server boundary

### Key Flows (Client-Side Cryptographic Operations)
- **Key Generation**: User authenticates → Browser generates Ed25519 keypair using existing capabilities → Private key stored in React state → Public key registered with server using existing public key operations ([`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs))
- **Key Import**: User authenticates → User inputs existing private key → Client-side validation and format conversion → Private key stored in React state → Public key derived and registered with server using existing public key operations ([`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs))
- **Signing Operations**: User initiates action → Private key retrieved from React state → Signature generated client-side → Server verifies signature using existing infrastructure ([`src/security/signing.rs`](src/security/signing.rs:1))
- **Session Security**: Private keys cleared from React state on logout/session expiry → No persistent client-side storage of cryptographic material

## Integration Requirements

### Existing Infrastructure (Already Available)
- ✅ **HTTP Server Framework**: [`src/datafold_node/http_server.rs`](src/datafold_node/http_server.rs:1) - Ready for new endpoints
- ✅ **Security Routes Foundation**: [`src/datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1) - Base security endpoint structure
- ✅ **Cryptographic Operations**: [`src/security/keys.rs`](src/security/keys.rs:18), [`src/security/signing.rs`](src/security/signing.rs:1), [`src/security/encryption.rs`](src/security/encryption.rs:1) - Core crypto functionality
- ✅ **Database Operations**: [`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs) - Public key storage
- ✅ **Permission System**: [`src/permissions/permission_manager.rs`](src/permissions/permission_manager.rs) - User authorization
- ✅ **Audit Logging**: [`src/security/audit.rs`](src/security/audit.rs:1) - Security event tracking

### New Components Required
- 🔨 **Enhanced React UI Components**: Key management interface with generation and import options, temporary key storage in React state, signing request interface
- 🔨 **New API Endpoints**: Integration with existing security routes for public key registration and signature verification
- 🔨 **Database Schema Extensions**: Public key and metadata storage tables (using existing infrastructure)
- 🔨 **Client-Side Cryptography**: Browser-based key generation, import validation, and signing libraries with React state integration
- 🔨 **Key Import Validation**: Format detection, cryptographic validation, and secure error handling for imported private keys

## Technical Implementation Plan

### Phase 1: Backend Extensions (Leveraging Existing Infrastructure)
- **Extend Security Routes**: Add new endpoints to existing [`src/datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1):
  - `POST /api/v1/keys/register` (register public key)
  - `POST /api/v1/keys/verify` (verify signature)
  - `POST /api/v1/data/encrypt` and `/decrypt` (for optional encrypted storage)
- **Enhance Database Schema**: Extend existing public key operations ([`src/db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs)) for key metadata
- **Integrate with Existing Crypto**: Utilize existing signing/verification ([`src/security/signing.rs`](src/security/signing.rs:1)) and encryption ([`src/security/encryption.rs`](src/security/encryption.rs:1)) capabilities
- **Audit Integration**: Leverage existing audit logging ([`src/security/audit.rs`](src/security/audit.rs:1)) for key management events

### Phase 2: Frontend Development (New Components)
- **Build React UI**: Extend existing React infrastructure for key generation and import, temporary key storage in React state, and metadata management
- **Client-Side Cryptography**: Implement browser-based Ed25519 key generation, import validation, and signing with React state integration
- **Key Import System**: Build secure private key input validation, format conversion, and error handling components
- **Integration with Backend**: Connect to extended security routes for public key registration and signature verification

### Phase 3: Testing and Integration
- **Security Testing**: Validate integration with existing security infrastructure
- **Permission Testing**: Ensure compatibility with existing permission system ([`src/permissions/permission_manager.rs`](src/permissions/permission_manager.rs))
- **End-to-End Testing**: Test complete flows using existing HTTP server framework

### Phase 4: Launch
- **User Testing**: Validate UI/UX within existing Datafold interface
- **Documentation**: Key management and recovery best practices
- **Production Deployment**: Leverage existing deployment infrastructure
- **Monitoring**: Utilize existing logging and monitoring systems ([`src/logging/outputs/structured.rs`](src/logging/outputs/structured.rs))

**Critical Principle**: No backend storage or handling of private keys at any point - this maintains compatibility with existing security architecture.

## Security Principles

### Core Security Tenets (Building on Existing Infrastructure)
- **No server-side private key storage**: Private keys never reach the server and are never stored or transmitted to the backend. This principle is enforced by existing security architecture and audit logging ([`src/security/audit.rs`](src/security/audit.rs:1)).
- **Temporary client-side storage only**: Private keys may exist temporarily in browser memory/React state for cryptographic operations but are never persisted. Leverages existing permission management system ([`src/permissions/permission_manager.rs`](src/permissions/permission_manager.rs)) for session authorization.
- **Deterministic, identity-based key derivation**: All keys are derived from user identity in a reproducible, secure way using existing cryptographic capabilities ([`src/security/keys.rs`](src/security/keys.rs:18)), minimizing risk of key loss or duplication.

### Risk Assessment

**Client-Side Key Exposure Mitigation:**
- **Temporary memory storage risk**: Private keys in React state are cleared on logout/session expiry, preventing long-term exposure
- **Browser security boundary**: Relies on browser memory isolation and HTTPS for protection during temporary storage
- **No persistent storage risk**: Elimination of localStorage/cookie storage prevents key persistence across sessions
- **Network transmission risk**: Complete elimination through client-side-only private key operations

### Integration with Existing Security Framework
- **Cryptographic Operations**: Client-side signing utilizes browser-compatible implementations, server-side verification uses existing infrastructure in [`src/security/signing.rs`](src/security/signing.rs:1) and [`src/security/encryption.rs`](src/security/encryption.rs:1).
- **Audit Trail**: All key management activities are logged through existing audit infrastructure ([`src/security/audit.rs`](src/security/audit.rs:1)) for security monitoring and compliance.
- **Permission Enforcement**: Key operations respect existing permission boundaries managed by [`src/permissions/permission_manager.rs`](src/permissions/permission_manager.rs).

## Implementation Requirements

The implementation should be created through a proper [Product Backlog](docs/delivery/backlog.md) following the project's [`.cursorrules`](.cursorrules:94) guidelines. Since the backend already provides comprehensive security infrastructure, implementation should focus on React UI components and client-side cryptographic integration.

**Key Implementation Focus:**
- **React UI Components**: Key management interface with generation and import options leveraging existing infrastructure
- **Client-Side Cryptography**: Browser-based Ed25519 key generation, import validation, and signing
- **Private Key Import**: Secure validation and format conversion for existing Ed25519 private keys
- **API Integration**: Connection to existing [`security_routes.rs`](src/datafold_node/security_routes.rs:1) endpoints
- **Session Management**: Secure temporary key storage in React state

**Existing Infrastructure to Leverage:**
- [`security/keys.rs`](src/security/keys.rs:18) - Ed25519 keypair generation
- [`security/signing.rs`](src/security/signing.rs:1) - Signature verification infrastructure
- [`db_operations/public_key_operations.rs`](src/db_operations/public_key_operations.rs) - Public key storage
- [`datafold_node/security_routes.rs`](src/datafold_node/security_routes.rs:1) - Security API endpoints
- [`security/audit.rs`](src/security/audit.rs:1) - Security event logging

---

*Prepared by Datafold Engineering – 2025-06-20*