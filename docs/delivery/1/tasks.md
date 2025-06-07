# Tasks for PBI 1: Create collection atom refs when schemas with collection fields are approved

This document lists all tasks associated with PBI 1.

**Parent PBI**: [PBI 1: Create collection atom refs when schemas with collection fields are approved](./prd.md)

## Task Summary

| Task ID | Name | Status | Description |
| :------ | :--- | :----- | :---------- |
| 1-1 | [Add CollectionField variant to FieldVariant enum](./1-1.md) | Review | Re-introduce the Collection variant to the FieldVariant enum to support collection fields |
| 1-2 | [Update map_fields to handle collection fields](./1-2.md) | Review | Modify the map_fields function to create AtomRefCollection for collection fields during schema approval |
| 1-3 | [Update convert_field to handle Collection field type](./1-3.md) | Review | Modify convert_field function to create CollectionField instances from JSON schema definitions |
| 1-4 | [Add unit tests for collection atom ref creation](./1-4.md) | Proposed | Create comprehensive unit tests to verify collection atom ref creation during schema approval |
| 1-5 | [Add integration test for collection field schema approval](./1-5.md) | Proposed | Create end-to-end integration test for approving schemas with collection fields |