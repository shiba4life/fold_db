# DataFold CLI Tool

The DataFold CLI tool provides command-line access to DataFold's signature authentication system and APIs. It's designed for developers, DevOps teams, and automated workflows.

## 🚀 Installation

### Download Pre-built Binaries

```bash
# macOS (Intel)
curl -L https://github.com/datafold/cli/releases/latest/download/datafold-macos-intel -o datafold
chmod +x datafold
sudo mv datafold /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/datafold/cli/releases/latest/download/datafold-macos-arm64 -o datafold
chmod +x datafold
sudo mv datafold /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/datafold/cli/releases/latest/download/datafold-linux-x86_64 -o datafold
chmod +x datafold
sudo mv datafold /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/datafold/cli/releases/latest/download/datafold-windows.exe" -OutFile "datafold.exe"
# Add to PATH or move to desired location
```

### Install via Package Managers

```bash
# Homebrew (macOS/Linux)
brew install datafold/tap/datafold

# Cargo (Rust)
cargo install datafold-cli

# npm (Node.js)
npm install -g @datafold/cli

# pip (Python)
pip install datafold-cli

# Snap (Linux)
sudo snap install datafold

# Chocolatey (Windows)
choco install datafold
```

### Install Script (Recommended)

```bash
# Auto-detect platform and install
curl -sSL https://install.datafold.com | sh

# Or with options
curl -sSL https://install.datafold.com | sh -s -- --version=latest --prefix=/usr/local
```

## 🎯 Quick Start

### 1. Generate Authentication Keys

```bash
# Generate Ed25519 keypair
datafold auth keygen

# Output:
# Generated Ed25519 keypair successfully!
# Private Key: 1234567890abcdef... (keep this secret!)
# Public Key: abcdef1234567890... (register with server)
# 
# Keys saved to: ~/.datafold/keys/

# Generate with custom output
datafold auth keygen --output ~/.my-keys/ --format env
```

### 2. Register with Server

```bash
# Register public key with DataFold server
datafold auth register \
  --server-url https://api.datafold.com \
  --client-id my-app-prod \
  --key-name "Production Key" \
  --key-file ~/.datafold/keys/public.pem

# Interactive registration
datafold auth register --interactive
```

### 3. Configure Authentication

```bash
# Configure default authentication
datafold config set server-url https://api.datafold.com
datafold config set client-id my-app-prod
datafold config set key-file ~/.datafold/keys/private.pem

# Verify configuration
datafold config list
datafold auth status
```

### 4. Make API Calls

```bash
# List schemas
datafold api get /api/schemas

# Create schema
datafold schemas create \
  --name user_events \
  --version 1.0.0 \
  --field user_id:string:required \
  --field event_type:string:required \
  --field timestamp:datetime:required

# Validate data
echo '{"user_id": "123", "event_type": "login"}' | datafold schemas validate user_events
```

## 📋 Command Reference

### Global Options

```bash
datafold [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

Global Options:
  -h, --help              Show help information
  -V, --version           Show version information
  -v, --verbose           Enable verbose output
  -q, --quiet             Suppress output except errors
  --config-file PATH      Use custom config file
  --server-url URL        Override server URL
  --client-id ID          Override client ID
  --key-file PATH         Override private key file
  --timeout SECONDS       Request timeout (default: 30)
  --retries COUNT         Retry attempts (default: 3)
  --output FORMAT         Output format: json, yaml, table (default: table)
  --no-color              Disable colored output
```

### Authentication Commands

#### `datafold auth`

Manage authentication and cryptographic keys.

```bash
# Generate new keypair
datafold auth keygen [OPTIONS]

Options:
  --output PATH           Output directory (default: ~/.datafold/keys/)
  --format FORMAT         Output format: pem, hex, env (default: pem)
  --algorithm ALGO        Key algorithm: ed25519 (default: ed25519)
  --overwrite             Overwrite existing keys
  --no-backup             Don't create backup of existing keys

Examples:
  datafold auth keygen
  datafold auth keygen --output ./keys --format hex
  datafold auth keygen --overwrite --no-backup
```

```bash
# Register public key with server
datafold auth register [OPTIONS]

Options:
  --server-url URL        DataFold server URL
  --client-id ID          Client identifier
  --key-name NAME         Human-readable key name
  --key-file PATH         Public key file (default: ~/.datafold/keys/public.pem)
  --metadata KEY=VALUE    Additional metadata (can be repeated)
  --interactive           Interactive registration mode
  --dry-run               Validate without registering

Examples:
  datafold auth register --server-url https://api.datafold.com
  datafold auth register --client-id prod-api --key-name "Production Key"
  datafold auth register --interactive
  datafold auth register --metadata env=prod --metadata team=backend
```

```bash
# Check authentication status
datafold auth status [OPTIONS]

Options:
  --verbose               Show detailed status information
  --check-server          Verify server connectivity
  --check-signature       Test signature generation

Examples:
  datafold auth status
  datafold auth status --verbose --check-server
```

```bash
# Test authentication
datafold auth test [OPTIONS]

Options:
  --endpoint PATH         Test endpoint (default: /api/status)
  --method METHOD         HTTP method (default: GET)
  --body TEXT             Request body for POST/PUT
  --show-request          Show signed request details
  --show-response         Show response details

Examples:
  datafold auth test
  datafold auth test --endpoint /api/schemas --show-request
  datafold auth test --method POST --body '{"test": true}'
```

### Configuration Commands

#### `datafold config`

Manage CLI configuration settings.

```bash
# Set configuration value
datafold config set <KEY> <VALUE>

# Get configuration value
datafold config get <KEY>

# List all configuration
datafold config list [--verbose]

# Delete configuration value
datafold config delete <KEY>

# Reset configuration to defaults
datafold config reset [--confirm]

# Show configuration file location
datafold config path

Configuration Keys:
  server-url              Default DataFold server URL
  client-id               Default client identifier
  key-file                Default private key file path
  security-profile        Security profile: strict, standard, lenient
  timeout                 Default request timeout (seconds)
  retries                 Default retry attempts
  output-format           Default output format: json, yaml, table
  color                   Enable colored output: true, false
  verbose                 Enable verbose output: true, false

Examples:
  datafold config set server-url https://api.datafold.com
  datafold config set client-id my-app-prod
  datafold config set key-file ~/.datafold/keys/private.pem
  datafold config set security-profile strict
  datafold config list --verbose
```

### Schema Commands

#### `datafold schemas`

Manage DataFold schemas.

```bash
# List schemas
datafold schemas list [OPTIONS]

Options:
  --filter PATTERN        Filter schemas by name pattern
  --version VERSION       Filter by version
  --limit COUNT           Limit number of results
  --offset COUNT          Skip number of results
  --sort FIELD            Sort by field: name, version, created_at
  --order ORDER           Sort order: asc, desc

Examples:
  datafold schemas list
  datafold schemas list --filter "user_*" --sort created_at
  datafold schemas list --version "1.*" --limit 10
```

```bash
# Get schema details
datafold schemas get <SCHEMA_NAME> [OPTIONS]

Options:
  --version VERSION       Specific version (default: latest)
  --show-fields           Show field definitions
  --show-metadata         Show schema metadata
  --output FORMAT         Output format: json, yaml, table

Examples:
  datafold schemas get user_events
  datafold schemas get user_events --version 1.0.0 --show-fields
  datafold schemas get user_events --output json
```

```bash
# Create new schema
datafold schemas create [OPTIONS]

Options:
  --name NAME             Schema name (required)
  --version VERSION       Schema version (default: 1.0.0)
  --description TEXT      Schema description
  --field DEFINITION      Field definition: name:type:constraints (can be repeated)
  --file PATH             Load schema from file
  --validate              Validate schema before creation
  --dry-run               Validate without creating

Field Definition Format:
  name:type:constraints
  
  Types: string, integer, float, boolean, datetime, array, object
  Constraints: required, optional, min=N, max=N, pattern=REGEX

Examples:
  datafold schemas create --name user_events --version 1.0.0 \
    --field user_id:string:required \
    --field event_type:string:required \
    --field timestamp:datetime:required \
    --field metadata:object:optional

  datafold schemas create --file schema.json --validate
```

```bash
# Update existing schema
datafold schemas update <SCHEMA_NAME> [OPTIONS]

Options:
  --version VERSION       New version number (required)
  --description TEXT      Updated description
  --field DEFINITION      Add/update field definition
  --remove-field NAME     Remove field
  --file PATH             Load updates from file
  --validate              Validate before updating
  --dry-run               Validate without updating

Examples:
  datafold schemas update user_events --version 1.1.0 \
    --field user_agent:string:optional
  
  datafold schemas update user_events --version 1.2.0 \
    --remove-field deprecated_field \
    --field new_field:integer:required
```

```bash
# Delete schema
datafold schemas delete <SCHEMA_NAME> [OPTIONS]

Options:
  --version VERSION       Specific version (if not specified, deletes all versions)
  --force                 Skip confirmation prompt
  --dry-run               Show what would be deleted

Examples:
  datafold schemas delete old_schema --force
  datafold schemas delete user_events --version 1.0.0
```

```bash
# Validate data against schema
datafold schemas validate <SCHEMA_NAME> [OPTIONS]

Options:
  --version VERSION       Schema version (default: latest)
  --file PATH             Data file to validate
  --data TEXT             JSON data to validate
  --stdin                 Read data from stdin
  --strict                Strict validation mode
  --show-errors           Show detailed error messages
  --show-warnings         Show warning messages
  --batch                 Validate multiple records

Examples:
  datafold schemas validate user_events --file data.json
  datafold schemas validate user_events --data '{"user_id": "123"}'
  echo '{"user_id": "123"}' | datafold schemas validate user_events --stdin
  datafold schemas validate user_events --file batch.json --batch
```

### Data Commands

#### `datafold data`

Upload and manage data.

```bash
# Upload data
datafold data upload <SCHEMA_NAME> [OPTIONS]

Options:
  --file PATH             Data file to upload
  --data TEXT             JSON data to upload
  --stdin                 Read data from stdin
  --format FORMAT         Input format: json, csv, parquet
  --validate              Validate before upload
  --batch-size SIZE       Batch size for large uploads (default: 1000)
  --parallel COUNT        Number of parallel uploads (default: 1)
  --dry-run               Validate without uploading

Examples:
  datafold data upload user_events --file events.json --validate
  datafold data upload user_events --file events.csv --format csv
  cat events.json | datafold data upload user_events --stdin --batch-size 500
```

```bash
# Query data
datafold data query <SCHEMA_NAME> [OPTIONS]

Options:
  --filter EXPRESSION     Filter expression
  --fields FIELD_LIST     Comma-separated field list
  --limit COUNT           Limit number of results
  --offset COUNT          Skip number of results
  --sort FIELD            Sort by field
  --order ORDER           Sort order: asc, desc
  --output FORMAT         Output format: json, csv, table

Examples:
  datafold data query user_events --limit 100
  datafold data query user_events --filter "event_type=login" --fields user_id,timestamp
  datafold data query user_events --sort timestamp --order desc --output csv
```

### API Commands

#### `datafold api`

Make direct API calls with authentication.

```bash
# Generic API request
datafold api <METHOD> <PATH> [OPTIONS]

Methods: GET, POST, PUT, DELETE, PATCH

Options:
  --body TEXT             Request body (for POST/PUT/PATCH)
  --file PATH             Request body from file
  --header KEY:VALUE      Additional header (can be repeated)
  --query KEY=VALUE       Query parameter (can be repeated)
  --output FORMAT         Output format: json, yaml, raw
  --show-request          Show request details
  --show-headers          Show response headers

Examples:
  datafold api GET /api/schemas
  datafold api POST /api/schemas --file schema.json
  datafold api GET /api/schemas --query limit=10 --query offset=20
  datafold api POST /api/test --body '{"test": true}' --show-request
```

### Utility Commands

#### `datafold completion`

Generate shell completion scripts.

```bash
# Generate completion for bash
datafold completion bash > ~/.datafold_completion
echo 'source ~/.datafold_completion' >> ~/.bashrc

# Generate completion for zsh
datafold completion zsh > ~/.datafold_completion
echo 'source ~/.datafold_completion' >> ~/.zshrc

# Generate completion for fish
datafold completion fish > ~/.config/fish/completions/datafold.fish

# Generate completion for PowerShell
datafold completion powershell > datafold.ps1
```

#### `datafold version`

Show version information.

```bash
# Show version
datafold version

# Show detailed version information
datafold version --verbose

# Check for updates
datafold version --check-updates
```

## 🔧 Configuration

### Configuration File

The CLI stores configuration in `~/.datafold/config.toml`:

```toml
[auth]
server_url = "https://api.datafold.com"
client_id = "my-app-prod"
key_file = "~/.datafold/keys/private.pem"
security_profile = "standard"

[http]
timeout = 30
retries = 3
verify_ssl = true

[output]
format = "table"
color = true
verbose = false

[keys]
algorithm = "ed25519"
key_directory = "~/.datafold/keys/"
auto_backup = true
```

### Environment Variables

Override configuration with environment variables:

```bash
export DATAFOLD_SERVER_URL="https://api.datafold.com"
export DATAFOLD_CLIENT_ID="my-app-prod"
export DATAFOLD_PRIVATE_KEY="path/to/private.pem"
export DATAFOLD_SECURITY_PROFILE="strict"
export DATAFOLD_TIMEOUT="30"
export DATAFOLD_RETRIES="3"
export DATAFOLD_OUTPUT_FORMAT="json"
export DATAFOLD_VERBOSE="true"
```

### Key Management

```bash
# Key file locations (default)
~/.datafold/keys/
├── private.pem          # Ed25519 private key (PEM format)
├── public.pem           # Ed25519 public key (PEM format)
├── private.hex          # Private key (hex format)
├── public.hex           # Public key (hex format)
└── backup/              # Automatic backups
    ├── private.pem.bak.20250609
    └── public.pem.bak.20250609

# Security recommendations
chmod 600 ~/.datafold/keys/private.*
chmod 644 ~/.datafold/keys/public.*
```

## 🎨 Output Formats

### Table Format (Default)

```bash
datafold schemas list
```

```
┌──────────────┬─────────┬────────────────────┬────────┐
│ Name         │ Version │ Created            │ Fields │
├──────────────┼─────────┼────────────────────┼────────┤
│ user_events  │ 1.2.0   │ 2025-06-09 10:30  │ 5      │
│ page_views   │ 2.1.0   │ 2025-06-08 14:15  │ 8      │
│ purchases    │ 1.0.0   │ 2025-06-07 09:45  │ 12     │
└──────────────┴─────────┴────────────────────┴────────┘
```

### JSON Format

```bash
datafold schemas list --output json
```

```json
{
  "schemas": [
    {
      "name": "user_events",
      "version": "1.2.0",
      "created_at": "2025-06-09T10:30:00Z",
      "field_count": 5
    },
    {
      "name": "page_views", 
      "version": "2.1.0",
      "created_at": "2025-06-08T14:15:00Z",
      "field_count": 8
    }
  ]
}
```

### YAML Format

```bash
datafold schemas get user_events --output yaml
```

```yaml
name: user_events
version: 1.2.0
description: User interaction events
created_at: 2025-06-09T10:30:00Z
fields:
  - name: user_id
    type: string
    required: true
  - name: event_type
    type: string
    required: true
  - name: timestamp
    type: datetime
    required: true
```

## 🚀 Advanced Usage

### Scripting and Automation

```bash
#!/bin/bash
# deploy-schema.sh - Deploy schema changes

set -euo pipefail

SCHEMA_NAME="user_events"
SCHEMA_FILE="schemas/user_events.json"
ENVIRONMENT="${1:-production}"

echo "Deploying schema $SCHEMA_NAME to $ENVIRONMENT..."

# Configure for environment
case $ENVIRONMENT in
  production)
    datafold config set server-url https://api.datafold.com
    datafold config set client-id prod-deployment
    ;;
  staging)
    datafold config set server-url https://staging.datafold.com
    datafold config set client-id staging-deployment
    ;;
  *)
    echo "Unknown environment: $ENVIRONMENT"
    exit 1
    ;;
esac

# Validate schema file
echo "Validating schema file..."
datafold schemas validate-file "$SCHEMA_FILE"

# Check if schema exists
if datafold schemas get "$SCHEMA_NAME" &>/dev/null; then
  echo "Schema exists, updating..."
  
  # Get current version
  CURRENT_VERSION=$(datafold schemas get "$SCHEMA_NAME" --output json | jq -r '.version')
  
  # Calculate new version (simplified)
  NEW_VERSION=$(echo "$CURRENT_VERSION" | awk -F. '{$NF++; print}' OFS=.)
  
  # Update schema
  datafold schemas update "$SCHEMA_NAME" \
    --version "$NEW_VERSION" \
    --file "$SCHEMA_FILE" \
    --validate
else
  echo "Schema doesn't exist, creating..."
  
  # Create new schema
  datafold schemas create \
    --file "$SCHEMA_FILE" \
    --validate
fi

echo "Schema deployment completed successfully!"
```

### CI/CD Integration

```yaml
# .github/workflows/datafold-schema-validation.yml
name: DataFold Schema Validation

on:
  push:
    paths: ['schemas/**']
  pull_request:
    paths: ['schemas/**']

jobs:
  validate-schemas:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install DataFold CLI
      run: curl -sSL https://install.datafold.com | sh
    
    - name: Configure Authentication
      run: |
        echo "${{ secrets.DATAFOLD_PRIVATE_KEY }}" > /tmp/datafold.pem
        chmod 600 /tmp/datafold.pem
        
        datafold config set server-url "${{ secrets.DATAFOLD_SERVER_URL }}"
        datafold config set client-id "${{ secrets.DATAFOLD_CLIENT_ID }}"
        datafold config set key-file /tmp/datafold.pem
    
    - name: Validate Schemas
      run: |
        for schema in schemas/*.json; do
          echo "Validating $schema..."
          datafold schemas validate-file "$schema"
        done
    
    - name: Test Schema Deployment (Dry Run)
      if: github.event_name == 'pull_request'
      run: |
        for schema in schemas/*.json; do
          schema_name=$(basename "$schema" .json)
          echo "Testing deployment of $schema_name..."
          datafold schemas create --file "$schema" --dry-run
        done
    
    - name: Deploy Schemas
      if: github.ref == 'refs/heads/main'
      run: |
        for schema in schemas/*.json; do
          schema_name=$(basename "$schema" .json)
          echo "Deploying $schema_name..."
          
          if datafold schemas get "$schema_name" &>/dev/null; then
            # Update existing schema
            current_version=$(datafold schemas get "$schema_name" --output json | jq -r '.version')
            new_version=$(echo "$current_version" | awk -F. '{$NF++; print}' OFS=.)
            datafold schemas update "$schema_name" --version "$new_version" --file "$schema"
          else
            # Create new schema
            datafold schemas create --file "$schema"
          fi
        done
```

### Batch Processing

```bash
#!/bin/bash
# process-data-files.sh - Process multiple data files

DATA_DIR="./data"
SCHEMA_NAME="user_events"
BATCH_SIZE=1000

echo "Processing data files in $DATA_DIR..."

# Find all JSON files
find "$DATA_DIR" -name "*.json" -type f | while read -r file; do
  echo "Processing $file..."
  
  # Validate first
  if datafold schemas validate "$SCHEMA_NAME" --file "$file" --batch; then
    echo "✓ Validation passed for $file"
    
    # Upload data
    if datafold data upload "$SCHEMA_NAME" --file "$file" --batch-size "$BATCH_SIZE"; then
      echo "✓ Upload completed for $file"
      
      # Move to processed directory
      mkdir -p "$DATA_DIR/processed"
      mv "$file" "$DATA_DIR/processed/"
    else
      echo "✗ Upload failed for $file"
      
      # Move to failed directory
      mkdir -p "$DATA_DIR/failed"
      mv "$file" "$DATA_DIR/failed/"
    fi
  else
    echo "✗ Validation failed for $file"
    
    # Move to failed directory
    mkdir -p "$DATA_DIR/failed"
    mv "$file" "$DATA_DIR/failed/"
  fi
done

echo "Batch processing completed!"
```

### Data Pipeline Integration

```bash
#!/bin/bash
# data-pipeline.sh - Complete data processing pipeline

set -euo pipefail

PIPELINE_NAME="${1:-default}"
CONFIG_DIR="./pipelines/$PIPELINE_NAME"

echo "Starting data pipeline: $PIPELINE_NAME"

# Load pipeline configuration
source "$CONFIG_DIR/config.sh"

# 1. Extract data
echo "Step 1: Extracting data..."
"$CONFIG_DIR/extract.sh"

# 2. Transform data
echo "Step 2: Transforming data..."
"$CONFIG_DIR/transform.sh"

# 3. Validate against schema
echo "Step 3: Validating data..."
datafold schemas validate "$SCHEMA_NAME" \
  --file "$TRANSFORMED_DATA_FILE" \
  --batch \
  --strict

# 4. Upload to DataFold
echo "Step 4: Uploading data..."
datafold data upload "$SCHEMA_NAME" \
  --file "$TRANSFORMED_DATA_FILE" \
  --batch-size "$BATCH_SIZE" \
  --parallel "$PARALLEL_UPLOADS"

# 5. Verify upload
echo "Step 5: Verifying upload..."
UPLOAD_COUNT=$(datafold data query "$SCHEMA_NAME" \
  --filter "uploaded_at >= $(date -d '1 hour ago' -Iseconds)" \
  --output json | jq '.count')

echo "Pipeline completed successfully! Uploaded $UPLOAD_COUNT records."
```

## 🐛 Troubleshooting

### Common Issues

#### Authentication Failures

```bash
# Check authentication status
datafold auth status --verbose

# Test authentication
datafold auth test --show-request

# Verify key file permissions
ls -la ~/.datafold/keys/

# Regenerate keys if needed
datafold auth keygen --overwrite
```

#### Network Connectivity

```bash
# Test basic connectivity
curl -v "$(datafold config get server-url)/api/status"

# Test with CLI
datafold api GET /api/status --show-request --show-headers

# Check proxy settings
env | grep -i proxy
```

#### Configuration Issues

```bash
# Show current configuration
datafold config list --verbose

# Reset to defaults
datafold config reset --confirm

# Show configuration file location
datafold config path
```

### Debug Mode

```bash
# Enable verbose output
datafold --verbose schemas list

# Show request/response details
datafold api GET /api/schemas --show-request --show-headers

# Use debug environment variable
export DATAFOLD_DEBUG=true
datafold schemas list
```

### Log Files

```bash
# View CLI logs
tail -f ~/.datafold/logs/cli.log

# View authentication logs
tail -f ~/.datafold/logs/auth.log

# Clear logs
rm ~/.datafold/logs/*.log
```

## 🔗 Related Documentation

- **[Commands Reference](commands.md)** - Complete command documentation
- **[Authentication Guide](authentication.md)** - CLI authentication setup
- **[Configuration Reference](configuration.md)** - Configuration options
- **[JavaScript SDK](../javascript/README.md)** - JavaScript implementation
- **[Python SDK](../python/README.md)** - Python implementation

## 📞 Support

- **Documentation**: [CLI Commands Reference](commands.md)
- **Issues**: [GitHub Issues](https://github.com/datafold/cli/issues)
- **Releases**: [GitHub Releases](https://github.com/datafold/cli/releases)
- **Community**: [Discord](https://discord.gg/datafold)

---

**Next**: Explore the [complete commands reference](commands.md) or learn about [authentication setup](authentication.md).