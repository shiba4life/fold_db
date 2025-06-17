# HTTP REST API Overview

The DataFold HTTP REST API provides a comprehensive interface for managing schemas, executing data operations, controlling network functionality, and monitoring system health through standard HTTP methods.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Protocol**: HTTP/1.1 (HTTPS recommended for production)
**Content-Type**: `application/json` for all POST/PUT requests
**Authentication**: Bearer token or signature-based

## API Structure

The API is organized into logical endpoint groups:

### Core Data Operations
- **[Schema Management](./schema-management-api.md)**: Create and manage data schemas
- **[Data Operations](./data-operations-api.md)**: Query and mutate data within schemas

### Network & System
- **[Network Management](./network-api.md)**: Peer discovery and connection management  
- **[System Monitoring](./system-monitoring-api.md)**: Health checks, metrics, and logging
- **[Transform Management](./transform-api.md)**: Register and manage data transforms

### Security & Access
- **[Authentication](./authentication.md)**: API keys and cryptographic signatures
- **[Permissions & Payments](./permissions-payments-api.md)**: Access control and Lightning payments

## Quick Start

### 1. Health Check
Verify the API is running:

```bash
curl http://localhost:9001/api/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:45:00Z",
  "services": {
    "database": "healthy",
    "network": "healthy",
    "transforms": "healthy"
  }
}
```

### 2. List Available Schemas
Get current schemas in the system:

```bash
curl http://localhost:9001/api/schemas
```

### 3. Load a Schema
Create a new schema:

```bash
curl -X POST http://localhost:9001/api/schema \
  -H "Content-Type: application/json" \
  -d '{
    "name": "UserProfile",
    "fields": {
      "username": {
        "field_type": "Single",
        "permission_policy": {"default_access": "read"}
      }
    }
  }'
```

### 4. Query Data
Execute a query against loaded schema:

```bash
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"query\",\"schema\":\"UserProfile\",\"fields\":[\"username\"]}"
  }'
```

## Endpoint Categories

### Schema Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/schema` | Load new schema |
| GET | `/api/schemas` | List all schemas |
| GET | `/api/schema/{name}` | Get schema details |
| DELETE | `/api/schema/{name}` | Unload schema |

### Data Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/execute` | Execute query/mutation |
| POST | `/api/batch` | Execute multiple operations |

### Network Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/network/start` | Initialize networking |
| POST | `/api/network/discover` | Discover peers |
| POST | `/api/network/connect` | Connect to peer |
| GET | `/api/network/status` | Get network status |
| POST | `/api/network/request-schema` | Request schema from peer |

### Transform Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/transform/register` | Register transform |
| GET | `/api/transforms` | List transforms |
| GET | `/api/transform/{id}` | Get transform details |
| DELETE | `/api/transform/{id}` | Unregister transform |

### System Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/status` | System status |
| GET | `/api/metrics` | Performance metrics |
| POST | `/api/system/shutdown` | Graceful shutdown |

### Logging Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/logs/stream` | Stream real-time logs |
| POST | `/api/logs/features` | Update log levels |
| POST | `/api/logs/reload` | Reload log config |

### Permission Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/permissions/trust-distance` | Set trust distances |
| POST | `/api/permissions/explicit` | Grant explicit permissions |
| GET | `/api/permissions` | List permissions |

### Payment Endpoints  
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/payments/lightning/invoice` | Generate Lightning invoice |
| POST | `/api/payments/verify` | Verify payment |
| GET | `/api/payments/status/{hash}` | Check payment status |

## Authentication

### API Key Authentication
Include bearer token in Authorization header:

```bash
curl -H "Authorization: Bearer your-api-key" \
  http://localhost:9001/api/schemas
```

### Signature-Based Authentication
Use Ed25519 cryptographic signatures:

```bash
curl -H "X-Signature: ed25519:signature-hash" \
  -H "X-Public-Key: ed25519:public-key" \
  -H "X-Timestamp: 1609459200" \
  http://localhost:9001/api/schemas
```

## Request/Response Format

### Standard Request Format
All POST/PUT requests use JSON:

```json
{
  "field1": "value1",
  "field2": "value2"
}
```

### Standard Response Format
All responses include results, errors, and metadata:

```json
{
  "results": [...],
  "errors": [...],
  "metadata": {
    "execution_time_ms": 15
  }
}
```

## Error Handling

### HTTP Status Codes
- `200 OK`: Success
- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Permission denied
- `404 Not Found`: Resource not found
- `402 Payment Required`: Payment needed
- `500 Internal Server Error`: Server error

### Error Response Format
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable description",
    "details": {...}
  }
}
```

## Best Practices

### Performance
1. **Use batch operations** for multiple related requests
2. **Implement connection pooling** for high-throughput applications
3. **Cache schema information** to reduce API calls
4. **Use appropriate timeouts** for long-running operations

### Security
1. **Use HTTPS in production** to encrypt API communications
2. **Rotate API keys regularly** for enhanced security
3. **Validate all inputs** before sending to API
4. **Monitor for unusual activity** in API logs

### Error Handling
1. **Implement retry logic** with exponential backoff
2. **Handle rate limiting** gracefully with appropriate delays
3. **Log errors appropriately** for debugging and monitoring
4. **Use circuit breaker patterns** to prevent cascading failures

### Monitoring
1. **Monitor API health** regularly with health check endpoint
2. **Track response times** and set up alerting for degradation
3. **Monitor error rates** and investigate spikes
4. **Use structured logging** for better observability

## Rate Limiting

The API implements rate limiting to ensure fair usage:

- **Default limits**: 1000 requests per minute per API key
- **Burst allowance**: Up to 100 requests in 10-second window
- **Rate limit headers**: Included in all responses

**Rate Limit Headers:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1609459260
```

## Pagination

Large result sets are paginated:

**Request:**
```bash
curl "http://localhost:9001/api/execute?limit=50&offset=100"
```

**Response:**
```json
{
  "results": [...],
  "pagination": {
    "limit": 50,
    "offset": 100,
    "total": 1250,
    "has_more": true
  }
}
```

## Content Negotiation

### Supported Content Types
- **Request**: `application/json`
- **Response**: `application/json`
- **Streaming**: `text/event-stream` (for logs)

### Response Formats
Use `Accept` header or query parameter:

```bash
# JSON (default)
curl -H "Accept: application/json" http://localhost:9001/api/schemas

# Query parameter
curl "http://localhost:9001/api/schemas?format=json"
```

## Development Tools

### API Testing
```bash
# Health check
curl http://localhost:9001/api/health

# Schema operations
curl -X POST http://localhost:9001/api/schema -d @schema.json

# Data queries
curl -X POST http://localhost:9001/api/execute -d @query.json
```

### SDK Integration
- **[Python SDK](./sdks/python/README.md)**: Full-featured Python client
- **[JavaScript SDK](./sdks/javascript/README.md)**: Browser and Node.js support
- **[CLI Tool](./cli-interface.md)**: Command-line interface

## Related Documentation

- **[Data Operations API](./data-operations-api.md)**: Detailed query and mutation operations
- **[Schema Management API](./schema-management-api.md)**: Schema creation and management
- **[Authentication](./authentication.md)**: Security and access control
- **[Request/Response Formats](./request-response-formats.md)**: Detailed format specifications
- **[Error Handling](./error-handling.md)**: Error codes and troubleshooting

## Return to Index

[← Back to API Reference Index](./index.md)