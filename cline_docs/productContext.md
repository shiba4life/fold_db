# Product Context

## Why this project exists
FoldDB is a database implementation that stores data in an atom-based format, where each piece of data is stored as a chain of atoms. The project exists to provide a flexible and traceable data storage solution with built-in version history.

## Problems it solves
- Provides version history for all data changes
- Abstracts internal UUID-based references through schema mappings
- Enables GraphQL-based data access without exposing internal complexities
- Maintains data integrity through immutable atom chains

## How it should work
1. Data is stored as atoms in a key-value store (using sled)
2. Each atom contains:
   - UUID
   - Content (as JSON string)
   - Type
   - Source
   - Creation timestamp
   - Previous atom reference
3. AtomRefs point to the latest atom in a chain
4. Internal schemas map field names to aref_uuids
5. GraphQL interface provides clean data access without exposing internal UUIDs
