# FoldDB Integration for Social App

This document explains how the Social App integrates with FoldDB for data persistence.

## Overview

The Social App uses FoldDB as its primary data store. FoldDB is a schema-based, atom-oriented database that provides:

- Schema validation
- Atom-based storage for data versioning
- Permissions management
- Query and mutation operations

## Architecture

The integration follows this architecture:

```
+----------------+      +----------------+      +----------------+
|                |      |                |      |                |
|  Social App UI |----->| FoldDB Client  |----->|    FoldDB     |
|                |      |                |      |                |
+----------------+      +----------------+      +----------------+
```

1. The Social App UI makes API requests to the server
2. The server uses the FoldDB Client to interact with FoldDB
3. FoldDB handles data storage, validation, and retrieval

## Schema Definition

The app defines schemas for its data models in the `schemas/` directory:

- `post.json`: Defines the structure of posts
- `user-profile.json`: Defines the structure of user profiles
- `comment.json`: Defines the structure of comments

These schemas are loaded by the FoldDB client during initialization.

## Data Operations

### Queries

The app uses FoldDB's query operations to retrieve data:

```javascript
// Example query operation
const queryOperation = {
  type: "query",
  schema: "post",
  fields: ["content", "timestamp", "likes", "comments", "author"],
  sort: { field: "timestamp", order: "desc" }
};

// Execute query
const results = await foldDBClient.executeQuery(queryOperation);
```

### Mutations

The app uses FoldDB's mutation operations to create, update, and delete data:

```javascript
// Example mutation operation
const createOperation = {
  type: "mutation",
  schema: "post",
  data: {
    content: "Hello, world!",
    author: { id: "1", username: "alice" },
    timestamp: new Date().toISOString(),
    likes: [],
    comments: []
  },
  mutation_type: "create"
};

// Execute mutation
const result = await foldDBClient.executeMutation(createOperation);
```

## Atom-Based Storage

FoldDB stores data as atoms, which are immutable units of data. Each mutation creates a new atom, preserving the history of changes.

For example, when a post is created:

1. A new atom is created with the post data
2. The atom is assigned a unique ID
3. The atom is stored in FoldDB

When a post is updated:

1. A new atom is created with the updated data
2. The atom references the previous atom
3. The atom is stored in FoldDB, creating a chain of changes

## Permissions

The app uses FoldDB's permission system to control access to data:

- Read permissions: Control who can read posts, profiles, etc.
- Write permissions: Control who can create, update, or delete posts, profiles, etc.

These permissions are defined in the app's manifest and enforced by FoldDB.

## Error Handling

The app handles FoldDB errors gracefully:

- Schema validation errors: Displayed to the user with helpful messages
- Permission errors: Displayed to the user with appropriate messages
- Connection errors: Handled with retries and fallbacks

## Testing

The app includes tests for FoldDB integration:

- Unit tests: Test individual FoldDB operations
- Integration tests: Test the interaction between the app and FoldDB
- API tests: Test the API endpoints that use FoldDB

## Future Improvements

- Implement real-time updates using FoldDB's change notification system
- Add support for more complex queries
- Improve error handling and recovery
- Add support for offline mode with local caching
