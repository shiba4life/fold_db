# PBI-34: Network Layer Consolidation

## Overview
This PBI unifies network abstraction and node transport logic from separate modules (`src/network/` and `src/datafold_node/transport/`) into a single, maintainable network module to eliminate duplication and improve consistency across network operations.

## Problem Statement
The current architecture has network functionality split across two modules:
- `src/network/` provides core networking, configuration, discovery, and propagation logic
- `src/datafold_node/transport/` handles API endpoints, TCP transport, and routing implementation

This separation creates duplication in network handling, inconsistent patterns between core networking and transport implementation, and maintenance overhead. A unified approach will provide consistent network operations and reduce complexity for developers working with networking functionality.

## User Stories
- As a DevOps engineer, I want all network operations to use consistent interfaces and implementations so that deployment and troubleshooting are simplified across the entire system.
- As a developer, I want to work with network functionality through a unified module rather than navigating between core networking and transport implementations.
- As a system architect, I want to eliminate duplicated network logic to reduce maintenance burden and ensure consistent behavior across all network operations.

## Technical Approach
1. **Analysis Phase**: Review both modules to identify duplicated logic and integration points
2. **Design Phase**: Propose unified architecture for consolidated network layer
3. **Core Integration**: Refactor and merge network core and transport logic
4. **Feature Migration**: Integrate node-specific API endpoints and routing into unified module
5. **Validation Phase**: Ensure all network functionality works seamlessly in unified structure
6. **Cleanup Phase**: Remove legacy modules and update all references

## Architecture

### Current State
```mermaid
flowchart LR
    A[network/ (Config, Core, Discovery, Propagation)] -- Used by --> B[datafold_node/transport/ (API, TCP, Routing)]
```

### Target State
```mermaid
flowchart LR
    C[unified_network/ (Config, Core, Discovery, API, TCP, Routing, Propagation)]
```

## Performance Considerations
- Network operations must maintain current performance characteristics
- TCP transport efficiency should be preserved or improved
- Discovery and propagation mechanisms must remain responsive
- API endpoint performance should not degrade during consolidation

## UX/UI Considerations
This is primarily a backend networking refactoring with no direct UI impact. However, network API consumers will benefit from more consistent interfaces and improved error handling.

## Acceptance Criteria
- Network logic unified from both source modules into single maintainable module
- Single module containing config, core, discovery, API, TCP, routing, and propagation functionality
- All network flows use unified codebase with no duplicated logic
- Node-specific features (API endpoints, routing) integrated without loss of functionality
- Legacy network modules completely removed with all references updated
- Network performance characteristics maintained or improved

## Dependencies
- DevOps team validation of network configuration changes
- Integration testing with existing network consumers

## Risk Assessment
- **Medium Risk**: Network functionality regression during consolidation
- **Mitigation**: Comprehensive integration testing and phased migration approach
- **Medium Risk**: Performance degradation in network operations
- **Mitigation**: Performance benchmarking throughout migration process
- **Low Risk**: API endpoint disruption during transport layer migration
- **Mitigation**: Careful API preservation and versioning strategy

## Open Questions
- Should the unified module maintain separate sub-modules for different network concerns (core vs transport)?
- Are there performance considerations that require keeping certain network components separate?
- What is the migration strategy for systems that depend on current network APIs?
- How should we handle backward compatibility for existing network interfaces?

## Related Tasks
- [Task list for this PBI](./tasks.md)

[View in Backlog](../backlog.md#user-content-34)