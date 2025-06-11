# DataFold API Documentation

Welcome to the comprehensive DataFold API documentation. This guide covers all aspects of DataFold's signature authentication system, from basic concepts to advanced integration patterns.

## 🚀 Quick Start

Get started with DataFold's signature authentication in just a few steps:

1. **[Generate Keys](authentication/getting-started.md#key-generation)** - Create Ed25519 keypairs
2. **[Register Public Key](authentication/getting-started.md#registration)** - Register with DataFold server
3. **[Sign Requests](authentication/getting-started.md#signing-requests)** - Authenticate API calls
4. **[Make API Calls](authentication/getting-started.md#api-usage)** - Access protected endpoints

```javascript
// Quick example - JavaScript SDK
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'your-client-id',
  privateKey: yourPrivateKey
});

const response = await client.post('/api/schemas', schemaData);
```

## 📚 Documentation Sections

### Authentication System
- **[Overview](authentication/overview.md)** - How DataFold authentication works
- **[Getting Started](authentication/getting-started.md)** - Step-by-step setup guide  
- **[Key Management](authentication/key-management.md)** - Key generation, rotation, and security
- **[Request Signing](authentication/request-signing.md)** - RFC 9421 implementation details
- **[Troubleshooting](authentication/troubleshooting.md)** - Common issues and solutions

### API Endpoints
- **[Public Key Registration](endpoints/public-key-registration.md)** - Register and manage public keys
- **[Signature Verification](endpoints/signature-verification.md)** - Verify digital signatures
- **[Crypto Initialization](endpoints/crypto-initialization.md)** - Database crypto setup
- **[Error Codes Reference](endpoints/error-codes.md)** - Complete error code documentation

### Client SDKs
- **[JavaScript SDK](sdks/javascript/README.md)** - Browser and Node.js implementation
- **[Python SDK](sdks/python/README.md)** - Python client library
- **[CLI Tool](sdks/cli/README.md)** - Command-line interface

### Integration Guides
- **[Security Best Practices](guides/security-best-practices.md)** - Production security guidelines
- **[Performance Optimization](guides/performance-optimization.md)** - Optimize for scale
- **[Migration Guide](guides/migration-guide.md)** - Migrate from existing authentication
- **[Deployment Guide](guides/deployment-guide.md)** - Production deployment setup

### Technical Reference
- **[RFC 9421 Implementation](reference/rfc9421-implementation.md)** - HTTP Message Signatures standard
- **[Signature Components](reference/signature-components.md)** - Message components reference
- **[Configuration Options](reference/configuration-options.md)** - All configuration settings

## 🔐 Authentication Overview

DataFold uses **RFC 9421 HTTP Message Signatures** with **Ed25519** digital signatures for authentication:

- **Stateless**: No server-side sessions required
- **Secure**: Ed25519 provides 128-bit security level
- **Standards-Based**: Full RFC 9421 compliance
- **Replay-Protected**: Timestamp and nonce validation prevents replay attacks
- **Cross-Platform**: Consistent implementation across all SDKs

### Key Components

| Component | Purpose | Example |
|-----------|---------|---------|
| **Client ID** | Unique client identifier | `"client-123-prod"` |
| **Ed25519 Keypair** | Cryptographic identity | 32-byte private/public keys |
| **Signature Headers** | Request authentication | `Signature`, `Signature-Input` |
| **Message Signing** | Request integrity | RFC 9421 canonical message |

## 🛠️ Supported Platforms

DataFold's signature authentication works across all major platforms:

### JavaScript/TypeScript
- **Browsers**: Modern browsers with WebCrypto API
- **Node.js**: All LTS versions (16+)
- **React/Vue/Angular**: Framework-agnostic
- **Next.js/Nuxt**: SSR and client-side support

### Python
- **Versions**: Python 3.8+
- **Frameworks**: Django, Flask, FastAPI, asyncio
- **Deployment**: Docker, Lambda, traditional servers

### Command Line
- **Cross-Platform**: Windows, macOS, Linux
- **CI/CD**: GitHub Actions, GitLab CI, Jenkins
- **Scripting**: Bash, PowerShell integration

## 📖 Common Use Cases

### Web Applications
```javascript
// Authenticate user requests to DataFold APIs
const client = new DataFoldClient(config);
const schemas = await client.get('/api/schemas');
```

### Backend Services
```python
# Server-to-server API authentication
from datafold_sdk import DataFoldClient

client = DataFoldClient(config)
result = client.post('/api/data/validate', data)
```

### CLI Operations
```bash
# Command-line data operations
datafold auth login --client-id my-service
datafold schema upload ./schema.json
```

### CI/CD Pipelines
```yaml
# GitHub Actions integration
- name: Validate schemas
  run: |
    datafold auth configure --key-file ${{ secrets.DATAFOLD_KEY }}
    datafold schema validate --all
```

## 🚦 API Status

All DataFold authentication APIs are production-ready:

- **Base URL**: `https://api.datafold.com`
- **API Version**: v1 (stable)
- **Rate Limits**: 1000 req/min per client
- **SLA**: 99.9% uptime
- **Support**: Enterprise support available

## 🔗 Quick Links

- **[5-Minute Setup Guide](authentication/getting-started.md)**
- **[JavaScript Examples](sdks/javascript/examples.md)**
- **[Python Examples](sdks/python/examples.md)**
- **[CLI Command Reference](sdks/cli/commands.md)**
- **[Security Best Practices](guides/security-best-practices.md)**
- **[Troubleshooting Guide](authentication/troubleshooting.md)**

## 📞 Support

Need help? We're here to assist:

- **Documentation Issues**: [Create an issue](https://github.com/datafold/docs/issues)
- **API Questions**: [Stack Overflow](https://stackoverflow.com/questions/tagged/datafold)
- **Enterprise Support**: [Contact Sales](mailto:sales@datafold.com)
- **Security Issues**: [Security Contact](mailto:security@datafold.com)

---

**Next Steps**: Start with the [Getting Started Guide](authentication/getting-started.md) or explore specific [SDK documentation](sdks/).