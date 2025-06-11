# DataFold Signature Authentication - Code Snippet Library

This library provides reusable, copy-paste ready code snippets for common DataFold signature authentication scenarios across multiple programming languages and frameworks.

## 📚 Snippet Categories

### 🔑 Basic Authentication
- **[Key Generation](basic/key-generation.md)** - Generate Ed25519 keypairs
- **[Signature Creation](basic/signature-creation.md)** - Create RFC 9421 signatures
- **[Signature Verification](basic/signature-verification.md)** - Verify incoming signatures
- **[Request Signing](basic/request-signing.md)** - Sign HTTP requests

### 🌐 Framework Integration
- **[Express.js](frameworks/express.md)** - Node.js Express middleware
- **[FastAPI](frameworks/fastapi.md)** - Python FastAPI integration
- **[Spring Boot](frameworks/spring-boot.md)** - Java Spring Boot setup
- **[Django](frameworks/django.md)** - Django middleware patterns
- **[Next.js](frameworks/nextjs.md)** - React/Next.js client-side auth
- **[Flask](frameworks/flask.md)** - Python Flask decorators

### ⚠️ Error Handling
- **[Authentication Errors](error-handling/auth-errors.md)** - Handle auth failures gracefully
- **[Network Errors](error-handling/network-errors.md)** - Retry and recovery patterns
- **[Validation Errors](error-handling/validation-errors.md)** - Input validation failures
- **[Performance Errors](error-handling/performance-errors.md)** - Timeout and performance issues

### 🔧 Edge Cases
- **[Clock Skew](edge-cases/clock-skew.md)** - Handle time synchronization issues
- **[Key Rotation](edge-cases/key-rotation.md)** - Seamless key rotation
- **[Large Payloads](edge-cases/large-payloads.md)** - Handle large request bodies
- **[Concurrent Requests](edge-cases/concurrent-requests.md)** - Thread-safe implementations

### 🚀 Performance Optimization
- **[Signature Caching](performance/signature-caching.md)** - Cache signatures securely
- **[Connection Pooling](performance/connection-pooling.md)** - Optimize HTTP connections
- **[Batch Operations](performance/batch-operations.md)** - Bulk request signing
- **[Memory Management](performance/memory-management.md)** - Efficient resource usage

### 🔒 Security Patterns
- **[Replay Prevention](security/replay-prevention.md)** - Prevent replay attacks
- **[Nonce Generation](security/nonce-generation.md)** - Secure nonce creation
- **[Timing Attack Prevention](security/timing-attacks.md)** - Constant-time comparisons
- **[Secure Storage](security/secure-storage.md)** - Store keys securely

## 🎯 Quick Reference

### JavaScript/TypeScript
```typescript
// Basic signature creation
import { RFC9421Signer } from '@datafold/signature-auth';

const signer = new RFC9421Signer({
  keyId: 'my-key',
  privateKey: keypair.privateKey
});

const signature = await signer.sign(request);
```

### Python
```python
# Basic signature creation
from datafold_sdk.signing import RFC9421Signer

signer = RFC9421Signer(
    key_id='my-key',
    private_key=keypair.private_key
)

signature = await signer.sign(request)
```

### Rust/CLI
```rust
// Basic signature creation
use datafold::crypto::ed25519::MasterKeyPair;
use datafold::cli::auth::CliRequestSigner;

let signer = CliRequestSigner::new(keypair, "my-key".to_string())?;
let signed_request = signer.sign_request(request).await?;
```

## 📋 Snippet Format

Each snippet follows this structure:

```markdown
# Snippet Title

## Overview
Brief description and use case

## Code
```language
// Complete, working code example
```

## Usage
How to integrate and use the snippet

## Security Notes
Important security considerations

## Testing
How to test the implementation

## Variations
Alternative implementations or configurations
```

## 🔄 Language Support

### Primary Languages
- **JavaScript/TypeScript** - Full SDK support with automatic signing
- **Python** - Enhanced HTTP client with comprehensive features
- **Rust** - CLI and server implementations

### Framework Integrations
- **Node.js**: Express, Fastify, NestJS, Next.js
- **Python**: FastAPI, Django, Flask, Starlette
- **Java**: Spring Boot, Quarkus, Micronaut
- **Go**: Gin, Echo, Chi, net/http
- **PHP**: Laravel, Symfony, Slim
- **Ruby**: Rails, Sinatra, Grape

## 🛠️ Development Tools

### Testing Utilities
- Signature validation helpers
- Mock server implementations
- Test vector generators
- Performance benchmarking tools

### Debugging Tools
- Signature inspection utilities
- Request/response analyzers
- Performance profilers
- Security audit helpers

### IDE Extensions
- VS Code signature helpers
- Syntax highlighting for configs
- Auto-completion for APIs
- Error detection and suggestions

## 📊 Complexity Levels

### 🟢 Basic (5-10 lines)
Simple, straightforward implementations for common tasks

### 🟡 Intermediate (10-50 lines)
More comprehensive examples with error handling

### 🔴 Advanced (50+ lines)
Complex scenarios with optimization and security features

## 🔍 Search and Navigation

### By Use Case
- Authentication setup
- Request signing
- Response verification
- Error handling
- Performance optimization
- Security hardening

### By Technology
- Frontend frameworks
- Backend frameworks
- Mobile development
- CLI tools
- Infrastructure code

### By Security Level
- Basic security
- Production-ready
- Enterprise-grade
- Compliance-focused

## 🧪 Testing Integration

### Unit Tests
Every snippet includes corresponding unit tests

### Integration Tests
Examples include end-to-end testing scenarios

### Performance Tests
Benchmarking code for performance-critical snippets

### Security Tests
Security validation for all authentication flows

## 📖 Documentation Standards

### Code Comments
- Clear, descriptive comments
- Security considerations highlighted
- Performance notes included
- Error handling explained

### Examples
- Multiple usage scenarios
- Configuration variations
- Integration patterns
- Best practices

### References
- Links to official documentation
- Related security recipes
- Framework-specific guides
- Performance optimization tips

## 🤝 Contributing

### Adding New Snippets
1. Follow the snippet format template
2. Include comprehensive testing
3. Add security analysis
4. Provide multiple language examples when possible
5. Submit for review

### Improving Existing Snippets
1. Performance optimizations
2. Security enhancements
3. Better error handling
4. Additional language support
5. Documentation improvements

## 📄 License

All code snippets are provided under the MIT license for maximum reusability.

---

**Next Steps**: Browse the categories above or jump to the [Quick Start Guide](quick-start.md) for immediate implementation.