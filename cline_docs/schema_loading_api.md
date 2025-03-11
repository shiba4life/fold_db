# Schema Loading API

The DataFold Node now exposes HTTP endpoints for loading schemas, making it queryable, mutable, and capable of loading new schemas dynamically.

## Endpoints

### List Schemas

```
GET /api/schemas
```

Returns a list of all loaded schemas.

**Response:**
```json
{
  "schemas": ["SchemaName1", "SchemaName2"]
}
```

### Create/Update Schema

```
POST /api/schema
```

Creates a new schema or updates an existing one.

**Request Body:**
```json
{
  "name": "SchemaName",
  "fields": {
    "field1": {
      "permission_policy": {
        "read_policy": {
          "NoRequirement": null
        },
        "write_policy": {
          "Distance": 0
        }
      },
      "payment_config": {
        "base_multiplier": 1.0
      },
      "field_mappers": {}
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 0
  }
}
```

**Response:**
```json
{
  "success": true
}
```

### Load Schema from File

```
POST /api/schema/load/file
```

Loads a schema from a file on the server.

**Request Body:**
```json
{
  "file_path": "/path/to/schema.json"
}
```

**Response:**
```json
{
  "data": {
    "schema_name": "SchemaName",
    "message": "Schema loaded successfully"
  }
}
```

### Load Schema from JSON

```
POST /api/schema/load/json
```

Loads a schema directly from JSON content.

**Request Body:**
```json
{
  "schema_json": {
    "name": "SchemaName",
    "fields": {
      "field1": {
        "permission_policy": {
          "read_policy": {
            "NoRequirement": null
          },
          "write_policy": {
            "Distance": 0
          }
        },
        "payment_config": {
          "base_multiplier": 1.0
        },
        "field_mappers": {}
      }
    },
    "payment_config": {
      "base_multiplier": 1.0,
      "min_payment_threshold": 0
    }
  }
}
```

**Response:**
```json
{
  "data": {
    "schema_name": "SchemaName",
    "message": "Schema loaded successfully"
  }
}
```

### Delete Schema

```
DELETE /api/schema/:name
```

Deletes a schema by name.

**Response:**
```json
{
  "success": true
}
```

### Execute Operation (Query/Mutation)

```
POST /api/execute
```

Executes a query or mutation operation on a schema.

**Request Body for Query:**
```json
{
  "operation": "{\"type\":\"query\",\"schema\":\"SchemaName\",\"fields\":[\"field1\",\"field2\"],\"filter\":{\"field1\":\"value1\"}}"
}
```

**Response for Query:**
```json
{
  "results": [
    {
      "field1": "value1",
      "field2": "value2"
    }
  ],
  "count": 1
}
```

**Request Body for Mutation:**
```json
{
  "operation": "{\"type\":\"mutation\",\"schema\":\"SchemaName\",\"operation\":\"create\",\"data\":{\"field1\":\"value1\",\"field2\":\"value2\"}}"
}
```

**Response for Mutation:**
```json
{
  "success": true,
  "affected_count": 1
}
```

## Client Usage

The DataFold client has been updated to support the new schema loading functionality:

```javascript
// Create a client instance
const client = new DataFoldClient({
  baseUrl: 'http://localhost:8080'
});

// Load schema from file
const fileLoadResult = await client.loadSchemaFromFile('/path/to/schema.json');
console.log(fileLoadResult);

// Load schema from JSON
const schema = {
  name: 'ExampleSchema',
  fields: {
    title: {
      permission_policy: {
        read_policy: { NoRequirement: null },
        write_policy: { Distance: 0 }
      },
      payment_config: {
        base_multiplier: 1.0,
        trust_distance_scaling: { None: null }
      },
      field_mappers: {}
    }
  },
  payment_config: {
    base_multiplier: 1.0,
    min_payment_threshold: 0
  }
};

const jsonLoadResult = await client.loadSchemaFromJson(schema);
console.log(jsonLoadResult);
```

## Authentication

Most schema operations require authentication. The client must provide a valid public key in the `X-Public-Key` header, along with a signature in the `X-Signature` header.

## Error Handling

All endpoints return appropriate HTTP status codes and error messages in case of failure:

```json
{
  "success": false,
  "error": "Error message"
}
```

Common error scenarios:
- 400 Bad Request: Invalid schema format, missing required fields
- 404 Not Found: Schema or file not found
- 500 Internal Server Error: Server-side errors
