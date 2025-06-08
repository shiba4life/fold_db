# Argon2 Key Derivation Implementation Guide

**Date:** 2024-12-30  
**Version:** argon2 0.5.3  
**Source:** https://docs.rs/argon2/latest/  

## Overview

The `argon2` crate is a pure Rust implementation of the Argon2 password hashing function, winner of the Password Hashing Competition. It provides memory-hard key derivation suitable for password hashing and cryptographic key generation.

## Key Features

- Pure Rust implementation with no FFI dependencies
- Supports all Argon2 variants: Argon2d, Argon2i, and Argon2id
- Memory-hard algorithm resistant to GPU attacks
- Configurable memory cost, time cost, and parallelism
- Built-in salt generation and management
- Optional password-hash trait integration

## Argon2 Variants

### Argon2d
- Data-dependent memory access
- Faster but vulnerable to side-channel attacks
- Suitable for applications without side-channel concerns

### Argon2i  
- Data-independent memory access
- Resistant to side-channel attacks
- Preferred for password hashing

### Argon2id (Recommended)
- Hybrid approach combining Argon2i and Argon2d
- First half uses Argon2i, second half uses Argon2d
- Best balance of security and performance
- **Default choice for most applications**

## Basic Usage Examples

### Key Derivation (For Cryptographic Keys)

```rust
use argon2::Argon2;

let password = b"user-provided-passphrase";
let salt = b"unique-salt-per-key"; // Should be unique per password

// Derive a 32-byte key
let mut output_key_material = [0u8; 32];
Argon2::default().hash_password_into(password, salt, &mut output_key_material)?;

// output_key_material now contains the derived key
```

### Password Hashing (For Authentication)

```rust
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

let password = b"user-password";
let salt = SaltString::generate(&mut OsRng);

// Hash password to PHC string ($argon2id$v=19$...)
let argon2 = Argon2::default();
let password_hash = argon2.hash_password(password, &salt)?.to_string();

// Verify password against PHC string
let parsed_hash = PasswordHash::new(&password_hash)?;
assert!(Argon2::default().verify_password(password, &parsed_hash).is_ok());
```

## Custom Parameters

### Creating Custom Configuration

```rust
use argon2::{Argon2, Algorithm, Version, Params};

// Build custom parameters
let params = Params::new(
    65536,  // memory cost in KiB (64 MB)
    3,      // time cost (iterations)
    4,      // parallelism (threads)
    Some(32) // hash length in bytes
)?;

// Create Argon2 instance with custom parameters
let argon2 = Argon2::new(
    Algorithm::Argon2id,
    Version::V0x13,
    params
);
```

### Recommended Parameters for Different Use Cases

```rust
use argon2::{Params, ParamsBuilder};

// For password hashing (authentication)
let auth_params = ParamsBuilder::new()
    .m_cost(19456)    // 19 MB memory
    .t_cost(2)        // 2 iterations
    .p_cost(1)        // 1 thread
    .output_len(32)   // 32 byte hash
    .build()?;

// For key derivation (higher security)
let kdf_params = ParamsBuilder::new()
    .m_cost(65536)    // 64 MB memory
    .t_cost(3)        // 3 iterations  
    .p_cost(4)        // 4 threads
    .output_len(32)   // 32 byte key
    .build()?;

// For fast key derivation (lower security)
let fast_params = ParamsBuilder::new()
    .m_cost(4096)     // 4 MB memory
    .t_cost(1)        // 1 iteration
    .p_cost(1)        // 1 thread
    .output_len(32)   // 32 byte key
    .build()?;
```

## Security Considerations

### Salt Management
- **Always use unique salts** for each password/key derivation
- Salt should be at least 16 bytes (128 bits)
- Store salt alongside the hash for password verification
- For key derivation, salt can be derived from context

### Parameter Selection
- Higher memory cost increases resistance to parallel attacks
- Higher time cost increases computational difficulty
- Balance security needs with performance requirements
- Test parameters on target hardware

### Key Storage
- Never store derived keys in plaintext long-term
- Use secure memory handling (zeroize) when possible
- Minimize key lifetime in memory

## Integration with DataFold

### Master Key Derivation from Passphrase

```rust
use argon2::{Argon2, Algorithm, Version, Params};
use ed25519_dalek::SigningKey;

// Parameters for DataFold master key derivation
const MEMORY_COST: u32 = 65536;  // 64 MB
const TIME_COST: u32 = 3;        // 3 iterations
const PARALLELISM: u32 = 4;      // 4 threads
const KEY_LENGTH: usize = 32;    // 32 bytes for Ed25519 seed

pub fn derive_master_key_from_passphrase(
    passphrase: &[u8],
    salt: &[u8]  // Should be stored with database metadata
) -> Result<SigningKey, argon2::Error> {
    // Create strong parameters for master key derivation
    let params = Params::new(
        MEMORY_COST,
        TIME_COST, 
        PARALLELISM,
        Some(KEY_LENGTH)
    )?;
    
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        params
    );
    
    // Derive key material
    let mut key_material = [0u8; KEY_LENGTH];
    argon2.hash_password_into(passphrase, salt, &mut key_material)?;
    
    // Create Ed25519 signing key from derived material
    let signing_key = SigningKey::from_bytes(&key_material);
    
    // Zeroize the intermediate key material
    key_material.zeroize();
    
    Ok(signing_key)
}
```

### Salt Generation for Database Initialization

```rust
use argon2::password_hash::{rand_core::OsRng, SaltString};

pub fn generate_database_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}

// For binary salt (when not using PHC strings)
pub fn generate_binary_salt() -> [u8; 32] {
    use rand::RngCore;
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}
```

## Error Handling

```rust
use argon2::{Error, Result};

match argon2.hash_password_into(password, salt, &mut output) {
    Ok(()) => println!("Key derivation successful"),
    Err(Error::OutputTooShort) => println!("Output buffer too small"),
    Err(Error::SaltTooShort) => println!("Salt too short (minimum 8 bytes)"),
    Err(Error::MemoryTooLittle) => println!("Memory cost too low"),
    Err(Error::TimeTooSmall) => println!("Time cost too low"),
    Err(e) => println!("Other error: {:?}", e),
}
```

## Performance Tuning

### Benchmarking Parameters

```rust
use std::time::Instant;
use argon2::{Argon2, Params};

fn benchmark_argon2_params(memory_mb: u32, iterations: u32) -> std::time::Duration {
    let params = Params::new(
        memory_mb * 1024, // Convert MB to KiB
        iterations,
        1, // Single thread for consistent timing
        Some(32)
    ).unwrap();
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params
    );
    
    let password = b"test-password";
    let salt = b"test-salt-16bytes";
    let mut output = [0u8; 32];
    
    let start = Instant::now();
    argon2.hash_password_into(password, salt, &mut output).unwrap();
    start.elapsed()
}

// Test different parameter combinations
let duration_4mb_2iter = benchmark_argon2_params(4, 2);
let duration_16mb_2iter = benchmark_argon2_params(16, 2);
let duration_64mb_3iter = benchmark_argon2_params(64, 3);
```

## Cargo.toml Configuration

```toml
[dependencies]
argon2 = { version = "0.5", features = ["password-hash"] }
ed25519-dalek = "2.1"
rand = "0.8"
zeroize = "1.7"
```

## Constants and Limits

- **Minimum salt length:** 8 bytes
- **Recommended salt length:** 16-32 bytes  
- **Minimum memory cost:** 8 KiB
- **Minimum time cost:** 1 iteration
- **Minimum parallelism:** 1 thread
- **Maximum memory cost:** ~4 GB (platform dependent)

## Best Practices

1. **Use Argon2id variant** for most applications
2. **Generate unique salts** for each key derivation
3. **Test parameters** on target hardware for performance
4. **Store salt with hash** for password verification
5. **Use appropriate memory/time costs** for security requirements
6. **Zeroize sensitive material** after use
7. **Consider parallel execution** for improved performance
8. **Validate input parameters** before processing 