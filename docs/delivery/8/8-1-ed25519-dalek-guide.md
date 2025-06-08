# Ed25519-Dalek Implementation Guide

**Date:** 2024-12-30  
**Version:** ed25519-dalek 2.1.1  
**Source:** https://docs.rs/ed25519-dalek/latest/  

## Overview

The `ed25519-dalek` crate is a pure Rust implementation of Ed25519 key generation, signing, and verification. It provides fast, secure elliptic curve cryptography using the Ed25519 signature scheme.

## Key Features

- Pure Rust implementation with no FFI dependencies
- Constant-time operations for security
- Support for batch signature verification
- Optional serde serialization support
- Optional PKCS#8 encoding/decoding
- WASM-friendly design

## Core Types

### SigningKey
- Represents a complete Ed25519 signing key (private key + public key)
- Used for creating signatures
- Can be serialized to 32 bytes (secret) or 64 bytes (keypair format)

### VerifyingKey  
- Represents an Ed25519 public key
- Used for signature verification
- Serializes to 32 bytes

### Signature
- Represents an Ed25519 signature
- Serializes to 64 bytes
- Can be created from signing and verified against public keys

## Basic Usage Examples

### Key Generation

```rust
use rand::rngs::OsRng;
use ed25519_dalek::SigningKey;

// Generate a new signing key using secure random number generator
let mut csprng = OsRng;
let signing_key: SigningKey = SigningKey::generate(&mut csprng);

// Extract the verifying (public) key
let verifying_key = signing_key.verifying_key();
```

### Creating Signatures

```rust
use ed25519_dalek::{Signature, Signer};

let message: &[u8] = b"This is a test message";
let signature: Signature = signing_key.sign(message);
```

### Verifying Signatures

```rust
use ed25519_dalek::Verifier;

// Verify with the signing key (has access to public key)
assert!(signing_key.verify(message, &signature).is_ok());

// Verify with just the public key
assert!(verifying_key.verify(message, &signature).is_ok());
```

## Serialization

### To Bytes

```rust
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};

// Serialize different components
let verifying_key_bytes: [u8; PUBLIC_KEY_LENGTH] = verifying_key.to_bytes();
let secret_key_bytes: [u8; SECRET_KEY_LENGTH] = signing_key.to_bytes();
let signing_key_bytes: [u8; KEYPAIR_LENGTH] = signing_key.to_keypair_bytes();
let signature_bytes: [u8; SIGNATURE_LENGTH] = signature.to_bytes();
```

### From Bytes

```rust
// Deserialize from bytes
let verifying_key_restored = VerifyingKey::from_bytes(&verifying_key_bytes)?;
let signing_key_restored = SigningKey::from_bytes(&signing_key_bytes);
let signature_restored = Signature::try_from(&signature_bytes[..])?;
```

## PKCS#8 Support (Optional Feature)

When the `pkcs8` feature is enabled:

```rust
use ed25519_dalek::{VerifyingKey, pkcs8::DecodePublicKey};

let pem = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=
-----END PUBLIC KEY-----";

let verifying_key = VerifyingKey::from_public_key_pem(pem)
    .expect("invalid public key PEM");
```

## Security Considerations

### Key Storage
- **Never store private keys in plaintext**
- Use secure key derivation for key generation
- Consider using the `zeroize` feature for secure memory cleanup
- Private keys should be kept in memory for minimal time

### Random Number Generation
- Always use a cryptographically secure random number generator (CSPRNG)
- `OsRng` is recommended for most applications
- Never use predictable or weak random sources

### Signature Verification
- Always verify signatures before trusting signed data
- Use constant-time verification operations (built into the crate)
- Be aware of potential timing attacks in application logic

## Error Handling

The crate uses `SignatureError` for most operations:

```rust
use ed25519_dalek::SignatureError;

match verifying_key.verify(message, &signature) {
    Ok(()) => println!("Signature is valid"),
    Err(SignatureError::VerificationError) => println!("Invalid signature"),
}
```

## Integration with DataFold

For DataFold's database master key encryption:

```rust
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

// Generate master key pair for database
pub fn generate_master_keypair() -> SigningKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng)
}

// Sign database operations or metadata
pub fn sign_database_operation(signing_key: &SigningKey, operation_data: &[u8]) -> Signature {
    signing_key.sign(operation_data)
}

// Verify database operation signatures
pub fn verify_database_operation(
    verifying_key: &VerifyingKey, 
    operation_data: &[u8], 
    signature: &Signature
) -> Result<(), SignatureError> {
    verifying_key.verify(operation_data, signature)
}
```

## Cargo.toml Configuration

```toml
[dependencies]
ed25519-dalek = { version = "2.1", features = ["rand_core", "serde", "pkcs8"] }
rand = "0.8"
```

## Constants Reference

- `PUBLIC_KEY_LENGTH = 32` - Length of Ed25519 public key in bytes
- `SECRET_KEY_LENGTH = 32` - Length of Ed25519 secret key in bytes  
- `KEYPAIR_LENGTH = 64` - Length of Ed25519 keypair in bytes
- `SIGNATURE_LENGTH = 64` - Length of Ed25519 signature in bytes

## Best Practices

1. **Always use secure random number generation**
2. **Verify all signatures before trusting data**
3. **Store private keys securely and minimize exposure time**
4. **Use the latest version of the crate for security updates**
5. **Enable zeroize feature for automatic memory cleanup**
6. **Consider using PKCS#8 for interoperability**
7. **Test signature round-trips in your integration tests** 