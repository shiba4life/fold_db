# DataFold Node API

This document describes how to interact with a running DataFold node. The node exposes both HTTP and TCP interfaces that accept JSON payloads.

## Schema States

A schema can be in one of two states:

- **Loaded** – active and queryable in memory.
- **Unloaded** – persisted on disk but not loaded. Unloaded schemas remain visible through `list_available_schemas` and can be reloaded later.

## HTTP API Routes

All HTTP routes are prefixed with `/api`.

| Method | Path | Description |
| ------ | ---- | ----------- |
| `GET` | `/schemas` | List schemas currently loaded. |
| `GET` | `/schema/{name}` | Retrieve a schema by name. |
| `POST` | `/schema` | Create a new schema from JSON. |
| `PUT` | `/schema/{name}` | Replace an existing schema. |
| `DELETE` | `/schema/{name}` | Unload a schema without deleting it from disk. |
| `POST` | `/execute` | Execute a query or mutation operation. |

The TCP protocol accepts the same operations. Additional operation `list_available_schemas` returns all schemas stored on disk with their state.

### Example: Unload a Schema via HTTP

```bash
curl -X DELETE http://localhost:8080/api/schema/UserProfile
```

### Example: List Available Schemas via TCP

```json
{
  "operation": "list_available_schemas"
}
```

## CLI Commands

The `datafold_cli` binary mirrors these routes. New commands include:

- `list-available-schemas` – show schemas stored on disk.
- `unload-schema --name <NAME>` – mark a schema as unloaded.

### Example

```bash
# Unload the current schema
datafold_cli unload-schema --name UserProfile
```
