# Schema Management

This document provides comprehensive guidance on schema definition, validation, immutability, and management in Fold DB.

## Table of Contents

1. [Schema Fundamentals](#schema-fundamentals)
2. [Field Types](#field-types)
3. [Permission Policies](#permission-policies)
4. [Payment Configuration](#payment-configuration)
5. [Schema Immutability](#schema-immutability)
6. [Field Mappers](#field-mappers)
7. [Schema Validation](#schema-validation)
8. [Best Practices](#best-practices)
9. [Examples](#examples)

## Schema Fundamentals

### Schema Structure

A schema defines the structure, permissions, and behavior of data in Fold DB:

```json
{
  "name": "SchemaName",
  "fields": {
    "field_name": {
      "field_type": "Single|Collection|Range",
      "permission_policy": {
        "read_policy": "permission_requirement",
        "write_policy": "permission_requirement",
        "explicit_read_policy": "optional_override",
        "explicit_write_policy": "optional_override"
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": "scaling_config",
        "min_payment": "optional_minimum"
      },
      "field_mappers": {},
      "transform": "optional_transform_definition",
      "writable": true
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 0
  }
}
```

### Schema Loading

**Via HTTP API:**
```bash
curl -X POST http://localhost:9001/api/schema \
  -H "Content-Type: application/json" \
  -d @schema.json
```

**Via CLI:**
```bash
datafold_cli load-schema schema.json
```

**Via TCP:**
```json
{
  "app_id": "my-app",
  "operation": "create_schema",
  "params": {
    "schema": { /* schema definition */ }
  }
}
```

### Schema Validation

**Validation Rules:**
- Schema name must be unique within the node
- Field names must be valid identifiers (alphanumeric + underscore)
- Permission policies must specify valid requirements
- Payment configurations must have positive values
- Transform definitions must be syntactically valid

**Validation Response:**
```json
{
  "valid": true,
  "errors": [],
  "warnings": [
    "Field 'deprecated_field' has no read restrictions"
  ]
}
```

## Field Types

### Single Fields

Store scalar values (strings, numbers, booleans).

```json
{
  "username": {
    "field_type": "Single",
    "permission_policy": {
      "read_policy": {"NoRequirement": null},
      "write_policy": {"Distance": 1}
    },
    "payment_config": {
      "base_multiplier": 1.0,
      "trust_distance_scaling": {"None": null},
      "min_payment": null
    }
  }
}
```

**Use Cases:**
- User identifiers
- Configuration values
- Timestamps
- Numeric metrics
- Boolean flags

### Collection Fields

Store arrays of values.

```json
{
  "tags": {
    "field_type": "Collection",
    "permission_policy": {
      "read_policy": {"NoRequirement": null},
      "write_policy": {"Distance": 0}
    },
    "payment_config": {
      "base_multiplier": 1.2,
      "min_payment": null
    }
  }
}
```

**Data Example:**
```json
{
  "tags": ["technology", "database", "distributed"]
}
```

**Use Cases:**
- Tags and categories
- Lists of related items
- Multiple values per field
- Dynamic arrays

### Range Fields

Store key-value pairs for hierarchical and time-series data.

```json
{
  "metrics_by_timeframe": {
    "field_type": "Range",
    "permission_policy": {
      "read_policy": {"NoRequirement": null},
      "write_policy": {"Distance": 0}
    },
    "payment_config": {
      "base_multiplier": 1.5,
      "trust_distance_scaling": {
        "Linear": {
          "slope": 0.2,
          "intercept": 1.0,
          "min_factor": 1.0
        }
      },
      "min_payment": 100
    }
  }
}
```

**Data Example:**
```json
{
  "metrics_by_timeframe": {
    "2024-01-01:daily": "1250",
    "2024-01-01:hourly:00": "45",
    "2024-01-01:hourly:01": "52",
    "2024-01-02:daily": "1180"
  }
}
```

**Query Capabilities:**
- **Key**: Exact key match
- **KeyPrefix**: Keys starting with prefix
- **KeyRange**: Keys within lexicographic range
- **Keys**: Multiple specific keys
- **KeyPattern**: Glob pattern matching
- **Value**: Match by value

**Use Cases:**
- Time-series data
- Hierarchical structures
- Configuration by environment
- Metrics by dimension
- Inventory by location

## Permission Policies

### Permission Types

**NoRequirement:**
```json
{
  "read_policy": {"NoRequirement": null}
}
```
Public access, no restrictions.

**Distance-Based:**
```json
{
  "read_policy": {"Distance": 1}
}
```
Requires specific trust distance or closer.

**Public Key:**
```json
{
  "read_policy": {"PublicKey": "ed25519:ABC123..."}
}
```
Requires specific public key for access.

**Explicit Permission:**
```json
{
  "read_policy": {"Explicit": "admin_access"}
}
```
Requires explicit permission grant.

### Permission Hierarchy

**Field-Level Permissions:**
Override schema-level defaults for specific fields.

```json
{
  "name": "UserProfile",
  "fields": {
    "public_info": {
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 1}
      }
    },
    "private_data": {
      "permission_policy": {
        "read_policy": {"Distance": 0},
        "write_policy": {"Distance": 0}
      }
    },
    "admin_only": {
      "permission_policy": {
        "read_policy": {"Explicit": "admin_read"},
        "write_policy": {"Explicit": "admin_write"}
      }
    }
  }
}
```

### Explicit Permission Overrides

**Runtime Permission Grants:**
```bash
curl -X POST http://localhost:9001/api/permissions/explicit \
  -H "Content-Type: application/json" \
  -d '{
    "schema": "UserProfile",
    "field": "private_data",
    "permission": "read",
    "public_key": "ed25519:ABC123...",
    "expires_at": "2024-12-31T23:59:59Z"
  }'
```

**Schema-Level Overrides:**
```json
{
  "permission_policy": {
    "read_policy": {"Distance": 1},
    "write_policy": {"Distance": 1},
    "explicit_read_policy": {
      "ed25519:ABC123...": {"NoRequirement": null}
    },
    "explicit_write_policy": {
      "ed25519:DEF456...": {"Distance": 0}
    }
  }
}
```

## Payment Configuration

### Payment Models

**Flat Fee:**
```json
{
  "payment_config": {
    "base_multiplier": 1.0,
    "trust_distance_scaling": {"None": null},
    "min_payment": 100
  }
}
```

**Linear Scaling:**
```json
{
  "payment_config": {
    "base_multiplier": 2.0,
    "trust_distance_scaling": {
      "Linear": {
        "slope": 1.0,
        "intercept": 0.0,
        "min_factor": 1.0
      }
    },
    "min_payment": 50
  }
}
```
Formula: `fee = base_multiplier * (slope * distance + intercept)`

**Exponential Scaling:**
```json
{
  "payment_config": {
    "base_multiplier": 3.0,
    "trust_distance_scaling": {
      "Exponential": {
        "base": 1.5,
        "scale": 1.0,
        "min_factor": 1.0
      }
    },
    "min_payment": 25
  }
}
```
Formula: `fee = base_multiplier * (base^distance * scale)`

### Schema-Level Payment Configuration

**Default Payment Rules:**
```json
{
  "name": "Analytics",
  "payment_config": {
    "base_multiplier": 1.5,
    "min_payment_threshold": 150
  },
  "fields": {
    /* fields inherit these defaults unless overridden */
  }
}
```

### Payment Calculation Examples

**Example 1: Linear Scaling**
- Base multiplier: 2.0
- Linear: slope=1.0, intercept=0.0
- Trust distance: 2
- Calculation: 2.0 * (1.0 * 2 + 0.0) = 4.0 sats

**Example 2: Exponential Scaling**
- Base multiplier: 3.0
- Exponential: base=1.5, scale=1.0
- Trust distance: 2
- Calculation: 3.0 * (1.5^2 * 1.0) = 6.75 sats

## Schema Immutability

### Core Principles

**Schema Immutability:**
- Schemas in fold db are immutable once created
- Schema structure cannot be modified after creation
- To change schema structure, create a new schema with a different name
- Schema immutability ensures data consistency and integrity

**Benefits of Immutability:**
- **Data Consistency**: Ensures all data conforms to a fixed structure
- **Predictable Behavior**: Applications can rely on consistent schema definitions
- **Simplified Concurrency**: No need to handle concurrent schema modifications
- **Version Control**: Clear versioning through distinct schema names

### Migration Patterns

When you need structural changes, follow these recommended patterns:

**Pattern 1: Versioned Schemas**
```json
{
  "name": "UserProfileV1",
  "fields": {
    "username": {"field_type": "Single"},
    "email": {"field_type": "Single"}
  }
}
```

```json
{
  "name": "UserProfileV2",
  "fields": {
    "username": {"field_type": "Single"},
    "email": {"field_type": "Single"},
    "created_at": {"field_type": "Single"},
    "profile_settings": {"field_type": "Range"}
  }
}
```

**Pattern 2: Feature-Specific Schemas**
```json
{
  "name": "UserBasic",
  "fields": {
    "username": {"field_type": "Single"},
    "email": {"field_type": "Single"}
  }
}
```

```json
{
  "name": "UserExtended",
  "fields": {
    "username": {"field_type": "Single"},
    "email": {"field_type": "Single"},
    "profile_data": {"field_type": "Range"},
    "preferences": {"field_type": "Range"}
  }
}
```

### Data Migration Strategy

**Step 1: Create New Schema**
```bash
curl -X POST http://localhost:9001/api/schema \
  -H "Content-Type: application/json" \
  -d @new_schema.json
```

**Step 2: Migrate Data**
```bash
# Query data from old schema
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"query\",\"schema\":\"UserProfileV1\",\"fields\":[\"username\",\"email\"]}"
  }'

# Insert data into new schema with additional fields
curl -X POST http://localhost:9001/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "{\"type\":\"mutation\",\"schema\":\"UserProfileV2\",\"operation\":\"create\",\"data\":{\"username\":\"alice\",\"email\":\"alice@example.com\",\"created_at\":\"2024-01-15T10:30:00Z\"}}"
  }'
```

**Step 3: Update Applications**
Update your applications to use the new schema name:
```rust
// Old code
let query = Query::new("UserProfileV1")
    .select(&["username", "email"]);

// New code
let query = Query::new("UserProfileV2")
    .select(&["username", "email", "created_at"]);
```

**Step 4: Remove Old Schema (Optional)**
```bash
curl -X DELETE http://localhost:9001/api/schema/UserProfileV1
```

### Schema Lifecycle Management

**Development Phase:**
- Create experimental schemas with descriptive names
- Use prefixes like `dev_`, `test_`, or `experimental_`
- Example: `dev_UserProfile_feature_x`

**Production Phase:**
- Use semantic versioning in schema names
- Example: `UserProfile_v1_0_0`, `UserProfile_v2_0_0`
- Maintain clear documentation of changes between versions

**Deprecation Phase:**
- Mark schemas as deprecated in documentation
- Provide migration guides for applications
- Keep deprecated schemas available during transition period

## Field Mappers

### Cross-Schema Mapping

**Field Mapping Configuration:**
```json
{
  "name": "UserProfileV2",
  "fields": {
    "user_name": {
      "field_type": "Single",
      "field_mappers": {
        "UserProfile.username": {}
      }
    },
    "contact_email": {
      "field_type": "Single",
      "field_mappers": {
        "UserProfile.email": {}
      }
    },
    "profile_summary": {
      "field_type": "Single",
      "field_mappers": {
        "UserProfile.profile_data": {
          "key_filter": "bio"
        }
      }
    }
  }
}
```

### Range Field Mapping

**Key-Based Mapping:**
```json
{
  "location_data": {
    "field_type": "Range",
    "field_mappers": {
      "UserProfile.profile_data": {
        "key_pattern": "location:*",
        "key_transform": "remove_prefix:location:"
      }
    }
  }
}
```

**Value Transformation:**
```json
{
  "normalized_metrics": {
    "field_type": "Range",
    "field_mappers": {
      "Analytics.raw_metrics": {
        "value_transform": "multiply:100"
      }
    }
  }
}
```

### Mapping Functions

**Built-in Transformations:**
- `remove_prefix:PREFIX` - Remove prefix from keys
- `add_prefix:PREFIX` - Add prefix to keys
- `multiply:FACTOR` - Multiply numeric values
- `divide:FACTOR` - Divide numeric values
- `uppercase` - Convert to uppercase
- `lowercase` - Convert to lowercase
- `trim` - Remove whitespace

**Custom Mapping Functions:**
```json
{
  "field_mappers": {
    "SourceSchema.source_field": {
      "custom_transform": "my_transform_function",
      "parameters": {
        "param1": "value1",
        "param2": "value2"
      }
    }
  }
}
```

## Schema Validation

### Validation Levels

**Strict Validation:**
```json
{
  "validation": {
    "level": "strict",
    "enforce_types": true,
    "require_all_fields": true,
    "allow_extra_fields": false
  }
}
```

**Permissive Validation:**
```json
{
  "validation": {
    "level": "permissive", 
    "enforce_types": false,
    "require_all_fields": false,
    "allow_extra_fields": true
  }
}
```

### Field Validation Rules

**Single Field Validation:**
```json
{
  "email": {
    "field_type": "Single",
    "validation": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
      "max_length": 255
    }
  }
}
```

**Range Field Validation:**
```json
{
  "metrics": {
    "field_type": "Range",
    "validation": {
      "key_pattern": "^[a-zA-Z0-9_:-]+$",
      "value_type": "number",
      "max_keys": 1000
    }
  }
}
```

**Collection Field Validation:**
```json
{
  "tags": {
    "field_type": "Collection",
    "validation": {
      "item_type": "string",
      "max_items": 50,
      "unique_items": true
    }
  }
}
```

### Validation API

**Schema Validation:**
```bash
curl -X POST http://localhost:9001/api/schema/validate \
  -H "Content-Type: application/json" \
  -d @schema.json
```

**Data Validation:**
```bash
curl -X POST http://localhost:9001/api/data/validate \
  -H "Content-Type: application/json" \
  -d '{
    "schema": "UserProfile",
    "data": {
      "username": "alice",
      "email": "alice@example.com"
    }
  }'
```

## Best Practices

### Schema Design

**1. Use Descriptive Names:**
```json
{
  "name": "UserProfile",  // Good: clear and descriptive
  "fields": {
    "created_at": {},     // Good: indicates timestamp
    "usr_nm": {}          // Bad: unclear abbreviation
  }
}
```

**2. Choose Appropriate Field Types:**
```json
{
  "user_id": {
    "field_type": "Single"     // Good: scalar identifier
  },
  "tags": {
    "field_type": "Collection" // Good: multiple values
  },
  "metrics_by_date": {
    "field_type": "Range"      // Good: key-value structure
  }
}
```

**3. Design for Evolution:**
```json
{
  "name": "UserProfile",
  "version": "1.0.0",
  "fields": {
    "username": {
      "field_type": "Single",
      "validation": {
        "type": "string",
        "immutable": true  // Username cannot change
      }
    },
    "profile_data": {
      "field_type": "Range"  // Flexible for future additions
    }
  }
}
```

### Permission Strategy

**1. Principle of Least Privilege:**
```json
{
  "public_info": {
    "permission_policy": {
      "read_policy": {"NoRequirement": null},
      "write_policy": {"Distance": 1}
    }
  },
  "sensitive_data": {
    "permission_policy": {
      "read_policy": {"Distance": 0},
      "write_policy": {"Distance": 0}
    }
  }
}
```

**2. Layered Security:**
```json
{
  "admin_controls": {
    "permission_policy": {
      "read_policy": {"Explicit": "admin_read"},
      "write_policy": {"Explicit": "admin_write"}
    }
  }
}
```

### Payment Configuration

**1. Fair Pricing:**
```json
{
  "free_tier_data": {
    "payment_config": {
      "base_multiplier": 0.0  // Free access
    }
  },
  "premium_analytics": {
    "payment_config": {
      "base_multiplier": 1.0,
      "min_payment": 100      // Reasonable minimum
    }
  }
}
```

**2. Distance-Based Scaling:**
```json
{
  "public_metrics": {
    "payment_config": {
      "base_multiplier": 1.0,
      "trust_distance_scaling": {
        "Linear": {
          "slope": 0.5,
          "intercept": 1.0,
          "min_factor": 1.0
        }
      }
    }
  }
}
```

### Performance Optimization

**1. Index Strategy:**
```json
{
  "indexed_fields": {
    "field_type": "Single",
    "indexing": {
      "enabled": true,
      "type": "btree"
    }
  }
}
```

**2. Range Field Design:**
```json
{
  "time_series_data": {
    "field_type": "Range",
    "key_structure": "YYYY-MM-DD:HH:mm:ss",  // Sortable format
    "partitioning": "daily"                   // Partition strategy
  }
}
```

## Examples

### User Management Schema

```json
{
  "name": "UserProfile",
  "version": "1.0.0",
  "fields": {
    "user_id": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      },
      "payment_config": {
        "base_multiplier": 0.0
      },
      "validation": {
        "type": "string",
        "pattern": "^[a-zA-Z0-9_-]+$",
        "immutable": true
      }
    },
    "username": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 1}
      },
      "payment_config": {
        "base_multiplier": 0.0
      }
    },
    "email": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"Distance": 1},
        "write_policy": {"Distance": 1}
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "min_payment": 10
      }
    },
    "profile_settings": {
      "field_type": "Range",
      "permission_policy": {
        "read_policy": {"Distance": 0},
        "write_policy": {"Distance": 0}
      },
      "payment_config": {
        "base_multiplier": 0.0
      }
    },
    "public_data": {
      "field_type": "Range",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 1}
      },
      "payment_config": {
        "base_multiplier": 0.0
      }
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 5
  }
}
```

### Analytics Schema

```json
{
  "name": "EventAnalytics",
  "version": "1.0.0",
  "fields": {
    "event_id": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      }
    },
    "event_name": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      }
    },
    "event_type": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      }
    },
    "metrics_by_timeframe": {
      "field_type": "Range",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      },
      "payment_config": {
        "base_multiplier": 1.2,
        "trust_distance_scaling": {
          "Linear": {
            "slope": 0.2,
            "intercept": 1.0,
            "min_factor": 1.0
          }
        },
        "min_payment": 100
      }
    },
    "user_segments": {
      "field_type": "Range",
      "permission_policy": {
        "read_policy": {"Distance": 1},
        "write_policy": {"Distance": 0}
      },
      "payment_config": {
        "base_multiplier": 1.5,
        "trust_distance_scaling": {
          "Linear": {
            "slope": 0.3,
            "intercept": 1.0,
            "min_factor": 1.0
          }
        },
        "min_payment": 200
      }
    }
  },
  "payment_config": {
    "base_multiplier": 1.1,
    "min_payment_threshold": 150
  }
}
```

### E-commerce Schema

```json
{
  "name": "ProductCatalog",
  "version": "1.0.0",
  "fields": {
    "product_id": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      }
    },
    "name": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 1}
      }
    },
    "description": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 1}
      }
    },
    "price": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 0}
      }
    },
    "inventory_by_location": {
      "field_type": "Range",
      "permission_policy": {
        "read_policy": {"Distance": 1},
        "write_policy": {"Distance": 0}
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "min_payment": 50
      }
    },
    "attributes": {
      "field_type": "Range",
      "permission_policy": {
        "read_policy": {"NoRequirement": null},
        "write_policy": {"Distance": 1}
      }
    }
  }
}
```

---

**Next**: See [Transforms](./transforms.md) for transform system and DSL documentation.