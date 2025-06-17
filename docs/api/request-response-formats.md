# Request/Response Formats

This document provides detailed specifications for request and response formats used across DataFold's HTTP REST API and TCP protocol interfaces.

## HTTP Request Formats

### Query Request
Standard format for data query operations.

```json
{
  "type": "query",
  "schema": "SchemaName",
  "fields": ["field1", "field2"],
  "filter": {
    "field": "field_name",
    "operator": "eq|gt|lt|gte|lte|ne",
    "value": "value"
  }
}
```

**Fields:**
- `type`: Operation type (always "query")
- `schema`: Target schema name
- `fields`: Array of field names to retrieve
- `filter`: Optional filtering criteria

**Example:**
```json
{
  "type": "query",
  "schema": "UserProfile",
  "fields": ["username", "email", "age"],
  "filter": {
    "field": "age",
    "operator": "gte",
    "value": 18
  }
}
```

### Range Query Request
Extended query format for range fields with specialized filtering.

```json
{
  "type": "query",
  "schema": "SchemaName",
  "fields": ["field1", "range_field"],
  "filter": {
    "field": "range_field",
    "range_filter": {
      "Key": "specific_key" |
      "KeyPrefix": "prefix" |
      "KeyRange": {"start": "start_key", "end": "end_key"} |
      "Keys": ["key1", "key2"] |
      "KeyPattern": "pattern*" |
      "Value": "value"
    }
  }
}
```

**Range Filter Types:**

#### Specific Key
```json
{
  "range_filter": {
    "Key": "2024-01-15"
  }
}
```

#### Key Prefix
```json
{
  "range_filter": {
    "KeyPrefix": "2024-01"
  }
}
```

#### Key Range
```json
{
  "range_filter": {
    "KeyRange": {
      "start": "2024-01-01",
      "end": "2024-01-31"
    }
  }
}
```

#### Multiple Keys
```json
{
  "range_filter": {
    "Keys": ["2024-01-01", "2024-01-15", "2024-01-30"]
  }
}
```

#### Pattern Matching
```json
{
  "range_filter": {
    "KeyPattern": "2024-*-01"
  }
}
```

#### Value Matching
```json
{
  "range_filter": {
    "Value": "high_activity"
  }
}
```

### Mutation Request
Format for data modification operations.

```json
{
  "type": "mutation",
  "schema": "SchemaName",
  "operation": "create|update|delete",
  "data": {
    "field1": "value1",
    "field2": "value2"
  },
  "filter": {
    "field": "field_name",
    "value": "filter_value"
  }
}
```

**Fields:**
- `type`: Operation type (always "mutation")
- `schema`: Target schema name
- `operation`: Mutation type (create, update, delete)
- `data`: Data to insert/update (required for create/update)
- `filter`: Condition for update/delete operations

**Create Example:**
```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "operation": "create",
  "data": {
    "username": "alice",
    "email": "alice@example.com",
    "age": 25
  }
}
```

**Update Example:**
```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "operation": "update",
  "filter": {
    "field": "username",
    "value": "alice"
  },
  "data": {
    "email": "alice.new@example.com"
  }
}
```

**Delete Example:**
```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "operation": "delete",
  "filter": {
    "field": "username",
    "value": "alice"
  }
}
```

## Response Formats

### Standard Response
Common response format for all operations.

```json
{
  "results": [
    {
      "field1": "value1",
      "field2": "value2"
    }
  ],
  "errors": [
    {
      "code": "ERROR_CODE",
      "message": "Human-readable error message",
      "field": "field_name"
    }
  ],
  "metadata": {
    "execution_time_ms": 25,
    "rows_affected": 1,
    "total_fee_sats": 100
  }
}
```

**Fields:**
- `results`: Array of result objects
- `errors`: Array of error objects (empty if no errors)
- `metadata`: Execution metadata

### Query Response
Response format for successful queries.

```json
{
  "results": [
    {
      "username": "alice",
      "email": "alice@example.com",
      "age": 25
    },
    {
      "username": "bob", 
      "email": "bob@example.com",
      "age": 30
    }
  ],
  "errors": [],
  "metadata": {
    "execution_time_ms": 15,
    "rows_returned": 2,
    "total_fee_sats": 50
  }
}
```

### Mutation Response
Response format for successful mutations.

```json
{
  "results": [
    {
      "success": true,
      "operation": "create",
      "affected_records": 1
    }
  ],
  "errors": [],
  "metadata": {
    "execution_time_ms": 8,
    "rows_affected": 1
  }
}
```

### Error Response
Response format when errors occur.

```json
{
  "results": [],
  "errors": [
    {
      "code": "PERMISSION_DENIED",
      "message": "Insufficient permissions for field: email",
      "field": "email",
      "details": {
        "required_trust_distance": 0,
        "current_trust_distance": 2
      }
    }
  ],
  "metadata": {
    "execution_time_ms": 5
  }
}
```

## TCP Protocol Formats

### TCP Request Format
Binary message format for TCP protocol.

```json
{
  "app_id": "client-application-name",
  "operation": "operation-type",
  "params": {
    // Operation-specific parameters
  },
  "signature": "optional-signature",
  "timestamp": 1234567890
}
```

**TCP Query Example:**
```json
{
  "app_id": "my-app",
  "operation": "query",
  "params": {
    "schema": "UserProfile",
    "fields": ["username", "email"],
    "filter": {
      "username": "alice"
    }
  }
}
```

**TCP Mutation Example:**
```json
{
  "app_id": "my-app",
  "operation": "mutation",
  "params": {
    "schema": "UserProfile",
    "mutation_type": "create",
    "data": {
      "username": "bob",
      "email": "bob@example.com"
    }
  }
}
```

### TCP Response Format
Standard TCP response format.

```json
{
  "results": [...],
  "errors": [...],
  "metadata": {
    "execution_time_ms": 15,
    "request_id": "req_123"
  }
}
```

## Batch Operation Formats

### Batch Request
Execute multiple operations in a single request.

```json
{
  "operations": [
    {
      "type": "query",
      "schema": "UserProfile",
      "fields": ["username"]
    },
    {
      "type": "mutation",
      "schema": "UserProfile",
      "operation": "create",
      "data": {"username": "charlie", "email": "charlie@example.com"}
    }
  ]
}
```

### Batch Response
Response format for batch operations.

```json
{
  "results": [
    {
      "operation_index": 0,
      "results": [
        {"username": "alice"},
        {"username": "bob"}
      ],
      "errors": []
    },
    {
      "operation_index": 1,
      "results": [
        {"success": true, "affected_records": 1}
      ],
      "errors": []
    }
  ]
}
```

## Field Type Handling

### Single Fields
Simple value fields.

**Input:**
```json
{
  "username": "alice",
  "age": 25,
  "active": true
}
```

**Output:**
```json
{
  "username": "alice",
  "age": 25,
  "active": true
}
```

### Collection Fields
Array or list fields.

**Input:**
```json
{
  "tags": ["developer", "python", "rust"],
  "scores": [95, 87, 92]
}
```

**Output:**
```json
{
  "tags": ["developer", "python", "rust"],
  "scores": [95, 87, 92]
}
```

### Range Fields
Key-value mapping fields.

**Input:**
```json
{
  "activity_log": {
    "2024-01-01": {"logins": 5, "queries": 20},
    "2024-01-02": {"logins": 3, "queries": 15}
  }
}
```

**Output:**
```json
{
  "activity_log": {
    "2024-01-01": {"logins": 5, "queries": 20},
    "2024-01-02": {"logins": 3, "queries": 15}
  }
}
```

## Data Type Specifications

### Supported Data Types

#### Primitives
- `string`: UTF-8 text
- `number`: Integer or floating-point
- `boolean`: true/false
- `null`: Null value

#### Complex Types
- `object`: JSON object
- `array`: JSON array
- `binary`: Base64-encoded binary data

### Type Validation
DataFold validates input data against schema field types:

```json
{
  "field_name": {
    "field_type": "Single",
    "value_type": "string|number|boolean|object|array",
    "constraints": {
      "min_length": 1,
      "max_length": 255,
      "pattern": "^[a-zA-Z0-9_]+$"
    }
  }
}
```

## Authentication Headers

### HTTP Authentication Headers

#### API Key
```
Authorization: Bearer your-api-key
```

#### Signature-Based
```
X-Public-Key: ed25519:public-key-base64
X-Signature: ed25519:signature-base64
X-Timestamp: 1609459200
```

### TCP Authentication Fields
```json
{
  "app_id": "my-app",
  "operation": "query",
  "params": {...},
  "public_key": "ed25519:public-key-base64",
  "signature": "ed25519:signature-base64",
  "timestamp": 1609459200
}
```

## Content Types

### HTTP Content Types
- Request: `application/json`
- Response: `application/json`
- Streaming: `text/event-stream` (for log streaming)

### TCP Encoding
- Length prefix: 4 bytes, little-endian unsigned integer
- Payload: UTF-8 encoded JSON

## Validation Rules

### Field Names
- Must be valid identifiers: `^[a-zA-Z][a-zA-Z0-9_]*$`
- Case sensitive
- Maximum length: 255 characters

### Schema Names
- Must be valid identifiers: `^[a-zA-Z][a-zA-Z0-9_]*$`
- Case sensitive
- Maximum length: 255 characters

### Timestamps
- Unix timestamps in seconds
- Must be within 5 minutes of server time for authentication
- ISO 8601 format for human-readable timestamps

## Related Documentation

- [Data Operations API](./data-operations-api.md) - Using these formats in HTTP API
- [TCP Protocol](./tcp-protocol.md) - TCP-specific format details
- [Authentication](./authentication.md) - Authentication format specifications
- [Error Handling](./error-handling.md) - Error response formats
- [Schema Management API](./schema-management-api.md) - Schema definition formats

## Return to Index

[← Back to API Reference Index](./index.md)