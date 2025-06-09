# CLI Server Integration Guide

This guide covers the DataFold CLI commands for integrating with the DataFold server for public key registration and signature verification.

## Overview

The DataFold CLI provides comprehensive server integration functionality that enables:

- **Public Key Registration**: Register your Ed25519 public keys with the DataFold server
- **Registration Status Checking**: Check the status of registered public keys
- **Message Signing & Verification**: Sign messages locally and verify signatures with the server
- **End-to-End Testing**: Complete workflow testing with automated integration tests

## Prerequisites

1. **DataFold Server Running**: Ensure the DataFold HTTP server is running (typically on `http://localhost:8080`)
2. **Key Storage**: Have keys stored locally using the `store-key` command
3. **Network Access**: CLI must be able to reach the DataFold server endpoints

## Commands

### 1. Register Public Key (`register-key`)

Register a locally stored public key with the DataFold server for authentication and signature verification.

```bash
# Basic registration
datafold_cli register-key \
    --key-id my_signing_key \
    --server-url http://localhost:8080

# Registration with custom client ID and metadata
datafold_cli register-key \
    --key-id my_signing_key \
    --server-url http://localhost:8080 \
    --client-id my_client_001 \
    --user-id alice@example.com \
    --key-name "Alice's Signing Key" \
    --timeout 60 \
    --retries 5
```

**Parameters:**
- `--key-id` (required): Identifier of the locally stored key
- `--server-url` (optional): DataFold server URL (default: `http://localhost:8080`)
- `--client-id` (optional): Custom client identifier (auto-generated if not provided)
- `--user-id` (optional): User identifier for the key
- `--key-name` (optional): Human-readable name for the key
- `--storage-dir` (optional): Key storage directory (default: `~/.datafold/keys`)
- `--timeout` (optional): Connection timeout in seconds (default: 30)
- `--retries` (optional): Number of retry attempts (default: 3)

**Output:**
```
✅ Public key registered successfully!
Registration ID: reg_123456789
Client ID: cli_abcd1234
Status: active
Registered at: 2024-01-15T10:30:00Z
Client information saved for future use
```

### 2. Check Registration Status (`check-registration`)

Check the current status of a registered public key on the server.

```bash
# Check registration status
datafold_cli check-registration \
    --client-id cli_abcd1234 \
    --server-url http://localhost:8080
```

**Parameters:**
- `--client-id` (required): Client ID to check
- `--server-url` (optional): DataFold server URL (default: `http://localhost:8080`)
- `--timeout` (optional): Connection timeout in seconds (default: 30)
- `--retries` (optional): Number of retry attempts (default: 3)

**Output:**
```
✅ Registration status retrieved successfully!
Registration ID: reg_123456789
Client ID: cli_abcd1234
Public Key: a1b2c3d4e5f6...
Status: active
Registered at: 2024-01-15T10:30:00Z
Last used: 2024-01-15T15:45:30Z
Key name: Alice's Signing Key
```

### 3. Sign and Verify Messages (`sign-and-verify`)

Sign a message locally and verify the signature with the DataFold server.

```bash
# Sign and verify a text message
datafold_cli sign-and-verify \
    --key-id my_signing_key \
    --client-id cli_abcd1234 \
    --message "Hello, DataFold!" \
    --server-url http://localhost:8080

# Sign and verify from a file
datafold_cli sign-and-verify \
    --key-id my_signing_key \
    --client-id cli_abcd1234 \
    --message-file /path/to/message.txt \
    --message-encoding utf8 \
    --server-url http://localhost:8080
```

**Parameters:**
- `--key-id` (required): Identifier of the locally stored signing key
- `--client-id` (required): Registered client ID on the server
- `--message` (optional): Message to sign (text string)
- `--message-file` (optional): File containing message to sign
- `--message-encoding` (optional): Message encoding for server verification (`utf8`, `hex`, `base64`)
- `--storage-dir` (optional): Key storage directory (default: `~/.datafold/keys`)
- `--server-url` (optional): DataFold server URL (default: `http://localhost:8080`)
- `--timeout` (optional): Connection timeout in seconds (default: 30)
- `--retries` (optional): Number of retry attempts (default: 3)

**Output:**
```
Message signed successfully
Signature: a1b2c3d4e5f6789012345678901234567890abcdef...
Sending verification request to: http://localhost:8080/api/crypto/signatures/verify
✅ Signature verification completed!
Verified: ✅ SUCCESS
Client ID: cli_abcd1234
Public Key: a1b2c3d4e5f6...
Verified at: 2024-01-15T15:45:30Z
Message hash: sha256_hash_of_message
```

### 4. End-to-End Integration Test (`test-server-integration`)

Run a complete end-to-end test of the server integration workflow.

```bash
# Run complete integration test
datafold_cli test-server-integration \
    --server-url http://localhost:8080 \
    --key-id integration_test \
    --test-message "Integration test message" \
    --cleanup

# Run test with custom security settings
datafold_cli test-server-integration \
    --server-url http://localhost:8080 \
    --key-id secure_test \
    --security-level sensitive \
    --timeout 60 \
    --retries 5 \
    --cleanup
```

**Parameters:**
- `--server-url` (optional): DataFold server URL (default: `http://localhost:8080`)
- `--key-id` (optional): Test key identifier (default: `test_integration_key`)
- `--test-message` (optional): Message to test signing/verification (default: `Hello, DataFold server integration test!`)
- `--storage-dir` (optional): Key storage directory (default: `~/.datafold/keys`)
- `--security-level` (optional): Security level for key generation (`interactive`, `balanced`, `sensitive`)
- `--timeout` (optional): Connection timeout in seconds (default: 30)
- `--retries` (optional): Number of retry attempts (default: 3)
- `--cleanup` (optional): Clean up test key after completion

**Output:**
```
🧪 Starting end-to-end server integration test...
Step 1: Generating test keypair...
✅ Test key generated and stored
Step 2: Registering public key with server...
✅ Key registration successful
Step 3: Checking registration status...
✅ Registration status check successful
Step 4: Signing and verifying message...
✅ Message signing and verification successful
Step 5: Cleaning up test key...
✅ Test key cleaned up
🎉 End-to-end server integration test completed successfully!
All server integration functionality is working correctly.
```

## Workflow Examples

### Complete Setup and Usage

1. **Generate and store a key:**
```bash
# Generate a new key
datafold_cli generate-key --format hex --public-only
# Store the key securely
datafold_cli store-key --key-id my_app_key
```

2. **Register with server:**
```bash
datafold_cli register-key \
    --key-id my_app_key \
    --key-name "My Application Key" \
    --user-id myuser@company.com
```

3. **Sign and verify messages:**
```bash
datafold_cli sign-and-verify \
    --key-id my_app_key \
    --client-id cli_generated_id \
    --message "Important business transaction data"
```

### Batch Operations

For multiple keys or automated workflows:

```bash
# Test server connectivity first
datafold_cli test-server-integration --server-url http://localhost:8080

# Register multiple keys
for key_id in app_key_1 app_key_2 app_key_3; do
    datafold_cli register-key --key-id $key_id --key-name "App Key $key_id"
done

# Check all registrations
for client_id in $(cat client_ids.txt); do
    datafold_cli check-registration --client-id $client_id
done
```

## Error Handling

The CLI provides comprehensive error handling with retry logic:

### Network Errors
- **Connection timeout**: Automatically retries with exponential backoff
- **Server unavailable**: Retries up to the specified limit
- **DNS resolution failures**: Clear error messages with troubleshooting tips

### Authentication Errors
- **Invalid client ID**: Clear error message with suggested corrections
- **Key not found**: Helpful guidance on key storage and registration
- **Signature verification failures**: Detailed error information for debugging

### Server Errors
- **Rate limiting**: Automatic retry with appropriate delays
- **Server maintenance**: Clear status messages
- **API errors**: Detailed error codes and messages from server

## Integration with Automation

### CI/CD Integration

```bash
#!/bin/bash
# CI/CD script for key management

# Test server connectivity
if ! datafold_cli test-server-integration --cleanup; then
    echo "Server integration test failed"
    exit 1
fi

# Register deployment key
datafold_cli register-key \
    --key-id deploy_key_$(date +%Y%m%d) \
    --key-name "Deploy Key $(date)" \
    --user-id ci-cd@company.com

echo "Key registration completed successfully"
```

### Monitoring and Health Checks

```bash
#!/bin/bash
# Health check script

CLIENT_ID="monitoring_client_123"

# Check registration status
if datafold_cli check-registration --client-id $CLIENT_ID; then
    echo "Key registration healthy"
else
    echo "Key registration issue detected"
    # Alert or remediation logic here
fi
```

## Security Considerations

1. **Key Storage**: Private keys are encrypted at rest and never transmitted
2. **Network Security**: Use HTTPS in production environments
3. **Access Control**: Implement proper authentication on the DataFold server
4. **Key Rotation**: Regularly rotate keys using the built-in rotation commands
5. **Audit Logging**: All operations are logged for security auditing

## Troubleshooting

### Common Issues

**Connection Refused:**
```bash
# Check if server is running
curl http://localhost:8080/api/crypto/status

# Test with explicit server URL
datafold_cli check-registration --server-url http://127.0.0.1:8080 --client-id test
```

**Key Not Found:**
```bash
# List available keys
datafold_cli list-keys

# Verify key storage location
datafold_cli list-keys --storage-dir ~/.datafold/keys --verbose
```

**Signature Verification Failed:**
```bash
# Verify key pair is valid
datafold_cli verify-key --private-key-file ~/.datafold/keys/my_key.json

# Check registration status
datafold_cli check-registration --client-id your_client_id
```

## API Endpoints Used

The CLI integrates with these DataFold server endpoints:

- `POST /api/crypto/keys/register` - Public key registration
- `GET /api/crypto/keys/status/{client_id}` - Registration status lookup
- `POST /api/crypto/signatures/verify` - Signature verification
- `GET /api/crypto/status` - Server crypto status (for health checks)

For detailed API documentation, see the server documentation in `docs/api/`.