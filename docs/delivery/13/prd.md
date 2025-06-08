# PBI-13: Enhanced HTTP Security APIs

[View in Backlog](../backlog.md#user-content-13)

## Overview

This PBI implements comprehensive HTTP API endpoints for secure DataFold database setup and management, enabling programmatic initialization and configuration of all security features. The enhanced APIs provide complete cryptographic setup workflows accessible via HTTP interfaces.

## Problem Statement

Current DataFold HTTP APIs lack comprehensive security configuration capabilities, creating gaps in automated deployment and management:

- No HTTP endpoints for cryptographic initialization
- Limited programmatic access to security configuration
- Missing client key registration APIs
- No secure schema creation with field-level permissions
- Insufficient security status and monitoring endpoints

## User Stories

**Primary User Story:**
As a system integrator, I want enhanced HTTP API endpoints for secure database setup so that I can initialize and configure DataFold security features programmatically.

**Supporting User Stories:**
- As a DevOps engineer, I want automated security setup for infrastructure-as-code deployments
- As a security administrator, I want programmatic client key management capabilities
- As a system operator, I want HTTP APIs for security monitoring and status checking
- As an application developer, I want consistent API patterns for all security operations
- As a deployment engineer, I want scriptable security configuration workflows

## Technical Approach

### 1. Cryptographic Initialization APIs

#### Database Crypto Setup
- `POST /api/crypto/init` - Initialize database with master key encryption
- `GET /api/crypto/status` - Check cryptographic configuration status
- `POST /api/crypto/rotate-keys` - Rotate database master keys
- `GET /api/crypto/health` - Cryptographic health monitoring

#### Client Key Management
- `POST /api/crypto/client/register` - Register client public keys
- `GET /api/crypto/client/status/{key_id}` - Check client key status
- `POST /api/crypto/client/revoke` - Revoke client access keys
- `GET /api/crypto/client/list` - List registered client keys

### 2. Enhanced Schema Management

#### Secure Schema Creation
- `POST /api/schema/secure` - Create schemas with encryption and permissions
- `PUT /api/schema/{name}/security` - Update security settings for existing schemas
- `GET /api/schema/{name}/permissions` - Retrieve schema permission configuration
- `POST /api/schema/{name}/encrypt` - Migrate schema to encrypted storage

#### Permission Management
- `POST /api/permissions/set` - Set field-level permissions
- `GET /api/permissions/audit` - Retrieve permission audit logs
- `DELETE /api/permissions/revoke` - Revoke specific permissions
- `POST /api/permissions/bulk` - Bulk permission operations

### 3. Network Security Configuration

#### Network Setup
- `POST /api/network/secure/init` - Initialize network with enhanced security
- `PUT /api/network/security/config` - Update network security settings
- `GET /api/network/security/status` - Check network security status
- `POST /api/network/peers/trust` - Manage trusted peer relationships

#### Security Monitoring
- `GET /api/security/events` - Retrieve security event logs
- `GET /api/security/metrics` - Security performance metrics
- `POST /api/security/alerts` - Configure security alerting
- `GET /api/security/audit` - Comprehensive security audit reports

### 4. Setup and Configuration Workflows

#### Automated Setup Scripts
- Complete setup workflow via single API call
- Idempotent operations for reliable deployment
- Configuration validation and verification
- Rollback mechanisms for failed setups

#### Status and Health Checking
- Comprehensive security status endpoints
- Health checks for all security components
- Configuration drift detection
- Security compliance reporting

## UX/UI Considerations

### API Design Principles
- RESTful design patterns for consistency
- Clear, descriptive error messages with actionable guidance
- Comprehensive request/response documentation
- Standardized authentication across all endpoints

### Developer Experience
- OpenAPI/Swagger documentation for all endpoints
- SDK generation for major programming languages
- Interactive API documentation and testing
- Example scripts and automation templates

### Administrative Interface
- Web-based API explorer for testing and debugging
- Security configuration wizards
- Visual status dashboards for security components
- Export/import capabilities for security configurations

## Acceptance Criteria

1. **Cryptographic APIs**
   - ✅ Complete crypto initialization via HTTP API
   - ✅ Client key registration and management endpoints
   - ✅ Key rotation and lifecycle management APIs
   - ✅ Comprehensive crypto status and health checking

2. **Schema Security APIs**
   - ✅ Secure schema creation with encryption and permissions
   - ✅ Field-level permission management via API
   - ✅ Schema migration to encrypted storage
   - ✅ Permission audit and compliance reporting

3. **Network Security APIs**
   - ✅ Network security initialization and configuration
   - ✅ Trusted peer management via API
   - ✅ Security event monitoring and alerting
   - ✅ Network security status and health checks

4. **API Quality and Documentation**
   - ✅ OpenAPI 3.0 specification for all endpoints
   - ✅ Comprehensive error handling and status codes
   - ✅ Rate limiting and authentication for all endpoints
   - ✅ Interactive documentation and testing interface

5. **Security Requirements**
   - ✅ All security APIs require proper authentication
   - ✅ Audit logging for all security configuration changes
   - ✅ Input validation and sanitization for all parameters
   - ✅ Protection against API abuse and unauthorized access

6. **Integration and Automation**
   - ✅ Idempotent operations for reliable automation
   - ✅ Bulk operations for enterprise-scale deployments
   - ✅ Configuration export/import for backup and migration
   - ✅ Integration examples for popular automation tools

## Dependencies

- **Internal**: 
  - PBI-8 (Database Master Key Encryption) - Required for crypto APIs
  - PBI-10 (Client-Side Key Management) - Required for client key APIs
  - PBI-11 (Signed Message Authentication) - Required for API authentication
  - Existing HTTP server infrastructure
- **External**: 
  - HTTP framework with middleware support
  - OpenAPI documentation generation tools
  - Rate limiting and authentication middleware
- **Documentation**: API documentation system and interactive testing tools

## Open Questions

1. **API Versioning**: How should API versions be managed as security features evolve?
2. **Batch Operations**: What level of batch operations should be supported for enterprise deployments?
3. **Webhook Integration**: Should security events support webhook notifications for external systems?
4. **API Rate Limiting**: What are appropriate rate limits for security-sensitive operations?
5. **Emergency Procedures**: Should there be special emergency APIs for security incident response?

## Related Tasks

Tasks will be created in [tasks.md](./tasks.md) upon PBI approval. 