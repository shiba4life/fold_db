# Composable AtomRef Types Design Document

## Executive Summary

This document outlines the design and implementation plan for composable AtomRef types in the Datafold system. The proposal introduces a hierarchical AtomRef system supporting up to 2-layer composition (e.g., `Range:Collection`, `Hash:Range`) while ensuring schema definitions remain pure and ARefs are created during schema approval rather than at runtime.

## Table of Contents

1. [Current State Analysis](#current-state-analysis)
2. [Requirements](#requirements)
3. [Proposed Architecture](#proposed-architecture)
4. [System Changes Required](#system-changes-required)
5. [Implementation Plan](#implementation-plan)
6. [Migration Strategy](#migration-strategy)
7. [Testing Strategy](#testing-strategy)
8. [Risk Assessment](#risk-assessment)

## Current State Analysis

### Existing AtomRef Types

The current system supports three basic AtomRef types:

1. **AtomRef (Single)** - Points to a single atom UUID
2. **AtomRefRange** - Stores key-value pairs in a BTreeMap<String, String>
3. **AtomRefCollection** - Stores an ordered list of atom UUIDs

### Current Schema Definition Process

```rust
pub struct JsonSchemaField {
    pub permission_policy: JsonPermissionPolicy,
    pub ref_atom_uuid: Option<String>, // ❌ Schema definitions can reference ARefs
    pub payment_config: JsonFieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    pub field_type: FieldType, // Single, Collection, Range
    pub transform: Option<JsonTransform>,
}
```

### Problems with Current System

1. **Non-Composable:** Cannot combine types (e.g., Range of Collections)
2. **Schema Pollution:** Schema definitions can directly reference `aref_uuids`
3. **Runtime Creation:** ARefs are created during field operations, not schema approval
4. **Limited Flexibility:** Only three basic patterns supported
5. **Missing Hash Type:** No hash-based indexing available

## Requirements

### Functional Requirements

1. **Composable Types:** Support 2-layer composition (Container:Element)
2. **Base Types:** Range, Hash, Collection as fundamental building blocks
3. **Schema Purity:** Schema definitions must not reference `aref_uuids`
4. **Early Creation:** All ARefs created during schema approval
5. **Backward Compatibility:** Existing single-layer types continue to work

### Non-Functional Requirements

1. **Performance:** No significant performance degradation
2. **Memory Efficiency:** Reasonable memory usage for nested structures
3. **Type Safety:** Compile-time guarantees for composition validity
4. **Maintainability:** Clear separation of concerns

## Proposed Architecture

### Composable Type System

```rust
/// Base AtomRef container types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomRefContainer {
    Range(AtomRefRange),
    Hash(AtomRefHash),
    Collection(AtomRefCollection),
}

/// Composable AtomRef supporting up to 2-layer nesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposableAtomRef {
    /// Single-layer AtomRef (current behavior)
    Single(AtomRefContainer),
    /// Two-layer composition: Container<Element>
    Composed {
        container: AtomRefContainer,
        element_type: AtomRefType,
    },
}

/// AtomRef type discriminator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomRefType {
    Range,
    Hash, 
    Collection,
}
```

### New Hash Type Implementation

```rust
/// Hash-based AtomRef for key-value lookups with O(1) access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefHash {
    uuid: String,
    /// HashMap for O(1) key lookups
    pub atom_uuids: HashMap<String, String>,
    updated_at: DateTime<Utc>,
    status: AtomRefStatus,
    update_history: Vec<AtomRefUpdate>,
}
```

### Field Type Extensions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Single,
    Collection,
    Range,
    Hash,              // New: Hash-based field
    RangeCollection,   // New: Range of Collections
    RangeHash,         // New: Range of Hashes  
    HashCollection,    // New: Hash of Collections
    HashRange,         // New: Hash of Ranges
    CollectionRange,   // New: Collection of Ranges
    CollectionHash,    // New: Collection of Hashes
}
```

### Pure Schema Definition

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
```

## System Changes Required

### Core Components

#### 1. AtomRef Module (`src/atom/`)

**New Files:**
- `atom_ref_hash.rs` - Hash-based AtomRef implementation
- `composable_atom_ref.rs` - Main composable type system
- `atom_ref_factory.rs` - Factory for creating ARefs during schema approval

**Modified Files:**
- `mod.rs` - Export new types
- `atom_ref_types.rs` - Add new type discriminators

#### 2. Schema Module (`src/schema/`)

**Modified Files:**
- `types/field/common.rs` - Remove `ref_atom_uuid` field access
- `types/field/variant.rs` - Add new composable field variants
- `types/json_schema.rs` - Remove `ref_atom_uuid` from schema definitions
- `core.rs` - Implement ARef creation during schema approval

#### 3. Database Operations (`src/fold_db_core/`)

**Modified Files:**
- `managers/atom/request_handlers.rs` - Handle composable ARef types
- `managers/atom/field_processing.rs` - Use pre-created ARefs instead of creating them
- `operations/db_operations.rs` - Support composable ARef storage/retrieval

#### 4. Message Bus (`src/fold_db_core/infrastructure/message_bus/`)

**Modified Files:**
- `events.rs` - Add events for composable ARef operations
- `constructors.rs` - Constructor methods for new event types

### Configuration Changes

#### Field Type Mapping

```rust
impl FieldType {
    /// Returns the constituent types for composable fields
    pub fn composition(&self) -> (AtomRefType, Option<AtomRefType>) {
        match self {
            FieldType::Single => (AtomRefType::Single, None),
            FieldType::Collection => (AtomRefType::Collection, None),
            FieldType::Range => (AtomRefType::Range, None),
            FieldType::Hash => (AtomRefType::Hash, None),
            FieldType::RangeCollection => (AtomRefType::Range, Some(AtomRefType::Collection)),
            FieldType::RangeHash => (AtomRefType::Range, Some(AtomRefType::Hash)),
            FieldType::HashCollection => (AtomRefType::Hash, Some(AtomRefType::Collection)),
            FieldType::HashRange => (AtomRefType::Hash, Some(AtomRefType::Range)),
            FieldType::CollectionRange => (AtomRefType::Collection, Some(AtomRefType::Range)),
            FieldType::CollectionHash => (AtomRefType::Collection, Some(AtomRefType::Hash)),
        }
    }
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1-2)

#### Step 1.1: Create New AtomRef Types
- Implement `AtomRefHash` with HashMap-based storage
- Create `ComposableAtomRef` enum structure
- Add comprehensive unit tests

#### Step 1.2: Extend Type System
- Add new `FieldType` variants for all 2-layer combinations
- Implement composition validation logic
- Update field type parsing and serialization

#### Step 1.3: Factory Pattern
- Create `AtomRefFactory` for generating ARefs during schema approval
- Implement factory methods for all composable types
- Add factory unit tests

### Phase 2: Schema Purification (Week 3)

#### Step 2.1: Remove ARef References from Schema
- Remove `ref_atom_uuid` field from `JsonSchemaField`
- Update schema serialization/deserialization
- Create migration path for existing schemas

#### Step 2.2: Schema Approval Integration
- Modify schema approval process to create all ARefs
- Implement ARef-to-field mapping during approval
- Add approval validation for composable types

#### Step 2.3: Database Integration
- Update database operations to handle composable ARefs
- Implement storage/retrieval for nested structures
- Add database migration scripts

### Phase 3: Runtime Integration (Week 4)

#### Step 3.1: Field Processing Updates
- Modify field processors to use pre-created ARefs
- Remove runtime ARef creation logic
- Update error handling for missing ARefs

#### Step 3.2: Message Bus Integration
- Add events for composable ARef operations
- Update request/response handlers
- Implement event routing for nested operations

#### Step 3.3: API Updates
- Update HTTP API endpoints to support composable types
- Add validation for composable field operations
- Update API documentation

### Phase 4: Testing & Validation (Week 5)

#### Step 4.1: Integration Tests
- Comprehensive tests for all composable type combinations
- Schema approval and ARef creation workflows
- Field operations with composable ARefs

#### Step 4.2: Performance Testing
- Benchmarks for composable vs. simple ARefs
- Memory usage analysis for nested structures
- Query performance validation

#### Step 4.3: Migration Testing
- Test migration from current to new system
- Validate backward compatibility
- Test error scenarios and recovery

## Migration Strategy

### Backward Compatibility

1. **Existing Schemas:** Continue to work without modification
2. **Legacy ARefs:** Automatically wrapped in `ComposableAtomRef::Single`
3. **API Compatibility:** All existing endpoints remain functional

### Migration Process

```rust
impl SchemaManager {
    /// Migrate existing schema to use composable ARefs
    pub fn migrate_schema_to_composable(&self, schema_name: &str) -> Result<(), SchemaError> {
        // 1. Load existing schema
        // 2. Create ARefs for fields missing them
        // 3. Wrap legacy ARefs in ComposableAtomRef::Single
        // 4. Update schema state
        // 5. Persist changes
    }
}
```

### Migration Timeline

1. **Pre-Migration:** Deploy new code with backward compatibility
2. **Migration Window:** Convert schemas during maintenance window  
3. **Post-Migration:** Remove legacy code paths after validation

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_collection_composition() {
        let factory = AtomRefFactory::new();
        let aref = factory.create_composable(
            AtomRefType::Range, 
            Some(AtomRefType::Collection)
        ).unwrap();
        
        assert!(matches!(aref, ComposableAtomRef::Composed { .. }));
    }

    #[test]
    fn test_invalid_triple_composition() {
        let factory = AtomRefFactory::new();
        let result = factory.create_composable(
            AtomRefType::Range, 
            Some(AtomRefType::Collection),
            Some(AtomRefType::Hash) // ❌ Third layer not allowed
        );
        
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_composable_field_operations() {
    // 1. Create schema with Range:Collection field
    // 2. Approve schema (should create ARefs)
    // 3. Perform field operations
    // 4. Validate data integrity
}
```

### Performance Tests

```rust
#[bench]
fn bench_composable_vs_simple_lookup(b: &mut Bencher) {
    // Compare lookup performance between:
    // - ComposableAtomRef::Composed
    // - ComposableAtomRef::Single
}
```

## Risk Assessment

### High Risk
- **Schema Migration Complexity:** Risk of data loss during migration
- **Performance Impact:** Nested structures may impact query performance
- **Backward Compatibility:** Breaking changes to existing functionality

### Medium Risk  
- **Memory Usage:** Increased memory consumption for nested structures
- **Code Complexity:** More complex type system increases maintenance burden
- **Testing Coverage:** Ensuring all composition combinations are tested

### Low Risk
- **Type Safety:** Rust's type system provides compile-time guarantees
- **Factory Pattern:** Centralized ARef creation reduces inconsistency

### Mitigation Strategies

1. **Comprehensive Testing:** Extensive unit, integration, and performance tests
2. **Gradual Rollout:** Phase-based implementation with validation at each step
3. **Monitoring:** Runtime monitoring for performance and memory usage
4. **Rollback Plan:** Ability to revert to previous system if issues arise

## Success Criteria

### Functional Success
- [ ] All 6 composable type combinations working correctly
- [ ] Schema definitions pure (no `aref_uuid` references)
- [ ] ARefs created during schema approval
- [ ] Backward compatibility maintained
- [ ] Hash type implementation complete

### Performance Success
- [ ] <10% performance degradation for simple operations
- [ ] <20% memory increase for composable types
- [ ] Query performance within acceptable limits

### Quality Success
- [ ] >95% test coverage for new code
- [ ] All existing tests passing
- [ ] Documentation updated
- [ ] Migration path validated

## Conclusion

The composable AtomRef type system represents a significant enhancement to Datafold's data modeling capabilities. By supporting hierarchical data structures while maintaining schema purity and ensuring early ARef creation, this design provides the flexibility needed for complex data relationships while preserving system integrity and performance.

The phased implementation approach minimizes risk while ensuring thorough testing and validation at each step. The resulting system will be more expressive, maintainable, and aligned with best practices for schema-driven data systems.

---

**Document Version:** 1.0  
**Last Updated:** 2024-12-19  
**Author:** AI Assistant  
**Status:** Draft - Awaiting Review 