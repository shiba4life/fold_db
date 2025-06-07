# Executable Memory & Indexing Transform Implementation Plan

## Overview

This document outlines the plan to implement Executable Memory in FoldDB, starting with a new class of indexing transforms. These transforms enable dynamic, code-driven indexing and composable queries, allowing the system to evolve indexing and sorting logic over time without requiring direct database changes. This is the first step toward a fully programmable, self-improving data layer.

## Motivation

- **Dynamic Indexing**: Allow transforms to create and maintain indexes based on schema.field changes, using flexible matching (including regex and wildcards).
- **Composable Queries**: Enable queries that resolve via index transforms, supporting patterns like `{Q: indexSchema.field[token]}` that map tokens to schema.field routes.
- **Evolvable Logic**: Since transforms are sandboxed code, indexing and sorting strategies can be improved or replaced over time without DB migrations or manual intervention.
- **Executable Memory**: This approach treats transforms as executable memory, where the logic for data access and organization is itself stored and versioned in the database.

## Requirements

1. **Transform Matching**
    - Transforms can match on schema.field changes using regex or wildcard patterns (e.g., `*.*`).
    - Support for registering transforms that listen to all changes or specific patterns.
2. **Indexing Transform**
    - A special transform type that, when triggered, creates or updates an `indexSchema` mapping tokens to schema.field routes.
    - Indexes are themselves stored as schema data, accessible via queries.
3. **Composable Query Resolution**
    - Queries can reference index transforms, e.g., `{Q: indexSchema.field[token]}`.
    - The system resolves the inner block to a schema.field route, then executes the outer query using the resolved route.
4. **Sandboxed Execution**
    - All transform logic runs in a secure, sandboxed environment.
    - Indexing/sorting logic can be updated by deploying new transform code.
5. **Documentation & API**
    - Update [transforms.md](./transforms.md) and [api-reference.md](./api-reference.md) to document new transform types, registration, and query patterns.
    - Provide usage examples and migration guidance.

## Technical Design

### 1. Transform Matching Engine
- Extend the transform registration system to support regex/wildcard patterns for schema.field triggers.
- Example: Register a transform with `inputs: ["*.*"]` to listen to all field changes.

### 2. Indexing Transform
- Define a new transform type (or convention) for indexing transforms.
- When triggered, the transform updates an `indexSchema` (e.g., `TokenIndex`) with mappings: `token -> schema.field route`.
- Indexes are stored as regular schema data, making them queryable and updatable by transforms.

### 3. Composable Query Resolution
- Extend the query engine to support nested/composable queries:
    - Parse queries like `{Q: indexSchema.field[token]}`.
    - Resolve the inner block to a schema.field route using the index.
    - Execute the outer query using the resolved route.
- Ensure this is documented in [api-reference.md](./api-reference.md).

### 4. Executable Memory
- Treat transforms (including indexing logic) as first-class, versioned code artifacts in the database.
- Allow updating/replacing indexing transforms without DB migrations.
- Ensure all transform execution is sandboxed and auditable.

### 5. API & CLI Extensions
- Update HTTP and CLI APIs to support:
    - Registering transforms with pattern-based triggers
    - Querying and managing index schemas
    - Composable query syntax
- Document all changes and provide migration/usage examples.

## Implementation Steps

1. **Design & Spec**
    - Finalize transform matching syntax (regex/wildcard support)
    - Define index schema conventions (e.g., `TokenIndex`)
    - Specify composable query syntax and resolution flow
2. **Transform Engine Update**
    - Implement pattern-based transform triggers
    - Add support for indexing transforms
3. **Index Schema Implementation**
    - Create schema(s) for storing indexes (token -> route)
    - Ensure transforms can write to these schemas
4. **Composable Query Engine**
    - Update query parser and executor to support nested queries
    - Implement resolution logic for index-based lookups
5. **API & CLI Update**
    - Extend endpoints/commands for new transform and query features
    - Add documentation and usage examples
6. **Testing & Validation**
    - Unit and integration tests for transform matching, indexing, and query resolution
    - E2E tests for real-world scenarios
7. **Documentation**
    - Update [transforms.md](./transforms.md), [api-reference.md](./api-reference.md), and add examples to [use-cases.md](./use-cases.md)

## Open Questions

- What is the best convention for naming and structuring index schemas?
- Should indexing transforms be a special type, or just a convention?
- How should versioning and migration of index transforms be managed?
- What are the security/audit requirements for executable memory?
- How should errors in indexing transforms be surfaced to users?

---

**References:**
- [Transform System and DSL](./transforms.md)
- [API Reference](./api-reference.md)
- [Architecture](./architecture.md)
- [Use Cases](./use-cases.md) 