# JSON Schema Definition Format

This document describes how to define FoldDB schemas using JSON. The JSON format allows you to specify all aspects of a schema including fields, permissions, and payment configurations.

## Basic Schema Structure

```json
{
  "name": "string",
  "fields": {
    "field_name": {
      "permission_policy": {
        "read_policy": {},
        "write_policy": {},
        "explicit_read_policy": {},
        "explicit_write_policy": {}
      },
      "ref_atom_uuid": "string",
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": {},
        "min_payment": null
      }
    }
  },
  "schema_mappers": [
    {
      "source_schema_name": "source_schema",
      "target_schema_name": "target_schema_name",
      "rules": []
    }
  ],
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment": 0
  }
}
```

## Schema Mapper Configuration

Schema mappers define how data from a source schema is transformed into the target schema format. Each schema can have multiple mappers, but each mapper operates on a single source schema.

```json
{
  "source_schema_name": "source_schema",
  "target_schema_name": "target_schema_name",
  "rules": [
    {
      "rule": "rename",
      "source_field": "old_name",
      "target_field": "new_name"
    },
    {
      "rule": "drop",
      "field": "unwanted_field"
    },
    {
      "rule": "map",
      "field_name": "field_to_map"
    }
  ]
}
```

### Mapping Rules

1. **Rename**: Move a field's value from source_field to target_field
   ```json
   {
     "rule": "rename",
     "source_field": "old_name",
     "target_field": "new_name"
   }
   ```

2. **Drop**: Remove a field from the output
   ```json
   {
     "rule": "drop",
     "field": "unwanted_field"
   }
   ```

3. **Map**: Apply default mapping for a field
   ```json
   {
     "rule": "map",
     "field_name": "field_to_map"
   }
   ```

## Permission Policy Definition

The permission policy can be defined using the following format:

```json
{
  "read_policy": {
    "Distance": 2
  },
  "write_policy": {
    "NoRequirement": null
  },
  "explicit_read_policy": {
    "counts_by_pub_key": {
      "pub_key1": 1,
      "pub_key2": 2
    }
  },
  "explicit_write_policy": {
    "counts_by_pub_key": {
      "pub_key1": 1
    }
  }
}
```

### Trust Distance Options

Trust distance can be specified in two ways:
1. `{"Distance": number}` - Specifies a required trust distance
2. `{"NoRequirement": null}` - Indicates no trust distance requirement

## Payment Configuration

### Field-Level Payment Config

```json
{
  "base_multiplier": 1.0,
  "trust_distance_scaling": {
    "Linear": {
      "slope": 0.1,
      "intercept": 1.0,
      "min_factor": 1.0
    }
  },
  "min_payment": 1000
}
```

### Trust Distance Scaling Options

1. Linear Scaling:
```json
{
  "Linear": {
    "slope": 0.1,
    "intercept": 1.0,
    "min_factor": 1.0
  }
}
```

2. Exponential Scaling:
```json
{
  "Exponential": {
    "base": 2.0,
    "scale": 0.5,
    "min_factor": 1.0
  }
}
```

3. No Scaling:
```json
{
  "None": null
}
```

### Schema-Level Payment Config

```json
{
  "base_multiplier": 1.0,
  "min_payment_threshold": 1000
}
```

## Complete Example

Here's a complete example of a schema definition:

```json
{
  "name": "UserProfile",
  "fields": {
    "username": {
      "permission_policy": {
        "read_policy": {
          "NoRequirement": null
        },
        "write_policy": {
          "Distance": 0
        },
        "explicit_read_policy": null,
        "explicit_write_policy": null
      },
      "ref_atom_uuid": "username_atom_123",
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": {
          "None": null
        },
        "min_payment": null
      }
    },
    "email": {
      "permission_policy": {
        "read_policy": {
          "Distance": 1
        },
        "write_policy": {
          "Distance": 0
        },
        "explicit_read_policy": {
          "counts_by_pub_key": {
            "trusted_service_key": 1
          }
        },
        "explicit_write_policy": null
      },
      "ref_atom_uuid": "email_atom_456",
      "payment_config": {
        "base_multiplier": 2.0,
        "trust_distance_scaling": {
          "Linear": {
            "slope": 0.5,
            "intercept": 1.0,
            "min_factor": 1.0
          }
        },
        "min_payment": 1000
      }
    }
  },
  "schema_mappers": [
    {
      "source_schema_name": "LegacyUser",
      "target_schema_name": "UserProfile",
      "rules": [
        {
          "rule": "rename",
          "source_field": "user_name",
          "target_field": "username"
        },
        {
          "rule": "map",
          "field_name": "email_address"
        }
      ]
    }
  ],
  "payment_config": {
    "base_multiplier": 1.5,
    "min_payment_threshold": 500
  }
}
```

## Validation Rules

1. All base_multiplier values must be positive (> 0.0)
2. For Linear and Exponential scaling, min_factor must be >= 1.0
3. For Exponential scaling, base must be positive
4. Schema min_payment_threshold must be >= 0
5. Field min_payment values must be positive if specified
6. Trust distances must be non-negative integers
7. Transform names must be valid and registered in the system
8. All ref_atom_uuid values must be valid UUIDs referencing existing atoms

## Usage

1. Create a JSON file following this schema definition format
2. Use the schema interpreter to load the JSON and convert it to a FoldDB schema
3. The interpreter will validate all constraints and relationships
4. If validation passes, the schema will be created in FoldDB

This format allows for complete definition of schemas including all permissions, payment configurations, and transformations in a single JSON file.
