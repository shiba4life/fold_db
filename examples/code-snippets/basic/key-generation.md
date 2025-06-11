# Key Generation - Code Snippets

Complete, working examples for generating secure Ed25519 keypairs for DataFold signature authentication across multiple programming languages.

## 🎯 Overview

These snippets demonstrate secure cryptographic key generation for DataFold signature authentication. All examples use Ed25519 signatures as specified in RFC 9421 and include proper security practices for key storage and management.

## 🔐 Security Requirements

### Cryptographic Standards
- **Algorithm**: Ed25519 (EdDSA using Curve25519)
- **Key Length**: 32 bytes (256 bits)
- **Entropy Source**: Cryptographically secure random number generator (CSPRNG)
- **Format**: Raw bytes, PEM, or DER encoding options

### Security Principles
- Use platform-provided CSPRNG
- Never log or expose private keys
- Store private keys securely (HSM, key vault, encrypted storage)
- Implement proper key rotation policies
- Follow principle of least privilege for key access

## 📚 Language Examples

### JavaScript/TypeScript

#### Basic Key Generation
```typescript
import { generateKeyPair, exportKey, importKey } from '@datafold/signature-auth';
import crypto from 'crypto';

async function generateSecureKeypair() {
  try {
    console.log('🔑 Generating Ed25519 keypair...');
    
    // Generate cryptographically secure Ed25519 keypair
    const keypair = await generateKeyPair();
    
    // Extract raw key bytes
    const privateKeyBytes = await exportKey(keypair.privateKey, 'raw');
    const publicKeyBytes = await exportKey(keypair.publicKey, 'raw');
    
    console.log('✅ Keypair generated successfully');
    console.log(`Private key length: ${privateKeyBytes.length} bytes`);
    console.log(`Public key length: ${publicKeyBytes.length} bytes`);
    
    // Generate key identifier
    const keyId = generateKeyId();
    
    return {
      keyId,
      publicKey: publicKeyBytes,
      privateKey: privateKeyBytes,
      algorithm: 'ed25519',
      createdAt: new Date().toISOString()
    };
    
  } catch (error) {
    console.error('❌ Key generation failed:', error.message);
    throw error;
  }
}

function generateKeyId(): string {
  // Generate unique, collision-resistant key identifier
  const timestamp = Date.now();
  const randomBytes = crypto.randomBytes(8);
  const randomHex = randomBytes.toString('hex');
  
  return `key-${timestamp}-${randomHex}`;
}

// Alternative: Generate key ID from public key hash
function generateKeyIdFromPublicKey(publicKey: Uint8Array): string {
  const hash = crypto.createHash('sha256').update(publicKey).digest();
  return `key-${hash.toString('hex').substring(0, 16)}`;
}

// Usage example
async function main() {
  try {
    const keypair = await generateSecureKeypair();
    
    console.log('\n📋 Generated Keypair:');
    console.log(`Key ID: ${keypair.keyId}`);
    console.log(`Algorithm: ${keypair.algorithm}`);
    console.log(`Created: ${keypair.createdAt}`);
    console.log(`Public Key: ${Buffer.from(keypair.publicKey).toString('hex')}`);
    console.log('Private Key: [REDACTED]');
    
    // Store keys securely (see secure storage examples below)
    await storeKeypairSecurely(keypair);
    
  } catch (error) {
    console.error('Failed to generate keypair:', error);
  }
}

// Secure storage example
async function storeKeypairSecurely(keypair: any) {
  // Option 1: Store in environment variables (development only)
  if (process.env.NODE_ENV === 'development') {
    console.log('\n🔒 Development Storage Instructions:');
    console.log(`Export these environment variables:`);
    console.log(`export DATAFOLD_KEY_ID="${keypair.keyId}"`);
    console.log(`export DATAFOLD_PRIVATE_KEY="${Buffer.from(keypair.privateKey).toString('base64')}"`);
    console.log(`export DATAFOLD_PUBLIC_KEY="${Buffer.from(keypair.publicKey).toString('base64')}"`);
  }
  
  // Option 2: Store in secure key management service
  await storeInKeyVault(keypair);
  
  // Option 3: Store in encrypted file
  await storeInEncryptedFile(keypair);
}

async function storeInKeyVault(keypair: any) {
  // Example: Azure Key Vault, AWS KMS, HashiCorp Vault
  const keyVault = new KeyVaultClient(process.env.KEY_VAULT_URL);
  
  try {
    // Store private key as secret
    await keyVault.setSecret(`${keypair.keyId}-private`, 
      Buffer.from(keypair.privateKey).toString('base64'));
    
    // Store public key (can be less restricted)
    await keyVault.setSecret(`${keypair.keyId}-public`, 
      Buffer.from(keypair.publicKey).toString('base64'));
    
    // Store metadata
    await keyVault.setSecret(`${keypair.keyId}-metadata`, JSON.stringify({
      algorithm: keypair.algorithm,
      createdAt: keypair.createdAt,
      purpose: 'datafold-signature-auth'
    }));
    
    console.log(`✅ Keypair stored in key vault: ${keypair.keyId}`);
    
  } catch (error) {
    console.error('❌ Failed to store in key vault:', error.message);
    throw error;
  }
}
```

#### Advanced Key Generation with Metadata
```typescript
interface KeypairMetadata {
  keyId: string;
  algorithm: string;
  createdAt: string;
  expiresAt?: string;
  purpose: string;
  environment: string;
  clientId?: string;
  version: number;
}

interface SecureKeypair {
  metadata: KeypairMetadata;
  publicKey: Uint8Array;
  privateKey: Uint8Array;
  publicKeyPem: string;
  privateKeyPem: string;
}

class SecureKeyGenerator {
  constructor(private config: {
    environment: string;
    clientId?: string;
    defaultTtl?: number; // days
  }) {}
  
  async generateKeypair(purpose: string = 'authentication'): Promise<SecureKeypair> {
    // Generate the cryptographic keypair
    const keypair = await generateKeyPair();
    
    // Create metadata
    const metadata: KeypairMetadata = {
      keyId: this.generateSecureKeyId(),
      algorithm: 'ed25519',
      createdAt: new Date().toISOString(),
      expiresAt: this.config.defaultTtl ? 
        new Date(Date.now() + this.config.defaultTtl * 24 * 60 * 60 * 1000).toISOString() : 
        undefined,
      purpose,
      environment: this.config.environment,
      clientId: this.config.clientId,
      version: 1
    };
    
    // Export keys in multiple formats
    const privateKeyBytes = await exportKey(keypair.privateKey, 'raw');
    const publicKeyBytes = await exportKey(keypair.publicKey, 'raw');
    const privateKeyPem = await exportKey(keypair.privateKey, 'pem');
    const publicKeyPem = await exportKey(keypair.publicKey, 'pem');
    
    return {
      metadata,
      publicKey: publicKeyBytes,
      privateKey: privateKeyBytes,
      publicKeyPem,
      privateKeyPem
    };
  }
  
  private generateSecureKeyId(): string {
    // Use timestamp + secure random for uniqueness
    const timestamp = Date.now();
    const randomBytes = crypto.randomBytes(16);
    const randomString = randomBytes.toString('base64url');
    
    return `${this.config.environment}-${timestamp}-${randomString}`;
  }
  
  async rotateKeypair(currentKeyId: string): Promise<SecureKeypair> {
    console.log(`🔄 Rotating keypair: ${currentKeyId}`);
    
    // Generate new keypair
    const newKeypair = await this.generateKeypair('authentication');
    
    // Set overlap period for smooth transition
    const now = new Date();
    const overlapPeriod = 7 * 24 * 60 * 60 * 1000; // 7 days
    
    newKeypair.metadata.expiresAt = new Date(now.getTime() + overlapPeriod).toISOString();
    
    return newKeypair;
  }
  
  validateKeypair(keypair: SecureKeypair): boolean {
    // Validate key lengths
    if (keypair.privateKey.length !== 32) {
      throw new Error('Invalid private key length');
    }
    
    if (keypair.publicKey.length !== 32) {
      throw new Error('Invalid public key length');
    }
    
    // Validate expiration
    if (keypair.metadata.expiresAt) {
      const expiresAt = new Date(keypair.metadata.expiresAt);
      if (expiresAt < new Date()) {
        throw new Error('Keypair has expired');
      }
    }
    
    return true;
  }
}

// Usage
const keyGenerator = new SecureKeyGenerator({
  environment: 'production',
  clientId: 'web-app-v1',
  defaultTtl: 90 // 90 days
});

async function generateProductionKeypair() {
  const keypair = await keyGenerator.generateKeypair('api-authentication');
  keyGenerator.validateKeypair(keypair);
  
  console.log('Generated keypair:', keypair.metadata);
  return keypair;
}
```

### Python

#### Basic Key Generation
```python
import secrets
import hashlib
import base64
import json
from datetime import datetime, timedelta
from typing import Dict, Optional, Tuple
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519

def generate_secure_keypair() -> Dict:
    """Generate a secure Ed25519 keypair for DataFold authentication"""
    
    try:
        print('🔑 Generating Ed25519 keypair...')
        
        # Generate cryptographically secure Ed25519 keypair
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        
        # Extract raw key bytes
        private_key_bytes = private_key.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption()
        )
        
        public_key_bytes = public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        
        print('✅ Keypair generated successfully')
        print(f'Private key length: {len(private_key_bytes)} bytes')
        print(f'Public key length: {len(public_key_bytes)} bytes')
        
        # Generate key identifier
        key_id = generate_key_id()
        
        return {
            'key_id': key_id,
            'public_key': public_key_bytes,
            'private_key': private_key_bytes,
            'algorithm': 'ed25519',
            'created_at': datetime.utcnow().isoformat() + 'Z'
        }
        
    except Exception as error:
        print(f'❌ Key generation failed: {error}')
        raise

def generate_key_id() -> str:
    """Generate unique, collision-resistant key identifier"""
    timestamp = int(datetime.utcnow().timestamp() * 1000)
    random_bytes = secrets.token_bytes(8)
    random_hex = random_bytes.hex()
    
    return f'key-{timestamp}-{random_hex}'

def generate_key_id_from_public_key(public_key_bytes: bytes) -> str:
    """Generate deterministic key ID from public key hash"""
    hash_obj = hashlib.sha256(public_key_bytes)
    hash_hex = hash_obj.hexdigest()
    
    return f'key-{hash_hex[:16]}'

async def store_keypair_securely(keypair: Dict) -> None:
    """Store keypair using secure methods"""
    
    # Option 1: Environment variables (development only)
    if os.getenv('ENVIRONMENT') == 'development':
        print('\n🔒 Development Storage Instructions:')
        print('Export these environment variables:')
        print(f'export DATAFOLD_KEY_ID="{keypair["key_id"]}"')
        print(f'export DATAFOLD_PRIVATE_KEY="{base64.b64encode(keypair["private_key"]).decode()}"')
        print(f'export DATAFOLD_PUBLIC_KEY="{base64.b64encode(keypair["public_key"]).decode()}"')
    
    # Option 2: Store in key management service
    await store_in_key_vault(keypair)
    
    # Option 3: Store in encrypted file
    await store_in_encrypted_file(keypair)

async def store_in_key_vault(keypair: Dict) -> None:
    """Store keypair in cloud key management service"""
    # Example: AWS KMS, Azure Key Vault, Google Secret Manager
    
    try:
        key_vault = KeyVaultClient(os.getenv('KEY_VAULT_URL'))
        
        # Store private key as secret
        await key_vault.set_secret(
            f"{keypair['key_id']}-private",
            base64.b64encode(keypair['private_key']).decode()
        )
        
        # Store public key
        await key_vault.set_secret(
            f"{keypair['key_id']}-public",
            base64.b64encode(keypair['public_key']).decode()
        )
        
        # Store metadata
        metadata = {
            'algorithm': keypair['algorithm'],
            'created_at': keypair['created_at'],
            'purpose': 'datafold-signature-auth'
        }
        await key_vault.set_secret(
            f"{keypair['key_id']}-metadata",
            json.dumps(metadata)
        )
        
        print(f'✅ Keypair stored in key vault: {keypair["key_id"]}')
        
    except Exception as error:
        print(f'❌ Failed to store in key vault: {error}')
        raise

# Usage example
async def main():
    try:
        keypair = generate_secure_keypair()
        
        print('\n📋 Generated Keypair:')
        print(f'Key ID: {keypair["key_id"]}')
        print(f'Algorithm: {keypair["algorithm"]}')
        print(f'Created: {keypair["created_at"]}')
        print(f'Public Key: {keypair["public_key"].hex()}')
        print('Private Key: [REDACTED]')
        
        # Store keys securely
        await store_keypair_securely(keypair)
        
    except Exception as error:
        print(f'Failed to generate keypair: {error}')

if __name__ == '__main__':
    import asyncio
    asyncio.run(main())
```

#### Advanced Key Generation with Management
```python
from dataclasses import dataclass
from typing import Optional, List
import os
import json
from pathlib import Path

@dataclass
class KeypairMetadata:
    key_id: str
    algorithm: str
    created_at: str
    expires_at: Optional[str]
    purpose: str
    environment: str
    client_id: Optional[str]
    version: int

@dataclass
class SecureKeypair:
    metadata: KeypairMetadata
    public_key: bytes
    private_key: bytes
    public_key_pem: str
    private_key_pem: str

class SecureKeyGenerator:
    def __init__(self, environment: str, client_id: Optional[str] = None, default_ttl_days: int = 90):
        self.environment = environment
        self.client_id = client_id
        self.default_ttl_days = default_ttl_days
    
    def generate_keypair(self, purpose: str = 'authentication') -> SecureKeypair:
        """Generate a secure keypair with comprehensive metadata"""
        
        # Generate the cryptographic keypair
        private_key = ed25519.Ed25519PrivateKey.generate()
        public_key = private_key.public_key()
        
        # Create metadata
        now = datetime.utcnow()
        expires_at = None
        if self.default_ttl_days:
            expires_at = (now + timedelta(days=self.default_ttl_days)).isoformat() + 'Z'
        
        metadata = KeypairMetadata(
            key_id=self._generate_secure_key_id(),
            algorithm='ed25519',
            created_at=now.isoformat() + 'Z',
            expires_at=expires_at,
            purpose=purpose,
            environment=self.environment,
            client_id=self.client_id,
            version=1
        )
        
        # Export keys in multiple formats
        private_key_bytes = private_key.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption()
        )
        
        public_key_bytes = public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        
        private_key_pem = private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        ).decode('utf-8')
        
        public_key_pem = public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        ).decode('utf-8')
        
        return SecureKeypair(
            metadata=metadata,
            public_key=public_key_bytes,
            private_key=private_key_bytes,
            public_key_pem=private_key_pem,
            private_key_pem=public_key_pem
        )
    
    def _generate_secure_key_id(self) -> str:
        """Generate a secure, unique key identifier"""
        timestamp = int(datetime.utcnow().timestamp() * 1000)
        random_bytes = secrets.token_bytes(16)
        random_string = base64.urlsafe_b64encode(random_bytes).decode().rstrip('=')
        
        return f'{self.environment}-{timestamp}-{random_string}'
    
    def rotate_keypair(self, current_key_id: str) -> SecureKeypair:
        """Generate a new keypair for rotation"""
        print(f'🔄 Rotating keypair: {current_key_id}')
        
        # Generate new keypair with overlap period
        new_keypair = self.generate_keypair('authentication')
        
        # Set overlap period for smooth transition (7 days)
        overlap_period = timedelta(days=7)
        new_keypair.metadata.expires_at = (
            datetime.utcnow() + overlap_period
        ).isoformat() + 'Z'
        
        return new_keypair
    
    def validate_keypair(self, keypair: SecureKeypair) -> bool:
        """Validate keypair integrity and expiration"""
        
        # Validate key lengths
        if len(keypair.private_key) != 32:
            raise ValueError('Invalid private key length')
        
        if len(keypair.public_key) != 32:
            raise ValueError('Invalid public key length')
        
        # Validate expiration
        if keypair.metadata.expires_at:
            expires_at = datetime.fromisoformat(keypair.metadata.expires_at.rstrip('Z'))
            if expires_at < datetime.utcnow():
                raise ValueError('Keypair has expired')
        
        return True

# Secure key storage utilities
class KeypairStorage:
    @staticmethod
    def save_to_file(keypair: SecureKeypair, file_path: str, password: Optional[str] = None):
        """Save keypair to encrypted file"""
        
        data = {
            'metadata': keypair.metadata.__dict__,
            'public_key': base64.b64encode(keypair.public_key).decode(),
            'private_key': base64.b64encode(keypair.private_key).decode(),
            'public_key_pem': keypair.public_key_pem,
            'private_key_pem': keypair.private_key_pem
        }
        
        if password:
            # Encrypt the data before saving
            encrypted_data = encrypt_data(json.dumps(data), password)
            with open(file_path, 'wb') as f:
                f.write(encrypted_data)
        else:
            # Save as plain JSON (not recommended for production)
            with open(file_path, 'w') as f:
                json.dump(data, f, indent=2)
    
    @staticmethod
    def load_from_file(file_path: str, password: Optional[str] = None) -> SecureKeypair:
        """Load keypair from file"""
        
        if password:
            # Decrypt the data
            with open(file_path, 'rb') as f:
                encrypted_data = f.read()
            decrypted_json = decrypt_data(encrypted_data, password)
            data = json.loads(decrypted_json)
        else:
            # Load plain JSON
            with open(file_path, 'r') as f:
                data = json.load(f)
        
        metadata = KeypairMetadata(**data['metadata'])
        
        return SecureKeypair(
            metadata=metadata,
            public_key=base64.b64decode(data['public_key']),
            private_key=base64.b64decode(data['private_key']),
            public_key_pem=data['public_key_pem'],
            private_key_pem=data['private_key_pem']
        )

# Usage
async def generate_production_keypair():
    key_generator = SecureKeyGenerator(
        environment='production',
        client_id='api-service-v1',
        default_ttl_days=90
    )
    
    keypair = key_generator.generate_keypair('api-authentication')
    key_generator.validate_keypair(keypair)
    
    print('Generated keypair:', keypair.metadata)
    return keypair
```

### Rust

#### Basic Key Generation
```rust
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct GeneratedKeypair {
    key_id: String,
    algorithm: String,
    created_at: String,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

fn generate_secure_keypair() -> Result<GeneratedKeypair, Box<dyn std::error::Error>> {
    println!("🔑 Generating Ed25519 keypair...");
    
    // Use OS random number generator for cryptographic security
    let mut csprng = OsRng;
    
    // Generate Ed25519 keypair
    let keypair = Keypair::generate(&mut csprng);
    
    // Extract key bytes
    let private_key_bytes = keypair.secret.to_bytes().to_vec();
    let public_key_bytes = keypair.public.to_bytes().to_vec();
    
    println!("✅ Keypair generated successfully");
    println!("Private key length: {} bytes", private_key_bytes.len());
    println!("Public key length: {} bytes", public_key_bytes.len());
    
    // Generate key identifier
    let key_id = generate_key_id()?;
    
    let result = GeneratedKeypair {
        key_id,
        algorithm: "ed25519".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        public_key: public_key_bytes,
        private_key: private_key_bytes,
    };
    
    Ok(result)
}

fn generate_key_id() -> Result<String, Box<dyn std::error::Error>> {
    // Generate unique key identifier
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    
    let mut random_bytes = [0u8; 8];
    getrandom::getrandom(&mut random_bytes)?;
    let random_hex = hex::encode(random_bytes);
    
    Ok(format!("key-{}-{}", timestamp, random_hex))
}

fn generate_key_id_from_public_key(public_key: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();
    let hash_hex = hex::encode(&hash[..8]); // First 8 bytes
    
    format!("key-{}", hash_hex)
}

fn store_keypair_securely(keypair: &GeneratedKeypair) -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Environment variables (development only)
    if std::env::var("ENVIRONMENT").unwrap_or_default() == "development" {
        println!("\n🔒 Development Storage Instructions:");
        println!("Export these environment variables:");
        println!("export DATAFOLD_KEY_ID=\"{}\"", keypair.key_id);
        println!("export DATAFOLD_PRIVATE_KEY=\"{}\"", 
                general_purpose::STANDARD.encode(&keypair.private_key));
        println!("export DATAFOLD_PUBLIC_KEY=\"{}\"", 
                general_purpose::STANDARD.encode(&keypair.public_key));
    }
    
    // Option 2: Store in secure file
    store_in_encrypted_file(keypair)?;
    
    Ok(())
}

fn store_in_encrypted_file(keypair: &GeneratedKeypair) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;
    
    // Create secure storage directory
    let storage_dir = Path::new("./secure-keys");
    fs::create_dir_all(storage_dir)?;
    
    // Store metadata (safe to store unencrypted)
    let metadata = serde_json::json!({
        "key_id": keypair.key_id,
        "algorithm": keypair.algorithm,
        "created_at": keypair.created_at,
        "public_key": general_purpose::STANDARD.encode(&keypair.public_key)
    });
    
    let metadata_path = storage_dir.join(format!("{}-metadata.json", keypair.key_id));
    fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    // Store private key (should be encrypted in production)
    let private_key_path = storage_dir.join(format!("{}-private.key", keypair.key_id));
    
    // In production, encrypt the private key before storage
    let private_key_b64 = general_purpose::STANDARD.encode(&keypair.private_key);
    fs::write(private_key_path, private_key_b64)?;
    
    // Set restrictive permissions (Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&private_key_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600); // Read/write owner only
        fs::set_permissions(&private_key_path, permissions)?;
    }
    
    println!("✅ Keypair stored securely: {}", keypair.key_id);
    
    Ok(())
}

// Advanced key generation with metadata
#[derive(Debug, Serialize, Deserialize)]
struct KeypairMetadata {
    key_id: String,
    algorithm: String,
    created_at: String,
    expires_at: Option<String>,
    purpose: String,
    environment: String,
    client_id: Option<String>,
    version: u32,
}

#[derive(Debug)]
struct SecureKeypair {
    metadata: KeypairMetadata,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    public_key_pem: String,
    private_key_pem: String,
}

struct SecureKeyGenerator {
    environment: String,
    client_id: Option<String>,
    default_ttl_days: Option<u32>,
}

impl SecureKeyGenerator {
    fn new(environment: String, client_id: Option<String>, default_ttl_days: Option<u32>) -> Self {
        Self {
            environment,
            client_id,
            default_ttl_days,
        }
    }
    
    fn generate_keypair(&self, purpose: &str) -> Result<SecureKeypair, Box<dyn std::error::Error>> {
        // Generate cryptographic keypair
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        
        // Create metadata
        let now = chrono::Utc::now();
        let expires_at = self.default_ttl_days.map(|days| {
            (now + chrono::Duration::days(days as i64)).to_rfc3339()
        });
        
        let metadata = KeypairMetadata {
            key_id: self.generate_secure_key_id()?,
            algorithm: "ed25519".to_string(),
            created_at: now.to_rfc3339(),
            expires_at,
            purpose: purpose.to_string(),
            environment: self.environment.clone(),
            client_id: self.client_id.clone(),
            version: 1,
        };
        
        // Extract key bytes
        let private_key_bytes = keypair.secret.to_bytes().to_vec();
        let public_key_bytes = keypair.public.to_bytes().to_vec();
        
        // Convert to PEM format (simplified - use proper PEM encoding in production)
        let private_key_pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
            general_purpose::STANDARD.encode(&private_key_bytes)
        );
        
        let public_key_pem = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            general_purpose::STANDARD.encode(&public_key_bytes)
        );
        
        Ok(SecureKeypair {
            metadata,
            public_key: public_key_bytes,
            private_key: private_key_bytes,
            public_key_pem,
            private_key_pem,
        })
    }
    
    fn generate_secure_key_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        
        let mut random_bytes = [0u8; 16];
        getrandom::getrandom(&mut random_bytes)?;
        let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
        
        Ok(format!("{}-{}-{}", self.environment, timestamp, random_string))
    }
    
    fn validate_keypair(&self, keypair: &SecureKeypair) -> Result<bool, Box<dyn std::error::Error>> {
        // Validate key lengths
        if keypair.private_key.len() != 32 {
            return Err("Invalid private key length".into());
        }
        
        if keypair.public_key.len() != 32 {
            return Err("Invalid public key length".into());
        }
        
        // Validate expiration
        if let Some(expires_at) = &keypair.metadata.expires_at {
            let expires = chrono::DateTime::parse_from_rfc3339(expires_at)?;
            if expires < chrono::Utc::now() {
                return Err("Keypair has expired".into());
            }
        }
        
        Ok(true)
    }
}

// Usage example
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic key generation
    let keypair = generate_secure_keypair()?;
    
    println!("\n📋 Generated Keypair:");
    println!("Key ID: {}", keypair.key_id);
    println!("Algorithm: {}", keypair.algorithm);
    println!("Created: {}", keypair.created_at);
    println!("Public Key: {}", hex::encode(&keypair.public_key));
    println!("Private Key: [REDACTED]");
    
    // Store securely
    store_keypair_securely(&keypair)?;
    
    // Advanced key generation
    let key_generator = SecureKeyGenerator::new(
        "production".to_string(),
        Some("api-service-v1".to_string()),
        Some(90), // 90 days
    );
    
    let advanced_keypair = key_generator.generate_keypair("api-authentication")?;
    key_generator.validate_keypair(&advanced_keypair)?;
    
    println!("\n🔧 Advanced keypair generated: {}", advanced_keypair.metadata.key_id);
    
    Ok(())
}
```

## 🔒 Security Best Practices

### Entropy Sources
```javascript
// ✅ Good: Use cryptographically secure random sources
const crypto = require('crypto');
const secureRandom = crypto.randomBytes(32);

// ✅ Good: Platform-provided CSPRNG
const keypair = await generateKeyPair(); // Uses Web Crypto API or Node.js crypto

// ❌ Bad: Predictable random sources
const badRandom = Math.random(); // Never use for keys!
```

### Key Storage
```javascript
// ✅ Good: Secure key storage options
const storeKey = async (keyId, privateKey) => {
  // Option 1: Hardware Security Module
  await hsm.storeKey(keyId, privateKey);
  
  // Option 2: Key Management Service
  await keyVault.setSecret(keyId, privateKey);
  
  // Option 3: Encrypted file with strong encryption
  await encryptAndStore(keyId, privateKey, strongPassword);
};

// ❌ Bad: Insecure storage
const badStorage = {
  // Never store in plain text files
  fs.writeFileSync('private-key.txt', privateKey),
  
  // Never hardcode in source code
  const privateKey = 'hardcoded-key-data',
  
  // Never log private keys
  console.log('Private key:', privateKey)
};
```

### Key Rotation
```javascript
// ✅ Good: Automated key rotation
class KeyRotationManager {
  async rotateKeys(currentKeyId) {
    // Generate new keypair
    const newKeypair = await generateSecureKeypair();
    
    // Overlap period for smooth transition
    const overlapPeriod = 7 * 24 * 60 * 60 * 1000; // 7 days
    
    // Update key registry
    await this.keyRegistry.addKey(newKeypair, {
      expiresAt: Date.now() + overlapPeriod
    });
    
    // Schedule old key deactivation
    setTimeout(() => {
      this.keyRegistry.deactivateKey(currentKeyId);
    }, overlapPeriod);
    
    return newKeypair;
  }
}
```

## 🧪 Testing

### Key Generation Tests
```javascript
describe('Key Generation', () => {
  it('should generate valid Ed25519 keypairs', async () => {
    const keypair = await generateSecureKeypair();
    
    expect(keypair.privateKey).toHaveLength(32);
    expect(keypair.publicKey).toHaveLength(32);
    expect(keypair.algorithm).toBe('ed25519');
    expect(keypair.keyId).toMatch(/^key-\d+-[a-f0-9]+$/);
  });
  
  it('should generate unique key IDs', async () => {
    const keypair1 = await generateSecureKeypair();
    const keypair2 = await generateSecureKeypair();
    
    expect(keypair1.keyId).not.toBe(keypair2.keyId);
  });
  
  it('should create valid signatures', async () => {
    const keypair = await generateSecureKeypair();
    const message = 'test message';
    
    // Sign with private key
    const signature = await sign(message, keypair.privateKey);
    
    // Verify with public key
    const isValid = await verify(signature, message, keypair.publicKey);
    expect(isValid).toBe(true);
  });
});
```

## 🔗 Related Snippets

- **[Signature Creation](signature-creation.md)** - Use generated keys for signing
- **[Signature Verification](signature-verification.md)** - Verify signatures with public keys
- **[Secure Storage](../security/secure-storage.md)** - Store keys securely
- **[Key Rotation](../edge-cases/key-rotation.md)** - Rotate keys safely

## 🆘 Troubleshooting

### Common Issues

**"Insufficient entropy"**
```javascript
// Ensure proper entropy source
if (!crypto.getRandomValues) {
  throw new Error('Cryptographically secure random not available');
}
```

**"Invalid key length"**
```javascript
// Verify key generation
const keypair = await generateKeyPair();
console.log('Private key length:', keypair.privateKey.length); // Should be 32
console.log('Public key length:', keypair.publicKey.length);   // Should be 32
```

**"Key storage failed"**
```javascript
// Check permissions and storage availability
try {
  await storeKey(keyId, privateKey);
} catch (error) {
  if (error.code === 'EACCES') {
    console.error('Permission denied - check file/directory permissions');
  } else if (error.code === 'ENOSPC') {
    console.error('Insufficient storage space');
  }
}
```

## 📄 License

These code snippets are provided under the MIT license for maximum reusability.