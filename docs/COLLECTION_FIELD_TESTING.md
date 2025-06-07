# Testing Collection Field Functionality

This document explains how to test the collection atom ref creation feature implemented in PBI 1.

## Overview

We have re-enabled collection field support in the fold db schema system. When schemas with collection field types are approved, the system now automatically creates `AtomRefCollection` instances for each collection field.

## Key Changes

1. **CollectionField struct** added to `src/schema/types/field/collection_field.rs`
2. **Collection variant** added to `FieldVariant` enum 
3. **map_fields function** updated to handle collection fields and create `AtomRefCollection` instances
4. **convert_field function** updated to create `CollectionField` instances from JSON schemas
5. **FieldType enum** now includes Collection variant

## Testing with BlogPost Schema

A sample schema `BlogPost.json` has been created in the `available_schemas` directory with two collection fields:
- `tags` - a collection field for blog post tags
- `comments` - a collection field for blog post comments

### Testing Steps

1. **Start the fold db server**:
   ```bash
   ./run_http_server.sh
   ```

2. **Load the BlogPost schema**:
   ```bash
   curl -X POST http://localhost:9001/api/schema/BlogPost/load
   ```

3. **Approve the schema**:
   ```bash
   curl -X POST http://localhost:9001/api/schema/BlogPost/approve
   ```

4. **Verify the schema has collection fields with atom refs**:
   ```bash
   curl http://localhost:9001/api/schema/BlogPost
   ```

   You should see that the `tags` and `comments` fields now have `ref_atom_uuid` values assigned.

5. **Check the logs** to verify `AtomRefCollection` instances were created:
   Look for log messages like:
   - `✅ Persisted AtomRefCollection: ref:{uuid}`

## Expected Behavior

When a schema with collection fields is approved:

1. The `map_fields` function detects collection fields
2. For each collection field without a `ref_atom_uuid`:
   - A new UUID is generated
   - An `AtomRefCollection` instance is created with `new("system")`
   - The collection is stored in the database with key `ref:{uuid}`
   - The field's `ref_atom_uuid` is updated to point to this collection

## Verification

After approving a schema with collection fields:

1. The collection fields should have `ref_atom_uuid` values
2. The database should contain `AtomRefCollection` instances at `ref:{uuid}` keys
3. The schema can now store array data in collection fields

## Limitations

Currently, only the atom ref creation is implemented. Full collection operations (add, remove, update items) would need additional implementation in the mutation system.