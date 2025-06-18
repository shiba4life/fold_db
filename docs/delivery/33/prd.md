# PBI-33: Cryptography Module Consolidation

## Overview
This PBI consolidates cryptographic primitives and operational logic from separate modules (`src/crypto/` and `src/datafold_node/crypto/`) into a single, authoritative crypto module with clear separation between core logic and API/compliance layers.

## Problem Statement
The current architecture has cryptographic functionality split across two modules:
- `src/crypto/` contains core primitives, audit functionality, and monitoring
- `src/datafold_node/crypto/` handles compliance, API endpoints, initialization, and encryption operations

This separation creates potential for duplication, inconsistent security implementations, and maintenance overhead. A unified approach will ensure all cryptographic operations use consistent, audited implementations while maintaining clear architectural boundaries.

## User Stories
- As a security architect, I want all cryptographic functionality consolidated into a single authoritative module so that security implementations are consistent and auditable across the entire system.
- As a developer, I want to access cryptographic functions through a unified interface rather than navigating between core primitives and operational implementations.
- As a compliance officer, I want clear separation between cryptographic primitives and compliance/API layers while ensuring all operations use the same underlying security implementations.

## Technical Approach
1. **Inventory Phase**: Catalog all cryptographic functions, types, and flows in both modules
2. **Design Phase**: Define unified module structure with clear boundaries between core, compliance, and API layers
3. **Refactoring Phase**: Merge cryptographic primitives and eliminate duplication
4. **Integration Phase**: Integrate compliance, API, and operational layers into unified module
5. **Security Review**: Validate that consolidation maintains security properties
6. **Cleanup Phase**: Remove legacy modules and update all references

## Architecture

### Current State
```mermaid
flowchart LR
    A[crypto/ (Primitives, Audit, Monitoring)] -- Used by --> B[datafold_node/crypto/ (Compliance, API, Init, Encryption)]
```

### Target State
```mermaid
flowchart LR
    C[unified_crypto/ (Primitives, Audit, Monitoring, Compliance, API, Encryption)]
```

## Security Considerations
- All cryptographic operations must maintain current security properties
- No regression in cryptographic strength or implementation quality
- Security review required before legacy module removal
- Audit trail must be preserved for compliance purposes
- Key management and encryption operations must remain secure during transition

## UX/UI Considerations
This is primarily a backend security refactoring with no direct UI impact. However, API consumers will benefit from more consistent cryptographic interfaces.

## Acceptance Criteria
- Cryptographic functionality unified from both source modules into single authoritative location
- Clear architectural boundaries maintained between primitives, audit, monitoring, compliance, and API layers
- All cryptographic operations use unified codebase with no duplicated security logic
- Security review validates no regression in cryptographic security or compliance
- Legacy crypto modules completely removed with all references updated
- Performance characteristics of cryptographic operations maintained or improved

## Dependencies
- Security team review and approval of unified architecture
- Compliance validation of consolidated audit and monitoring capabilities

## Risk Assessment
- **High Risk**: Security regression during cryptographic code consolidation
- **Mitigation**: Mandatory security review, comprehensive test coverage, and phased migration
- **Medium Risk**: API breakage for existing cryptographic consumers
- **Mitigation**: Versioning strategy and detailed migration guides
- **Medium Risk**: Compliance audit trail disruption
- **Mitigation**: Careful preservation of audit functionality and validation with compliance team

## Open Questions
- Should cryptographic primitives remain as a separate sub-module within the unified structure?
- What is the migration strategy for external systems that depend on current crypto APIs?
- Are there performance considerations that require maintaining separation between certain crypto components?
- How should we handle backward compatibility for existing cryptographic interfaces?

## Related Tasks
- [Task list for this PBI](./tasks.md)

[View in Backlog](../backlog.md#user-content-33)