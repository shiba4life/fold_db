# DataFold Ingestion Module

## Overview

The DataFold Ingestion Module provides AI-powered automatic data ingestion capabilities for the DataFold distributed database system. It analyzes JSON data, automatically creates or selects appropriate schemas, and generates mutations to store the data seamlessly.

## ✨ Features

- **🤖 AI-Powered Schema Analysis** - Uses OpenRouter API with Claude 3.5 Sonnet to intelligently analyze data
- **🔄 Automatic Schema Creation** - Creates new schemas when existing ones don't match
- **🎯 Smart Field Mapping** - Maps JSON field paths to schema fields automatically
- **⚡ Real-time Processing** - Processes and stores data in real-time
- **🛡️ Robust Error Handling** - Comprehensive error handling with retry logic
- **🔧 Flexible Configuration** - Environment-based configuration with sensible defaults

## 🚀 Quick Start

### 1. Environment Setup

Set the required environment variables:

```bash
export FOLD_OPENROUTER_API_KEY="your-openrouter-api-key"
export INGESTION_ENABLED="true"
export OPENROUTER_MODEL="anthropic/claude-3.5-sonnet"
```

### 2. Start the Server

Use the provided script to start the server:

```bash
./run_http_server.sh
```

This script will:
1. Build the React frontend
2. Build the Rust backend
3. Start the HTTP server on port 9001

The server will start on `http://localhost:9001` with ingestion endpoints available and the UI accessible via the "Ingestion" tab.

**Alternative Manual Start:**
```bash
cd fold_node
cargo run --bin datafold_http_server -- --port 9001
```

### 3. Test the Ingestion

```bash
curl -X POST http://localhost:9001/api/ingestion/process \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "name": "John Doe",
      "email": "john@example.com",
      "age": 30,
      "preferences": {
        "theme": "dark",
        "notifications": true
      }
    },
    "auto_execute": true
  }'
```

## 📡 API Endpoints

### Process JSON Data
**POST** `/api/ingestion/process`

Processes JSON data and automatically creates schemas or maps to existing ones.

**Request:**
```json
{
  "data": { /* Your JSON data */ },
  "auto_execute": true,
  "trust_distance": 0,
  "pub_key": "default"
}
```

**Response:**
```json
{
  "success": true,
  "schema_used": "UserProfile",
  "new_schema_created": true,
  "mutations_generated": 1,
  "mutations_executed": 1,
  "errors": []
}
```

### Get Status
**GET** `/api/ingestion/status`

Returns the current status of the ingestion service.

### Health Check
**GET** `/api/ingestion/health`

Health check endpoint for monitoring.

### Get Configuration
**GET** `/api/ingestion/config`

Returns configuration information (without sensitive data).

### Validate JSON
**POST** `/api/ingestion/validate`

Validates JSON data without processing it.

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `FOLD_OPENROUTER_API_KEY` | OpenRouter API key (required) | - |
| `OPENROUTER_MODEL` | AI model to use | `anthropic/claude-3.5-sonnet` |
| `OPENROUTER_BASE_URL` | API base URL | `https://openrouter.ai/api/v1` |
| `INGESTION_ENABLED` | Enable/disable ingestion | `true` |
| `INGESTION_MAX_RETRIES` | Max API retries | `3` |
| `INGESTION_TIMEOUT_SECONDS` | API timeout | `30` |
| `INGESTION_AUTO_EXECUTE` | Auto-execute mutations | `true` |
| `INGESTION_DEFAULT_TRUST_DISTANCE` | Default trust distance | `0` |

### Example Configuration

```bash
# Required
export FOLD_OPENROUTER_API_KEY="sk-or-v1-your-key-here"

# Optional (with defaults shown)
export OPENROUTER_MODEL="anthropic/claude-3.5-sonnet"
export INGESTION_ENABLED="true"
export INGESTION_AUTO_EXECUTE="true"
export INGESTION_DEFAULT_TRUST_DISTANCE="0"
```

## 🏗️ Architecture

### Core Components

1. **Ingestion Core** (`core.rs`) - Main orchestrator
2. **OpenRouter Service** (`openrouter_service.rs`) - AI API integration
3. **Schema Stripper** (`schema_stripper.rs`) - Removes sensitive data for AI analysis
4. **Mutation Generator** (`mutation_generator.rs`) - Creates mutations from AI responses
5. **Simple Service** (`simple_service.rs`) - Simplified service for DataFoldNode integration
6. **Routes** (`routes.rs`) - HTTP API endpoints

### Data Flow

```
JSON Input → Schema Analysis → AI Processing → Schema Decision → Mutation Generation → Data Storage
```

1. **Input Validation** - Validates incoming JSON data
2. **Schema Retrieval** - Gets available schemas (stripped of payment/permissions)
3. **AI Analysis** - Sends data and schemas to OpenRouter for analysis
4. **Schema Decision** - Uses existing schema or creates new one based on AI response
5. **Mutation Generation** - Creates mutations to store the data
6. **Execution** - Executes mutations to persist data in DataFold

## 🤖 AI Integration

### OpenRouter API

The module uses OpenRouter to access various AI models. The default model is Claude 3.5 Sonnet, which provides excellent JSON analysis and schema generation capabilities.

### AI Response Format

The AI returns structured responses:

```json
{
  "existing_schemas": ["ProductCatalog", "InventoryItem"],
  "new_schemas": {
    "name": "UserProfile",
    "fields": {
      "name": {"type": "string"},
      "email": {"type": "string"},
      "age": {"type": "number"}
    }
  },
  "mutation_mappers": {
    "name": "UserProfile.name",
    "email": "UserProfile.email",
    "age": "UserProfile.age",
    "preferences.theme": "UserProfile.theme"
  }
}
```

### Field Path Mapping

- **JSON Paths**: Use dot notation with array indices (`user.preferences[0].value`)
- **Schema Paths**: Use schema.field format (`UserSchema.preferences["setting"]`)

## 📝 Examples

### Basic User Data

```bash
curl -X POST http://localhost:9001/api/ingestion/process \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "username": "johndoe",
      "email": "john@example.com",
      "profile": {
        "firstName": "John",
        "lastName": "Doe",
        "age": 30
      }
    }
  }'
```

### Product Catalog

```bash
curl -X POST http://localhost:9001/api/ingestion/process \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "product_id": "LAPTOP001",
      "name": "Gaming Laptop",
      "price": 1299.99,
      "category": "Electronics",
      "specs": {
        "cpu": "Intel i7",
        "ram": "16GB",
        "storage": "512GB SSD"
      },
      "tags": ["gaming", "laptop", "high-performance"]
    }
  }'
```

### Analytics Data

```bash
curl -X POST http://localhost:9001/api/ingestion/process \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "event": "page_view",
      "timestamp": "2024-01-15T10:30:00Z",
      "user_id": "user123",
      "page": "/products/laptop",
      "metadata": {
        "referrer": "google.com",
        "user_agent": "Mozilla/5.0...",
        "session_id": "sess_abc123"
      }
    }
  }'
```

## 🎛️ UI Configuration Management

### OpenRouter Configuration

The ingestion module now includes a web UI for managing OpenRouter configuration without requiring environment variables:

1. **Navigate to the Ingestion Tab** in the web interface at `http://localhost:9001`
2. **Configure OpenRouter Settings**:
   - Enter your OpenRouter API Key
   - Select the AI model (Claude 3.5 Sonnet recommended)
   - Click "Save Configuration"

The configuration is persisted to `./config/openrouter_config.json` and automatically loaded on server restart.

### Configuration API Endpoints

**GET** `/api/ingestion/openrouter-config`
```bash
curl http://localhost:9001/api/ingestion/openrouter-config
```

**POST** `/api/ingestion/openrouter-config`
```bash
curl -X POST http://localhost:9001/api/ingestion/openrouter-config \
  -H "Content-Type: application/json" \
  -d '{
    "api_key": "your-openrouter-api-key",
    "model": "anthropic/claude-3.5-sonnet"
  }'
```

## 🔍 Monitoring and Debugging

### Health Checks

```bash
# Check if ingestion service is healthy
curl http://localhost:9001/api/ingestion/health

# Get detailed status
curl http://localhost:9001/api/ingestion/status

# Get configuration
curl http://localhost:9001/api/ingestion/config
```

### Enhanced AI Response Logging

The ingestion module now provides comprehensive logging of AI interactions:

```
INFO - === FULL AI RESPONSE ===
INFO - AI Response (length: 1109 chars):
{
  "existing_schemas": [],
  "new_schemas": {
    "Product": {
      "name": "Product",
      "fields": {
        "category": {"type": "Single", "transform": null},
        "name": {"type": "Single", "transform": null},
        "price": {"type": "Single", "transform": null}
      }
    }
  },
  "mutation_mappers": {
    "category": "Product.category",
    "name": "Product.name",
    "price": "Product.price"
  }
}
INFO - === END AI RESPONSE ===
INFO - === FINAL PARSED AI RESPONSE ===
INFO - Existing schemas: []
INFO - New schemas: {"Product": {...}}
INFO - Mutation mappers: {"category": "Product.category", ...}
INFO - === END PARSED AI RESPONSE ===
```

### Standard Logs

The ingestion module provides detailed logging:

```
INFO - Starting JSON ingestion process
INFO - Retrieved 5 available schemas
INFO - Received AI recommendation: 1 existing schemas, new schema: false
INFO - Using existing schema: UserProfile
INFO - Generated 1 mutations
INFO - Ingestion completed successfully: schema 'UserProfile', 1 mutations generated, 1 executed
```

### Error Handling

Common error scenarios and responses:

```json
{
  "success": false,
  "errors": [
    "OpenRouter API error: Invalid API key",
    "Configuration error: FOLD_OPENROUTER_API_KEY environment variable not set"
  ]
}
```

## 🛠️ Development

### Building

```bash
cd fold_node
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run ingestion-specific tests
cargo test ingestion

# Run with logs
RUST_LOG=info cargo test ingestion
```

### Adding New Features

1. **New AI Providers**: Implement the AI service trait in `openrouter_service.rs`
2. **Custom Field Mappers**: Extend `mutation_generator.rs`
3. **New Endpoints**: Add routes in `routes.rs`

## 🔒 Security

### API Key Management

- Store API keys securely using environment variables
- Never commit API keys to version control
- Use different keys for development and production

### Input Validation

- All JSON input is validated before processing
- Schema definitions are validated before creation
- Mutation data is sanitized before execution

### Rate Limiting

The module includes built-in rate limiting:
- Configurable retry logic with exponential backoff
- Timeout protection for API calls
- Request queuing to prevent API abuse

## 🔐 Schema Permissions and UI Enhancements

### Automatic Permission Configuration

The ingestion module now automatically creates schemas with appropriate permissions:

- **Read Permissions**: Set to `NoRequirement` (anyone can read)
- **Write Permissions**: Set to `Trust Distance 0` (only trusted users can write)

This ensures that AI-generated schemas are immediately usable without permission errors.

### Enhanced Schema UI

The Schema tab now displays detailed permission information for each field:

- **Read Policy**: Shows trust distance requirements or "No Requirement"
- **Write Policy**: Shows trust distance requirements
- **Field Types**: Single, Collection, Range
- **Writable Status**: Visual indicator of write permissions

### Permission Policy Format

```json
{
  "permission_policy": {
    "read_policy": { "NoRequirement": null },
    "write_policy": { "Distance": 0 }
  }
}
```

### Troubleshooting Permission Issues

If you encounter "Read access denied" errors:

1. **Check Schema Permissions**: Use the Schema tab to view field permissions
2. **Verify Trust Distance**: Ensure your requests include appropriate trust distance
3. **Review Logs**: Check for permission-related error messages

## � Troubleshooting

### Common Issues

**404 Errors on Ingestion Endpoints**
- Ensure the server is running with the latest code
- Check that ingestion routes are properly registered
- Verify the server restarted after code changes

**OpenRouter API Errors**
- Verify `FOLD_OPENROUTER_API_KEY` is set correctly
- Check API key permissions and quotas
- Ensure network connectivity to OpenRouter

**Schema Creation Failures**
- Check AI response format matches expected structure
- Verify schema definitions are valid JSON
- Review logs for detailed error messages

**Mutation Execution Errors**
- Ensure target schema exists and is approved
- Check field mapping accuracy
- Verify data types match schema expectations

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run --bin datafold_http_server
```

## 📚 Additional Resources

- [Architecture Plan](INGESTION_MODULE_PLAN.md) - Detailed technical architecture
- [Usage Examples](INGESTION_EXAMPLE.md) - More usage examples and API documentation
- [DataFold Documentation](README.md) - Main DataFold system documentation

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

This project is licensed under the same license as the main DataFold project.

---

**🎉 The DataFold Ingestion Module is now ready for intelligent, automated data ingestion!**