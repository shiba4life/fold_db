# DataFold CLI Authentication Guide

This guide explains how to set up and use automatic signature injection with the DataFold CLI for seamless authentication.

## Overview

The DataFold CLI now supports automatic RFC 9421 HTTP Message Signatures for transparent authentication. Once configured, requests are automatically signed without manual intervention while providing fine-grained control over signing behavior.

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

### Signing Modes

The CLI supports three signing modes:

- **`auto`**: Automatically sign all requests
- **`manual`**: Only sign when explicitly requested with `--sign`
- **`disabled`**: Never sign requests

### Global Signing Configuration

```bash
# Enable automatic signing globally
datafold auth-configure --enable-auto-sign true

# Set default signing mode
datafold auth-configure --default-mode auto

# Enable debug logging for troubleshooting
datafold auth-configure --debug true

# View current configuration
datafold auth-configure --show
```

### Per-Command Signing

You can configure signing behavior for specific commands:

```bash
# Always sign query commands
datafold auth-configure --command query --command-mode auto

# Never sign status commands
datafold auth-configure --command auth-status --command-mode disabled

# Remove command-specific override
datafold auth-configure --remove-command-override query
```

## Using the CLI with Authentication

### Global Flags

The CLI provides global flags to control signing behavior:

```bash
# Force signing for this request
datafold query --sign --schema my-schema --fields id,name

# Force no signing for this request
datafold query --no-sign --schema my-schema --fields id,name

# Use specific profile
datafold query --profile production --schema my-schema --fields id,name

# Enable debug logging
datafold query --sign-debug --schema my-schema --fields id,name

# Verbose output
datafold query --verbose --schema my-schema --fields id,name
```

### Environment Variables

You can use environment variables to control signing:

```bash
# Enable automatic signing for all commands
export DATAFOLD_AUTO_SIGN=auto

# Disable signing for all commands
export DATAFOLD_AUTO_SIGN=disabled

# Use manual signing mode
export DATAFOLD_AUTO_SIGN=manual
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

2. **Use appropriate signing modes**
   - Use `auto` mode for trusted environments
   - Use `manual` mode for shared systems
   - Use `disabled` mode for debugging only

3. **Monitor authentication**
   - Regularly check authentication status
   - Monitor for failed authentication attempts
   - Use debug mode only when necessary

4. **Profile management**
   - Use different profiles for different environments
   - Regularly review and clean up unused profiles
   - Set appropriate user IDs for audit trails

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

## Migration from Manual Authentication

If you're upgrading from manual authentication:

1. **Create configuration file**
   ```bash
   datafold auth-setup --create-config
   ```

2. **Import existing keys**
   ```bash
   datafold import-key --export-file my-old-key.json --key-id imported-key
   ```

3. **Create profiles for existing setups**
   ```bash
   datafold auth-profile create existing \
     --server-url https://your-existing-server.com \
     --key-id imported-key
   ```

4. **Configure automatic signing**
   ```bash
   datafold auth-configure --enable-auto-sign true --default-mode manual
   ```

5. **Test the setup**
   ```bash
   datafold auth-test
   datafold --sign query --schema test --fields id
   ```

For more information, see the [API Authentication Guide](./api-authentication.md) and [Security Best Practices](./security-best-practices.md).