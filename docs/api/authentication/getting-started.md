# Getting Started with DataFold Authentication

This guide will get you up and running with DataFold's signature authentication in under 10 minutes. We'll walk through key generation, registration, and making your first authenticated API call.

## 🎯 Prerequisites

Before you begin, ensure you have:

- **DataFold Server Access**: Server URL and network connectivity
- **Development Environment**: Node.js 16+, Python 3.8+, or Rust toolchain
- **Basic HTTP Knowledge**: Understanding of HTTP requests and headers
- **Cryptography Concepts**: Basic understanding of public/private keys

## 🚀 Quick Setup (5 Minutes)

### Step 1: Choose Your Platform

<details>
<summary><strong>🟨 JavaScript/TypeScript</strong></summary>

```bash
# Install the DataFold JavaScript SDK
npm install @datafold/sdk

# Or with yarn
yarn add @datafold/sdk
```

</details>

<details>
<summary><strong>🐍 Python</strong></summary>

```bash
# Install the DataFold Python SDK
pip install datafold-sdk

# Or with poetry
poetry add datafold-sdk
```

</details>

<details>
<summary><strong>⚡ CLI Tool</strong></summary>

```bash
# Download and install the DataFold CLI
curl -sSL https://install.datafold.com | sh

# Or install via cargo
cargo install datafold-cli
```

</details>

### Step 2: Generate Your Keypair

<details>
<summary><strong>🟨 JavaScript/TypeScript</strong></summary>

```javascript
import { generateKeyPair, createSigner } from '@datafold/sdk';

// Generate Ed25519 keypair
const keyPair = await generateKeyPair();
console.log('Private Key (keep secret!):', keyPair.privateKey);
console.log('Public Key (register with server):', keyPair.publicKey);

// Save keys securely (example uses environment variables)
process.env.DATAFOLD_PRIVATE_KEY = keyPair.privateKey;
process.env.DATAFOLD_PUBLIC_KEY = keyPair.publicKey;
```

</details>

<details>
<summary><strong>🐍 Python</strong></summary>

```python
from datafold_sdk.crypto import generate_keypair
import os

# Generate Ed25519 keypair
private_key, public_key = generate_keypair()
print(f'Private Key (keep secret!): {private_key.hex()}')
print(f'Public Key (register with server): {public_key.hex()}')

# Save keys securely (example uses environment variables)
os.environ['DATAFOLD_PRIVATE_KEY'] = private_key.hex()
os.environ['DATAFOLD_PUBLIC_KEY'] = public_key.hex()
```

</details>

<details>
<summary><strong>⚡ CLI Tool</strong></summary>

```bash
# Generate Ed25519 keypair
datafold auth keygen --output-format env

# This outputs:
# export DATAFOLD_PRIVATE_KEY="1234567890abcdef..."
# export DATAFOLD_PUBLIC_KEY="abcdef1234567890..."

# Add to your shell profile
datafold auth keygen --output-format env >> ~/.bashrc
source ~/.bashrc
```

</details>

### Step 3: Register Your Public Key

<details>
<summary><strong>🟨 JavaScript/TypeScript</strong></summary>

```javascript
import { DataFoldClient } from '@datafold/sdk';

// Register public key with DataFold server
const registrationResponse = await fetch('https://api.datafold.com/api/crypto/keys/register', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    client_id: 'my-app-client-123',
    public_key: process.env.DATAFOLD_PUBLIC_KEY,
    key_name: 'Production Key',
    metadata: {
      environment: 'production',
      version: '1.0.0'
    }
  })
});

const registration = await registrationResponse.json();
console.log('Registration successful:', registration.data);

// Save client ID for future use
process.env.DATAFOLD_CLIENT_ID = registration.data.client_id;
```

</details>

<details>
<summary><strong>🐍 Python</strong></summary>

```python
import requests
import os

# Register public key with DataFold server
registration_data = {
    'client_id': 'my-app-client-123',
    'public_key': os.environ['DATAFOLD_PUBLIC_KEY'],
    'key_name': 'Production Key',
    'metadata': {
        'environment': 'production',
        'version': '1.0.0'
    }
}

response = requests.post(
    'https://api.datafold.com/api/crypto/keys/register',
    json=registration_data
)

registration = response.json()
print('Registration successful:', registration['data'])

# Save client ID for future use
os.environ['DATAFOLD_CLIENT_ID'] = registration['data']['client_id']
```

</details>

<details>
<summary><strong>⚡ CLI Tool</strong></summary>

```bash
# Register public key with DataFold server
datafold auth register \
  --server-url https://api.datafold.com \
  --client-id my-app-client-123 \
  --key-name "Production Key" \
  --metadata environment=production,version=1.0.0

# This automatically saves the configuration
echo "Registration complete! Configuration saved to ~/.datafold/config"
```

</details>

### Step 4: Make Your First Authenticated Request

<details>
<summary><strong>🟨 JavaScript/TypeScript</strong></summary>

```javascript
import { DataFoldClient } from '@datafold/sdk';

// Create authenticated client
const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: process.env.DATAFOLD_CLIENT_ID,
  privateKey: process.env.DATAFOLD_PRIVATE_KEY
});

// Make authenticated API request
try {
  const schemas = await client.get('/api/schemas');
  console.log('Schemas retrieved:', schemas.data);
  
  // Create a new schema
  const newSchema = await client.post('/api/schemas', {
    name: 'user_events',
    version: '1.0.0',
    fields: [
      { name: 'user_id', type: 'string', required: true },
      { name: 'event_type', type: 'string', required: true },
      { name: 'timestamp', type: 'datetime', required: true }
    ]
  });
  
  console.log('Schema created:', newSchema.data);
} catch (error) {
  console.error('API request failed:', error.message);
}
```

</details>

<details>
<summary><strong>🐍 Python</strong></summary>

```python
from datafold_sdk import DataFoldClient
import os

# Create authenticated client
client = DataFoldClient(
    server_url='https://api.datafold.com',
    client_id=os.environ['DATAFOLD_CLIENT_ID'],
    private_key=bytes.fromhex(os.environ['DATAFOLD_PRIVATE_KEY'])
)

# Make authenticated API request
try:
    schemas = client.get('/api/schemas')
    print('Schemas retrieved:', schemas.json())
    
    # Create a new schema
    new_schema = client.post('/api/schemas', json={
        'name': 'user_events',
        'version': '1.0.0',
        'fields': [
            {'name': 'user_id', 'type': 'string', 'required': True},
            {'name': 'event_type', 'type': 'string', 'required': True},
            {'name': 'timestamp', 'type': 'datetime', 'required': True}
        ]
    })
    
    print('Schema created:', new_schema.json())
except Exception as error:
    print(f'API request failed: {error}')
```

</details>

<details>
<summary><strong>⚡ CLI Tool</strong></summary>

```bash
# Make authenticated API requests
datafold schemas list

# Create a new schema
datafold schemas create \
  --name user_events \
  --version 1.0.0 \
  --field user_id:string:required \
  --field event_type:string:required \
  --field timestamp:datetime:required

echo "Schema created successfully!"
```

</details>

## 🔧 Complete Integration Example

Here's a complete example showing authentication integration in a real application:

### Web Application (React + TypeScript)

```typescript
// src/lib/datafold.ts
import { DataFoldClient } from '@datafold/sdk';

class DataFoldService {
  private client: DataFoldClient;

  constructor() {
    this.client = new DataFoldClient({
      serverUrl: process.env.REACT_APP_DATAFOLD_SERVER_URL!,
      clientId: process.env.REACT_APP_DATAFOLD_CLIENT_ID!,
      privateKey: process.env.REACT_APP_DATAFOLD_PRIVATE_KEY!,
      // Optional: configure security profile
      securityProfile: 'standard'
    });
  }

  // Validate data against schema
  async validateData(schemaName: string, data: any) {
    try {
      const result = await this.client.post(`/api/schemas/${schemaName}/validate`, {
        data: data,
        options: {
          strict: true,
          return_errors: true
        }
      });

      return {
        valid: result.data.valid,
        errors: result.data.errors || []
      };
    } catch (error) {
      console.error('Data validation failed:', error);
      throw new Error(`Validation failed: ${error.message}`);
    }
  }

  // Get schema information
  async getSchema(schemaName: string) {
    const response = await this.client.get(`/api/schemas/${schemaName}`);
    return response.data;
  }

  // Upload data with validation
  async uploadData(schemaName: string, records: any[]) {
    const response = await this.client.post(`/api/data/${schemaName}`, {
      records: records,
      validate: true,
      batch_size: 1000
    });
    
    return response.data;
  }
}

export const datafoldService = new DataFoldService();

// src/components/DataUpload.tsx
import React, { useState } from 'react';
import { datafoldService } from '../lib/datafold';

export function DataUpload() {
  const [file, setFile] = useState<File | null>(null);
  const [schema, setSchema] = useState('');
  const [uploading, setUploading] = useState(false);
  const [result, setResult] = useState<any>(null);

  const handleUpload = async () => {
    if (!file || !schema) return;

    setUploading(true);
    try {
      // Parse CSV/JSON file
      const text = await file.text();
      const data = JSON.parse(text); // Simplified - use proper CSV parser

      // Validate and upload
      const uploadResult = await datafoldService.uploadData(schema, data);
      setResult(uploadResult);
    } catch (error) {
      console.error('Upload failed:', error);
      setResult({ error: error.message });
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="data-upload">
      <h2>Upload Data to DataFold</h2>
      
      <div>
        <label>Schema:</label>
        <input 
          type="text" 
          value={schema} 
          onChange={(e) => setSchema(e.target.value)}
          placeholder="user_events"
        />
      </div>

      <div>
        <label>Data File:</label>
        <input 
          type="file" 
          accept=".json,.csv"
          onChange={(e) => setFile(e.target.files?.[0] || null)}
        />
      </div>

      <button 
        onClick={handleUpload} 
        disabled={!file || !schema || uploading}
      >
        {uploading ? 'Uploading...' : 'Upload Data'}
      </button>

      {result && (
        <div className="result">
          {result.error ? (
            <div className="error">Error: {result.error}</div>
          ) : (
            <div className="success">
              Uploaded {result.processed_records} records successfully!
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

### Backend Service (Python + FastAPI)

```python
# app/datafold_client.py
from datafold_sdk import DataFoldClient
from typing import Dict, List, Any
import os
import logging

logger = logging.getLogger(__name__)

class DataFoldService:
    def __init__(self):
        self.client = DataFoldClient(
            server_url=os.environ['DATAFOLD_SERVER_URL'],
            client_id=os.environ['DATAFOLD_CLIENT_ID'],
            private_key=bytes.fromhex(os.environ['DATAFOLD_PRIVATE_KEY']),
            # Configure for backend service
            security_profile='strict',
            timeout=30
        )

    async def validate_user_data(self, user_id: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate user data against registered schema"""
        try:
            # Add user context
            enriched_data = {
                **data,
                'user_id': user_id,
                'validated_at': datetime.utcnow().isoformat(),
                'source': 'api'
            }

            result = await self.client.post('/api/schemas/user_events/validate', {
                'data': enriched_data,
                'options': {
                    'strict': True,
                    'return_errors': True,
                    'return_warnings': True
                }
            })

            return {
                'valid': result['data']['valid'],
                'errors': result['data'].get('errors', []),
                'warnings': result['data'].get('warnings', [])
            }

        except Exception as e:
            logger.error(f"Data validation failed for user {user_id}: {e}")
            raise ValueError(f"Validation failed: {e}")

    async def store_validated_data(self, schema_name: str, records: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Store validated data in DataFold"""
        try:
            result = await self.client.post(f'/api/data/{schema_name}', {
                'records': records,
                'options': {
                    'validate': True,
                    'batch_size': 1000,
                    'return_summary': True
                }
            })

            return result['data']

        except Exception as e:
            logger.error(f"Data storage failed: {e}")
            raise ValueError(f"Storage failed: {e}")

datafold_service = DataFoldService()

# app/main.py
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel
from typing import Dict, Any, List
from .datafold_client import datafold_service

app = FastAPI(title="DataFold Integration API")

class UserEvent(BaseModel):
    event_type: str
    event_data: Dict[str, Any]
    metadata: Dict[str, Any] = {}

class ValidationResponse(BaseModel):
    valid: bool
    errors: List[str] = []
    warnings: List[str] = []

@app.post("/users/{user_id}/events/validate", response_model=ValidationResponse)
async def validate_user_event(user_id: str, event: UserEvent):
    """Validate user event data against schema"""
    try:
        # Prepare data for validation
        event_data = {
            'user_id': user_id,
            'event_type': event.event_type,
            'event_data': event.event_data,
            'metadata': event.metadata
        }

        # Validate with DataFold
        result = await datafold_service.validate_user_data(user_id, event_data)
        
        return ValidationResponse(**result)

    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail="Internal validation error")

@app.post("/users/{user_id}/events")
async def store_user_event(user_id: str, event: UserEvent):
    """Validate and store user event"""
    try:
        # First validate the data
        validation = await validate_user_event(user_id, event)
        
        if not validation.valid:
            raise HTTPException(
                status_code=422, 
                detail={
                    "message": "Data validation failed",
                    "errors": validation.errors
                }
            )

        # Store the validated data
        event_data = {
            'user_id': user_id,
            'event_type': event.event_type,
            'event_data': event.event_data,
            'metadata': event.metadata
        }

        result = await datafold_service.store_validated_data('user_events', [event_data])
        
        return {
            "status": "success",
            "message": "Event stored successfully",
            "summary": result
        }

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail="Internal storage error")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### CI/CD Pipeline Integration

```yaml
# .github/workflows/datafold-validation.yml
name: DataFold Schema Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  validate-schemas:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup DataFold CLI
      run: |
        curl -sSL https://install.datafold.com | sh
        echo "$HOME/.datafold/bin" >> $GITHUB_PATH
    
    - name: Configure DataFold Authentication
      run: |
        datafold auth configure \
          --server-url ${{ secrets.DATAFOLD_SERVER_URL }} \
          --client-id ${{ secrets.DATAFOLD_CLIENT_ID }} \
          --private-key "${{ secrets.DATAFOLD_PRIVATE_KEY }}"
    
    - name: Validate Schema Changes
      run: |
        # Validate all schema files
        for schema in schemas/*.json; do
          echo "Validating $schema..."
          datafold schemas validate --file "$schema"
        done
    
    - name: Test Data Compatibility
      run: |
        # Test sample data against schemas
        for test_data in test-data/*.json; do
          schema_name=$(basename "$test_data" .json)
          echo "Testing $test_data against $schema_name..."
          datafold data validate \
            --schema "$schema_name" \
            --file "$test_data" \
            --strict
        done
    
    - name: Generate Validation Report
      run: |
        datafold schemas report \
          --format markdown \
          --output schema-validation-report.md
    
    - name: Upload Report
      uses: actions/upload-artifact@v3
      with:
        name: schema-validation-report
        path: schema-validation-report.md
```

## 🔧 Environment Configuration

### Development Environment

```bash
# .env.development
DATAFOLD_SERVER_URL=http://localhost:9001
DATAFOLD_CLIENT_ID=dev-client-123
DATAFOLD_PRIVATE_KEY=your-development-private-key
DATAFOLD_SECURITY_PROFILE=lenient
DATAFOLD_DEBUG=true
```

### Production Environment

```bash
# .env.production
DATAFOLD_SERVER_URL=https://api.datafold.com
DATAFOLD_CLIENT_ID=prod-client-456
DATAFOLD_PRIVATE_KEY=your-production-private-key
DATAFOLD_SECURITY_PROFILE=strict
DATAFOLD_DEBUG=false
DATAFOLD_TIMEOUT=10
DATAFOLD_RETRY_ATTEMPTS=3
```

## ✅ Verification Checklist

Before going to production, verify:

- [ ] **Keys Generated**: Ed25519 keypair created securely
- [ ] **Public Key Registered**: Server accepts your public key
- [ ] **Authentication Works**: Signed requests succeed
- [ ] **Error Handling**: Failed requests handled gracefully
- [ ] **Security Profile**: Appropriate profile for environment
- [ ] **Monitoring**: Logging and metrics configured
- [ ] **Backup**: Keys backed up securely
- [ ] **Documentation**: Team understands implementation

## 🚨 Common Issues

### Authentication Failures

```bash
# Check registration status
curl -H "Content-Type: application/json" \
  https://api.datafold.com/api/crypto/keys/status/your-client-id

# Verify signature generation
datafold auth test-signature --debug
```

### Time Synchronization Issues

```bash
# Check server time
curl -I https://api.datafold.com/api/status

# Sync local time (Linux/macOS)
sudo ntpdate -s time.nist.gov
```

### Network Connectivity

```bash
# Test basic connectivity
curl -v https://api.datafold.com/api/status

# Test with authentication
datafold api get /api/schemas --debug
```

## 🔗 Next Steps

Now that you have authentication working:

1. **[Explore SDK Features](../sdks/javascript/api-reference.md)** - Learn advanced SDK capabilities
2. **[Security Best Practices](../guides/security-best-practices.md)** - Secure your implementation
3. **[Performance Optimization](../guides/performance-optimization.md)** - Optimize for production
4. **[Error Handling](troubleshooting.md)** - Handle edge cases gracefully

## 📞 Need Help?

- **Documentation Issues**: Check the [troubleshooting guide](troubleshooting.md)
- **Integration Questions**: [Community Forum](https://community.datafold.com)
- **Enterprise Support**: [Contact Sales](mailto:sales@datafold.com)
- **Security Issues**: [Report Security Issues](mailto:security@datafold.com)

---

**Congratulations!** 🎉 You now have DataFold signature authentication working. Your API requests are cryptographically secured and ready for production use.