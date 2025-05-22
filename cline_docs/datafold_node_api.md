# DataFold Node API Documentation

This document provides a comprehensive guide to the DataFold Node API, enabling third-party applications to connect to a DataFold node and interact with its data through schemas.

## Overview

DataFold is a distributed database system that provides:
- Schema-based data storage with atomic operations
- Fine-grained permissions control at the field level
- Trust-based access control with explicit permissions and trust distance
- Version history tracking for data changes
- Pay-per-query access using Lightning Network
- Schema transformation and interpretation

The API allows third-party applications to:
1. Connect to a DataFold node
2. List available schemas
3. Query and mutate data
4. Create and manage schemas
5. Discover other nodes in the network

## Connection Details

### TCP Connection

DataFold nodes expose a TCP server that accepts JSON-based requests and responses. By default, the server listens on port 9000, but this can be configured.

To connect to a DataFold node:

1. Establish a TCP connection to the node's IP address and port
2. Send requests as JSON objects with a length prefix
3. Receive responses as JSON objects with a length prefix

### Message Format

All messages follow this binary format:
1. 4-byte unsigned integer (u32) representing the length of the JSON payload
2. JSON payload of the specified length

Example in pseudocode:
```
// Sending a request
let request = { "operation": "list_schemas" };
let request_json = json_encode(request);
let request_length = request_json.length;
send_u32(request_length);
send_bytes(request_json);

// Receiving a response
let response_length = receive_u32();
let response_json = receive_bytes(response_length);
let response = json_decode(response_json);
```

## API Operations

### 1. List Schemas

Lists all available schemas on the node.

**Request:**
```json
{
  "operation": "list_schemas"
}
```

**Response:**
```json
[
  "user_profile",
  "post",
  "comment",
  "product"
]
```

### 2. Get Schema

Retrieves the definition of a specific schema.

**Request:**
```json
{
  "operation": "get_schema",
  "params": {
    "schema_name": "user_profile"
  }
}
```

**Response:**
```json
{
  "name": "user_profile",
  "fields": {
    "username": {
      "permission_policy": {
        "read_policy": {"Distance": 0},
        "write_policy": {"Distance": 1}
      },
      "ref_atom_uuid": "550e8400-e29b-41d4-a716-446655440000",
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": "None",
        "min_payment": null
      },
      "field_mappers": {},
      "field_type": "Single"
    },
    "email": {
      "permission_policy": {
        "read_policy": {"Distance": 2},
        "write_policy": {"Distance": 0}
      },
      "ref_atom_uuid": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "payment_config": {
        "base_multiplier": 2.0,
        "trust_distance_scaling": {
          "Linear": {
            "min_factor": 1.0,
            "max_factor": 5.0
          }
        },
        "min_payment": 100
      },
      "field_mappers": {},
      "field_type": "Single"
    }
  },
  "payment_config": {
    "base_multiplier": 1.0
  }
}
```

### 3. Create Schema

Creates a new schema in the database.

**Request:**
```json
{
  "operation": "create_schema",
  "params": {
    "schema": {
      "name": "product",
      "fields": {
        "name": {
          "permission_policy": {
            "read_policy": {"Distance": 0},
            "write_policy": {"Distance": 1}
          },
          "ref_atom_uuid": "",
          "payment_config": {
            "base_multiplier": 1.0,
            "trust_distance_scaling": "None",
            "min_payment": null
          },
          "field_mappers": {},
          "field_type": "Single"
        },
        "price": {
          "permission_policy": {
            "read_policy": {"Distance": 0},
            "write_policy": {"Distance": 1}
          },
          "ref_atom_uuid": "",
          "payment_config": {
            "base_multiplier": 1.0,
            "trust_distance_scaling": "None",
            "min_payment": null
          },
          "field_mappers": {},
          "field_type": "Single"
        }
      },
      "payment_config": {
        "base_multiplier": 1.0
      }
    }
  }
}
```

**Response:**
```json
{
  "success": true
}
```

### 4. Update Schema

Updates an existing schema.

**Request:**
```json
{
  "operation": "update_schema",
  "params": {
    "schema": {
      "name": "product",
      "fields": {
        "name": {
          "permission_policy": {
            "read_policy": {"Distance": 0},
            "write_policy": {"Distance": 1}
          },
          "ref_atom_uuid": "550e8400-e29b-41d4-a716-446655440000",
          "payment_config": {
            "base_multiplier": 1.0,
            "trust_distance_scaling": "None",
            "min_payment": null
          },
          "field_mappers": {},
          "field_type": "Single"
        },
        "price": {
          "permission_policy": {
            "read_policy": {"Distance": 0},
            "write_policy": {"Distance": 1}
          },
          "ref_atom_uuid": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
          "payment_config": {
            "base_multiplier": 1.0,
            "trust_distance_scaling": "None",
            "min_payment": null
          },
          "field_mappers": {},
          "field_type": "Single"
        },
        "description": {
          "permission_policy": {
            "read_policy": {"Distance": 0},
            "write_policy": {"Distance": 1}
          },
          "ref_atom_uuid": "",
          "payment_config": {
            "base_multiplier": 1.0,
            "trust_distance_scaling": "None",
            "min_payment": null
          },
          "field_mappers": {},
          "field_type": "Single"
        }
      },
      "payment_config": {
        "base_multiplier": 1.0
      }
    }
  }
}
```

**Response:**
```json
{
  "success": true
}
```

### 5. Unload Schema

Marks a schema as unloaded so it is no longer queryable.

**Request:**
```json
{
  "operation": "unload_schema",
  "params": {
    "schema_name": "product"
  }
}
```

**Response:**
```json
{
  "success": true
}
```

### 6. Query

Queries data from a schema.

**Request:**
```json
{
  "operation": "query",
  "params": {
    "schema": "user_profile",
    "fields": ["username", "email"],
    "filter": {
      "username": "john_doe"
    }
  }
}
```

**Response:**
```json
{
  "results": [
    ["john_doe", "john@example.com"],
    ["john_smith", "smith@example.com"]
  ],
  "errors": []
}
```

### 7. Mutation

Performs a mutation (create, update, delete) on a schema.

**Request:**
```json
{
  "operation": "mutation",
  "params": {
    "schema": "user_profile",
    "data": {
      "username": "jane_doe",
      "email": "jane@example.com"
    },
    "mutation_type": "create"
  }
}
```

**Response:**
```json
{
  "success": true,
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### 8. Discover Nodes

Discovers other nodes on the network.

**Request:**
```json
{
  "operation": "discover_nodes"
}
```

**Response:**
```json
[
  {
    "id": "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N",
    "trust_distance": 1
  },
  {
    "id": "QmZMxNdpMkewiVZLMRxaNxUeZpDUb34pWjZ1kZvsd16Zic",
    "trust_distance": 2
  }
]
```

## Cross-Node Communication

DataFold nodes can forward requests to other nodes in the network. To target a specific node, include the `target_node_id` field in your request:

```json
{
  "operation": "query",
  "target_node_id": "550e8400-e29b-41d4-a716-446655440000",
  "params": {
    "schema": "user_profile",
    "fields": ["username", "email"]
  }
}
```

The node will forward the request to the target node and return the response.

## Schema Structure

### Schema Definition

A schema defines the structure of data in the database:

```json
{
  "name": "schema_name",
  "fields": {
    "field_name": {
      "permission_policy": {
        "read_policy": {"Distance": 0},
        "write_policy": {"Distance": 1},
        "explicit_read_policy": null,
        "explicit_write_policy": null
      },
      "ref_atom_uuid": "uuid-string",
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": "None",
        "min_payment": null
      },
      "field_mappers": {},
      "field_type": "Single"
    }
  },
  "payment_config": {
    "base_multiplier": 1.0
  }
}
```

### Field Types

- `Single`: A single value
- `Collection`: A collection of values

### Permission Policies

Permission policies control access to fields:

```json
{
  "read_policy": {"Distance": 0},
  "write_policy": {"Distance": 1},
  "explicit_read_policy": null,
  "explicit_write_policy": null
}
```

- `Distance`: Trust distance (lower means higher trust)
- `ExplicitOnly`: Only explicit permissions allowed
- `None`: No access allowed

### Payment Configuration

Payment configuration defines how payments are calculated:

```json
{
  "base_multiplier": 1.0,
  "trust_distance_scaling": "None",
  "min_payment": null
}
```

Trust distance scaling can be:
- `None`: No scaling
- `Linear`: Linear scaling with min and max factors
- `Exponential`: Exponential scaling with min and max factors

## Error Handling

Errors are returned as JSON objects with an `error` field:

```json
{
  "error": "Schema not found: user_profile"
}
```

## Implementation Examples

### Connecting to a Node in Python

```python
import socket
import json
import struct

def send_request(sock, request):
    # Serialize the request to JSON
    request_json = json.dumps(request).encode('utf-8')
    
    # Send the length prefix
    sock.sendall(struct.pack('!I', len(request_json)))
    
    # Send the JSON payload
    sock.sendall(request_json)
    
    # Receive the response length
    response_len_bytes = sock.recv(4)
    response_len = struct.unpack('!I', response_len_bytes)[0]
    
    # Receive the response
    response_json = sock.recv(response_len)
    
    # Deserialize the response
    return json.loads(response_json)

# Connect to the node
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(('localhost', 9000))

# List schemas
request = {
    "operation": "list_schemas"
}
response = send_request(sock, request)
print("Available schemas:", response)

# Close the connection
sock.close()
```

### Connecting to a Node in JavaScript

```javascript
const net = require('net');

function sendRequest(socket, request) {
    return new Promise((resolve, reject) => {
        // Serialize the request to JSON
        const requestJson = JSON.stringify(request);
        const requestBuffer = Buffer.from(requestJson, 'utf-8');
        
        // Create a buffer for the length prefix
        const lengthBuffer = Buffer.alloc(4);
        lengthBuffer.writeUInt32BE(requestBuffer.length, 0);
        
        // Send the length prefix and JSON payload
        socket.write(Buffer.concat([lengthBuffer, requestBuffer]));
        
        // Prepare to receive the response
        let responseLength = null;
        let responseBuffer = Buffer.alloc(0);
        
        socket.on('data', (data) => {
            if (responseLength === null) {
                // Read the length prefix
                responseLength = data.readUInt32BE(0);
                responseBuffer = data.slice(4);
            } else {
                // Append to the response buffer
                responseBuffer = Buffer.concat([responseBuffer, data]);
            }
            
            // Check if we've received the complete response
            if (responseBuffer.length >= responseLength) {
                const response = JSON.parse(responseBuffer.toString('utf-8', 0, responseLength));
                resolve(response);
            }
        });
        
        socket.on('error', (err) => {
            reject(err);
        });
    });
}

// Connect to the node
const socket = net.createConnection({ host: 'localhost', port: 9000 }, async () => {
    try {
        // List schemas
        const request = {
            operation: 'list_schemas'
        };
        const response = await sendRequest(socket, request);
        console.log('Available schemas:', response);
        
        // Close the connection
        socket.end();
    } catch (err) {
        console.error('Error:', err);
        socket.end();
    }
});
```

### Connecting to a Node in Rust

```rust
use std::io::{Read, Write};
use std::net::TcpStream;
use serde_json::{json, Value};

fn send_request(stream: &mut TcpStream, request: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    // Serialize the request to JSON
    let request_json = serde_json::to_vec(request)?;
    
    // Send the length prefix
    let request_len = request_json.len() as u32;
    stream.write_all(&request_len.to_be_bytes())?;
    
    // Send the JSON payload
    stream.write_all(&request_json)?;
    
    // Receive the response length
    let mut response_len_bytes = [0u8; 4];
    stream.read_exact(&mut response_len_bytes)?;
    let response_len = u32::from_be_bytes(response_len_bytes) as usize;
    
    // Receive the response
    let mut response_json = vec![0u8; response_len];
    stream.read_exact(&mut response_json)?;
    
    // Deserialize the response
    let response: Value = serde_json::from_slice(&response_json)?;
    
    Ok(response)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the node
    let mut stream = TcpStream::connect("localhost:9000")?;
    
    // List schemas
    let request = json!({
        "operation": "list_schemas"
    });
    let response = send_request(&mut stream, &request)?;
    println!("Available schemas: {:?}", response);
    
    Ok(())
}
```

## Security Considerations

1. **Authentication**: The API currently uses public key authentication. Include your public key in the `pub_key` field for operations that require authentication.

2. **Trust Distance**: Operations are subject to trust distance validation. Lower trust distance means higher trust and more access.

3. **Permissions**: Fields have read and write permissions that are enforced based on trust distance and explicit permissions.

4. **Payments**: Some operations may require payments through the Lightning Network. The payment amount is calculated based on the field's payment configuration and trust distance.

## Best Practices

1. **Connection Management**: Maintain a persistent connection to the node when possible to reduce connection overhead.

2. **Error Handling**: Always check for errors in responses and handle them appropriately.

3. **Schema Discovery**: Use the `list_schemas` operation to discover available schemas before attempting to query or mutate data.

4. **Field Selection**: Only request the fields you need to minimize data transfer and payment requirements.

5. **Trust Management**: Be aware of trust distances and their impact on permissions and payments.

6. **Cross-Node Communication**: Use the `target_node_id` field to target specific nodes in the network.

## Conclusion

The DataFold Node API provides a powerful interface for interacting with the distributed database system. By following this documentation, third-party applications can connect to a DataFold node, list available schemas, and perform operations on the data.
