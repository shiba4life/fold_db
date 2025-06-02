# RangeSchema DB-Level Validation Enforcement

## Overview

This document describes the comprehensive DB-level validation enforcement that ensures all fields in a RangeSchema use the same range_key. This validation prevents data corruption and maintains schema consistency by enforcing critical constraints at the database level.

## Design Constraint

**CRITICAL CONSTRAINT**: All fields within a RangeSchema MUST be Range fields and MUST use the same range_key value.

This constraint ensures:
- **Data Consistency**: All data within a RangeSchema is partitioned by the same key
- **Query Efficiency**: Range queries can efficiently access related data
- **Schema Integrity**: Prevents mixing of field types within range-based schemas
- **Operational Safety**: Eliminates data corruption from inconsistent range keys

## Why This Design Decision Was Made

### 1. **Data Integrity Protection**
Without this constraint, developers could accidentally create RangeSchemas with:
- Mixed field types (Single, Collection, Range)
- Different range_key values across fields
- Missing range_key fields

### 2. **Query Performance Optimization**
RangeSchemas are designed for efficient range-based queries. Mixed field types or inconsistent range keys would:
- Break range query semantics
- Cause performance degradation
- Lead to unpredictable query results

### 3. **Operational Safety**
DB-level enforcement prevents:
- Runtime errors from inconsistent schemas
- Data corruption from range key mismatches
- Silent failures in range-based operations

## DB-Level Enforcement Implementation

### Schema Creation/Update Validation

The validation is enforced at multiple levels:

#### 1. **Programmatic Schema Validation** (`fold_node/src/schema/validator.rs`)

```rust
/// Validates that all fields in a RangeSchema are Range fields and maintain consistency.
fn validate_range_field_consistency(&self, schema: &Schema, range_key: &str) -> Result<(), SchemaError> {
    // 1. Ensure range_key field exists and is a Range field
    match schema.fields.get(range_key) {
        Some(FieldVariant::Range(_)) => {
            // Valid - range_key is a Range field
        }
        Some(field_variant) => {
            // ERROR: range_key field is not a Range field
            return Err(SchemaError::InvalidField(format!(
                "RangeSchema '{}' has range_key field '{}' that is a {} field, but range_key must be a Range field",
                schema.name, range_key, field_type
            )));
        }
        None => {
            // ERROR: range_key field doesn't exist
            return Err(SchemaError::InvalidField(format!(
                "RangeSchema '{}' range_key field '{}' does not exist in the schema",
                schema.name, range_key
            )));
        }
    }

    // 2. Validate ALL fields are Range fields
    for (field_name, field_variant) in &schema.fields {
        match field_variant {
            FieldVariant::Range(_) => {
                // Valid - this is a Range field
                continue;
            }
            FieldVariant::Single(_) => {
                // ERROR: Single field in RangeSchema
                return Err(SchemaError::InvalidField(format!(
                    "RangeSchema '{}' contains Single field '{}', but ALL fields must be Range fields",
                    schema.name, field_name
                )));
            }
            FieldVariant::Collection(_) => {
                // ERROR: Collection field in RangeSchema
                return Err(SchemaError::InvalidField(format!(
                    "RangeSchema '{}' contains Collection field '{}', but ALL fields must be Range fields",
                    schema.name, field_name
                )));
            }
        }
    }

    Ok(())
}
```

#### 2. **JSON Schema Definition Validation**

```rust
/// Validates JSON RangeSchema field consistency
fn validate_json_range_field_consistency(&self, schema: &JsonSchemaDefinition, range_key: &str) -> Result<(), SchemaError> {
    // 1. Ensure range_key field exists
    let range_key_field = schema.fields.get(range_key)
        .ok_or_else(|| SchemaError::InvalidField(format!(
            "JSON RangeSchema '{}' is missing the range_key field '{}'",
            schema.name, range_key
        )))?;

    // 2. Ensure range_key field is Range type
    if !matches!(range_key_field.field_type, FieldType::Range) {
        return Err(SchemaError::InvalidField(format!(
            "JSON RangeSchema '{}' has range_key field '{}' defined as {:?} field, but it must be a Range field",
            schema.name, range_key, range_key_field.field_type
        )));
    }

    // 3. Validate ALL fields are Range type
    for (field_name, field_def) in &schema.fields {
        if !matches!(field_def.field_type, FieldType::Range) {
            return Err(SchemaError::InvalidField(format!(
                "JSON RangeSchema '{}' contains {} field '{}', but ALL fields must be Range fields",
                schema.name, field_type_name, field_name
            )));
        }
    }

    Ok(())
}
```

### Mutation-Time Validation

#### 3. **Range Mutation Validation** (`fold_node/src/fold_db_core/mutation.rs`)

```rust
/// Validates Range schema mutations to ensure:
/// - Range schema mutations MUST include a range_key field
/// - All fields in Range schemas are RangeFields
/// - All fields have consistent range_key values
fn validate_range_schema_mutation(&self, schema: &Schema, mutation: &Mutation) -> Result<(), SchemaError> {
    if let Some(range_key) = schema.range_key() {
        // 1. MANDATORY: Range schema mutations MUST include the range_key field
        let range_key_value = mutation.fields_and_values.get(range_key)
            .ok_or_else(|| SchemaError::InvalidData(format!(
                "Range schema mutation for '{}' is missing required range_key field '{}'",
                schema.name, range_key
            )))?;
        
        // 2. Validate range_key value is not null or empty
        if range_key_value.is_null() {
            return Err(SchemaError::InvalidData(format!(
                "Range schema mutation for '{}' has null value for range_key field '{}'",
                schema.name, range_key
            )));
        }
        
        // 3. Validate all fields are RangeFields
        for (field_name, field_variant) in &schema.fields {
            match field_variant {
                FieldVariant::Range(_) => {
                    // Valid
                }
                FieldVariant::Single(_) => {
                    return Err(SchemaError::InvalidData(format!(
                        "Range schema '{}' contains Single field '{}', but all fields must be RangeFields",
                        schema.name, field_name
                    )));
                }
                FieldVariant::Collection(_) => {
                    return Err(SchemaError::InvalidData(format!(
                        "Range schema '{}' contains Collection field '{}', but all fields must be RangeFields",
                        schema.name, field_name
                    )));
                }
            }
        }
        
        // 4. Validate all mutation field values have consistent range_key
        for (field_name, field_value) in &mutation.fields_and_values {
            if field_name == range_key {
                continue; // Skip the range_key field itself
            }
            
            if let Some(value_obj) = field_value.as_object() {
                if let Some(field_range_value) = value_obj.get(range_key) {
                    if field_range_value != range_key_value {
                        return Err(SchemaError::InvalidData(format!(
                            "Inconsistent range_key value in field '{}': expected {:?}, got {:?}",
                            field_name, range_key_value, field_range_value
                        )));
                    }
                }
            }
        }
    }
    
    Ok(())
}
```

## Validation Enforcement Points

The validation is enforced at these critical points:

### 1. **Schema Creation Time**
- **Location**: `SchemaValidator::validate()`
- **Trigger**: When creating new RangeSchemas programmatically
- **Validation**: Ensures all fields are Range fields and range_key exists

### 2. **JSON Schema Loading Time**
- **Location**: `SchemaValidator::validate_json_schema()`
- **Trigger**: When loading schemas from JSON definitions
- **Validation**: Ensures all field_type values are "Range" and range_key is defined

### 3. **Mutation Execution Time**
- **Location**: `FoldDB::validate_range_schema_mutation()`
- **Trigger**: Before processing any mutation on a RangeSchema
- **Validation**: Ensures mutation includes range_key and all values are consistent

### 4. **Query Execution Time**
- **Location**: `Schema::validate_range_filter()`
- **Trigger**: When executing queries against RangeSchemas
- **Validation**: Ensures queries include proper range_filter with correct range_key

## Error Messages and Recovery

### Schema Creation Errors

```
// Missing range_key field
RangeSchema range_key 'user_id' must be one of the schema's fields.

// Non-Range field in RangeSchema
RangeSchema 'UserData' contains Single field 'name', but ALL fields must be Range fields. 
Consider using a regular Schema (not RangeSchema) if you need Single fields, 
or convert 'name' to a Range field to maintain RangeSchema consistency.

// Wrong field type for range_key
RangeSchema 'UserData' has range_key field 'user_id' that is a Single field, but range_key must be a Range field
```

### JSON Schema Errors

```
// Missing range_key field in JSON
JSON RangeSchema 'UserData' is missing the range_key field 'user_id'. 
The range_key must be defined as a field in the schema.

// Wrong field_type in JSON
JSON RangeSchema 'UserData' contains Single field 'name', but ALL fields must be Range fields. 
Consider using a regular Schema (not RangeSchema) if you need Single fields, 
or change 'name' to field_type: "Range" to maintain RangeSchema consistency.
```

### Mutation Errors

```
// Missing range_key in mutation
Range schema mutation for 'UserData' is missing required range_key field 'user_id'. 
All range schema mutations must provide a value for the range_key.

// Null range_key value
Range schema mutation for 'UserData' has null value for range_key field 'user_id'. 
Range key must have a valid value.

// Inconsistent range_key values
Inconsistent range_key value in field 'score': expected "user123", got "user456"
```

## Valid vs Invalid Examples

### ✅ Valid RangeSchema

```rust
// Programmatic creation
let mut schema = Schema::new_range("UserScores".to_string(), "user_id".to_string());

// All fields are Range fields
let user_id_field = RangeField::new(
    PermissionsPolicy::default(),
    FieldPaymentConfig::default(),
    HashMap::new(),
);
schema.fields.insert("user_id".to_string(), FieldVariant::Range(user_id_field));

let score_field = RangeField::new(
    PermissionsPolicy::default(),
    FieldPaymentConfig::default(),
    HashMap::new(),
);
schema.fields.insert("score".to_string(), FieldVariant::Range(score_field));

// ✅ Validation passes - all fields are Range fields
```

```json
// JSON Schema Definition
{
  "name": "UserScores",
  "schema_type": {
    "Range": {
      "range_key": "user_id"
    }
  },
  "fields": {
    "user_id": {
      "field_type": "Range",
      "permission_policy": { "read_policy": {"Distance": 0}, "write_policy": {"Distance": 0} },
      "payment_config": { "base_multiplier": 1.0, "trust_distance_scaling": "None" }
    },
    "score": {
      "field_type": "Range",
      "permission_policy": { "read_policy": {"Distance": 0}, "write_policy": {"Distance": 0} },
      "payment_config": { "base_multiplier": 1.0, "trust_distance_scaling": "None" }
    }
  },
  "payment_config": { "base_multiplier": 1.0, "min_payment_threshold": 0 }
}
```

### ❌ Invalid RangeSchema Examples

#### 1. Mixed Field Types (INVALID)

```rust
let mut schema = Schema::new_range("UserData".to_string(), "user_id".to_string());

// Range field (valid)
let user_id_field = RangeField::new(/*...*/);
schema.fields.insert("user_id".to_string(), FieldVariant::Range(user_id_field));

// ❌ Single field in RangeSchema (INVALID)
let name_field = SingleField::new(/*...*/);
schema.fields.insert("name".to_string(), FieldVariant::Single(name_field));

// Validation ERROR: "RangeSchema 'UserData' contains Single field 'name', but ALL fields must be Range fields"
```

#### 2. Missing Range Key Field (INVALID)

```rust
let mut schema = Schema::new_range("UserData".to_string(), "user_id".to_string());

// ❌ Missing the range_key field "user_id"
let score_field = RangeField::new(/*...*/);
schema.fields.insert("score".to_string(), FieldVariant::Range(score_field));

// Validation ERROR: "RangeSchema range_key 'user_id' must be one of the schema's fields"
```

#### 3. Wrong Field Type in JSON (INVALID)

```json
{
  "name": "UserData",
  "schema_type": { "Range": { "range_key": "user_id" } },
  "fields": {
    "user_id": {
      "field_type": "Range"
    },
    "name": {
      "field_type": "Single"  // ❌ INVALID: Single field in RangeSchema
    }
  }
}
```

### Valid Mutation Examples

```json
// ✅ Valid RangeSchema mutation
{
  "schema_name": "UserScores",
  "mutation_type": "create",
  "fields_and_values": {
    "user_id": "user123",  // ✅ Range key field present
    "score": {             // ✅ Range field with embedded range_key
      "user_id": "user123",
      "value": 95,
      "timestamp": "2024-01-01T00:00:00Z"
    }
  },
  "pub_key": "test_key",
  "trust_distance": 0
}
```

### Invalid Mutation Examples

```json
// ❌ Missing range_key field
{
  "schema_name": "UserScores",
  "mutation_type": "create",
  "fields_and_values": {
    // ❌ Missing "user_id" field
    "score": {
      "value": 95,
      "timestamp": "2024-01-01T00:00:00Z"
    }
  }
}

// ❌ Inconsistent range_key values
{
  "schema_name": "UserScores",
  "mutation_type": "create",
  "fields_and_values": {
    "user_id": "user123",
    "score": {
      "user_id": "user456",  // ❌ Different range_key value
      "value": 95
    }
  }
}
```

## Benefits of DB-Level Enforcement

### 1. **Early Error Detection**
- Catches schema definition errors at creation time
- Prevents invalid schemas from being stored
- Provides clear error messages for debugging

### 2. **Data Consistency Guarantee**
- Ensures all RangeSchema data follows the same partitioning scheme
- Prevents data corruption from inconsistent range keys
- Maintains referential integrity within range partitions

### 3. **Query Performance Protection**
- Guarantees that range queries will work efficiently
- Prevents performance degradation from mixed field types
- Ensures predictable query behavior

### 4. **Operational Safety**
- Eliminates runtime errors from schema inconsistencies
- Provides fail-fast behavior for invalid operations
- Simplifies debugging and maintenance

## Migration and Compatibility

### Existing Schemas
- Existing valid RangeSchemas continue to work without changes
- Invalid schemas will be rejected during validation
- Clear error messages guide migration to valid structures

### Schema Updates
- Any updates to RangeSchemas must pass validation
- Adding fields requires them to be Range fields
- Changing field types is prevented to maintain consistency

### Rollback Safety
- Invalid schema changes are rejected before persistence
- No partial updates that could corrupt data
- Safe rollback to previous valid state always possible

## Conclusion

The DB-level enforcement of RangeSchema field consistency provides:
- **Robust data integrity** through comprehensive validation
- **Clear error messages** for quick problem resolution
- **Performance optimization** by ensuring proper schema structure
- **Operational safety** through fail-fast validation

This validation system ensures that RangeSchemas maintain their semantic guarantees and provide reliable, efficient data access patterns while preventing common configuration errors that could lead to data corruption or performance issues.