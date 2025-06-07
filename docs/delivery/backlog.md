# Product Backlog

This document contains all Product Backlog Items (PBIs) for the project, ordered by priority.

## Backlog

| ID | Actor | User Story | Status | Conditions of Satisfaction (CoS) |
|----|-------|------------|--------|----------------------------------|
| 1 | Developer | As a developer, I want collection atom refs to be automatically created when schemas with collection field types become approved, so that collection fields can properly store and reference data | InProgress | 1. When a schema containing collection field types is approved, collection atom refs are automatically created for each collection field<br>2. The created collection atom refs are properly stored in the database<br>3. The schema fields are updated with the correct ref_atom_uuid<br>4. The system handles both new schemas and existing schemas being approved<br>5. Error handling is in place for atom ref creation failures |

## History

| Timestamp | PBI_ID | Event Type | Details | User |
|-----------|--------|------------|---------|------|
| 20250119-120000 | 1 | create_pbi | Created PBI for collection atom ref creation on schema approval | User |
| 20250119-120100 | 1 | propose_for_backlog | PBI approved and moved to Agreed status | User |
| 20250119-121000 | 1 | start_implementation | Created tasks and started implementation | User |