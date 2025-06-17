# Transform API

The Transform API provides HTTP endpoints for registering, managing, and executing transform functions that process data within schemas.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Content-Type**: `application/json` for all POST/PUT requests

## Transform Endpoints

### POST /api/transform/register
Register a new transform function.

**Request Body:**
```json
{
  "name": "user_status_transform",
  "inputs": ["age"],
  "logic": "if age >= 18 { return \"adult\" } else { return \"minor\" }",
  "output": "UserProfile.status"
}
```

**Response:**
```json
{
  "success": true,
  "transform_id": "transform_123",
  "registered_at": "2024-01-15T10:40:00Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/transform/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "user_status_transform",
    "inputs": ["age"],
    "logic": "if age >= 18 { return \"adult\" } else { return \"minor\" }",
    "output": "UserProfile.status"
  }'
```

### GET /api/transforms
List all registered transforms.

**Query Parameters:**
- `schema` (optional): Filter transforms by schema name

**Response:**
```json
{
  "transforms": [
    {
      "id": "transform_123",
      "name": "user_status_transform",
      "schema": "UserProfile",
      "output_field": "status",
      "registered_at": "2024-01-15T10:40:00Z"
    }
  ]
}
```

**Examples:**
```bash
# List all transforms
curl http://localhost:9001/api/transforms

# Filter by schema
curl http://localhost:9001/api/transforms?schema=UserProfile
```

### GET /api/transform/{transform_id}
Get detailed information about a specific transform.

**Response:**
```json
{
  "id": "transform_123",
  "name": "user_status_transform",
  "inputs": ["age"],
  "logic": "if age >= 18 { return \"adult\" } else { return \"minor\" }",
  "output": "UserProfile.status",
  "schema": "UserProfile",
  "registered_at": "2024-01-15T10:40:00Z",
  "execution_stats": {
    "total_executions": 1250,
    "avg_execution_time_ms": 2.5,
    "last_executed": "2024-01-15T11:30:00Z"
  }
}
```

### DELETE /api/transform/{transform_id}
Unregister a transform function.

**Response:**
```json
{
  "success": true,
  "message": "Transform unregistered successfully"
}
```

**Example:**
```bash
curl -X DELETE http://localhost:9001/api/transform/transform_123
```

### POST /api/transform/{transform_id}/execute
Manually execute a transform with test data.

**Request Body:**
```json
{
  "test_inputs": {
    "age": 25
  }
}
```

**Response:**
```json
{
  "success": true,
  "result": "adult",
  "execution_time_ms": 2,
  "timestamp": "2024-01-15T11:35:00Z"
}
```

## Transform Definition Format

### Basic Transform Structure
```json
{
  "name": "user_status_transform",
  "description": "Determines if user is adult or minor based on age",
  "inputs": ["age"],
  "logic": "if age >= 18 { return \"adult\" } else { return \"minor\" }",
  "output": "UserProfile.status",
  "trigger_conditions": {
    "on_insert": true,
    "on_update": ["age"],
    "on_query": false
  }
}
```

### Transform Logic
Transform logic is written in a simple expression language:

#### Basic Conditions
```javascript
// Simple comparison
if age >= 18 { return "adult" } else { return "minor" }

// Multiple conditions
if age >= 65 { return "senior" } 
else if age >= 18 { return "adult" } 
else { return "minor" }

// String operations
if username.length > 10 { return "long" } else { return "short" }
```

#### Available Functions
```javascript
// Math functions
max(a, b)
min(a, b)
abs(value)
round(value)

// String functions
length(string)
substring(string, start, end)
uppercase(string)
lowercase(string)

// Date functions
now()
date_add(date, days)
date_diff(date1, date2)
```

### Input Types
Transforms can accept various input types:

```json
{
  "inputs": [
    "age",                    // Single field
    "preferences.theme",      // Nested field
    "activity_log.last_7d"   // Range field aggregation
  ]
}
```

### Output Targets
Transforms can output to different targets:

```json
{
  "output": "UserProfile.computed_status"      // Single field
}
```

```json
{
  "output": "UserProfile.activity_summary.*"  // Multiple fields
}
```

## Transform Execution

### Automatic Execution
Transforms execute automatically based on trigger conditions:

```json
{
  "trigger_conditions": {
    "on_insert": true,        // Execute when new data is inserted
    "on_update": ["age"],     // Execute when 'age' field is updated
    "on_query": false         // Don't execute during queries (default)
  }
}
```

### Manual Execution
Use the execute endpoint to test transforms or trigger manual execution:

```bash
curl -X POST http://localhost:9001/api/transform/transform_123/execute \
  -H "Content-Type: application/json" \
  -d '{"test_inputs": {"age": 25}}'
```

## Error Handling

### Transform Errors
- `TRANSFORM_NOT_FOUND`: Transform ID does not exist
- `TRANSFORM_VALIDATION_FAILED`: Transform logic is invalid
- `TRANSFORM_EXECUTION_FAILED`: Transform execution error
- `INVALID_INPUT`: Input data doesn't match expected format

**Example Error Response:**
```json
{
  "error": {
    "code": "TRANSFORM_EXECUTION_FAILED",
    "message": "Transform execution failed: division by zero",
    "details": {
      "transform_id": "transform_123",
      "input_data": {"age": 0},
      "error_line": 1
    }
  }
}
```

## Transform File Format

Transforms can be loaded from JSON files:

```json
{
  "name": "user_analytics_transform",
  "description": "Calculate user engagement metrics",
  "inputs": [
    "activity_log.last_30d",
    "login_count",
    "last_seen"
  ],
  "logic": "
    let activity_score = activity_log.last_30d.length * 2;
    let login_score = login_count / 30;
    let recency_score = date_diff(now(), last_seen) > 7 ? 0 : 10;
    return activity_score + login_score + recency_score;
  ",
  "output": "UserProfile.engagement_score",
  "trigger_conditions": {
    "on_insert": true,
    "on_update": ["activity_log", "login_count", "last_seen"],
    "on_query": false
  }
}
```

## CLI Equivalents

Transform operations have CLI command equivalents:

- Register ↔ [`datafold_cli register-transform`](./cli-interface.md#register-transform)
- List ↔ [`datafold_cli list-transforms`](./cli-interface.md#list-transforms)

## Performance Considerations

### Optimization Tips
1. **Keep transforms simple** - Complex logic can slow down data operations
2. **Minimize dependencies** - Fewer input fields = faster execution
3. **Use appropriate triggers** - Don't execute on every query unless necessary
4. **Cache results** - Consider output field caching for expensive computations

### Monitoring
- Monitor `execution_stats` for performance metrics
- Track `avg_execution_time_ms` to identify slow transforms
- Use manual execution for testing before deploying

## Related Documentation

- [Transforms Guide](../transforms.md) - Detailed transform concepts
- [Schema Management API](./schema-management-api.md) - Setting up schemas for transforms
- [Data Operations API](./data-operations-api.md) - How transforms affect data operations
- [Error Handling](./error-handling.md) - Transform error troubleshooting

## Return to Index

[← Back to API Reference Index](./index.md)