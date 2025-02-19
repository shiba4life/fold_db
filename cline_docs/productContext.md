# Product Context

FoldDB is a database system that provides:
- Schema-based data storage with atomic operations
- Fine-grained permissions control at the field level
- Trust-based access control with explicit permissions and trust distance
- Version history tracking for data changes
- Pay-per-query access using Lightning Network
- Schema transformation and interpretation

## Problems Solved
- Granular data access control through permissions policies
- Atomic data versioning and history tracking
- Schema-based data validation and organization
- Flexible trust-based access model
- Monetization of data access through micropayments
- Schema transformation and migration
- Complex data access pricing based on trust relationships

## How It Works

### Core System
- Data is stored in Atoms which are immutable units containing content and metadata
- AtomRefs provide references to the latest version of data
- Schemas define the structure and permissions of data fields
- Permissions are controlled through:
  - Trust distance (lower means higher trust)
  - Explicit read/write policies per field
  - Public key based access control
- Payments are managed through:
  - Lightning Network integration
  - Per-field payment calculation
  - Trust distance based scaling
  - Hold invoices for complex operations
- Schema interpretation:
  - JSON-based schema definitions
  - Field-level configurations
  - Schema transformation rules
  - Validation constraints

## Key Features
1. Data Storage
   - Immutable versioning
   - Atomic operations
   - Schema validation

2. Access Control
   - Field-level permissions
   - Trust-based access
   - Public key authentication

3. Payment System
   - Lightning Network payments
   - Dynamic pricing based on trust
   - Hold invoice support
   - Payment verification

4. Schema Management
   - JSON schema definitions
   - Schema transformation
   - Field configurations
   - Validation rules
