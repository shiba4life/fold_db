# CLI Signature Verification Guide

This guide covers the **mandatory signature verification** utilities in the DataFold CLI. All DataFold communication requires RFC 9421 HTTP Message Signatures using Ed25519 cryptography.

## Overview

The DataFold CLI includes comprehensive signature verification capabilities that are **required** for all server communication. These tools allow you to:

- Verify message signatures using Ed25519 cryptography (mandatory for all requests)
- Inspect signature format and analyze components
- Verify server response signatures (automatic in all CLI operations)
- Manage verification policies and public keys
- Debug signature-related issues
- Validate authentication configuration

## ⚠️ Important: Mandatory Verification

**Signature verification is mandatory** for all DataFold CLI operations. The CLI automatically:

- Verifies all server responses
- Validates signature format and cryptographic authenticity
- Enforces security policies based on your environment configuration
- Rejects communications with invalid signatures

You cannot disable signature verification as it is a core security requirement.

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

### Automatic Verification (Always Enabled)

All CLI commands automatically perform signature verification:

```bash
# All queries automatically verify server responses
datafold query --schema users --fields name,email

# Enable debug verification output to see verification details
datafold query --schema users --fields name,email --auth-debug

# Use verbose mode to see full authentication flow
datafold query --schema users --fields name,email --verbose
```

**Note**: The `--verify` flag is no longer needed since verification is always performed.

### Complete Authentication and Verification Workflow

1. **Setup Authentication Profile** (mandatory):
   ```bash
   datafold auth-profile create production \
     --server-url "https://api.example.com" \
     --key-id "client-key-1" \
     --security-profile "strict"
   ```

2. **Configure Server Verification Keys** (required for response verification):
   ```bash
   datafold verification-config add-public-key \
     --key-id "server-key-1" \
     --public-key-file server_public_key.hex
   ```

3. **Test Complete Authentication Flow**:
   ```bash
   # Test both client authentication and server verification
   datafold auth-test --profile production --full-verification
   
   # Test specific endpoint with full verification
   datafold verify-response \
     --url "https://api.example.com/health" \
     --method get \
     --profile production
   ```

4. **Verify Environment Configuration**:
   ```bash
   # Check authentication and verification status
   datafold auth-status --verification-details
   
   # Validate all security settings
   datafold auth-validate --environment production
   ```

## Troubleshooting

### Common Issues

**Authentication fails with "Signature verification failed":**
- Verify your authentication profile is correctly configured: `datafold auth-status`
- Check key registration with server: `datafold auth-test`
- Ensure security profile matches server requirements
- Use `--auth-debug` to see detailed signature analysis

**Server response verification fails:**
- Verify server public keys are configured: `datafold verification-config show --keys`
- Check server signature format with: `datafold inspect-signature`
- Ensure server is using compatible signature algorithms
- Use `--verbose` to see detailed verification process

**Timestamp verification fails:**
- Check system clock synchronization with NTP
- Verify security profile timestamp tolerances
- Use `datafold auth-test --timestamp-check` to diagnose clock issues
- Consider adjusting security profile for development environments

**Environment configuration issues:**
- Validate configuration: `datafold auth-validate --environment [env]`
- Check environment switching: `datafold auth-configure --show`
- Verify security profile settings match requirements
- Use `datafold auth-setup --interactive` to reconfigure

**Key management errors:**
- List available keys: `datafold list-keys --verbose`
- Verify key permissions and accessibility
- Check key format and encryption status
- Use `datafold auth-keygen` to regenerate if necessary

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

1. **Mandatory Security**: Signature verification cannot be disabled - this is by design for security
2. **Key Management**: Store all keys securely and verify their authenticity before configuration
3. **Security Profile Selection**: Use appropriate profiles for each environment:
   - **Production**: Use `strict` profile for maximum security
   - **Staging**: Use `standard` profile for testing
   - **Development**: Use `lenient` profile only for local development
4. **Timestamp Management**: Ensure system clocks are synchronized across all environments
5. **Error Monitoring**: Monitor authentication failures and implement alerting
6. **Performance Impact**: Signature verification adds security but monitor latency in high-throughput scenarios

## Compliance and Auditing

The mandatory signature verification supports compliance requirements:

- **Audit Trails**: All authentication attempts are logged with correlation IDs
- **Non-Repudiation**: Ed25519 signatures provide cryptographic proof of authenticity
- **Compliance Standards**: Supports SOC 2, HIPAA, and other security frameworks
- **Monitoring Integration**: Compatible with security information and event management (SIEM) systems

## Examples

See the `examples/` directory for complete examples of:
- Setting up mandatory authentication in different environments
- Configuring security profiles for compliance requirements
- Implementing monitoring and alerting for authentication events
- Troubleshooting common authentication and verification issues