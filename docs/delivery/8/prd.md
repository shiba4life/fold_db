# PBI-8: Database Master Key Encryption

[View in Backlog](../backlog.md#user-content-8)

## Overview

This PBI implements cryptographic initialization for DataFold databases, establishing a master Ed25519 key pair that serves as the foundation for all database-level security operations. The implementation ensures that databases are secured from the moment of creation.

## Problem Statement

Currently, DataFold databases are initialized without any cryptographic protection at the database level. While the system has permission management and network-level security, there is no master cryptographic identity for database instances themselves. This creates security gaps where:

- Database files have no protection against physical access
- No cryptographic foundation exists for advanced security features
- Database initialization doesn't establish cryptographic identity
- No master key for deriving encryption keys for data at rest

## User Stories

**Primary User Story:**
As a database administrator, I want to initialize DataFold databases with master key encryption so that all data stored is protected by cryptographic security from the start.

**Supporting User Stories:**
- As a security officer, I want databases to have cryptographic identities for audit and compliance
- As a system operator, I want secure key derivation from user-controlled passphrases
- As a backup administrator, I want master keys that enable secure backup/restore operations

## Technical Approach

### 1. Master Key Generation
- Generate Ed25519 key pairs during database initialization
- Use secure random number generation for key material
- Implement key derivation from user passphrase using Argon2id
- Store public key for verification, never store private key in plaintext

### 2. Database Initialization Enhancement
- Extend `NodeConfig` to include cryptographic initialization
- Add passphrase-based key derivation during setup
- Create secure storage layer during initialization
- Integrate with existing database creation workflow

### 3. Key Storage and Management
- Store master public key in database metadata
- Derive encryption keys from master key + passphrase
- Implement secure key zeroization on process termination
- Add key validation during database startup

### 4. Integration Points
- Enhance HTTP API initialization endpoints
- Update CLI tools for secure database creation
- Integrate with existing permission system
- Prepare foundation for encryption at rest

## UX/UI Considerations

### Command Line Interface
- Clear prompts for passphrase during initialization
- Secure passphrase input (hidden from terminal history)
- Confirmation of key generation success
- Backup reminder for key recovery information

### HTTP API
- POST `/api/crypto/init` endpoint for programmatic initialization
- Clear error messages for initialization failures
- Status endpoints to verify cryptographic setup
- Security warnings for development vs production

### Configuration
- Secure configuration file format for crypto settings
- Environment variable support for automation
- Clear separation of public vs private key material
- Validation of cryptographic parameters

## Acceptance Criteria

1. **Key Generation**
   - ✅ Database initialization generates Ed25519 master key pair
   - ✅ Key generation uses cryptographically secure random number generator
   - ✅ Public key stored in database metadata for verification
   - ✅ Private key derived from user passphrase, never stored directly

2. **Passphrase Security**
   - ✅ Passphrase-based key derivation using Argon2id with secure parameters
   - ✅ Salt generation and storage for key derivation
   - ✅ Passphrase validation and strength checking
   - ✅ Secure memory handling for passphrase material

3. **Database Integration**
   - ✅ Enhanced database initialization includes crypto setup
   - ✅ Existing database operations remain unaffected
   - ✅ Master public key accessible for verification operations
   - ✅ Database startup validates cryptographic configuration

4. **API Integration**
   - ✅ HTTP endpoint for crypto initialization
   - ✅ Status endpoint for crypto configuration verification
   - ✅ Error handling for initialization failures
   - ✅ Security headers and response validation

5. **Security Requirements**
   - ✅ Private key material never logged or stored in plaintext
   - ✅ Secure zeroization of key material on process termination
   - ✅ Cryptographic parameter validation
   - ✅ Protection against timing attacks in key operations

## Dependencies

- **Internal**: Existing database initialization system
- **External**: 
  - `ed25519-dalek` crate for cryptographic operations
  - `argon2` crate for key derivation
  - `zeroize` crate for secure memory handling
- **Configuration**: Enhanced configuration system for crypto parameters

## Open Questions

1. **Key Recovery**: Should we implement key recovery mechanisms or rely on user backup processes?
2. **Hardware Security**: Should we support hardware security modules (HSMs) for enterprise deployments?
3. **Key Rotation**: How should master key rotation be handled (addressed in PBI-12)?
4. **Multi-Instance**: How should multiple database instances be handled with different master keys?

## Related Tasks

Tasks will be created in [tasks.md](./tasks.md) upon PBI approval. 