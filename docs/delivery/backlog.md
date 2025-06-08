# DataFold Product Backlog

This document contains all Product Backlog Items (PBIs) for the DataFold project, ordered by priority.

## PBI Table

| ID | Actor | User Story | Status | Conditions of Satisfaction (CoS) |
|----|--------|------------|--------|-----------------------------------|
| 8 | Database Administrator | As a database administrator, I want to initialize DataFold databases with master key encryption so that all data stored is protected by cryptographic security from the start | Done | • Database can be initialized with Ed25519 master key pair<br/>• Master private key is derived from user passphrase using secure key derivation<br/>• Database initialization creates encrypted storage layer<br/>• Public key is stored for verification purposes<br/>• Private key never stored in plaintext<br/>• [View Details](./8/prd.md) |
| 9 | DataFold Node Operator | As a node operator, I want all database files to be encrypted at rest using AES-256-GCM so that physical access to storage doesn't compromise data confidentiality | Done | • All atom data encrypted before writing to disk<br/>• Encryption uses AES-256-GCM with secure nonces<br/>• Keys derived from master key pair and passphrase<br/>• Decryption transparent during normal operations<br/>• Backup/restore maintains encryption<br/>• Performance impact < 20% for typical operations<br/>• [View Details](./9/prd.md) |
| 10 | Client Application User | As a client user, I want to generate and manage my own Ed25519 key pairs so that I have full control over my cryptographic identity without trusting the server | Proposed | • Client-side key generation using cryptographically secure methods<br/>• Private keys never transmitted to server<br/>• Public key registration with server for access control<br/>• Key backup and recovery mechanisms<br/>• Multi-platform client library support (JS, Python, CLI)<br/>• [View Details](./10/prd.md) |
| 11 | API Consumer | As an API consumer, I want all network requests to require cryptographic signatures so that only authorized clients can access data and all operations are authenticated | Proposed | • All API requests require Ed25519 signature verification<br/>• Timestamp and nonce protection against replay attacks<br/>• Signature verification before any data access<br/>• Clear error messages for authentication failures<br/>• Graceful handling of signature verification<br/>• [View Details](./11/prd.md) |
| 12 | Client Application User | As a client user, I want to rotate my access keys without service interruption so that I can maintain security hygiene through regular key updates | Proposed | • Client-initiated key replacement process<br/>• Atomic key replacement (old key invalidated, new key activated)<br/>• Signed replacement requests proving ownership<br/>• Immediate propagation across distributed network<br/>• No service downtime during key replacement<br/>• [View Details](./12/prd.md) |
| 13 | System Integrator | As a system integrator, I want enhanced HTTP API endpoints for secure database setup so that I can initialize and configure DataFold security features programmatically | Proposed | • HTTP endpoints for crypto initialization<br/>• Client key registration endpoints<br/>• Secure schema creation with field-level permissions<br/>• Network security configuration APIs<br/>• Setup validation and status checking<br/>• [View Details](./13/prd.md) |
| 14 | Security Administrator | As a security administrator, I want comprehensive key lifecycle management so that I can monitor, audit, and manage all cryptographic operations in the system | Proposed | • Key registration and revocation audit logs<br/>• Network-wide key status synchronization<br/>• Emergency key revocation capabilities<br/>• Key usage analytics and monitoring<br/>• Compliance reporting for key operations<br/>• [View Details](./14/prd.md) |

## PBI History

| Timestamp | PBI_ID | Event_Type | Details | User |
|-----------|--------|------------|---------|------|
| 20241230-120000 | 8 | create_pbi | Database master key encryption PBI created | system |
| 20241230-120001 | 9 | create_pbi | Encryption at rest PBI created | system |
| 20241230-120002 | 10 | create_pbi | Client-side key management PBI created | system |
| 20241230-120003 | 11 | create_pbi | Signed message authentication PBI created | system |
| 20241230-120004 | 12 | create_pbi | Key rotation and replacement PBI created | system |
| 20241230-120005 | 13 | create_pbi | Enhanced HTTP security APIs PBI created | system |
| 20241230-120006 | 14 | create_pbi | Key lifecycle management PBI created | system |
| 20241230-130000 | 8 | propose_for_backlog | PBI 8 status changed from Proposed to Agreed | tomtang |
| 20250608-093000 | 8 | complete | PBI 8 status changed from Agreed to Done | tomtang |
| 20250608-093038 | 9 | propose_for_backlog | PBI 9 status changed from Proposed to Agreed | tomtang |
| 20250608-111043 | 9 | complete | PBI 9 status changed from Agreed to Done | tomtang |