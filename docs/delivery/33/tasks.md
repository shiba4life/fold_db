# Tasks for PBI 33: Cryptography Module Consolidation

This document lists all tasks associated with PBI 33.

**Parent PBI**: [PBI 33: Cryptography Module Consolidation](./prd.md)

## Task Summary

| Task ID | Name | Status | Effort | Description |
| :------ | :--- | :----- | :----- | :---------- |
| 33-1    | [Inventory and Map Cryptography Functionality](./33-1.md) | Proposed | M | Catalog all cryptographic functions, types, and flows in both modules |
| 33-2    | [Define Unified Crypto Module Boundaries](./33-2.md) | Proposed | M | Design structure and interfaces for unified crypto module with security team review |
| 33-3    | [Refactor and Merge Cryptographic Primitives](./33-3.md) | Proposed | L | Move and unify all cryptographic primitives and utilities into new module |
| 33-4    | [Integrate Compliance and API Layers](./33-4.md) | Proposed | L | Integrate compliance, initialization, and API endpoints into unified module |
| 33-5    | [Remove Legacy Crypto Modules](./33-5.md) | Proposed | S | Remove old crypto modules and update all references |

## Task Details

### Task 33-1: Inventory and Map Cryptography Functionality
**Dependencies:** None  
**Acceptance Criteria:**
- Complete inventory and mapping document of all cryptographic functionality
- All public and internal APIs cataloged from both `src/crypto/` and `src/datafold_node/crypto/`
- Identified duplicated or similar logic between modules
- Security implications documented for each component

**Technical Tasks:**
- List all public and internal cryptographic APIs in both modules
- Identify duplicated or similar cryptographic logic
- Map data flows and dependencies between crypto components
- Document security properties and compliance requirements for each component
- Document findings in architecture wiki with security review

**Risks & Mitigation:** 
- Overlooked critical cryptographic code paths; mitigate with static analysis tools and security team review

### Task 33-2: Define Unified Crypto Module Boundaries
**Dependencies:** Task 33-1  
**Acceptance Criteria:**
- Approved design document and interface definitions reviewed by security team
- Clear module boundaries defined between primitives, compliance, and API layers
- API surface defined for core and compliance layers
- Migration strategy approved by security and compliance teams

**Technical Tasks:**
- Propose unified module boundaries and architecture
- Define API surface for core cryptographic primitives
- Define interfaces for compliance and operational layers
- Create migration strategy for existing crypto consumers
- Review design with security team and obtain approval

**Risks & Mitigation:** 
- Security concerns in unified design; mitigate with mandatory security architecture review

### Task 33-3: Refactor and Merge Cryptographic Primitives
**Dependencies:** Task 33-2  
**Acceptance Criteria:**
- All cryptographic operations use unified codebase
- Core primitives (key management, encryption, audit) successfully consolidated
- No duplicated cryptographic logic remains
- Security properties maintained through consolidation

**Technical Tasks:**
- Refactor and merge cryptographic primitives (key management, encryption, audit)
- Update all consumers to use unified cryptographic interfaces
- Remove redundant cryptographic code
- Validate security properties with comprehensive test coverage
- Conduct security review of consolidated primitives

**Risks & Mitigation:** 
- Security regression during consolidation; mitigate with extensive test coverage and security review

### Task 33-4: Integrate Compliance and API Layers
**Dependencies:** Task 33-3  
**Acceptance Criteria:**
- All operational and compliance features available in unified module
- API endpoints successfully migrated without breaking changes
- Compliance audit capabilities preserved
- Integration tests validate feature completeness

**Technical Tasks:**
- Move and adapt compliance and audit code to unified module
- Integrate initialization and operational crypto code
- Migrate API endpoints to use unified crypto module
- Update technical documentation for unified interfaces
- Validate compliance capabilities with audit team
- Validate with comprehensive integration tests

**Risks & Mitigation:** 
- API breakage during migration; mitigate with versioning strategy and detailed migration guides

### Task 33-5: Remove Legacy Crypto Modules
**Dependencies:** Task 33-4  
**Acceptance Criteria:**
- No references to legacy crypto modules remain in codebase
- Build and all tests pass successfully
- Security validation confirms no regression
- Documentation updated to reflect unified crypto module

**Technical Tasks:**
- Remove old crypto module files
- Update all imports and references to use unified crypto module
- Update security documentation and developer guides
- Conduct final security review of consolidated implementation
- Validate removal with CI/CD pipeline and security tests

**Risks & Mitigation:** 
- Missed references could break cryptographic functionality; mitigate with thorough code search, comprehensive testing, and security validation