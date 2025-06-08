# DataFold Database Security Report

## Executive Summary

This report outlines comprehensive security proposals for the DataFold distributed database, focusing on public/private key encryption, encryption at rest, and signed message authentication for network access.

## Current Security Architecture

### Existing Security Features

The DataFold project already implements several security components:

1. **Permission System** ([`src/permissions/permission_manager.rs`](../src/permissions/permission_manager.rs)) - Public key-based access control
2. **Trust Distance Model** - Network-based access control using trust relationships
3. **P2P Security** - libp2p with built-in encryption and authentication
4. **Lightning Network Integration** - Secure payment channels

### Security Gaps Identified

1. **No encryption at rest** - Database files stored in plaintext
2. **Missing database-level key initialization** - No master key pair for database instance
3. **No signed message authentication** - Network access not requiring message signatures
4. **Limited cryptographic key management** - No key rotation

## Security Enhancement Proposals

### Proposal 1: Database Initialization with Master Key Pair

#### Implementation Approach

**1.1 Master Key Generation**
```
// Key generation is performed client-side.
// The server only receives and stores the public key portion,
// and uses it for verifying signatures and managing permissions.
// Clients should generate Ed25519 key pairs using browser APIs,
// OpenSSL, or language-specific crypto libraries.
```

**1.2 Database Initialization Process**
```rust
// Enhanced NodeConfig initialization expects a provided public key
```

### Proposal 2: Encryption at Rest

#### Implementation Strategy

**2.1 Database Encryption Layer**
```rust
// New module: src/crypto/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;

pub struct DatabaseEncryption {
    cipher: Aes256Gcm,
    master_key: Key<Aes256Gcm>,
}

impl DatabaseEncryption {
    pub fn new(master_keypair: &MasterKeyPair, passphrase: &str) -> Self {
        // Derive encryption key from master private key + passphrase
        let salt = blake3::hash(&master_keypair.public_key.to_bytes()).as_bytes()[..16];
        let mut key_material = [0u8; 32];
        
        Argon2::default()
            .hash_password_into(
                passphrase.as_bytes(),
                salt,
                &mut key_material
            )
            .expect("Key derivation failed");
            
        let master_key = Key::<Aes256Gcm>::from_slice(&key_material);
        let cipher = Aes256Gcm::new(master_key);
        
        Self { cipher, master_key: *master_key }
    }
    
    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        let nonce = Nonce::from_slice(&self.generate_nonce());
        self.cipher.encrypt(nonce, data).expect("Encryption failed")
    }
    
    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> FoldDbResult<Vec<u8>> {
        // Extract nonce and decrypt
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| FoldDbError::Crypto(format!("Decryption failed: {}", e)))
    }

    /// Export encrypted data to a file for backup.
    pub fn export_encrypted_data(&self, data: &[u8], out_path: &Path) -> FoldDbResult<()> {
        let encrypted = self.encrypt_data(data);
        std::fs::write(out_path, &encrypted)?;
        Ok(())
    }

    /// Import encrypted data from a backup file and decrypt it.
    pub fn import_encrypted_data(&self, in_path: &Path) -> FoldDbResult<Vec<u8>> {
        let encrypted = std::fs::read(in_path)?;
        self.decrypt_data(&encrypted)
    }
}
```


**2.2 Storage Layer Integration**
```rust
// Enhanced database operations with encryption
impl FoldDB {
    pub fn store_encrypted_atom(&mut self, atom_data: &[u8]) -> FoldDbResult<String> {
        let encrypted_data = self.encryption.encrypt_data(atom_data);
        let atom_id = self.store_raw_data(&encrypted_data)?;
        Ok(atom_id)
    }
    
    pub fn retrieve_encrypted_atom(&self, atom_id: &str) -> FoldDbResult<Vec<u8>> {
        let encrypted_data = self.retrieve_raw_data(atom_id)?;
        self.encryption.decrypt_data(&encrypted_data)
    }
}
```

### 2.3 Encrypted Backup and Restore

To enable secure backups and portability, encrypted atom data can be exported and re-imported using:

```rust
let encryption = DatabaseEncryption::new(&master_keypair, "passphrase");

// Backup
encryption.export_encrypted_data(&atom_data, Path::new("backup.atom"))?;

// Restore
let restored = encryption.import_encrypted_data(Path::new("backup.atom"))?;
```

This allows encrypted data to be securely saved to disk or transferred between nodes, with decryption only possible using the original public key and passphrase combination.

### Proposal 3: Signed Message Authentication

#### Authentication Protocol

**3.1 Message Signing Requirements**
```rust
// New module: src/crypto/authentication.rs
#[derive(Serialize, Deserialize)]
pub struct SignedRequest {
    pub timestamp: u64,
    pub nonce: String,
    pub payload: serde_json::Value,
    pub signature: String,
    pub public_key: String,
}

impl SignedRequest {
    pub fn new(payload: serde_json::Value, keypair: &MasterKeyPair) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = uuid::Uuid::new_v4().to_string();
        
        // Create message to sign
        let message = format!("{}:{}:{}", timestamp, nonce, payload.to_string());
        let signature = keypair.sign_message(message.as_bytes());
        
        Self {
            timestamp,
            nonce,
            payload,
            signature: base64::encode(signature.to_bytes()),
            public_key: base64::encode(keypair.public_key.to_bytes()),
        }
    }
    
    pub fn verify(&self) -> FoldDbResult<bool> {
        // Check timestamp (prevent replay attacks)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if current_time - self.timestamp > 300 { // 5 minute window
            return Ok(false);
        }
        
        // Verify signature
        let public_key = ed25519_dalek::PublicKey::from_bytes(
            &base64::decode(&self.public_key)?
        )?;
        
        let signature = ed25519_dalek::Signature::from_bytes(
            &base64::decode(&self.signature)?
        )?;
        
        let message = format!("{}:{}:{}", self.timestamp, self.nonce, self.payload.to_string());
        
        Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
    }
}
```

**3.2 HTTP API Authentication**
```rust
// Enhanced HTTP routes with signature verification
pub async fn authenticated_query(
    req: web::Json<SignedRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Verify signature
    if !req.verify()? {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Invalid signature or expired request"
        })));
    }
    
    // Check permissions
    let node = data.node.lock().await;
    let permission_manager = PermissionManager::new();
    
    if !permission_manager.has_read_permission(
        &req.public_key,
        &schema_permissions,
        trust_distance,
    ) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Insufficient permissions"
        })));
    }
    
    // Process request
    let result = node.execute_operation(&req.payload).await?;
    Ok(HttpResponse::Ok().json(result))
}
```

### Proposal 4: Comprehensive Key Management

#### Key Management System

**4.1 Key Rotation Strategy**
```rust
// Key rotation implementation
pub struct KeyRotationManager {
    current_keys: MasterKeyPair,
    previous_keys: Vec<MasterKeyPair>,
    rotation_schedule: Duration,
}

impl KeyRotationManager {
    pub fn rotate_keys(&mut self) -> FoldDbResult<()> {
        // Generate new key pair
        let new_keys = MasterKeyPair::generate();
        
        // Archive current keys
        self.previous_keys.push(self.current_keys.clone());
        
        // Update current keys
        self.current_keys = new_keys;
        
        // Re-encrypt critical data with new keys
        self.re_encrypt_database()?;
        
        Ok(())
    }
    
    pub fn verify_with_any_key(&self, message: &[u8], signature: &Signature) -> bool {
        // Try current key first
        if self.current_keys.public_key.verify(message, signature).is_ok() {
            return true;
        }
        
        // Try previous keys for backward compatibility
        for old_keys in &self.previous_keys {
            if old_keys.public_key.verify(message, signature).is_ok() {
                return true;
            }
        }
        
        false
    }
}
```


## Implementation Roadmap

### Phase 1: Core Cryptographic Infrastructure (2-3 weeks)
1. Implement master key pair generation and storage
2. Add encryption at rest for database files
3. Create signed message authentication framework
4. Update database initialization process

### Phase 2: Network Security Enhancement (2-3 weeks)
1. Integrate signed authentication with HTTP API
2. Enhance P2P networking with additional verification
3. Implement key rotation mechanisms
4. Add security audit logging

### Phase 3: Advanced Security Features (3-4 weeks)
2. Multi-signature support for critical operations
3. Security monitoring and alerting system
4. Comprehensive security testing suite

## Security Configuration Example

```json
{
  "security": {
    "encryption_at_rest": {
      "enabled": true,
      "algorithm": "AES-256-GCM",
      "key_derivation": "Argon2id"
    },
    "authentication": {
      "require_signatures": true,
      "signature_algorithm": "Ed25519",
      "max_request_age": 300,
      "nonce_cache_size": 10000
    },
    "key_management": {
      "rotation_enabled": true,
      "rotation_interval": "30d",
      "backup_encryption": true
    },
    "network_security": {
      "tls_required": true,
      "peer_verification": "strict",
      "certificate_pinning": true
    }
  }
}
```

## Security Benefits

1. **Data Confidentiality**: Encryption at rest protects against physical access
2. **Authentication**: Public key signatures prevent unauthorized access
3. **Non-repudiation**: All operations are cryptographically signed
4. **Forward Secrecy**: Key rotation limits exposure from key compromise
5. **Audit Trail**: All security events are logged for compliance
6. **Zero-Trust Architecture**: Every request must be authenticated and authorized

## Client Implementation Example

> All key generation is done client-side. Clients are responsible for securely generating, storing, and backing up their own private keys.

### Generating Client Keys
```bash
# Generate client key pair
openssl genpkey -algorithm Ed25519 -out client_private.pem
openssl pkey -in client_private.pem -pubout -out client_public.pem

# Extract public key for registration
CLIENT_PUBLIC_KEY=$(openssl pkey -in client_public.pem -pubin -outform DER | base64 -w 0)
echo "Client Public Key: $CLIENT_PUBLIC_KEY"
```

### Signing Requests (JavaScript/Node.js)
```javascript
const crypto = require('crypto');
const { v4: uuidv4 } = require('uuid');

class DataFoldClient {
    constructor(privateKeyPem, publicKeyPem) {
        this.privateKey = crypto.createPrivateKey(privateKeyPem);
        this.publicKey = crypto.createPublicKey(publicKeyPem);
        this.publicKeyBase64 = this.publicKey
            .export({ format: 'der', type: 'spki' })
            .toString('base64');
    }
    
    signRequest(payload) {
        const timestamp = Math.floor(Date.now() / 1000);
        const nonce = uuidv4();
        const message = `${timestamp}:${nonce}:${JSON.stringify(payload)}`;
        
        const signature = crypto
            .sign(null, Buffer.from(message), this.privateKey)
            .toString('base64');
        
        return {
            timestamp,
            nonce,
            payload,
            signature,
            public_key: this.publicKeyBase64
        };
    }
    
    async query(endpoint, payload) {
        const signedRequest = this.signRequest(payload);
        
        const response = await fetch(`http://localhost:9001/api/${endpoint}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(signedRequest)
        });
        
        return response.json();
    }
}

// Usage example
const fs = require('fs');
const client = new DataFoldClient(
    fs.readFileSync('client_private.pem'),
    fs.readFileSync('client_public.pem')
);

// Query data with signed request
client.query('query', {
    schema: 'UserProfile',
    fields: ['username', 'email'],
    filter: { field: 'active', value: true }
}).then(result => {
    console.log('Query result:', result);
});
```

### Python Client Implementation
```python
import json
import time
import uuid
import base64
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.primitives.asymmetric import ed25519
import requests

class DataFoldClient:
    def __init__(self, private_key_path, public_key_path):
        with open(private_key_path, 'rb') as f:
            self.private_key = serialization.load_pem_private_key(
                f.read(), password=None
            )
        
        with open(public_key_path, 'rb') as f:
            public_key_pem = f.read()
            public_key = serialization.load_pem_public_key(public_key_pem)
            self.public_key_base64 = base64.b64encode(
                public_key.public_bytes(
                    encoding=serialization.Encoding.DER,
                    format=serialization.PublicFormat.SubjectPublicKeyInfo
                )
            ).decode()
    
    def sign_request(self, payload):
        timestamp = int(time.time())
        nonce = str(uuid.uuid4())
        message = f"{timestamp}:{nonce}:{json.dumps(payload, sort_keys=True)}"
        
        signature = self.private_key.sign(message.encode())
        
        return {
            'timestamp': timestamp,
            'nonce': nonce,
            'payload': payload,
            'signature': base64.b64encode(signature).decode(),
            'public_key': self.public_key_base64
        }
    
    def query(self, endpoint, payload):
        signed_request = self.sign_request(payload)
        
        response = requests.post(
            f'http://localhost:9001/api/{endpoint}',
            json=signed_request,
            headers={'Content-Type': 'application/json'}
        )
        
        return response.json()

# Usage example
client = DataFoldClient('client_private.pem', 'client_public.pem')

result = client.query('query', {
    'schema': 'UserProfile',
    'fields': ['username', 'email'],
    'filter': {'field': 'active', 'value': True}
})

print('Query result:', result)
```

## Database Setup with Security

### Secure Initialization Script
```bash
#!/bin/bash

# DataFold Secure Setup Script

echo "Setting up DataFold with enhanced security..."

# Generate master database keys
echo "Generating master key pair..."
cargo run --bin datafold_cli -- init-crypto \
    --storage-path ./secure_data \
    --passphrase-file ./master.passphrase

# Set restrictive permissions on data directory
chmod 700 ./secure_data
chmod 600 ./secure_data/*.key

# Generate client access keys
echo "Generating client access keys..."
mkdir -p ./client_keys
openssl genpkey -algorithm Ed25519 -out ./client_keys/admin_private.pem
openssl pkey -in ./client_keys/admin_private.pem -pubout -out ./client_keys/admin_public.pem

# Set permissions for schemas
echo "Configuring initial permissions..."
cargo run --bin datafold_cli -- set-permission \
    --schema "UserProfile" \
    --field "email" \
    --read-permission "explicit:admin_access" \
    --write-permission "distance:0"

echo "Security setup complete!"
echo "Admin public key: $(openssl pkey -in ./client_keys/admin_public.pem -pubin -outform DER | base64 -w 0)"
```

## Conclusion

These proposals transform DataFold into a highly secure distributed database suitable for enterprise and high-security deployments. The implementation builds upon existing security infrastructure while adding comprehensive encryption and authentication capabilities.

The security enhancements provide:
- **End-to-end encryption** for data at rest and in transit
- **Cryptographic authentication** for all network operations
- **Flexible key management** with rotation
- **Audit trails** for all security-related operations
- **Client libraries** for easy integration

This security framework ensures DataFold can handle sensitive data while maintaining the distributed, peer-to-peer architecture that makes it unique.

## Database Setup via HTTP Server

### Current Database Setup Process

The DataFold HTTP server provides several endpoints for database initialization and management:

#### Available HTTP Endpoints

```http
# System Management
GET  /api/system/status           # Check system status
POST /api/system/reset-database   # Reset database (destructive)

# Schema Management
GET  /api/schemas                 # List all schemas
POST /api/schema                  # Create new schema
POST /api/schema/{name}/load      # Load schema into database
GET  /api/schema/{name}           # Get schema details

# Network Management
POST /api/network/init            # Initialize network layer
POST /api/network/start           # Start network services
GET  /api/network/status          # Check network status
```

#### Current Setup Flow

1. **Check System Status**
```bash
curl http://localhost:9001/api/system/status
```

2. **Create Schema**
```bash
curl -X POST http://localhost:9001/api/schema \
  -H "Content-Type: application/json" \
  -d '{
    "name": "UserProfile",
    "fields": {
      "username": {"type": "string"},
      "email": {"type": "string"},
      "age": {"type": "number"}
    }
  }'
```

3. **Load Schema**
```bash
curl -X POST http://localhost:9001/api/schema/UserProfile/load
```

4. **Initialize Network**
```bash
curl -X POST http://localhost:9001/api/network/init \
  -H "Content-Type: application/json" \
  -d '{
    "port": 9000,
    "enable_mdns": true
  }'
```

### Enhanced Secure Database Setup Process

With the proposed security enhancements, the database setup process would include cryptographic initialization:

#### Client-Side Key Generation (Primary Approach)

**Security Philosophy:**
- **Private keys never leave the client** - Maximum security
- **Zero-trust architecture** - Server never sees private keys
- **User controls their identity** - True decentralization
- **Forward secrecy** - Compromised server doesn't expose client keys

#### Client-Side Generation Methods

**1. Browser/Web Application:**
```javascript
// DataFold Web Client Key Generation
class DataFoldKeyManager {
    constructor() {
        this.keyPair = null;
        this.publicKeyB64 = null;
    }
    
    async generateKeys() {
        // Generate Ed25519 key pair in browser
        this.keyPair = await crypto.subtle.generateKey(
            {
                name: "Ed25519",
                namedCurve: "Ed25519"
            },
            true, // extractable for backup
            ["sign", "verify"]
        );
        
        // Export public key for server registration
        const publicKeyBuffer = await crypto.subtle.exportKey(
            "spki",
            this.keyPair.publicKey
        );
        this.publicKeyB64 = btoa(
            String.fromCharCode(...new Uint8Array(publicKeyBuffer))
        );
        
        return {
            publicKey: this.publicKeyB64,
            keyPair: this.keyPair
        };
    }
    
    async signMessage(message) {
        const encoder = new TextEncoder();
        const data = encoder.encode(message);
        const signature = await crypto.subtle.sign(
            "Ed25519",
            this.keyPair.privateKey,
            data
        );
        return btoa(String.fromCharCode(...new Uint8Array(signature)));
    }
    
    async exportPrivateKey(password) {
        // Export private key for backup (encrypted with user password)
        const privateKeyBuffer = await crypto.subtle.exportKey(
            "pkcs8",
            this.keyPair.privateKey
        );
        
        // Encrypt with user password using PBKDF2 + AES-GCM
        const passwordKey = await this.deriveKeyFromPassword(password);
        const encrypted = await crypto.subtle.encrypt(
            {
                name: "AES-GCM",
                iv: crypto.getRandomValues(new Uint8Array(12))
            },
            passwordKey,
            privateKeyBuffer
        );
        
        return btoa(String.fromCharCode(...new Uint8Array(encrypted)));
    }
    
    async registerWithServer(apiEndpoint) {
        const response = await fetch(`${apiEndpoint}/api/crypto/client/register`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                public_key: `ed25519:${this.publicKeyB64}`,
                client_type: "web_browser",
                generated_at: new Date().toISOString()
            })
        });
        
        if (!response.ok) {
            throw new Error(`Registration failed: ${response.statusText}`);
        }
        
        return await response.json();
    }
}

// Usage example
const keyManager = new DataFoldKeyManager();
await keyManager.generateKeys();
await keyManager.registerWithServer('http://localhost:9001');
```

**2. Desktop/Mobile Applications:**
```python
# Python Client Application Key Generation
import os
import json
import base64
import getpass
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.fernet import Fernet

class DataFoldClientKeys:
    def __init__(self):
        self.private_key = None
        self.public_key = None
        self.public_key_b64 = None
    
    def generate_keys(self):
        """Generate new Ed25519 key pair"""
        self.private_key = ed25519.Ed25519PrivateKey.generate()
        self.public_key = self.private_key.public_key()
        
        # Export public key for server registration
        public_key_der = self.public_key.public_bytes(
            encoding=serialization.Encoding.DER,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        )
        self.public_key_b64 = base64.b64encode(public_key_der).decode()
        
        print(f"🔑 Generated new key pair")
        print(f"📝 Public key: ed25519:{self.public_key_b64}")
        
        return self.public_key_b64
    
    def save_keys_encrypted(self, filepath, password=None):
        """Save private key encrypted with password"""
        if not password:
            password = getpass.getpass("Enter password to encrypt private key: ")
        
        # Derive key from password
        salt = os.urandom(16)
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            iterations=100000,
        )
        key = base64.urlsafe_b64encode(kdf.derive(password.encode()))
        
        # Encrypt private key
        fernet = Fernet(key)
        private_key_pem = self.private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        )
        
        encrypted_key = fernet.encrypt(private_key_pem)
        
        # Save to file
        key_data = {
            'encrypted_private_key': base64.b64encode(encrypted_key).decode(),
            'public_key': self.public_key_b64,
            'salt': base64.b64encode(salt).decode(),
            'created_at': str(datetime.now()),
            'key_type': 'ed25519'
        }
        
        with open(filepath, 'w') as f:
            json.dump(key_data, f, indent=2)
        
        os.chmod(filepath, 0o600)  # Restrict file permissions
        print(f"🔒 Keys saved encrypted to: {filepath}")
    
    def sign_message(self, message):
        """Sign a message with the private key"""
        signature = self.private_key.sign(message.encode())
        return base64.b64encode(signature).decode()
    
    def register_with_server(self, api_endpoint):
        """Register public key with DataFold server"""
        import requests
        
        response = requests.post(f"{api_endpoint}/api/crypto/client/register", json={
            'public_key': f'ed25519:{self.public_key_b64}',
            'client_type': 'desktop_application',
            'client_info': {
                'platform': os.name,
                'generated_at': str(datetime.now())
            }
        })
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ Successfully registered with server")
            print(f"📋 Client ID: {result.get('client_id')}")
            return result
        else:
            raise Exception(f"Registration failed: {response.text}")

# Usage example
if __name__ == "__main__":
    client = DataFoldClientKeys()
    client.generate_keys()
    client.save_keys_encrypted('./my_datafold_keys.json')
    client.register_with_server('http://localhost:9001')
```

**3. Command Line Tool:**
```bash
#!/bin/bash
# DataFold Client Key Generator

KEYS_DIR="$HOME/.datafold/keys"
API_ENDPOINT="http://localhost:9001"

echo "🔐 DataFold Client Key Generator"
echo "================================"

# Create keys directory
mkdir -p "$KEYS_DIR"
chmod 700 "$KEYS_DIR"

# Generate private key
PRIVATE_KEY_FILE="$KEYS_DIR/private_key.pem"
PUBLIC_KEY_FILE="$KEYS_DIR/public_key.pem"

echo "1. Generating Ed25519 key pair..."
openssl genpkey -algorithm Ed25519 -out "$PRIVATE_KEY_FILE"
openssl pkey -in "$PRIVATE_KEY_FILE" -pubout -out "$PUBLIC_KEY_FILE"

# Secure the private key
chmod 600 "$PRIVATE_KEY_FILE"
chmod 644 "$PUBLIC_KEY_FILE"

# Extract public key for registration
PUBLIC_KEY_B64=$(openssl pkey -in "$PUBLIC_KEY_FILE" -pubin -outform DER | base64 -w 0)

echo "✅ Keys generated successfully!"
echo "📁 Private key: $PRIVATE_KEY_FILE"
echo "📁 Public key: $PUBLIC_KEY_FILE"

# Register with server
echo ""
echo "2. Registering with DataFold server..."

REGISTRATION_RESPONSE=$(curl -s -X POST "$API_ENDPOINT/api/crypto/client/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"public_key\": \"ed25519:$PUBLIC_KEY_B64\",
    \"client_type\": \"command_line\",
    \"generated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }")

if echo "$REGISTRATION_RESPONSE" | jq -e '.client_id' > /dev/null 2>&1; then
    CLIENT_ID=$(echo "$REGISTRATION_RESPONSE" | jq -r '.client_id')
    echo "✅ Successfully registered with server!"
    echo "📋 Client ID: $CLIENT_ID"
    
    # Save client configuration
    cat > "$KEYS_DIR/config.json" << EOF
{
  "api_endpoint": "$API_ENDPOINT",
  "client_id": "$CLIENT_ID",
  "public_key": "ed25519:$PUBLIC_KEY_B64",
  "private_key_file": "$PRIVATE_KEY_FILE",
  "registered_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
    
    echo "📝 Configuration saved to: $KEYS_DIR/config.json"
else
    echo "❌ Registration failed: $REGISTRATION_RESPONSE"
    exit 1
fi

echo ""
echo "🎉 Setup complete! You can now use DataFold with your client keys."
echo "🔑 Public key: ed25519:$PUBLIC_KEY_B64"
```

#### Master Database Keys

**Note**: Only the database master keys for encryption at rest are generated and managed by the server. All client access keys are generated, stored, and managed solely by the clients; the server never generates or sees client private keys.
## Client-Side Key Rotation and Revocation

### Key Rotation Strategy

**Client-side key rotation** is fully supported and recommended for security best practices:

#### 1. Client-Initiated Key Rotation

**JavaScript/Browser Implementation:**
```javascript
class DataFoldKeyRotation {
    constructor(keyManager, apiEndpoint) {
        this.keyManager = keyManager;
        this.apiEndpoint = apiEndpoint;
        this.oldKeyPair = null;
        this.newKeyPair = null;
    }
    
    async rotateKeys() {
        console.log("🔄 Starting key rotation...");
        
        // Step 1: Generate new key pair
        this.oldKeyPair = this.keyManager.keyPair;
        const newKeys = await this.keyManager.generateKeys();
        this.newKeyPair = newKeys.keyPair;
        
        // Step 2: Register new key with server (signed by old key)
        const rotationRequest = await this.createRotationRequest();
        const response = await fetch(`${this.apiEndpoint}/api/crypto/key/rotate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(rotationRequest)
        });
        
        if (!response.ok) {
            throw new Error(`Key rotation failed: ${response.statusText}`);
        }
        
        const result = await response.json();
        console.log("✅ New key registered, transition period started");
        
        // Step 3: Wait for transition period (both keys valid)
        await this.waitForTransition(result.transition_period_seconds || 300);
        
        // Step 4: Revoke old key
        await this.revokeOldKey();
        
        console.log("🎉 Key rotation completed successfully");
        return result;
    }
    
    async createRotationRequest() {
        const timestamp = Math.floor(Date.now() / 1000);
        const nonce = crypto.randomUUID();
        const newPublicKeyB64 = await this.exportPublicKey(this.newKeyPair.publicKey);
        
        const payload = {
            action: "rotate_key",
            new_public_key: `ed25519:${newPublicKeyB64}`,
            timestamp: timestamp,
            nonce: nonce,
            transition_period: 300 // 5 minutes
        };
        
        // Sign with OLD private key to prove ownership
        const message = `${timestamp}:${nonce}:${JSON.stringify(payload)}`;
        const signature = await this.signWithKey(this.oldKeyPair.privateKey, message);
        const oldPublicKeyB64 = await this.exportPublicKey(this.oldKeyPair.publicKey);
        
        return {
            payload: payload,
            signature: signature,
            current_public_key: `ed25519:${oldPublicKeyB64}`
        };
    }
    
    async waitForTransition(seconds) {
        console.log(`⏳ Waiting ${seconds} seconds for transition period...`);
        return new Promise(resolve => setTimeout(resolve, seconds * 1000));
    }
    
    async revokeOldKey() {
        const timestamp = Math.floor(Date.now() / 1000);
        const nonce = crypto.randomUUID();
        const oldPublicKeyB64 = await this.exportPublicKey(this.oldKeyPair.publicKey);
        
        const payload = {
            action: "revoke_key",
            revoked_public_key: `ed25519:${oldPublicKeyB64}`,
            reason: "key_rotation_completed",
            timestamp: timestamp,
            nonce: nonce
        };
        
        // Sign with NEW private key to prove new key ownership
        const message = `${timestamp}:${nonce}:${JSON.stringify(payload)}`;
        const signature = await this.signWithKey(this.newKeyPair.privateKey, message);
        const newPublicKeyB64 = await this.exportPublicKey(this.newKeyPair.publicKey);
        
        const response = await fetch(`${this.apiEndpoint}/api/crypto/key/revoke`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                payload: payload,
                signature: signature,
                authorizing_public_key: `ed25519:${newPublicKeyB64}`
            })
        });
        
        if (response.ok) {
            console.log("🗑️ Old key successfully revoked");
        } else {
            console.warn("⚠️ Old key revocation failed, but new key is active");
        }
    }
}

// Usage
const rotator = new DataFoldKeyRotation(keyManager, 'http://localhost:9001');
await rotator.rotateKeys();
```

#### 2. Python Client Key Rotation

```python
import time
import json
import uuid
import base64
import requests
from datetime import datetime, timedelta

class DataFoldKeyRotation:
    def __init__(self, client_keys, api_endpoint):
        self.client_keys = client_keys
        self.api_endpoint = api_endpoint
        self.old_private_key = None
        self.new_private_key = None
    
    def rotate_keys(self, transition_period_minutes=5):
        """Perform complete key rotation with transition period"""
        print("🔄 Starting client-side key rotation...")
        
        # Step 1: Backup current key
        self.old_private_key = self.client_keys.private_key
        old_public_key_b64 = self.client_keys.public_key_b64
        
        # Step 2: Generate new key pair
        print("1. Generating new key pair...")
        new_public_key_b64 = self.client_keys.generate_keys()
        self.new_private_key = self.client_keys.private_key
        
        # Step 3: Register new key (signed by old key)
        print("2. Registering new key with server...")
        rotation_response = self._register_new_key(
            old_public_key_b64, 
            new_public_key_b64,
            transition_period_minutes
        )
        
        # Step 4: Transition period
        print(f"3. Transition period: {transition_period_minutes} minutes...")
        print("   Both keys are valid during this time")
        time.sleep(transition_period_minutes * 60)
        
        # Step 5: Revoke old key
        print("4. Revoking old key...")
        self._revoke_old_key(old_public_key_b64, new_public_key_b64)
        
        print("✅ Key rotation completed successfully!")
        return rotation_response
    
    def _register_new_key(self, old_public_key_b64, new_public_key_b64, transition_minutes):
        timestamp = int(time.time())
        nonce = str(uuid.uuid4())
        
        payload = {
            "action": "rotate_key",
            "new_public_key": f"ed25519:{new_public_key_b64}",
            "timestamp": timestamp,
            "nonce": nonce,
            "transition_period": transition_minutes * 60
        }
        
        # Sign with old private key to prove current ownership
        message = f"{timestamp}:{nonce}:{json.dumps(payload, sort_keys=True)}"
        signature = self.old_private_key.sign(message.encode())
        
        request_data = {
            "payload": payload,
            "signature": base64.b64encode(signature).decode(),
            "current_public_key": f"ed25519:{old_public_key_b64}"
        }
        
        response = requests.post(
            f"{self.api_endpoint}/api/crypto/key/rotate",
            json=request_data
        )
        
        if response.status_code != 200:
            raise Exception(f"Key rotation failed: {response.text}")
        
        return response.json()
    
    def _revoke_old_key(self, old_public_key_b64, new_public_key_b64):
        timestamp = int(time.time())
        nonce = str(uuid.uuid4())
        
        payload = {
            "action": "revoke_key",
            "revoked_public_key": f"ed25519:{old_public_key_b64}",
            "reason": "key_rotation_completed",
            "timestamp": timestamp,
            "nonce": nonce
        }
        
        # Sign with new private key to prove new ownership
        message = f"{timestamp}:{nonce}:{json.dumps(payload, sort_keys=True)}"
        signature = self.new_private_key.sign(message.encode())
        
        request_data = {
            "payload": payload,
            "signature": base64.b64encode(signature).decode(),
            "authorizing_public_key": f"ed25519:{new_public_key_b64}"
        }
        
        response = requests.post(
            f"{self.api_endpoint}/api/crypto/key/revoke",
            json=request_data
        )
        
        if response.status_code == 200:
            print("🗑️ Old key successfully revoked")
        else:
            print(f"⚠️ Old key revocation warning: {response.text}")

# Usage
rotator = DataFoldKeyRotation(client_keys, 'http://localhost:9001')
rotator.rotate_keys(transition_period_minutes=5)
```

### Key Revocation System

#### 1. Immediate Revocation

**Client-Initiated Revocation:**
```bash
# Emergency key revocation via API
curl -X POST http://localhost:9001/api/crypto/key/revoke \
  -H "Content-Type: application/json" \
  -d '{
    "payload": {
      "action": "emergency_revoke",
      "revoked_public_key": "ed25519:COMPROMISED_KEY...",
      "reason": "key_compromise_suspected",
      "timestamp": 1704067200,
      "nonce": "emergency-123"
    },
    "signature": "SIGNED_BY_SAME_KEY...",
    "authorizing_public_key": "ed25519:COMPROMISED_KEY..."
  }'
```

#### 2. Server-Side Revocation Management

```rust
// Server-side revocation handling
pub struct KeyRevocationManager {
    revoked_keys: HashMap<String, RevocationRecord>,
    active_transitions: HashMap<String, TransitionRecord>,
}

#[derive(Serialize, Deserialize)]
pub struct RevocationRecord {
    pub public_key: String,
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
    pub revoked_by: Option<String>, // If revoked by different key
    pub signature: String,
}

impl KeyRevocationManager {
    pub fn revoke_key(&mut self, request: RevocationRequest) -> FoldDbResult<()> {
        // Verify signature
        self.verify_revocation_signature(&request)?;
        
        // Add to revocation list
        let record = RevocationRecord {
            public_key: request.public_key.clone(),
            revoked_at: Utc::now(),
            reason: request.reason,
            revoked_by: request.authorizing_key,
            signature: request.signature,
        };
        
        self.revoked_keys.insert(request.public_key.clone(), record);
        
        // Broadcast revocation to network peers
        self.broadcast_revocation(&request.public_key)?;
        
        log::info!("Key revoked: {}", request.public_key);
        Ok(())
    }
    
    pub fn is_key_revoked(&self, public_key: &str) -> bool {
        self.revoked_keys.contains_key(public_key)
    }
    
    pub fn start_key_transition(&mut self, old_key: String, new_key: String, duration: Duration) {
        let transition = TransitionRecord {
            old_key: old_key.clone(),
            new_key,
            started_at: Utc::now(),
            expires_at: Utc::now() + duration,
        };
        
        self.active_transitions.insert(old_key, transition);
    }
}
```

#### 3. Revocation Distribution

**P2P Network Propagation:**
```rust
// Propagate revocations across the network
impl NetworkRevocationHandler {
    pub async fn broadcast_revocation(&self, revocation: &RevocationRecord) {
        let message = NetworkMessage::KeyRevocation {
            public_key: revocation.public_key.clone(),
            revoked_at: revocation.revoked_at,
            reason: revocation.reason.clone(),
            signature: revocation.signature.clone(),
        };
        
        // Send to all connected peers
        for peer in &self.connected_peers {
            if let Err(e) = peer.send_message(&message).await {
                log::warn!("Failed to send revocation to peer {}: {}", peer.id(), e);
            }
        }
    }
    
    pub async fn handle_revocation_message(&mut self, message: NetworkMessage) {
        if let NetworkMessage::KeyRevocation { public_key, .. } = message {
            // Verify and apply revocation
            if self.verify_revocation_signature(&message).await {
                self.revocation_manager.add_revoked_key(public_key);
                log::info!("Applied network revocation for key");
### Simplified Key Replacement Process

**Single Client-Initiated Process**: There is one unified process for key replacement that handles both rotation and revocation scenarios.

#### Process Overview

The key replacement process is simple and atomic:
1. **Client generates new key pair** (locally, private key never leaves client)
2. **Client creates signed replacement request** (signed with old private key)
3. **Database atomically replaces old key with new key** (erase old instance, write new instance)
4. **Old key is immediately invalidated** across the network

#### API Endpoint

**Single Key Replacement Endpoint:**
```http
POST /api/crypto/key/replace
```

#### Implementation

**JavaScript/Browser Implementation:**
```javascript
class DataFoldKeyReplacement {
    constructor(keyManager, apiEndpoint) {
        this.keyManager = keyManager;
        this.apiEndpoint = apiEndpoint;
    }
    
    async replaceKey(reason = "scheduled_rotation") {
        console.log("🔄 Starting key replacement...");
        
        // Step 1: Generate new key pair (client-side)
        const oldKeyPair = this.keyManager.keyPair;
        const oldPublicKeyB64 = await this.exportPublicKey(oldKeyPair.publicKey);
        
        const newKeys = await this.keyManager.generateKeys();
        const newKeyPair = newKeys.keyPair;
        const newPublicKeyB64 = await this.exportPublicKey(newKeyPair.publicKey);
        
        // Step 2: Create replacement request
        const timestamp = Math.floor(Date.now() / 1000);
        const nonce = crypto.randomUUID();
        
        const payload = {
            action: "replace_key",
            old_public_key: `ed25519:${oldPublicKeyB64}`,
            new_public_key: `ed25519:${newPublicKeyB64}`,
            reason: reason,
            timestamp: timestamp,
            nonce: nonce
        };
        
        // Step 3: Sign with OLD private key to prove ownership
        const message = `${timestamp}:${nonce}:${JSON.stringify(payload)}`;
        const signature = await this.signWithKey(oldKeyPair.privateKey, message);
        
        const request = {
            payload: payload,
            signature: signature,
            authorizing_public_key: `ed25519:${oldPublicKeyB64}`
        };
        
        // Step 4: Submit atomic replacement request
        const response = await fetch(`${this.apiEndpoint}/api/crypto/key/replace`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(request)
        });
        
        if (!response.ok) {
            throw new Error(`Key replacement failed: ${response.statusText}`);
        }
        
        const result = await response.json();
        console.log("✅ Key replacement completed successfully");
        
        // Update local key manager to use new key
        this.keyManager.keyPair = newKeyPair;
        this.keyManager.publicKeyB64 = newPublicKeyB64;
        
        return {
            old_key: `ed25519:${oldPublicKeyB64}`,
            new_key: `ed25519:${newPublicKeyB64}`,
            replaced_at: result.replaced_at,
            transaction_id: result.transaction_id
        };
    }
    
    async exportPublicKey(publicKey) {
        const publicKeyBuffer = await crypto.subtle.exportKey("spki", publicKey);
        return btoa(String.fromCharCode(...new Uint8Array(publicKeyBuffer)));
    }
    
    async signWithKey(privateKey, message) {
        const encoder = new TextEncoder();
        const data = encoder.encode(message);
        const signature = await crypto.subtle.sign("Ed25519", privateKey, data);
        return btoa(String.fromCharCode(...new Uint8Array(signature)));
    }
}

// Usage
const keyReplacer = new DataFoldKeyReplacement(keyManager, 'http://localhost:9001');

// Scheduled rotation
await keyReplacer.replaceKey("scheduled_rotation");

// Emergency replacement
await keyReplacer.replaceKey("security_compromise");
```

**Python Implementation:**
```python
import json
import time
import uuid
import base64
import requests
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519

class DataFoldKeyReplacement:
    def __init__(self, api_endpoint):
        self.api_endpoint = api_endpoint
    
    def replace_key(self, old_private_key, reason="scheduled_rotation"):
        """Replace old key with new key in single atomic operation"""
        
        print("🔄 Starting key replacement...")
        
        # Step 1: Generate new key pair
        new_private_key = ed25519.Ed25519PrivateKey.generate()
        new_public_key = new_private_key.public_key()
        
        # Export keys for request
        old_public_key = old_private_key.public_key()
        old_public_key_b64 = base64.b64encode(
            old_public_key.public_bytes(
                encoding=serialization.Encoding.DER,
                format=serialization.PublicFormat.SubjectPublicKeyInfo
            )
        ).decode()
        
        new_public_key_b64 = base64.b64encode(
            new_public_key.public_bytes(
                encoding=serialization.Encoding.DER,
                format=serialization.PublicFormat.SubjectPublicKeyInfo
            )
        ).decode()
        
        # Step 2: Create replacement payload
        timestamp = int(time.time())
        nonce = str(uuid.uuid4())
        
        payload = {
            "action": "replace_key",
            "old_public_key": f"ed25519:{old_public_key_b64}",
            "new_public_key": f"ed25519:{new_public_key_b64}",
            "reason": reason,
            "timestamp": timestamp,
            "nonce": nonce
        }
        
        # Step 3: Sign with old private key
        message = f"{timestamp}:{nonce}:{json.dumps(payload, sort_keys=True)}"
        signature = old_private_key.sign(message.encode())
        
        request_data = {
            "payload": payload,
            "signature": base64.b64encode(signature).decode(),
            "authorizing_public_key": f"ed25519:{old_public_key_b64}"
        }
        
        # Step 4: Submit atomic replacement
        response = requests.post(
            f"{self.api_endpoint}/api/crypto/key/replace",
            json=request_data
        )
        
        if response.status_code == 200:
            result = response.json()
            print("✅ Key replacement completed successfully")
            print(f"📋 Transaction ID: {result.get('transaction_id')}")
            
            return {
                'old_key': f"ed25519:{old_public_key_b64}",
                'new_key': f"ed25519:{new_public_key_b64}",
                'new_private_key': new_private_key,
                'replaced_at': result.get('replaced_at'),
                'transaction_id': result.get('transaction_id')
            }
        else:
            raise Exception(f"Key replacement failed: {response.text}")

# Usage
replacer = DataFoldKeyReplacement('http://localhost:9001')

# Load current private key
with open('current_private_key.pem', 'rb') as f:
    current_private_key = serialization.load_pem_private_key(f.read(), password=None)

# Replace key
result = replacer.replace_key(current_private_key, "scheduled_rotation")

# Save new private key
new_private_pem = result['new_private_key'].private_bytes(
    encoding=serialization.Encoding.PEM,
    format=serialization.PrivateFormat.PKCS8,
    encryption_algorithm=serialization.NoEncryption()
)

with open('new_private_key.pem', 'wb') as f:
    f.write(new_private_pem)

print(f"🔑 New key saved: {result['new_key']}")
```

**Command Line Tool:**
```bash
#!/bin/bash
# DataFold Key Replacement Tool

OLD_PRIVATE_KEY="./current_private_key.pem"
NEW_PRIVATE_KEY="./new_private_key.pem"
NEW_PUBLIC_KEY="./new_public_key.pem"
API_ENDPOINT="http://localhost:9001"

echo "🔄 DataFold Key Replacement"
echo "=========================="

# Step 1: Generate new key pair
echo "1. Generating new key pair..."
openssl genpkey -algorithm Ed25519 -out "$NEW_PRIVATE_KEY"
openssl pkey -in "$NEW_PRIVATE_KEY" -pubout -out "$NEW_PUBLIC_KEY"
chmod 600 "$NEW_PRIVATE_KEY"

# Step 2: Extract public keys
OLD_PUBLIC_KEY_B64=$(openssl pkey -in "$OLD_PRIVATE_KEY" -pubout -outform DER | base64 -w 0)
NEW_PUBLIC_KEY_B64=$(openssl pkey -in "$NEW_PUBLIC_KEY" -pubin -outform DER | base64 -w 0)

# Step 3: Create replacement request
TIMESTAMP=$(date +%s)
NONCE=$(uuidgen)

PAYLOAD=$(cat << EOF
{
  "action": "replace_key",
  "old_public_key": "ed25519:$OLD_PUBLIC_KEY_B64",
  "new_public_key": "ed25519:$NEW_PUBLIC_KEY_B64",
  "reason": "manual_replacement",
  "timestamp": $TIMESTAMP,
  "nonce": "$NONCE"
}
EOF
)

# Step 4: Sign with old private key
MESSAGE="$TIMESTAMP:$NONCE:$PAYLOAD"
SIGNATURE=$(echo -n "$MESSAGE" | openssl dgst -sha256 -sign "$OLD_PRIVATE_KEY" | base64 -w 0)

# Step 5: Submit replacement request
echo "2. Submitting key replacement..."
RESPONSE=$(curl -s -X POST "$API_ENDPOINT/api/crypto/key/replace" \
  -H "Content-Type: application/json" \
  -d "{
    \"payload\": $PAYLOAD,
    \"signature\": \"$SIGNATURE\",
    \"authorizing_public_key\": \"ed25519:$OLD_PUBLIC_KEY_B64\"
  }")

if echo "$RESPONSE" | jq -e '.status == "replaced"' > /dev/null; then
    TRANSACTION_ID=$(echo "$RESPONSE" | jq -r '.transaction_id')
    
    echo "✅ Key replacement successful!"
    echo "📋 Transaction ID: $TRANSACTION_ID"
    echo "🔑 New public key: ed25519:$NEW_PUBLIC_KEY_B64"
    
    # Step 6: Secure cleanup of old key
    echo "3. Securing old private key..."
    shred -vfz -n 3 "$OLD_PRIVATE_KEY"
    
    # Replace old key with new key
    mv "$NEW_PRIVATE_KEY" "$OLD_PRIVATE_KEY"
    
    echo "🎉 Key replacement completed!"
else
    echo "❌ Key replacement failed: $RESPONSE"
    rm -f "$NEW_PRIVATE_KEY" "$NEW_PUBLIC_KEY"
    exit 1
fi
```

#### Server-Side Database Operation

**Atomic Key Replacement:**
```rust
// Server-side atomic key replacement
impl KeyReplacementHandler {
    pub async fn replace_key(&mut self, request: KeyReplacementRequest) -> FoldDbResult<KeyReplacementResponse> {
        // Step 1: Verify signature with old key
        self.verify_replacement_signature(&request)?;
        
        // Step 2: Begin atomic transaction
        let mut transaction = self.database.begin_transaction().await?;
        
        // Step 3: Find all instances with old key
        let old_instances = transaction
            .find_instances_by_key(&request.old_public_key)
            .await?;
        
        if old_instances.is_empty() {
            return Err(FoldDbError::KeyNotFound(request.old_public_key));
        }
        
        // Step 4: Create new instances with new key
        let mut new_instances = Vec::new();
        for old_instance in old_instances {
            let new_instance = DataInstance {
                id: old_instance.id,
                data: old_instance.data,
                public_key: request.new_public_key.clone(),
                created_at: old_instance.created_at,
                updated_at: Utc::now(),
                version: old_instance.version + 1,
            };
            new_instances.push(new_instance);
        }
        
        // Step 5: Atomic replace operation
        transaction.delete_instances_by_key(&request.old_public_key).await?;
        transaction.insert_instances(&new_instances).await?;
        
        // Step 6: Update key registry
        transaction.remove_key_registration(&request.old_public_key).await?;
        transaction.add_key_registration(&request.new_public_key, &request.reason).await?;
        
        // Step 7: Commit transaction
        transaction.commit().await?;
        
        // Step 8: Broadcast replacement to network
        self.broadcast_key_replacement(&request.old_public_key, &request.new_public_key).await?;
        
        log::info!("Key replaced: {} -> {}",
                   request.old_public_key,
                   request.new_public_key);
        
        Ok(KeyReplacementResponse {
            status: "replaced".to_string(),
            transaction_id: Uuid::new_v4().to_string(),
            replaced_at: Utc::now(),
            old_key: request.old_public_key,
            new_key: request.new_public_key,
        })
    }
}
```

This simplified process ensures that key replacement is **atomic**, **client-controlled**, and **immediately effective** across the entire DataFold network.

```

### Key Lifecycle Management

#### Simple Lifecycle

```
1. Key Generation (Client-Side)
   ↓
2. Key Registration (Server)
   ↓
3. Active Use Period
   ↓
4. Key Replacement Trigger (User-Initiated)
   ↓
5. New Key Generation (Client-Side)
   ↓
6. Atomic Key Replacement (Database erases old, writes new)
   ↓
7. New Key Active Period
   ↓
(Repeat from step 3)
```

#### Benefits of Client-Side Key Replacement

✅ **User Control**: Users control their own key lifecycle
✅ **Zero Server Trust**: Server never sees private keys
✅ **Atomic Operation**: Database ensures consistency with atomic erase/write
✅ **Immediate Effect**: Old key invalidated instantly
✅ **Audit Trail**: All replacements are cryptographically signed
✅ **Network Resilience**: Distributed updates across P2P network

This approach ensures maximum security while providing users complete control over their cryptographic identity in the DataFold network.

#### New Security Endpoints

```http
# Crypto Management
POST /api/crypto/init             # Initialize database with master keys
GET  /api/crypto/status          # Check encryption status
POST /api/crypto/rotate-keys     # Rotate master keys
POST /api/crypto/client/register # Register client public key

# Secure Schema Management
POST /api/schema/secure          # Create schema with encryption
POST /api/permissions/set        # Set field-level permissions
GET  /api/permissions/audit      # View permission audit log
```

#### Enhanced Setup Flow

**Step 1: Initialize Secure Database**
```bash
# Initialize database with master key pair and encryption
curl -X POST http://localhost:9001/api/crypto/init \
  -H "Content-Type: application/json" \
  -d '{
    "passphrase": "your-secure-passphrase",
    "key_algorithm": "Ed25519",
    "encryption_algorithm": "AES-256-GCM",
    "storage_path": "./secure_data"
  }'
```

**Response:**
```json
{
  "status": "success",
  "database_id": "db_a1b2c3d4",
  "master_public_key": "ed25519:ABC123DEF456...",
  "encryption_enabled": true,
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Step 2: Register Client Access Keys**
```bash
# Register client public key for database access
curl -X POST http://localhost:9001/api/crypto/client/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_name": "admin_client",
    "public_key": "ed25519:CLIENT_PUB_KEY_HERE...",
    "permissions": ["schema_create", "schema_read", "data_write"],
    "expires_at": "2024-12-31T23:59:59Z"
  }'
```

**Step 3: Create Secure Schema with Permissions**
```bash
# Create schema with field-level security
curl -X POST http://localhost:9001/api/schema/secure \
  -H "Content-Type: application/json" \
  -d '{
    "name": "SecureUserProfile",
    "encrypted": true,
    "fields": {
      "username": {
        "type": "string",
        "permissions": {
          "read_policy": {"NoRequirement": null},
          "write_policy": {"Distance": 1}
        }
      },
      "email": {
        "type": "string",
        "encrypted": true,
        "permissions": {
          "read_policy": {"Distance": 1},
          "write_policy": {"Distance": 0}
        }
      },
      "ssn": {
        "type": "string",
        "encrypted": true,
        "permissions": {
          "read_policy": {"Explicit": "admin_access"},
          "write_policy": {"Distance": 0}
        }
      }
    },
    "payment_config": {
      "base_multiplier": 100.0,
      "min_payment": 50
    }
  }'
```

**Step 4: Set Explicit Permissions**
```bash
# Grant explicit permissions for sensitive fields
curl -X POST http://localhost:9001/api/permissions/set \
  -H "Content-Type: application/json" \
  -d '{
    "permission_name": "admin_access",
    "schema": "SecureUserProfile",
    "field": "ssn",
    "granted_keys": [
      "ed25519:ADMIN_PUB_KEY_1...",
      "ed25519:ADMIN_PUB_KEY_2..."
    ],
    "expires_at": "2024-12-31T23:59:59Z"
  }'
```

**Step 5: Initialize Secure Network**
```bash
# Start network with enhanced security
curl -X POST http://localhost:9001/api/network/init \
  -H "Content-Type: application/json" \
  -d '{
    "port": 9000,
    "enable_mdns": true,
    "require_signatures": true,
    "tls_enabled": true,
    "trusted_peers": [
      "12D3KooWTrustedPeer1...",
      "12D3KooWTrustedPeer2..."
    ]
  }'
```

### Complete Setup Script

**Automated Secure Setup via HTTP API:**

```bash
#!/bin/bash

# DataFold Secure HTTP Setup Script
set -e

BASE_URL="http://localhost:9001"
CLIENT_PRIVATE_KEY="./client_private.pem"
CLIENT_PUBLIC_KEY="./client_public.pem"

echo "🔐 Setting up secure DataFold database..."

# Step 1: Initialize crypto system
echo "1. Initializing encryption..."
INIT_RESPONSE=$(curl -s -X POST $BASE_URL/api/crypto/init \
  -H "Content-Type: application/json" \
  -d '{
    "passphrase": "'"$DB_PASSPHRASE"'",
    "key_algorithm": "Ed25519",
    "encryption_algorithm": "AES-256-GCM"
  }')

DATABASE_ID=$(echo $INIT_RESPONSE | jq -r '.database_id')
MASTER_PUBLIC_KEY=$(echo $INIT_RESPONSE | jq -r '.master_public_key')

echo "✅ Database initialized with ID: $DATABASE_ID"
echo "   Master public key: $MASTER_PUBLIC_KEY"

# Step 2: Generate client keys if they don't exist
if [ ! -f "$CLIENT_PRIVATE_KEY" ]; then
    echo "2. Generating client keys..."
    openssl genpkey -algorithm Ed25519 -out $CLIENT_PRIVATE_KEY
    openssl pkey -in $CLIENT_PRIVATE_KEY -pubout -out $CLIENT_PUBLIC_KEY
    chmod 600 $CLIENT_PRIVATE_KEY
    echo "✅ Client keys generated"
else
    echo "2. Using existing client keys"
fi

# Extract client public key
CLIENT_PUB_KEY_B64=$(openssl pkey -in $CLIENT_PUBLIC_KEY -pubin -outform DER | base64 -w 0)

# Step 3: Register client
echo "3. Registering client access..."
curl -s -X POST $BASE_URL/api/crypto/client/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_name": "setup_admin",
    "public_key": "ed25519:'"$CLIENT_PUB_KEY_B64"'",
    "permissions": ["schema_create", "schema_read", "data_write", "admin_access"]
  }' > /dev/null

echo "✅ Client registered successfully"

# Step 4: Create secure schema
echo "4. Creating secure schema..."
curl -s -X POST $BASE_URL/api/schema/secure \
  -H "Content-Type: application/json" \
  -d '{
    "name": "SecureUserProfile",
    "encrypted": true,
    "fields": {
      "id": {
        "type": "string",
        "permissions": {
          "read_policy": {"NoRequirement": null},
          "write_policy": {"Distance": 0}
        }
      },
      "username": {
        "type": "string",
        "permissions": {
          "read_policy": {"Distance": 1},
          "write_policy": {"Distance": 1}
        }
      },
      "email": {
        "type": "string",
        "encrypted": true,
        "permissions": {
          "read_policy": {"Distance": 1},
          "write_policy": {"Distance": 0}
        }
      },
      "personal_data": {
        "type": "object",
        "encrypted": true,
        "permissions": {
          "read_policy": {"Explicit": "admin_access"},
          "write_policy": {"Distance": 0}
        }
      }
    }
  }' > /dev/null

echo "✅ Secure schema created"

# Step 5: Start secure network
echo "5. Initializing secure network..."
curl -s -X POST $BASE_URL/api/network/init \
  -H "Content-Type: application/json" \
  -d '{
    "port": 9000,
    "enable_mdns": true,
    "require_signatures": true,
    "tls_enabled": true
  }' > /dev/null

curl -s -X POST $BASE_URL/api/network/start > /dev/null

echo "✅ Secure network started"

# Step 6: Verify setup
echo "6. Verifying setup..."
STATUS=$(curl -s $BASE_URL/api/system/status)
CRYPTO_STATUS=$(curl -s $BASE_URL/api/crypto/status)
NETWORK_STATUS=$(curl -s $BASE_URL/api/network/status)

echo "✅ Setup complete!"
echo ""
echo "📊 System Status:"
echo "   Database ID: $DATABASE_ID"
echo "   Encryption: $(echo $CRYPTO_STATUS | jq -r '.encryption_enabled')"
echo "   Network: $(echo $NETWORK_STATUS | jq -r '.status')"
echo ""
echo "🔑 Client Configuration:"
echo "   Private Key: $CLIENT_PRIVATE_KEY"
echo "   Public Key: $CLIENT_PUBLIC_KEY"
echo "   Database Access: Enabled"
echo ""
echo "🌐 Access URLs:"
echo "   HTTP API: $BASE_URL/api"
echo "   Network Port: 9000"
echo "   Web UI: $BASE_URL"

# Save configuration
cat > datafold_client_config.json << EOF
{
  "database_id": "$DATABASE_ID",
  "master_public_key": "$MASTER_PUBLIC_KEY",
  "client_private_key": "$CLIENT_PRIVATE_KEY",
  "client_public_key_b64": "$CLIENT_PUB_KEY_B64",
  "api_endpoint": "$BASE_URL",
  "encryption_enabled": true,
  "signature_required": true
}
EOF

echo "📝 Configuration saved to: datafold_client_config.json"
```

### Usage Example

```bash
# Set database passphrase
export DB_PASSPHRASE="your-secure-database-passphrase"

# Run setup script
chmod +x setup_secure_datafold.sh
./setup_secure_datafold.sh

# Test secure access with client
python3 test_secure_client.py
```

### Client Testing Script

```python
#!/usr/bin/env python3
"""Test secure DataFold client access."""

import json
import time
import uuid
import base64
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519
import requests

def load_config():
    with open('datafold_client_config.json', 'r') as f:
        return json.load(f)

def test_secure_access():
    config = load_config()
    
    # Load client private key
    with open(config['client_private_key'], 'rb') as f:
        private_key = serialization.load_pem_private_key(f.read(), password=None)
    
    # Create signed request
    timestamp = int(time.time())
    nonce = str(uuid.uuid4())
    payload = {
        "schema": "SecureUserProfile",
        "operation": "query",
        "fields": ["id", "username"]
    }
    
    message = f"{timestamp}:{nonce}:{json.dumps(payload, sort_keys=True)}"
    signature = private_key.sign(message.encode())
    
    signed_request = {
        "timestamp": timestamp,
        "nonce": nonce,
        "payload": payload,
        "signature": base64.b64encode(signature).decode(),
        "public_key": config['client_public_key_b64']
    }
    
    # Send secure request
    response = requests.post(
        f"{config['api_endpoint']}/api/query",
        json=signed_request
    )
    
    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")

if __name__ == "__main__":
    test_secure_access()
```

This comprehensive setup process ensures that DataFold databases are initialized with proper encryption, authentication, and access controls from the very beginning, making security a foundational aspect rather than an afterthought.