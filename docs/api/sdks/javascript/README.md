# DataFold JavaScript SDK

The DataFold JavaScript SDK provides a complete implementation of RFC 9421 HTTP Message Signatures for browser and Node.js environments. It offers a simple, type-safe API for authenticating requests to DataFold services.

## 🚀 Quick Start

### Installation

```bash
# npm
npm install @datafold/sdk

# yarn
yarn add @datafold/sdk

# pnpm
pnpm add @datafold/sdk
```

### Basic Usage

```javascript
import { DataFoldClient, generateKeyPair } from '@datafold/sdk';

// Generate Ed25519 keypair
const keyPair = await generateKeyPair();

// Create authenticated client
const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'your-client-id',
  privateKey: keyPair.privateKey
});

// Make authenticated requests
const schemas = await client.get('/api/schemas');
const newSchema = await client.post('/api/schemas', schemaData);
```

## 📦 Package Contents

The SDK provides several modules for different use cases:

### Core Modules
- **[`DataFoldClient`](api-reference.md#datafoldclient)** - Main HTTP client with authentication
- **[`RFC9421Signer`](api-reference.md#rfc9421signer)** - Low-level request signing
- **[`generateKeyPair`](api-reference.md#generatekeypair)** - Ed25519 key generation

### Crypto Modules
- **[`Ed25519`](api-reference.md#ed25519)** - Ed25519 cryptographic operations
- **[`KeyDerivation`](api-reference.md#keyderivation)** - Key derivation utilities
- **[`SecureStorage`](api-reference.md#securestorage)** - Browser key storage

### Utility Modules
- **[`Validation`](api-reference.md#validation)** - Input validation helpers
- **[`Utils`](api-reference.md#utils)** - Common utilities

## 🎯 Platform Support

### Browser Support
- **Modern Browsers**: Chrome 67+, Firefox 60+, Safari 13.1+, Edge 79+
- **WebCrypto API**: Required for cryptographic operations
- **ES2020**: Module syntax and modern JavaScript features
- **TypeScript**: Full type definitions included

### Node.js Support
- **Versions**: Node.js 16+ (LTS)
- **ES Modules**: Native ESM support
- **CommonJS**: Available via build tools
- **TypeScript**: Native TypeScript support

### Framework Compatibility
- **React**: ✅ Hooks and components available
- **Vue**: ✅ Composition API integration
- **Angular**: ✅ Service and injection support
- **Svelte**: ✅ Store integration
- **Next.js**: ✅ SSR and client-side support
- **Nuxt**: ✅ Universal app support

## 🔧 Configuration

### Basic Configuration

```javascript
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
  // Required
  serverUrl: 'https://api.datafold.com',
  clientId: 'your-client-id',
  privateKey: privateKeyBytes,
  
  // Optional
  timeout: 10000,           // Request timeout (ms)
  retries: 3,               // Retry attempts
  securityProfile: 'standard', // Security profile
  debug: false              // Debug logging
});
```

### Advanced Configuration

```javascript
import { DataFoldClient, SecurityProfiles } from '@datafold/sdk';

const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'your-client-id',
  privateKey: privateKeyBytes,
  
  // Security settings
  securityProfile: SecurityProfiles.STRICT,
  
  // Signing configuration
  signingConfig: {
    algorithm: 'ed25519',
    components: {
      method: true,
      targetUri: true,
      headers: ['content-type', 'content-digest'],
      contentDigest: true
    },
    timestampGenerator: () => Math.floor(Date.now() / 1000),
    nonceGenerator: () => crypto.randomUUID().replace(/-/g, '')
  },
  
  // HTTP configuration
  httpConfig: {
    timeout: 15000,
    retries: 5,
    retryDelay: 1000,
    userAgent: 'MyApp/1.0.0 DataFoldSDK/2.0.0'
  },
  
  // Interceptors
  requestInterceptor: (config) => {
    console.log('Request:', config);
    return config;
  },
  responseInterceptor: (response) => {
    console.log('Response:', response);
    return response;
  }
});
```

### Environment-Specific Configuration

```javascript
// Development
const devClient = new DataFoldClient({
  serverUrl: 'http://localhost:9001',
  clientId: 'dev-client',
  privateKey: devPrivateKey,
  securityProfile: 'lenient',
  debug: true
});

// Production
const prodClient = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'prod-client',
  privateKey: prodPrivateKey,
  securityProfile: 'strict',
  debug: false,
  timeout: 5000
});
```

## 🔐 Authentication Setup

### 1. Generate Keys

```javascript
import { generateKeyPair } from '@datafold/sdk';

// Generate new Ed25519 keypair
const keyPair = await generateKeyPair();

console.log('Private Key (keep secret!):', keyPair.privateKey);
console.log('Public Key (register with server):', keyPair.publicKey);
```

### 2. Register Public Key

```javascript
import { registerPublicKey } from '@datafold/sdk';

const registration = await registerPublicKey({
  serverUrl: 'https://api.datafold.com',
  clientId: 'my-app-client',
  publicKey: keyPair.publicKey,
  keyName: 'Production Key',
  metadata: {
    environment: 'production',
    version: '1.0.0'
  }
});

console.log('Registration successful:', registration);
```

### 3. Create Authenticated Client

```javascript
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: registration.client_id,
  privateKey: keyPair.privateKey
});
```

## 💻 Usage Examples

### Basic API Operations

```javascript
// Get all schemas
const schemas = await client.get('/api/schemas');

// Get specific schema
const schema = await client.get('/api/schemas/user_events');

// Create new schema
const newSchema = await client.post('/api/schemas', {
  name: 'product_events',
  version: '1.0.0',
  fields: [
    { name: 'product_id', type: 'string', required: true },
    { name: 'event_type', type: 'string', required: true },
    { name: 'timestamp', type: 'datetime', required: true }
  ]
});

// Update schema
const updatedSchema = await client.put('/api/schemas/product_events', {
  version: '1.1.0',
  fields: [
    { name: 'product_id', type: 'string', required: true },
    { name: 'event_type', type: 'string', required: true },
    { name: 'timestamp', type: 'datetime', required: true },
    { name: 'user_id', type: 'string', required: false }
  ]
});

// Delete schema
await client.delete('/api/schemas/old_schema');
```

### Data Validation

```javascript
// Validate data against schema
const validation = await client.post('/api/schemas/user_events/validate', {
  data: {
    user_id: 'user123',
    event_type: 'page_view',
    timestamp: '2025-06-09T23:27:09Z',
    page_url: 'https://example.com/products'
  },
  options: {
    strict: true,
    return_errors: true
  }
});

if (validation.valid) {
  console.log('Data is valid!');
} else {
  console.log('Validation errors:', validation.errors);
}
```

### Batch Operations

```javascript
// Batch validate multiple records
const batchValidation = await client.post('/api/schemas/user_events/validate/batch', {
  records: [
    { user_id: 'user1', event_type: 'login', timestamp: '2025-06-09T10:00:00Z' },
    { user_id: 'user2', event_type: 'logout', timestamp: '2025-06-09T11:00:00Z' },
    { user_id: 'user3', event_type: 'purchase', timestamp: '2025-06-09T12:00:00Z' }
  ],
  options: {
    fail_fast: false,
    return_details: true
  }
});

console.log(`Validated ${batchValidation.total_records} records`);
console.log(`${batchValidation.valid_records} valid, ${batchValidation.invalid_records} invalid`);
```

## 🌐 Framework Integration

### React Integration

```jsx
import React, { createContext, useContext, useState, useEffect } from 'react';
import { DataFoldClient } from '@datafold/sdk';

// Create DataFold context
const DataFoldContext = createContext(null);

// Provider component
export function DataFoldProvider({ children, config }) {
  const [client, setClient] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function initializeClient() {
      try {
        const datafoldClient = new DataFoldClient(config);
        await datafoldClient.authenticate();
        setClient(datafoldClient);
      } catch (err) {
        setError(err);
      } finally {
        setLoading(false);
      }
    }

    initializeClient();
  }, [config]);

  return (
    <DataFoldContext.Provider value={{ client, loading, error }}>
      {children}
    </DataFoldContext.Provider>
  );
}

// Hook for using DataFold client
export function useDataFold() {
  const context = useContext(DataFoldContext);
  if (!context) {
    throw new Error('useDataFold must be used within DataFoldProvider');
  }
  return context;
}

// Component example
function SchemaManager() {
  const { client, loading, error } = useDataFold();
  const [schemas, setSchemas] = useState([]);

  useEffect(() => {
    if (client) {
      client.get('/api/schemas')
        .then(response => setSchemas(response.data))
        .catch(err => console.error('Failed to load schemas:', err));
    }
  }, [client]);

  if (loading) return <div>Loading DataFold...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      <h2>DataFold Schemas</h2>
      <ul>
        {schemas.map(schema => (
          <li key={schema.name}>{schema.name} v{schema.version}</li>
        ))}
      </ul>
    </div>
  );
}
```

### Vue 3 Composition API

```vue
<template>
  <div>
    <h2>Schema Validation</h2>
    <form @submit.prevent="validateData">
      <textarea v-model="jsonData" placeholder="Enter JSON data..."></textarea>
      <select v-model="selectedSchema">
        <option v-for="schema in schemas" :key="schema.name" :value="schema.name">
          {{ schema.name }}
        </option>
      </select>
      <button type="submit" :disabled="loading">Validate</button>
    </form>
    
    <div v-if="validationResult">
      <h3>Validation Result</h3>
      <p>Valid: {{ validationResult.valid }}</p>
      <ul v-if="validationResult.errors">
        <li v-for="error in validationResult.errors" :key="error">{{ error }}</li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
  serverUrl: import.meta.env.VITE_DATAFOLD_SERVER_URL,
  clientId: import.meta.env.VITE_DATAFOLD_CLIENT_ID,
  privateKey: import.meta.env.VITE_DATAFOLD_PRIVATE_KEY
});

const schemas = ref([]);
const selectedSchema = ref('');
const jsonData = ref('');
const validationResult = ref(null);
const loading = ref(false);

onMounted(async () => {
  try {
    const response = await client.get('/api/schemas');
    schemas.value = response.data;
  } catch (error) {
    console.error('Failed to load schemas:', error);
  }
});

async function validateData() {
  if (!selectedSchema.value || !jsonData.value) return;
  
  loading.value = true;
  try {
    const data = JSON.parse(jsonData.value);
    const result = await client.post(`/api/schemas/${selectedSchema.value}/validate`, {
      data,
      options: { strict: true, return_errors: true }
    });
    
    validationResult.value = result.data;
  } catch (error) {
    console.error('Validation failed:', error);
    validationResult.value = { valid: false, errors: [error.message] };
  } finally {
    loading.value = false;
  }
}
</script>
```

### Next.js Integration

```javascript
// lib/datafold.js
import { DataFoldClient } from '@datafold/sdk';

let client;

export function getDataFoldClient() {
  if (!client) {
    client = new DataFoldClient({
      serverUrl: process.env.NEXT_PUBLIC_DATAFOLD_SERVER_URL,
      clientId: process.env.NEXT_PUBLIC_DATAFOLD_CLIENT_ID,
      privateKey: process.env.DATAFOLD_PRIVATE_KEY
    });
  }
  return client;
}

// pages/api/schemas/index.js
import { getDataFoldClient } from '../../../lib/datafold';

export default async function handler(req, res) {
  const client = getDataFoldClient();
  
  try {
    if (req.method === 'GET') {
      const schemas = await client.get('/api/schemas');
      res.status(200).json(schemas.data);
    } else if (req.method === 'POST') {
      const newSchema = await client.post('/api/schemas', req.body);
      res.status(201).json(newSchema.data);
    } else {
      res.setHeader('Allow', ['GET', 'POST']);
      res.status(405).end(`Method ${req.method} Not Allowed`);
    }
  } catch (error) {
    console.error('DataFold API error:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
}

// pages/schemas.js
import { useState, useEffect } from 'react';

export default function Schemas() {
  const [schemas, setSchemas] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('/api/schemas')
      .then(res => res.json())
      .then(data => {
        setSchemas(data);
        setLoading(false);
      })
      .catch(err => {
        console.error('Failed to load schemas:', err);
        setLoading(false);
      });
  }, []);

  if (loading) return <div>Loading schemas...</div>;

  return (
    <div>
      <h1>DataFold Schemas</h1>
      <ul>
        {schemas.map(schema => (
          <li key={schema.name}>
            <h3>{schema.name}</h3>
            <p>Version: {schema.version}</p>
            <p>Fields: {schema.fields.length}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}
```

## 🔧 Advanced Usage

### Custom HTTP Client

```javascript
import { RFC9421Signer } from '@datafold/sdk';
import axios from 'axios';

// Create custom signer
const signer = new RFC9421Signer({
  algorithm: 'ed25519',
  keyId: 'my-client-id',
  privateKey: privateKeyBytes,
  components: {
    method: true,
    targetUri: true,
    headers: ['content-type', 'content-digest'],
    contentDigest: true
  }
});

// Create axios instance with interceptor
const httpClient = axios.create({
  baseURL: 'https://api.datafold.com'
});

httpClient.interceptors.request.use(async (config) => {
  // Convert axios config to signable request
  const signableRequest = {
    method: config.method.toUpperCase(),
    url: config.url,
    headers: config.headers,
    body: config.data ? JSON.stringify(config.data) : undefined
  };

  // Sign the request
  const signatureResult = await signer.signRequest(signableRequest);
  
  // Apply signature headers
  Object.assign(config.headers, signatureResult.headers);
  
  return config;
});

// Use the authenticated client
const response = await httpClient.post('/api/schemas', schemaData);
```

### WebWorker Integration

```javascript
// worker.js
import { RFC9421Signer } from '@datafold/sdk';

let signer;

self.onmessage = async function(e) {
  const { type, data } = e.data;
  
  switch (type) {
    case 'init':
      signer = new RFC9421Signer(data.config);
      self.postMessage({ type: 'ready' });
      break;
      
    case 'sign':
      try {
        const result = await signer.signRequest(data.request);
        self.postMessage({ type: 'signed', result });
      } catch (error) {
        self.postMessage({ type: 'error', error: error.message });
      }
      break;
  }
};

// main.js
const worker = new Worker('/worker.js');

worker.postMessage({
  type: 'init',
  data: {
    config: {
      algorithm: 'ed25519',
      keyId: 'my-client-id',
      privateKey: privateKeyBytes
    }
  }
});

worker.onmessage = function(e) {
  const { type, result, error } = e.data;
  
  if (type === 'ready') {
    console.log('Worker initialized');
  } else if (type === 'signed') {
    console.log('Request signed:', result);
  } else if (type === 'error') {
    console.error('Signing error:', error);
  }
};

// Sign request in worker
worker.postMessage({
  type: 'sign',
  data: {
    request: {
      method: 'POST',
      url: '/api/schemas',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(schemaData)
    }
  }
});
```

## 📊 Performance Optimization

### Connection Pooling

```javascript
import { DataFoldClient } from '@datafold/sdk';

// Create client with connection pooling
const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'your-client-id',
  privateKey: privateKeyBytes,
  
  httpConfig: {
    // Keep connections alive
    keepAlive: true,
    keepAliveMsecs: 1000,
    
    // Connection pooling
    maxSockets: 50,
    maxFreeSockets: 10,
    
    // Timeout settings
    timeout: 10000,
    freeSocketTimeout: 15000
  }
});
```

### Request Batching

```javascript
import { BatchClient } from '@datafold/sdk';

const batchClient = new BatchClient({
  client: client,
  batchSize: 100,
  flushInterval: 1000,
  maxWaitTime: 5000
});

// Queue multiple requests
batchClient.queue('POST', '/api/data/events', event1);
batchClient.queue('POST', '/api/data/events', event2);
batchClient.queue('POST', '/api/data/events', event3);

// Requests are automatically batched and sent
await batchClient.flush();
```

### Caching

```javascript
import { CachedDataFoldClient } from '@datafold/sdk';

const cachedClient = new CachedDataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'your-client-id',
  privateKey: privateKeyBytes,
  
  cache: {
    // Cache GET requests for 5 minutes
    ttl: 300000,
    maxSize: 1000,
    
    // Cache keys
    keyGenerator: (method, url) => `${method}:${url}`,
    
    // Storage backend
    storage: 'memory' // or 'localStorage' in browser
  }
});

// First request hits the server
const schemas1 = await cachedClient.get('/api/schemas');

// Second request returns cached result
const schemas2 = await cachedClient.get('/api/schemas');
```

## 🧪 Testing

### Unit Testing

```javascript
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { DataFoldClient, generateKeyPair } from '@datafold/sdk';

describe('DataFold SDK', () => {
  let client;
  let keyPair;

  beforeEach(async () => {
    keyPair = await generateKeyPair();
    client = new DataFoldClient({
      serverUrl: 'http://localhost:9001',
      clientId: 'test-client',
      privateKey: keyPair.privateKey
    });
  });

  it('should generate valid keypairs', async () => {
    const keys = await generateKeyPair();
    
    expect(keys.privateKey).toBeInstanceOf(Uint8Array);
    expect(keys.privateKey).toHaveLength(32);
    expect(keys.publicKey).toBeInstanceOf(Uint8Array);
    expect(keys.publicKey).toHaveLength(32);
  });

  it('should sign requests correctly', async () => {
    const signer = new RFC9421Signer({
      algorithm: 'ed25519',
      keyId: 'test-client',
      privateKey: keyPair.privateKey
    });

    const request = {
      method: 'POST',
      url: '/api/test',
      headers: { 'content-type': 'application/json' },
      body: '{"test": true}'
    };

    const result = await signer.signRequest(request);
    
    expect(result.signatureInput).toContain('ed25519');
    expect(result.signature).toMatch(/^sig1=:.+:$/);
    expect(result.headers).toHaveProperty('signature-input');
    expect(result.headers).toHaveProperty('signature');
  });
});
```

### Integration Testing

```javascript
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { DataFoldClient, generateKeyPair, registerPublicKey } from '@datafold/sdk';

describe('DataFold Integration', () => {
  let client;
  let testServer;

  beforeAll(async () => {
    // Start test server
    testServer = await startTestServer();
    
    // Generate test keys
    const keyPair = await generateKeyPair();
    
    // Register public key
    await registerPublicKey({
      serverUrl: 'http://localhost:9001',
      clientId: 'integration-test',
      publicKey: keyPair.publicKey
    });
    
    // Create client
    client = new DataFoldClient({
      serverUrl: 'http://localhost:9001',
      clientId: 'integration-test',
      privateKey: keyPair.privateKey
    });
  });

  afterAll(async () => {
    await testServer.close();
  });

  it('should authenticate and make API calls', async () => {
    const response = await client.get('/api/schemas');
    expect(response.status).toBe(200);
    expect(Array.isArray(response.data)).toBe(true);
  });

  it('should handle errors gracefully', async () => {
    await expect(client.get('/api/nonexistent')).rejects.toThrow();
  });
});
```

## 🔗 Related Documentation

- **[API Reference](api-reference.md)** - Complete API documentation
- **[Examples](examples.md)** - Working code examples
- **[Integration Guide](integration-guide.md)** - Framework integration
- **[Python SDK](../python/README.md)** - Python implementation
- **[CLI Tool](../cli/README.md)** - Command-line interface

## 📞 Support

- **Documentation**: [API Reference](api-reference.md)
- **Examples**: [Usage Examples](examples.md)
- **Issues**: [GitHub Issues](https://github.com/datafold/js-sdk/issues)
- **Community**: [Discord](https://discord.gg/datafold)

---

**Next**: Explore the [complete API reference](api-reference.md) or check out [practical examples](examples.md).