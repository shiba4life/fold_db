# PBI-32: Transform Engine Consolidation

## Overview
This PBI consolidates transform execution and management logic from separate modules (`src/transform/` and `src/fold_db_core/transform_manager/`) into a unified transform engine to eliminate duplication and improve maintainability.

## Problem Statement
The current architecture has transform functionality split across two separate modules:
- `src/transform/` handles AST and execution logic
- `src/fold_db_core/transform_manager/` handles management, orchestration, and state

This split creates duplication, maintenance overhead, and integration complexity. A unified approach will provide better cohesion and reduce the cognitive load for developers working with transform functionality.

## User Stories
- As a system architect, I want transform execution and management logic consolidated so that developers have a single, coherent interface for all transform operations.
- As a developer, I want to work with transforms through a unified API rather than navigating between separate execution and management modules.
- As a maintainer, I want to eliminate duplicated transform logic to reduce the maintenance burden and potential for inconsistencies.

## Technical Approach
1. **Analysis Phase**: Map all transform-related functionality across both modules to identify overlap and unique responsibilities
2. **Design Phase**: Create unified architecture that merges DSL execution with transform management
3. **Implementation Phase**: Refactor code to eliminate duplication and merge into unified module
4. **Integration Phase**: Migrate management features (orchestration, registration, persistence, metrics)
5. **Cleanup Phase**: Remove legacy modules and update all references

## Architecture

### Current State
```mermaid
flowchart LR
    A[transform/ (AST, Executor)] -- DSL Execution --> B[fold_db_core/transform_manager/ (Manager, Orchestration, State)]
    B -- Uses --> A
```

### Target State
```mermaid
flowchart LR
    C[unified_transform/ (AST, Executor, Manager, Orchestration, State)]
```

## UX/UI Considerations
This is a backend refactoring effort with no direct UI impact. However, the unified API will provide a better developer experience for those working with transform functionality.

## Acceptance Criteria
- Transform execution logic unified from both source modules
- Single coherent module containing AST, execution, management, orchestration, and state functionality
- All transform flows use unified codebase with no duplicated logic
- Legacy transform modules completely removed with all references updated
- Comprehensive testing validates no functionality loss during consolidation
- Performance characteristics maintained or improved

## Dependencies
- None anticipated - this is primarily an internal refactoring

## Risk Assessment
- **Medium Risk**: Potential for regression during code consolidation
- **Mitigation**: Comprehensive test coverage and phased migration approach
- **Medium Risk**: Complex integration points between execution and management layers
- **Mitigation**: Thorough analysis phase and stakeholder review of unified design

## Open Questions
- Should the unified module maintain separate sub-modules for different concerns (execution vs management)?
- Are there performance considerations that require keeping certain components separate?
- What is the migration strategy for external consumers of the current APIs?

## Related Tasks
- [Task list for this PBI](./tasks.md)

[View in Backlog](../backlog.md#user-content-32)