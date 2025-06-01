# Ingestion Module Example

This document demonstrates how to use the ingestion module with the DataFold system.

## Setup

### Option 1: Environment Variables (Traditional)

1. Set the required environment variables:
```bash
export OPENROUTER_API_KEY="your-openrouter-api-key"
export INGESTION_ENABLED="true"
export OPENROUTER_MODEL="anthropic/claude-3.5-sonnet"
```

2. Start the DataFold HTTP server:
```bash
./run_http_server.sh
```

### Option 2: UI Configuration (Recommended)

1. Start the server without environment variables:
```bash
./run_http_server.sh
```

2. Navigate to `http://localhost:9001` and click on the "Ingestion" tab

3. Configure OpenRouter settings in the UI:
   - Enter your OpenRouter API Key
   - Select the AI model (Claude 3.5 Sonnet recommended)
   - Click "Save Configuration"

The configuration will be saved to `./config/openrouter_config.json` and persist across server restarts.

**Alternative manual start:**
```bash
cd fold_node
cargo run --bin datafold_http_server -- --port 9001
```

## API Endpoints

The ingestion module provides the following HTTP endpoints:

### 1. Process JSON Data
**POST** `/api/ingestion/process`

Process JSON data and automatically create schemas or map to existing ones.

**Request Body:**
```json
{
  "data": {
    "name": "John Doe",
    "email": "john@example.com",
    "age": 30,
    "preferences": {
      "theme": "dark",
      "notifications": true
    }
  },
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

### 2. Get Ingestion Status
**GET** `/api/ingestion/status`

Get the current status of the ingestion module.

**Response:**
```json
{
  "enabled": true,
  "configured": true,
  "model": "anthropic/claude-3.5-sonnet",
  "auto_execute_mutations": true,
  "default_trust_distance": 0
}
```

### 3. Health Check
**GET** `/api/ingestion/health`

Check if the ingestion service is healthy and ready to process requests.

**Response:**
```json
{
  "status": "healthy",
  "service": "ingestion",
  "details": {
    "enabled": true,
    "configured": true,
    "model": "anthropic/claude-3.5-sonnet"
  }
}
```

### 4. Get Configuration
**GET** `/api/ingestion/config`

Get the current ingestion configuration (without sensitive data).

**Response:**
```json
{
  "enabled": true,
  "model": "anthropic/claude-3.5-sonnet",
  "auto_execute_mutations": true,
  "default_trust_distance": 0,
  "api_key_configured": true,
  "configured": true
}
```

### 5. Validate JSON
**POST** `/api/ingestion/validate`

Validate JSON data without processing it.

**Request Body:**
```json
{
  "name": "Test User",
  "data": [1, 2, 3]
}
```

**Response:**
```json
{
  "valid": true,
  "message": "JSON data is valid for ingestion"
}
```

### 6. OpenRouter Configuration Management
**GET** `/api/ingestion/openrouter-config`

Get the current OpenRouter configuration.

**Response:**
```json
{
  "api_key_configured": true,
  "model": "anthropic/claude-3.5-sonnet",
  "base_url": "https://openrouter.ai/api/v1"
}
```

**POST** `/api/ingestion/openrouter-config`

Save OpenRouter configuration.

**Request Body:**
```json
{
  "api_key": "your-openrouter-api-key",
  "model": "anthropic/claude-3.5-sonnet"
}
```

**Response:**
```json
{
  "success": true,
  "message": "OpenRouter configuration saved successfully"
}
```

## Enhanced Logging and Monitoring

The ingestion module now provides comprehensive logging of AI interactions. When processing data, you'll see detailed logs including:

### Full AI Response Logging
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
```

### Parsed Response Details
```
INFO - === FINAL PARSED AI RESPONSE ===
INFO - Existing schemas: []
INFO - New schemas: {"Product": {...}}
INFO - Mutation mappers: {"category": "Product.category", ...}
INFO - === END PARSED AI RESPONSE ===
```

### Schema Creation and Permission Logs
```
INFO - Creating new schema from AI definition
INFO - Found wrapped schema with name: Product
INFO - Creating basic schema from wrapped definition for: Product
INFO - Processing 6 fields for schema Product
INFO - Successfully created schema 'Product' with 6 fields
```

## Example Usage with curl

### Process JSON Data
```bash
curl -X POST http://localhost:9001/api/ingestion/process \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "product_name": "Laptop",
      "price": 999.99,
      "category": "Electronics",
      "in_stock": true,
      "tags": ["computer", "portable", "work"]
    },
    "auto_execute": true
  }'
```

### Check Status
```bash
curl http://localhost:9001/api/ingestion/status
```

### Health Check
```bash
curl http://localhost:9001/api/ingestion/health
```

## How It Works

1. **JSON Analysis**: The ingestion module sends your JSON data along with available schemas to OpenRouter AI
2. **Schema Decision**: The AI determines whether to use an existing schema or create a new one
3. **Schema Creation**: If needed, a new schema is created and automatically approved
4. **Mutation Generation**: The module generates mutations to store your data according to the schema
5. **Data Storage**: Mutations are executed to persist your data in the DataFold system

## AI Response Format

The AI service returns responses in this format:

```json
{
  "existing_schemas": ["ProductCatalog", "InventoryItem"],
  "new_schemas": {
    "name": "ProductCatalog",
    "fields": {
      "product_name": {"type": "string"},
      "price": {"type": "number"},
      "category": {"type": "string"},
      "in_stock": {"type": "boolean"},
      "tags": {"type": "array"}
    }
  },
  "mutation_mappers": {
    "product_name": "ProductCatalog.product_name",
    "price": "ProductCatalog.price",
    "category": "ProductCatalog.category",
    "in_stock": "ProductCatalog.in_stock",
    "tags[0]": "ProductCatalog.tags[\"tag_value\"]"
  }
}
```

## Error Handling

The ingestion module provides detailed error information:

```json
{
  "success": false,
  "schema_used": null,
  "new_schema_created": false,
  "mutations_generated": 0,
  "mutations_executed": 0,
  "errors": [
    "OpenRouter API error: Invalid API key",
    "Configuration error: OPENROUTER_API_KEY environment variable not set"
  ]
}
```

## Configuration Options

Environment variables for configuration:

- `OPENROUTER_API_KEY` - Your OpenRouter API key (required)
- `OPENROUTER_MODEL` - AI model to use (default: "anthropic/claude-3.5-sonnet")
- `OPENROUTER_BASE_URL` - API base URL (default: "https://openrouter.ai/api/v1")
- `INGESTION_ENABLED` - Enable/disable ingestion (default: "true")
- `INGESTION_MAX_RETRIES` - Max API retries (default: "3")
- `INGESTION_TIMEOUT_SECONDS` - API timeout (default: "30")
- `INGESTION_AUTO_EXECUTE` - Auto-execute mutations (default: "true")
- `INGESTION_DEFAULT_TRUST_DISTANCE` - Default trust distance (default: "0")