# DataFold Public Key Registration Integration Guides

A comprehensive guide for integrating public key registration and signature verification across JavaScript SDK, Python SDK, and CLI platforms.

---

## Table of Contents

1. [Introduction & Prerequisites](#introduction--prerequisites)
2. [Key Generation](#key-generation)
3. [Secure Key Storage](#secure-key-storage)
4. [Key Backup & Recovery](#key-backup--recovery)
5. [Public Key Registration](#public-key-registration)
6. [Signature Generation & Verification](#signature-generation--verification)
7. [Practical Integration Examples](#practical-integration-examples)
8. [Server Configuration & Deployment](#server-configuration--deployment)
9. [Authentication & Authorization Patterns](#authentication--authorization-patterns)
10. [Troubleshooting Common Issues](#troubleshooting-common-issues)
11. [Security Best Practices](#security-best-practices)
12. [Performance Optimization](#performance-optimization)
13. [Deployment Checklists & Operational Procedures](#deployment-checklists--operational-procedures)
14. [Monitoring & Logging Recommendations](#monitoring--logging-recommendations)

---

## Introduction & Prerequisites

### Overview

This guide enables developers and operations teams to integrate DataFold's public key registration system with their applications. The system provides client-side Ed25519 key management with secure server registration and signature verification capabilities.

### Architecture Flow

```mermaid
sequenceDiagram
    participant Dev as Developer/CI
    participant SDK as SDK/CLI
    participant Server as DataFold Server
    participant Storage as Secure Storage

    Dev->>SDK: Generate Ed25519 key pair
    SDK->>Storage: Store private key (encrypted)
    SDK->>Server: Register public key (POST /api/crypto/register)
    Server-->>SDK: Registration confirmation (client ID)
    SDK->>SDK: Sign message with private key
    SDK->>Server: Verify signature (POST /api/crypto/verify)
    Server-->>SDK: Verification result
```

### Prerequisites

#### JavaScript SDK
- **Browser Requirements**: Modern browsers with WebCrypto API support
- **Secure Context**: HTTPS or localhost required
- **Dependencies**: `@datafold/js-sdk` package
- **Storage**: IndexedDB support required

#### Python SDK
- **Python Version**: 3.8+ required
- **Dependencies**: `datafold-sdk`, `cryptography>=41.0.0`
- **OS Support**: Windows (DPAPI), macOS (Keychain), Linux (Secret Service)
- **Fallback**: Encrypted file storage with proper permissions

#### CLI
- **System Requirements**: OpenSSL or native crypto support
- **Permissions**: File system write access for key storage
- **Network**: TLS-enabled HTTP client
- **Configuration**: Configurable security parameters

---

## Key Generation

All platforms generate Ed25519 key pairs using cryptographically secure random number generators, ensuring private keys never leave the client environment.

### JavaScript SDK

```javascript
import { 
  generateKeyPair, 
  initializeSDK, 
  formatKey 
} from '@datafold/js-sdk';

// Initialize and check compatibility
const { compatible, warnings } = await initializeSDK();
if (!compatible) {
  throw new Error(`Browser not compatible: ${warnings.join(', ')}`);
}

// Generate Ed25519 key pair
const keyPair = await generateKeyPair({
  validate: true  // Verify key pair integrity
});

console.log('Private key (hex):', formatKey(keyPair.privateKey, 'hex'));
console.log('Public key (base64):', formatKey(keyPair.publicKey, 'base64'));
```

### Python SDK

```python
import datafold_sdk
from datafold_sdk.crypto import SecureKeyStorage

# Initialize SDK and check compatibility
result = datafold_sdk.initialize_sdk()
if not result['compatible']:
    raise RuntimeError(f"Platform not compatible: {result['warnings']}")

# Generate Ed25519 key pair
key_pair = datafold_sdk.generate_key_pair(validate=True)

# Format keys for different uses
private_hex = datafold_sdk.format_key(key_pair.private_key, 'hex')
public_base64 = datafold_sdk.format_key(key_pair.public_key, 'base64')
public_pem = datafold_sdk.format_key(key_pair.public_key, 'pem')

print(f"Generated key pair - Private: {len(key_pair.private_key)} bytes")
print(f"Public key (base64): {public_base64}")
```

### CLI

```bash
# Initialize cryptographic system
datafold_cli crypto init --method random --security-level balanced

# Generate new Ed25519 key pair
datafold_cli crypto generate-key \
  --key-id "my-app-key" \
  --format hex \
  --output keypair.txt

# Export public key for registration
datafold_cli crypto export-public-key \
  --key-id "my-app-key" \
  --format base64
```

---

## Secure Key Storage

Each platform provides secure storage appropriate to its environment with encryption at rest.

### JavaScript SDK - IndexedDB Storage

```javascript
import { createStorage } from '@datafold/js-sdk';

// Create secure storage with custom configuration
const storage = await createStorage({
  dbName: 'MyAppCrypto',
  debug: false
});

// Store key pair with metadata
await storage.storeKeyPair('production-key', keyPair, 'SecurePass123!', {
  name: 'Production Signing Key',
  description: 'Primary key for production API signing',
  tags: ['production', 'api', 'signing'],
  created: new Date().toISOString()
});

// Retrieve key pair
const retrievedKeyPair = await storage.retrieveKeyPair(
  'production-key', 
  'SecurePass123!'
);
```

**Storage Security:**
- **Encryption**: AES-GCM with PBKDF2-derived keys (100,000+ iterations)
- **Isolation**: Browser same-origin policy enforcement
- **Limitations**: Subject to browser data clearing

### Python SDK - OS Integration

```python
from datafold_sdk.crypto import SecureKeyStorage

# Create storage with OS keychain integration
storage = SecureKeyStorage(
    use_keyring=True,  # Prefer OS keychain
    storage_dir=None   # Use default secure directory
)

# Store key with metadata
metadata = storage.store_key(
    key_id="production-signing-key",
    key_pair=key_pair,
    passphrase="SecurePassphrase123!"
)

print(f"Stored key with ID: {metadata.key_id}")
print(f"Storage method: {metadata.storage_method}")  # 'keyring' or 'file'

# Retrieve key
retrieved_key_pair = storage.retrieve_key(
    key_id="production-signing-key",
    passphrase="SecurePassphrase123!"
)
```

**Storage Security:**
- **Primary**: OS keychain (Keychain Access, DPAPI, Secret Service)
- **Fallback**: Encrypted files with restrictive permissions (0600)
- **Encryption**: Scrypt KDF with Fernet authenticated encryption

### CLI - File-Based Storage

```bash
# Store key securely (prompts for passphrase)
datafold_cli crypto store-key \
  --key-id "production-key" \
  --private-key "$(cat keypair.txt)" \
  --format hex

# Retrieve key (prompts for passphrase)
datafold_cli crypto retrieve-key \
  --key-id "production-key" \
  --format base64
```

**Storage Security:**
- **Encryption**: Argon2id KDF with AES-GCM
- **Permissions**: Restrictive file permissions (0600)
- **Configuration**: Configurable via [`CryptoConfig`](../../src/datafold_node/config.rs)

---

## Key Backup & Recovery

All platforms support encrypted backup and restore with cross-platform compatibility using a standardized JSON format.

### Standardized Backup Format

```json
{
  "version": 1,
  "key_id": "production-signing-key",
  "algorithm": "Ed25519",
  "kdf": "argon2id",
  "kdf_params": {
    "memory": 65536,
    "iterations": 3,
    "parallelism": 1
  },
  "encryption": "xchacha20-poly1305",
  "salt": "base64-encoded-salt",
  "nonce": "base64-encoded-nonce",
  "ciphertext": "base64-encoded-encrypted-key-data",
  "metadata": {
    "name": "Production Signing Key",
    "created": "2025-06-08T19:59:22Z"
  }
}
```

### JavaScript Backup/Recovery

```javascript
import { exportKey, importKey } from '@datafold/js-sdk';

// Export encrypted backup
const exportResult = await exportKey(storage, 'production-key', 'BackupPass123!', {
  format: 'json',
  includeMetadata: true,
  kdfIterations: 100000
});

// Save backup (implementation depends on environment)
await saveBackupFile('production-key-backup.json', exportResult.data);

// Import from backup
const backupData = await loadBackupFile('production-key-backup.json');
const importResult = await importKey(storage, backupData, 'BackupPass123!', {
  overwriteExisting: false,
  validateIntegrity: true
});

console.log('Import successful:', importResult.success);
```

### Python Backup/Recovery

```python
from datafold_sdk.crypto import export_key_to_file, import_key_from_file

# Export to encrypted backup file
export_metadata = export_key_to_file(
    key_pair=key_pair,
    passphrase="BackupPassphrase123!",
    key_id="production-signing-key",
    file_path="production-key-backup.json",
    export_format='json'
)

print(f"Backup created: {export_metadata.file_path}")

# Import from backup file
restored_key_pair, metadata = import_key_from_file(
    file_path="production-key-backup.json",
    passphrase="BackupPassphrase123!",
    verify_integrity=True
)

print(f"Restored key: {metadata.key_id}")
```

### CLI Backup/Recovery

```bash
# Export encrypted backup
datafold_cli crypto export-key \
  --key-id "production-key" \
  --output "production-key-backup.json" \
  --format json

# Import from backup
datafold_cli crypto import-key \
  --input "production-key-backup.json" \
  --key-id "restored-production-key"

# Verify backup integrity
datafold_cli crypto verify-backup \
  --input "production-key-backup.json"
```

---

## Public Key Registration

Register public keys with the DataFold server to enable signature verification.

### Registration Workflow

1. **Generate**: Create Ed25519 key pair locally
2. **Store**: Save private key securely (encrypted)
3. **Register**: Send public key to server
4. **Confirm**: Receive client ID and registration status
5. **Verify**: Test signature verification

### JavaScript Registration

```javascript
import { createServerIntegration } from '@datafold/js-sdk';

// Create server integration client
const integration = createServerIntegration({
  baseUrl: 'https://api.mydatafold.com',
  timeout: 30000,
  retries: 3
});

// Register public key with server
try {
  const registration = await integration.registerKeyPair(keyPair, {
    clientId: 'myapp-production-001',
    keyName: 'Production API Signing Key',
    keyDescription: 'Primary signing key for production API requests'
  });

  console.log('Registration successful!');
  console.log('Client ID:', registration.clientId);
  console.log('Key fingerprint:', registration.keyFingerprint);
  
  // Store client ID for future use
  localStorage.setItem('datafold_client_id', registration.clientId);
  
} catch (error) {
  console.error('Registration failed:', error.message);
  throw error;
}
```

### Python Registration

```python
from datafold_sdk import DataFoldClient

# Create client session with auto-registration
client = DataFoldClient("https://api.mydatafold.com")

try:
    session = client.create_new_session(
        client_id="myapp-production-001",
        key_name="Production API Signing Key",
        key_pair=key_pair,
        auto_register=True,
        save_to_storage=True
    )
    
    print(f"Registration successful! Client ID: {session.client_id}")
    print(f"Key fingerprint: {session.key_fingerprint}")
    
except Exception as error:
    print(f"Registration failed: {error}")
    raise
```

### CLI Registration

```bash
# Register public key with server
datafold_cli crypto register-key \
  --key-id "production-key" \
  --server "https://api.mydatafold.com" \
  --client-id "myapp-production-001" \
  --key-name "Production API Signing Key"

# Verify registration status
datafold_cli crypto check-registration \
  --client-id "myapp-production-001" \
  --server "https://api.mydatafold.com"
```

---

## Signature Generation & Verification

Generate digital signatures locally and verify them with the server.

### JavaScript Signing & Verification

```javascript
// Generate signature for a message
const message = "Important API request data";
const signature = await integration.generateSignature(
  message,
  keyPair.privateKey
);

console.log('Signature (base64):', signature.signature);

// Verify signature with server
const verification = await integration.verifySignature({
  clientId: 'myapp-production-001',
  message: message,
  signature: signature.signature
});

if (verification.verified) {
  console.log('Signature verified successfully!');
  console.log('Verification timestamp:', verification.timestamp);
} else {
  console.error('Signature verification failed:', verification.error);
}
```

### Python Signing & Verification

```python
# Sign message and verify with server
message = "Important API request data"

try:
    # Generate signature
    signature = session.sign_message(message)
    print(f"Generated signature: {signature}")
    
    # Verify with server
    verification_result = session.verify_with_server(message, signature)
    
    if verification_result.verified:
        print("Signature verified successfully!")
        print(f"Verification ID: {verification_result.verification_id}")
    else:
        print(f"Verification failed: {verification_result.error}")
        
except Exception as error:
    print(f"Signing/verification error: {error}")
```

### CLI Signing & Verification

```bash
# Sign a message
datafold_cli crypto sign \
  --key-id "production-key" \
  --message "Important API request data" \
  --output signature.bin

# Verify signature with server
datafold_cli crypto verify \
  --key-id "production-key" \
  --message "Important API request data" \
  --signature signature.bin \
  --server "https://api.mydatafold.com"
```

---

## Practical Integration Examples

### Web Application Integration

```javascript
// Web app initialization
class DataFoldCrypto {
  constructor(serverUrl) {
    this.serverUrl = serverUrl;
    this.storage = null;
    this.integration = null;
    this.clientId = null;
  }

  async initialize() {
    // Check browser compatibility
    const { compatible, warnings } = await initializeSDK();
    if (!compatible) {
      throw new Error(`Browser not supported: ${warnings.join(', ')}`);
    }

    // Create storage and server integration
    this.storage = await createStorage({ dbName: 'MyAppCrypto' });
    this.integration = createServerIntegration({ baseUrl: this.serverUrl });
    
    // Load existing client ID or generate new key pair
    this.clientId = localStorage.getItem('datafold_client_id');
    if (!this.clientId) {
      await this.setupNewUser();
    }
  }

  async setupNewUser() {
    // Generate new key pair
    const keyPair = await generateKeyPair();
    
    // Store securely with user-provided passphrase
    const passphrase = await this.promptForPassphrase();
    await this.storage.storeKeyPair('user-key', keyPair, passphrase, {
      name: 'User Signing Key',
      created: new Date().toISOString()
    });

    // Register with server
    const registration = await this.integration.registerKeyPair(keyPair, {
      clientId: `user-${Date.now()}`,
      keyName: 'User Signing Key'
    });

    this.clientId = registration.clientId;
    localStorage.setItem('datafold_client_id', this.clientId);
  }

  async signAPIRequest(requestData) {
    const passphrase = await this.promptForPassphrase();
    const keyPair = await this.storage.retrieveKeyPair('user-key', passphrase);
    
    const message = JSON.stringify(requestData);
    const signature = await this.integration.generateSignature(message, keyPair.privateKey);
    
    return {
      data: requestData,
      signature: signature.signature,
      clientId: this.clientId
    };
  }

  async promptForPassphrase() {
    // Implement secure passphrase prompt
    return prompt('Enter your key passphrase:');
  }
}

// Usage
const crypto = new DataFoldCrypto('https://api.mydatafold.com');
await crypto.initialize();

// Sign API requests
const signedRequest = await crypto.signAPIRequest({ 
  action: 'transfer', 
  amount: 1000 
});
```

### Desktop Application Integration

```python
import os
import json
from pathlib import Path
from datafold_sdk import DataFoldClient
from datafold_sdk.crypto import SecureKeyStorage, export_key_to_file

class DesktopCryptoManager:
    def __init__(self, app_name, server_url):
        self.app_name = app_name
        self.server_url = server_url
        self.storage = SecureKeyStorage()
        self.client = DataFoldClient(server_url)
        self.config_dir = Path.home() / f'.{app_name}'
        self.config_dir.mkdir(exist_ok=True)
        
    def initialize(self):
        """Initialize crypto system for desktop app."""
        # Check for existing configuration
        config_file = self.config_dir / 'crypto_config.json'
        if config_file.exists():
            return self.load_existing_config()
        else:
            return self.setup_new_installation()
    
    def setup_new_installation(self):
        """Set up crypto for new installation."""
        print("Setting up cryptographic keys for first time...")
        
        # Generate key pair
        key_pair = datafold_sdk.generate_key_pair()
        
        # Get passphrase from user
        passphrase = self.get_secure_passphrase()
        
        # Store key securely
        key_id = f"{self.app_name}-desktop-key"
        metadata = self.storage.store_key(key_id, key_pair, passphrase)
        
        # Create backup
        backup_path = self.config_dir / 'key_backup.json'
        export_key_to_file(
            key_pair=key_pair,
            passphrase=passphrase,
            key_id=key_id,
            file_path=str(backup_path)
        )
        
        # Register with server
        session = self.client.create_new_session(
            client_id=f"{self.app_name}-{os.getenv('USER', 'user')}",
            key_name=f"{self.app_name} Desktop Key",
            key_pair=key_pair,
            auto_register=True
        )
        
        # Save configuration
        config = {
            'key_id': key_id,
            'client_id': session.client_id,
            'key_fingerprint': session.key_fingerprint,
            'backup_path': str(backup_path)
        }
        
        with open(self.config_dir / 'crypto_config.json', 'w') as f:
            json.dump(config, f, indent=2)
        
        print(f"Setup complete! Client ID: {session.client_id}")
        return config
    
    def sign_data(self, data):
        """Sign data for API requests."""
        config = self.load_config()
        passphrase = self.get_secure_passphrase()
        
        # Retrieve key
        key_pair = self.storage.retrieve_key(config['key_id'], passphrase)
        
        # Create session and sign
        session = self.client.create_session_with_key(
            client_id=config['client_id'],
            key_pair=key_pair
        )
        
        message = json.dumps(data, sort_keys=True)
        signature = session.sign_message(message)
        
        return {
            'data': data,
            'signature': signature,
            'client_id': config['client_id']
        }
    
    def get_secure_passphrase(self):
        """Get passphrase securely (implement based on UI framework)."""
        import getpass
        return getpass.getpass("Enter key passphrase: ")
    
    def load_config(self):
        """Load existing configuration."""
        with open(self.config_dir / 'crypto_config.json', 'r') as f:
            return json.load(f)

# Usage
crypto_manager = DesktopCryptoManager('myapp', 'https://api.mydatafold.com')
config = crypto_manager.initialize()

# Sign data
signed_data = crypto_manager.sign_data({'action': 'backup', 'files': ['data.db']})
```

### CI/CD Pipeline Integration

```bash
#!/bin/bash
# CI/CD script for automated key registration and signing

set -e

# Configuration
DATAFOLD_SERVER="https://api.mydatafold.com"
CLIENT_ID="ci-pipeline-${CI_PIPELINE_ID}"
KEY_ID="ci-signing-key"

# Initialize crypto system
echo "Initializing DataFold crypto system..."
datafold_cli crypto init --method random --security-level balanced

# Generate key pair for this pipeline run
echo "Generating key pair..."
datafold_cli crypto generate-key \
  --key-id "$KEY_ID" \
  --format hex

# Register public key with server
echo "Registering public key..."
datafold_cli crypto register-key \
  --key-id "$KEY_ID" \
  --server "$DATAFOLD_SERVER" \
  --client-id "$CLIENT_ID" \
  --key-name "CI Pipeline Key"

# Function to sign build artifacts
sign_artifact() {
  local artifact_path="$1"
  local signature_path="${artifact_path}.sig"
  
  echo "Signing artifact: $artifact_path"
  
  # Calculate hash of artifact
  local artifact_hash=$(sha256sum "$artifact_path" | cut -d' ' -f1)
  
  # Sign the hash
  datafold_cli crypto sign \
    --key-id "$KEY_ID" \
    --message "$artifact_hash" \
    --output "$signature_path"
  
  # Verify signature with server
  datafold_cli crypto verify \
    --key-id "$KEY_ID" \
    --message "$artifact_hash" \
    --signature "$signature_path" \
    --server "$DATAFOLD_SERVER"
  
  echo "Artifact signed and verified: $signature_path"
}

# Sign all build artifacts
for artifact in dist/*.tar.gz; do
  if [ -f "$artifact" ]; then
    sign_artifact "$artifact"
  fi
done

# Create deployment manifest with signatures
cat > deployment_manifest.json << EOF
{
  "pipeline_id": "$CI_PIPELINE_ID",
  "client_id": "$CLIENT_ID",
  "artifacts": [
$(for artifact in dist/*.tar.gz; do
  if [ -f "$artifact" ]; then
    echo "    {"
    echo "      \"file\": \"$(basename "$artifact")\","
    echo "      \"hash\": \"$(sha256sum "$artifact" | cut -d' ' -f1)\","
    echo "      \"signature\": \"$(base64 -w 0 "${artifact}.sig")\""
    echo "    },"
  fi
done | sed '$ s/,$//')
  ],
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "Deployment manifest created with verified signatures"
```

### Automation Script Integration

```python
#!/usr/bin/env python3
"""
Automation script for batch operations with DataFold key management.
"""

import os
import sys
import json
import argparse
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from datafold_sdk import DataFoldClient
from datafold_sdk.crypto import SecureKeyStorage

class DataFoldAutomation:
    def __init__(self, server_url, config_file=None):
        self.server_url = server_url
        self.config_file = config_file or Path.home() / '.datafold_automation.json'
        self.storage = SecureKeyStorage()
        self.client = DataFoldClient(server_url)
        
    def setup(self, client_id, passphrase):
        """Set up automation with new key pair."""
        # Generate key pair
        key_pair = datafold_sdk.generate_key_pair()
        
        # Store key
        key_id = "automation-key"
        self.storage.store_key(key_id, key_pair, passphrase)
        
        # Register with server
        session = self.client.create_new_session(
            client_id=client_id,
            key_name="Automation Script Key",
            key_pair=key_pair,
            auto_register=True
        )
        
        # Save configuration
        config = {
            'key_id': key_id,
            'client_id': session.client_id,
            'server_url': self.server_url
        }
        
        with open(self.config_file, 'w') as f:
            json.dump(config, f, indent=2)
        
        print(f"Setup complete. Client ID: {session.client_id}")
        return config
    
    def process_batch(self, data_files, passphrase, max_workers=5):
        """Process multiple files with signature verification."""
        config = self.load_config()
        
        # Retrieve key
        key_pair = self.storage.retrieve_key(config['key_id'], passphrase)
        
        # Create session
        session = self.client.create_session_with_key(
            client_id=config['client_id'],
            key_pair=key_pair
        )
        
        def process_file(file_path):
            """Process a single file."""
            try:
                with open(file_path, 'r') as f:
                    data = json.load(f)
                
                # Sign data
                message = json.dumps(data, sort_keys=True)
                signature = session.sign_message(message)
                
                # Verify signature
                verification = session.verify_with_server(message, signature)
                
                if verification.verified:
                    # Process data (implement your business logic)
                    result = self.process_data(data)
                    return {
                        'file': file_path,
                        'status': 'success',
                        'result': result,
                        'verification_id': verification.verification_id
                    }
                else:
                    return {
                        'file': file_path,
                        'status': 'verification_failed',
                        'error': verification.error
                    }
                    
            except Exception as error:
                return {
                    'file': file_path,
                    'status': 'error',
                    'error': str(error)
                }
        
        # Process files concurrently
        results = []
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            future_to_file = {
                executor.submit(process_file, file_path): file_path 
                for file_path in data_files
            }
            
            for future in as_completed(future_to_file):
                result = future.result()
                results.append(result)
                print(f"Processed {result['file']}: {result['status']}")
        
        return results
    
    def process_data(self, data):
        """Implement your data processing logic here."""
        # Example: validate and transform data
        return {'processed': True, 'record_count': len(data.get('records', []))}
    
    def load_config(self):
        """Load configuration file."""
        with open(self.config_file, 'r') as f:
            return json.load(f)

def main():
    parser = argparse.ArgumentParser(description='DataFold Automation Script')
    parser.add_argument('--server', required=True, help='DataFold server URL')
    parser.add_argument('--setup', help='Setup with client ID')
    parser.add_argument('--process', nargs='+', help='Files to process')
    parser.add_argument('--workers', type=int, default=5, help='Number of workers')
    
    args = parser.parse_args()
    
    automation = DataFoldAutomation(args.server)
    
    if args.setup:
        passphrase = getpass.getpass("Enter passphrase for key storage: ")
        automation.setup(args.setup, passphrase)
    elif args.process:
        passphrase = getpass.getpass("Enter key passphrase: ")
        results = automation.process_batch(args.process, passphrase, args.workers)
        
        # Print summary
        success_count = sum(1 for r in results if r['status'] == 'success')
        print(f"\nProcessing complete: {success_count}/{len(results)} successful")
    else:
        parser.print_help()

if __name__ == '__main__':
    import getpass
    import datafold_sdk
    main()
```

---

## Server Configuration & Deployment

### Required Server Settings

#### DataFold Server Configuration

```yaml
# config/crypto.yml
crypto:
  enabled: true
  algorithms:
    - ed25519
  
  # Key registration settings
  registration:
    enabled: true
    rate_limit: "100/hour"  # Per client IP
    max_keys_per_client: 10
    key_expiry_days: 365
    
  # Signature verification settings
  verification:
    enabled: true
    rate_limit: "1000/hour"  # Per client ID
    cache_ttl: 300  # Cache verification results
    
  # Security settings
  security:
    require_tls: true
    cors_origins:
      - "https://myapp.example.com"
      - "https://staging.myapp.example.com"
    
  # Storage settings
  storage:
    backend: "postgresql"  # or "sqlite", "redis"
    connection_string: "${DATABASE_URL}"
    table_prefix: "datafold_crypto_"
```

#### TLS Configuration

```nginx
# nginx.conf
server {
    listen 443 ssl http2;
    server_name api.mydatafold.com;
    
    ssl_certificate /etc/ssl/certs/mydatafold.crt;
    ssl_certificate_key /etc/ssl/private/mydatafold.key;
    
    # Strong TLS configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;
    
    location /api/crypto/ {
        proxy_pass http://datafold-server:9001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Rate limiting
        limit_req zone=api burst=20 nodelay;
    }
}

# Rate limiting configuration
http {
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
}
```

### Database Schema

```sql
-- PostgreSQL schema for key registration
CREATE TABLE datafold_crypto_registered_keys (
    id SERIAL PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL UNIQUE,
    public_key BYTEA NOT NULL,
    key_algorithm VARCHAR(50) NOT NULL DEFAULT 'ed25519',
    key_name VARCHAR(255),
    key_fingerprint VARCHAR(128) NOT NULL,
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT TRUE,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE datafold_crypto_verification_log (
    id SERIAL PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL,
    message_hash VARCHAR(128) NOT NULL,
    signature BYTEA NOT NULL,
    verified BOOLEAN NOT NULL,
    verification_id UUID DEFAULT gen_random_uuid(),
    client_ip INET,
    user_agent TEXT,
    verified_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_registered_keys_client_id ON datafold_crypto_registered_keys(client_id);
CREATE INDEX idx_registered_keys_fingerprint ON datafold_crypto_registered_keys(key_fingerprint);
CREATE INDEX idx_registered_keys_active ON datafold_crypto_registered_keys(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_verification_log_client_id ON datafold_crypto_verification_log(client_id);
CREATE INDEX idx_verification_log_verified_at ON datafold_crypto_verification_log(verified_at);
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin datafold_node

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/datafold_node /usr/local/bin/
COPY config/ /app/config/

EXPOSE 9001
CMD ["datafold_node", "--config", "/app/config/production.yml"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  datafold-server:
    build: .
    ports:
      - "9001:9001"
    environment:
      - DATABASE_URL=postgresql://user:pass@postgres:5432/datafold
      - RUST_LOG=info
    depends_on:
      - postgres
    volumes:
      - ./config:/app/config:ro
    restart: unless-stopped
    
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=datafold
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped
    
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl:ro
    depends_on:
      - datafold-server
    restart: unless-stopped

volumes:
  postgres_data:
```

---

## Authentication & Authorization Patterns

### Client Authentication Methods

#### API Token-Based Authentication

```javascript
// JavaScript - API Token Authentication
const integration = createServerIntegration({
  baseUrl: 'https://api.mydatafold.com',
  authentication: {
    type: 'bearer_token',
    token: process.env.DATAFOLD_API_TOKEN
  }
});

// Register with API token
const registration = await integration.registerKeyPair(keyPair, {
  clientId: 'myapp-production-001',
  keyName: 'Production API Key'
});
```

#### Client Certificate Authentication

```python
# Python - Client Certificate Authentication
import ssl
from datafold_sdk import DataFoldClient

# Create SSL context with client certificate
ssl_context = ssl.create_default_context()
ssl_context.load_cert_chain(
    certfile='client.pem',
    keyfile='client.key'
)

client = DataFoldClient(
    "https://api.mydatafold.com",
    ssl_context=ssl_context
)
```

#### HMAC Request Signing

```bash
# CLI - HMAC Request Signing
export DATAFOLD_API_KEY="your-api-key"
export DATAFOLD_API_SECRET="your-api-secret"

datafold_cli crypto register-key \
  --key-id "production-key" \
  --server "https://api.mydatafold.com" \
  --client-id "myapp-production-001" \
  --auth-method hmac
```

### Role-Based Access Control

```yaml
# RBAC Configuration
rbac:
  roles:
    admin:
      permissions:
        - crypto:register
        - crypto:verify
        - crypto:revoke
        - crypto:audit
        
    developer:
      permissions:
        - crypto:register
        - crypto:verify
        
    ci_service:
      permissions:
        - crypto:register
        - crypto:verify
      restrictions:
        client_id_pattern: "^ci-.*"
        
    readonly:
      permissions:
        - crypto:verify

  assignments:
    - user: "admin@mycompany.com"
      role: admin
    - api_key: "dev-team-key"
      role: developer
    - client_cert_cn: "ci.mycompany.com"
      role: ci_service
```

---

## Troubleshooting Common Issues

### Registration Failures

#### Network Connectivity Issues

```javascript
// JavaScript - Network troubleshooting
const integration = createServerIntegration({
  baseUrl: 'https://api.mydatafold.com',
  timeout: 30000,
  retries: 3,
  retryDelay: 1000
});

try {
  const registration = await integration.registerKeyPair(keyPair, options);
} catch (error) {
  if (error.code === 'NETWORK_ERROR') {
    console.error('Network connectivity issue:', error.message);
    // Check DNS resolution, firewall, proxy settings
  } else if (error.code === 'TIMEOUT') {
    console.error('Request timeout - server may be overloaded');
    // Implement exponential backoff
  }
}
```

#### Duplicate Key Registration

```python
# Python - Handle duplicate key registration
try:
    session = client.create_new_session(
        client_id="myapp-001",
        key_pair=key_pair,
        auto_register=True
    )
except Exception as error:
    if "duplicate key" in str(error).lower():
        print("Key already registered. Retrieving existing registration...")
        # Use existing registration or rotate key
        session = client.create_session_with_existing_key(
            client_id="myapp-001",
            key_pair=key_pair
        )
    else:
        raise
```

#### Invalid Key Format

```bash
# CLI - Diagnose key format issues
datafold_cli crypto validate-key --key-id "my-key"

# Check key format
datafold_cli crypto export-public-key \
  --key-id "my-key" \
  --format hex \
  --validate

# Regenerate if corrupted
datafold_cli crypto regenerate-key \
  --key-id "my-key" \
  --backup-old
```

### Signature Verification Errors

#### Clock Skew Issues

```javascript
// JavaScript - Handle clock skew
const verification = await integration.verifySignature({
  clientId: 'myapp-001',
  message: message,
  signature: signature,
  timestamp: Date.now(),  // Include timestamp
  tolerance: 300  // 5-minute tolerance
});

if (!verification.verified && verification.error === 'TIMESTAMP_SKEW') {
  console.warn('Clock skew detected. Syncing time...');
  // Implement NTP sync or time adjustment
}
```

#### Key Rotation During Verification

```python
# Python - Handle key rotation
try:
    verification = session.verify_with_server(message, signature)
except KeyRotationError as error:
    print(f"Key was rotated. New key version: {error.new_version}")
    # Update local key version
    session.update_key_version(error.new_version)
    # Retry verification
    verification = session.verify_with_server(message, signature)
```

### Storage and Backup Issues

#### Corrupted Key Storage

```javascript
// JavaScript - Recover from corrupted storage
import { validateKeyStorage, repairStorage } from '@datafold/js-sdk';

try {
  const keyPair = await storage.retrieveKeyPair('my-key', passphrase);
} catch (error) {
  if (error.code === 'STORAGE_CORRUPTED') {
    console.warn('Storage corrupted. Attempting repair...');
    
    const repairResult = await repairStorage(storage);
    if (repairResult.success) {
      const keyPair = await storage.retrieveKeyPair('my-key', passphrase);
    } else {
      console.error('Repair failed. Restore from backup.');
      // Implement backup restoration
    }
  }
}
```

#### Backup File Issues

```python
# Python - Validate and repair backup files
from datafold_sdk.crypto import validate_backup, repair_backup

backup_path = "my-key-backup.json"

try:
    # Validate backup integrity
    validation = validate_backup(backup_path)
    if not validation.valid:
        print(f"Backup validation failed: {validation.errors}")
        
        # Attempt repair
        if validation.repairable:
            repaired_backup = repair_backup(backup_path)
            print(f"Backup repaired: {repaired_backup.path}")
        else:
            print("Backup is not repairable. Manual intervention required.")
            
except Exception as error:
    print(f"Backup error: {error}")
```

#### Permission Issues

```bash
# CLI - Fix file permission issues
# Check current permissions
ls -la ~/.datafold/keys/

# Fix permissions if needed
chmod 700 ~/.datafold/keys/
chmod 600 ~/.datafold/keys/*

# Verify crypto system integrity
datafold_cli crypto verify-system \
  --check-permissions \
  --check-encryption \
  --repair
```

### Migration Issues

#### Cross-Platform Compatibility

```python
# Python - Handle cross-platform migration issues
import platform
from datafold_sdk.crypto import KeyMigrationManager

migration_manager = KeyMigrationManager()

# Check compatibility
compatibility = migration_manager.check_compatibility(backup_data)
if not compatibility.compatible:
    print(f"Compatibility issues: {compatibility.issues}")
    
    # Apply compatibility fixes
    fixed_backup = migration_manager.apply_compatibility_fixes(
        backup_data, 
        target_platform=platform.system()
    )
    
    # Import fixed backup
    key_pair, metadata = migration_manager.import_key(fixed_backup, passphrase)
```

---

## Security Best Practices

### Passphrase Policies

#### Strong Passphrase Requirements

```javascript
// JavaScript - Passphrase validation
import { validatePassphrase } from '@datafold/js-sdk';

function enforcePassphrasePolicy(passphrase) {
  const validation = validatePassphrase(passphrase);
  
  if (!validation.valid) {
    throw new Error(`Weak passphrase: ${validation.issues.join(', ')}`);
  }
  
  const requirements = {
    minLength: 12,
    requireUppercase: true,
    requireLowercase: true,
    requireNumbers: true,
    requireSymbols: true,
    prohibitCommonWords: true
  };
  
  return validation.score >= 4; // Strong passphrase required
}

// Usage
const passphrase = await promptSecurePassphrase();
if (!enforcePassphrasePolicy(passphrase)) {
  throw new Error('Passphrase does not meet security requirements');
}
```

#### Passphrase Storage Considerations

```python
# Python - Secure passphrase handling
import getpass
import os
from datafold_sdk.crypto import SecureMemory

class SecurePassphraseManager:
    @staticmethod
    def get_passphrase_interactive():
        """Get passphrase from user input."""
        return getpass.getpass("Enter key passphrase: ")
    
    @staticmethod
    def get_passphrase_from_env():
        """Get passphrase from environment (CI/CD)."""
        passphrase = os.getenv('DATAFOLD_KEY_PASSPHRASE')
        if not passphrase:
            raise ValueError("DATAFOLD_KEY_PASSPHRASE environment variable not set")
        return passphrase
    
    @staticmethod
    def clear_passphrase(passphrase):
        """Securely clear passphrase from memory."""
        SecureMemory.clear_string(passphrase)
```

### Memory Security

```javascript
// JavaScript - Secure memory handling
import { clearKeyMaterial, SecureArray } from '@datafold/js-sdk';

async function secureKeyOperation(keyPair, message) {
  const signature = await generateSignature(message, keyPair.privateKey);
  
  // Clear sensitive data when done
  clearKeyMaterial(keyPair);
  
  return signature;
}

// Use SecureArray for sensitive data
const sensitiveData = new SecureArray(32);
// ... use data ...
sensitiveData.clear(); // Explicitly clear when done
```

### File System Security

```bash
# CLI - Secure file permissions
# Set restrictive permissions on crypto directories
chmod 700 ~/.datafold/
chmod 700 ~/.datafold/keys/
chmod 600 ~/.datafold/keys/*
chmod 600 ~/.datafold/config/*

# Verify no world-readable files
find ~/.datafold -type f -perm /044 -exec echo "Warning: {} is world-readable" \;

# Set up encrypted home directory (Linux)
sudo apt-get install ecryptfs-utils
ecryptfs-setup-private
```

### Network Security

#### TLS Configuration

```python
# Python - Strict TLS verification
import ssl
from datafold_sdk import DataFoldClient

# Create strict SSL context
ssl_context = ssl.create_default_context()
ssl_context.check_hostname = True
ssl_context.verify_mode = ssl.CERT_REQUIRED
ssl_context.minimum_version = ssl.TLSVersion.TLSv1_2

# Pin certificate (optional)
ssl_context.load_verify_locations('/path/to/server.pem')

client = DataFoldClient(
    "https://api.mydatafold.com",
    ssl_context=ssl_context
)
```

#### Certificate Pinning

```javascript
// JavaScript - Certificate pinning (Node.js)
import https from 'https';
import crypto from 'crypto';

const expectedFingerprint = 'AA:BB:CC:DD:EE:FF:...'; // Server cert fingerprint

const agent = new https.Agent({
  checkServerIdentity: (hostname, cert) => {
    const fingerprint = crypto
      .createHash('sha256')
      .update(cert.raw)
      .digest('hex')
      .toUpperCase()
      .match(/.{2}/g)
      .join(':');
      
    if (fingerprint !== expectedFingerprint) {
      throw new Error(`Certificate fingerprint mismatch: ${fingerprint}`);
    }
  }
});

const integration = createServerIntegration({
  baseUrl: 'https://api.mydatafold.com',
  httpsAgent: agent
});
```

### Production Deployment Security

#### Security Checklist

- [ ] TLS 1.2+ required for all connections
- [ ] Certificate pinning implemented
- [ ] Rate limiting configured
- [ ] Request/response logging enabled
- [ ] Key rotation procedures documented
- [ ] Backup encryption verified
- [ ] File permissions locked down
- [ ] Environment variables secured
- [ ] Security monitoring enabled
- [ ] Incident response plan ready

#### Environment-Specific Security

```yaml
# Production environment configuration
production:
  security:
    tls_min_version: "1.2"
    require_client_certs: true
    rate_limit_strict: true
    audit_logging: full
    
development:
  security:
    tls_min_version: "1.2"
    require_client_certs: false
    rate_limit_strict: false
    audit_logging: minimal
```

---

## Performance Optimization

### Key Operation Benchmarks

#### JavaScript SDK Performance

```javascript
// Performance testing for JavaScript SDK
import { performance } from 'perf_hooks';

async function benchmarkKeyOperations() {
  const iterations = 1000;
  const results = {};
  
  // Benchmark key generation
  const genStart = performance.now();
  for (let i = 0; i < iterations; i++) {
    await generateKeyPair();
  }
  results.keyGeneration = (performance.now() - genStart) / iterations;
  
  // Benchmark signing
  const keyPair = await generateKeyPair();
  const message = "benchmark message";
  const signStart = performance.now();
  for (let i = 0; i < iterations; i++) {
    await generateSignature(message, keyPair.privateKey);
  }
  results.signing = (performance.now() - signStart) / iterations;
  
  console.log('JavaScript SDK Benchmarks (ms/operation):');
  console.log(`Key Generation: ${results.keyGeneration.toFixed(2)}`);
  console.log(`Signing: ${results.signing.toFixed(2)}`);
  
  return results;
}
```

#### Python SDK Performance

```python
# Performance testing for Python SDK
import time
import statistics
import datafold_sdk

def benchmark_key_operations(iterations=1000):
    """Benchmark key operations."""
    results = {}
    
    # Benchmark key generation
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        key_pair = datafold_sdk.generate_key_pair()
        times.append(time.perf_counter() - start)
    
    results['key_generation'] = {
        'mean': statistics.mean(times) * 1000,  # Convert to ms
        'median': statistics.median(times) * 1000,
        'stdev': statistics.stdev(times) * 1000
    }
    
    # Benchmark signing
    key_pair = datafold_sdk.generate_key_pair()
    message = "benchmark message"
    times = []
    
    for _ in range(iterations):
        start = time.perf_counter()
        signature = datafold_sdk.sign_message(message, key_pair.private_key)
        times.append(time.perf_counter() - start)
    
    results['signing'] = {
        'mean': statistics.mean(times) * 1000,
        'median': statistics.median(times) * 1000,
        'stdev': statistics.stdev(times) * 1000
    }
    
    print("Python SDK Benchmarks (ms/operation):")
    for operation, stats in results.items():
        print(f"{operation.title()}: {stats['mean']:.2f} ± {stats['stdev']:.2f}")
    
    return results
```

#### CLI Performance

```bash
#!/bin/bash
# CLI performance benchmarking

iterations=100
echo "CLI Performance Benchmark ($iterations iterations)"

# Benchmark key generation
echo "Benchmarking key generation..."
start_time=$(date +%s.%N)
for i in $(seq 1 $iterations); do
  datafold_cli crypto generate-key --key-id "bench-$i" --format hex > /dev/null
done
end_time=$(date +%s.%N)
gen_time=$(echo "scale=2; ($end_time - $start_time) * 1000 / $iterations" | bc)
echo "Key generation: ${gen_time}ms/operation"

# Benchmark signing
echo "Benchmarking signing..."
datafold_cli crypto generate-key --key-id "bench-signing" --format hex > /dev/null
start_time=$(date +%s.%N)
for i in $(seq 1 $iterations); do
  datafold_cli crypto sign --key-id "bench-signing" --message "benchmark" --output /tmp/sig-$i > /dev/null
done
end_time=$(date +%s.%N)
sign_time=$(echo "scale=2; ($end_time - $start_time) * 1000 / $iterations" | bc)
echo "Signing: ${sign_time}ms/operation"

# Cleanup
datafold_cli crypto delete-key --key-id "bench-signing"
rm -f /tmp/sig-*
```

### Storage Performance Optimization

#### JavaScript IndexedDB Optimization

```javascript
// Optimized storage operations
class OptimizedKeyStorage {
  constructor() {
    this.connectionPool = new Map();
    this.cacheEnabled = true;
    this.cache = new Map();
  }
  
  async storeKeyPairBatch(keyPairs, passphrase) {
    // Batch storage operations for better performance
    const transaction = await this.beginTransaction('readwrite');
    const promises = [];
    
    for (const [keyId, keyPair] of keyPairs.entries()) {
      promises.push(this.storeKeyPairInTransaction(
        transaction, keyId, keyPair, passphrase
      ));
    }
    
    await Promise.all(promises);
    await transaction.complete();
  }
  
  async retrieveKeyPairCached(keyId, passphrase) {
    // Use cache for frequently accessed keys
    const cacheKey = `${keyId}:${this.hashPassphrase(passphrase)}`;
    
    if (this.cacheEnabled && this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey);
    }
    
    const keyPair = await this.retrieveKeyPair(keyId, passphrase);
    
    if (this.cacheEnabled) {
      this.cache.set(cacheKey, keyPair);
      // Clear cache after 5 minutes
      setTimeout(() => this.cache.delete(cacheKey), 300000);
    }
    
    return keyPair;
  }
}
```

#### Python Storage Optimization

```python
# Optimized Python storage with connection pooling
import threading
from concurrent.futures import ThreadPoolExecutor
from datafold_sdk.crypto import SecureKeyStorage

class OptimizedSecureStorage:
    def __init__(self, max_workers=5):
        self.storage = SecureKeyStorage()
        self.executor = ThreadPoolExecutor(max_workers=max_workers)
        self.cache = {}
        self.cache_lock = threading.RLock()
        
    def store_keys_batch(self, key_data_list):
        """Store multiple keys concurrently."""
        futures = []
        
        for key_id, key_pair, passphrase in key_data_list:
            future = self.executor.submit(
                self.storage.store_key, key_id, key_pair, passphrase
            )
            futures.append(future)
        
        # Wait for all operations to complete
        results = []
        for future in futures:
            results.append(future.result())
        
        return results
    
    def retrieve_key_cached(self, key_id, passphrase, cache_ttl=300):
        """Retrieve key with caching."""
        cache_key = f"{key_id}:{hash(passphrase)}"
        
        with self.cache_lock:
            if cache_key in self.cache:
                cached_time, key_pair = self.cache[cache_key]
                if time.time() - cached_time < cache_ttl:
                    return key_pair
                else:
                    del self.cache[cache_key]
        
        # Retrieve from storage
        key_pair = self.storage.retrieve_key(key_id, passphrase)
        
        # Cache result
        with self.cache_lock:
            self.cache[cache_key] = (time.time(), key_pair)
        
        return key_pair
```

### Server-Side Performance Tuning

#### Database Optimization

```sql
-- PostgreSQL performance optimizations
-- Optimize registration queries
CREATE INDEX CONCURRENTLY idx_registered_keys_client_lookup 
ON datafold_crypto_registered_keys(client_id, is_active) 
WHERE is_active = TRUE;

-- Optimize verification log queries
CREATE INDEX CONCURRENTLY idx_verification_log_recent 
ON datafold_crypto_verification_log(verified_at DESC) 
WHERE verified_at > NOW() - INTERVAL '30 days';

-- Partition verification log by date
CREATE TABLE datafold_crypto_verification_log_y2025m01 
PARTITION OF datafold_crypto_verification_log 
FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

-- Analyze tables for query optimization
ANALYZE datafold_crypto_registered_keys;
ANALYZE datafold_crypto_verification_log;
```

#### Redis Caching

```yaml
# Redis configuration for verification caching
redis:
  host: redis-cluster
  port: 6379
  db: 0
  password: "${REDIS_PASSWORD}"
  
  # Cache configuration
  cache:
    verification_results:
      ttl: 300  # 5 minutes
      max_size: 10000
    
    public_keys:
      ttl: 3600  # 1 hour
      max_size: 5000
```

#### Rate Limiting Optimization

```yaml
# Optimized rate limiting configuration
rate_limiting:
  # Different limits for different operations
  registration:
    rate: "10/hour"
    burst: 5
    
  verification:
    rate: "1000/hour"
    burst: 50
    
  # IP-based limits
  per_ip:
    rate: "100/minute"
    burst: 20
    
  # Client-based limits
  per_client:
    rate: "500/hour"
    burst: 30
```

---

## Deployment Checklists & Operational Procedures

### Pre-Deployment Checklist

#### Infrastructure Requirements

- [ ] **Server Resources**
  - [ ] CPU: Minimum 2 cores, recommended 4+ cores
  - [ ] RAM: Minimum 4GB, recommended 8GB+
  - [ ] Storage: SSD recommended, 50GB+ available
  - [ ] Network: TLS-capable load balancer

- [ ] **Database Setup**
  - [ ] PostgreSQL 12+ or equivalent configured
  - [ ] Database schema deployed
  - [ ] Backup strategy implemented
  - [ ] Connection pooling configured

- [ ] **Security Configuration**
  - [ ] TLS certificates installed and valid
  - [ ] Firewall rules configured
  - [ ] Rate limiting enabled
  - [ ] Monitoring and alerting setup

#### Application Configuration

- [ ] **Crypto Settings**
  - [ ] Ed25519 algorithm enabled
  - [ ] Key expiry policies configured
  - [ ] Registration limits set
  - [ ] Verification caching enabled

- [ ] **Environment Variables**
  - [ ] Database connection strings set
  - [ ] API keys and secrets configured
  - [ ] Logging levels appropriate
  - [ ] Debug mode disabled in production

- [ ] **Testing**
  - [ ] Health check endpoint responding
  - [ ] Key registration functional
  - [ ] Signature verification working
  - [ ] Performance benchmarks acceptable

### Key Rotation Procedures

#### Scheduled Key Rotation

```bash
#!/bin/bash
# Automated key rotation script

set -e

KEY_ID="production-api-key"
CLIENT_ID="myapp-production-001"
SERVER_URL="https://api.mydatafold.com"
BACKUP_DIR="/secure/backups/keys"

echo "Starting scheduled key rotation for $KEY_ID"

# Create backup of current key
echo "Creating backup of current key..."
datafold_cli crypto export-key \
  --key-id "$KEY_ID" \
  --output "$BACKUP_DIR/$(date +%Y%m%d_%H%M%S)_${KEY_ID}_backup.json" \
  --format json

# Rotate key
echo "Rotating key..."
datafold_cli crypto rotate-key \
  --key-id "$KEY_ID" \
  --method derive \
  --keep-old \
  --reason "scheduled_rotation"

# Re-register new key with server
echo "Re-registering key with server..."
datafold_cli crypto register-key \
  --key-id "$KEY_ID" \
  --server "$SERVER_URL" \
  --client-id "$CLIENT_ID" \
  --key-name "Production API Key (Rotated)"

# Verify new key works
echo "Verifying rotated key..."
datafold_cli crypto sign \
  --key-id "$KEY_ID" \
  --message "rotation test" \
  --output /tmp/rotation_test.sig

datafold_cli crypto verify \
  --key-id "$KEY_ID" \
  --message "rotation test" \
  --signature /tmp/rotation_test.sig \
  --server "$SERVER_URL"

echo "Key rotation completed successfully"

# Cleanup
rm -f /tmp/rotation_test.sig

# Send notification
curl -X POST "$NOTIFICATION_WEBHOOK" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"Key rotation completed for $KEY_ID\"}"
```

#### Emergency Key Rotation

```python
#!/usr/bin/env python3
"""
Emergency key rotation script for security incidents.
"""

import sys
import json
import logging
from datetime import datetime
from datafold_sdk import DataFoldClient
from datafold_sdk.crypto import SecureKeyStorage

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def emergency_rotation(client_id, reason, server_url):
    """Perform emergency key rotation."""
    logger.info(f"Starting emergency rotation for {client_id}")
    
    storage = SecureKeyStorage()
    client = DataFoldClient(server_url)
    
    try:
        # Revoke existing key on server
        logger.info("Revoking existing key on server...")
        revocation_result = client.revoke_key(
            client_id=client_id,
reason=reason,
            immediate=True
        )
        
        # Generate new key pair
        logger.info("Generating new key pair...")
        new_key_pair = datafold_sdk.generate_key_pair()
        
        # Store new key
        passphrase = getpass.getpass("Enter passphrase for new key: ")
        storage.store_key(
            key_id=f"{client_id}-emergency",
            key_pair=new_key_pair,
            passphrase=passphrase
        )
        
        # Register new key
        logger.info("Registering new key...")
        session = client.create_new_session(
            client_id=client_id,
            key_name=f"Emergency Key - {reason}",
            key_pair=new_key_pair,
            auto_register=True
        )
        
        # Verify new key works
        test_message = f"emergency rotation test {datetime.now().isoformat()}"
        signature = session.sign_message(test_message)
        verification = session.verify_with_server(test_message, signature)
        
        if verification.verified:
            logger.info("Emergency rotation completed successfully")
            return {
                'success': True,
                'client_id': client_id,
                'new_key_fingerprint': session.key_fingerprint,
                'verification_id': verification.verification_id
            }
        else:
            raise Exception(f"Verification failed: {verification.error}")
            
    except Exception as error:
        logger.error(f"Emergency rotation failed: {error}")
        return {'success': False, 'error': str(error)}

if __name__ == '__main__':
    if len(sys.argv) != 4:
        print("Usage: emergency_rotation.py <client_id> <reason> <server_url>")
        sys.exit(1)
    
    result = emergency_rotation(sys.argv[1], sys.argv[2], sys.argv[3])
    print(json.dumps(result, indent=2))
```

### Backup and Recovery Procedures

#### Automated Backup Strategy

```bash
#!/bin/bash
# Automated backup script for production keys

BACKUP_ROOT="/secure/backups/datafold-keys"
RETENTION_DAYS=90
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory structure
mkdir -p "$BACKUP_ROOT/$DATE"/{keys,config,verification-logs}

echo "Starting automated backup - $DATE"

# Backup all registered keys
datafold_cli crypto list-keys --format json > "$BACKUP_ROOT/$DATE/keys/key-inventory.json"

while IFS= read -r key_id; do
    echo "Backing up key: $key_id"
    datafold_cli crypto export-key \
        --key-id "$key_id" \
        --output "$BACKUP_ROOT/$DATE/keys/${key_id}-backup.json" \
        --format json
done < <(datafold_cli crypto list-keys --format text | grep -v "No keys found")

# Backup configuration files
cp -r /app/config/* "$BACKUP_ROOT/$DATE/config/"

# Backup recent verification logs (if accessible)
if command -v pg_dump &> /dev/null; then
    pg_dump -t datafold_crypto_verification_log \
        --data-only \
        --where "verified_at > NOW() - INTERVAL '$RETENTION_DAYS days'" \
        "$DATABASE_URL" > "$BACKUP_ROOT/$DATE/verification-logs/recent-verifications.sql"
fi

# Create integrity manifest
cd "$BACKUP_ROOT/$DATE"
find . -type f -exec sha256sum {} \; > integrity-manifest.sha256

# Compress backup
tar -czf "../datafold-backup-$DATE.tar.gz" .
cd ..
rm -rf "$DATE"

# Cleanup old backups
find "$BACKUP_ROOT" -name "datafold-backup-*.tar.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: datafold-backup-$DATE.tar.gz"

# Verify backup integrity
tar -tzf "datafold-backup-$DATE.tar.gz" > /dev/null
echo "Backup integrity verified"
```

#### Disaster Recovery Procedure

```python
#!/usr/bin/env python3
"""
Disaster recovery script for DataFold key management system.
"""

import os
import json
import tarfile
import hashlib
from pathlib import Path
from datafold_sdk.crypto import SecureKeyStorage, import_key_from_file

class DisasterRecovery:
    def __init__(self, backup_path, recovery_config):
        self.backup_path = Path(backup_path)
        self.recovery_config = recovery_config
        self.storage = SecureKeyStorage()
        
    def verify_backup_integrity(self):
        """Verify backup file integrity."""
        print("Verifying backup integrity...")
        
        # Extract backup
        with tarfile.open(self.backup_path, 'r:gz') as tar:
            tar.extractall('/tmp/recovery')
        
        # Verify checksums
        manifest_path = Path('/tmp/recovery/integrity-manifest.sha256')
        if not manifest_path.exists():
            raise Exception("Integrity manifest not found in backup")
        
        with open(manifest_path, 'r') as f:
            for line in f:
                checksum, filename = line.strip().split('  ')
                file_path = Path('/tmp/recovery') / filename
                
                if file_path.exists():
                    actual_checksum = hashlib.sha256(file_path.read_bytes()).hexdigest()
                    if actual_checksum != checksum:
                        raise Exception(f"Integrity check failed for {filename}")
                else:
                    raise Exception(f"Missing file in backup: {filename}")
        
        print("Backup integrity verified successfully")
        return True
    
    def restore_keys(self, passphrase_map):
        """Restore keys from backup."""
        print("Restoring keys from backup...")
        
        keys_dir = Path('/tmp/recovery/keys')
        inventory_path = keys_dir / 'key-inventory.json'
        
        if not inventory_path.exists():
            raise Exception("Key inventory not found in backup")
        
        with open(inventory_path, 'r') as f:
            inventory = json.load(f)
        
        restored_keys = []
        
        for key_info in inventory['keys']:
            key_id = key_info['key_id']
            backup_file = keys_dir / f"{key_id}-backup.json"
            
            if not backup_file.exists():
                print(f"Warning: Backup file not found for {key_id}")
                continue
            
            # Get passphrase for this key
            passphrase = passphrase_map.get(key_id)
            if not passphrase:
                passphrase = getpass.getpass(f"Enter passphrase for {key_id}: ")
            
            try:
                # Import key
                key_pair, metadata = import_key_from_file(
                    file_path=str(backup_file),
                    passphrase=passphrase,
                    verify_integrity=True
                )
                
                # Store in new storage
                self.storage.store_key(key_id, key_pair, passphrase)
                restored_keys.append(key_id)
                print(f"Successfully restored key: {key_id}")
                
            except Exception as error:
                print(f"Failed to restore key {key_id}: {error}")
        
        return restored_keys
    
    def re_register_keys(self, server_url, restored_keys):
        """Re-register restored keys with server."""
        print("Re-registering keys with server...")
        
        from datafold_sdk import DataFoldClient
        client = DataFoldClient(server_url)
        
        registration_results = []
        
        for key_id in restored_keys:
            try:
                # Retrieve key
                passphrase = getpass.getpass(f"Enter passphrase for {key_id}: ")
                key_pair = self.storage.retrieve_key(key_id, passphrase)
                
                # Register with server
                session = client.create_new_session(
                    client_id=f"recovered-{key_id}",
                    key_name=f"Recovered Key - {key_id}",
                    key_pair=key_pair,
                    auto_register=True
                )
                
                registration_results.append({
                    'key_id': key_id,
                    'client_id': session.client_id,
                    'status': 'success'
                })
                
                print(f"Re-registered key {key_id} with client ID: {session.client_id}")
                
            except Exception as error:
                registration_results.append({
                    'key_id': key_id,
                    'status': 'failed',
                    'error': str(error)
                })
                print(f"Failed to re-register key {key_id}: {error}")
        
        return registration_results
    
    def run_recovery(self, server_url, passphrase_map=None):
        """Run complete disaster recovery process."""
        try:
            # Verify backup
            self.verify_backup_integrity()
            
            # Restore keys
            restored_keys = self.restore_keys(passphrase_map or {})
            
            # Re-register with server
            registration_results = self.re_register_keys(server_url, restored_keys)
            
            # Generate recovery report
            report = {
                'timestamp': datetime.now().isoformat(),
                'backup_file': str(self.backup_path),
                'keys_restored': len(restored_keys),
                'registrations': registration_results,
                'status': 'completed'
            }
            
            # Save report
            with open('recovery-report.json', 'w') as f:
                json.dump(report, f, indent=2)
            
            print(f"Disaster recovery completed. Report saved to recovery-report.json")
            return report
            
        except Exception as error:
            print(f"Disaster recovery failed: {error}")
            return {'status': 'failed', 'error': str(error)}
        finally:
            # Cleanup
            if Path('/tmp/recovery').exists():
                import shutil
                shutil.rmtree('/tmp/recovery')

if __name__ == '__main__':
    import sys
    import getpass
    from datetime import datetime
    
    if len(sys.argv) != 3:
        print("Usage: disaster_recovery.py <backup_file> <server_url>")
        sys.exit(1)
    
    recovery = DisasterRecovery(sys.argv[1], {})
    result = recovery.run_recovery(sys.argv[2])
    
    if result['status'] == 'completed':
        print("Disaster recovery completed successfully")
        sys.exit(0)
    else:
        print("Disaster recovery failed")
        sys.exit(1)
```

---

## Monitoring & Logging Recommendations

### Application Metrics

#### Key Performance Indicators

```yaml
# Prometheus metrics configuration
metrics:
  registration:
    - name: datafold_key_registrations_total
      type: counter
      help: "Total number of key registration attempts"
      labels: [status, client_type]
      
    - name: datafold_key_registration_duration_seconds
      type: histogram
      help: "Time taken for key registration operations"
      buckets: [0.1, 0.5, 1.0, 2.0, 5.0]
      
  verification:
    - name: datafold_signature_verifications_total
      type: counter
      help: "Total number of signature verification attempts"
      labels: [status, client_id]
      
    - name: datafold_verification_duration_seconds
      type: histogram
      help: "Time taken for signature verification"
      buckets: [0.01, 0.05, 0.1, 0.5, 1.0]
      
  storage:
    - name: datafold_active_keys_total
      type: gauge
      help: "Number of currently active registered keys"
      
    - name: datafold_key_storage_errors_total
      type: counter
      help: "Number of key storage errors"
      labels: [error_type]
```

#### Structured Logging

```rust
// Rust - Structured logging for DataFold server
use serde_json::json;
use tracing::{info, warn, error, instrument};

#[instrument]
pub async fn register_public_key(
    client_id: String,
    public_key: Vec<u8>,
    metadata: KeyMetadata,
) -> Result<RegistrationResponse, CryptoError> {
    info!(
        client_id = %client_id,
        key_algorithm = %metadata.algorithm,
        "Starting key registration"
    );
    
    match validate_public_key(&public_key) {
        Ok(_) => {
            let fingerprint = calculate_key_fingerprint(&public_key);
            
            // Store key
            match store_registered_key(&client_id, &public_key, &metadata).await {
                Ok(registration) => {
                    info!(
                        client_id = %client_id,
                        key_fingerprint = %fingerprint,
                        registration_id = %registration.id,
                        "Key registration successful"
                    );
                    
                    // Emit metrics
                    metrics::counter!("datafold_key_registrations_total", 
                        "status" => "success",
                        "client_type" => metadata.client_type.as_str()
                    ).increment(1);
                    
                    Ok(RegistrationResponse {
                        client_id,
                        registration_id: registration.id,
                        key_fingerprint: fingerprint,
                        expires_at: registration.expires_at,
                    })
                },
                Err(error) => {
                    error!(
                        client_id = %client_id,
                        error = %error,
                        "Key registration failed - storage error"
                    );
                    
                    metrics::counter!("datafold_key_registrations_total",
                        "status" => "storage_error"
                    ).increment(1);
                    
                    Err(error)
                }
            }
        },
        Err(validation_error) => {
            warn!(
                client_id = %client_id,
                error = %validation_error,
                "Key registration failed - validation error"
            );
            
            metrics::counter!("datafold_key_registrations_total",
                "status" => "validation_error"
            ).increment(1);
            
            Err(validation_error)
        }
    }
}
```

### Security Monitoring

#### Anomaly Detection

```python
# Python - Security monitoring and anomaly detection
import time
from collections import defaultdict, deque
from datetime import datetime, timedelta

class SecurityMonitor:
    def __init__(self):
        self.failed_registrations = defaultdict(deque)
        self.verification_patterns = defaultdict(list)
        self.rate_limits = {
            'failed_registrations': 5,  # per hour
            'rapid_verifications': 100,  # per minute
        }
    
    def check_registration_attempt(self, client_id, ip_address, success):
        """Monitor registration attempts for anomalies."""
        current_time = time.time()
        
        if not success:
            # Track failed registrations
            self.failed_registrations[ip_address].append(current_time)
            
            # Remove old entries (older than 1 hour)
            while (self.failed_registrations[ip_address] and 
                   current_time - self.failed_registrations[ip_address][0] > 3600):
                self.failed_registrations[ip_address].popleft()
            
            # Check if rate limit exceeded
            if len(self.failed_registrations[ip_address]) > self.rate_limits['failed_registrations']:
                self.alert_suspicious_activity(
                    'excessive_failed_registrations',
                    {
                        'ip_address': ip_address,
                        'failed_count': len(self.failed_registrations[ip_address]),
                        'timeframe': '1 hour'
                    }
                )
    
    def check_verification_patterns(self, client_id, verification_count):
        """Monitor verification patterns for anomalies."""
        current_time = datetime.now()
        
        # Track verification frequency
        self.verification_patterns[client_id].append(current_time)
        
        # Remove old entries (older than 1 minute)
        cutoff_time = current_time - timedelta(minutes=1)
        self.verification_patterns[client_id] = [
            t for t in self.verification_patterns[client_id] 
            if t > cutoff_time
        ]
        
        # Check for rapid verification attempts
        recent_count = len(self.verification_patterns[client_id])
        if recent_count > self.rate_limits['rapid_verifications']:
            self.alert_suspicious_activity(
                'rapid_verification_attempts',
                {
                    'client_id': client_id,
                    'verification_count': recent_count,
                    'timeframe': '1 minute'
                }
            )
    
    def alert_suspicious_activity(self, alert_type, details):
        """Send security alert."""
        alert = {
            'timestamp': datetime.now().isoformat(),
            'alert_type': alert_type,
            'severity': 'high',
            'details': details
        }
        
        # Log alert
        logger.warning(f"Security alert: {alert_type}", extra=alert)
        
        # Send to monitoring system
        self.send_to_monitoring(alert)
    
    def send_to_monitoring(self, alert):
        """Send alert to monitoring system."""
        # Implementation depends on your monitoring stack
        # Examples: Datadog, New Relic, custom webhook
        pass
```

### Operational Dashboards

#### Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "title": "DataFold Key Management Operations",
    "panels": [
      {
        "title": "Key Registrations",
        "type": "stat",
        "targets": [
          {
            "expr": "increase(datafold_key_registrations_total[1h])",
            "legendFormat": "Registrations/hour"
          }
        ]
      },
      {
        "title": "Signature Verifications",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(datafold_signature_verifications_total[5m])",
            "legendFormat": "Verifications/sec"
          }
        ]
      },
      {
        "title": "Active Keys",
        "type": "stat",
        "targets": [
          {
            "expr": "datafold_active_keys_total",
            "legendFormat": "Active Keys"
          }
        ]
      },
      {
        "title": "Error Rates",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(datafold_key_registrations_total{status!=\"success\"}[5m])",
            "legendFormat": "Registration Errors"
          },
          {
            "expr": "rate(datafold_signature_verifications_total{status!=\"success\"}[5m])",
            "legendFormat": "Verification Errors"
          }
        ]
      }
    ]
  }
}
```

### Alerting Rules

#### Prometheus Alert Rules

```yaml
# alert-rules.yml
groups:
  - name: datafold-crypto
    rules:
      - alert: HighRegistrationFailureRate
        expr: |
          (
            rate(datafold_key_registrations_total{status!="success"}[5m]) /
            rate(datafold_key_registrations_total[5m])
          ) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High key registration failure rate"
          description: "Registration failure rate is {{ $value | humanizePercentage }} over the last 5 minutes"
      
      - alert: ExcessiveVerificationErrors
        expr: rate(datafold_signature_verifications_total{status!="success"}[5m]) > 10
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Excessive signature verification errors"
          description: "{{ $value }} verification errors per second over the last minute"
      
      - alert: SlowKeyOperations
        expr: |
          histogram_quantile(0.95, datafold_key_registration_duration_seconds_bucket) > 5 or
          histogram_quantile(0.95, datafold_verification_duration_seconds_bucket) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow key operations detected"
          description: "95th percentile response times are above acceptable thresholds"
      
      - alert: SuspiciousActivityPattern
        expr: increase(datafold_security_alerts_total[1h]) > 5
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Suspicious activity pattern detected"
          description: "{{ $value }} security alerts in the last hour"
```

---

## Conclusion

This comprehensive integration guide provides all necessary information for integrating DataFold's public key registration system across JavaScript SDK, Python SDK, and CLI platforms. The guide covers the complete lifecycle from key generation through deployment and operations.

### Key Takeaways

1. **Security First**: All platforms implement Ed25519 with client-side key generation and secure storage
2. **Cross-Platform Compatibility**: Standardized backup format enables key migration between platforms
3. **Production Ready**: Includes comprehensive operational procedures, monitoring, and security best practices
4. **Scalable**: Performance optimizations and server configuration for production deployments

### Implementation Checklist

- [ ] Review platform requirements and dependencies
- [ ] Implement key generation and secure storage
- [ ] Set up public key registration with server
- [ ] Configure backup and recovery procedures
- [ ] Deploy server with proper security configuration
- [ ] Set up monitoring and alerting
- [ ] Test disaster recovery procedures
- [ ] Train operations team on procedures

### Next Steps

- **Task 10-7-3**: Provide code examples and usage recipes building on these integration guides
- **Implementation**: Use these guides to implement public key registration in your applications
- **Operations**: Follow the deployment checklists and monitoring recommendations for production deployment

For additional support or questions about these integration guides, refer to the comprehensive API documentation and troubleshooting sections provided.

### Support and Resources

- **API Documentation**: [`docs/delivery/10/key-management-api-documentation.md`](key-management-api-documentation.md)
- **Task Planning**: [`docs/delivery/10/10-7-2_integration_guides_plan.md`](10-7-2_integration_guides_plan.md)
- **Implementation Examples**: See the practical integration examples section above
- **Security Guidelines**: Review the security best practices section for production deployments

---

*Document Version: 1.0*  
*Last Updated: 2025-06-08T20:05:24-07:00*  
*Task: 10-7-2 - Write integration guides for public key registration*