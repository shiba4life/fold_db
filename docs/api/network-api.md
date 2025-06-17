# Network API

The Network API provides HTTP endpoints for managing peer-to-peer network connections, node discovery, and schema sharing between nodes.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Content-Type**: `application/json` for all POST/PUT requests

## Network Endpoints

### POST /api/network/start
Initialize and start the networking layer.

**Request Body:**
```json
{
  "port": 9000,
  "enable_mdns": true,
  "bootstrap_peers": [
    "/ip4/192.168.1.100/tcp/9000/p2p/12D3KooWGK8YLjL..."
  ]
}
```

**Response:**
```json
{
  "success": true,
  "node_id": "12D3KooWABC123...",
  "listening_addresses": [
    "/ip4/192.168.1.50/tcp/9000"
  ]
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/network/start \
  -H "Content-Type: application/json" \
  -d '{
    "port": 9000,
    "enable_mdns": true,
    "bootstrap_peers": []
  }'
```

### POST /api/network/discover
Discover peers on the local network.

**Request Body:**
```json
{
  "timeout_seconds": 10
}
```

**Response:**
```json
{
  "peers": [
    {
      "node_id": "12D3KooWGK8YLjL...",
      "addresses": ["/ip4/192.168.1.100/tcp/9000"],
      "discovered_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/network/discover \
  -H "Content-Type: application/json" \
  -d '{"timeout_seconds": 30}'
```

### POST /api/network/connect
Connect to a specific peer node.

**Request Body:**
```json
{
  "node_id": "12D3KooWGK8YLjL...",
  "address": "/ip4/192.168.1.100/tcp/9000"
}
```

**Response:**
```json
{
  "success": true,
  "connected_at": "2024-01-15T10:35:00Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/network/connect \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "12D3KooWGK8YLjL...",
    "address": "/ip4/192.168.1.100/tcp/9000"
  }'
```

### GET /api/network/status
Get current network status and connected peers.

**Response:**
```json
{
  "node_id": "12D3KooWABC123...",
  "listening_addresses": ["/ip4/192.168.1.50/tcp/9000"],
  "connected_peers": [
    {
      "node_id": "12D3KooWGK8YLjL...",
      "address": "/ip4/192.168.1.100/tcp/9000",
      "connected_at": "2024-01-15T10:35:00Z"
    }
  ],
  "network_active": true
}
```

**Example:**
```bash
curl http://localhost:9001/api/network/status
```

### POST /api/network/request-schema
Request a schema from a peer node.

**Request Body:**
```json
{
  "peer_id": "12D3KooWGK8YLjL...",
  "schema_name": "UserProfile"
}
```

**Response:**
```json
{
  "success": true,
  "schema": {
    "name": "UserProfile",
    "fields": {...},
    "received_at": "2024-01-15T10:40:00Z"
  }
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/network/request-schema \
  -H "Content-Type: application/json" \
  -d '{
    "peer_id": "12D3KooWGK8YLjL...",
    "schema_name": "UserProfile"
  }'
```

## Address Formats

### Multiaddr Format
DataFold uses the multiaddr format for peer addressing:

```
/ip4/192.168.1.100/tcp/9000
/ip6/::1/tcp/9000
/dns4/example.com/tcp/9000
```

### Node ID Format
Node IDs are libp2p peer identifiers:

```
12D3KooWGK8YLjL... (base58-encoded)
```

## Network Configuration

### Bootstrap Peers
Initial peers to connect to when starting the network:

```json
{
  "bootstrap_peers": [
    "/ip4/203.0.113.1/tcp/9000/p2p/12D3KooWBootstrap1...",
    "/dns4/bootstrap.example.com/tcp/9000/p2p/12D3KooWBootstrap2..."
  ]
}
```

### mDNS Discovery
Automatic local network discovery:

```json
{
  "enable_mdns": true
}
```

When enabled, nodes will automatically discover peers on the local network without requiring bootstrap peers.

## Error Responses

### Network Errors
- `PEER_NOT_FOUND`: Requested peer not available
- `CONNECTION_FAILED`: Failed to connect to peer
- `NETWORK_TIMEOUT`: Network operation timed out
- `INVALID_ADDRESS`: Address format is invalid
- `NETWORK_NOT_STARTED`: Network layer not initialized

**Example Error Response:**
```json
{
  "error": {
    "code": "CONNECTION_FAILED",
    "message": "Failed to connect to peer 12D3KooWGK8YLjL...",
    "details": {
      "peer_id": "12D3KooWGK8YLjL...",
      "address": "/ip4/192.168.1.100/tcp/9000",
      "reason": "Connection timeout"
    },
    "retry_after": 30
  }
}
```

## CLI Equivalents

Network operations have CLI command equivalents:

- Discovery ↔ [`datafold_cli discover-nodes`](./cli-interface.md#discover-nodes)
- Connection ↔ [`datafold_cli connect-node`](./cli-interface.md#connect-node)

## Best Practices

### Network Security
1. **Verify peer identities** before establishing connections
2. **Use TLS** for production deployments
3. **Implement rate limiting** for discovery operations
4. **Monitor connection counts** to prevent resource exhaustion

### Performance Optimization
1. **Limit concurrent connections** to prevent resource exhaustion
2. **Use persistent connections** for frequently accessed peers
3. **Cache peer discovery results** to reduce network overhead
4. **Implement connection pooling** for better resource management

## Related Documentation

- [Network Operations Guide](../network-operations.md) - Detailed networking concepts
- [Schema Management API](./schema-management-api.md) - Sharing schemas across network
- [Authentication](./authentication.md) - Securing network operations
- [Error Handling](./error-handling.md) - Network error troubleshooting

## Return to Index

[← Back to API Reference Index](./index.md)