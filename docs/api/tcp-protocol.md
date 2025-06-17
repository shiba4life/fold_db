# TCP Protocol

The DataFold TCP protocol provides a high-performance binary interface for real-time data operations using length-prefixed JSON messages.

## Connection Configuration

**Default Port**: `9000`
**Protocol**: Binary with length-prefixed JSON messages
**Encoding**: UTF-8 for JSON payloads

## Message Format

All messages use the following binary format:

1. **Length Prefix**: 4 bytes (u32, little-endian) indicating JSON payload length
2. **JSON Payload**: UTF-8 encoded JSON message

```
┌─────────────┬─────────────────────┐
│ Length (4B) │ JSON Payload (UTF-8)│
│ Little-End  │                     │
└─────────────┴─────────────────────┘
```

### Example Binary Layout
```
[0x1A, 0x00, 0x00, 0x00] [{"app_id":"test",...}]
     26 bytes length           JSON payload
```

## Request Format

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

**Required Fields:**
- `app_id`: Client application identifier
- `operation`: Operation type to execute
- `params`: Operation-specific parameters

**Optional Fields:**
- `signature`: Ed25519 signature for authentication
- `timestamp`: Unix timestamp for replay protection
- `public_key`: Public key for signature verification
- `nonce`: Additional replay protection

## Response Format

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

## Supported Operations

### list_schemas
List all loaded schemas.

**Request:**
```json
{
  "app_id": "my-app",
  "operation": "list_schemas",
  "params": {}
}
```

**Response:**
```json
{
  "results": [
    {"name": "UserProfile", "fields": 5},
    {"name": "EventAnalytics", "fields": 4}
  ],
  "errors": []
}
```

### get_schema
Get detailed schema information.

**Request:**
```json
{
  "app_id": "my-app",
  "operation": "get_schema",
  "params": {
    "schema_name": "UserProfile"
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "name": "UserProfile",
      "fields": {
        "username": {
          "field_type": "Single",
          "permission_policy": {...}
        }
      }
    }
  ],
  "errors": []
}
```

### create_schema
Load a new schema into the system.

**Request:**
```json
{
  "app_id": "my-app",
  "operation": "create_schema",
  "params": {
    "schema": {
      "name": "UserProfile",
      "fields": {
        "username": {
          "field_type": "Single",
          "permission_policy": {...}
        }
      }
    }
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "success": true,
      "schema_name": "UserProfile"
    }
  ],
  "errors": []
}
```

### query
Execute a data query.

**Request:**
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
    "rows_returned": 1
  }
}
```

### mutation
Execute a data mutation (create, update, delete).

**Request:**
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

**Response:**
```json
{
  "results": [
    {
      "success": true,
      "rows_affected": 1
    }
  ],
  "errors": []
}
```

### discover_nodes
Discover peer nodes on the network.

**Request:**
```json
{
  "app_id": "my-app",
  "operation": "discover_nodes", 
  "params": {
    "timeout_seconds": 10
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "node_id": "12D3KooWGK8YLjL...",
      "addresses": ["/ip4/192.168.1.100/tcp/9000"],
      "discovered_at": "2024-01-15T10:30:00Z"
    }
  ],
  "errors": []
}
```

## Authentication

### Public Key Authentication
Include authentication fields in TCP messages:

```json
{
  "app_id": "my-app",
  "operation": "query",
  "params": {...},
  "public_key": "ed25519:ABC123...",
  "signature": "ed25519:signature-hash",
  "timestamp": 1609459200
}
```

**Signature Creation:**
1. Serialize operation parameters to JSON
2. Create payload: `operation|params_json|timestamp`
3. Sign payload with Ed25519 private key
4. Include public key and signature in message

## Client Libraries

### Python Client Example

```python
import socket
import json
import struct
import ed25519
import time

class FoldDBClient:
    def __init__(self, host='localhost', port=9000):
        self.host = host
        self.port = port
        self.sock = None
        self.private_key = None
        self.public_key = None
    
    def set_auth_keys(self, private_key_bytes, public_key_bytes):
        self.private_key = ed25519.SigningKey(private_key_bytes)
        self.public_key = public_key_bytes
    
    def connect(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))
    
    def disconnect(self):
        if self.sock:
            self.sock.close()
            self.sock = None
    
    def send_request(self, operation, params, app_id="python-client"):
        request = {
            "app_id": app_id,
            "operation": operation,
            "params": params
        }
        
        # Add authentication if keys are set
        if self.private_key and self.public_key:
            timestamp = int(time.time())
            params_json = json.dumps(params, sort_keys=True)
            payload = f"{operation}|{params_json}|{timestamp}"
            signature = self.private_key.sign(payload.encode('utf-8'))
            
            request.update({
                "public_key": f"ed25519:{base64.b64encode(self.public_key).decode()}",
                "signature": f"ed25519:{base64.b64encode(signature).decode()}",
                "timestamp": timestamp
            })
        
        # Serialize and send
        request_json = json.dumps(request).encode('utf-8')
        length_prefix = struct.pack('<I', len(request_json))
        
        self.sock.sendall(length_prefix)
        self.sock.sendall(request_json)
        
        # Receive response
        response_len_bytes = self.sock.recv(4)
        response_len = struct.unpack('<I', response_len_bytes)[0]
        
        response_json = b''
        while len(response_json) < response_len:
            chunk = self.sock.recv(response_len - len(response_json))
            if not chunk:
                raise ConnectionError("Connection closed unexpectedly")
            response_json += chunk
        
        return json.loads(response_json.decode('utf-8'))
    
    def query(self, schema, fields, filter_=None):
        params = {
            "schema": schema,
            "fields": fields
        }
        if filter_:
            params["filter"] = filter_
        
        return self.send_request("query", params)
    
    def create(self, schema, data):
        params = {
            "schema": schema,
            "mutation_type": "create",
            "data": data
        }
        return self.send_request("mutation", params)
    
    def list_schemas(self):
        return self.send_request("list_schemas", {})

# Usage example
client = FoldDBClient()
client.connect()

# Query without authentication
result = client.query("UserProfile", ["username", "email"])
print("Query result:", result)

# Create user
result = client.create("UserProfile", {
    "username": "alice",
    "email": "alice@example.com"
})
print("Create result:", result)

# List schemas
schemas = client.list_schemas()
print("Available schemas:", schemas)

client.disconnect()
```

### JavaScript Client Example

```javascript
const net = require('net');

class FoldDBClient {
    constructor(host = 'localhost', port = 9000) {
        this.host = host;
        this.port = port;
        this.socket = null;
    }
    
    connect() {
        return new Promise((resolve, reject) => {
            this.socket = net.createConnection(this.port, this.host);
            this.socket.on('connect', resolve);
            this.socket.on('error', reject);
        });
    }
    
    disconnect() {
        if (this.socket) {
            this.socket.end();
            this.socket = null;
        }
    }
    
    sendRequest(operation, params, appId = 'node-client') {
        return new Promise((resolve, reject) => {
            const request = {
                app_id: appId,
                operation: operation,
                params: params
            };
            
            const requestJson = JSON.stringify(request);
            const requestBuffer = Buffer.from(requestJson, 'utf8');
            const lengthBuffer = Buffer.allocUnsafe(4);
            lengthBuffer.writeUInt32LE(requestBuffer.length, 0);
            
            // Send request
            this.socket.write(lengthBuffer);
            this.socket.write(requestBuffer);
            
            // Handle response
            let responseLength = null;
            let responseBuffer = Buffer.alloc(0);
            
            const onData = (data) => {
                responseBuffer = Buffer.concat([responseBuffer, data]);
                
                // Read length if not yet known
                if (responseLength === null && responseBuffer.length >= 4) {
                    responseLength = responseBuffer.readUInt32LE(0);
                    responseBuffer = responseBuffer.slice(4);
                }
                
                // Check if complete response received
                if (responseLength !== null && responseBuffer.length >= responseLength) {
                    this.socket.removeListener('data', onData);
                    const responseJson = responseBuffer.slice(0, responseLength).toString('utf8');
                    resolve(JSON.parse(responseJson));
                }
            };
            
            this.socket.on('data', onData);
            this.socket.on('error', reject);
        });
    }
    
    async query(schema, fields, filter = null) {
        const params = { schema, fields };
        if (filter) params.filter = filter;
        return this.sendRequest('query', params);
    }
    
    async create(schema, data) {
        const params = {
            schema,
            mutation_type: 'create',
            data
        };
        return this.sendRequest('mutation', params);
    }
}

// Usage example
(async () => {
    const client = new FoldDBClient();
    await client.connect();
    
    try {
        const result = await client.query('UserProfile', ['username', 'email']);
        console.log('Query result:', result);
        
        const createResult = await client.create('UserProfile', {
            username: 'bob',
            email: 'bob@example.com'
        });
        console.log('Create result:', createResult);
    } finally {
        client.disconnect();
    }
})();
```

## Error Handling

TCP protocol errors are returned in the standard response format:

```json
{
  "results": [],
  "errors": [
    {
      "code": "SCHEMA_NOT_FOUND",
      "message": "Schema 'InvalidSchema' not found",
      "details": {
        "schema_name": "InvalidSchema"
      }
    }
  ]
}
```

## Performance Considerations

### Connection Management
- **Persistent connections**: Reuse connections for multiple operations
- **Connection pooling**: Manage multiple connections for high throughput
- **Graceful disconnection**: Properly close connections when done

### Message Optimization
- **Batch operations**: Group multiple operations in single requests where possible
- **Minimize payload size**: Only request needed fields
- **Compression**: Consider implementing compression for large payloads

### Error Recovery
- **Connection retry**: Implement reconnection logic for network failures
- **Operation retry**: Retry failed operations with exponential backoff
- **Circuit breaker**: Prevent cascading failures

## Related Documentation

- [Data Operations API](./data-operations-api.md) - HTTP equivalents of TCP operations
- [Authentication](./authentication.md) - TCP authentication methods
- [Request/Response Formats](./request-response-formats.md) - Detailed format specifications
- [Error Handling](./error-handling.md) - Error codes and troubleshooting

## Return to Index

[← Back to API Reference Index](./index.md)