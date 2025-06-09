# Platform Crypto API Comparison for Ed25519 Key Management

**Task**: 10-1-1 - Research platform crypto APIs  
**Author**: tomtang  
**Date**: 2025-06-08  
**Status**: Research Complete  

## 1. Introduction

### Purpose and Scope
This document provides a comprehensive comparison of cryptographic APIs across three key platforms for implementing Ed25519 key generation and secure storage in the DataFold client-side key management system:

1. **WebCrypto API** (Browser/JavaScript environment)
2. **Python cryptography package** (Python applications/services)
3. **OpenSSL & Rust crypto libraries** (CLI tools and native applications)

### Research Objectives
- Survey Ed25519 key generation capabilities on each platform
- Identify secure storage options and limitations
- Document API constraints and platform-specific considerations
- Provide actionable recommendations for implementation tasks 10-2-1, 10-3-1, and 10-4-1

## 2. WebCrypto API (Browser/JavaScript)

### Ed25519 Key Generation Support

**Current Status**: Limited and evolving
- **Ed25519 support**: Not universally supported in WebCrypto API as of 2025
- **Browser compatibility**:
  - Chrome/Edge: Partial support via OKP (Octet Key Pair) format
  - Firefox: Limited experimental support
  - Safari: No native support
- **Alternative approach**: Use JavaScript crypto libraries (e.g., `tweetnacl`, `@noble/ed25519`)

### API Capabilities
```javascript
// Modern WebCrypto (where supported)
const keyPair = await crypto.subtle.generateKey(
  {
    name: "Ed25519",
    namedCurve: "Ed25519"
  },
  true, // extractable
  ["sign", "verify"]
);

// Fallback: JavaScript library approach
import { generateKeyPair } from '@noble/ed25519';
const privateKey = generateKeyPair();
```

### Secure Storage Options

1. **IndexedDB with encryption**
   - Persistent browser storage
   - Requires client-side encryption before storage
   - Storage quota limitations (typically 50MB-2GB)
   - Vulnerable to XSS attacks

2. **WebCrypto Key Storage**
   - Non-extractable keys stored in browser's secure context
   - Limited to WebCrypto-supported algorithms
   - Keys cannot be backed up or exported

3. **localStorage/sessionStorage**
   - Not recommended for private keys
   - Plain text storage, vulnerable to XSS

### Limitations and Constraints

- **Ed25519 compatibility**: Requires polyfills or alternative libraries
- **Storage persistence**: Browser data can be cleared by user
- **Cross-browser compatibility**: Inconsistent API support
- **Security model**: Vulnerable to XSS and browser exploits
- **Key export**: Complex for backup scenarios

### Security Considerations
- Use Content Security Policy (CSP) to mitigate XSS
- Implement key derivation from user passwords
- Consider hardware security module (HSM) integration where available
- Encrypt all stored key material

## 3. Python cryptography Package

### Ed25519 Key Generation Support

**Current Status**: Excellent native support
```python
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives import serialization

# Generate Ed25519 key pair
private_key = Ed25519PrivateKey.generate()
public_key = private_key.public_key()
```

### API Capabilities

1. **Key Generation**
   - Native Ed25519 support since cryptography 2.6
   - Secure random number generation
   - Multiple serialization formats (PEM, DER, raw bytes)

2. **Key Serialization**
   ```python
   # Private key serialization with password protection
   encrypted_pem = private_key.private_bytes(
       encoding=serialization.Encoding.PEM,
       format=serialization.PrivateFormat.PKCS8,
       encryption_algorithm=serialization.BestAvailableEncryption(password)
   )
   
   # Public key serialization
   public_pem = public_key.public_bytes(
       encoding=serialization.Encoding.PEM,
       format=serialization.PublicFormat.SubjectPublicKeyInfo
   )
   ```

### Secure Storage Options

1. **OS Keychain Integration**
   - **macOS**: Keychain Services via `keyring` library
   - **Windows**: Windows Credential Manager
   - **Linux**: Secret Service API (GNOME Keyring, KWallet)

2. **Encrypted File Storage**
   ```python
   import keyring
   from cryptography.fernet import Fernet
   
   # Store in OS keychain
   keyring.set_password("datafold", "ed25519_key", encrypted_key_data)
   
   # Encrypted file with proper permissions
   with open(key_file, 'wb') as f:
       f.write(encrypted_key_data)
   os.chmod(key_file, 0o600)  # Read/write for owner only
   ```

3. **Hardware Security Modules (HSM)**
   - PKCS#11 integration via `python-pkcs11`
   - TPM integration via `python-tpm2-pytss`

### Limitations and Constraints

- **OS dependency**: Keychain availability varies by platform
- **Installation requirements**: Requires system crypto libraries
- **Memory management**: Python GC may leave key material in memory
- **Packaging complexity**: Binary dependencies for distribution

### Security Considerations
- Use `secrets` module for secure random generation
- Clear sensitive variables explicitly: `del private_key`
- Implement secure memory handling where possible
- Use platform-specific secure storage APIs

## 4. OpenSSL & Rust Crypto (CLI)

### Ed25519 Key Generation Support

**OpenSSL Approach**:
```bash
# Generate Ed25519 private key
openssl genpkey -algorithm Ed25519 -out private.pem

# Extract public key
openssl pkey -in private.pem -pubout -out public.pem

# Generate with password protection
openssl genpkey -algorithm Ed25519 -aes256 -out private_encrypted.pem
```

**Rust Crypto Approach**:
```rust
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use rand::rngs::OsRng;

// Generate keypair
let mut csprng = OsRng{};
let keypair: Keypair = Keypair::generate(&mut csprng);
```

### API Capabilities

1. **OpenSSL CLI**
   - Mature, well-tested Ed25519 implementation
   - Multiple output formats (PEM, DER)
   - Password-based encryption
   - Batch operations support

2. **Rust Libraries**
   - `ed25519-dalek`: Pure Rust implementation
   - `ring`: Fast, safe cryptography
   - `rustcrypto`: Modular crypto libraries
   - `openssl-sys`: OpenSSL bindings

### Secure Storage Options

1. **File System with Permissions**
   ```bash
   # Secure file permissions
   chmod 600 private_key.pem
   chown $USER:$USER private_key.pem
   
   # Encrypted storage
   gpg --symmetric --cipher-algo AES256 private_key.pem
   ```

2. **OS Integration**
   - **macOS**: `security` command for Keychain
   - **Linux**: `secret-tool` for Secret Service
   - **Windows**: PowerShell Credential Manager

3. **Hardware Security**
   - PKCS#11 token support via OpenSSL
   - TPM integration
   - Hardware wallets (via vendor SDKs)

### Limitations and Constraints

- **OpenSSL dependency**: Version compatibility issues
- **Platform variations**: Different crypto library availability
- **File permissions**: Limited on some filesystems
- **User experience**: Command-line complexity for end users

### Security Considerations
- Use secure random sources (`/dev/urandom`, `CryptGenRandom`)
- Implement secure file deletion (overwrite + delete)
- Monitor file access permissions
- Consider hardware-backed storage

## 5. Comparison Matrix

| Feature | WebCrypto API | Python cryptography | OpenSSL/Rust CLI |
|---------|---------------|---------------------|------------------|
| **Ed25519 Support** | Limited/Polyfill | Native/Excellent | Native/Excellent |
| **Browser Compatibility** | Inconsistent | N/A | N/A |
| **Key Generation** | Via libraries | Built-in | Built-in |
| **Secure Storage** | IndexedDB + encryption | OS Keychain | File + permissions |
| **Cross-platform** | Yes (browsers) | Yes (with deps) | Yes (with setup) |
| **Hardware Integration** | Limited | PKCS#11/TPM | PKCS#11/TPM |
| **Backup/Export** | Complex | Straightforward | Straightforward |
| **Memory Security** | Browser-dependent | Limited | Good (Rust) |
| **Installation Complexity** | None (web) | Medium | Medium |
| **Performance** | Good | Excellent | Excellent |

## 6. Security Assessment

### Threat Models by Platform

#### WebCrypto (Browser)
- **Primary threats**: XSS, browser exploits, data clearing
- **Mitigations**: CSP, key derivation, hardware tokens
- **Risk level**: Medium-High

#### Python 
- **Primary threats**: Memory dumps, filesystem access, dependency attacks
- **Mitigations**: OS keychain, secure coding, dependency pinning
- **Risk level**: Medium

#### CLI (OpenSSL/Rust)
- **Primary threats**: File access, privilege escalation, side-channel attacks
- **Mitigations**: File permissions, hardware storage, secure compilation
- **Risk level**: Low-Medium

## 7. Implementation Recommendations

### For JavaScript SDK (Task 10-2-1)
1. **Primary approach**: Use `@noble/ed25519` library for consistent Ed25519 support
2. **Storage strategy**: IndexedDB with PBKDF2-derived encryption keys
3. **Fallback**: Prompt for hardware wallet integration
4. **Browser support**: Target modern browsers with polyfill detection

### For Python SDK (Task 10-3-1)
1. **Primary approach**: Use `cryptography` package native Ed25519 support
2. **Storage strategy**: OS keychain via `keyring` with encrypted file fallback
3. **Memory handling**: Explicit key deletion and `secrets` module usage
4. **Platform support**: macOS/Windows/Linux with graceful degradation

### For CLI (Task 10-4-1)
1. **Primary approach**: OpenSSL CLI with Rust wrapper for advanced features
2. **Storage strategy**: Encrypted files with 600 permissions + optional hardware
3. **UX considerations**: Interactive prompts for key generation and passwords
4. **Distribution**: Static binaries with embedded OpenSSL where possible

## 8. Performance Benchmarks

### Key Generation Performance (approximate)
- **WebCrypto**: 50-100ms (library-dependent)
- **Python cryptography**: 1-5ms
- **OpenSSL CLI**: 1-10ms (process overhead)
- **Rust ed25519-dalek**: <1ms

### Storage/Retrieval Performance
- **IndexedDB**: 10-50ms (async)
- **OS Keychain**: 50-200ms (OS calls)
- **File system**: 1-10ms (SSD)

## 9. Platform-Specific Constraints

### WebCrypto
- **Constraint**: Ed25519 not universally supported
- **Workaround**: JavaScript library fallback
- **Impact**: Larger bundle size, potential security review complexity

### Python
- **Constraint**: Binary dependencies for some platforms
- **Workaround**: Wheel distribution, conda packaging
- **Impact**: Installation complexity in some environments

### CLI
- **Constraint**: OpenSSL version compatibility
- **Workaround**: Static linking or bundled libraries
- **Impact**: Larger binary size, potential security patch lag

## 10. Next Steps for Implementation

### Immediate Actions (Task Dependencies)
1. **Task 10-2-1**: Begin with `@noble/ed25519` integration
2. **Task 10-3-1**: Implement `cryptography` package wrapper
3. **Task 10-4-1**: Create OpenSSL CLI wrapper with Rust helpers

### Research Gaps to Address
1. Hardware wallet integration APIs
2. Browser extension security models
3. Enterprise key management system integration
4. Quantum-resistant algorithm migration paths

## 11. References

- [WebCrypto API Specification](https://w3c.github.io/webcrypto/)
- [Python cryptography Documentation](https://cryptography.io/)
- [OpenSSL Ed25519 Documentation](https://www.openssl.org/docs/)
- [`@noble/ed25519` Library](https://github.com/paulmillr/noble-ed25519)
- [`ed25519-dalek` Rust Crate](https://docs.rs/ed25519-dalek/)
- [NIST SP 800-186: Ed25519 and Ed448 Standards](https://csrc.nist.gov/publications/detail/sp/800-186/final)

---

**Document Status**: Complete  
**Next Review**: Before implementation tasks 10-2-1, 10-3-1, 10-4-1  
**Acceptance Criteria Met**: ✓ Comprehensive API comparison, ✓ Security assessment, ✓ Implementation recommendations, ✓ Performance benchmarks