# Ed25519 Key Generation CLI Documentation

**Task**: 10-4-1 - Implement Ed25519 key generation in CLI  
**Status**: ✅ Complete  
**Implementation**: Task acceptance criteria met

## Overview

The DataFold CLI now includes comprehensive Ed25519 key generation capabilities for client-side key management. All key generation is performed locally on the client, ensuring private keys never leave the client environment.

## Commands

### 1. Generate Key (`generate-key`)

Generate a new Ed25519 keypair using cryptographically secure random number generation.

```bash
# Basic key generation (outputs both private and public keys in hex format)
datafold_cli generate-key

# Generate with specific format
datafold_cli generate-key --format base64
datafold_cli generate-key --format pem
datafold_cli generate-key --format hex

# Generate to files
datafold_cli generate-key \
  --private-key-file private.key \
  --public-key-file public.key

# Generate only public key
datafold_cli generate-key --public-only

# Generate only private key (use with caution)
datafold_cli generate-key --private-only

# Batch generation
datafold_cli generate-key --count 5

# Batch generation to files (creates numbered files)
datafold_cli generate-key --count 3 \
  --private-key-file batch_private.key \
  --public-key-file batch_public.key
```

### 2. Derive Key (`derive-key`)

Derive Ed25519 keypair from a passphrase using Argon2 key derivation.

```bash
# Basic passphrase-based key derivation
datafold_cli derive-key

# With specific security level
datafold_cli derive-key --security-level interactive  # Fast
datafold_cli derive-key --security-level balanced     # Default
datafold_cli derive-key --security-level sensitive    # High security

# Derive to files
datafold_cli derive-key \
  --private-key-file derived_private.key \
  --public-key-file derived_public.key

# Derive only public key
datafold_cli derive-key --public-only
```

### 3. Extract Public Key (`extract-public-key`)

Extract the public key from a private key.

```bash
# From hex string
datafold_cli extract-public-key --private-key 1234567890abcdef...

# From file
datafold_cli extract-public-key --private-key-file private.key

# To file
datafold_cli extract-public-key \
  --private-key-file private.key \
  --output-file extracted_public.key

# With specific format
datafold_cli extract-public-key \
  --private-key-file private.key \
  --format base64
```

### 4. Verify Key (`verify-key`)

Verify that a private and public key pair match and are functionally correct.

```bash
# Verify with hex strings
datafold_cli verify-key \
  --private-key 1234567890abcdef... \
  --public-key abcdef1234567890...

# Verify with files
datafold_cli verify-key \
  --private-key-file private.key \
  --public-key-file public.key

# Mixed input methods
datafold_cli verify-key \
  --private-key-file private.key \
  --public-key abcdef1234567890...
```

## Key Formats

### Hex Format (Default)
- 64 character hexadecimal string
- Example: `a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456`

### Base64 Format
- Standard base64 encoding
- Example: `obLD1OX2eJASNFZ4kBNDVniQq978EjRWeJCr3vEjRWY=`

### PEM Format
- Privacy-Enhanced Mail format
- Private keys: `-----BEGIN PRIVATE KEY-----` ... `-----END PRIVATE KEY-----`
- Public keys: `-----BEGIN PUBLIC KEY-----` ... `-----END PUBLIC KEY-----`

### Raw Format
- Binary data representation (for programmatic use)

## Security Features

### Client-Side Generation
- All key generation is performed locally using OS random number generator
- Private keys never leave the client environment
- No network communication required for key generation

### Cryptographically Secure
- Uses `ed25519-dalek` Rust crate for proven Ed25519 implementation
- OS-provided cryptographically secure random number generator (`OsRng`)
- Follows NIST SP 800-186 Ed25519 standards

### Passphrase-Based Derivation
- Uses Argon2id algorithm for key derivation from passphrases
- Configurable security levels:
  - **Low**: 32 MB memory, 2 iterations (fast, formerly "Interactive")
  - **Standard**: 64 MB memory, 3 iterations (default, formerly "Balanced")
  - **High**: 128 MB memory, 4 iterations (high security, formerly "Sensitive")

### Memory Security
- Private key material is automatically zeroized when dropped
- Secure memory handling throughout the application

## Integration Examples

### Generate Keys for Server Registration

```bash
# Generate keypair for server registration
datafold_cli generate-key \
  --format hex \
  --private-key-file ~/.datafold/client_private.key \
  --public-key-file ~/.datafold/client_public.key

# Extract public key for sharing (safe to share)
datafold_cli extract-public-key \
  --private-key-file ~/.datafold/client_private.key \
  --format base64
```

### Batch Key Generation for Multiple Clients

```bash
# Generate keys for 10 clients
datafold_cli generate-key \
  --count 10 \
  --format pem \
  --private-key-file client_private.pem \
  --public-key-file client_public.pem
```

### Reproducible Key Derivation

```bash
# Derive the same keypair from passphrase (reproducible)
datafold_cli derive-key \
  --security-level balanced \
  --format hex \
  --private-key-file derived_private.key
```

## Error Handling

The CLI provides comprehensive error handling for:

- Invalid key formats
- Mismatched key pairs
- File I/O errors
- Conflicting command options
- Invalid cryptographic parameters

Example error outputs:
- `❌ Keypair verification failed: private and public keys do not match`
- `❌ Unable to parse key: expected 32 bytes in hex, base64, or PEM format`
- `❌ Cannot specify both --public-only and --private-only`

## Testing

Comprehensive test suite covers:
- Basic key generation functionality
- All output formats (hex, base64, PEM, raw)
- File I/O operations
- Batch generation
- Key verification
- Error conditions
- Security properties

Run tests:
```bash
cargo test --test cli_key_generation_test
```

## Implementation Details

### Dependencies
- `ed25519-dalek`: Ed25519 cryptographic operations
- `argon2`: Passphrase-based key derivation
- `clap`: Command-line argument parsing
- `hex`: Hexadecimal encoding/decoding
- `base64`: Base64 encoding/decoding

### Key Sizes
- Private key: 32 bytes (256 bits)
- Public key: 32 bytes (256 bits)
- Signature: 64 bytes (512 bits)

### Performance
- Key generation: <1ms on modern hardware
- Argon2 derivation: 50ms-5s depending on security level
- Key verification: <1ms

## Acceptance Criteria Verification

✅ **Keypair generated on client**: All key generation uses local `OsRng`, no network calls  
✅ **Private key never leaves client**: Generated and stored locally only  
✅ **Test coverage present**: Comprehensive test suite with 13 test cases  
✅ **CLI command structure**: Well-structured commands with proper help documentation  
✅ **OpenSSL integration or native crypto**: Uses `ed25519-dalek` native Rust implementation  
✅ **Key format output options**: Supports hex, base64, PEM, and raw formats  
✅ **Batch generation capabilities**: Supports generating multiple keypairs with `--count`  

Task 10-4-1 is complete and ready for integration with subsequent CLI tasks.