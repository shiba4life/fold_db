# Data Operations API

The Data Operations API provides HTTP endpoints for querying and mutating data within loaded schemas.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Content-Type**: `application/json` for all POST/PUT requests

## Data Endpoints

### POST /api/execute
Execute a query or mutation operation.

**Request Body:**
```json
{
  "operation": "{\"type\":\"query|mutation\",\"schema\":\"SchemaName\",\"fields\":[...],\"filter\":{...}}"
}
```

**Query Example:**
```bash
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"query\",\"schema\":\"UserProfile\",\"fields\":[\"username\",\"email\"],\"filter\":{\"username\":\"alice\"}}"
  }'
```

**Mutation Example:**
```bash
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"create\",\"data\":{\"username\":\"bob\",\"email\":\"bob@example.com\"}}"
  }'
```

**Response:**
```json
{
  "results": [
    {
      "username": "alice",
      "email": "alice@example.com"
    }
  ],
  "errors": [],
  "metadata": {
    "execution_time_ms": 15,
    "rows_affected": 1
  }
}
```

### POST /api/batch
Execute multiple operations in a single request.

**Request Body:**
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

**Response:**
```json
{
  "results": [
    {
      "operation_index": 0,
      "results": [...],
      "errors": []
    },
    {
      "operation_index": 1, 
      "results": [...],
      "errors": []
    }
  ]
}
```

## Query Operations

### Basic Query
Query specific fields from a schema with optional filtering.

```json
{
  "type": "query",
  "schema": "UserProfile",
  "fields": ["username", "email"],
  "filter": {
    "username": "alice"
  }
}
```

### Advanced Filtering
Use operators for more complex queries.

```json
{
  "type": "query",
  "schema": "UserProfile",
  "fields": ["username", "age"],
  "filter": {
    "field": "age",
    "operator": "gte",
    "value": 18
  }
}
```

**Available Operators:**
- `eq` - Equal to
- `ne` - Not equal to
- `gt` - Greater than
- `gte` - Greater than or equal to
- `lt` - Less than
- `lte` - Less than or equal to

### Range Queries
Query range fields with various filter types.

```json
{
  "type": "query",
  "schema": "EventAnalytics",
  "fields": ["event_name", "metrics_by_timeframe"],
  "filter": {
    "field": "metrics_by_timeframe",
    "range_filter": {
      "KeyPrefix": "2024-01-01"
    }
  }
}
```

**Range Filter Types:**
- `Key`: Get specific key
- `KeyPrefix`: Get all keys with prefix
- `KeyRange`: Get keys within range
- `Keys`: Get multiple specific keys
- `KeyPattern`: Get keys matching pattern
- `Value`: Get by value match

**Range Filter Examples:**
```json
// Specific key
{"Key": "2024-01-01"}

// Key prefix
{"KeyPrefix": "2024-01"}

// Key range
{"KeyRange": {"start": "2024-01-01", "end": "2024-01-31"}}

// Multiple keys
{"Keys": ["2024-01-01", "2024-01-15", "2024-01-30"]}

// Pattern matching
{"KeyPattern": "2024-*-01"}

// Value matching
{"Value": "high_activity"}
```

## Mutation Operations

### Create Operation
Add new data to a schema.

```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "operation": "create",
  "data": {
    "username": "bob",
    "email": "bob@example.com",
    "age": 25
  }
}
```

### Update Operation
Modify existing data with filtering.

```json
{
  "type": "mutation",
  "schema": "UserProfile", 
  "operation": "update",
  "filter": {
    "username": "bob"
  },
  "data": {
    "email": "newemail@example.com"
  }
}
```

### Delete Operation
Remove data matching filter criteria.

```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "operation": "delete",
  "filter": {
    "username": "bob"
  }
}
```

## Response Format

All data operations return a standard response format:

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
      "code": "PERMISSION_DENIED",
      "message": "Insufficient permissions for field: email",
      "field": "email"
    }
  ],
  "metadata": {
    "execution_time_ms": 25,
    "rows_affected": 1,
    "total_fee_sats": 100
  }
}
```

## Error Handling

### Data Errors
- `FIELD_NOT_FOUND`: Requested field does not exist
- `INVALID_FILTER`: Filter syntax is invalid
- `TYPE_MISMATCH`: Data type does not match field type

### Permission Errors
- `PERMISSION_DENIED`: Insufficient permissions for operation
- `TRUST_DISTANCE_EXCEEDED`: Required trust distance not met
- `EXPLICIT_PERMISSION_REQUIRED`: Explicit permission needed

### Payment Errors
- `PAYMENT_REQUIRED`: Payment needed for operation
- `INSUFFICIENT_PAYMENT`: Payment amount too low
- `PAYMENT_EXPIRED`: Payment has expired

## CLI Equivalents

HTTP data operations have CLI command equivalents:

- Query operations ↔ [`datafold_cli query`](./cli-interface.md#query)
- Mutation operations ↔ [`datafold_cli mutate`](./cli-interface.md#mutate)

## Related Documentation

- [Request/Response Formats](./request-response-formats.md) - Detailed format specifications
- [Schema Management API](./schema-management-api.md) - Managing schemas first
- [Authentication](./authentication.md) - Securing operations
- [Permissions & Payments API](./permissions-payments-api.md) - Access control
- [Error Handling](./error-handling.md) - Complete error reference

## Return to Index

[← Back to API Reference Index](./index.md)