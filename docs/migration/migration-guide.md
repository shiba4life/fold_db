# DataFold Signature Authentication Migration Guide

**Version:** 1.0  
**Date:** June 9, 2025  
**Task ID:** T11.6-3  

## Executive Summary

This comprehensive migration guide enables existing DataFold systems and external integrations to successfully adopt DataFold's signature authentication system. The guide provides step-by-step procedures, migration strategies, and practical examples to ensure smooth transitions with minimal disruption.

**Key Benefits of Migration:**
- **Enhanced Security**: RFC 9421 HTTP Message Signatures with Ed25519 cryptography
- **Zero-Trust Authentication**: Cryptographic verification for every request
- **Improved Performance**: Stateless authentication with no server-side sessions
- **Standards Compliance**: Industry-standard authentication protocols
- **Future-Proof**: Scalable, maintainable authentication infrastructure

## Table of Contents

1. [Migration Overview](#1-migration-overview)
2. [Pre-Migration Assessment](#2-pre-migration-assessment)
3. [Migration Planning](#3-migration-planning)
4. [Technical Migration Procedures](#4-technical-migration-procedures)
5. [Client-Specific Migration Guides](#5-client-specific-migration-guides)
6. [Deployment Strategies](#6-deployment-strategies)
7. [Testing and Validation](#7-testing-and-validation)
8. [Common Migration Scenarios](#8-common-migration-scenarios)
9. [Monitoring and Operations](#9-monitoring-and-operations)
10. [Troubleshooting](#10-troubleshooting)
11. [Post-Migration Operations](#11-post-migration-operations)

---

## 1. Migration Overview

### 1.1 What is DataFold Signature Authentication?

DataFold's signature authentication implements **RFC 9421 HTTP Message Signatures** with **Ed25519** digital signatures, providing:

- **Cryptographic Request Authentication**: Every API request is digitally signed
- **Message Integrity**: Tamper-proof request verification
- **Replay Protection**: Built-in timestamp and nonce validation
- **Stateless Operation**: No server-side session management required
- **Cross-Platform Consistency**: Identical authentication across all DataFold SDKs

### 1.2 Migration Scenarios

This guide covers migration from:

| Current Authentication | Target Authentication | Migration Type |
|----------------------|---------------------|---------------|
| **No Authentication** | Signature Authentication | **Green Field** |
| **API Tokens/Keys** | Signature Authentication | **Token Replacement** |
| **Basic Authentication** | Signature Authentication | **Credential Migration** |
| **Custom Authentication** | Signature Authentication | **Integration Migration** |
| **Mixed Authentication** | Signature Authentication | **Hybrid Transition** |

### 1.3 Migration Benefits

- **Security Enhancement**: Military-grade Ed25519 cryptography
- **Performance Improvement**: Eliminate authentication bottlenecks
- **Operational Simplicity**: No session management or token rotation
- **Developer Experience**: Seamless SDK integration
- **Compliance**: Meet modern security standards

### 1.4 Prerequisites

Before starting migration:

- [ ] **DataFold Server**: Version supporting signature authentication
- [ ] **Network Access**: Connectivity to DataFold APIs
- [ ] **Development Tools**: SDK for your platform (JavaScript, Python, CLI)
- [ ] **Cryptographic Understanding**: Basic knowledge of public/private keys
- [ ] **System Access**: Ability to modify client applications and server configuration

---

## 2. Pre-Migration Assessment

### 2.1 System Inventory Checklist

Complete this assessment before beginning migration:

#### 2.1.1 Current Authentication Audit

- [ ] **Authentication Methods**: Document all current authentication mechanisms
- [ ] **API Usage Patterns**: Identify all DataFold API consumers
- [ ] **Client Applications**: Catalog all applications using DataFold APIs
- [ ] **Integration Points**: Map third-party integrations and webhooks
- [ ] **Service Accounts**: Identify automated systems and service accounts
- [ ] **User Accounts**: Document interactive user authentication patterns

#### 2.1.2 Infrastructure Assessment

- [ ] **Server Versions**: Verify DataFold server supports signature authentication
- [ ] **Network Configuration**: Check firewall and proxy settings
- [ ] **Load Balancers**: Ensure signature headers are preserved
- [ ] **API Gateways**: Verify compatibility with RFC 9421 signatures
- [ ] **Monitoring Systems**: Plan for new authentication metrics
- [ ] **Logging Infrastructure**: Prepare for enhanced security logging

#### 2.1.3 Application Assessment

- [ ] **SDK Versions**: Update to signature-capable SDK versions
- [ ] **Code Dependencies**: Review and update cryptographic dependencies
- [ ] **Configuration Management**: Plan for key management integration
- [ ] **Deployment Pipelines**: Update CI/CD for signature authentication
- [ ] **Testing Frameworks**: Adapt tests for signature verification

### 2.2 Risk Assessment Framework

#### 2.2.1 Technical Risks

| Risk Category | Risk Level | Mitigation Strategy |
|--------------|------------|-------------------|
| **Authentication Failures** | High | Implement gradual rollout with fallback |
| **Performance Impact** | Medium | Conduct performance testing and optimization |
| **Integration Breakage** | High | Comprehensive testing and rollback procedures |
| **Key Management Issues** | Medium | Establish secure key storage and rotation |
| **Network Compatibility** | Low | Verify proxy and gateway configuration |

#### 2.2.2 Operational Risks

| Risk Category | Risk Level | Mitigation Strategy |
|--------------|------------|-------------------|
| **Service Downtime** | High | Use blue-green or canary deployment |
| **Data Loss** | Low | No data migration required for auth changes |
| **Compliance Issues** | Medium | Validate against security requirements |
| **Team Training** | Medium | Provide comprehensive training and documentation |
| **Support Overhead** | Medium | Prepare troubleshooting procedures |

### 2.3 Resource Planning

#### 2.3.1 Personnel Requirements

- **Technical Lead**: 1 person, 2-4 weeks
- **Backend Developers**: 1-2 people, 1-3 weeks  
- **Frontend Developers**: 1-2 people, 1-2 weeks
- **DevOps Engineers**: 1 person, 1-2 weeks
- **QA Engineers**: 1-2 people, 1-2 weeks

#### 2.3.2 Timeline Estimation

| Migration Scope | Estimated Duration | Complexity Level |
|----------------|-------------------|-----------------|
| **Single Application** | 1-2 weeks | Low |
| **Multiple Applications** | 2-4 weeks | Medium |
| **Enterprise Deployment** | 4-8 weeks | High |
| **Complex Integrations** | 6-12 weeks | Very High |

---

## 3. Migration Planning

### 3.1 Migration Strategy Selection

Choose the appropriate migration strategy based on your requirements:

#### 3.1.1 Big Bang Migration

**Best for:** Small systems, development environments

**Characteristics:**
- Complete migration in single deployment
- All authentication switches simultaneously
- Shortest migration window
- Highest risk if issues occur

**Timeline:** 1-2 weeks

#### 3.1.2 Gradual Migration

**Best for:** Production systems, risk-averse organizations

**Characteristics:**
- Phased migration over multiple deployments
- Hybrid authentication during transition
- Lower risk with incremental validation
- Longer migration window

**Timeline:** 3-6 weeks

#### 3.1.3 Canary Migration

**Best for:** Large-scale deployments, high-availability requirements

**Characteristics:**
- Percentage-based traffic migration
- Real-time monitoring and validation
- Immediate rollback capability
- Minimal user impact

**Timeline:** 4-8 weeks

### 3.2 Migration Phases

#### Phase 1: Preparation (Week 1)

**Objectives:**
- Complete system assessment
- Establish migration plan
- Set up development environment
- Train team members

**Deliverables:**
- [ ] Migration plan document
- [ ] Risk assessment and mitigation plan
- [ ] Development environment with signature authentication
- [ ] Team training completion

#### Phase 2: Development Migration (Week 2-3)

**Objectives:**
- Migrate development environment
- Update client applications
- Implement testing procedures
- Validate integration

**Deliverables:**
- [ ] Development server migrated
- [ ] Client applications updated
- [ ] Automated tests implemented
- [ ] Integration validation passed

#### Phase 3: Staging Validation (Week 4-5)

**Objectives:**
- Deploy to staging environment
- Conduct comprehensive testing
- Performance validation
- Security verification

**Deliverables:**
- [ ] Staging environment migrated
- [ ] End-to-end testing completed
- [ ] Performance benchmarks met
- [ ] Security audit passed

#### Phase 4: Production Migration (Week 6-7)

**Objectives:**
- Execute production deployment
- Monitor system health
- Validate authentication functionality
- Complete legacy cleanup

**Deliverables:**
- [ ] Production migration completed
- [ ] Monitoring and alerting active
- [ ] Authentication validation passed
- [ ] Legacy systems decommissioned

### 3.3 Rollback Planning

#### 3.3.1 Rollback Triggers

Initiate rollback if:
- Authentication failure rate > 5%
- API response time increases > 50%
- Critical business functions fail
- Security incidents detected
- Performance degradation beyond acceptable limits

#### 3.3.2 Rollback Procedures

1. **Immediate Actions**
   - Switch server to hybrid authentication mode
   - Route traffic to legacy authentication
   - Alert stakeholders and support teams

2. **Investigation**
   - Collect logs and metrics
   - Identify root cause
   - Document issues for resolution

3. **Recovery Planning**
   - Develop fix strategy
   - Plan re-migration approach
   - Update procedures based on lessons learned

---

## 4. Technical Migration Procedures

### 4.1 Server Configuration Migration

#### 4.1.1 DataFold Server Setup

**Step 1: Update Server Configuration**

```rust
// Example: Enable signature authentication
use datafold::datafold_node::{NodeConfig, SignatureAuthConfig};

let config = NodeConfig::production_with_signature_auth(storage_path)
    .with_signature_auth_config(SignatureAuthConfig {
        enabled: true,
        required_for_endpoints: vec!["/api/*".to_string()],
        optional_for_endpoints: vec!["/health".to_string()],
        timestamp_tolerance_seconds: 300,
        nonce_cache_size: 10000,
        nonce_cleanup_interval_seconds: 3600,
    });
```

**Step 2: Configure Hybrid Authentication (Optional)**

For gradual migration, enable hybrid authentication:

```rust
let config = NodeConfig::with_optional_signature_auth(storage_path)
    .with_legacy_auth_fallback(true)
    .with_migration_mode(MigrationMode::Gradual);
```

**Step 3: Verify Server Configuration**

```bash
# Test server configuration
curl -X GET "http://localhost:8080/api/system/auth-status" \
     -H "Content-Type: application/json"
```

#### 4.1.2 Database and Storage Updates

**Key Storage Configuration:**

```rust
// Configure public key storage
let key_storage = KeyStorage::new()
    .with_storage_backend(StorageBackend::Database)
    .with_encryption_at_rest(true)
    .with_backup_enabled(true);
```

**Database Schema Updates:**

No database schema changes are required. Public keys are stored in the existing metadata system.

#### 4.1.3 Network Configuration

**Load Balancer Configuration:**

Ensure signature headers are preserved:

```nginx
# Nginx configuration
location /api/ {
    proxy_pass http://datafold-backend;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    
    # Preserve signature headers
    proxy_set_header Signature $http_signature;
    proxy_set_header Signature-Input $http_signature_input;
}
```

### 4.2 Key Generation and Registration

#### 4.2.1 Generate Ed25519 Keypairs

**JavaScript/TypeScript:**
```javascript
import { DataFoldClient } from '@datafold/sdk';

// Generate keypair
const keyPair = await DataFoldClient.generateKeyPair();
console.log('Private Key:', keyPair.privateKey);
console.log('Public Key:', keyPair.publicKey);
```

**Python:**
```python
from datafold_sdk import DataFoldClient

# Generate keypair
key_pair = DataFoldClient.generate_key_pair()
print(f"Private Key: {key_pair.private_key}")
print(f"Public Key: {key_pair.public_key}")
```

**CLI:**
```bash
# Generate keypair
datafold auth-keygen --key-id production-client --output-format json

# Output private key securely
datafold auth-keygen --key-id prod-client --private-key-file ~/.datafold/keys/private.key
```

#### 4.2.2 Register Public Keys

**JavaScript/TypeScript:**
```javascript
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com',
    clientId: 'my-application'
});

// Register public key
await client.registerPublicKey({
    keyId: 'my-application-v1',
    publicKey: keyPair.publicKey,
    metadata: {
        application: 'web-frontend',
        environment: 'production',
        version: '1.0.0'
    }
});
```

**Python:**
```python
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    base_url='https://api.datafold.com',
    client_id='my-service'
)

# Register public key
client.register_public_key(
    key_id='my-service-v1',
    public_key=key_pair.public_key,
    metadata={
        'service': 'data-processor',
        'environment': 'production',
        'version': '2.1.0'
    }
)
```

**CLI:**
```bash
# Register public key
datafold crypto register-key \
    --key-id my-cli-app \
    --public-key ~/.datafold/keys/public.key \
    --metadata '{"app":"cli","env":"prod"}'
```

---

## 5. Client-Specific Migration Guides

### 5.1 JavaScript Application Migration

#### 5.1.1 Update Dependencies

```bash
# Update to signature-enabled SDK
npm update @datafold/sdk

# Verify version supports signatures
npm list @datafold/sdk
```

#### 5.1.2 Code Migration Examples

**Before: Unauthenticated Client**
```javascript
// Legacy unauthenticated usage
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com'
});

// Make API call
const schemas = await client.getSchemas();
```

**After: Signature Authentication**
```javascript
// Modern signature authentication
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com',
    clientId: 'web-frontend-v1',
    
    // Signature authentication
    authentication: {
        type: 'signature',
        privateKey: process.env.DATAFOLD_PRIVATE_KEY,
        keyId: 'web-frontend-v1'
    }
});

// Make authenticated API call
const schemas = await client.getSchemas();
```

#### 5.1.3 React Application Example

**Complete React Integration:**
```typescript
// src/services/datafold.ts
import { DataFoldClient } from '@datafold/sdk';

class DataFoldService {
    private client: DataFoldClient;
    
    constructor() {
        this.client = new DataFoldClient({
            baseUrl: process.env.REACT_APP_DATAFOLD_URL!,
            clientId: 'react-app',
            authentication: {
                type: 'signature',
                privateKey: process.env.REACT_APP_DATAFOLD_PRIVATE_KEY!,
                keyId: process.env.REACT_APP_DATAFOLD_KEY_ID!
            }
        });
    }
    
    async getSchemas() {
        try {
            return await this.client.getSchemas();
        } catch (error) {
            console.error('Authentication failed:', error);
            throw error;
        }
    }
}

export default new DataFoldService();
```

**React Component Usage:**
```typescript
// src/components/SchemaList.tsx
import React, { useEffect, useState } from 'react';
import dataFoldService from '../services/datafold';

export const SchemaList: React.FC = () => {
    const [schemas, setSchemas] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    
    useEffect(() => {
        const loadSchemas = async () => {
            try {
                const data = await dataFoldService.getSchemas();
                setSchemas(data);
            } catch (err) {
                setError('Failed to load schemas. Check authentication.');
            } finally {
                setLoading(false);
            }
        };
        
        loadSchemas();
    }, []);
    
    if (loading) return <div>Loading...</div>;
    if (error) return <div>Error: {error}</div>;
    
    return (
        <div>
            <h2>DataFold Schemas</h2>
            {schemas.map(schema => (
                <div key={schema.id}>{schema.name}</div>
            ))}
        </div>
    );
};
```

### 5.2 Python Application Migration

#### 5.2.1 Update Dependencies

```bash
# Update to signature-enabled SDK
pip install --upgrade datafold-sdk

# Verify version
python -c "import datafold_sdk; print(datafold_sdk.__version__)"
```

#### 5.2.2 Code Migration Examples

**Before: Token-Based Authentication**
```python
# Legacy token authentication
import os
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    base_url='https://api.datafold.com',
    api_token=os.getenv('DATAFOLD_API_TOKEN')
)

# Make API call
schemas = client.get_schemas()
```

**After: Signature Authentication**
```python
# Modern signature authentication
import os
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    base_url='https://api.datafold.com',
    client_id='python-service',
    authentication={
        'type': 'signature',
        'private_key': os.getenv('DATAFOLD_PRIVATE_KEY'),
        'key_id': 'python-service-v1'
    }
)

# Make authenticated API call
schemas = client.get_schemas()
```

#### 5.2.3 Django Integration Example

**Django Settings Configuration:**
```python
# settings.py
import os

DATAFOLD_CONFIG = {
    'BASE_URL': os.getenv('DATAFOLD_URL', 'https://api.datafold.com'),
    'CLIENT_ID': 'django-app',
    'AUTHENTICATION': {
        'type': 'signature',
        'private_key': os.getenv('DATAFOLD_PRIVATE_KEY'),
        'key_id': os.getenv('DATAFOLD_KEY_ID', 'django-app-v1')
    }
}
```

**Django Service Class:**
```python
# services/datafold_service.py
from django.conf import settings
from datafold_sdk import DataFoldClient
import logging

logger = logging.getLogger(__name__)

class DataFoldService:
    def __init__(self):
        self.client = DataFoldClient(
            base_url=settings.DATAFOLD_CONFIG['BASE_URL'],
            client_id=settings.DATAFOLD_CONFIG['CLIENT_ID'],
            authentication=settings.DATAFOLD_CONFIG['AUTHENTICATION']
        )
    
    def get_schemas(self):
        try:
            return self.client.get_schemas()
        except Exception as e:
            logger.error(f"DataFold authentication failed: {e}")
            raise
    
    def create_schema(self, schema_data):
        try:
            return self.client.create_schema(schema_data)
        except Exception as e:
            logger.error(f"DataFold schema creation failed: {e}")
            raise

# Singleton instance
datafold_service = DataFoldService()
```

### 5.3 CLI Tool Migration

#### 5.3.1 Update CLI Tool

```bash
# Update DataFold CLI
curl -sSL https://install.datafold.com | sh

# Verify version supports signatures
datafold --version
```

#### 5.3.2 Authentication Setup

**Initialize Authentication:**
```bash
# Generate keypair for CLI
datafold auth-keygen --key-id cli-user-$(whoami)

# Register public key
datafold crypto register-key \
    --key-id cli-user-$(whoami) \
    --public-key ~/.datafold/keys/public.key

# Create authentication profile
datafold auth-init \
    --key-id cli-user-$(whoami) \
    --server-url https://api.datafold.com
```

**Test Authentication:**
```bash
# Test authentication setup
datafold auth-test --endpoint /api/schemas

# Check authentication status
datafold auth-status --verbose
```

#### 5.3.3 Automated Script Migration

**Before: Unauthenticated Scripts**
```bash
#!/bin/bash
# Legacy script without authentication

# Get schemas
datafold api get /api/schemas > schemas.json

# Process data
jq '.data[]' schemas.json
```

**After: Authenticated Scripts**
```bash
#!/bin/bash
# Modern script with signature authentication

# Verify authentication is configured
if ! datafold auth-status --quiet; then
    echo "Error: DataFold authentication not configured"
    echo "Run: datafold auth-init --interactive"
    exit 1
fi

# Get schemas with automatic authentication
datafold api get /api/schemas > schemas.json

# Process data
jq '.data[]' schemas.json
```

### 5.4 Legacy Client Transition

#### 5.4.1 Custom Integration Migration

**Legacy HTTP Client:**
```python
# Legacy custom HTTP client
import requests
import os

def make_request(method, endpoint, data=None):
    url = f"https://api.datafold.com{endpoint}"
    headers = {
        'Authorization': f"Bearer {os.getenv('API_TOKEN')}",
        'Content-Type': 'application/json'
    }
    
    response = requests.request(method, url, headers=headers, json=data)
    return response.json()

# Usage
schemas = make_request('GET', '/api/schemas')
```

**Migrated HTTP Client:**
```python
# Migrated to signature authentication
import requests
import os
from datafold_sdk.signing import RFC9421Signer

def make_signed_request(method, endpoint, data=None):
    url = f"https://api.datafold.com{endpoint}"
    
    # Initialize signer
    signer = RFC9421Signer(
        client_id='custom-client',
        private_key=os.getenv('DATAFOLD_PRIVATE_KEY'),
        key_id='custom-client-v1'
    )
    
    # Sign request
    headers = signer.sign_request(method, url, data)
    headers['Content-Type'] = 'application/json'
    
    response = requests.request(method, url, headers=headers, json=data)
    return response.json()

# Usage (same interface)
schemas = make_signed_request('GET', '/api/schemas')
```

---

## 6. Deployment Strategies

### 6.1 Blue-Green Deployment

**Benefits:**
- Zero-downtime migration
- Instant rollback capability
- Complete environment isolation
- Full validation before traffic switch

**Implementation:**

```yaml
# docker-compose.yml for blue-green deployment
version: '3.8'

services:
  # Blue environment (current production)
  datafold-blue:
    image: datafold:current
    environment:
      - SIGNATURE_AUTH_ENABLED=false
    ports:
      - "8080:8080"
    
  # Green environment (with signature auth)
  datafold-green:
    image: datafold:latest
    environment:
      - SIGNATURE_AUTH_ENABLED=true
      - SIGNATURE_AUTH_REQUIRED=true
    ports:
      - "8081:8080"
    
  # Load balancer
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
```

**Deployment Process:**

1. **Deploy Green Environment**
   ```bash
   # Deploy new version with signature auth
   docker-compose up -d datafold-green
   
   # Verify green environment health
   curl http://localhost:8081/health
   ```

2. **Validate Green Environment**
   ```bash
   # Test signature authentication
   datafold auth-test --server-url http://localhost:8081
   
   # Run integration tests
   npm run test:integration -- --server=http://localhost:8081
   ```

3. **Switch Traffic**
   ```bash
   # Update load balancer configuration
   # Point traffic from blue (8080) to green (8081)
   
   # Verify production traffic
   curl http://localhost/api/schemas
   ```

4. **Cleanup**
   ```bash
   # Stop blue environment after validation
   docker-compose stop datafold-blue
   ```

### 6.2 Canary Deployment

**Benefits:**
- Gradual traffic migration
- Real-time monitoring
- Minimal user impact
- Data-driven rollout decisions

**Implementation:**

```python
# canary-controller.py
import time
import requests
from typing import Dict, List

class CanaryController:
    def __init__(self, old_endpoint: str, new_endpoint: str):
        self.old_endpoint = old_endpoint
        self.new_endpoint = new_endpoint
        self.canary_percentage = 0
        self.success_threshold = 95.0  # 95% success rate required
        
    def route_request(self, request_data: Dict) -> Dict:
        """Route request based on canary percentage"""
        import random
        
        # Determine which endpoint to use
        if random.randint(1, 100) <= self.canary_percentage:
            endpoint = self.new_endpoint  # Signature auth
            auth_type = 'signature'
        else:
            endpoint = self.old_endpoint   # Legacy auth
            auth_type = 'legacy'
            
        try:
            response = self.make_request(endpoint, request_data, auth_type)
            self.record_success(auth_type)
            return response
        except Exception as e:
            self.record_failure(auth_type)
            # Fallback to old endpoint if new fails
            if auth_type == 'signature':
                return self.make_request(self.old_endpoint, request_data, 'legacy')
            raise e
    
    def increase_canary_traffic(self, increment: int = 10):
        """Gradually increase canary traffic"""
        success_rate = self.get_success_rate('signature')
        
        if success_rate >= self.success_threshold:
            self.canary_percentage = min(100, self.canary_percentage + increment)
            print(f"Increased canary traffic to {self.canary_percentage}%")
        else:
            print(f"Success rate {success_rate}% below threshold, not increasing traffic")
    
    def rollback(self):
        """Emergency rollback to 0% canary traffic"""
        self.canary_percentage = 0
        print("Emergency rollback executed")
```

**Canary Deployment Schedule:**

| Week | Canary % | Validation Criteria | Action |
|------|----------|-------------------|---------|
| 1 | 5% | Error rate < 1% | Monitor closely |
| 2 | 10% | Performance within 10% | Increase traffic |
| 3 | 25% | No security incidents | Continue rollout |
| 4 | 50% | User feedback positive | Accelerate |
| 5 | 75% | All metrics green | Near completion |
| 6 | 100% | Full migration | Complete rollout |

### 6.3 Feature Flag Approach

**Benefits:**
- Runtime configuration switching
- A/B testing capability
- Gradual user migration
- Emergency disable functionality

**Implementation:**

```python
# feature_flags.py
import os
from enum import Enum

class AuthMode(Enum):
    LEGACY = "legacy"
    SIGNATURE = "signature"
    HYBRID = "hybrid"

class FeatureFlags:
    def __init__(self):
        self.auth_mode = AuthMode(os.getenv('AUTH_MODE', 'legacy'))
        self.signature_auth_percentage = int(os.getenv('SIGNATURE_AUTH_PERCENTAGE', '0'))
        self.force_signature_for_users = os.getenv('FORCE_SIGNATURE_USERS', '').split(',')
    
    def should_use_signature_auth(self, user_id: str = None) -> bool:
        """Determine if signature auth should be used"""
        
        # Force signature auth for specific users
        if user_id and user_id in self.force_signature_for_users:
            return True
            
        # Check mode
        if self.auth_mode == AuthMode.SIGNATURE:
            return True
        elif self.auth_mode == AuthMode.LEGACY:
            return False
        elif self.auth_mode == AuthMode.HYBRID:
            # Use percentage-based rollout
            import random
            return random.randint(1, 100) <= self.signature_auth_percentage
            
        return False

# Usage in application
feature_flags = FeatureFlags()

def authenticate_request(request, user_id=None):
    if feature_flags.should_use_signature_auth(user_id):
        return signature_authenticate(request)
    else:
        return legacy_authenticate(request)
```

---

## 7. Testing and Validation

### 7.1 Pre-Migration Testing

#### 7.1.1 Development Environment Validation

**Test Checklist:**

- [ ] **Key Generation**: Verify Ed25519 keypair generation
- [ ] **Key Registration**: Test public key registration with server
- [ ] **Signature Generation**: Validate RFC 9421 signature creation
- [ ] **Signature Verification**: Confirm server signature validation
- [ ] **Error Handling**: Test authentication failure scenarios
- [ ] **Performance**: Measure signature operation latency

**Test Scripts:**

```bash
#!/bin/bash
# pre-migration-tests.sh

echo "=== DataFold Migration Pre-Tests ==="

# Test 1: Key generation
echo "1. Testing key generation..."
datafold auth-keygen --key-id test-migration --test-mode
if [ $? -eq 0 ]; then
    echo "✅ Key generation successful"
else
    echo "❌ Key generation failed"
    exit 1
fi

# Test 2: Public key registration
echo "2. Testing public key registration..."
datafold crypto register-key --key-id test-migration --test-mode
if [ $? -eq 0 ]; then
    echo "✅ Public key registration successful"
else
    echo "❌ Public key registration failed"
    exit 1
fi

# Test 3: Authentication test
echo "3. Testing signature authentication..."
datafold auth-test --key-id test-migration
if [ $? -eq 0 ]; then
    echo "✅ Signature authentication successful"
else
    echo "❌ Signature authentication failed"
    exit 1
fi

# Test 4: API access
echo "4. Testing authenticated API access..."
datafold api get /api/schemas --key-id test-migration > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ Authenticated API access successful"
else
    echo "❌ Authenticated API access failed"
    exit 1
fi

echo "=== All pre-migration tests passed ✅ ==="
```

#### 7.1.2 Integration Testing

**End-to-End Test Suite:**

```python
# tests/test_migration_e2e.py
import pytest
import os
from datafold_sdk import DataFoldClient

class TestMigrationEndToEnd:
    
    def setup_method(self):
        """Setup test environment"""
        self.test_server_url = os.getenv('TEST_DATAFOLD_URL', 'http://localhost:8080')
        self.client_id = 'migration-test-client'
        
    @pytest.mark.asyncio
    async def test_signature_authentication_flow(self):
        """Test complete signature authentication flow"""
        
        # Step 1: Generate keypair
        key_pair = DataFoldClient.generate_key_pair()
        assert key_pair.private_key
        assert key_pair.public_key
        
        # Step 2: Register public key
        client = DataFoldClient(
            base_url=self.test_server_url,
            client_id=self.client_id
        )
        
        registration_result = await client.register_public_key(
            key_id=self.client_id,
            public_key=key_pair.public_key
        )
        assert registration_result['success'] is True
        
        # Step 3: Configure signature authentication
        auth_client = DataFoldClient(
            base_url=self.test_server_url,
            client_id=self.client_id,
            authentication={
                'type': 'signature',
                'private_key': key_pair.private_key,
                'key_id': self.client_id
            }
        )
        
        # Step 4: Test authenticated API calls
        schemas = await auth_client.get_schemas()
        assert isinstance(schemas, list)
        
        # Step 5: Test invalid signature handling
        invalid_client = DataFoldClient(
            base_url=self.test_server_url,
            client_id=self.client_id,
            authentication={
                'type': 'signature',
                'private_key': 'invalid-key',
                'key_id': self.client_id
            }
        )
        
        with pytest.raises(Exception) as exc_info:
            await invalid_client.get_schemas()
        assert 'authentication' in str(exc_info.value).lower()
        
    def test_performance_benchmarks(self):
        """Verify signature authentication performance"""
        import time
        
        # Test signature generation performance
        start_time = time.time()
        for _ in range(100):
            DataFoldClient.generate_key_pair()
        key_gen_time = (time.time() - start_time) / 100
        
        # Should be under 10ms per keypair
        assert key_gen_time < 0.01, f"Key generation too slow: {key_gen_time}s"
        
        print(f"✅ Key generation: {key_gen_time*1000:.2f}ms per operation")
```

### 7.2 Migration Testing

#### 7.2.1 Hybrid Authentication Testing

**Test hybrid mode during migration:**

```python
# tests/test_hybrid_auth.py
import pytest
from datafold_sdk import DataFoldClient

class TestHybridAuthentication:
    
    def test_legacy_auth_fallback(self):
        """Test that legacy authentication still works during migration"""
        
        # Legacy client (token-based)
        legacy_client = DataFoldClient(
            base_url='http://localhost:8080',
            api_token='legacy-token'
        )
        
        # Should still work during hybrid mode
        schemas = legacy_client.get_schemas()
        assert isinstance(schemas, list)
        
    def test_signature_auth_preferred(self):
        """Test that signature authentication works in hybrid mode"""
        
        # Signature client
        sig_client = DataFoldClient(
            base_url='http://localhost:8080',
            client_id='test-client',
            authentication={
                'type': 'signature',
                'private_key': 'test-private-key',
                'key_id': 'test-key'
            }
        )
        
        # Should work with signature auth
        schemas = sig_client.get_schemas()
        assert isinstance(schemas, list)
        
    def test_auth_mode_switching(self):
        """Test runtime switching between auth modes"""
        
        # Test different auth configurations
        auth_configs = [
            {'mode': 'legacy'},
            {'mode': 'hybrid', 'percentage': 50},
            {'mode': 'signature'},
        ]
        
        for config in auth_configs:
            response = self.update_server_auth_config(config)
            assert response.status_code == 200
            
            # Verify auth mode is active
            status = self.check_auth_status()
            assert status['mode'] == config['mode']
```

### 7.3 Post-Migration Validation

#### 7.3.1 Production Validation Checklist

**Immediate Post-Migration (0-1 hours):**

- [ ] **Authentication Success Rate**: > 99%
- [ ] **API Response Times**: Within 10% of baseline
- [ ] **Error Rates**: < 1%
- [ ] **System Health**: All services green
- [ ] **User Access**: Critical functions working
- [ ] **Monitoring**: All metrics collecting

**Short-term Validation (1-24 hours):**

- [ ] **Performance Stability**: No degradation trends
- [ ] **Error Pattern Analysis**: No new error types
- [ ] **User Feedback**: No critical issues reported
- [ ] **Security Events**: No authentication bypass attempts
- [ ] **Load Testing**: System handles normal traffic
- [ ] **Backup Procedures**: Rollback plan validated

**Long-term Validation (1-7 days):**

- [ ] **Performance Optimization**: Identify improvement opportunities
- [ ] **Security Audit**: Complete security validation
- [ ] **User Training**: Ensure team understands new system
- [ ] **Documentation**: Update operational procedures
- [ ] **Monitoring Tuning**: Optimize alerting thresholds
- [ ] **Legacy Cleanup**: Remove deprecated authentication code

---

## 8. Common Migration Scenarios

### 8.1 Scenario 1: Unauthenticated → Signature Authentication

**Use Case:** Moving from completely open APIs to secured APIs

**Migration Strategy:**
1. **Phase 1**: Deploy server with signature auth optional
2. **Phase 2**: Update all clients to use signature auth
3. **Phase 3**: Enable signature auth requirement
4. **Phase 4**: Remove unauthenticated access

**Implementation:**

```typescript
// Phase 1: Server configuration
const serverConfig = {
    signatureAuth: {
        enabled: true,
        required: false,  // Optional during migration
        exemptPaths: ['/health', '/metrics']
    }
};

// Phase 2: Client update
const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com',
    // Add authentication
    authentication: {
        type: 'signature',
        privateKey: process.env.DATAFOLD_PRIVATE_KEY,
        keyId: 'web-app-v1'
    }
});

// Phase 3: Require signature auth
const serverConfig = {
    signatureAuth: {
        enabled: true,
        required: true,   // Now required
        exemptPaths: ['/health']  // Reduced exemptions
    }
};
```

### 8.2 Scenario 2: Token-Based → Signature Authentication

**Use Case:** Replacing API tokens with cryptographic signatures

**Migration Strategy:**
1. **Parallel Implementation**: Support both token and signature auth
2. **Client Migration**: Update clients one by one
3. **Gradual Transition**: Increase signature auth percentage
4. **Token Deprecation**: Remove token support

**Implementation:**

```python
# Hybrid authentication client
class HybridDataFoldClient:
    def __init__(self, base_url, api_token=None, signature_auth=None):
        self.base_url = base_url
        self.api_token = api_token
        self.signature_auth = signature_auth
        
    def make_request(self, method, endpoint, data=None):
        # Try signature auth first, fallback to token
        if self.signature_auth:
            try:
                return self._make_signed_request(method, endpoint, data)
            except AuthenticationError:
                if self.api_token:
                    return self._make_token_request(method, endpoint, data)
                raise
        elif self.api_token:
            return self._make_token_request(method, endpoint, data)
        else:
            raise ValueError("No authentication method configured")

# Migration usage
client = HybridDataFoldClient(
    base_url='https://api.datafold.com',
    api_token='legacy-token',  # Fallback
    signature_auth={           # Preferred
        'private_key': os.getenv('DATAFOLD_PRIVATE_KEY'),
        'key_id': 'python-service-v1',
        'client_id': 'python-service'
    }
)
```

### 8.3 Scenario 3: Multi-Environment Migration

**Use Case:** Coordinated migration across dev/staging/production

**Migration Timeline:**

| Environment | Week 1 | Week 2 | Week 3 | Week 4 |
|-------------|--------|--------|--------|--------|
| **Development** | Migrate | Validate | Optimize | Document |
| **Staging** | - | Migrate | Validate | Optimize |
| **Production** | - | - | Migrate | Validate |

**Environment-Specific Configuration:**

```yaml
# environments/development.yml
datafold:
  signature_auth:
    enabled: true
    required: true
    debug_mode: true
    timestamp_tolerance: 600  # Relaxed for dev

# environments/staging.yml  
datafold:
  signature_auth:
    enabled: true
    required: true
    debug_mode: false
    timestamp_tolerance: 300  # Production-like

# environments/production.yml
datafold:
  signature_auth:
    enabled: true
    required: true
    debug_mode: false
    timestamp_tolerance: 300
    monitoring:
      enabled: true
      alert_threshold: 0.01  # 1% error rate
```

### 8.4 Scenario 4: Third-Party Integration Migration

**Use Case:** Migrating external services and partners

**Challenges:**
- External systems outside your control
- Varied technical capabilities
- Different migration schedules
- Support and communication overhead

**Migration Approach:**

```python
# Partner integration adapter
class PartnerIntegrationAdapter:
    def __init__(self, partner_config):
        self.partner_id = partner_config['partner_id']
        self.auth_capability = partner_config.get('signature_auth_capable', False)
        
        if self.auth_capability:
            self.client = self._create_signature_client(partner_config)
        else:
            self.client = self._create_legacy_client(partner_config)
            # Schedule migration notification
            self._schedule_migration_notice()
    
    def _create_signature_client(self, config):
        return DataFoldClient(
            base_url=config['base_url'],
            client_id=f"partner-{self.partner_id}",
            authentication={
                'type': 'signature',
                'private_key': config['private_key'],
                'key_id': config['key_id']
            }
        )
    
    def _create_legacy_client(self, config):
        return DataFoldClient(
            base_url=config['base_url'],
            api_token=config['api_token']
        )
    
    def _schedule_migration_notice(self):
        # Notify partner about upcoming signature auth requirement
        migration_deadline = datetime.now() + timedelta(weeks=12)
        self._send_migration_notice(migration_deadline)
```

**Partner Communication Template:**

```markdown
# Partner Migration Notice

Dear [Partner Name],

DataFold is upgrading to RFC 9421 HTTP Message Signatures for enhanced security.

## What You Need to Know:
- **Migration Deadline**: [Date + 12 weeks]
- **Current Status**: Your integration uses legacy authentication
- **Required Action**: Implement signature authentication

## Migration Support:
- **Documentation**: https://docs.datafold.com/migration
- **SDK Updates**: Available for JavaScript, Python, CLI
- **Technical Support**: migration-support@datafold.com
- **Office Hours**: Every Tuesday 2-3 PM PST

## Timeline:
- **Weeks 1-4**: Review documentation and plan migration
- **Weeks 5-8**: Implement and test signature authentication
- **Weeks 9-12**: Deploy to production and validate

Please contact us if you need assistance or have questions.
```

---

## 9. Monitoring and Operations

### 9.1 Migration Metrics and KPIs

#### 9.1.1 Authentication Metrics

**Primary Metrics:**

| Metric | Target | Description |
|--------|--------|-------------|
| **Authentication Success Rate** | > 99.5% | Percentage of successful signature verifications |
| **Authentication Latency** | < 50ms | Average time for signature verification |
| **Key Registration Rate** | 100% | Percentage of clients with registered keys |
| **Migration Progress** | Track weekly | Percentage of traffic using signature auth |
| **Error Rate** | < 0.5% | Authentication-related errors |

**Monitoring Implementation:**

```python
# monitoring/auth_metrics.py
from prometheus_client import Counter, Histogram, Gauge
import time

class AuthenticationMetrics:
    def __init__(self):
        # Success/failure counters
        self.auth_attempts = Counter(
            'datafold_auth_attempts_total',
            'Total authentication attempts',
            ['method', 'status', 'client_id']
        )
        
        # Latency histogram
        self.auth_latency = Histogram(
            'datafold_auth_latency_seconds',
            'Authentication latency',
            ['method']
        )
        
        # Migration progress gauge
        self.migration_progress = Gauge(
            'datafold_migration_progress_percent',
            'Signature auth migration progress',
            ['environment']
        )
        
        # Active keys gauge
        self.active_keys = Gauge(
            'datafold_active_keys_total',
            'Number of active public keys'
        )
    
    def record_auth_attempt(self, method, status, client_id, latency):
        """Record authentication attempt"""
        self.auth_attempts.labels(
            method=method,
            status=status,
            client_id=client_id
        ).inc()
        
        self.auth_latency.labels(method=method).observe(latency)
    
    def update_migration_progress(self, environment, percentage):
        """Update migration progress"""
        self.migration_progress.labels(environment=environment).set(percentage)
    
    def update_active_keys(self, count):
        """Update active key count"""
        self.active_keys.set(count)

# Usage in authentication middleware
metrics = AuthenticationMetrics()

def signature_auth_middleware(request):
    start_time = time.time()
    client_id = extract_client_id(request)
    
    try:
        result = verify_signature(request)
        metrics.record_auth_attempt(
            method='signature',
            status='success',
            client_id=client_id,
            latency=time.time() - start_time
        )
        return result
    except AuthenticationError as e:
        metrics.record_auth_attempt(
            method='signature',
            status='failure',
            client_id=client_id,
            latency=time.time() - start_time
        )
        raise
```

#### 9.1.2 Performance Monitoring

**Grafana Dashboard Configuration:**

```json
{
  "dashboard": {
    "title": "DataFold Signature Authentication",
    "panels": [
      {
        "title": "Authentication Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(datafold_auth_attempts_total{status='success'}[5m]) / rate(datafold_auth_attempts_total[5m]) * 100"
          }
        ]
      },
      {
        "title": "Authentication Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(datafold_auth_latency_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Migration Progress",
        "type": "gauge",
        "targets": [
          {
            "expr": "datafold_migration_progress_percent"
          }
        ]
      }
    ]
  }
}
```

### 9.2 Alerting and Incident Response

#### 9.2.1 Alert Configuration

**Critical Alerts:**

```yaml
# alerts/auth-critical.yml
groups:
  - name: datafold-auth-critical
    rules:
      - alert: AuthenticationFailureRateHigh
        expr: rate(datafold_auth_attempts_total{status="failure"}[5m]) / rate(datafold_auth_attempts_total[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High authentication failure rate detected"
          description: "Authentication failure rate is {{ $value | humanizePercentage }} over last 5 minutes"
          
      - alert: AuthenticationLatencyHigh
        expr: histogram_quantile(0.95, rate(datafold_auth_latency_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High authentication latency detected"
          description: "95th percentile authentication latency is {{ $value }}s"
          
      - alert: MigrationStalled
        expr: increase(datafold_migration_progress_percent[1h]) == 0 and datafold_migration_progress_percent < 100
        for: 2h
        labels:
          severity: warning
        annotations:
          summary: "Migration progress has stalled"
          description: "No migration progress in the last 2 hours"
```

#### 9.2.2 Incident Response Procedures

**Authentication Failure Incident:**

1. **Immediate Response (0-15 minutes)**
   ```bash
   # Check system status
   kubectl get pods -l app=datafold
   
   # Check authentication metrics
   curl http://metrics-endpoint/metrics | grep datafold_auth
   
   # Check server logs
   kubectl logs -l app=datafold --tail=100 | grep -i auth
   ```

2. **Investigation (15-60 minutes)**
   ```bash
   # Analyze failure patterns
   kubectl logs -l app=datafold --since=1h | grep "AUTH_FAILURE" | jq '.client_id' | sort | uniq -c
   
   # Check specific client issues
   kubectl logs -l app=datafold --since=1h | grep "client_id:problematic-client"
   
   # Verify server configuration
   kubectl get configmap datafold-config -o yaml
   ```

3. **Resolution Actions**
   ```bash
   # Option 1: Enable hybrid auth mode (emergency fallback)
   kubectl patch configmap datafold-config --patch '{"data":{"auth_mode":"hybrid"}}'
   
   # Option 2: Restart authentication service
   kubectl rollout restart deployment/datafold
   
   # Option 3: Rollback to previous version
   kubectl rollout undo deployment/datafold
   ```

### 9.3 Security Monitoring

#### 9.3.1 Security Event Detection

**Security Metrics:**

```python
# security/monitoring.py
class SecurityMonitor:
    def __init__(self):
        self.suspicious_activity = Counter(
            'datafold_suspicious_activity_total',
            'Suspicious authentication activity',
            ['type', 'client_id', 'source_ip']
        )
        
        self.replay_attempts = Counter(
            'datafold_replay_attempts_total',
            'Potential replay attack attempts',
            ['client_id']
        )
        
        self.invalid_signatures = Counter(
            'datafold_invalid_signatures_total',
            'Invalid signature attempts',
            ['reason', 'client_id']
        )
    
    def detect_brute_force(self, client_id, source_ip, failure_count):
        """Detect brute force attacks"""
        if failure_count > 10:  # 10 failures in window
            self.suspicious_activity.labels(
                type='brute_force',
                client_id=client_id,
                source_ip=source_ip
            ).inc()
            
            # Alert security team
            self.send_security_alert('brute_force', {
                'client_id': client_id,
                'source_ip': source_ip,
                'failure_count': failure_count
            })
    
    def detect_replay_attack(self, nonce, timestamp, client_id):
        """Detect replay attacks"""
        if self.is_nonce_reused(nonce):
            self.replay_attempts.labels(client_id=client_id).inc()
            
            self.send_security_alert('replay_attack', {
                'client_id': client_id,
                'nonce': nonce,
                'timestamp': timestamp
            })
    
    def track_invalid_signature(self, reason, client_id):
        """Track invalid signature patterns"""
        self.invalid_signatures.labels(
            reason=reason,
            client_id=client_id
        ).inc()
        
        # Check for attack patterns
        recent_failures = self.get_recent_failures(client_id)
        if recent_failures > 5:
            self.suspicious_activity.labels(
                type='signature_attack',
                client_id=client_id,
                source_ip='unknown'
            ).inc()
```

#### 9.3.2 Audit Logging

**Comprehensive Audit Trail:**

```python
# audit/logger.py
import json
import datetime
from typing import Dict, Any

class AuditLogger:
    def __init__(self, log_file='/var/log/datafold/auth-audit.log'):
        self.log_file = log_file
    
    def log_auth_event(self, event_type: str, client_id: str, 
                      details: Dict[str, Any], success: bool):
        """Log authentication events for audit"""
        
        audit_record = {
            'timestamp': datetime.datetime.utcnow().isoformat(),
            'event_type': event_type,
            'client_id': client_id,
            'success': success,
            'details': details,
            'correlation_id': details.get('correlation_id'),
            'source_ip': details.get('source_ip'),
            'user_agent': details.get('user_agent')
        }
        
        # Write to audit log
        with open(self.log_file, 'a') as f:
            f.write(json.dumps(audit_record) + '\n')
    
    def log_key_event(self, event_type: str, key_id: str, 
                     client_id: str, details: Dict[str, Any]):
        """Log key management events"""
        
        key_record = {
            'timestamp': datetime.datetime.utcnow().isoformat(),
            'event_type': event_type,
            'key_id': key_id,
            'client_id': client_id,
            'details': details
        }
        
        with open(self.log_file, 'a') as f:
            f.write(json.dumps(key_record) + '\n')

# Usage in authentication system
audit = AuditLogger()

def authenticate_request(request):
    correlation_id = generate_correlation_id()
    client_id = extract_client_id(request)
    
    try:
        result = verify_signature(request)
        
        # Log successful authentication
        audit.log_auth_event(
            event_type='signature_auth_success',
            client_id=client_id,
            details={
                'correlation_id': correlation_id,
                'source_ip': request.remote_addr,
                'user_agent': request.headers.get('User-Agent'),
                'endpoint': request.path,
                'method': request.method
            },
            success=True
        )
        
        return result
        
    except AuthenticationError as e:
        # Log failed authentication
        audit.log_auth_event(
            event_type='signature_auth_failure',
            client_id=client_id,
            details={
                'correlation_id': correlation_id,
                'source_ip': request.remote_addr,
                'user_agent': request.headers.get('User-Agent'),
                'error_reason': str(e),
                'signature_present': 'signature' in request.headers
            },
            success=False
        )
        
        raise
```

---

## 10. Troubleshooting

### 10.1 Common Migration Issues

#### 10.1.1 Authentication Failures

**Issue:** "Signature verification failed"

**Diagnostic Steps:**
```bash
# 1. Verify key registration
datafold crypto list-keys --client-id my-app

# 2. Test signature generation
datafold auth-test --debug --verbose

# 3. Check server logs
kubectl logs -l app=datafold | grep "signature verification"

# 4. Validate timestamp/nonce
datafold auth-test --show-request-details
```

**Common Causes & Solutions:**

| Cause | Symptoms | Solution |
|-------|----------|----------|
| **Clock Skew** | "Timestamp out of range" | Sync system clocks with NTP |
| **Wrong Key** | "Public key not found" | Verify key registration and key ID |
| **Header Issues** | "Missing signature headers" | Check proxy/gateway configuration |
| **Nonce Reuse** | "Nonce already used" | Ensure unique nonce generation |
| **Signature Format** | "Invalid signature format" | Update to compatible SDK version |

#### 10.1.2 Performance Issues

**Issue:** High authentication latency

**Diagnostic Steps:**
```python
# Performance profiling script
import time
import statistics
from datafold_sdk import DataFoldClient

def benchmark_auth_performance():
    client = DataFoldClient(
        base_url='https://api.datafold.com',
        client_id='benchmark-client',
        authentication={
            'type': 'signature',
            'private_key': 'your-private-key',
            'key_id': 'benchmark-key'
        }
    )
    
    latencies = []
    
    for i in range(100):
        start = time.time()
        try:
            client.get_health()
            latency = time.time() - start
            latencies.append(latency)
        except Exception as e:
            print(f"Request {i} failed: {e}")
    
    print(f"Mean latency: {statistics.mean(latencies)*1000:.2f}ms")
    print(f"95th percentile: {statistics.quantiles(latencies, n=20)[18]*1000:.2f}ms")
    print(f"Success rate: {len(latencies)/100*100:.1f}%")

benchmark_auth_performance()
```

**Performance Optimization:**

```python
# Optimized client configuration
client = DataFoldClient(
    base_url='https://api.datafold.com',
    client_id='optimized-client',
    authentication={
        'type': 'signature',
        'private_key': private_key,
        'key_id': 'optimized-key'
    },
    # Performance optimizations
    connection_pool_size=20,
    request_timeout=30,
    retry_config={
        'max_retries': 3,
        'backoff_factor': 0.5
    }
)
```

#### 10.1.3 Integration Issues

**Issue:** Third-party systems can't authenticate

**Migration Support Script:**

```python
# tools/migration_support.py
class MigrationSupport:
    def __init__(self, client_id):
        self.client_id = client_id
        
    def diagnose_integration(self):
        """Comprehensive integration diagnosis"""
        results = {}
        
        # Test 1: Key pair generation
        try:
            key_pair = DataFoldClient.generate_key_pair()
            results['key_generation'] = 'PASS'
        except Exception as e:
            results['key_generation'] = f'FAIL: {e}'
        
        # Test 2: Public key registration
        try:
            client = DataFoldClient(base_url=self.base_url)
            client.register_public_key(
                key_id=f"{self.client_id}-test",
                public_key=key_pair.public_key
            )
            results['key_registration'] = 'PASS'
        except Exception as e:
            results['key_registration'] = f'FAIL: {e}'
        
        # Test 3: Signature authentication
        try:
            auth_client = DataFoldClient(
                base_url=self.base_url,
                client_id=self.client_id,
                authentication={
                    'type': 'signature',
                    'private_key': key_pair.private_key,
                    'key_id': f"{self.client_id}-test"
                }
            )
            auth_client.get_health()
            results['signature_auth'] = 'PASS'
        except Exception as e:
            results['signature_auth'] = f'FAIL: {e}'
        
        return results
    
    def generate_migration_report(self):
        """Generate migration status report"""
        results = self.diagnose_integration()
        
        report = f"""
DataFold Migration Diagnostic Report
===================================
Client ID: {self.client_id}
Timestamp: {datetime.now().isoformat()}

Test Results:
"""
        
        for test, result in results.items():
            status = "✅" if result == 'PASS' else "❌"
            report += f"{status} {test}: {result}\n"
        
        # Recommendations
        if any('FAIL' in result for result in results.values()):
            report += "\nRecommendations:\n"
            if 'FAIL' in results.get('key_generation', ''):
                report += "- Update SDK to latest version\n"
            if 'FAIL' in results.get('key_registration', ''):
                report += "- Check network connectivity to DataFold server\n"
            if 'FAIL' in results.get('signature_auth', ''):
                report += "- Verify key registration and signature implementation\n"
        
        return report

# Usage
support = MigrationSupport('problematic-client')
print(support.generate_migration_report())
```

### 10.2 Emergency Procedures

#### 10.2.1 Emergency Rollback

**Complete System Rollback:**

```bash
#!/bin/bash
# emergency-rollback.sh

echo "🚨 EMERGENCY ROLLBACK: Disabling signature authentication"

# Step 1: Switch to hybrid mode (immediate)
kubectl patch configmap datafold-config --patch '{
  "data": {
    "SIGNATURE_AUTH_REQUIRED": "false",
    "SIGNATURE_AUTH_MODE": "hybrid"
  }
}'

# Step 2: Restart services to apply config
kubectl rollout restart deployment/datafold

# Step 3: Wait for rollout to complete
kubectl rollout status deployment/datafold

# Step 4: Verify system health
echo "Checking system health..."
for i in {1..30}; do
  if curl -f http://datafold-service/health > /dev/null 2>&1; then
import pytest
import os
from datafold_sdk import DataFoldClient

class TestMigrationEndToEnd:
    
    def setup_method(self):
        """Setup test environment"""
        self.test_server_url = os.getenv('TEST_DATAFOLD_URL', 'http://localhost:8080')
        self.client_id = 'migration-test-client'
        
    @pytest.mark.asyncio
    async def test_signature_authentication_flow(self):
        """Test complete signature authentication flow"""
        
        # Step 1: Generate keypair
        key_pair = DataFoldClient.generate_key_pair()
        assert key_pair.private_key
        assert key_pair.public_key
        
        # Step 2: Register public key
        client = DataFoldClient(
            base_url=self.test_server_url,
            client_id=self.client_id
        )
        
        registration_result = await client.register_public_key(
            key_id=self.client_id,
            public_key=key_pair.public_key
        )
        assert registration_result['success'] is True
        
        # Step 3: Configure signature authentication
        auth_client = DataFoldClient(
            base_url=self.test_server_url,
            client_id=self.client_id,
            authentication={
                'type': 'signature',
                'private_key': key_pair.private_key,
                'key_id': self.client_id
            }
        )
        
        # Step 4: Test authenticated API calls
        schemas = await auth_client.get_schemas()
        assert isinstance(schemas, list)
        
        # Step 5: Test invalid signature handling
        invalid_client = DataFoldClient(
            base_url=self.test_server_url,
            client_id=self.client_id,
            authentication={
                'type': 'signature',
                'private_key': 'invalid-key',
                'key_id': self.client_id
            }
        )
        
        with pytest.raises(Exception) as exc_info:
            await invalid_client.get_schemas()
        assert 'authentication' in str(exc_info.value).lower()
        
    def test_performance_benchmarks(self):
        """Verify signature authentication performance"""
        import time
        
        # Test signature generation performance
        start_time = time.time()
        for _ in range(100):
            DataFoldClient.generate_key_pair()
        key_gen_time = (time.time() - start_time) / 100
        
        # Should be under 10ms per keypair
        assert key_gen_time < 0.01, f"Key generation too slow: {key_gen_time}s"
        
        print(f"✅ Key generation: {key_gen_time*1000:.2f}ms per operation")
```

### 7.2 Migration Testing

#### 7.2.1 Hybrid Authentication Testing

**Test hybrid mode during migration:**

```python
# tests/test_hybrid_auth.py
import pytest
from datafold_sdk import DataFoldClient

class TestHybridAuthentication:
    
    def test_legacy_auth_fallback(self):
        """Test that legacy authentication still works during migration"""
        
        # Legacy client (token-based)
        legacy_client = DataFoldClient(
            base_url='http://localhost:8080',
            api_token='legacy-token'
        )
        
        # Should still work during hybrid mode
        schemas = legacy_client.get_schemas()
        assert isinstance(schemas, list)
        
    def test_signature_auth_preferred(self):
        """Test that signature authentication works in hybrid mode"""
        
        # Signature client
        sig_client = DataFoldClient(
            base_url='http://localhost:8080',
            client_id='test-client',
            authentication={
                'type': 'signature',
                'private_key': 'test-private-key',
                'key_id': 'test-key'
            }
        )
        
        # Should work with signature auth
        schemas = sig_client.get_schemas()
        assert isinstance(schemas, list)
        
    def test_auth_mode_switching(self):
        """Test runtime switching between auth modes"""
        
        # Test different auth configurations
        auth_configs = [
            {'mode': 'legacy'},
            {'mode': 'hybrid', 'percentage': 50},
            {'mode': 'signature'},
        ]
        
        for config in auth_configs:
            response = self.update_server_auth_config(config)
            assert response.status_code == 200
            
            # Verify auth mode is active
            status = self.check_auth_status()
            assert status['mode'] == config['mode']
```

### 7.3 Post-Migration Validation

#### 7.3.1 Production Validation Checklist

**Immediate Post-Migration (0-1 hours):**

- [ ] **Authentication Success Rate**: > 99%
- [ ] **API Response Times**: Within 10% of baseline
- [ ] **Error Rates**: < 1%
- [ ] **System Health**: All services green
- [ ] **User Access**: Critical functions working
- [ ] **Monitoring**: All metrics collecting

**Short-term Validation (1-24 hours):**

- [ ] **Performance Stability**: No degradation trends
- [ ] **Error Pattern Analysis**: No new error types
- [ ] **User Feedback**: No critical issues reported
- [ ] **Security Events**: No authentication bypass attempts
- [ ] **Load Testing**: System handles normal traffic
- [ ] **Backup Procedures**: Rollback plan validated

**Long-term Validation (1-7 days):**

- [ ] **Performance Optimization**: Identify improvement opportunities
- [ ] **Security Audit**: Complete security validation
- [ ] **User Training**: Ensure team understands new system
- [ ] **Documentation**: Update operational procedures
- [ ] **Monitoring Tuning**: Optimize alerting thresholds
- [ ] **Legacy Cleanup**: Remove deprecated authentication code

---

## 8. Common Migration Scenarios

### 8.1 Scenario 1: Unauthenticated → Signature Authentication

**Use Case:** Moving from completely open APIs to secured APIs

**Migration Strategy:**
1. **Phase 1**: Deploy server with signature auth optional
2. **Phase 2**: Update all clients to use signature auth
3. **Phase 3**: Enable signature auth requirement
4. **Phase 4**: Remove unauthenticated access

**Implementation:**

```typescript
// Phase 1: Server configuration
const serverConfig = {
    signatureAuth: {
        enabled: true,
        required: false,  // Optional during migration
        exemptPaths: ['/health', '/metrics']
    }
};

// Phase 2: Client update
const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com',
    // Add authentication
    authentication: {
        type: 'signature',
        privateKey: process.env.DATAFOLD_PRIVATE_KEY,
        keyId: 'web-app-v1'
    }
});

// Phase 3: Require signature auth
const serverConfig = {
    signatureAuth: {
        enabled: true,
        required: true,   // Now required
        exemptPaths: ['/health']  // Reduced exemptions
    }
};
```

### 8.2 Scenario 2: Token-Based → Signature Authentication

**Use Case:** Replacing API tokens with cryptographic signatures

**Migration Strategy:**
1. **Parallel Implementation**: Support both token and signature auth
2. **Client Migration**: Update clients one by one
3. **Gradual Transition**: Increase signature auth percentage
4. **Token Deprecation**: Remove token support

**Implementation:**

```python
# Hybrid authentication client
class HybridDataFoldClient:
    def __init__(self, base_url, api_token=None, signature_auth=None):
        self.base_url = base_url
        self.api_token = api_token
        self.signature_auth = signature_auth
        
    def make_request(self, method, endpoint, data=None):
        # Try signature auth first, fallback to token
        if self.signature_auth:
            try:
                return self._make_signed_request(method, endpoint, data)
            except AuthenticationError:
                if self.api_token:
                    return self._make_token_request(method, endpoint, data)
                raise
        elif self.api_token:
            return self._make_token_request(method, endpoint, data)
        else:
            raise ValueError("No authentication method configured")

# Migration usage
client = HybridDataFoldClient(
    base_url='https://api.datafold.com',
    api_token='legacy-token',  # Fallback
    signature_auth={           # Preferred
        'private_key': os.getenv('DATAFOLD_PRIVATE_KEY'),
        'key_id': 'python-service-v1',
        'client_id': 'python-service'
    }
)
```

### 8.3 Scenario 3: Multi-Environment Migration

**Use Case:** Coordinated migration across dev/staging/production

**Migration Timeline:**

| Environment | Week 1 | Week 2 | Week 3 | Week 4 |
|-------------|--------|--------|--------|--------|
| **Development** | Migrate | Validate | Optimize | Document |
| **Staging** | - | Migrate | Validate | Optimize |
| **Production** | - | - | Migrate | Validate |

**Environment-Specific Configuration:**

```yaml
# environments/development.yml
datafold:
  signature_auth:
    enabled: true
    required: true
    debug_mode: true
    timestamp_tolerance: 600  # Relaxed for dev

# environments/staging.yml  
datafold:
  signature_auth:
    enabled: true
    required: true
    debug_mode: false
    timestamp_tolerance: 300  # Production-like

# environments/production.yml
datafold:
  signature_auth:
    enabled: true
    required: true
    debug_mode: false
    timestamp_tolerance: 300
    monitoring:
      enabled: true
      alert_threshold: 0.01  # 1% error rate
```

### 8.4 Scenario 4: Third-Party Integration Migration

**Use Case:** Migrating external services and partners

**Challenges:**
- External systems outside your control
- Varied technical capabilities
- Different migration schedules
- Support and communication overhead

**Migration Approach:**

```python
# Partner integration adapter
class PartnerIntegrationAdapter:
    def __init__(self, partner_config):
        self.partner_id = partner_config['partner_id']
        self.auth_capability = partner_config.get('signature_auth_capable', False)
        
        if self.auth_capability:
            self.client = self._create_signature_client(partner_config)
        else:
            self.client = self._create_legacy_client(partner_config)
            # Schedule migration notification
            self._schedule_migration_notice()
    
    def _create_signature_client(self, config):
        return DataFoldClient(
            base_url=config['base_url'],
            client_id=f"partner-{self.partner_id}",
            authentication={
                'type': 'signature',
                'private_key': config['private_key'],
                'key_id': config['key_id']
            }
        )
    
    def _create_legacy_client(self, config):
        return DataFoldClient(
            base_url=config['base_url'],
            api_token=config['api_token']
        )
    
    def _schedule_migration_notice(self):
        # Notify partner about upcoming signature auth requirement
        migration_deadline = datetime.now() + timedelta(weeks=12)
        self._send_migration_notice(migration_deadline)
```

**Partner Communication Template:**

```markdown
# Partner Migration Notice

Dear [Partner Name],

DataFold is upgrading to RFC 9421 HTTP Message Signatures for enhanced security.

## What You Need to Know:
- **Migration Deadline**: [Date + 12 weeks]
- **Current Status**: Your integration uses legacy authentication
- **Required Action**: Implement signature authentication

## Migration Support:
- **Documentation**: https://docs.datafold.com/migration
- **SDK Updates**: Available for JavaScript, Python, CLI
- **Technical Support**: migration-support@datafold.com
- **Office Hours**: Every Tuesday 2-3 PM PST

## Timeline:
- **Weeks 1-4**: Review documentation and plan migration
- **Weeks 5-8**: Implement and test signature authentication
- **Weeks 9-12**: Deploy to production and validate

Please contact us if you need assistance or have questions.
```

---

## 9. Monitoring and Operations

### 9.1 Migration Metrics and KPIs

#### 9.1.1 Authentication Metrics

**Primary Metrics:**

| Metric | Target | Description |
|--------|--------|-------------|
| **Authentication Success Rate** | > 99.5% | Percentage of successful signature verifications |
| **Authentication Latency** | < 50ms | Average time for signature verification |
| **Key Registration Rate** | 100% | Percentage of clients with registered keys |
| **Migration Progress** | Track weekly | Percentage of traffic using signature auth |
| **Error Rate** | < 0.5% | Authentication-related errors |

**Monitoring Implementation:**

```python
# monitoring/auth_metrics.py
from prometheus_client import Counter, Histogram, Gauge
import time

class AuthenticationMetrics:
    def __init__(self):
        # Success/failure counters
        self.auth_attempts = Counter(
            'datafold_auth_attempts_total',
            'Total authentication attempts',
            ['method', 'status', 'client_id']
        )
        
        # Latency histogram
        self.auth_latency = Histogram(
            'datafold_auth_latency_seconds',
            'Authentication latency',
            ['method']
        )
        
        # Migration progress gauge
        self.migration_progress = Gauge(
            'datafold_migration_progress_percent',
            'Signature auth migration progress',
            ['environment']
        )
        
        # Active keys gauge
        self.active_keys = Gauge(
            'datafold_active_keys_total',
            'Number of active public keys'
        )
    
    def record_auth_attempt(self, method, status, client_id, latency):
        """Record authentication attempt"""
        self.auth_attempts.labels(
            method=method,
            status=status,
            client_id=client_id
        ).inc()
        
        self.auth_latency.labels(method=method).observe(latency)
    
    def update_migration_progress(self, environment, percentage):
        """Update migration progress"""
        self.migration_progress.labels(environment=environment).set(percentage)
    
    def update_active_keys(self, count):
        """Update active key count"""
        self.active_keys.set(count)

# Usage in authentication middleware
metrics = AuthenticationMetrics()

def signature_auth_middleware(request):
    start_time = time.time()
    client_id = extract_client_id(request)
    
    try:
        result = verify_signature(request)
        metrics.record_auth_attempt(
            method='signature',
            status='success',
            client_id=client_id,
            latency=time.time() - start_time
        )
        return result
    except AuthenticationError as e:
        metrics.record_auth_attempt(
            method='signature',
            status='failure',
            client_id=client_id,
            latency=time.time() - start_time
        )
        raise
```

#### 9.1.2 Performance Monitoring

**Grafana Dashboard Configuration:**

```json
{
  "dashboard": {
    "title": "DataFold Signature Authentication",
    "panels": [
      {
        "title": "Authentication Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(datafold_auth_attempts_total{status='success'}[5m]) / rate(datafold_auth_attempts_total[5m]) * 100"
          }
        ]
      },
      {
        "title": "Authentication Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(datafold_auth_latency_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Migration Progress",
        "type": "gauge",
        "targets": [
          {
            "expr": "datafold_migration_progress_percent"
          }
        ]
      }
    ]
  }
}
```

### 9.2 Alerting and Incident Response

#### 9.2.1 Alert Configuration

**Critical Alerts:**

```yaml
# alerts/auth-critical.yml
groups:
  - name: datafold-auth-critical
    rules:
      - alert: AuthenticationFailureRateHigh
        expr: rate(datafold_auth_attempts_total{status="failure"}[5m]) / rate(datafold_auth_attempts_total[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High authentication failure rate detected"
          description: "Authentication failure rate is {{ $value | humanizePercentage }} over last 5 minutes"
          
      - alert: AuthenticationLatencyHigh
        expr: histogram_quantile(0.95, rate(datafold_auth_latency_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High authentication latency detected"
          description: "95th percentile authentication latency is {{ $value }}s"
          
      - alert: MigrationStalled
        expr: increase(datafold_migration_progress_percent[1h]) == 0 and datafold_migration_progress_percent < 100
        for: 2h
        labels:
          severity: warning
        annotations:
          summary: "Migration progress has stalled"
          description: "No migration progress in the last 2 hours"
```

#### 9.2.2 Incident Response Procedures

**Authentication Failure Incident:**

1. **Immediate Response (0-15 minutes)**
   ```bash
   # Check system status
   kubectl get pods -l app=datafold
   
   # Check authentication metrics
   curl http://metrics-endpoint/metrics | grep datafold_auth
   
   # Check server logs
   kubectl logs -l app=datafold --tail=100 | grep -i auth
   ```

2. **Investigation (15-60 minutes)**
   ```bash
   # Analyze failure patterns
   kubectl logs -l app=datafold --since=1h | grep "AUTH_FAILURE" | jq '.client_id' | sort | uniq -c
   
   # Check specific client issues
   kubectl logs -l app=datafold --since=1h | grep "client_id:problematic-client"
   
   # Verify server configuration
   kubectl get configmap datafold-config -o yaml
   ```

3. **Resolution Actions**
   ```bash
   # Option 1: Enable hybrid auth mode (emergency fallback)
   kubectl patch configmap datafold-config --patch '{"data":{"auth_mode":"hybrid"}}'
   
   # Option 2: Restart authentication service
   kubectl rollout restart deployment/datafold
   
   # Option 3: Rollback to previous version
   kubectl rollout undo deployment/datafold
   ```

### 9.3 Security Monitoring

#### 9.3.1 Security Event Detection

**Security Metrics:**

```python
# security/monitoring.py
class SecurityMonitor:
    def __init__(self):
        self.suspicious_activity = Counter(
            'datafold_suspicious_activity_total',
            'Suspicious authentication activity',
            ['type', 'client_id', 'source_ip']
        )
        
        self.replay_attempts = Counter(
            'datafold_replay_attempts_total',
            'Potential replay attack attempts',
            ['client_id']
        )
        
        self.invalid_signatures = Counter(
            'datafold_invalid_signatures_total',
            'Invalid signature attempts',
            ['reason', 'client_id']
        )
    
    def detect_brute_force(self, client_id, source_ip, failure_count):
        """Detect brute force attacks"""
        if failure_count > 10:  # 10 failures in window
            self.suspicious_activity.labels(
                type='brute_force',
                client_id=client_id,
                source_ip=source_ip
            ).inc()
            
            # Alert security team
            self.send_security_alert('brute_force', {
                'client_id': client_id,
                'source_ip': source_ip,
                'failure_count': failure_count
            })
    
    def detect_replay_attack(self, nonce, timestamp, client_id):
        """Detect replay attacks"""
        if self.is_nonce_reused(nonce):
            self.replay_attempts.labels(client_id=client_id).inc()
            
            self.send_security_alert('replay_attack', {
                'client_id': client_id,
                'nonce': nonce,
                'timestamp': timestamp
            })
    
    def track_invalid_signature(self, reason, client_id):
        """Track invalid signature patterns"""
        self.invalid_signatures.labels(
            reason=reason,
            client_id=client_id
        ).inc()
        
        # Check for attack patterns
        recent_failures = self.get_recent_failures(client_id)
        if recent_failures > 5:
            self.suspicious_activity.labels(
                type='signature_attack',
                client_id=client_id,
                source_ip='unknown'
            ).inc()
```

---

## 10. Troubleshooting

### 10.1 Common Migration Issues

#### 10.1.1 Authentication Failures

**Issue:** "Signature verification failed"

**Diagnostic Steps:**
```bash
# 1. Verify key registration
datafold crypto list-keys --client-id my-app

# 2. Test signature generation
datafold auth-test --debug --verbose

# 3. Check server logs
kubectl logs -l app=datafold | grep "signature verification"

# 4. Validate timestamp/nonce
datafold auth-test --show-request-details
```

**Common Causes & Solutions:**

| Cause | Symptoms | Solution |
|-------|----------|----------|
| **Clock Skew** | "Timestamp out of range" | Sync system clocks with NTP |
| **Wrong Key** | "Public key not found" | Verify key registration and key ID |
| **Header Issues** | "Missing signature headers" | Check proxy/gateway configuration |
| **Nonce Reuse** | "Nonce already used" | Ensure unique nonce generation |
| **Signature Format** | "Invalid signature format" | Update to compatible SDK version |

#### 10.1.2 Performance Issues

**Issue:** High authentication latency

**Diagnostic Steps:**
```python
# Performance profiling script
import time
import statistics
from datafold_sdk import DataFoldClient

def benchmark_auth_performance():
    client = DataFoldClient(
        base_url='https://api.datafold.com',
        client_id='benchmark-client',
        authentication={
            'type': 'signature',
            'private_key': 'your-private-key',
            'key_id': 'benchmark-key'
        }
    )
    
    latencies = []
    
    for i in range(100):
        start = time.time()
        try:
            client.get_health()
            latency = time.time() - start
            latencies.append(latency)
        except Exception as e:
            print(f"Request {i} failed: {e}")
    
    print(f"Mean latency: {statistics.mean(latencies)*1000:.2f}ms")
    print(f"95th percentile: {statistics.quantiles(latencies, n=20)[18]*1000:.2f}ms")
    print(f"Success rate: {len(latencies)/100*100:.1f}%")

benchmark_auth_performance()
```

### 10.2 Emergency Procedures

#### 10.2.1 Emergency Rollback

**Complete System Rollback:**

```bash
#!/bin/bash
# emergency-rollback.sh

echo "🚨 EMERGENCY ROLLBACK: Disabling signature authentication"

# Step 1: Switch to hybrid mode (immediate)
kubectl patch configmap datafold-config --patch '{
  "data": {
    "SIGNATURE_AUTH_REQUIRED": "false",
    "SIGNATURE_AUTH_MODE": "hybrid"
  }
}'

# Step 2: Restart services to apply config
kubectl rollout restart deployment/datafold

# Step 3: Wait for rollout to complete
kubectl rollout status deployment/datafold

# Step 4: Verify system health
echo "Checking system health..."
for i in {1..30}; do
  if curl -f http://datafold-service/health > /dev/null 2>&1; then
    echo "✅ System is healthy"
    break
  fi
  echo "Waiting for system recovery... ($i/30)"
  sleep 10
done

# Step 5: Notify stakeholders
echo "📧 Sending rollback notification..."
curl -X POST "https://api.slack.com/incoming/webhook" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "🚨 DATAFOLD EMERGENCY ROLLBACK EXECUTED\nSignature authentication has been disabled.\nSystem is now operating in hybrid mode.\nIncident response team has been notified."
  }'

echo "🔄 Emergency rollback completed"
```

---

## 11. Post-Migration Operations

### 11.1 Legacy System Cleanup

#### 11.1.1 Decommissioning Legacy Authentication

**Cleanup Checklist:**

- [ ] **Remove API Tokens**: Deactivate all legacy API tokens
- [ ] **Update Documentation**: Remove references to legacy authentication
- [ ] **Clean Up Code**: Remove legacy authentication code paths
- [ ] **Database Cleanup**: Remove legacy authentication tables/fields
- [ ] **Configuration Cleanup**: Remove legacy authentication configuration
- [ ] **Monitoring Cleanup**: Update dashboards and alerts

### 11.2 Performance Optimization

After migration, optimize signature authentication performance:

```rust
// Performance optimizations for production
pub struct OptimizedSignatureVerifier {
    key_cache: LruCache<String, PublicKey>,
    verification_cache: LruCache<String, bool>,
    nonce_bloom: BloomFilter,
}
```

### 11.3 Security Hardening

#### 11.3.1 Post-Migration Security Audit

**Security Checklist:**

- [ ] **Key Rotation**: Implement regular key rotation schedule
- [ ] **Access Control**: Review and update key access permissions
- [ ] **Monitoring**: Ensure all security events are logged and monitored
- [ ] **Incident Response**: Update incident response procedures
- [ ] **Compliance**: Verify compliance with security standards

### 11.4 Team Training and Documentation

**Training Schedule:**

| Week | Audience | Topic | Duration |
|------|----------|-------|----------|
| 1 | **Developers** | Signature Authentication Fundamentals | 2 hours |
| 1 | **DevOps** | Server Configuration and Monitoring | 2 hours |
| 2 | **Support** | Troubleshooting Authentication Issues | 1 hour |

---

## Conclusion

This comprehensive migration guide provides everything needed to successfully migrate existing DataFold systems to signature authentication. The migration approach prioritizes:

1. **Safety First**: Multiple rollback options and gradual migration strategies
2. **Comprehensive Testing**: Extensive validation at every stage
3. **Operational Excellence**: Detailed monitoring, alerting, and troubleshooting procedures
4. **Team Enablement**: Training programs and updated documentation
5. **Security**: Enhanced security posture with modern cryptographic authentication

### Migration Success Criteria

✅ **Technical Success:**
- 99.5%+ authentication success rate
- <50ms authentication latency
- Zero data loss or service interruption
- All legacy authentication removed

✅ **Operational Success:**
- Team trained on new authentication system
- Monitoring and alerting operational
- Documentation updated and current
- Support procedures tested and validated

✅ **Security Success:**
- All API endpoints secured with signature authentication
- Security audit passed
- Incident response procedures updated
- Compliance requirements met

### Next Steps

1. **Complete Pre-Migration Assessment** using the provided checklist
2. **Select Appropriate Migration Strategy** based on your environment and requirements
3. **Execute Migration Plan** following the phase-based approach
4. **Validate Migration Success** using the testing and monitoring procedures
5. **Complete Post-Migration Activities** including cleanup and optimization

For additional support during migration:
- **Technical Support:** migration-support@datafold.com
- **Documentation:** https://docs.datafold.com/migration
- **Community:** https://community.datafold.com/migration

**Migration successful!** Welcome to secure, modern authentication with DataFold. 🎉