# CLI Signature Verification Guide

This guide covers the signature verification utilities available in the DataFold CLI, which enable validation of signed responses and command-line signature verification tools.

## Overview

The DataFold CLI includes comprehensive signature verification capabilities that complement the existing signing functionality. These tools allow you to:

- Verify message signatures using Ed25519 cryptography
- Inspect signature format and analyze components
- Verify server response signatures
- Manage verification policies and public keys
- Debug signature-related issues

## Commands

### `verify-signature` - Verify a signature against a message

Verify a signature against a message using a specified public key.

```bash
# Verify a signature with inline message and public key
datafold verify-signature \
  --message "Hello, World!" \
  --signature "base64-encoded-signature" \
  --key-id "client-key-1" \
  --public-key "hex-encoded-public-key"

# Verify a signature with message and key from files
datafold verify-signature \
  --message-file message.txt \
  --signature "base64-encoded-signature" \
  --key-id "client-key-1" \
  --public-key-file public_key.hex

# Use specific verification policy
datafold verify-signature \
  --message "Hello, World!" \
  --signature "base64-encoded-signature" \
  --key-id "client-key-1" \
  --public-key "hex-encoded-public-key" \
  --policy strict

# Enable debug output
datafold verify-signature \
  --message "Hello, World!" \
  --signature "base64-encoded-signature" \
  --key-id "client-key-1" \
  --public-key "hex-encoded-public-key" \
  --debug \
  --output-format json
```

**Parameters:**
- `--message` - Message to verify (as string)
- `--message-file` - Message file path
- `--signature` - Signature to verify (base64 encoded)
- `--key-id` - Key ID for verification
- `--public-key` - Public key (hex, base64, or PEM format)
- `--public-key-file` - Public key file path
- `--policy` - Verification policy to use (default, strict, permissive)
- `--output-format` - Output format (json, table, compact)
- `--debug` - Enable debug output

### `inspect-signature` - Analyze signature format and components

Inspect and analyze signature headers for RFC 9421 compliance and format issues.

```bash
# Inspect signature format from headers
datafold inspect-signature \
  --signature-input 'sig1=("@method" "@target-uri");alg="ed25519"' \
  --signature "base64-encoded-signature"

# Inspect from headers file
datafold inspect-signature \
  --headers-file headers.json \
  --detailed

# Enable debug analysis
datafold inspect-signature \
  --signature-input 'sig1=("@method" "@target-uri");alg="ed25519"' \
  --signature "base64-encoded-signature" \
  --debug \
  --output-format json
```

**Parameters:**
- `--signature-input` - Signature input header value
- `--signature` - Signature header value (base64 encoded)
- `--headers-file` - Headers file (JSON format)
- `--output-format` - Output format (json, table, compact)
- `--detailed` - Show detailed analysis
- `--debug` - Enable debug output

### `verify-response` - Verify server response signatures

Verify signatures on HTTP responses from servers.

```bash
# Verify a GET response
datafold verify-response \
  --url "https://api.example.com/data" \
  --method get \
  --key-id "server-key-1" \
  --public-key-file server_public_key.hex

# Verify a POST response with request body
datafold verify-response \
  --url "https://api.example.com/data" \
  --method post \
  --headers '{"Content-Type": "application/json"}' \
  --body '{"key": "value"}' \
  --key-id "server-key-1" \
  --public-key-file server_public_key.hex \
  --policy strict

# Verify with custom timeout and debug output
datafold verify-response \
  --url "https://api.example.com/data" \
  --method get \
  --key-id "server-key-1" \
  --public-key-file server_public_key.hex \
  --timeout 60 \
  --debug \
  --output-format json
```

**Parameters:**
- `--url` - Server URL to test
- `--method` - HTTP method (get, post, put, patch, delete)
- `--headers` - Request headers (JSON format)
- `--body` - Request body for POST/PUT requests
- `--body-file` - Request body file
- `--key-id` - Key ID for verification
- `--public-key` - Public key (hex, base64, or PEM format)
- `--public-key-file` - Public key file path
- `--policy` - Verification policy to use
- `--output-format` - Output format (json, table, compact)
- `--debug` - Enable debug output
- `--timeout` - Timeout in seconds (default: 30)

### `verification-config` - Manage verification settings

Manage verification policies and public keys.

```bash
# Show current configuration
datafold verification-config show

# Show verification policies
datafold verification-config show --policies

# Show configured public keys
datafold verification-config show --keys

# Add a public key
datafold verification-config add-public-key \
  --key-id "server-key-1" \
  --public-key-file server_public_key.hex

# List all public keys
datafold verification-config list-public-keys --verbose

# Add a custom verification policy
datafold verification-config add-policy \
  --name "custom-strict" \
  --config-file custom_policy.json
```

**Subcommands:**
- `show` - Show current verification configuration
- `add-policy` - Add a verification policy from JSON file
- `remove-policy` - Remove a verification policy
- `set-default-policy` - Set default verification policy
- `add-public-key` - Add a public key for verification
- `remove-public-key` - Remove a public key
- `list-public-keys` - List all configured public keys

## Verification Policies

The CLI supports multiple verification policies with different security levels:

### Default Policy
- Verifies timestamp validity (5 minutes max age)
- Verifies nonce format
- Verifies content digest
- Requires `@method` and `@target-uri` components
- Allows only Ed25519 algorithm

### Strict Policy
- Verifies timestamp validity (1 minute max age)
- Verifies nonce format
- Verifies content digest
- Requires `@method`, `@target-uri`, `@authority`, `content-type`, and `content-digest` components
- Allows only Ed25519 algorithm
- Requires all specified headers

### Permissive Policy
- No timestamp verification
- No nonce verification
- Extended timestamp validity (1 hour max age)
- Requires only `@method` component
- Allows Ed25519 algorithm
- Does not require all headers

## Output Formats

### Table Format (Default)
Human-readable table format with verification results and diagnostics.

```
=== Signature Verification Result ===
Status: VALID
Signature Valid: true
Total Time: 45ms

=== Individual Checks ===
Format Valid: true
Cryptographic Valid: true
Timestamp Valid: true
Nonce Valid: true
Content Digest Valid: true
Component Coverage Valid: true
Policy Compliance Valid: true
```

### JSON Format
Structured JSON output suitable for programmatic processing.

```json
{
  "status": "Valid",
  "signature_valid": true,
  "checks": {
    "format_valid": true,
    "cryptographic_valid": true,
    "timestamp_valid": true,
    "nonce_valid": true,
    "content_digest_valid": true,
    "component_coverage_valid": true,
    "policy_compliance_valid": true
  },
  "diagnostics": {
    "signature_analysis": {
      "algorithm": "ed25519",
      "key_id": "client-key-1",
      "created": 1640995200,
      "age_seconds": 30,
      "covered_components": ["@method", "@target-uri"]
    },
    "performance": {
      "total_time_ms": 45,
      "step_timings": {
        "signature_extraction": 5,
        "cryptographic_verification": 25,
        "timestamp_verification": 2
      }
    }
  }
}
```

### Compact Format
Single line output suitable for scripts and monitoring.

```
VALID: ✓
```

## Integration with Existing CLI Features

### Automatic Verification with `--verify` Flag

Add automatic verification to existing CLI commands:

```bash
# Query with response verification
datafold query --schema users --fields name,email --verify

# Enable debug verification output
datafold query --schema users --fields name,email --verify-debug
```

### Signing and Verification Workflow

1. **Setup Authentication Profile** (if not already done):
   ```bash
   datafold auth-profile create \
     --name "production" \
     --server-url "https://api.example.com" \
     --key-id "client-key-1"
   ```

2. **Configure Verification Keys**:
   ```bash
   datafold verification-config add-public-key \
     --key-id "server-key-1" \
     --public-key-file server_public_key.hex
   ```

3. **Test End-to-End Verification**:
   ```bash
   datafold verify-response \
     --url "https://api.example.com/health" \
     --method get \
     --key-id "server-key-1" \
     --public-key-file server_public_key.hex
   ```

## Troubleshooting

### Common Issues

**Signature verification fails with "Invalid signature format":**
- Check that the signature is properly base64 encoded
- Verify the signature-input header format follows RFC 9421
- Use `inspect-signature` to analyze format issues

**Timestamp verification fails:**
- Check system clock synchronization
- Adjust verification policy timestamp tolerance
- Use `--debug` to see detailed timestamp analysis

**Content digest mismatch:**
- Verify request/response body hasn't been modified
- Check content-type header matches actual content
- Enable debug output to see digest calculation details

**Key not found errors:**
- Verify the key ID matches the configured keys
- Check public key format (hex, base64, or PEM)
- Use `verification-config list-public-keys` to see configured keys

### Debug Output

Enable debug output with `--debug` flag to get detailed verification information:

```bash
datafold verify-signature \
  --message "test" \
  --signature "..." \
  --key-id "test-key" \
  --public-key "..." \
  --debug
```

This will show:
- Signature component analysis
- Cryptographic verification details
- Timestamp and nonce validation
- Performance metrics
- Policy compliance checks

## Security Considerations

1. **Key Management**: Store public keys securely and verify their authenticity
2. **Policy Selection**: Use appropriate verification policies for your security requirements
3. **Timestamp Tolerance**: Balance security with clock synchronization realities
4. **Error Handling**: Don't expose sensitive verification details in production logs
5. **Performance**: Verification adds latency - monitor performance in production

## Examples

See the `examples/` directory for complete examples of using the CLI verification utilities in various scenarios.