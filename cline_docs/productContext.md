# Product Context

FoldDB is a database system that provides:
- Schema-based data storage with atomic operations
- Fine-grained permissions control at the field level
- Trust-based access control with explicit permissions and trust distance
- Version history tracking for data changes

## Problems Solved
- Granular data access control through permissions policies
- Atomic data versioning and history tracking
- Schema-based data validation and organization
- Flexible trust-based access model

## How It Works
- Data is stored in Atoms which are immutable units containing content and metadata
- AtomRefs provide references to the latest version of data
- Schemas define the structure and permissions of data fields
- Permissions are controlled through:
  - Trust distance (lower means higher trust)
  - Explicit read/write policies per field
  - Public key based access control
