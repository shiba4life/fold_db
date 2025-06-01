# Schema De-duping in DataFold

This document explains how schema de-duplication works in the DataFold system to prevent duplicate schemas and maintain data integrity.

## Overview

DataFold implements a multi-layered schema de-duping system that prevents duplicate schemas from being added to the `available_schemas` directory. The system checks for duplicates at both the file level and content level.

## De-duping Mechanisms

### 1. File-based De-duping (Primary)

**Location**: `fold_node/src/schema/core.rs` - `add_schema_to_available_directory` method

**How it works**:
```rust
// Check if schema file already exists
if target_path.exists() {
    // Read and compare content...
}
```

**Process**:
1. **File existence check**: Checks if a file with the same name already exists in `fold_node/available_schemas/`
2. **Content comparison**: If file exists, reads the existing schema and compares JSON content
3. **Identical content handling**: If content is identical, returns success (idempotent operation)
4. **Different content handling**: If content differs, returns an error requiring manual resolution

### 2. Content-based Comparison

**Method**: JSON serialization comparison
```rust
let new_json = serde_json::to_string(&json_schema).unwrap_or_default();
let existing_json = serde_json::to_string(&existing_schema).unwrap_or_default();

if new_json == existing_json {
    // Identical schemas - allow
} else {
    // Different schemas - reject
}
```

**Benefits**:
- ✅ Handles all field types automatically
- ✅ Compares nested structures correctly
- ✅ Accounts for field ordering differences
- ✅ Simple and reliable implementation

## De-duping Scenarios

### Scenario 1: Identical Schema Re-submission
```bash
# First submission
cargo run --bin datafold_cli -- add-schema example_schema.json
# Result: ✅ Schema added successfully

# Second submission (identical content)
cargo run --bin datafold_cli -- add-schema example_schema.json
# Result: ✅ Schema already exists with identical content - skipping
```

**Behavior**: Returns success, logs that schema already exists

### Scenario 2: Different Schema with Same Name
```bash
# First submission
cargo run --bin datafold_cli -- add-schema user_schema_v1.json --name UserSchema

# Second submission (different content, same name)
cargo run --bin datafold_cli -- add-schema user_schema_v2.json --name UserSchema
# Result: ❌ Error: Schema 'UserSchema' already exists with different content
```

**Behavior**: Returns error, requires manual resolution

### Scenario 3: Same Schema with Different Name
```bash
# First submission
cargo run --bin datafold_cli -- add-schema example_schema.json --name Schema1

# Second submission (identical content, different name)
cargo run --bin datafold_cli -- add-schema example_schema.json --name Schema2
# Result: ✅ Schema added (currently allowed)
```

**Behavior**: Currently allowed - only file-based de-duping is enforced

## Error Messages and Resolution

### Common De-duping Errors

#### 1. File Already Exists with Different Content
```
Error: Schema 'ExampleSchema' already exists with different content. 
Use a different name or remove the existing schema first.
```

**Resolution Options**:
```bash
# Option 1: Use a different name
cargo run --bin datafold_cli -- add-schema new_schema.json --name ExampleSchemaV2

# Option 2: Remove existing schema first
rm fold_node/available_schemas/ExampleSchema.json
cargo run --bin datafold_cli -- add-schema new_schema.json

# Option 3: Update existing schema manually
# Edit fold_node/available_schemas/ExampleSchema.json directly
```

#### 2. File Already Exists (Cannot Read/Parse)
```
Error: Schema 'ExampleSchema' already exists in available_schemas directory
```

**Resolution**:
```bash
# Check the existing file
cat fold_node/available_schemas/ExampleSchema.json

# If corrupted, remove and re-add
rm fold_node/available_schemas/ExampleSchema.json
cargo run --bin datafold_cli -- add-schema new_schema.json
```

## Implementation Details

### Database Level (`fold_node/src/schema/core.rs`)

```rust
pub fn add_schema_to_available_directory(
    &self,
    json_content: &str,
    schema_name: Option<String>,
) -> Result<String, SchemaError> {
    // 1. Parse and validate JSON schema
    let json_schema: JsonSchemaDefinition = serde_json::from_str(json_content)?;
    let validator = SchemaValidator::new(self);
    validator.validate_json_schema(&json_schema)?;
    
    // 2. Determine final schema name
    let final_name = schema_name.unwrap_or_else(|| json_schema.name.clone());
    
    // 3. Check for file-based duplicates
    let target_path = PathBuf::from("fold_node/available_schemas")
        .join(format!("{}.json", final_name));
    
    if target_path.exists() {
        // Content comparison logic...
    }
    
    // 4. Write schema file
    // 5. Load into memory
}
```

### CLI Level (`fold_node/src/bin/datafold_cli.rs`)

```rust
fn handle_add_schema(
    path: PathBuf,
    name: Option<String>,
    node: &mut DataFoldNode
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read schema file
    let schema_content = fs::read_to_string(&path)?;
    
    // 2. Use database-level validation and de-duping
    let final_schema_name = node.add_schema_to_available_directory(&schema_content, name)?;
    
    // 3. Refresh schemas in memory
    node.refresh_schemas()?;
}
```

### HTTP API Level (`fold_node/src/datafold_node/schema_routes.rs`)

```rust
pub async fn add_schema_to_available(
    schema_data: web::Json<serde_json::Value>, 
    state: web::Data<AppState>
) -> impl Responder {
    // 1. Convert JSON to string
    let json_content = serde_json::to_string(&*schema_data)?;
    
    // 2. Use database-level validation and de-duping
    match node_guard.add_schema_to_available_directory(&json_content, custom_name) {
        Ok(schema_name) => HttpResponse::Created().json(success_response),
        Err(e) => HttpResponse::BadRequest().json(error_response),
    }
}
```

## File System Structure

```
fold_node/available_schemas/
├── Analytics.json          ← Existing schema
├── BlogPost.json          ← Existing schema
├── ExampleSchema.json     ← Your new schema (checked for duplicates)
├── Inventory.json         ← Existing schema
├── Product.json           ← Existing schema
├── README.md              ← Documentation
├── TransformBase.json     ← Existing schema
├── TransformSchema.json   ← Existing schema
└── User.json              ← Existing schema
```

## De-duping Flow Diagram

```
┌─────────────────┐
│ Schema Submission│
└─────────┬───────┘
          │
          ▼
┌─────────────────┐
│ Parse & Validate │
│ JSON Schema     │
└─────────┬───────┘
          │
          ▼
┌─────────────────┐
│ Determine Final │
│ Schema Name     │
└─────────┬───────┘
          │
          ▼
┌─────────────────┐
│ Check if File   │
│ Already Exists  │
└─────────┬───────┘
          │
    ┌─────▼─────┐
    │ Exists?   │
    └─────┬─────┘
          │
    ┌─────▼─────┐
    │    No     │
    └─────┬─────┘
          │
          ▼
┌─────────────────┐
│ Write Schema    │
│ File & Load     │
└─────────────────┘

    ┌─────▼─────┐
    │   Yes     │
    └─────┬─────┘
          │
          ▼
┌─────────────────┐
│ Read Existing   │
│ Schema Content  │
└─────────┬───────┘
          │
          ▼
┌─────────────────┐
│ Compare JSON    │
│ Content         │
└─────────┬───────┘
          │
    ┌─────▼─────┐
    │ Same?     │
    └─────┬─────┘
          │
    ┌─────▼─────┐        ┌─────▼─────┐
    │   Yes     │        │    No     │
    └─────┬─────┘        └─────┬─────┘
          │                    │
          ▼                    ▼
┌─────────────────┐    ┌─────────────────┐
│ Return Success  │    │ Return Error    │
│ (Idempotent)    │    │ (Conflict)      │
└─────────────────┘    └─────────────────┘
```

## Best Practices for De-duping

### 1. Schema Naming
- Use descriptive, unique names
- Include version information if needed: `UserSchemaV2`
- Avoid generic names like `Schema1`, `Test`

### 2. Schema Updates
- For schema evolution, use versioned names
- Remove old schemas when no longer needed
- Document schema changes in version control

### 3. Team Workflows
```bash
# Before adding a schema, check existing schemas
cargo run --bin datafold_cli -- list-available-schemas

# Use descriptive commit messages
git add fold_node/available_schemas/NewSchema.json
git commit -m "Add NewSchema for user profile management"
```

### 4. Error Handling
```bash
# Always check the error message
cargo run --bin datafold_cli -- add-schema schema.json 2>&1 | tee schema_add.log

# For conflicts, investigate before resolving
ls -la fold_node/available_schemas/
cat fold_node/available_schemas/ConflictingSchema.json
```

## Future Enhancements

### Potential Improvements
1. **Content-based de-duping across names**: Detect identical schemas with different names
2. **Schema versioning**: Built-in support for schema evolution
3. **Semantic comparison**: Compare schema meaning rather than exact JSON
4. **Merge conflict resolution**: Interactive resolution of schema conflicts
5. **Schema dependencies**: Track relationships between schemas

### Configuration Options
Future versions might include:
```json
{
  "schema_deduping": {
    "mode": "strict|permissive|content_aware",
    "allow_identical_content_different_names": false,
    "auto_version_conflicts": true,
    "backup_replaced_schemas": true
  }
}
```

## Summary

DataFold's schema de-duping system provides:

- ✅ **File-based conflict detection**: Prevents overwriting existing schemas
- ✅ **Content-aware comparison**: Allows idempotent operations for identical schemas
- ✅ **Clear error messages**: Guides users to resolve conflicts
- ✅ **Multiple interfaces**: Works with CLI, HTTP API, and direct database calls
- ✅ **Validation integration**: Combines de-duping with schema validation
- ✅ **Atomic operations**: Either succeeds completely or fails safely

The system ensures schema integrity while providing flexibility for legitimate use cases like re-running deployment scripts or updating identical schemas.