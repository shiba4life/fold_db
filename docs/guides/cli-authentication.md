# DataFold CLI Authentication Guide

This guide explains how to set up and use **mandatory signature authentication** with the DataFold CLI. All DataFold requests require RFC 9421 HTTP Message Signatures for authentication.

## Overview

The DataFold CLI requires RFC 9421 HTTP Message Signatures for all authenticated requests. Signature authentication is **mandatory** and cannot be disabled. Once configured, requests are automatically signed using Ed25519 cryptography with configurable security profiles.

## ⚠️ Important: Mandatory Authentication

**Signature authentication is required for all DataFold operations.** You cannot disable authentication or make unsigned requests. All CLI commands that interact with a DataFold server must be properly configured with:

- A valid Ed25519 key pair
- An authentication profile
- Appropriate security settings

Attempting to make requests without proper authentication will result in immediate failure.

## Quick Start

### 1. Interactive Setup

The fastest way to get started is with interactive setup:

```bash
# Start interactive authentication setup
datafold auth-setup --interactive

# Or with a specific server URL
datafold auth-setup --interactive --server-url https://api.yourcompany.com
```

This will guide you through:
- Creating a configuration file
- Generating or selecting a key pair
- Creating an authentication profile
- Configuring automatic signing preferences

### 2. Manual Setup

For more control, set up authentication manually:

```bash
# Create configuration file
datafold auth-setup --create-config --server-url https://api.yourcompany.com

# Generate a key pair
datafold auth-keygen --key-id my-key

# Create an authentication profile
datafold auth-profile create default \
  --server-url https://api.yourcompany.com \
  --key-id my-key

# Configure automatic signing
datafold auth-configure --enable-auto-sign true --default-mode auto
```

## Configuration

### Configuration File

The CLI uses a TOML configuration file located at `~/.datafold/config.toml`. You can start with the template:

```bash
cp config/cli-config-template.toml ~/.datafold/config.toml
```

### Security Profiles

The CLI supports three security profiles that determine authentication strictness:

- **`strict`**: Maximum security with tight time windows (production recommended)
- **`standard`**: Balanced security settings (default)
- **`lenient`**: Relaxed validation for development environments

**Note**: All requests are automatically signed. You cannot disable signature authentication.

### Global Authentication Configuration

```bash
# Configure security profile
datafold auth-configure --security-profile strict

# Set environment-specific settings
datafold auth-configure --environment production

# Enable debug logging for troubleshooting
datafold auth-configure --debug true

# View current configuration
datafold auth-configure --show
```

### Environment-Specific Configuration

Configure different security settings for different environments:

```bash
# Production environment with strict security
datafold auth-configure --environment production --security-profile strict

# Development environment with lenient settings
datafold auth-configure --environment development --security-profile lenient

# Staging environment with standard settings
datafold auth-configure --environment staging --security-profile standard
```

## Using the CLI with Authentication

### Global Flags

The CLI provides global flags for authentication configuration:

```bash
# Use specific profile
datafold query --profile production --schema my-schema --fields id,name

# Use specific environment
datafold query --environment production --schema my-schema --fields id,name

# Enable debug logging
datafold query --auth-debug --schema my-schema --fields id,name

# Verbose output (includes authentication details)
datafold query --verbose --schema my-schema --fields id,name
```

**Note**: All requests are automatically signed. The `--sign` and `--no-sign` flags are no longer supported since authentication is mandatory.

### Environment Variables

You can use environment variables to control authentication settings:

```bash
# Set security profile for all commands
export DATAFOLD_SECURITY_PROFILE=strict

# Set environment for all commands
export DATAFOLD_ENVIRONMENT=production

# Enable debug authentication logging
export DATAFOLD_AUTH_DEBUG=true

# Set default profile
export DATAFOLD_DEFAULT_PROFILE=production
```

## Key Management

### Generating Keys

```bash
# Generate a new key pair
datafold auth-keygen --key-id my-key

# Generate with higher security
datafold auth-keygen --key-id secure-key --security-level sensitive

# Auto-register with server
datafold auth-keygen --key-id my-key --auto-register --server-url https://api.yourcompany.com
```

### Managing Stored Keys

```bash
# List all stored keys
datafold list-keys --verbose

# Retrieve a key (shows public key by default)
datafold retrieve-key --key-id my-key --public-only

# Delete a key
datafold delete-key --key-id old-key --force

# Backup a key
datafold backup-key --key-id my-key --backup-file my-key-backup.json

# Restore from backup
datafold restore-key --backup-file my-key-backup.json --key-id restored-key
```

## Profile Management

### Creating Profiles

```bash
# Create a new profile
datafold auth-profile create production \
  --server-url https://api.yourcompany.com \
  --key-id prod-key \
  --user-id my-user-id \
  --set-default

# Create for local development
datafold auth-profile create local \
  --server-url http://localhost:8080 \
  --key-id dev-key
```

### Managing Profiles

```bash
# List all profiles
datafold auth-profile list --verbose

# Show profile details
datafold auth-profile show production

# Update a profile
datafold auth-profile update production --server-url https://new-api.yourcompany.com

# Set default profile
datafold auth-profile set-default production

# Delete a profile
datafold auth-profile delete old-profile --force
```

## Testing Authentication

### Check Status

```bash
# Check authentication status
datafold auth-status

# Detailed status for specific profile
datafold auth-status --verbose --profile production
```

### Test Requests

```bash
# Test authentication with default profile
datafold auth-test

# Test specific endpoint
datafold auth-test --endpoint /api/v1/health

# Test with different HTTP method
datafold auth-test --method post --payload '{"test": true}'

# Test with specific profile
datafold auth-test --profile production --timeout 60
```

## Troubleshooting

### Debug Mode

Enable debug mode to see detailed signing information:

```bash
# Enable debug globally
datafold auth-configure --debug true

# Enable debug for single request
datafold query --sign-debug --verbose --schema my-schema --fields id
```

### Common Issues

1. **"No authentication profile specified"**
   ```bash
   # Set a default profile
   datafold auth-profile set-default my-profile
   
   # Or specify profile explicitly
   datafold --profile my-profile query --schema my-schema --fields id
   ```

2. **"Key not found in storage"**
   ```bash
   # List available keys
   datafold list-keys
   
   # Generate a new key
   datafold auth-keygen --key-id my-key
   ```

3. **"Failed to load keypair"**
   ```bash
   # Check if passphrase is correct
   datafold retrieve-key --key-id my-key --public-only
   
   # Verify key integrity
   datafold verify-key --private-key-file ~/.datafold/keys/my-key.json
   ```

4. **Signature verification failures**
   ```bash
   # Enable debug mode to see signature details
   datafold auth-test --sign-debug --verbose
   
   # Check server time synchronization
   date
   ```

### Performance Tuning

For better performance with frequent requests:

```toml
# In ~/.datafold/config.toml
[signing.performance]
# Reduce signing timeout for faster failures
max_signing_time_ms = 2000
# Enable key caching
cache_keys = true
# Increase concurrent signing limit
max_concurrent_signs = 20
```

## Security Best Practices

1. **Protect your private keys**
   - Use strong passphrases for key encryption
   - Store keys in secure locations with proper file permissions
   - Regularly rotate keys using `datafold rotate-key`
   - Never share private keys between environments

2. **Use appropriate security profiles**
   - Use `strict` profile for production environments
   - Use `standard` profile for staging environments
   - Use `lenient` profile only for development
   - Never use lenient settings in production

3. **Monitor authentication**
   - Regularly check authentication status with `datafold auth-status`
   - Monitor for failed authentication attempts
   - Review authentication logs regularly
   - Set up alerts for authentication failures

4. **Environment and profile management**
   - Use separate profiles for different environments
   - Regularly review and clean up unused profiles
   - Set appropriate user IDs for audit trails
   - Implement key rotation policies for production

5. **Network security**
   - Always use HTTPS for server communications
   - Verify server certificates
   - Monitor for man-in-the-middle attacks
   - Use secure networks for key operations

## Examples

### Development Workflow

```bash
# Setup for local development
datafold auth-setup --interactive --server-url http://localhost:8080
datafold auth-configure --default-mode auto

# Daily usage
datafold query --schema users --fields id,name,email
datafold mutate --schema users --mutation-type create --data '{"name": "Alice"}'
```

### Production Workflow

```bash
# Setup for production
datafold auth-keygen --key-id prod-key --security-level sensitive
datafold auth-profile create production \
  --server-url https://api.yourcompany.com \
  --key-id prod-key \
  --set-default

# Configure for manual signing (more secure)
datafold auth-configure --default-mode manual

# Production usage with explicit signing
datafold --sign query --schema orders --fields id,status
datafold --sign --profile production mutate --schema products --mutation-type update --data '{"id": 123, "price": 29.99}'
```

### Multi-Environment Setup

```bash
# Setup multiple profiles
datafold auth-profile create dev --server-url http://localhost:8080 --key-id dev-key
datafold auth-profile create staging --server-url https://staging-api.yourcompany.com --key-id staging-key
datafold auth-profile create prod --server-url https://api.yourcompany.com --key-id prod-key

# Configure per-environment signing
datafold auth-configure --command query --command-mode auto  # Auto-sign queries
datafold auth-configure --command mutate --command-mode manual  # Manual signing for mutations

# Use with different environments
datafold --profile dev query --schema users --fields id
datafold --profile staging --sign mutate --schema users --mutation-type create --data '{...}'
datafold --profile prod --sign query --schema orders --fields id,total
```

## Migration from Optional Authentication

If you're upgrading from a previous version where authentication was optional:

1. **Verify existing configuration**
   ```bash
   datafold auth-status --verbose
   ```

2. **Update configuration for mandatory authentication**
   ```bash
   datafold auth-setup --upgrade-config
   ```

3. **Import existing keys if needed**
   ```bash
   datafold import-key --export-file my-old-key.json --key-id imported-key
   ```

4. **Create environment-specific profiles**
   ```bash
   # Production profile with strict security
   datafold auth-profile create production \
     --server-url https://api.yourcompany.com \
     --key-id prod-key \
     --security-profile strict
   
   # Development profile with lenient security
   datafold auth-profile create development \
     --server-url http://localhost:8080 \
     --key-id dev-key \
     --security-profile lenient
   ```

5. **Test mandatory authentication**
   ```bash
   datafold auth-test --profile production
   datafold query --schema test --fields id  # Authentication is automatic
   ```

6. **Update scripts and automation**
   - Remove any `--no-sign` flags from scripts
   - Update environment variables to use new authentication settings
   - Ensure all automation has proper authentication profiles configured

For more information, see the [Migration Guide](../migration/migration-guide.md) and [Production Deployment Guide](../deployment-guide.md).