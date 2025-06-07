# PBI-1: Create collection atom refs when schemas with collection fields are approved

## Overview

This PBI addresses the need to automatically create collection atom refs when schemas containing collection field types are approved. Currently, the system creates atom refs for Single and Range field types during schema approval, but collection fields are not handled, despite `AtomRefCollection` infrastructure existing in the codebase.

[View in Backlog](../backlog.md#user-content-1)

## Problem Statement

When schemas with collection field types are approved, no atom refs are created for those fields. This prevents collection fields from functioning properly as they have no way to store references to their data atoms. The `map_fields` function in `src/schema/core.rs` has a TODO comment indicating that collection fields are no longer supported, but the documentation still lists Collection as a valid field type.

## User Stories

As a developer, I want collection atom refs to be automatically created when schemas with collection field types become approved, so that collection fields can properly store and reference data.

## Technical Approach

1. **Extend map_fields function**: Modify the `map_fields` function in `src/schema/core.rs` to handle collection field types
2. **Create AtomRefCollection instances**: When a collection field is detected, create an `AtomRefCollection` instance
3. **Store in database**: Persist the collection atom ref using the existing database operations
4. **Update field reference**: Set the `ref_atom_uuid` on the collection field
5. **Handle edge cases**: Ensure proper error handling and support for both new and existing schemas

## UX/UI Considerations

This is a backend change with no direct UI impact. However, it enables collection fields to work properly, which will allow users to:
- Create schemas with collection fields via the UI
- Store array data in collection fields
- Query and mutate collection field data

## Acceptance Criteria

1. When a schema containing collection field types is approved, collection atom refs are automatically created for each collection field
2. The created collection atom refs are properly stored in the database with the key format `ref:{uuid}`
3. The schema fields are updated with the correct `ref_atom_uuid` pointing to the created collection atom ref
4. The system handles both new schemas and existing schemas being approved
5. Error handling is in place for atom ref creation failures with appropriate logging
6. Unit tests verify the collection atom ref creation process
7. Integration tests confirm end-to-end functionality

## Dependencies

- Existing `AtomRefCollection` class in `src/atom/atom_ref_collection.rs`
- Database operations for storing atom refs
- Schema approval workflow

## Open Questions

1. Should we reinstate full collection field support or just handle atom ref creation?
2. Are there any migration concerns for existing schemas with collection fields?
3. Should collection operations (add/remove/update) be re-enabled in the mutation system?

## Related Tasks

See [Tasks for PBI 1](./tasks.md) for the implementation breakdown.