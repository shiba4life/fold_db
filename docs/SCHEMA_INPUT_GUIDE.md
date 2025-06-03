# Schema Input Guide for DataFold

This guide explains how to input new JSON schemas into the DataFold 'available schemas' folder using the various methods now available.

## Overview

DataFold now provides multiple ways to add new schemas to the `fold_node/available_schemas/` directory with built-in validation:

1. **CLI Command** - Command-line interface for adding schemas
2. **HTTP API** - REST endpoint for web-based schema submission
3. **Database-level validation** - All methods use comprehensive validation

## Schema Format

All schemas must follow the DataFold JSON schema format. Here's the structure:

```json
{
  "name": "SchemaName",
  "fields": {
    "field_name": {
      "permission_policy": {
        "read_policy": { "Distance": 0 },
        "write_policy": { "Distance": 1 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": "None",
        "min_payment": null
      },
      "ref_atom_uuid": null,
      "field_type": "Single",
      "field_mappers": {},
      "transform": null,
      "writable": true
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 1
  }
}
```

### Field Types
- `"Single"` - Individual values
- `"Collection"` - Arrays of values  
- `"Range"` - Value ranges with filtering capabilities

### Trust Distance Scaling
- `"None"` - No scaling based on trust distance
- `"Linear"` - Linear scaling with slope, intercept, and min_factor
- `"Exponential"` - Exponential scaling with base, scale, and min_factor

## Method 1: CLI Command

### Usage
```bash
# Add schema (validation is always enforced)
cargo run --bin datafold_cli -- add-schema path/to/schema.json

# Add schema with custom name
cargo run --bin datafold_cli -- add-schema path/to/schema.json --name CustomName
```

### Examples
```bash
# Add the example schema
cargo run --bin datafold_cli -- add-schema example_schema.json

# Add with custom name
cargo run --bin datafold_cli -- add-schema example_schema.json --name MyCustomSchema
```

### CLI Features
- ✅ **Always enforced validation** using database-level validation
- ✅ Automatic schema name detection from JSON or filename
- ✅ Custom naming support
- ✅ Duplicate detection
- ✅ Automatic schema refresh after addition

## Method 2: HTTP API

### Endpoint
```
POST /schemas/available/add
Content-Type: application/json
```

### Request Body
Send the complete schema JSON as the request body:

```json
{
  "name": "ExampleSchema",
  "fields": {
    // ... schema fields
  },
  "payment_config": {
    // ... payment configuration
  }
}
```

### Response
```json
{
  "success": true,
  "schema_name": "ExampleSchema",
  "message": "Schema 'ExampleSchema' added to available_schemas directory and is ready for approval"
}
```

### Error Response
```json
{
  "error": "Schema validation failed: [specific error message]"
}
```

### Examples

#### Using curl
```bash
curl -X POST http://localhost:9001/schemas/available/add \
  -H "Content-Type: application/json" \
  -d @example_schema.json
```

#### Using JavaScript/fetch
```javascript
const response = await fetch('http://localhost:9001/schemas/available/add', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify(schemaData)
});

const result = await response.json();
console.log(result);
```

### HTTP API Features
- ✅ Full schema validation
- ✅ RESTful interface
- ✅ JSON response format
- ✅ Detailed error messages
- ✅ Automatic schema loading into memory

## Database-Level Validation

All methods use comprehensive validation that checks:

### Structure Validation
- ✅ Schema must be a valid JSON object
- ✅ Required fields: `name`, `fields`
- ✅ Schema name must be a non-empty string
- ✅ Fields must be properly structured

### Field Validation
- ✅ Each field must have `permission_policy`, `payment_config`, `field_type`
- ✅ Valid field types: `Single`, `Collection`, `Range`
- ✅ Permission policies must have `read_policy` and `write_policy`
- ✅ Payment config must have positive `base_multiplier`
- ✅ Min payment cannot be zero if specified

### Transform Validation
- ✅ Transform syntax validation
- ✅ Input/output field reference validation
- ✅ Cross-schema reference validation
- ✅ Circular dependency detection

### Business Logic Validation
- ✅ Duplicate schema name detection
- ✅ File system validation
- ✅ JSON formatting validation

## Schema Lifecycle

After adding a schema, it goes through these states:

1. **Available** - Schema is discovered and validated but not active
2. **Approved** - Schema is active and can be queried/mutated
3. **Blocked** - Schema is blocked from queries/mutations

### Managing Schema States

```bash
# List schemas by state
cargo run --bin datafold_cli -- list-schemas-by-state --state available
cargo run --bin datafold_cli -- list-schemas-by-state --state approved
cargo run --bin datafold_cli -- list-schemas-by-state --state blocked

# Approve a schema for use
cargo run --bin datafold_cli -- approve-schema --name ExampleSchema

# Block a schema
cargo run --bin datafold_cli -- block-schema --name ExampleSchema
```

### HTTP API for Schema Management

```bash
# List available schemas
curl http://localhost:9001/schemas/available

# Get schema status
curl http://localhost:9001/schemas/status

# Approve a schema
curl -X POST http://localhost:9001/schema/ExampleSchema/approve

# Block a schema
curl -X POST http://localhost:9001/schema/ExampleSchema/block
```

## File System Structure

Schemas are stored in:
```
fold_node/available_schemas/
├── Analytics.json
├── BlogPost.json
├── ExampleSchema.json  ← Your new schema
├── Inventory.json
├── Product.json
├── README.md
├── TransformBase.json
├── TransformSchema.json
└── User.json
```

## Error Handling

### Common Validation Errors

1. **Missing required fields**
   ```
   Schema must have a 'name' field
   Schema must have a 'fields' field
   ```

2. **Invalid field configuration**
   ```
   Field 'fieldname' missing required property 'field_type'
   Field 'fieldname' has invalid field_type. Must be one of: Single, Collection, Range
   ```

3. **Payment configuration errors**
   ```
   Field 'fieldname' base_multiplier must be positive
   Field 'fieldname' min_payment cannot be zero
   ```

4. **Duplicate schema**
   ```
   Schema 'ExampleSchema' already exists in available_schemas directory
   ```

### Troubleshooting

1. **Validation fails**: Check the error message and fix the schema structure
2. **File already exists**: Use a different name or remove the existing file
3. **Permission errors**: Ensure write access to `fold_node/available_schemas/`
4. **JSON syntax errors**: Validate JSON format using a JSON validator

## Best Practices

1. **Validation is always enforced** - All schemas are automatically validated for security and integrity
2. **Use descriptive names** - Schema names should be clear and unique
3. **Test schemas** - Add schemas to a test environment first
4. **Document fields** - Use clear field names and appropriate permission policies
5. **Version control** - Keep schema files in version control
6. **Backup** - Backup the available_schemas directory before making changes

## Integration Examples

### Web Application Integration

```html
<!DOCTYPE html>
<html>
<head>
    <title>Schema Upload</title>
</head>
<body>
    <form id="schemaForm">
        <textarea id="schemaJson" placeholder="Paste your schema JSON here..."></textarea>
        <button type="submit">Add Schema</button>
    </form>

    <script>
        document.getElementById('schemaForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const schemaJson = document.getElementById('schemaJson').value;
            
            try {
                const response = await fetch('http://localhost:9001/schemas/available/add', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: schemaJson
                });
                
                const result = await response.json();
                if (result.success) {
                    alert(`Schema '${result.schema_name}' added successfully!`);
                } else {
                    alert(`Error: ${result.error}`);
                }
            } catch (error) {
                alert(`Network error: ${error.message}`);
            }
        });
    </script>
</body>
</html>
```

### Python Integration

```python
import requests
import json

def add_schema_to_datafold(schema_data, base_url="http://localhost:9001"):
    """Add a schema to DataFold via HTTP API"""
    url = f"{base_url}/schemas/available/add"
    
    try:
        response = requests.post(url, json=schema_data)
        result = response.json()
        
        if response.status_code == 201:
            print(f"✅ Schema '{result['schema_name']}' added successfully!")
            return result['schema_name']
        else:
            print(f"❌ Error: {result['error']}")
            return None
    except Exception as e:
        print(f"❌ Network error: {e}")
        return None

# Example usage
with open('example_schema.json', 'r') as f:
    schema = json.load(f)

add_schema_to_datafold(schema)
```

## Summary

DataFold now provides robust, validated schema input capabilities through:

- **CLI**: `cargo run --bin datafold_cli -- add-schema <path>`
- **HTTP API**: `POST /schemas/available/add`
- **Database validation**: Comprehensive validation at the database level

All methods ensure schema integrity and provide clear error messages for troubleshooting. Schemas are automatically loaded into memory and ready for approval after addition.