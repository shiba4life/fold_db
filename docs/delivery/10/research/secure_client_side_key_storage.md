# Secure Client-Side Key Storage Strategies

**Task**: 10-1-2 - Document secure client-side key storage strategies  
**Author**: tomtang  
**Date**: 2025-06-08  
**Status**: Research Complete

---

## 1. Introduction

This document provides a comprehensive analysis of secure client-side private key storage strategies across browser, desktop/Python, and CLI environments for the DataFold client-side key management system. It builds on the platform crypto API research ([platform_crypto_api_comparison.md](./platform_crypto_api_comparison.md)) and is intended to guide implementation tasks 10-2-2, 10-3-2, and 10-4-2.

---

## 2. Secure Storage Options by Platform

### 2.1 Browser Environments

#### Options
- **IndexedDB (with encryption)**
  - Persistent, quota-limited storage
  - Requires client-side encryption before storing keys
  - Vulnerable to XSS if not properly protected
- **WebCrypto Key Storage (non-extractable keys)**
  - Keys generated as non-extractable remain within browser context
  - Not all browsers support Ed25519 non-extractable keys as of 2025

#### Threats
- XSS attacks exposing keys in memory or storage
- Browser vulnerabilities or compromised extensions
- Storage quota exhaustion

#### Mitigations
- Always encrypt private keys before storing in IndexedDB
- Use Content Security Policy (CSP) to reduce XSS risk
- Prefer non-extractable keys with WebCrypto where possible
- Isolate key access logic from UI code

#### Best Practices
- Never store plaintext keys in browser storage
- Use strong, user-derived passphrases for encryption
- Regularly audit code for XSS vectors

#### Implementation Guidelines
- Use WebCrypto API for key generation and encryption
- Store encrypted keys in IndexedDB
- Use non-extractable keys for signing when supported

---

### 2.2 Desktop/Python Environments

#### Options
- **OS Keychain/Keyring**
  - macOS Keychain, Windows DPAPI, Linux Secret Service
  - Secure, user-scoped storage with OS-level access control
- **Encrypted Files**
  - Use strong KDFs (e.g., Argon2) to derive encryption keys from user passphrases
  - Store files with restrictive permissions (0600)

#### Threats
- Malware or privilege escalation attacks
- Insecure file permissions
- Memory leaks exposing keys

#### Mitigations
- Prefer OS keychain/keyring for storage
- Restrict file permissions on encrypted key files
- Use secure memory handling libraries (e.g., `cryptography.hazmat.primitives`)

#### Best Practices
- Never store keys in plaintext files
- Use per-user keyrings where possible
- Prompt for passphrase on access if not using keyring

#### Implementation Guidelines
- Integrate with `keyring` Python package for cross-platform support
- Fallback to encrypted file storage with strong KDF if keyring unavailable

---

### 2.3 CLI Environments

#### Options
- **Encrypted Configuration Files**
  - Store keys in files encrypted with a strong passphrase-derived key
  - Restrict file permissions (0600)
- **System Keyrings**
  - Use where available (e.g., via `keyring` CLI tools)

#### Threats
- File system attacks (e.g., unauthorized access, backup leaks)
- Weak encryption or passphrases
- Environment variable leaks

#### Mitigations
- Enforce strong passphrase requirements
- Set strict file permissions on key files
- Avoid storing keys in environment variables

#### Best Practices
- Use strong KDFs (Argon2, PBKDF2) for encryption
- Document secure setup and usage for CLI users

#### Implementation Guidelines
- Default to encrypted file storage with strong KDF
- Provide option to use system keyring if available

---

## 3. Security Threats and Mitigation Strategies

| Platform | Threat | Mitigation |
|----------|--------|------------|
| Browser  | XSS, browser exploit | CSP, code audit, encryption, non-extractable keys |
| Desktop  | Malware, file leaks | OS keyring, file permissions, secure memory |
| CLI      | File access, weak pass | Strong KDF, file perms, avoid env vars |

---

## 4. Best Practices for Key Isolation and Access Control

- Apply the principle of least privilege: only allow key access to necessary code paths
- Use non-extractable keys where possible
- Require user authentication for key access
- Separate key management logic from application logic

---

## 5. Implementation Guidelines

### Browser
- Use WebCrypto for key operations
- Store only encrypted keys in IndexedDB
- Use non-extractable keys for signing

### Desktop/Python
- Use `keyring` for OS keychain integration
- Fallback to encrypted file storage with Argon2 KDF
- Restrict file permissions

### CLI
- Store keys in encrypted files with strong KDF
- Restrict file permissions
- Optionally use system keyring

---

## 6. References and Further Reading

- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [WebCrypto API Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API)
- [Python keyring documentation](https://pypi.org/project/keyring/)
- [Argon2 Password Hashing](https://github.com/P-H-C/phc-winner-argon2)

---

## 7. Change Log

- 2025-06-08: Initial version by tomtang. Status: Research Complete.