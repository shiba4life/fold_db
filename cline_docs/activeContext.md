# Active Context

## Current Task
Simplifying the project architecture by unifying components and streamlining error handling.

## Recent Changes
1. Created a new `SchemaCore` class that combines functionality from:
   - SchemaManager (schema storage, persistence, field mapping)
   - SchemaInterpreter (schema validation, JSON parsing)
   - SchemaValidator (validation logic)

2. Updated all references to SchemaManager and SchemaInterpreter to use SchemaCore instead:
   - Updated imports in various files
   - Modified method calls to use the new unified API
   - Updated tests to use the new component

3. Removed unused files:
   - Deleted src/schema/schema_manager.rs
   - Deleted src/schema_interpreter directory and its contents
   - Deleted src/schema/loader.rs (functionality duplicated in src/datafold_node/loader.rs)

4. Implemented a unified error handling system:
   - Created a centralized `FoldDbError` type in src/error.rs
   - Defined specific error categories (Schema, Database, Network, etc.)
   - Removed deprecated error types (NodeError, NetworkError)
   - Updated all modules to use the new error system directly
   - Eliminated backward compatibility code for cleaner implementation

5. Updated system documentation to reflect the simplified architecture

## Next Steps
1. ✅ Continue streamlining the network layer
   - ✅ Break down the NetworkManager into smaller components
   - ✅ Simplify message handling logic
   - ✅ Improve error recovery mechanisms
   - ✅ Unify client and server components (QueryService, SchemaService)
   - ✅ Simplify NetworkCore to use unified services

2. Evaluate if the permission system could benefit from similar simplification
   - Look for opportunities to reduce duplication in permission checking logic
   - Consider unifying trust distance and explicit permission checks

3. ✅ Fix remaining warnings in the code

4. Add tests for the new unified error handling system

5. Add tests for the new network layer components

6. ✅ Remove default schema loading
   - ✅ Removed hardcoded loading of UserProfile schema from config/schema.json
   - ✅ Now only previously loaded schemas are available on startup
   - ✅ Schemas are automatically loaded from disk during FoldDB initialization
