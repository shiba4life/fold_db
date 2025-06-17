# Error Handling

This document provides comprehensive information about error codes, error response formats, and troubleshooting guidance for the DataFold API.

## Error Response Format

All errors follow a consistent response format across HTTP and TCP interfaces:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "field_name",
      "expected": "expected_value",
      "actual": "actual_value"
    },
    "retry_after": 30
  }
}
```

**Fields:**
- `code`: Machine-readable error code
- `message`: Human-readable error description
- `details`: Additional context about the error
- `retry_after`: Suggested retry delay in seconds (optional)

## Error Categories

### Schema Errors

#### SCHEMA_NOT_FOUND
Requested schema does not exist in the system.

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

**Troubleshooting:**
1. Verify the schema name spelling and case
2. Check available schemas with `GET /api/schemas`
3. Load the schema if it should exist

#### SCHEMA_VALIDATION_FAILED
Schema definition contains invalid or malformed data.

```json
{
  "error": {
    "code": "SCHEMA_VALIDATION_FAILED",
    "message": "Invalid field type in schema definition",
    "details": {
      "field": "invalid_field",
      "error": "Unknown field type: 'InvalidType'"
    }
  }
}
```

**Troubleshooting:**
1. Validate JSON syntax in schema definition
2. Check field types are valid (Single, Collection, Range)
3. Verify all required fields are present

#### SCHEMA_ALREADY_EXISTS
Attempting to create a schema with a name that already exists.

```json
{
  "error": {
    "code": "SCHEMA_ALREADY_EXISTS",
    "message": "Schema 'UserProfile' already exists",
    "details": {
      "schema_name": "UserProfile",
      "loaded_at": "2024-01-15T10:30:00Z"
    }
  }
}
```

**Troubleshooting:**
1. Use a different schema name
2. Unload the existing schema first if replacement is intended
3. Check if the existing schema meets your needs

### Permission Errors

#### PERMISSION_DENIED
Insufficient permissions for the requested operation.

```json
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Insufficient permissions for field: email",
    "details": {
      "field": "email",
      "required_trust_distance": 0,
      "current_trust_distance": 2,
      "permission_type": "read"
    }
  }
}
```

**Troubleshooting:**
1. Check trust distance configuration
2. Request explicit permission for the field
3. Consider payment-based access if available

#### TRUST_DISTANCE_EXCEEDED
Required trust distance not met for operation.

```json
{
  "error": {
    "code": "TRUST_DISTANCE_EXCEEDED",
    "message": "Trust distance 2 exceeds required distance 1",
    "details": {
      "required_distance": 1,
      "current_distance": 2,
      "peer_id": "12D3KooWABC123..."
    }
  }
}
```

**Troubleshooting:**
1. Establish closer trust relationship with data owner
2. Request explicit permission
3. Use payment-based access alternative

#### EXPLICIT_PERMISSION_REQUIRED
Operation requires explicit permission grant.

```json
{
  "error": {
    "code": "EXPLICIT_PERMISSION_REQUIRED",
    "message": "Explicit permission required for private field",
    "details": {
      "field": "ssn",
      "schema": "UserProfile",
      "permission_endpoint": "/api/permissions/explicit"
    }
  }
}
```

**Troubleshooting:**
1. Request explicit permission from data owner
2. Use the permissions API to grant access
3. Check if payment-based access is available

### Payment Errors

#### PAYMENT_REQUIRED
Payment needed to access the requested operation or field.

```json
{
  "error": {
    "code": "PAYMENT_REQUIRED",
    "message": "Payment required for field access",
    "details": {
      "schema": "UserProfile",
      "field": "email",
      "required_amount_sats": 100,
      "payment_invoice": "lnbc1u1p...",
      "payment_hash": "abc123..."
    }
  }
}
```

**Troubleshooting:**
1. Pay the Lightning Network invoice
2. Verify payment with the payment hash
3. Retry operation with payment proof

#### INSUFFICIENT_PAYMENT
Payment amount is below the required threshold.

```json
{
  "error": {
    "code": "INSUFFICIENT_PAYMENT",
    "message": "Payment amount too low",
    "details": {
      "paid_amount_sats": 50,
      "required_amount_sats": 100,
      "shortfall_sats": 50
    }
  }
}
```

**Troubleshooting:**
1. Generate new invoice for correct amount
2. Pay the difference if supported
3. Check payment calculation logic

#### PAYMENT_EXPIRED
Payment or payment request has expired.

```json
{
  "error": {
    "code": "PAYMENT_EXPIRED",
    "message": "Payment has expired",
    "details": {
      "payment_hash": "abc123...",
      "expired_at": "2024-01-15T11:50:00Z",
      "current_time": "2024-01-15T12:00:00Z"
    }
  }
}
```

**Troubleshooting:**
1. Generate new payment invoice
2. Pay within expiration window
3. Increase expiry time for future payments

### Network Errors

#### PEER_NOT_FOUND
Requested peer node is not available or discoverable.

```json
{
  "error": {
    "code": "PEER_NOT_FOUND",
    "message": "Peer 12D3KooWGK8YLjL... not found",
    "details": {
      "peer_id": "12D3KooWGK8YLjL...",
      "last_seen": "2024-01-15T10:00:00Z"
    }
  }
}
```

**Troubleshooting:**
1. Verify peer ID is correct
2. Check network connectivity
3. Try peer discovery again
4. Check if peer is online

#### CONNECTION_FAILED
Failed to establish connection to peer node.

```json
{
  "error": {
    "code": "CONNECTION_FAILED",
    "message": "Failed to connect to peer",
    "details": {
      "peer_id": "12D3KooWGK8YLjL...",
      "address": "/ip4/192.168.1.100/tcp/9000",
      "reason": "Connection timeout"
    },
    "retry_after": 30
  }
}
```

**Troubleshooting:**
1. Check network connectivity to peer address
2. Verify peer is accepting connections
3. Check firewall and port configuration
4. Retry with exponential backoff

#### NETWORK_TIMEOUT
Network operation exceeded timeout limit.

```json
{
  "error": {
    "code": "NETWORK_TIMEOUT",
    "message": "Network operation timed out",
    "details": {
      "operation": "peer_discovery",
      "timeout_seconds": 30,
      "elapsed_seconds": 30
    }
  }
}
```

**Troubleshooting:**
1. Increase timeout for the operation
2. Check network latency and connectivity
3. Retry operation
4. Check for network congestion

### Data Errors

#### FIELD_NOT_FOUND
Requested field does not exist in the schema.

```json
{
  "error": {
    "code": "FIELD_NOT_FOUND",
    "message": "Field 'invalid_field' not found in schema",
    "details": {
      "field": "invalid_field",
      "schema": "UserProfile",
      "available_fields": ["username", "email", "age"]
    }
  }
}
```

**Troubleshooting:**
1. Check field name spelling and case
2. Verify field exists in schema definition
3. Check available fields with schema info

#### INVALID_FILTER
Filter syntax or logic is malformed.

```json
{
  "error": {
    "code": "INVALID_FILTER",
    "message": "Invalid filter operator",
    "details": {
      "filter": {"field": "age", "operator": "invalid_op", "value": 18},
      "valid_operators": ["eq", "ne", "gt", "gte", "lt", "lte"]
    }
  }
}
```

**Troubleshooting:**
1. Check filter syntax matches documentation
2. Verify operator is supported for field type
3. Validate filter value type matches field type

#### TYPE_MISMATCH
Data type does not match expected field type.

```json
{
  "error": {
    "code": "TYPE_MISMATCH",
    "message": "Type mismatch for field 'age'",
    "details": {
      "field": "age",
      "expected_type": "number",
      "actual_type": "string",
      "value": "twenty-five"
    }
  }
}
```

**Troubleshooting:**
1. Convert data to correct type before sending
2. Validate input data against schema
3. Check field type definitions

### Authentication Errors

#### AUTHENTICATION_REQUIRED
No authentication provided when required.

```json
{
  "error": {
    "code": "AUTHENTICATION_REQUIRED",
    "message": "Authentication required for this operation",
    "details": {
      "supported_methods": ["api_key", "signature"],
      "auth_endpoint": "/api/auth"
    }
  }
}
```

**Troubleshooting:**
1. Add authentication headers or fields
2. Check authentication configuration
3. Verify API key or signature format

#### INVALID_SIGNATURE
Cryptographic signature validation failed.

```json
{
  "error": {
    "code": "INVALID_SIGNATURE",
    "message": "Signature validation failed",
    "details": {
      "public_key": "ed25519:ABC123...",
      "timestamp": 1609459200,
      "expected_format": "method|path|body|timestamp"
    }
  }
}
```

**Troubleshooting:**
1. Verify signature calculation logic
2. Check timestamp is current
3. Ensure payload format matches expected
4. Verify private/public key pair

#### EXPIRED_TIMESTAMP
Request timestamp is outside acceptable window.

```json
{
  "error": {
    "code": "EXPIRED_TIMESTAMP",
    "message": "Request timestamp too old",
    "details": {
      "timestamp": 1609459200,
      "current_time": 1609459500,
      "max_age_seconds": 300
    }
  }
}
```

**Troubleshooting:**
1. Use current timestamp in requests
2. Check system clock synchronization
3. Reduce time between signature and request

### System Errors

#### SERVICE_UNAVAILABLE
System is temporarily unavailable.

```json
{
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "System is shutting down",
    "details": {
      "shutdown_initiated": "2024-01-15T10:50:00Z",
      "estimated_completion": "2024-01-15T10:50:30Z"
    },
    "retry_after": 60
  }
}
```

**Troubleshooting:**
1. Wait for system to become available
2. Check system status endpoints
3. Retry after suggested delay

#### INSUFFICIENT_RESOURCES
System resources are exhausted.

```json
{
  "error": {
    "code": "INSUFFICIENT_RESOURCES",
    "message": "Memory limit exceeded",
    "details": {
      "resource": "memory",
      "current_usage": "95%",
      "limit": "8GB"
    }
  }
}
```

**Troubleshooting:**
1. Reduce request size or complexity
2. Retry operation later
3. Contact system administrator

## HTTP Status Codes

### Success Codes
- `200 OK`: Request succeeded
- `201 Created`: Resource created successfully
- `202 Accepted`: Request accepted for processing

### Client Error Codes
- `400 Bad Request`: Invalid request format or parameters
- `401 Unauthorized`: Authentication required or invalid
- `403 Forbidden`: Permission denied for operation
- `404 Not Found`: Resource not found
- `422 Unprocessable Entity`: Valid request but semantic errors

### Payment Codes
- `402 Payment Required`: Payment needed for operation

### Rate Limiting
- `429 Too Many Requests`: Rate limit exceeded

### Server Error Codes
- `500 Internal Server Error`: Unexpected server error
- `502 Bad Gateway`: Upstream service error
- `503 Service Unavailable`: Service temporarily unavailable
- `504 Gateway Timeout`: Upstream service timeout

## Error Recovery Strategies

### Retry Logic
Implement exponential backoff for transient errors:

```python
import time
import random

def retry_with_backoff(func, max_retries=3, base_delay=1):
    for attempt in range(max_retries):
        try:
            return func()
        except Exception as e:
            if attempt == max_retries - 1:
                raise e
            
            # Exponential backoff with jitter
            delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
            time.sleep(delay)
```

### Circuit Breaker Pattern
Prevent cascading failures:

```python
class CircuitBreaker:
    def __init__(self, failure_threshold=5, timeout=60):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.failure_count = 0
        self.last_failure_time = None
        self.state = 'CLOSED'  # CLOSED, OPEN, HALF_OPEN
    
    def call(self, func):
        if self.state == 'OPEN':
            if time.time() - self.last_failure_time > self.timeout:
                self.state = 'HALF_OPEN'
            else:
                raise Exception("Circuit breaker is OPEN")
        
        try:
            result = func()
            if self.state == 'HALF_OPEN':
                self.state = 'CLOSED'
                self.failure_count = 0
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            
            if self.failure_count >= self.failure_threshold:
                self.state = 'OPEN'
            
            raise e
```

### Error Logging
Log errors for debugging and monitoring:

```python
import logging

def log_api_error(error_response, request_info):
    logging.error(
        "API Error: %s - %s",
        error_response.get('code'),
        error_response.get('message'),
        extra={
            'error_code': error_response.get('code'),
            'error_details': error_response.get('details'),
            'request_method': request_info.get('method'),
            'request_url': request_info.get('url'),
            'request_data': request_info.get('data')
        }
    )
```

## Troubleshooting Tools

### CLI Diagnostic Commands
```bash
# Test connectivity
datafold_cli health-check

# Test authentication
datafold_cli auth test-signature --private-key keys/private.pem

# Validate schema
datafold_cli validate-schema schema.json

# Check permissions
datafold_cli permissions check --schema UserProfile --field email
```

### API Testing
```bash
# Test API health
curl http://localhost:9001/api/health

# Test authentication
curl -H "Authorization: Bearer test-key" \
  http://localhost:9001/api/schemas

# Test network connectivity
curl -X POST http://localhost:9001/api/network/discover
```

## Related Documentation

- [Authentication](./authentication.md) - Authentication error resolution
- [Permissions & Payments API](./permissions-payments-api.md) - Permission and payment errors
- [Data Operations API](./data-operations-api.md) - Data operation errors
- [Network API](./network-api.md) - Network-related errors
- [System Monitoring API](./system-monitoring-api.md) - System status and health

## Return to Index

[← Back to API Reference Index](./index.md)