# PBI-3: Composable AtomRef Framework Implementation

## Overview

This PBI implements the core composable AtomRef framework that enables 2-layer composition of AtomRef types, providing type-safe hierarchical data structures while maintaining performance and backward compatibility.

[View in Backlog](../backlog.md#user-content-3)

## Problem Statement

The current AtomRef system only supports individual types (Single, Range, Collection, Hash) without composition capabilities. This limits data modeling flexibility for complex hierarchical structures. We need a composable framework that allows 2-layer combinations like Range:Collection or Hash:Range while maintaining type safety and preventing invalid compositions.

## User Stories

### Primary User Story
As a developer, I want a composable AtomRef framework implemented, so that I can create 2-layer AtomRef composition with type safety.

### Supporting User Stories
- As a developer, I want type-safe composition, so that invalid combinations are prevented at compile time
- As a developer, I want a factory pattern for creating composable ARefs, so that creation is consistent and validated
- As a developer, I want composition validation, so that invalid combinations are rejected with clear error messages
- As a developer, I want backward compatibility, so that existing single-layer ARefs continue to work

## Technical Approach

### Composable AtomRef Framework

#### Core Composable Types
```rust
/// Base AtomRef container types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomRefContainer {
    Single(AtomRef),
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
        /// Metadata about the element type configuration
        element_config: ElementConfig,
    },
}

/// AtomRef type discriminator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AtomRefType {
    Single,
    Range,
    Hash, 
    Collection,
}

/// Configuration for composed element types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConfig {
    /// Default configuration for creating element ARefs
    pub default_source_pub_key: String,
    /// Additional metadata for element creation
    pub metadata: HashMap<String, String>,
}

impl ComposableAtomRef {
    /// Get the UUID of this composable ARef
    pub fn uuid(&self) -> &str {
        match self {
            ComposableAtomRef::Single(container) => container.uuid(),
            ComposableAtomRef::Composed { container, .. } => container.uuid(),
        }
    }
    
    /// Get the composition of this ARef (container, element)
    pub fn composition(&self) -> (AtomRefType, Option<AtomRefType>) {
        match self {
            ComposableAtomRef::Single(container) => (container.aref_type(), None),
            ComposableAtomRef::Composed { container, element_type, .. } => {
                (container.aref_type(), Some(*element_type))
            }
        }
    }
    
    /// Check if this is a composed ARef
    pub fn is_composed(&self) -> bool {
        matches!(self, ComposableAtomRef::Composed { .. })
    }
}

impl AtomRefContainer {
    /// Get the type of this container
    pub fn aref_type(&self) -> AtomRefType {
        match self {
            AtomRefContainer::Single(_) => AtomRefType::Single,
            AtomRefContainer::Range(_) => AtomRefType::Range,
            AtomRefContainer::Hash(_) => AtomRefType::Hash,
            AtomRefContainer::Collection(_) => AtomRefType::Collection,
        }
    }
    
    /// Get the UUID of this container
    pub fn uuid(&self) -> &str {
        match self {
            AtomRefContainer::Single(aref) => aref.uuid(),
            AtomRefContainer::Range(aref) => aref.uuid(),
            AtomRefContainer::Hash(aref) => aref.uuid(),
            AtomRefContainer::Collection(aref) => aref.uuid(),
        }
    }
}
```

#### AtomRef Factory with Composition Validation
```rust
/// Factory for creating composable AtomRefs with validation
pub struct AtomRefFactory {
    /// Optional database operations for persistence
    db_ops: Option<Arc<DbOperations>>,
}

impl AtomRefFactory {
    pub fn new() -> Self {
        Self { db_ops: None }
    }
    
    pub fn with_db_ops(db_ops: Arc<DbOperations>) -> Self {
        Self { db_ops: Some(db_ops) }
    }
    
    /// Create a single-layer AtomRef
    pub fn create_single(
        &self, 
        aref_type: AtomRefType, 
        source_pub_key: String
    ) -> Result<ComposableAtomRef, FactoryError> {
        let container = match aref_type {
            AtomRefType::Single => {
                AtomRefContainer::Single(AtomRef::new(
                    Uuid::new_v4().to_string(), 
                    source_pub_key
                ))
            }
            AtomRefType::Range => {
                AtomRefContainer::Range(AtomRefRange::new(source_pub_key))
            }
            AtomRefType::Hash => {
                AtomRefContainer::Hash(AtomRefHash::new(source_pub_key))
            }
            AtomRefType::Collection => {
                AtomRefContainer::Collection(AtomRefCollection::new(source_pub_key))
            }
        };
        
        Ok(ComposableAtomRef::Single(container))
    }
    
    /// Create a composable AtomRef with validation
    pub fn create_composable(
        &self, 
        container_type: AtomRefType, 
        element_type: Option<AtomRefType>,
        source_pub_key: String
    ) -> Result<ComposableAtomRef, FactoryError> {
        match element_type {
            None => self.create_single(container_type, source_pub_key),
            Some(element) => {
                // Validate composition
                Self::validate_composition(&container_type, &element)?;
                
                // Create container
                let container = self.create_container(container_type, source_pub_key.clone())?;
                
                // Create element config
                let element_config = ElementConfig {
                    default_source_pub_key: source_pub_key,
                    metadata: HashMap::new(),
                };
                
                Ok(ComposableAtomRef::Composed {
                    container,
                    element_type: element,
                    element_config,
                })
            }
        }
    }
    
    /// Validate composition rules
    fn validate_composition(
        container: &AtomRefType, 
        element: &AtomRefType
    ) -> Result<(), FactoryError> {
        match (container, element) {
            // Valid combinations
            (AtomRefType::Range, AtomRefType::Collection) |
            (AtomRefType::Range, AtomRefType::Hash) |
            (AtomRefType::Hash, AtomRefType::Collection) |
            (AtomRefType::Hash, AtomRefType::Range) |
            (AtomRefType::Collection, AtomRefType::Range) |
            (AtomRefType::Collection, AtomRefType::Hash) => Ok(()),
            
            // Single cannot be an element in composition
            (_, AtomRefType::Single) => Err(FactoryError::InvalidComposition { 
                container: *container, 
                element: *element 
            }),
            
            // Single cannot be a container for composition
            (AtomRefType::Single, _) => Err(FactoryError::InvalidComposition { 
                container: *container, 
                element: *element 
            }),
            
            // Self-composition not allowed
            (container_type, element_type) if container_type == element_type => {
                Err(FactoryError::SelfComposition { 
                    aref_type: *container_type 
                })
            }
        }
    }
    
    /// Create a container ARef
    fn create_container(
        &self, 
        container_type: AtomRefType, 
        source_pub_key: String
    ) -> Result<AtomRefContainer, FactoryError> {
        match container_type {
            AtomRefType::Single => {
                Ok(AtomRefContainer::Single(AtomRef::new(
                    Uuid::new_v4().to_string(), 
                    source_pub_key
                )))
            }
            AtomRefType::Range => {
                Ok(AtomRefContainer::Range(AtomRefRange::new(source_pub_key)))
            }
            AtomRefType::Hash => {
                Ok(AtomRefContainer::Hash(AtomRefHash::new(source_pub_key)))
            }
            AtomRefType::Collection => {
                Ok(AtomRefContainer::Collection(AtomRefCollection::new(source_pub_key)))
            }
        }
    }
    
    /// Validate an existing composable ARef
    pub fn validate_composable_aref(
        &self, 
        aref: &ComposableAtomRef
    ) -> Result<(), FactoryError> {
        match aref {
            ComposableAtomRef::Single(_) => Ok(()),
            ComposableAtomRef::Composed { container, element_type, .. } => {
                Self::validate_composition(&container.aref_type(), element_type)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("Invalid composition: {container:?} cannot contain {element:?}")]
    InvalidComposition { container: AtomRefType, element: AtomRefType },
    #[error("Self-composition not allowed: {aref_type:?} cannot contain itself")]
    SelfComposition { aref_type: AtomRefType },
    #[error("Triple nesting not allowed: maximum 2 layers supported")]
    TripleNestingNotAllowed,
    #[error("Unknown AtomRef type: {type_name}")]
    UnknownType { type_name: String },
    #[error("Factory creation failed: {reason}")]
    CreationFailed { reason: String },
}
```

### Implementation Plan

#### Phase 1: Core Types (Days 1-2)
1. Implement `ComposableAtomRef` enum with Single and Composed variants
2. Create `AtomRefContainer` enum supporting all basic types
3. Add `AtomRefType` discriminator enum
4. Implement basic composition analysis methods

#### Phase 2: Factory Implementation (Days 3-4)
1. Implement `AtomRefFactory` with creation methods
2. Add comprehensive composition validation logic
3. Implement error handling with `FactoryError`
4. Add factory unit tests with all valid/invalid combinations

#### Phase 3: Integration and Validation (Days 5)
1. Integration tests with existing AtomRef types
2. Serialization/deserialization testing
3. Performance validation for composable operations
4. Documentation and examples

## UX/UI Considerations

### Developer Experience
- Clear error messages for invalid compositions
- Type-safe APIs that prevent runtime errors
- Intuitive factory methods for common use cases

### API Consistency
- Consistent method signatures across all composable types
- Uniform error handling for composition validation
- Clear naming conventions that indicate composition

## Acceptance Criteria

1. **ComposableAtomRef Framework**
   - [ ] ComposableAtomRef enum with Single and Composed variants
   - [ ] AtomRefContainer enum supporting all basic types
   - [ ] AtomRefType discriminator enum
   - [ ] Composition analysis methods working correctly

2. **AtomRef Factory**
   - [ ] Factory creates single-layer ARefs correctly
   - [ ] Factory creates composable ARefs with validation
   - [ ] 2-layer composition limit enforced
   - [ ] Invalid combinations rejected with clear error messages
   - [ ] All 6 valid compositions supported (RangeCollection, RangeHash, HashCollection, HashRange, CollectionRange, CollectionHash)

3. **Composition Validation**
   - [ ] Valid composition combinations accepted
   - [ ] Invalid combinations rejected (Single as element, self-composition, etc.)
   - [ ] Clear error messages for validation failures
   - [ ] Composition rules documented and tested

4. **Testing**
   - [ ] Unit tests for all factory methods
   - [ ] Validation tests for all valid and invalid combinations
   - [ ] Serialization tests for all composable types
   - [ ] Integration tests with existing AtomRef infrastructure

5. **Error Handling**
   - [ ] FactoryError covers all failure scenarios
   - [ ] Clear error messages for each error type
   - [ ] Graceful handling of invalid inputs
   - [ ] Error propagation working correctly

6. **Integration**
   - [ ] All composable types implement required traits
   - [ ] Backward compatibility with existing single-layer ARefs
   - [ ] No breaking changes to current AtomRef interfaces
   - [ ] Ready for use in field type system

## Dependencies

- **Prerequisite**: PBI-2 (HashAtomRef Type with Database Operations)
- **Internal**: Current AtomRef types (AtomRef, AtomRefRange, AtomRefCollection), AtomRefHash
- **External**: serde, uuid crates
- **Testing**: Existing test infrastructure

## Open Questions

1. **Element Creation**: Should elements be created lazily or eagerly during composition?
2. **Performance Impact**: How do composed ARefs affect memory usage and access performance?
3. **Serialization**: What's the optimal serialization format for composed ARefs?
4. **Migration**: How do we migrate existing single-layer ARefs to composable format when needed?

## Related Tasks

This PBI will generate detailed tasks covering:
- ComposableAtomRef enum and variant implementation
- AtomRefContainer enum and type discrimination
- AtomRefFactory implementation with validation
- Composition validation logic and error handling
- Integration testing with existing AtomRef infrastructure 