# System Monitoring API

The System Monitoring API provides HTTP endpoints for monitoring node health, performance metrics, system status, and real-time logging.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Content-Type**: `application/json` for all POST/PUT requests

## Health & Status Endpoints

### GET /api/health
Health check endpoint for basic service availability.

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

**Example:**
```bash
curl http://localhost:9001/api/health
```

**Service Status Values:**
- `healthy` - Service is operating normally
- `degraded` - Service is functional but experiencing issues
- `unhealthy` - Service is not functioning properly

### GET /api/status
Comprehensive system status with detailed information.

**Response:**
```json
{
  "node_id": "12D3KooWABC123...",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "schemas_loaded": 3,
  "transforms_registered": 5,
  "connected_peers": 2,
  "storage": {
    "path": "data/db",
    "size_bytes": 1048576
  }
}
```

**Example:**
```bash
curl http://localhost:9001/api/status
```

## Metrics Endpoint

### GET /api/metrics
System performance metrics and operational statistics.

**Response:**
```json
{
  "operations": {
    "queries_total": 1250,
    "mutations_total": 340,
    "avg_response_time_ms": 25
  },
  "resources": {
    "memory_usage_bytes": 67108864,
    "cpu_usage_percent": 15.5
  },
  "network": {
    "bytes_sent": 2048576,
    "bytes_received": 1536000
  }
}
```

**Example:**
```bash
curl http://localhost:9001/api/metrics
```

**Metrics Categories:**

#### Operations Metrics
- `queries_total` - Total number of query operations executed
- `mutations_total` - Total number of mutation operations executed
- `avg_response_time_ms` - Average response time in milliseconds
- `errors_total` - Total number of errors encountered

#### Resource Metrics
- `memory_usage_bytes` - Current memory usage in bytes
- `cpu_usage_percent` - Current CPU usage percentage
- `disk_usage_bytes` - Current disk space usage
- `open_connections` - Number of active connections

#### Network Metrics
- `bytes_sent` - Total bytes sent over network
- `bytes_received` - Total bytes received over network
- `peer_connections` - Number of connected peers
- `network_errors` - Number of network-related errors

## System Control Endpoints

### POST /api/system/shutdown
Gracefully shutdown the node.

**Request Body:** (optional)
```json
{
  "delay_seconds": 30,
  "force": false
}
```

**Response:**
```json
{
  "success": true,
  "message": "Shutdown initiated",
  "shutdown_time": "2024-01-15T10:50:30Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/system/shutdown \
  -H "Content-Type: application/json" \
  -d '{"delay_seconds": 30}'
```

### POST /api/system/restart
Restart the node (if supported by the deployment environment).

**Response:**
```json
{
  "success": true,
  "message": "Restart initiated",
  "restart_time": "2024-01-15T10:50:30Z"
}
```

## Logging Endpoints

### GET /api/logs/stream
Stream real-time logs using Server-Sent Events (SSE).

**Response Format:**
```
Content-Type: text/event-stream

data: {"timestamp":"2024-01-15T10:50:00Z","level":"INFO","message":"Query executed successfully"}

data: {"timestamp":"2024-01-15T10:50:01Z","level":"DEBUG","message":"Transform triggered for field: age"}

data: {"timestamp":"2024-01-15T10:50:02Z","level":"ERROR","message":"Authentication failed for user"}
```

**Example:**
```bash
curl -N http://localhost:9001/api/logs/stream
```

**Log Levels:**
- `TRACE` - Detailed debugging information
- `DEBUG` - Debug information
- `INFO` - General information
- `WARN` - Warning messages
- `ERROR` - Error messages

### POST /api/logs/features
Update log level for specific features or modules.

**Request Body:**
```json
{
  "feature": "transform",
  "level": "TRACE"
}
```

**Response:**
```json
{
  "success": true,
  "feature": "transform",
  "new_level": "TRACE"
}
```

**Available Features:**
- `transform` - Transform execution logging
- `network` - Network operations logging
- `auth` - Authentication logging
- `schema` - Schema management logging
- `query` - Query execution logging

**Example:**
```bash
curl -X POST http://localhost:9001/api/logs/features \
  -H "Content-Type: application/json" \
  -d '{"feature": "transform", "level": "DEBUG"}'
```

### POST /api/logs/reload
Reload logging configuration from file.

**Response:**
```json
{
  "success": true,
  "message": "Logging configuration reloaded"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/logs/reload
```

### GET /api/logs/history
Get historical log entries with filtering.

**Query Parameters:**
- `level` - Filter by log level (TRACE, DEBUG, INFO, WARN, ERROR)
- `feature` - Filter by feature/module
- `since` - ISO timestamp for earliest log entry
- `until` - ISO timestamp for latest log entry
- `limit` - Maximum number of entries (default: 100)

**Response:**
```json
{
  "logs": [
    {
      "timestamp": "2024-01-15T10:50:00Z",
      "level": "INFO",
      "feature": "query",
      "message": "Query executed successfully",
      "details": {
        "schema": "UserProfile",
        "execution_time_ms": 15
      }
    }
  ],
  "total_count": 1250,
  "filtered_count": 1
}
```

**Example:**
```bash
curl "http://localhost:9001/api/logs/history?level=ERROR&limit=50"
```

## Monitoring Best Practices

### Health Checks
1. **Regular monitoring** - Check `/api/health` every 30-60 seconds
2. **Service dependencies** - Monitor individual service status
3. **Alert thresholds** - Set up alerts for `degraded` or `unhealthy` status

### Performance Monitoring
1. **Response times** - Monitor `avg_response_time_ms` trends
2. **Resource usage** - Track memory and CPU usage patterns
3. **Operation counts** - Monitor query/mutation volume
4. **Error rates** - Track error percentages over time

### Log Management
1. **Appropriate levels** - Use DEBUG/TRACE only for troubleshooting
2. **Log rotation** - Implement log rotation for long-running nodes
3. **Structured logging** - Use JSON format for easier parsing
4. **Retention policies** - Define log retention periods

## Error Responses

### System Errors
- `SERVICE_UNAVAILABLE`: System is shutting down or unavailable
- `INSUFFICIENT_RESOURCES`: System resources exhausted
- `CONFIGURATION_ERROR`: Invalid system configuration

**Example Error Response:**
```json
{
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "System is shutting down",
    "details": {
      "shutdown_initiated": "2024-01-15T10:50:00Z",
      "estimated_completion": "2024-01-15T10:50:30Z"
    }
  }
}
```

## Integration Examples

### Monitoring Script
```bash
#!/bin/bash
# Simple health check script

HEALTH_URL="http://localhost:9001/api/health"
STATUS=$(curl -s "$HEALTH_URL" | jq -r '.status')

if [ "$STATUS" = "healthy" ]; then
    echo "System is healthy"
    exit 0
else
    echo "System is not healthy: $STATUS"
    exit 1
fi
```

### Log Streaming with JavaScript
```javascript
const eventSource = new EventSource('http://localhost:9001/api/logs/stream');

eventSource.onmessage = function(event) {
    const logEntry = JSON.parse(event.data);
    console.log(`[${logEntry.level}] ${logEntry.message}`);
};

eventSource.onerror = function(event) {
    console.error('Log stream error:', event);
};
```

## Related Documentation

- [Network API](./network-api.md) - Network-specific monitoring
- [Data Operations API](./data-operations-api.md) - Query/mutation performance
- [Error Handling](./error-handling.md) - Error interpretation and troubleshooting
- [Deployment Guide](../deployment-guide.md) - Production monitoring setup

## Return to Index

[← Back to API Reference Index](./index.md)