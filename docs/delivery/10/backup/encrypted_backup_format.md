# Encrypted Backup Format Specification for Private Keys

**Status:** Draft  
**Author:** AI_Agent  
**Date:** 2025-06-08  
**Reviewed by:** _(to be filled upon review)_  
**Approved by:** _(to be filled upon approval)_  

---

## 1. Introduction

This document defines the standardized, cross-platform encrypted backup format for private keys, supporting JavaScript SDK, Python SDK, and CLI implementations. The format is designed for interoperability, security, and future extensibility.

## 2. Format Overview

The backup format is a JSON object with the following structure:

```json
{
  "version": 1,
  "kdf": "argon2id",
  "kdf_params": { "salt": "...", "iterations": 3, "memory": 65536, "parallelism": 2 },
  "encryption": "xchacha20-poly1305",
  "nonce": "...",
  "ciphertext": "...",
  "created": "2025-06-08T17:00:00Z",
  "metadata": {
    "key_type": "ed25519",
    "label": "user-main"
  }
}
```

## 3. Field Definitions

- **version**: Integer. Format version. Increments for breaking changes.
- **kdf**: String. Key derivation function. Supported: `"argon2id"` (preferred), `"pbkdf2"` (legacy).
- **kdf_params**: Object. Parameters for the KDF:
  - **salt**: Base64-encoded random salt (min 16 bytes).
  - **iterations**: Integer (argon2id: min 3, pbkdf2: min 100,000).
  - **memory**: Integer (argon2id: min 65536 KiB).
  - **parallelism**: Integer (argon2id only, min 2).
- **encryption**: String. Symmetric AEAD cipher. Supported: `"xchacha20-poly1305"` (preferred), `"aes-gcm"` (fallback).
- **nonce**: Base64-encoded nonce (24 bytes for xchacha20, 12 bytes for aes-gcm).
- **ciphertext**: Base64-encoded AEAD-encrypted private key (Ed25519 PKCS#8 DER or raw).
- **created**: ISO 8601 UTC timestamp.
- **metadata**: (Optional) Object. Extensible for key type, label, or future fields.

## 4. Encryption and KDF Standards

- **Encryption**: Use XChaCha20-Poly1305 (preferred) or AES-GCM (fallback). Both provide authenticated encryption (AEAD).
- **KDF**: Use Argon2id (preferred) with strong parameters. PBKDF2 allowed for legacy compatibility.
- **Parameter Recommendations**:
  - Argon2id: salt ≥ 16 bytes, memory ≥ 64 MiB, iterations ≥ 3, parallelism ≥ 2.
  - PBKDF2: salt ≥ 16 bytes, iterations ≥ 100,000.

## 5. Integrity Verification

- AEAD ciphers provide built-in integrity (authentication tag).
- Implementations must reject backups with failed authentication.
- Optionally, a separate MAC or checksum field may be added for legacy support.

## 6. Versioning and Format Evolution

- The `version` field governs breaking changes.
- Implementations must ignore unknown fields for forward compatibility.
- New fields may be added; existing fields must not be removed or repurposed without a version bump.
- Migration strategies must be documented for each version change.

## 7. Cross-Platform Compatibility

- All fields must be JSON-serializable and use base64 encoding for binary data.
- No platform-specific binary formats.
- All SDKs/CLI must support both Argon2id and PBKDF2, and both XChaCha20-Poly1305 and AES-GCM.
- Test vectors must be provided and validated on all platforms.

## 8. Test Vectors

### Example Test Vector

- **Passphrase:** `correct horse battery staple`
- **Salt (base64):** `w7Z3pQ2v5Q8v1Q2v5Q8v1Q==`
- **Nonce (base64):** `AAAAAAAAAAAAAAAAAAAAAAAAAAA=`
- **KDF:** `argon2id`
- **KDF Params:** `{ "salt": "...", "iterations": 3, "memory": 65536, "parallelism": 2 }`
- **Encryption:** `xchacha20-poly1305`
- **Plaintext Key (hex):** `302e020100300506032b657004220420...` (Ed25519 PKCS#8 DER)
- **Ciphertext (base64):** `...` (to be generated)
- **Created:** `2025-06-08T17:00:00Z`

_Implementations must be able to decrypt this vector and recover the original key using the passphrase._

## 9. Security Considerations & Threat Model

| Threat                      | Description                                  | Mitigation                                 |
|-----------------------------|----------------------------------------------|--------------------------------------------|
| Device theft                | Attacker gains access to backup file         | Strong encryption, KDF, file permissions   |
| Phishing/social engineering | User tricked into revealing passphrase       | User education, UI warnings                |
| Weak passphrase             | Brute-force attack on backup                 | Enforce passphrase strength, KDF limits    |
| Malware/compromise          | Key/backup stolen by malware                 | Memory hygiene, OS security, alerts        |
| Backup file corruption      | Loss of access due to file damage            | Recovery verification, multiple backups    |

- Never store passphrases.
- Always prompt user for passphrase on export/import.
- Use restrictive file permissions (0600) for local storage.
- Warn users about risks of exporting private keys.

## 10. Implementation Guidelines

- Follow recommendations in [`docs/delivery/10/research/key_export_import_and_backup.md`](../research/key_export_import_and_backup.md).
- Zero sensitive buffers after use.
- Validate all fields and reject malformed or unauthenticated backups.
- Provide clear error messages for failed imports.
- Document migration steps for future format changes.

## 11. Review and Approval

- This document must be reviewed for:
  - Security (by cryptography reviewer)
  - Cross-platform compatibility (by SDK/CLI maintainers)
- Approval and review logs must be added below:

---

**Review Log:**  
- _2025-06-08: Created by AI_Agent (per .cursorrules)_

**Approval Log:**  
- _Pending_