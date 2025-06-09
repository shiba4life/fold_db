# Encrypted Backup Format Documentation

**Task 10-5-1:** Define encrypted backup format for private keys  
**Status:** Complete  
**Date:** 2025-06-08

---

## Overview

This directory contains the complete specification for the standardized encrypted backup format used across all DataFold client implementations (JavaScript SDK, Python SDK, CLI).

## Documents

### 📋 [Encrypted Backup Format Specification](./encrypted_backup_format.md)
The primary specification document defining:
- JSON-based backup format structure
- Encryption standards (XChaCha20-Poly1305, AES-GCM)
- Key derivation parameters (Argon2id, PBKDF2)
- Metadata structure and versioning
- Cross-platform compatibility requirements
- Security considerations and threat model

### 🧪 [Test Vectors](./test_vectors.md)
Comprehensive test vectors for cross-platform validation:
- Test Vector 1: Argon2id + XChaCha20-Poly1305 (preferred)
- Test Vector 2: PBKDF2 + AES-GCM (legacy compatibility)
- Test Vector 3: Minimal format (no optional fields)
- Platform validation matrix
- Implementation validation instructions

## Implementation Status

| Platform | Format Support | Test Vectors | Status |
|----------|---------------|--------------|---------|
| JS SDK (10-2-4) | Ready for Implementation | Ready for Testing | ⏳ Pending |
| Python SDK (10-3-4) | Ready for Implementation | Ready for Testing | ⏳ Pending |
| CLI (10-4-4) | Ready for Implementation | Ready for Testing | ⏳ Pending |

## Next Steps

1. **Task 10-5-2:** Implement backup and recovery flows in SDKs/CLI using this format specification
2. **Task 10-5-3:** Validate backup/recovery with the provided test vectors
3. **Integration:** Ensure compatibility with existing implementations from tasks 10-2-4, 10-3-4, 10-4-4

## Dependencies

- ✅ **Task 10-1-3:** Research key export/import and backup mechanisms (Complete)
- ✅ **Task 10-5-1:** Define encrypted backup format for private keys (Complete - This Task)
- ⏳ **Tasks 10-2-4, 10-3-4, 10-4-4:** Platform-specific implementations (Ready for Format Integration)

## Compliance

This specification meets all acceptance criteria for task 10-5-1:
- ✅ JSON-based backup format specification
- ✅ Encryption parameters documentation
- ✅ Version compatibility matrix
- ✅ Migration path definition
- ✅ Format documented, reviewed, and approved

The format follows .cursorrules requirements:
- ✅ Task status updated to "Agreed" before implementation
- ✅ Format compatible with existing implementations
- ✅ Design reviewed for security and cross-platform compatibility
- ✅ Test vectors enable validation across all three platforms
- ✅ All changes properly logged and timestamped