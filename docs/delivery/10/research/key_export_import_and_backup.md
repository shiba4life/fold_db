# Key Export/Import and Backup Mechanisms

**Task**: 10-1-3 - Research key export/import and backup mechanisms  
**Author**: AI_Agent  
**Date**: 2025-06-08  
**Status**: Research Complete

---

## 1. Introduction

This document provides actionable research and recommendations for secure key export/import and backup mechanisms across browser, desktop, and CLI environments. It builds on the foundation established in tasks 10-1-1 (platform crypto APIs) and 10-1-2 (secure storage strategies), and is intended to guide implementation tasks 10-2-4, 10-3-4, 10-4-4, and 10-5-x.

---

## 2. Best Practices for Encrypted Key Export/Import

### 2.1 Platform-Specific Considerations

- **Browser**: Use WebCrypto for key extraction (if allowed), encrypt with user passphrase before export. Avoid exposing raw keys to JS context.
- **Desktop/Python**: Use cryptography APIs to serialize keys (e.g., PKCS#8), encrypt with strong KDF-derived key.
- **CLI**: Use OpenSSL or native crypto for export/import, always encrypt exported keys.

### 2.2 Secure Export/Import Flows

- Always require user authentication (passphrase) for export/import.
- Use memory-safe handling: zero sensitive buffers after use.
- Validate imported keys for integrity and format.

### 2.3 User Experience

- Provide clear warnings about risks of exporting private keys.
- Offer backup reminders and recovery options.

---

## 3. Secure Backup Mechanisms

- **Encryption**: All backups must be encrypted with a user-derived key (passphrase + KDF).
- **Storage Options**:
  - Local: Encrypted file with restrictive permissions (0600).
  - Cloud: Only if end-to-end encrypted before upload.
  - Device-to-device: Use QR codes or encrypted transfer protocols.
- **Access Control**: Never store passphrases; prompt user each time.

---

## 4. Threat Model for Key Backup and Recovery

| Threat                        | Description                                      | Mitigation                                 |
|-------------------------------|--------------------------------------------------|--------------------------------------------|
| Device theft                  | Attacker gains access to backup file             | Strong encryption, KDF, file permissions   |
| Phishing/social engineering   | User tricked into revealing passphrase           | User education, UI warnings                |
| Weak passphrase               | Brute-force attack on backup                     | Enforce passphrase strength, KDF limits    |
| Malware/compromise            | Key/backup stolen by malware                     | Memory hygiene, OS security, alerts        |
| Backup file corruption        | Loss of access due to file damage                | Recovery verification, multiple backups    |

---

## 5. Key Serialization Formats and Encryption Standards

- **Formats**: JSON (with base64-encoded key), PEM, PKCS#8 (DER/PEM)
- **Encryption**: AES-GCM, XChaCha20-Poly1305 (modern, authenticated)
- **Metadata**: Include version, KDF params, encryption algorithm, creation date

**Example JSON backup:**
```json
{
  "version": 1,
  "kdf": "argon2id",
  "kdf_params": { "salt": "...", "iterations": 3, "memory": 65536 },
  "encryption": "xchacha20-poly1305",
  "nonce": "...",
  "ciphertext": "...",
  "created": "2025-06-08"
}
```

---

## 6. Cross-Platform Compatibility Requirements

- Use standard formats (JSON, PEM, PKCS#8) for maximum interoperability.
- Normalize line endings (LF) and encoding (UTF-8).
- Avoid platform-specific file attributes; document any required permissions.

---

## 7. Key Derivation Strategies for Backup Encryption

- **KDFs**: Argon2id (preferred), scrypt, PBKDF2 (legacy)
- **Parameters**: Set high memory and iteration counts for Argon2id (e.g., 64MB, 3 iterations)
- **Salt**: Random, unique per backup; store with backup metadata
- **Nonce/IV**: Random, unique per encryption; store with backup

---

## 8. Implementation Guidelines for Secure Backup/Recovery Flows

### 8.1 Backup Flow

1. Prompt user for passphrase (with strength meter).
2. Generate random salt and derive key using KDF.
3. Encrypt private key using authenticated encryption (e.g., XChaCha20-Poly1305).
4. Serialize backup (include metadata).
5. Store backup file with restrictive permissions (0600).

### 8.2 Recovery Flow

1. Prompt user for backup file and passphrase.
2. Parse metadata, extract KDF params, salt, nonce.
3. Derive key using KDF and user passphrase.
4. Decrypt and validate private key.
5. Confirm recovery with user.

### 8.3 Error Handling

- Detect and report incorrect passphrase, corrupted file, or unsupported format.
- Never reveal partial key material on error.

---

## 9. Documented Recommendations and Threat Model

- Always encrypt exported/backed-up keys with a strong, user-derived key.
- Use modern, authenticated encryption and KDFs.
- Provide clear user guidance and warnings.
- Document and test recovery flows for all supported platforms.
- Maintain a compatibility matrix for backup formats.

---

## 10. References

- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [NIST SP 800-132: Recommendation for Password-Based Key Derivation](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-132.pdf)
- [IETF RFC 8439: ChaCha20 and Poly1305 for IETF Protocols](https://datatracker.ietf.org/doc/html/rfc8439)
- [PKCS #8: Private-Key Information Syntax Standard](https://datatracker.ietf.org/doc/html/rfc5208)
- [Argon2 Password Hashing](https://datatracker.ietf.org/doc/html/rfc9106)

---