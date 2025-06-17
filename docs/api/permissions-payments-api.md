# Permissions & Payments API

The Permissions & Payments API provides HTTP endpoints for managing access control through trust distance mechanisms and Lightning Network payment integration.

## Base Configuration

**Default URL**: `http://localhost:9001`
**Content-Type**: `application/json` for all POST/PUT requests

## Permission Endpoints

### POST /api/permissions/trust-distance
Set trust distance configuration for peer access control.

**Request Body:**
```json
{
  "default_distance": 1,
  "peer_distances": {
    "12D3KooWGK8YLjL...": 0,
    "12D3KooWABC123...": 2
  }
}
```

**Response:**
```json
{
  "success": true,
  "updated_peers": 2,
  "default_distance": 1
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/permissions/trust-distance \
  -H "Content-Type: application/json" \
  -d '{
    "default_distance": 1,
    "peer_distances": {
      "12D3KooWGK8YLjL...": 0
    }
  }'
```

### POST /api/permissions/explicit
Grant explicit permissions to specific users or public keys.

**Request Body:**
```json
{
  "schema": "UserProfile",
  "field": "email",
  "permission": "read",
  "public_key": "ed25519:ABC123...",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "success": true,
  "permission_id": "perm_123",
  "granted_at": "2024-01-15T10:45:00Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/permissions/explicit \
  -H "Content-Type: application/json" \
  -d '{
    "schema": "UserProfile",
    "field": "email", 
    "permission": "read",
    "public_key": "ed25519:ABC123...",
    "expires_at": "2024-12-31T23:59:59Z"
  }'
```

### GET /api/permissions
List current permission configuration.

**Response:**
```json
{
  "trust_distance": {
    "default_distance": 1,
    "peer_distances": {
      "12D3KooWGK8YLjL...": 0,
      "12D3KooWABC123...": 2
    }
  },
  "explicit_permissions": [
    {
      "id": "perm_123",
      "schema": "UserProfile",
      "field": "email",
      "permission": "read",
      "public_key": "ed25519:ABC123...",
      "expires_at": "2024-12-31T23:59:59Z",
      "granted_at": "2024-01-15T10:45:00Z"
    }
  ]
}
```

### DELETE /api/permissions/explicit/{permission_id}
Revoke an explicit permission.

**Response:**
```json
{
  "success": true,
  "message": "Permission revoked successfully"
}
```

## Payment Endpoints

### POST /api/payments/lightning/invoice
Generate Lightning Network invoice for paid operations.

**Request Body:**
```json
{
  "amount_sats": 1000,
  "description": "Access to UserProfile.email field",
  "expiry": 3600
}
```

**Response:**
```json
{
  "payment_request": "lnbc10u1p...",
  "payment_hash": "abc123...",
  "expires_at": "2024-01-15T11:50:00Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/payments/lightning/invoice \
  -H "Content-Type: application/json" \
  -d '{
    "amount_sats": 1000,
    "description": "Access to UserProfile.email field",
    "expiry": 3600
  }'
```

### POST /api/payments/verify
Verify payment for operation access.

**Request Body:**
```json
{
  "payment_hash": "abc123...",
  "operation": "query",
  "schema": "UserProfile",
  "fields": ["email"]
}
```

**Response:**
```json
{
  "verified": true,
  "access_granted": true,
  "expires_at": "2024-01-15T12:00:00Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:9001/api/payments/verify \
  -H "Content-Type: application/json" \
  -d '{
    "payment_hash": "abc123...",
    "operation": "query",
    "schema": "UserProfile",
    "fields": ["email"]
  }'
```

### GET /api/payments/status/{payment_hash}
Check payment status.

**Response:**
```json
{
  "payment_hash": "abc123...",
  "status": "paid|pending|expired",
  "amount_sats": 1000,
  "paid_at": "2024-01-15T11:30:00Z",
  "expires_at": "2024-01-15T11:50:00Z"
}
```

## Trust Distance System

### Trust Distance Levels
- **Distance 0**: Direct trust (self or highly trusted peers)
- **Distance 1**: One degree of separation (friends)
- **Distance 2**: Two degrees of separation (friends of friends)
- **Distance 3+**: Extended network with limited access

### Configuration Examples

#### Basic Trust Setup
```json
{
  "default_distance": 2,
  "peer_distances": {
    "12D3KooWTrustedPeer1...": 0,
    "12D3KooWTrustedPeer2...": 0,
    "12D3KooWFriendPeer...": 1
  }
}
```

#### Schema-Specific Trust Requirements
In schema definitions, fields can specify required trust distances:

```json
{
  "name": "UserProfile",
  "fields": {
    "username": {
      "field_type": "Single",
      "permission_policy": {
        "default_access": "read",
        "required_trust_distance": 2
      }
    },
    "email": {
      "field_type": "Single", 
      "permission_policy": {
        "default_access": "private",
        "required_trust_distance": 0
      }
    }
  }
}
```

## Lightning Network Integration

### Payment Flow
1. **Request operation** requiring payment
2. **Receive payment requirement** with amount and invoice
3. **Pay Lightning invoice** using Lightning wallet
4. **Verify payment** with payment hash
5. **Access granted** for specified duration

### Payment Configuration
```json
{
  "payment_config": {
    "lightning": {
      "enabled": true,
      "node_uri": "lightning-node:9735",
      "default_expiry_seconds": 3600
    },
    "pricing": {
      "base_cost_sats": 10,
      "per_field_cost_sats": 50,
      "per_query_cost_sats": 25
    }
  }
}
```

### Cost Calculation
- **Base operation cost**: Fixed cost per operation
- **Field access cost**: Cost per field accessed
- **Schema complexity**: Additional cost for complex schemas
- **Trust distance**: Higher cost for lower trust

**Example Cost Calculation:**
```javascript
const baseCost = 10; // sats
const fieldCost = 50; // sats per field
const trustMultiplier = trustDistance === 0 ? 1.0 : 
                       trustDistance === 1 ? 1.5 :
                       trustDistance === 2 ? 2.0 : 3.0;

const totalCost = (baseCost + (fieldCount * fieldCost)) * trustMultiplier;
```

## Permission Policies

### Access Levels
- **public**: No restrictions
- **read**: Read access with trust/payment requirements
- **private**: No access without explicit permission
- **restricted**: Custom access rules

### Policy Examples

#### Public Field
```json
{
  "field_name": "username",
  "permission_policy": {
    "default_access": "public",
    "required_trust_distance": 3
  },
  "payment_config": {
    "cost_per_access": 0
  }
}
```

#### Protected Field
```json
{
  "field_name": "email",
  "permission_policy": {
    "default_access": "read",
    "required_trust_distance": 1
  },
  "payment_config": {
    "cost_per_access": 100
  }
}
```

#### Private Field
```json
{
  "field_name": "ssn",
  "permission_policy": {
    "default_access": "private",
    "required_trust_distance": 0
  },
  "payment_config": {
    "cost_per_access": 1000
  }
}
```

## Error Responses

### Permission Errors
- `PERMISSION_DENIED`: Insufficient permissions for operation
- `TRUST_DISTANCE_EXCEEDED`: Required trust distance not met
- `EXPLICIT_PERMISSION_REQUIRED`: Explicit permission needed
- `PERMISSION_EXPIRED`: Permission has expired

### Payment Errors
- `PAYMENT_REQUIRED`: Payment needed for operation
- `INSUFFICIENT_PAYMENT`: Payment amount too low
- `PAYMENT_EXPIRED`: Payment has expired
- `PAYMENT_NOT_FOUND`: Payment hash not found
- `LIGHTNING_NODE_UNAVAILABLE`: Lightning node connection failed

**Example Error Response:**
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

## Integration Examples

### Python Payment Integration
```python
import requests
import json

# Request operation that requires payment
response = requests.post("http://localhost:9001/api/execute", 
    json={"operation": json.dumps({
        "type": "query",
        "schema": "UserProfile", 
        "fields": ["email"]
    })})

if response.status_code == 402:  # Payment Required
    payment_info = response.json()["error"]["details"]
    
    # Pay invoice using Lightning client
    payment_hash = pay_lightning_invoice(payment_info["payment_invoice"])
    
    # Verify payment and retry operation
    verify_response = requests.post("http://localhost:9001/api/payments/verify",
        json={
            "payment_hash": payment_hash,
            "operation": "query",
            "schema": "UserProfile",
            "fields": ["email"]
        })
    
    if verify_response.json()["verified"]:
        # Retry original operation
        response = requests.post("http://localhost:9001/api/execute", 
            json={"operation": json.dumps({
                "type": "query",
                "schema": "UserProfile",
                "fields": ["email"]
            })},
            headers={"X-Payment-Hash": payment_hash})
```

## Related Documentation

- [Authentication](./authentication.md) - Identity verification before access control
- [Schema Management API](./schema-management-api.md) - Configuring field permissions
- [Data Operations API](./data-operations-api.md) - Operations affected by permissions
- [Error Handling](./error-handling.md) - Permission and payment error troubleshooting
- [Permissions Guide](../permissions-and-fees.md) - Detailed permission concepts

## Return to Index

[← Back to API Reference Index](./index.md)