# Active Context

## Current Task
Creating a permissions check wrapper for query and mutation operations to:
- Centralize permissions logic
- Ensure consistent permission checks
- Improve code maintainability

## Recent Changes
- Initial project setup with core components
- Basic permission system implementation
- Schema and atom storage system

## Next Steps
1. Create a permission wrapper module
2. Implement wrapper for query operations
3. Implement wrapper for mutation operations
4. Update FoldDB to use the wrapper
5. Add tests for the new wrapper functionality

## Implementation Plan
1. Create new permission wrapper in permissions module
2. Move permission check logic from FoldDB to wrapper
3. Add wrapper methods for query and mutation operations
4. Update FoldDB to use wrapper methods
5. Ensure all permission checks are consistent
