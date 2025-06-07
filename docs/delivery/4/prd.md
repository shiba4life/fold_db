# PBI-4: Schema Purification and Schema Approval ARef Creation

## Overview

This PBI removes `aref_uuid` references from schema definitions to ensure schema purity and implements automatic AtomRef creation during the schema approval process using the composable AtomRef framework.

[View in Backlog](../backlog.md#user-content-4)

## Problem Statement

Currently, schema definitions can directly reference `aref_uuids`, which pollutes the declarative nature of schemas and creates tight coupling between schema definitions and runtime AtomRef instances. AtomRefs are also created at runtime during field operations rather than during schema approval, leading to inconsistent state management.

## User Stories

### Primary User Story
As a developer, I want schema definitions to be pure (without aref_uuid references) and ARefs created during schema approval, so that schemas are declarative and ARefs are managed by the system.

### Supporting User Stories
- As a schema designer, I want schema definitions to be purely declarative, so that they can be version controlled and shared without runtime dependencies
- As a system administrator, I want all ARefs created during schema approval, so that the system state is consistent and predictable
- As a developer, I want automatic ARef-to-field mapping, so that I don't need to manually manage these relationships
- As a developer, I want migration support for existing schemas, so that the transition is seamless

## Technical Approach

### Schema Definition Purification

#### Remove ref_atom_uuid from JsonSchemaField
```rust
/// Clean schema field definition without ARef references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaField {
    pub permission_policy: JsonPermissionPolicy,
    // ❌ pub ref_atom_uuid: Option<String>, // REMOVED
    pub payment_config: JsonFieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    pub field_type: FieldType, // Now supports composable types
    pub transform: Option<JsonTransform>,
}

/// Legacy schema field for migration support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyJsonSchemaField {
    pub permission_policy: JsonPermissionPolicy,
    pub ref_atom_uuid: Option<String>, // Keep for migration
    pub payment_config: JsonFieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    pub field_type: FieldType,
    pub transform: Option<JsonTransform>,
}
```

#### Update FieldCommon Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCommon {
    pub permission_policy: PermissionsPolicy,
    pub payment_config: FieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    pub transform: Option<Transform>,
    pub writable: bool,
    // ❌ pub ref_atom_uuid: Option<String>, // REMOVED
}

/// Internal ARef mapping maintained separately from schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldARefMapping {
    pub schema_name: String,
    pub field_name: String,
    pub aref_uuid: String,
    pub field_type: FieldType,
    pub created_at: DateTime<Utc>,
}
```

### Schema Approval Process Enhancement

#### Enhanced Schema Manager
```rust
impl SchemaManager {
    /// Approve schema and create all required ARefs
    pub fn approve_schema(&self, schema_name: &str) -> Result<Vec<FieldARefMapping>, SchemaError> {
        let schema = self.get_schema(schema_name)?;
        let mut mappings = Vec::new();
        
        // Create ARefs for all fields using the factory
        for (field_name, field_def) in &schema.fields {
            let field_type = self.determine_field_type(field_def);
            let aref = self.create_aref_for_field(field_type, "system")?;
            
            // Store the ARef
            let aref_uuid = aref.uuid().to_string();
            self.store_composable_aref(&aref_uuid, aref)?;
            
            // Create mapping
            let mapping = FieldARefMapping {
                schema_name: schema_name.to_string(),
                field_name: field_name.clone(),
                aref_uuid: aref_uuid.clone(),
                field_type,
                created_at: Utc::now(),
            };
            
            mappings.push(mapping);
        }
        
        // Update schema state to Approved
        self.set_schema_state(schema_name, SchemaState::Approved)?;
        
        // Store ARef mappings
        self.store_aref_mappings(schema_name, &mappings)?;
        
        Ok(mappings)
    }
    
    /// Create appropriate ARef for field type using factory
    fn create_aref_for_field(
        &self, 
        field_type: FieldType, 
        source_pub_key: &str
    ) -> Result<ComposableAtomRef, SchemaError> {
        let factory = AtomRefFactory::new();
        
        let (container_type, element_type) = field_type.composition();
        
        factory.create_composable(
            container_type,
            element_type,
            source_pub_key.to_string()
        ).map_err(|e| SchemaError::ARefCreationFailed(e.to_string()))
    }
    
    /// Store composable ARef in database
    fn store_composable_aref(
        &self, 
        aref_uuid: &str, 
        aref: ComposableAtomRef
    ) -> Result<(), SchemaError> {
        let key = format!("aref:{}", aref_uuid);
        self.db_ops.store_item(&key, &aref)
            .map_err(|e| SchemaError::DatabaseError(e.to_string()))
    }
    
    /// Store ARef-to-field mappings
    fn store_aref_mappings(
        &self, 
        schema_name: &str, 
        mappings: &[FieldARefMapping]
    ) -> Result<(), SchemaError> {
        let key = format!("schema_mappings:{}", schema_name);
        self.db_ops.store_item(&key, mappings)
            .map_err(|e| SchemaError::DatabaseError(e.to_string()))
    }
    
    /// Retrieve ARef mappings for a schema
    pub fn get_aref_mappings(&self, schema_name: &str) -> Result<Vec<FieldARefMapping>, SchemaError> {
        let key = format!("schema_mappings:{}", schema_name);
        self.db_ops.get_item::<Vec<FieldARefMapping>>(&key)
            .map_err(|e| SchemaError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SchemaError::ARefMappingNotFound(schema_name.to_string()))
    }
    
    /// Get ARef UUID for a specific field
    pub fn get_field_aref_uuid(&self, schema_name: &str, field_name: &str) -> Result<String, SchemaError> {
        let mappings = self.get_aref_mappings(schema_name)?;
        mappings.iter()
            .find(|m| m.field_name == field_name)
            .map(|m| m.aref_uuid.clone())
            .ok_or_else(|| SchemaError::FieldARefNotFound {
                schema_name: schema_name.to_string(),
                field_name: field_name.to_string(),
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    // ... existing errors ...
    #[error("ARef creation failed: {0}")]
    ARefCreationFailed(String),
    #[error("ARef mapping not found for schema: {0}")]
    ARefMappingNotFound(String),
    #[error("Field ARef not found: {schema_name}.{field_name}")]
    FieldARefNotFound { schema_name: String, field_name: String },
}
```

### Database Operations Enhancement

#### Composable ARef Storage
```rust
impl DbOperations {
    /// Store a composable AtomRef
    pub fn store_composable_aref(
        &self, 
        aref_uuid: &str, 
        aref: &ComposableAtomRef
    ) -> Result<(), DbError> {
        let key = format!("aref:{}", aref_uuid);
        self.store_item(&key, aref)
    }
    
    /// Retrieve a composable AtomRef
    pub fn get_composable_aref(&self, aref_uuid: &str) -> Result<Option<ComposableAtomRef>, DbError> {
        let key = format!("aref:{}", aref_uuid);
        self.get_item(&key)
    }
    
    /// Update a composable AtomRef
    pub fn update_composable_aref(
        &self, 
        aref_uuid: &str, 
        aref: &ComposableAtomRef
    ) -> Result<(), DbError> {
        // Validate ARef exists
        if self.get_composable_aref(aref_uuid)?.is_none() {
            return Err(DbError::ARefNotFound(aref_uuid.to_string()));
        }
        
        self.store_composable_aref(aref_uuid, aref)
    }
    
    /// Delete a composable AtomRef
    pub fn delete_composable_aref(&self, aref_uuid: &str) -> Result<(), DbError> {
        let key = format!("aref:{}", aref_uuid);
        self.delete_item(&key)
    }
}
```

### Migration Support

#### Legacy Schema Migration
```rust
impl SchemaManager {
    /// Migrate legacy schema with ref_atom_uuids to pure schema
    pub fn migrate_legacy_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        // Load legacy schema
        let legacy_schema = self.load_legacy_schema(schema_name)?;
        
        // Extract existing ref_atom_uuids
        let mut existing_mappings = Vec::new();
        for (field_name, field_def) in &legacy_schema.fields {
            if let Some(ref_atom_uuid) = &field_def.ref_atom_uuid {
                // Check if ARef exists in database
                if let Ok(Some(legacy_aref)) = self.db_ops.get_item::<AtomRef>(&format!("ref:{}", ref_atom_uuid)) {
                    // Convert legacy ARef to composable ARef
                    let composable_aref = ComposableAtomRef::Single(
                        AtomRefContainer::Single(legacy_aref)
                    );
                    
                    // Store as composable ARef
                    self.store_composable_aref(ref_atom_uuid, composable_aref)?;
                    
                    // Create mapping
                    existing_mappings.push(FieldARefMapping {
                        schema_name: schema_name.to_string(),
                        field_name: field_name.clone(),
                        aref_uuid: ref_atom_uuid.clone(),
                        field_type: field_def.field_type.clone(),
                        created_at: Utc::now(),
                    });
                }
            }
        }
        
        // Convert to pure schema (remove ref_atom_uuids)
        let pure_schema = self.convert_to_pure_schema(legacy_schema)?;
        
        // Store pure schema
        self.persist_schema(&pure_schema)?;
        
        // Store existing mappings
        if !existing_mappings.is_empty() {
            self.store_aref_mappings(schema_name, &existing_mappings)?;
        }
        
        // Create missing ARefs for fields without them
        self.create_missing_arefs(schema_name, &pure_schema, &existing_mappings)?;
        
        Ok(())
    }
    
    /// Convert legacy schema to pure schema
    fn convert_to_pure_schema(&self, legacy_schema: LegacySchema) -> Result<Schema, SchemaError> {
        let mut fields = HashMap::new();
        
        for (field_name, legacy_field) in legacy_schema.fields {
            let pure_field = self.convert_legacy_field(legacy_field)?;
            fields.insert(field_name, pure_field);
        }
        
        Ok(Schema {
            name: legacy_schema.name,
            schema_type: legacy_schema.schema_type,
            fields,
            payment_config: legacy_schema.payment_config,
            hash: legacy_schema.hash,
        })
    }
}
```

### Implementation Plan

#### Phase 1: Schema Definition Cleanup (Days 1-2)
1. Remove `ref_atom_uuid` from `JsonSchemaField`
2. Create `LegacyJsonSchemaField` for migration support
3. Update `FieldCommon` structure
4. Create `FieldARefMapping` type

#### Phase 2: Schema Approval Enhancement (Days 3-4)
1. Implement enhanced `approve_schema` method
2. Add ARef creation using factory
3. Implement ARef-to-field mapping storage
4. Add ARef retrieval methods

#### Phase 3: Database Integration (Days 5-6)
1. Add composable ARef storage operations
2. Implement ARef mapping persistence
3. Add validation and error handling
4. Update database schema if needed

#### Phase 4: Migration Support (Days 7)
1. Implement legacy schema migration
2. Add ARef conversion utilities
3. Create migration validation
4. Add rollback capabilities

## UX/UI Considerations

### Schema Creation Experience
- Pure schema definitions without technical implementation details
- Clear separation between schema design and system implementation
- Automatic ARef management invisible to schema designers

### Migration Experience
- Seamless migration for existing schemas
- Clear migration status reporting
- Rollback capabilities for failed migrations

## Acceptance Criteria

1. **Schema Purification**
   - [ ] `ref_atom_uuid` field removed from `JsonSchemaField`
   - [ ] `LegacyJsonSchemaField` created for migration support
   - [ ] `FieldCommon` updated without ARef references
   - [ ] All schema serialization/deserialization updated

2. **Schema Approval Process**
   - [ ] Schema approval creates all required ARefs using factory
   - [ ] ARef-to-field mapping established and stored
   - [ ] Database operations support composable ARef storage/retrieval
   - [ ] Schema state properly updated to Approved

3. **ARef Management**
   - [ ] `FieldARefMapping` type implemented
   - [ ] ARef mappings stored and retrieved correctly
   - [ ] Field ARef lookup methods working
   - [ ] Composable ARef database operations complete

4. **Migration Support**
   - [ ] Legacy schema detection and migration
   - [ ] Existing ARefs converted to composable format
   - [ ] Missing ARefs created for unmapped fields
   - [ ] Migration validation and rollback support

5. **Error Handling**
   - [ ] Clear error messages for ARef creation failures
   - [ ] Graceful handling of missing mappings
   - [ ] Validation for schema approval prerequisites
   - [ ] Recovery procedures for failed operations

6. **Testing**
   - [ ] Unit tests for schema approval process
   - [ ] Integration tests for ARef creation and mapping
   - [ ] Migration tests for various legacy schema formats
   - [ ] Error scenario testing

## Dependencies

- **Prerequisite**: PBI-2 (Hash AtomRef Type and Composable Framework)
- **Prerequisite**: PBI-3 (Extended Field Type System)
- **Internal**: Current schema management, database operations
- **External**: Existing database schema for migration

## Open Questions

1. **Migration Strategy**: Should migration be automatic or manual for existing schemas?
2. **Performance Impact**: How does ARef creation during approval affect schema approval performance?
3. **Rollback Complexity**: What level of rollback support is needed for failed migrations?
4. **Validation Timing**: Should ARef validation happen during schema creation or approval?

## Related Tasks

This PBI will generate detailed tasks covering:
- JsonSchemaField structure updates and migration support
- Schema approval process enhancement with ARef creation
- Database operations for composable ARef storage
- Legacy schema migration and validation
- Error handling and recovery procedures 