# PBI-9: Encryption at Rest

[View in Backlog](../backlog.md#user-content-9)

## Overview

This PBI implements comprehensive encryption at rest for all DataFold database files using AES-256-GCM encryption. The implementation ensures that all atom data stored on disk is cryptographically protected, making physical access to storage devices insufficient to compromise data confidentiality.

## Problem Statement

Currently, DataFold stores all database files in plaintext on disk. This creates significant security vulnerabilities:

- Physical access to storage reveals all data in plaintext
- Database files can be copied and analyzed offline
- No protection against storage device theft or unauthorized access
- Backup files contain unencrypted sensitive data
- Compliance requirements for data protection are not met

## User Stories

**Primary User Story:**
As a node operator, I want all database files to be encrypted at rest using AES-256-GCM so that physical access to storage doesn't compromise data confidentiality.

**Supporting User Stories:**
- As a security officer, I want encrypted backups that maintain data protection
- As a compliance manager, I want encryption standards that meet regulatory requirements
- As a system administrator, I want transparent encryption that doesn't impact normal operations
- As a database user, I want encryption performance that doesn't significantly impact query speed

## Technical Approach

### 1. Encryption Layer Architecture
- Implement `DatabaseEncryption` module using AES-256-GCM
- Create encryption wrapper for database storage operations
- Use master key from PBI-8 for encryption key derivation
- Implement secure nonce generation for each encryption operation

### 2. Storage Integration
- Enhance `FoldDB` with encrypted storage methods
- Intercept all atom write operations for encryption
- Decrypt atom data transparently during read operations
- Maintain existing API surface for database operations

### 3. Key Management
- Derive encryption keys from master key pair + passphrase
- Use BLAKE3 for secure key derivation functions
- Implement key caching for performance optimization
- Add secure key material zeroization

### 4. Backup and Recovery
- Implement encrypted backup export functionality
- Create secure restore mechanisms for encrypted data
- Maintain encryption during data transfer operations
- Add backup verification and integrity checking

## UX/UI Considerations

### Performance Impact
- Target < 20% performance overhead for typical operations
- Implement async encryption where possible
- Add performance monitoring and metrics
- Provide configuration options for performance tuning

### Administrative Interface
- Clear status indication of encryption state
- Backup/restore progress indicators
- Encryption performance metrics
- Error reporting for encryption failures

### Configuration
- Encryption algorithm selection (defaulting to AES-256-GCM)
- Performance tuning parameters
- Backup encryption settings
- Key derivation parameters

## Acceptance Criteria

1. **Encryption Implementation**
   - ✅ All atom data encrypted with AES-256-GCM before disk storage
   - ✅ Unique nonce generation for each encryption operation
   - ✅ Secure key derivation from master key + passphrase
   - ✅ Transparent decryption during normal read operations

2. **Storage Integration**
   - ✅ Existing database APIs work without modification
   - ✅ Encrypted storage for all new atoms
   - ✅ Backward compatibility with existing unencrypted data
   - ✅ Atomic operations maintain consistency with encryption

3. **Performance Requirements**
   - ✅ < 20% performance overhead for typical read/write operations
   - ✅ Async encryption for non-blocking operations where possible
   - ✅ Memory usage increase < 50MB for typical workloads
   - ✅ Startup time increase < 5 seconds for encrypted databases

4. **Backup and Recovery**
   - ✅ Encrypted backup export maintaining data protection
   - ✅ Secure restore from encrypted backup files
   - ✅ Backup integrity verification with cryptographic checksums
   - ✅ Cross-platform backup compatibility

5. **Security Requirements**
   - ✅ Cryptographically secure nonce generation
   - ✅ No encryption key material in log files
   - ✅ Secure memory handling for encryption keys
   - ✅ Protection against side-channel attacks

6. **Error Handling**
   - ✅ Clear error messages for encryption/decryption failures
   - ✅ Graceful handling of corrupted encrypted data
   - ✅ Recovery mechanisms for partial encryption failures
   - ✅ Audit logging of encryption errors

## Dependencies

- **Internal**: 
  - PBI-8 (Database Master Key Encryption) - Required for key material
  - Existing atom storage system
  - Database initialization system
- **External**: 
  - `aes-gcm` crate for AES-256-GCM encryption
  - `blake3` crate for key derivation
  - `rand` crate for secure nonce generation
- **Performance**: Async runtime support for non-blocking encryption

## Open Questions

1. **Migration Strategy**: How should existing unencrypted databases be migrated to encrypted storage?
2. **Key Rotation**: How should encryption key rotation be handled when master keys change?
3. **Compression**: Should compression be applied before or after encryption?
4. **Index Encryption**: Should database indexes also be encrypted, and how would this impact performance?
5. **Memory Encryption**: Should in-memory data structures also be protected?

## Related Tasks

Tasks will be created in [tasks.md](./tasks.md) upon PBI approval. 