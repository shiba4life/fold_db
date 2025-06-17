# Schema Management API

The Schema Management API provides HTTP endpoints for creating, loading, and managing schemas in the DataFold system.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Content-Type**: `application/json` for all POST/PUT requests

## Schema Endpoints

### POST /api/schema
Load a new schema into the node.

**Note**: Schemas are immutable once created. This endpoint creates new schemas only. To change schema structure, create a new schema with a different name.

**Request Body:**
```json
{
  "name": "SchemaName",
  "fields": {
    "field_name": {
      "field_type": "Single|Collection|Range",
      "permission_policy": {...},
      "payment_config": {...}
    }
  },
  "payment_config": {...}
}
```

**Response:**
```json
{
  "success": true,
  "message": "Schema loaded successfully",
  "schema_name": "SchemaName"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/schema \
  -H "Content-Type: application/json" \
  -d @schema.json
```

### GET /api/schemas
List all loaded schemas.

**Response:**
```json
{
  "schemas": [
    {
      "name": "UserProfile",
      "fields": 5,
      "loaded_at": "2024-01-15T10:30:00Z"
    },
    {
      "name": "EventAnalytics", 
      "fields": 4,
      "loaded_at": "2024-01-15T11:00:00Z"
    }
  ]
}
```

### GET /api/schema/{schema_name}
Get detailed information about a specific schema.

**Response:**
```json
{
  "name": "UserProfile",
  "fields": {
    "username": {
      "field_type": "Single",
      "permission_policy": {...},
      "payment_config": {...}
    }
  },
  "payment_config": {...},
  "loaded_at": "2024-01-15T10:30:00Z"
}
```

### DELETE /api/schema/{schema_name}
Unload a schema from the node.

**Note**: This removes the schema from memory but does not delete any stored data. See [Schema Immutability](../schema-management.md#schema-immutability) for details.

**Response:**
```json
{
  "success": true,
  "message": "Schema unloaded successfully"
}
```

## Schema Definition Format

### Basic Schema Structure
```json
{
  "name": "UserProfile",
  "fields": {
    "username": {
      "field_type": "Single",
      "permission_policy": {
        "default_access": "read",
        "required_trust_distance": 1
      },
      "payment_config": {
        "cost_per_access": 0
      }
    },
    "email": {
      "field_type": "Single",
      "permission_policy": {
        "default_access": "private",
        "required_trust_distance": 0
      },
      "payment_config": {
        "cost_per_access": 100
      }
    },
    "activity_log": {
      "field_type": "Range",
      "permission_policy": {
        "default_access": "read",
        "required_trust_distance": 2
      },
      "payment_config": {
        "cost_per_access": 50
      }
    }
  },
  "payment_config": {
    "default_cost_per_operation": 10
  }
}
```

### Field Types

#### Single Field
For single-value fields like username, email, age.

```json
{
  "field_type": "Single",
  "permission_policy": {...},
  "payment_config": {...}
}
```

#### Collection Field
For arrays or lists of values.

```json
{
  "field_type": "Collection",
  "permission_policy": {...},
  "payment_config": {...}
}
```

#### Range Field
For key-value mappings with range queries support.

```json
{
  "field_type": "Range",
  "permission_policy": {...},
  "payment_config": {...}
}
```

## Error Responses

### Schema Errors
- `SCHEMA_NOT_FOUND`: Requested schema does not exist
- `SCHEMA_VALIDATION_FAILED`: Schema definition is invalid
- `SCHEMA_ALREADY_EXISTS`: Schema with same name already loaded

**Example Error Response:**
```json
{
  "error": {
    "code": "SCHEMA_NOT_FOUND",
    "message": "Schema 'NonExistentSchema' was not found",
    "details": {
      "schema_name": "NonExistentSchema",
      "available_schemas": ["UserProfile", "EventAnalytics"]
    }
  }
}
```

## CLI Equivalents

All HTTP endpoints have CLI command equivalents:

- `POST /api/schema` ↔ [`datafold_cli load-schema`](./cli-interface.md#load-schema)
- `GET /api/schemas` ↔ [`datafold_cli list-schemas`](./cli-interface.md#list-schemas)
- `GET /api/schema/{name}` ↔ [`datafold_cli get-schema`](./cli-interface.md#get-schema)
- `DELETE /api/schema/{name}` ↔ [`datafold_cli unload-schema`](./cli-interface.md#unload-schema)

## Related Documentation

- [Schema Management Guide](../schema-management.md) - Detailed schema concepts
- [CLI Interface](./cli-interface.md) - Command-line equivalents
- [Data Operations API](./data-operations-api.md) - Working with schema data
- [Authentication](./authentication.md) - Securing schema operations
- [Error Handling](./error-handling.md) - Complete error reference

## Return to Index

[← Back to API Reference Index](./index.md)