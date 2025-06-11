# DataFold Signature Authentication - Complete Application Examples

This directory contains complete, working application examples that demonstrate real-world usage of DataFold signature authentication across different scenarios and technologies.

## Available Examples

### 📱 Basic Applications
- **[`simple-api-client/`](simple-api-client/)** - Simple authenticated API with client
- **[`web-app-auth/`](web-app-auth/)** - Web application with signature authentication

### 🛒 E-commerce & Business
- **[`ecommerce-checkout/`](ecommerce-checkout/)** - E-commerce checkout with signature authentication
- **[`multi-tenant-saas/`](multi-tenant-saas/)** - Multi-tenant SaaS application patterns

### 🔧 Infrastructure & Integration
- **[`microservices-auth/`](microservices-auth/)** - Microservices with service-to-service authentication
- **[`realtime-websocket/`](realtime-websocket/)** - Real-time applications with WebSocket authentication
- **[`mobile-backend/`](mobile-backend/)** - Mobile app backend with signature verification

### 🔒 Security & Compliance
- **[`enterprise-security/`](enterprise-security/)** - Enterprise-grade security patterns
- **[`audit-compliance/`](audit-compliance/)** - Compliance and audit trail examples

## Running the Examples

Each example includes:
- Complete source code
- Deployment instructions
- Security considerations
- Performance guidelines
- Testing procedures

### Prerequisites

- Node.js 18+ or Python 3.8+
- DataFold server running
- Valid authentication credentials

### Quick Start

1. Choose an example directory
2. Follow the README instructions in that directory
3. Configure your authentication credentials
4. Run the example application

## Security Best Practices

All examples demonstrate:
- ✅ Proper key generation and storage
- ✅ RFC 9421 compliant signatures
- ✅ Replay attack prevention
- ✅ Secure error handling
- ✅ Performance optimization
- ✅ Monitoring and logging

## Integration with DataFold SDKs

Examples leverage the official SDKs:
- **JavaScript/TypeScript**: Uses automatic signature injection
- **Python**: Uses enhanced HTTP client with signing
- **CLI**: Uses configurable authentication profiles

## Support

For questions about these examples:
1. Check the individual example READMEs
2. Review the [Security Recipes](../../docs/security/recipes/)
3. Consult the [Integration Guides](../../docs/guides/integration/)
4. See the [Troubleshooting Cookbook](../../docs/guides/integration/examples/integration-recipes/troubleshooting-cookbook.md)